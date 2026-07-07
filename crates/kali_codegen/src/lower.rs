//! Program-level driver and LIR-walking analysis functions.
use crate::*;

/// Maps a representation decision to the wasm value type used for the matching
/// param/result/local slot. `I64` is the integer default; `F64` is an IEEE double.
pub(crate) fn wasm_type(repr: kali_common::Repr) -> wasm_encoder::ValType {
    match repr {
        kali_common::Repr::F64 => wasm_encoder::ValType::F64,
        kali_common::Repr::I64 | kali_common::Repr::Object(_) | kali_common::Repr::String => {
            wasm_encoder::ValType::I64
        }
    }
}

pub(crate) fn generator_lowering_unavailable_message(
    function_plans: &[FunctionPlan],
) -> &'static str {
    let has_generator = function_plans
        .iter()
        .any(|plan| matches!(plan.flavor, Some(FunctionFlavor::Generator)));
    let has_async_generator = function_plans
        .iter()
        .any(|plan| matches!(plan.flavor, Some(FunctionFlavor::AsyncGenerator)));

    kali_common::generator_function_lowering_unavailable_message_for_flavors(
        has_generator,
        has_async_generator,
    )
}

/// Names of every hand-emitted synthetic wasm function (not lowered from
/// source LIR): the page-pool allocator family, plus the runtime-substring
/// helper (Spec 2). Used to exclude them from coverage instrumentation (see
/// the `kali:coverage` custom-section count below) and, in later tasks,
/// anywhere else code needs to distinguish a real source-defined function
/// from these fixed compiler-internal slots.
pub const SYNTHETIC_FUNCTIONS: &[&str] = &[
    "__alloc",
    "__alloc_global",
    "__page_get",
    "__arena_reset",
    "__substring",
    "__join",
];

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, lir: &LirProgram) -> CodegenResult {
    let mut diagnostics = Vec::new();
    let function_plans = collect_functions(lir, &ctx.repr_table, &ctx.arena_table);
    // Module-scope mutable SCALAR (`var`/`let` numeric) bindings that are read
    // or written from inside a function are promoted to persistent mutable WASM
    // globals (indices AFTER the reserved arena globals). A module-only scalar
    // stays a `_start` local (byte-identical); heap types (object/array/string)
    // are NEVER promoted — a mutable global heap root is a persistent GC root
    // the region reclamation does not model, so those stay fail-closed (E5506).
    let module_global_slots = collect_module_scalar_globals(lir, &ctx.repr_table, &function_plans);
    if function_plans.iter().any(|plan| {
        matches!(
            plan.flavor,
            Some(FunctionFlavor::Generator | FunctionFlavor::AsyncGenerator)
        )
    }) {
        diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            generator_lowering_unavailable_message(&function_plans),
        ));
        return CodegenResult {
            wasm_bytes: Vec::new(),
            diagnostics,
        };
    }
    let mut function_name_to_index = BTreeMap::new();
    let mut string_pool = StringPool::new(crate::ENV_GET_BUFFER_RESERVED);
    let uses_env_get = program_uses_env_get(lir);
    let uses_env_has = program_uses_env_has(lir);
    let uses_env_set = program_uses_env_set(lir);
    let uses_env_delete = program_uses_env_delete(lir);
    let uses_cwd_set = program_uses_cwd_set(lir);
    let uses_process_exit = program_uses_process_exit(lir);
    let uses_stdout_write_bytes = program_uses_stdout_write_bytes(lir);
    let uses_env_access = uses_env_get || uses_env_has || uses_env_set || uses_env_delete;
    let function_index_offset = crate::FUNCTION_INDEX_OFFSET
        + if ctx.target.coverage { 1 } else { 0 }
        + if uses_env_set { 1 } else { 0 }
        + if uses_env_delete { 1 } else { 0 }
        + if uses_env_get { 1 } else { 0 }
        + if uses_env_has { 1 } else { 0 }
        + if uses_cwd_set { 1 } else { 0 }
        + if uses_process_exit { 1 } else { 0 }
        + if uses_stdout_write_bytes { 1 } else { 0 };
    let env_get_type_index = if uses_env_access { Some(6) } else { None };
    let env_has_type_index = if uses_env_has { Some(7) } else { None };
    let cwd_set_type_index = if uses_cwd_set { Some(5) } else { None };
    let env_set_import_index = if uses_env_set {
        Some(crate::COVERAGE_HIT_IMPORT_INDEX + if ctx.target.coverage { 1 } else { 0 })
    } else {
        None
    };
    let env_delete_import_index = if uses_env_delete {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 },
        )
    } else {
        None
    };
    let env_get_import_index = if uses_env_get {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 },
        )
    } else {
        None
    };
    let env_has_import_index = if uses_env_has {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 },
        )
    } else {
        None
    };
    let cwd_set_import_index = if uses_cwd_set {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 },
        )
    } else {
        None
    };
    let process_exit_import_index = if uses_process_exit {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 },
        )
    } else {
        None
    };
    // `stdout_write_bytes` is appended after every other conditional import (see
    // the `import_section.import(...)` block below), so its index sums ALL
    // preceding conditional-import flags in the same order they are declared
    // there: coverage, env_set, env_delete, env_get, env_has, cwd_set,
    // process_exit.
    let stdout_write_bytes_import_index = if uses_stdout_write_bytes {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 },
        )
    } else {
        None
    };

    // Keep the emitted order deterministic: imported registration hook first, synthetic entry
    // second, then named functions in source order.
    let mut start_locals = collect_function_locals(
        &lir.nodes,
        lir.root,
        &ctx.repr_table,
        &ctx.arena_table,
        "_start",
    );
    // A promoted module scalar lives in its persistent global, not a `_start`
    // local slot — its declarator init stores through `GlobalSet` (see the
    // module-global declarator branch in `emit/control_flow.rs`).
    start_locals.retain(|name| !module_global_slots.contains_key(name));
    let mut all_functions = vec![FunctionPlan {
        name: "_start".to_string(),
        params: Vec::new(),
        locals: start_locals,
        body: lir.root,
        result: false,
        is_entry: true,
        flavor: None,
    }];
    // Synthetic bump allocator `__alloc(size: i32) -> i32`, occupying a fixed
    // slot right after `_start` and before any named (source-defined)
    // function. Its body (and its three siblings' below) is hand-emitted by
    // `emit_bump_body` / `emit_page_get_body` / `emit_arena_reset_body`, not
    // lowered from LIR — `body` is unused (set to `lir.root` as an inert
    // placeholder) and `locals`/`flavor` are left at their inert defaults.
    // Object/array allocation sites resolve its index through
    // `function_name_to_index["__alloc"]` (see `FunctionEmitter::alloc_fn_index`)
    // exactly like any other named function, so inserting these four here
    // shifts every later function's index by exactly four — safe because
    // every call site in this crate resolves callee indices through that
    // same map (verified: the only hardcoded `Instruction::Call(..)` sites
    // are fixed *import* indices, which live in a separate index space
    // unaffected by `all_functions`).
    all_functions.push(FunctionPlan {
        name: "__alloc".to_string(),
        params: vec!["size".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Three more synthetic slots, same inert-placeholder pattern as `__alloc`
    // above (hand-emitted bodies, no real LIR, `body`/`locals`/`flavor` all
    // inert): `__alloc_global` is `__alloc`'s twin against the separate
    // "global" arena trio (used for host-runtime-allocated strings, which
    // must outlive any `__arena_reset`); `__page_get` is the shared page
    // supplier both bump allocators fall back to; `__arena_reset` recycles a
    // function/loop arena's page list onto the shared free list. See
    // `emit_bump_body` / `emit_page_get_body` / `emit_arena_reset_body`
    // below for their bodies.
    all_functions.push(FunctionPlan {
        name: "__alloc_global".to_string(),
        params: vec!["size".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__page_get".to_string(),
        params: vec!["pages".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__arena_reset".to_string(),
        params: Vec::new(),
        locals: Vec::new(),
        body: lir.root,
        result: false,
        is_entry: false,
        flavor: None,
    });
    // Synthetic runtime-substring `__substring(h: i64, s: i64, e: i64) -> i64`:
    // pure-ALU zero-copy slice re-tag over a tagged string handle (Spec 2).
    // Same inert-placeholder pattern as the four allocator synthetics above;
    // body hand-emitted by `emit_substring_body`. Pass `e = i64::MAX` for the
    // "to end of string" 0/1-arg forms — the clamp folds it to `len`.
    all_functions.push(FunctionPlan {
        name: "__substring".to_string(),
        params: vec!["h".to_string(), "s".to_string(), "e".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Synthetic runtime-join `__join(arr: i64, sep: i64) -> i64` (Spec 3):
    // two-pass copy of an all-string-element array into ONE fresh
    // __alloc_global string — sum lengths, allocate, memory.copy each
    // element and separator. NEVER __alloc: runtime strings must not
    // dangle across an arena reset (escape_flow relies on it).
    all_functions.push(FunctionPlan {
        name: "__join".to_string(),
        params: vec!["arr".to_string(), "sep".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.extend(function_plans);

    for (idx, function) in all_functions.iter().enumerate() {
        function_name_to_index.insert(function.name.clone(), idx as u32 + function_index_offset);
    }

    let mut type_section = TypeSection::new();
    type_section.ty().function(vec![ValType::I32], Vec::new());
    type_section.ty().function(vec![ValType::I64], Vec::new());
    type_section.ty().function(Vec::new(), vec![ValType::I32]);
    type_section
        .ty()
        .function(vec![ValType::I64, ValType::I64], vec![ValType::I64]);
    type_section
        .ty()
        .function(vec![ValType::I64], vec![ValType::I64]);
    type_section
        .ty()
        .function(vec![ValType::I64], vec![ValType::I32]);
    type_section.ty().function(
        vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        vec![ValType::I32],
    );
    type_section
        .ty()
        .function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
    // Type 8: float_to_fixed `(f64, i32) -> i64` (value, digit count -> string handle).
    type_section
        .ty()
        .function(vec![ValType::F64, ValType::I32], vec![ValType::I64]);
    // Type 9: float_to_string `(f64) -> i64` (value -> string handle, JS
    // `String(number)` semantics).
    type_section
        .ty()
        .function(vec![ValType::F64], vec![ValType::I64]);
    let mut import_section = ImportSection::new();
    import_section.import("kali:rt", "test_register", EntityType::Function(0));
    import_section.import("kali:rt", "console_log", EntityType::Function(1));
    import_section.import("kali:rt", "console_error", EntityType::Function(1));
    import_section.import("kali:rt", "console_warn", EntityType::Function(1));
    import_section.import("kali:rt", "console_info", EntityType::Function(1));
    import_section.import("kali:rt", "console_debug", EntityType::Function(1));
    import_section.import("kali:rt", "args_len", EntityType::Function(2));
    import_section.import("kali:rt", "math_max", EntityType::Function(3));
    import_section.import("kali:rt", "math_min", EntityType::Function(3));
    import_section.import("kali:rt", "math_abs", EntityType::Function(4));
    import_section.import("kali:rt", "math_sign", EntityType::Function(4));
    import_section.import("kali:rt", "math_imul", EntityType::Function(3));
    import_section.import("kali:rt", "math_round", EntityType::Function(4));
    import_section.import("kali:rt", "process_pid", EntityType::Function(2));
    import_section.import("kali:rt", "cwd", EntityType::Function(6));
    import_section.import("kali:rt", "math_clz32", EntityType::Function(4));
    import_section.import("kali:rt", "math_pow", EntityType::Function(3));
    // Four unconditional runtime helpers occupy fixed import indices 17 through 20
    // (see INT_TO_STRING_IMPORT_INDEX / STRING_CONCAT_IMPORT_INDEX /
    // FLOAT_TO_FIXED_IMPORT_INDEX / FLOAT_TO_STRING_IMPORT_INDEX). They are registered
    // here, before the conditional coverage/env/process imports, so the relative
    // bookkeeping below (all expressed against COVERAGE_HIT_IMPORT_INDEX = 21) stays
    // consistent. int_to_string is (i64) -> i64 (type 4); string_concat is
    // (i64, i64) -> i64 (type 3); float_to_fixed is (f64, i32) -> i64 (type 8);
    // float_to_string is (f64) -> i64 (type 9).
    import_section.import("kali:rt", "int_to_string", EntityType::Function(4));
    import_section.import("kali:rt", "string_concat", EntityType::Function(3));
    import_section.import("kali:rt", "float_to_fixed", EntityType::Function(8));
    import_section.import("kali:rt", "float_to_string", EntityType::Function(9));
    if ctx.target.coverage {
        import_section.import("kali:rt", "coverage_hit", EntityType::Function(0));
    }
    if env_set_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_set",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_delete_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_delete",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_get_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_get",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_has_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_has",
            EntityType::Function(env_has_type_index.unwrap()),
        );
    }
    if cwd_set_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "cwd_set",
            EntityType::Function(cwd_set_type_index.unwrap()),
        );
    }
    if process_exit_import_index.is_some() {
        import_section.import("kali:rt", "process_exit", EntityType::Function(1));
    }
    if stdout_write_bytes_import_index.is_some() {
        // `(i64) -> ()`: takes the byte-array's linear-memory handle, writes it
        // to stdout, and returns no value.
        import_section.import("kali:rt", "stdout_write_bytes", EntityType::Function(1));
    }
    // Function signatures are repr-directed: each param/result ValType comes from
    // the repr table (defaulting to I64). Two functions with equal arity but
    // differing float shapes need distinct wasm types, so the dedup key is the
    // full (params, results) ValType signature rather than (arity, has_result).
    // For an all-I64 (integer) program this collapses to the same signatures as
    // before, keeping the emitted type section byte-identical.
    let mut function_types = BTreeMap::<(Vec<ValType>, Vec<ValType>), u32>::new();
    let mut type_for_function = Vec::with_capacity(all_functions.len());

    for function in &all_functions {
        // The four allocator synthetic page-pool functions (plus
        // `__substring` and `__join`) are not repr-directed user functions
        // (they have no `ReprTable` entries at all), so their signatures are
        // fixed here rather than derived from
        // `ctx.repr_table.param`/`return_repr`, which would otherwise
        // default them to `(i64) -> i64`.
        // `__alloc`/`__alloc_global`/`__page_get` all share the one
        // `(i32) -> i32` signature (deduped below to the same type index);
        // `__arena_reset` is `() -> ()`, which `_start` (also no params, no
        // result) already registers as a type, so it reuses that entry too
        // rather than adding a new one. `__substring` is `(i64, i64, i64) ->
        // i64`, deduped below same as any other function with that shape
        // (e.g. a real 3-arg all-integer function, if one exists in the
        // program) rather than a new type per module. `__join` is
        // `(i64, i64) -> i64` (Spec 3), same dedup treatment.
        let (params, results) = if matches!(
            function.name.as_str(),
            "__alloc" | "__alloc_global" | "__page_get"
        ) {
            (vec![ValType::I32], vec![ValType::I32])
        } else if function.name == "__substring" {
            (
                vec![ValType::I64, ValType::I64, ValType::I64],
                vec![ValType::I64],
            )
        } else if function.name == "__join" {
            (vec![ValType::I64, ValType::I64], vec![ValType::I64])
        } else if function.name == "__arena_reset" {
            (Vec::new(), Vec::new())
        } else {
            let params: Vec<ValType> = (0..function.params.len())
                .map(|index| wasm_type(ctx.repr_table.param(&function.name, index)))
                .collect();
            let results = if function.result {
                vec![wasm_type(ctx.repr_table.return_repr(&function.name))]
            } else {
                Vec::new()
            };
            (params, results)
        };
        let key = (params.clone(), results.clone());
        let type_index = if let Some(&idx) = function_types.get(&key) {
            idx
        } else {
            let idx = function_types.len() as u32 + 10;
            type_section.ty().function(params, results);
            function_types.insert(key, idx);
            idx
        };
        type_for_function.push(type_index);
    }

    let mut function_section = FunctionSection::new();
    for type_index in &type_for_function {
        function_section.function(*type_index);
    }

    let mut memory_section = MemorySection::new();
    memory_section.memory(MemoryType {
        minimum: 16,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });

    let mut export_section = ExportSection::new();
    export_section.export("memory", ExportKind::Memory, 0);
    export_section.export("__heap", ExportKind::Global, 0);
    for function in &all_functions {
        if function.is_entry {
            export_section.export("_start", ExportKind::Func, function_name_to_index["_start"]);
        } else {
            let index = function_name_to_index[&function.name];
            export_section.export(&function.name, ExportKind::Func, index);
            // `Kali.test(name, callback)` registers its callback with the host via
            // `test_register(callback_index)` (see `kali_test_callback_index` /
            // `emit_call` in `emit/call.rs`), where `callback_index` is this same
            // raw wasm function index. Both consumers of that registration —
            // `kali_runtime::host::enforce::invoke_callback` (native wasmtime test
            // runner) and the browser-harness JS scripts in
            // `kali_runtime::browser::harness` — look the callback up by the export
            // name `__kali_callback_<index>`, not by the function's own declared or
            // synthetic name. Alias every non-entry function under that name too so
            // any function reachable as a Kali.test callback resolves correctly;
            // this previously went unexercised because arrow-shaped callbacks were
            // never compiled as real functions before this change.
            //
            // The four synthetic page-pool functions are non-entry functions
            // too, so each picks up this same `__kali_callback_<index>` alias
            // export in addition to its own name export (which is how
            // `__alloc_global` gets exported as `"__alloc_global"` per the
            // host/browser-glue contract, with no special-cased export call
            // needed here). Harmless: nothing ever calls `test_register` with
            // a synthetic function's index, so no host ever looks it up under
            // the alias as a Kali.test callback.
            export_section.export(&format!("__kali_callback_{index}"), ExportKind::Func, index);
        }
    }

    // Module-scope binding tables: `const name → init node` for compile-time
    // inlining inside functions, plus ALL top-level binding names so
    // non-inlinable reads can be gated instead of silently lowering through
    // the zero placeholder (see emit/control_flow.rs identifier fallback).
    let mut module_const_inits: BTreeMap<String, LirNodeId> = BTreeMap::new();
    let mut module_binding_names: BTreeSet<String> = BTreeSet::new();
    {
        let start = all_functions
            .iter()
            .find(|function| function.name == "_start");
        if let Some(start) = start {
            // `_start`'s LIR entry point is its `body` field (there is no
            // `root` field on `FunctionPlan`); `_start`'s own body IS `lir.root`
            // (see the `all_functions.push(FunctionPlan { ... body: lir.root, ... })`
            // above), so this walks the whole top-level program.
            let mut stack = vec![start.body];
            while let Some(id) = stack.pop() {
                let node = &lir.nodes[id.0 as usize];
                match node.kind {
                    LirNodeKind::Program | LirNodeKind::Block => {
                        stack.extend(node.children.iter().copied());
                    }
                    LirNodeKind::Instruction
                        if matches!(node.text.as_deref(), Some("const" | "let" | "var")) =>
                    {
                        let is_const = node.text.as_deref() == Some("const");
                        for declarator_id in &node.children {
                            let declarator = &lir.nodes[declarator_id.0 as usize];
                            let Some(name) = declarator.text.clone() else {
                                continue;
                            };
                            module_binding_names.insert(name.clone());
                            if is_const && declarator.children.len() >= 2 {
                                module_const_inits.insert(name, declarator.children[1]);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut code_section = CodeSection::new();
    for (coverage_id, function) in all_functions.iter().enumerate() {
        // Two extra i64 scratch locals: `self.locals.len()` is the general-purpose
        // scratch used throughout codegen (temp locals, tee targets, etc.), and
        // `self.locals.len() + 1` is a second scratch reserved for array allocation so
        // the size argument can be evaluated exactly once and reused for both the
        // length-header store and the `(n+1)*8` byte-count math (see
        // `emit_array_allocation` in `emit/call.rs`).
        // Per-named-local ValType is repr-directed: an F64 scalar binding gets an
        // f64 slot, everything else (array handles and unrecorded names) defaults
        // to i64. The two trailing scratch locals (general-purpose + array-alloc)
        // always stay i64. Consecutive same-type locals are grouped into runs; for
        // an all-i64 function this yields the single `(len + 2, I64)` run emitted
        // before, keeping the code section byte-identical for integer programs.
        //
        // The four synthetic page-pool functions are hand-emitted (not
        // lowered from LIR, have no `function.locals` names, and are not
        // repr-directed), and each needs its own fixed set of i32 scratch
        // locals instead of the two i64 scratch locals every other function
        // gets:
        //   `__alloc`/`__alloc_global` (`emit_bump_body`): 2 — `cur`, `p`
        //     (locals 1, 2; local/param 0 is `size`).
        //   `__page_get` (`emit_page_get_body`): 4 — `head`, `base`, `need`,
        //     and one grow-loop scratch (locals 1-4; local/param 0 is `n`).
        //     `head`'s slot is reused as the grow path's `cur_pages` (their
        //     lifetimes never overlap: `head` is only live inside the
        //     free-list branch, which always returns before the frontier/grow
        //     path runs), so only ONE further local (`deficit_pages`) is
        //     needed beyond `head`/`base`/`need` — matching the moved
        //     Phase-0 grow logic's 3-temporary shape (`new_top` is not
        //     stored at all; it is cheap enough to recompute as `base + need`
        //     each place it's needed instead).
        //   `__arena_reset` (`emit_arena_reset_body`): 2 — `p`, `next`
        //     (locals 0, 1; no params).
        //   `__substring` (`emit_substring_body`): 1 i64 — the swap temp
        //     (local 3; locals 0-2 are its `h`/`s`/`e` params). `len` is
        //     recomputed from `h` rather than stored, so no further local
        //     is needed.
        //   `__join` (`emit_join_body`, Spec 3): 6 i64 — `n`, `i`, `total`,
        //     `out`, `cur`, `h` (locals 2-7; locals 0-1 are its `arr`/`sep`
        //     params).
        let mut local_decls: Vec<(u32, ValType)> = Vec::new();
        if matches!(function.name.as_str(), "__alloc" | "__alloc_global") {
            local_decls.push((2, ValType::I32));
        } else if function.name == "__page_get" {
            local_decls.push((4, ValType::I32));
        } else if function.name == "__arena_reset" {
            local_decls.push((2, ValType::I32));
        } else if function.name == "__substring" {
            local_decls.push((1, ValType::I64));
        } else if function.name == "__join" {
            local_decls.push((6, ValType::I64));
        } else {
            for local_name in &function.locals {
                // A `__arena_save_*` local (Step 2 of loop-arena provisioning)
                // holds a saved copy of an i32 global (`g1`/`g2`/`g3`) and has
                // no `ReprTable` entry of its own; `scalar()`'s default
                // (`Repr::I64`) would mistype the slot and fail wasm
                // validation the first time `GlobalGet(1..3)` is stored into
                // it, so it is forced to i32 here ahead of the repr lookup.
                let val_type = if is_arena_save_local_name(local_name) {
                    ValType::I32
                } else {
                    wasm_type(ctx.repr_table.scalar(&function.name, local_name))
                };
                match local_decls.last_mut() {
                    Some((count, last_type)) if *last_type == val_type => *count += 1,
                    _ => local_decls.push((1, val_type)),
                }
            }
            match local_decls.last_mut() {
                Some((count, ValType::I64)) => *count += 2,
                _ => local_decls.push((2, ValType::I64)),
            }
        }
        let mut body = Function::new(local_decls);
        let mut emitter = FunctionEmitter::new(
            lir,
            &function_name_to_index,
            env_set_import_index,
            env_delete_import_index,
            env_get_import_index,
            env_has_import_index,
            cwd_set_import_index,
            process_exit_import_index,
            stdout_write_bytes_import_index,
            &mut diagnostics,
            &mut string_pool,
            ctx.source_path.clone(),
            function.flavor,
            &function.params,
            &function.locals,
            &ctx.repr_table,
            &ctx.arena_table,
            &function.name,
            function.body,
            &module_const_inits,
            &module_binding_names,
            &module_global_slots,
        );
        let coverage_id = ctx.target.coverage.then_some(coverage_id as u32);
        if SYNTHETIC_FUNCTIONS.contains(&function.name.as_str()) {
            // Hand-emitted: not lowered from LIR (there is no source-level
            // function body for these synthetic page-pool functions), and
            // deliberately uninstrumented (no `emit_coverage_hit`) since none
            // is a source-defined function.
            let page_get_index = function_name_to_index["__page_get"];
            let alloc_global_index = function_name_to_index["__alloc_global"];
            match function.name.as_str() {
                "__alloc" => emit_bump_body(&mut body, 1, 2, 3, page_get_index),
                "__alloc_global" => emit_bump_body(&mut body, 4, 5, 6, page_get_index),
                "__page_get" => emit_page_get_body(&mut body),
                "__arena_reset" => emit_arena_reset_body(&mut body),
                "__substring" => emit_substring_body(&mut body),
                "__join" => emit_join_body(&mut body, alloc_global_index),
                other => unreachable!("unhandled synthetic function {other}"),
            }
        } else if function.is_entry {
            emitter.emit_coverage_hit(&mut body, coverage_id);
            emitter.emit_sequence(&mut body, &top_level_children(lir), false);
        } else {
            emitter.emit_function_body(&mut body, function.body, function.result, coverage_id);
        }
        body.instruction(&Instruction::End);
        code_section.function(&body);
    }

    let mut data_section = DataSection::new();
    for (offset, text) in &string_pool.entries {
        data_section.active(
            0,
            &ConstExpr::i32_const(*offset as i32),
            text.as_bytes().iter().copied(),
        );
    }

    // Heap base: first 8-aligned byte after interned string data. `__heap`
    // (global index 0, g0) now means the *page frontier* — the first byte of
    // linear memory not yet carved into a 64KB page — rather than a flat bump
    // pointer; `__page_get` advances it by whole pages (or `n*PAGE` for a
    // multi-page span). Pages are 64KB *chunks* starting here, not
    // 64KB-*aligned*, so no alignment change to `heap_base` itself is needed.
    let heap_base = (string_pool.next_offset + 7) & !7;
    let mut global_section = GlobalSection::new();
    global_section.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(heap_base as i32),
    );
    // g1..g7: the page-pool's remaining state, all mutable i32 initialized to
    // 0 (an empty boot arena — see the Notes on `emit_bump_body` below for why
    // an all-zero trio correctly takes the slow/fresh-page path on first use).
    // Order fixed by `emit_bump_body`'s/`emit_page_get_body`'s/
    // `emit_arena_reset_body`'s own global-index parameters below:
    //   g1 = current-arena page-list head   (__alloc's trio;   reset by __arena_reset)
    //   g2 = current-arena bump cursor       ("
    //   g3 = current-arena bump limit        ("
    //   g4 = global-arena page-list head    (__alloc_global's trio; never reset)
    //   g5 = global-arena bump cursor        ("
    //   g6 = global-arena bump limit         ("
    //   g7 = free-list head (pages recycled by __arena_reset; consumed by __page_get)
    for _ in 0..7 {
        global_section.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(0),
        );
    }
    // Module-scope mutable scalar globals, appended after g0..g7 at indices
    // 8, 9, … (matching the ascending indices assigned in
    // `collect_module_scalar_globals`, which iterates the same sorted-by-name
    // `BTreeMap`). Each is zero-initialized (`var` hoisting semantics: the
    // binding reads `undefined`/0 until its declarator line runs `GlobalSet` in
    // `_start`); the declared wasm type follows the binding's repr.
    for (_index, repr) in module_global_slots.values() {
        let (val_type, init) = match repr {
            kali_common::Repr::F64 => (ValType::F64, ConstExpr::f64_const(0.0.into())),
            _ => (ValType::I64, ConstExpr::i64_const(0)),
        };
        global_section.global(
            GlobalType {
                val_type,
                mutable: true,
                shared: false,
            },
            &init,
        );
    }

    let mut module = Module::new();
    module.section(&type_section);
    module.section(&import_section);
    module.section(&function_section);
    module.section(&memory_section);
    // Globals come after memory and before exports per the wasm section ordering.
    module.section(&global_section);
    module.section(&export_section);
    module.section(&code_section);
    if ctx.target.coverage {
        // Exclude synthetic, uninstrumented functions (the page-pool family
        // in `SYNTHETIC_FUNCTIONS`; see their hand-emitted bodies above,
        // which deliberately have no `emit_coverage_hit`) from the
        // denominator. Counting them here would make 100% coverage
        // structurally unreachable: their `coverage_id` can never appear in
        // `coverage_hits` because nothing ever calls `coverage_hit` on their
        // behalf.
        let instrumented_function_count = all_functions
            .iter()
            .filter(|f| !SYNTHETIC_FUNCTIONS.contains(&f.name.as_str()))
            .count() as u32;
        module.section(&CustomSection {
            name: Cow::Borrowed("kali:coverage"),
            data: Cow::Owned(instrumented_function_count.to_le_bytes().to_vec()),
        });
    }
    if !data_section.is_empty() {
        module.section(&data_section);
    }

    let wasm_bytes = module.finish();

    CodegenResult {
        wasm_bytes,
        diagnostics,
    }
}

pub(crate) fn collect_functions(
    lir: &LirProgram,
    repr_table: &kali_common::ReprTable,
    arena_table: &kali_common::ArenaTable,
) -> Vec<FunctionPlan> {
    let mut plans = Vec::new();
    let mut visited = HashSet::new();
    collect_functions_from_node(
        lir,
        lir.root,
        &mut visited,
        &mut plans,
        repr_table,
        arena_table,
    );
    plans
}

pub(crate) fn program_uses_env_get(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("get") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_env_has(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("has") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn is_process_root(nodes: &[LirNode], id: LirNodeId) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };

    if node.text.as_deref() == Some("process") {
        return true;
    }

    node.text.as_deref() == Some("globalThis")
        && node.children.first().is_some_and(|child| {
            nodes
                .get(child.0 as usize)
                .is_some_and(|process| process.text.as_deref() == Some("process"))
        })
}

pub(crate) fn process_env_property_key(nodes: &[LirNode], id: LirNodeId) -> Option<String> {
    let node = nodes.get(id.0 as usize)?;
    let key = node.text.as_deref()?;
    let object = node.children.first().copied()?;
    let object_node = nodes.get(object.0 as usize)?;
    if object_node.text.as_deref() != Some("env") {
        return None;
    }
    let root = object_node.children.first().copied()?;
    if !is_process_root(nodes, root) {
        return None;
    }

    Some(key.to_string())
}

pub(crate) fn program_uses_env_set(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("=")
            && node.children.len() == 2
            && process_env_property_key(&lir.nodes, node.children[0]).is_some()
        {
            return true;
        }

        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("set") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_env_delete(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("delete")
            && node.children.len() == 1
            && process_env_property_key(&lir.nodes, node.children[0]).is_some()
        {
            return true;
        }

        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("delete") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_cwd_set(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("chdir") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };

        object_node.text.as_deref() == Some("Deno")
            || (object_node.text.as_deref() == Some("globalThis")
                && object_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_process_exit(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("exit") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };

        object_node.text.as_deref() == Some("process")
            || object_node.text.as_deref() == Some("Deno")
            || (object_node.text.as_deref() == Some("globalThis")
                && object_node.children.first().is_some_and(|child| {
                    lir.nodes.get(child.0 as usize).is_some_and(|host| {
                        matches!(host.text.as_deref(), Some("process") | Some("Deno"))
                    })
                }))
    })
}

pub(crate) fn program_uses_stdout_write_bytes(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("writeStdoutBytes") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };

        object_node.text.as_deref() == Some("Kali")
    })
}

pub(crate) fn collect_functions_from_node(
    lir: &LirProgram,
    id: LirNodeId,
    visited: &mut HashSet<LirNodeId>,
    plans: &mut Vec<FunctionPlan>,
    repr_table: &kali_common::ReprTable,
    arena_table: &kali_common::ArenaTable,
) {
    if !visited.insert(id) {
        return;
    }

    if let Some(plan) = function_plan(&lir.nodes, id, repr_table, arena_table) {
        plans.push(plan);
    }

    let Some(node) = lir.nodes.get(id.0 as usize) else {
        return;
    };

    for child in &node.children {
        collect_functions_from_node(lir, *child, visited, plans, repr_table, arena_table);
    }
}

/// Structural shape of a function-like `Instruction` node: name, body, and
/// params — everything `function_plan` needs except the repr-directed local
/// slots, which only `function_plan` itself computes. Shared so
/// `is_function_like` can answer its purely-structural question without a
/// `ReprTable` (it is consulted from contexts, like `is_function_like`'s own
/// callers, that only care whether a node IS a function, not its locals).
fn function_shape(
    nodes: &[LirNode],
    id: LirNodeId,
) -> Option<(String, Option<FunctionFlavor>, LirNodeId, Vec<String>)> {
    let node = nodes.get(id.0 as usize)?;
    if node.kind != LirNodeKind::Instruction {
        return None;
    }
    let name = node.text.clone()?;
    let flavor = node.function_flavor;
    if node.children.is_empty() {
        return None;
    }
    let body_id = *node.children.last()?;
    let body_node = nodes.get(body_id.0 as usize)?;
    // A function body is either a real `Block` (function declarations,
    // function expressions, block-bodied arrows) or the single synthesized
    // `Branch("return")` statement an expression-bodied arrow lowers to
    // (`(x, y) => x + y`). Recognizing the latter compiles const-bound arrows
    // as standalone wasm functions: inside their own function the emitted
    // `Instruction::Return` is correct, whereas inlining it at the declaration
    // site terminated the ENCLOSING function (silently truncating execution
    // with exit 0). Call sites already dispatch through the const `bindings`
    // resolution in `resolve_bound_member_callable_node`.
    let is_block_body = body_node.kind == LirNodeKind::Block;
    let is_arrow_return_body =
        body_node.kind == LirNodeKind::Branch && body_node.text.as_deref() == Some("return");
    if !is_block_body && !is_arrow_return_body {
        return None;
    }

    let mut params = Vec::new();
    for child in node.children.iter().take(node.children.len() - 1) {
        let child_node = nodes.get(child.0 as usize)?;
        if child_node.kind == LirNodeKind::Value {
            params.push(child_node.text.clone().unwrap_or_default());
        }
    }

    Some((name, flavor, body_id, params))
}

pub(crate) fn function_plan(
    nodes: &[LirNode],
    id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    arena_table: &kali_common::ArenaTable,
) -> Option<FunctionPlan> {
    let (name, flavor, body_id, params) = function_shape(nodes, id)?;
    let locals = collect_function_locals(nodes, body_id, repr_table, arena_table, &name);

    Some(FunctionPlan {
        name,
        params,
        locals,
        body: body_id,
        result: true,
        is_entry: false,
        flavor,
    })
}

pub(crate) fn is_function_like(nodes: &[LirNode], id: LirNodeId) -> bool {
    function_shape(nodes, id).is_some()
}

/// Pre-order, function-scoped loop-ordinal assignment over the LIR tree
/// rooted at `body`: the `k`-th loop-shaped `Branch` node (`for` / `while` /
/// `do-while` / `for-of` / `for-await-of` text) encountered while recursing
/// over `children` in array order gets ordinal `k` (0-based), assigned
/// *before* recursing into that node's own children (pre-order) — a nested
/// function body is treated as an opaque leaf (never descended into; its own
/// loops are numbered independently by a separate call to this same helper
/// against that function's own body, never by continuing this counter).
///
/// This MUST stay in lockstep with the per-function pre-order loop ordinal
/// the `kali_mir` escape-gate assigns while walking the matching HIR loop
/// nodes (`OwnershipAnalyzer::arena_enter_loop`, `kali_mir::analysis::walk`):
/// both recurse over children in the same array order, and both reset their
/// counter at a nested function boundary rather than carrying it through.
/// Every MIR→LIR lowering stage in this crate is a 1:1 structural copy (same
/// node count, same child order, same text) — see `kali_mir::lower` and
/// `kali_lir::lower` — so the two walks visit loop nodes in the identical
/// relative order. A single shared helper, called from both locals
/// provisioning (`collect_function_locals`) and emission
/// (`FunctionEmitter::new`), is used so the two call sites cannot diverge
/// from each other (see Task 6's brief: a divergence here would install an
/// arena on the wrong loop — a use-after-reset miscompile).
///
/// `for-in` is deliberately skipped by BOTH walks, and must stay that way: a
/// `for-in` loop's HIR/LIR node carries no distinguishing `text` (same as an
/// `if` statement without one), so it is invisible to this text-based
/// recognizer. `kali_mir::analysis::walk` mirrors that on purpose — its
/// `ForInStmt` arm does NOT call `arena_enter_loop()` — precisely so it never
/// advances its ordinal counter past a for-in either. (An earlier revision of
/// this comment claimed the `kali_mir` walk assigned for-in an ordinal too,
/// on the theory that a mismatch there was harmless because for-in itself is
/// unsupported by codegen; that was wrong — assigning for-in an ordinal on
/// only one side would desync every REAL loop lexically following it in the
/// same function, sending `loop_arena(fn, ordinal)` lookups to the wrong
/// loop, not just failing to support for-in's own.) `for-in` is still
/// unsupported by codegen today for unrelated reasons (it falls through
/// `emit_node`'s `Branch` match to `emit_branch`, i.e. is silently
/// mis-lowered as an `if`) — but a future `for-in` implementation must give
/// BOTH walks a way to recognize it, together, not just this one.
pub(crate) fn loop_preorder_ordinals(
    nodes: &[LirNode],
    body: LirNodeId,
) -> HashMap<LirNodeId, u32> {
    let mut ordinals = HashMap::new();
    let mut next = 0u32;
    loop_preorder_ordinals_walk(nodes, body, &mut next, &mut ordinals);
    ordinals
}

/// Pre-order, function-scoped ordinal assigned to each `for-in`-text `Branch`
/// node's LIR id. Completely independent of `loop_preorder_ordinals` (which
/// deliberately does NOT recognize `for-in` — see that function's doc comment
/// on the arena-ordinal desync danger of doing so) and never consulted by
/// `kali_mir`'s escape gate: this ordinal exists ONLY to name a dedicated
/// per-for-in scratch i64 local (`for_in_ord_local_name`) that holds the
/// loop's own counter, so nested emission inside the for-in body (e.g. an
/// object allocation, which reuses the function's generic trailing scratch
/// local — see `emit_object_allocation`) can never clobber it. Consulted from
/// exactly two call sites, both inside `kali_codegen`
/// (`collect_function_locals`, which reserves the local, and
/// `FunctionEmitter::new`, which resolves it back for `emit_for_in`) — it has
/// no bearing on arena placement and must never be threaded into
/// `ArenaTable`/`loop_arena` lookups.
pub(crate) fn for_in_preorder_ordinals(
    nodes: &[LirNode],
    body: LirNodeId,
) -> HashMap<LirNodeId, u32> {
    let mut ordinals = HashMap::new();
    let mut next = 0u32;
    for_in_preorder_ordinals_walk(nodes, body, &mut next, &mut ordinals);
    ordinals
}

/// Structural per-function set of "for-in-key provenance" binding names: every
/// `for..in` loop key declared in the function, plus every binding aliased
/// directly from such a key (`last = c`). Codegen's null-sentinel (`-1`) store
/// and truthiness (`>= 0`) special-cases key off this set — computed once up
/// front so the `var last = null` init (emitted BEFORE the loop) already
/// recognizes `last`. Structural twin of the types-side `for..in` key +
/// `last = c` provenance propagation (mirror binding provenance, not repr).
pub(crate) fn for_in_key_alias_names(nodes: &[LirNode], body: LirNodeId) -> HashSet<String> {
    let mut names: HashSet<String> = HashSet::new();
    for_in_loop_keys_walk(nodes, body, &mut names);
    // Transitive closure: `y = last` inherits provenance when `last` is already
    // recognized (a chain of `= <recognized alias>` from a loop key). Iterate to
    // a fixpoint so codegen recognizes exactly the transitive set the types side
    // admits (its `last = c` propagation reads a growing registry) — symmetric,
    // no fail-open on a two-plus-level alias.
    loop {
        let before = names.len();
        let mut next = names.clone();
        for_in_key_aliases_walk(nodes, body, &names, &mut next);
        names = next;
        if names.len() == before {
            break;
        }
    }
    names
}

/// The for-in loop key name for a for-in Branch node's `left` child, whether it
/// is a `var`/`let`/`const` declarator (`for (var c in obj)`) or a bare
/// identifier (`for (c in obj)`). Free-function twin of
/// `FunctionEmitter::for_in_key_name`.
fn for_in_loop_key_name(nodes: &[LirNode], left_id: LirNodeId) -> Option<String> {
    let left = nodes.get(left_id.0 as usize)?;
    if left.kind == LirNodeKind::Instruction
        && matches!(left.text.as_deref(), Some("let" | "var" | "const"))
    {
        if let Some(&declarator_id) = left.children.first() {
            if let Some(name) = nodes
                .get(declarator_id.0 as usize)
                .and_then(|n| n.text.clone())
            {
                return Some(name).filter(|t| !t.is_empty());
            }
        }
    }
    left.text.clone().filter(|t| !t.is_empty())
}

/// Free-function twin of `FunctionEmitter::bare_identifier_name` operating on a
/// raw node slice (used before any scratch nodes exist).
fn bare_identifier_name_of(nodes: &[LirNode], id: LirNodeId) -> Option<String> {
    let target = unwrap_transparent_value(nodes, id);
    let node = nodes.get(target.0 as usize)?;
    if node.kind == LirNodeKind::Value && node.children.is_empty() {
        node.text.clone().filter(|t| !t.is_empty())
    } else {
        None
    }
}

fn for_in_loop_keys_walk(nodes: &[LirNode], id: LirNodeId, keys: &mut HashSet<String>) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("for-in") {
        if let Some(&left_id) = node.children.first() {
            if let Some(name) = for_in_loop_key_name(nodes, left_id) {
                keys.insert(name);
            }
        }
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        for_in_loop_keys_walk(nodes, *child, keys);
    }
}

pub(crate) fn for_in_key_aliases_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    keys: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    // A direct alias `X = K` where `K` is a for-in loop key: `X` inherits the
    // key provenance. Represented as a 2-child `=` Value node with a bare
    // identifier on each side.
    if node.kind == LirNodeKind::Value
        && node.children.len() == 2
        && node.text.as_deref() == Some("=")
    {
        if let (Some(lhs), Some(rhs)) = (
            bare_identifier_name_of(nodes, node.children[0]),
            bare_identifier_name_of(nodes, node.children[1]),
        ) {
            if keys.contains(&rhs) {
                out.insert(lhs);
            }
        }
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        for_in_key_aliases_walk(nodes, *child, keys, out);
    }
}

fn for_in_preorder_ordinals_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    next: &mut u32,
    ordinals: &mut HashMap<LirNodeId, u32>,
) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("for-in") {
        ordinals.insert(id, *next);
        *next += 1;
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        for_in_preorder_ordinals_walk(nodes, *child, next, ordinals);
    }
}

/// Name of the dedicated i64 scratch local holding the `for-in` loop's own
/// ordinal counter (`for_in_preorder_ordinals`-keyed). The `#` makes this
/// unrepresentable as a source-level identifier, matching the convention
/// `arena_save_local_names` uses, so it can never collide with a real binding.
pub(crate) fn for_in_ord_local_name(ordinal: u32) -> String {
    format!("__for_in_ord#{ordinal}")
}

/// Name of the dedicated i64 local holding the base pointer of a `for-in`
/// loop's per-shape key handle table (Spec 4a Task 5, `emit_key_handle_table`).
/// The table is bump-allocated once in the loop preheader; this local must
/// PERSIST across the whole loop body (a `return c`/`c + x` string use loads
/// `base + ord*8` from it), so it is a dedicated reserved slot — NOT the
/// function's transient trailing scratch, which body emission (e.g. an
/// `obj[c] = v` write) reuses and would clobber. Same `#`-name convention +
/// two-call-site (reserve here, resolve in `emit_for_in`) discipline as
/// `for_in_ord_local_name`.
pub(crate) fn for_in_key_table_local_name(ordinal: u32) -> String {
    format!("__for_in_ktbl#{ordinal}")
}

/// Names of the three synthetic i32 locals that save/restore the
/// current-arena trio (`g1`/`g2`/`g3`) around the arena'd loop with pre-order
/// ordinal `ordinal` in its function. Shared by locals provisioning
/// (`collect_function_locals`, which reserves these slots) and emission
/// (`control_flow.rs::emit_loop`/`emit_arena_release`, which reads them back
/// by name through `FunctionEmitter::locals`) so the two cannot disagree on
/// naming. The `#` makes these unrepresentable as a source-level identifier,
/// so they can never collide with a real binding name.
pub(crate) fn arena_save_local_names(ordinal: u32) -> (String, String, String) {
    (
        format!("__arena_save_page#{ordinal}"),
        format!("__arena_save_cursor#{ordinal}"),
        format!("__arena_save_limit#{ordinal}"),
    )
}

/// Names of the three synthetic i32 locals that save/restore the
/// current-arena trio (`g1`/`g2`/`g3`) around a per-call FUNCTION-BODY arena
/// (Task 7) — the sibling of `arena_save_local_names` above for the single
/// function-level `ArenaFrame` (`loop_frame_index: None`) a function opens on
/// entry, rather than a per-loop one. Fixed (not keyed by an ordinal) since a
/// function has at most one such frame. Shared by `collect_function_locals`
/// (which reserves these slots when `ArenaTable::opens_arena` grants this
/// function one) and `control_flow.rs::emit_function_arena_prologue`/
/// `emit_function_arena_epilogue` (which read them back by name through
/// `FunctionEmitter::locals`), so the two cannot disagree on naming. The
/// `#fn` suffix can never collide with a per-loop name: `arena_save_local_names`
/// only ever formats a `u32` ordinal there, never the literal text `fn`.
pub(crate) fn arena_save_local_names_for_function() -> (String, String, String) {
    (
        "__arena_save_page#fn".to_string(),
        "__arena_save_cursor#fn".to_string(),
        "__arena_save_limit#fn".to_string(),
    )
}

/// True for a local name synthesized by `arena_save_local_names` — these hold
/// a saved copy of an i32 global (`g1`/`g2`/`g3`) and must be declared as i32
/// locals regardless of what `ReprTable` would otherwise infer for an
/// unrecorded name (its default, `Repr::I64`, would mistype the slot and fail
/// wasm validation the first time `GlobalGet(1..3)` is stored into it).
pub(crate) fn is_arena_save_local_name(name: &str) -> bool {
    name.starts_with("__arena_save_")
}

fn loop_preorder_ordinals_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    next: &mut u32,
    ordinals: &mut HashMap<LirNodeId, u32>,
) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Branch
        && matches!(
            node.text.as_deref(),
            Some("for" | "while" | "do-while" | "for-of" | "for-await-of")
        )
    {
        ordinals.insert(id, *next);
        *next += 1;
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        loop_preorder_ordinals_walk(nodes, *child, next, ordinals);
    }
}

pub(crate) fn collect_function_locals(
    nodes: &[LirNode],
    body_id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    arena_table: &kali_common::ArenaTable,
    function_name: &str,
) -> Vec<String> {
    // First identify every binding that holds a linear-memory array handle, so that
    // `const` reads of those arrays can be promoted to eagerly-evaluated locals.
    let mut array_names = HashSet::new();
    let mut array_seen = HashSet::new();
    collect_array_binding_names(
        nodes,
        body_id,
        repr_table,
        function_name,
        &mut array_seen,
        &mut array_names,
    );

    let mut locals = Vec::new();
    let mut seen = HashSet::new();
    collect_function_locals_from_node(
        nodes,
        body_id,
        &array_names,
        repr_table,
        function_name,
        &mut seen,
        &mut locals,
    );

    // Reserve 3 synthetic i32 locals (saved page/cursor/limit) per arena'd
    // loop in this function, keyed by the SAME pre-order loop ordinal
    // `loop_preorder_ordinals` assigns during emission — see that function's
    // doc comment for why this must be the one shared helper.
    let mut loop_ordinals: Vec<u32> = loop_preorder_ordinals(nodes, body_id)
        .into_values()
        .collect();
    loop_ordinals.sort_unstable();
    for ordinal in loop_ordinals {
        if arena_table.loop_arena(function_name, ordinal) {
            let (page, cursor, limit) = arena_save_local_names(ordinal);
            locals.push(page);
            locals.push(cursor);
            locals.push(limit);
        }
    }

    // Reserve 3 more synthetic i32 locals (Task 7) when this function itself
    // opens a function-body arena — the bottom-of-stack `ArenaFrame` pushed
    // by `emit_function_arena_prologue` and released by
    // `emit_function_arena_epilogue`/`emit_return`'s all-frames unwind.
    if arena_table.opens_arena(function_name) {
        let (page, cursor, limit) = arena_save_local_names_for_function();
        locals.push(page);
        locals.push(cursor);
        locals.push(limit);
    }

    // Reserve one dedicated i64 scratch local per `for-in` loop in this
    // function (Task 1 of Spec 4a) — see `for_in_preorder_ordinals`'s doc
    // comment for why this is a wholly separate, codegen-internal counter
    // from the arena-ordinal one above.
    let mut for_in_ordinals: Vec<u32> = for_in_preorder_ordinals(nodes, body_id)
        .into_values()
        .collect();
    for_in_ordinals.sort_unstable();
    for ordinal in for_in_ordinals {
        locals.push(for_in_ord_local_name(ordinal));
        // Spec 4a Task 5: a parallel dedicated i64 local per for-in loop for the
        // key handle-table base pointer (persists across the loop body).
        locals.push(for_in_key_table_local_name(ordinal));
    }

    locals
}

/// WASM globals reserved before any module-scope mutable scalar global:
/// g0 (heap/page frontier) + g1..g7 (arena page-pool state). Module scalar
/// globals are appended AFTER these, at indices `RESERVED_GLOBAL_COUNT`, +1, …
pub(crate) const RESERVED_GLOBAL_COUNT: u32 = 8;

/// Promote module-scope mutable SCALAR (`var`/`let` numeric) bindings that are
/// READ or WRITTEN from inside a function to persistent mutable WASM globals.
///
/// Returns `name -> (global_index, repr)`, sorted by name (so the index order
/// matches the order globals are appended to the `GlobalSection`). Only
/// numeric (`I64`/`F64`) scalars qualify: an array/object/string module binding
/// is a heap type, and a mutable global heap root is a persistent GC root the
/// GC-less region reclamation does not model — those stay fail-closed (E5506).
/// A `const` is excluded (it stays on the compile-time inline path). A scalar
/// referenced only at module scope is NOT promoted (it keeps its byte-identical
/// `_start`-local lowering).
pub(crate) fn collect_module_scalar_globals(
    lir: &LirProgram,
    repr_table: &kali_common::ReprTable,
    function_plans: &[FunctionPlan],
) -> BTreeMap<String, (u32, kali_common::Repr)> {
    // Top-level `var`/`let` numeric scalar declarators (never `const`).
    let mut candidates: BTreeMap<String, kali_common::Repr> = BTreeMap::new();
    let mut stack = vec![lir.root];
    while let Some(id) = stack.pop() {
        let Some(node) = lir.nodes.get(id.0 as usize) else {
            continue;
        };
        match node.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                stack.extend(node.children.iter().copied());
            }
            LirNodeKind::Instruction if matches!(node.text.as_deref(), Some("let" | "var")) => {
                for declarator_id in &node.children {
                    let Some(declarator) = lir.nodes.get(declarator_id.0 as usize) else {
                        continue;
                    };
                    let Some(name) = declarator.text.clone() else {
                        continue;
                    };
                    // Heap types stay fail-closed: never promote an array/object
                    // module binding to a mutable global root.
                    if repr_table.is_array_binding("_start", &name) {
                        continue;
                    }
                    // `Object(_)` / `String` reprs are heap — leave rejected;
                    // only a numeric scalar becomes a mutable global.
                    let repr = repr_table.scalar("_start", &name);
                    if matches!(repr, kali_common::Repr::I64 | kali_common::Repr::F64) {
                        candidates.insert(name, repr);
                    }
                }
            }
            _ => {}
        }
    }
    if candidates.is_empty() {
        return BTreeMap::new();
    }

    // Names used ANYWHERE as a member/index base (`o.x`, `a[i]`) are heap
    // (object/array) bindings — even when repr inference left them a default
    // `I64` (e.g. a module object mutated only cross-function never proves its
    // shape). Promoting such a name to a mutable scalar global would fail OPEN:
    // a member access on an i64 global silently reads 0. Exclude them so those
    // cases stay fail-closed (E5506), never mis-lowered.
    let mut heap_base_names: HashSet<String> = HashSet::new();
    {
        let mut seen = HashSet::new();
        collect_member_base_names(&lir.nodes, lir.root, &mut seen, &mut heap_base_names);
    }

    // Only bindings referenced from inside a function need a persistent global;
    // a module-only scalar stays a `_start` local (byte-identical).
    let mut referenced: HashSet<String> = HashSet::new();
    for plan in function_plans {
        if plan.name == "_start" {
            continue;
        }
        let mut seen = HashSet::new();
        collect_bare_identifier_names(&lir.nodes, plan.body, &mut seen, &mut referenced);
    }

    let mut slots = BTreeMap::new();
    let mut next_index = RESERVED_GLOBAL_COUNT;
    for (name, repr) in candidates {
        if referenced.contains(&name) && !heap_base_names.contains(&name) {
            slots.insert(name, (next_index, repr));
            next_index += 1;
        }
    }
    slots
}

/// Collect every identifier used as a member/index base — the `o` in `o.x`,
/// `o.length`, or `o[i]` — anywhere in the subtree at `id`. A member access is
/// a `Value` node whose first child is the base:
/// - 1 child + non-empty `text` that is NOT a unary operator (a dot/`.length`
///   access; a unary `-g`/`!g` also has 1 child + text but is not a base use),
/// - 2 children + `text` that is NOT a binary operator (a computed `o[i]`; a
///   binary expression also has 2 children but carries an operator text).
fn collect_member_base_names(
    nodes: &[LirNode],
    id: LirNodeId,
    seen: &mut HashSet<LirNodeId>,
    out: &mut HashSet<String>,
) {
    if !seen.insert(id) {
        return;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Value {
        let base_child = match node.children.len() {
            1 => node
                .text
                .as_deref()
                .filter(|text| !text.is_empty() && !is_unary_operator_text(text))
                .map(|_| node.children[0]),
            2 => (!is_binary_operator_text(node.text.as_deref().unwrap_or_default()))
                .then_some(node.children[0]),
            _ => None,
        };
        if let Some(base_child) = base_child {
            let base = unwrap_transparent_value(nodes, base_child);
            if let Some(base_node) = nodes.get(base.0 as usize) {
                if base_node.kind == LirNodeKind::Value && base_node.children.is_empty() {
                    if let Some(text) = base_node.text.as_deref() {
                        if !text.is_empty() {
                            out.insert(text.to_string());
                        }
                    }
                }
            }
        }
    }
    for child in &node.children {
        collect_member_base_names(nodes, *child, seen, out);
    }
}

/// Collect every bare identifier name (a childless `Value` node with non-empty
/// text — an identifier read or an assignment target) in the subtree at `id`.
/// Over-collection (e.g. numeric-literal texts) is harmless: it only ever
/// admits MORE names to the promotion set, and a name that is not a module
/// scalar candidate is ignored by the caller.
fn collect_bare_identifier_names(
    nodes: &[LirNode],
    id: LirNodeId,
    seen: &mut HashSet<LirNodeId>,
    out: &mut HashSet<String>,
) {
    if !seen.insert(id) {
        return;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Value && node.children.is_empty() {
        if let Some(text) = node.text.as_deref() {
            if !text.is_empty() {
                out.insert(text.to_string());
            }
        }
    }
    for child in &node.children {
        collect_bare_identifier_names(nodes, *child, seen, out);
    }
}

pub(crate) fn collect_array_binding_names(
    nodes: &[LirNode],
    id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    seen: &mut HashSet<LirNodeId>,
    array_names: &mut HashSet<String>,
) {
    if !seen.insert(id) {
        return;
    }

    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };

    if node.kind == LirNodeKind::Instruction
        && matches!(node.text.as_deref(), Some("let" | "var" | "const"))
    {
        for declarator in &node.children {
            let Some(declarator_node) = nodes.get(declarator.0 as usize) else {
                continue;
            };
            let Some(init) = declarator_node.children.get(1).copied() else {
                continue;
            };
            let Some(name) = declarator_node.text.clone() else {
                continue;
            };
            // An array literal of object references (`const bodies = [{…}, …]`)
            // is a real linear-memory array only once shape inference decided
            // its elements are `Repr::Object` (materialized, i.e. read/written
            // through more than the compile-time fold lane); a plain scalar
            // literal (`[1, 2, 3]`) has no `array_element` entry at all and
            // stays untouched. Mirrors `declarator_init_is_array_alloc`/`_fill`
            // below, which likewise mark the binding as a real array handle.
            let is_object_array_literal = declarator_init_is_array_literal(nodes, init)
                && matches!(
                    repr_table.array_element(function_name, &name),
                    kali_common::Repr::Object(_)
                );
            if declarator_init_is_array_alloc(nodes, init)
                || declarator_init_is_array_fill(nodes, init)
                || is_object_array_literal
            {
                array_names.insert(name);
            }
        }
    }

    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        collect_array_binding_names(nodes, *child, repr_table, function_name, seen, array_names);
    }
}

pub(crate) fn collect_function_locals_from_node(
    nodes: &[LirNode],
    id: LirNodeId,
    array_names: &HashSet<String>,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    seen: &mut HashSet<LirNodeId>,
    locals: &mut Vec<String>,
) {
    if !seen.insert(id) {
        return;
    }

    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };

    if node.kind == LirNodeKind::Instruction && matches!(node.text.as_deref(), Some("let" | "var"))
    {
        for declarator in &node.children {
            let Some(declarator_node) = nodes.get(declarator.0 as usize) else {
                continue;
            };
            if let Some(name) = declarator_node.text.clone() {
                if !locals.contains(&name) {
                    locals.push(name);
                }
            }
        }
    }

    // `const` bindings normally inline their initializer, but an allocation
    // (`new Array(n)`) needs a stable handle and a read of a mutable array
    // (`const t = a[i]`) must be evaluated eagerly; both are promoted to locals.
    // A materialized object-literal binding (its inferred repr is `Object`,
    // i.e. the shape inference recorded it as heap-allocated) needs the same
    // stable handle, so its mutations (`p.x = ...`) and aliases (`const q = p`)
    // observe the same storage; an object literal with no shape entry stays on
    // the compile-time fold lane untouched (fold-first).
    if node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some("const") {
        for declarator in &node.children {
            let Some(declarator_node) = nodes.get(declarator.0 as usize) else {
                continue;
            };
            let Some(init) = declarator_node.children.get(1).copied() else {
                continue;
            };
            let is_materialized_object = declarator_node.text.as_deref().is_some_and(|name| {
                declarator_init_is_object_literal(nodes, init)
                    && matches!(
                        repr_table.scalar(function_name, name),
                        kali_common::Repr::Object(_)
                    )
            });
            // An array literal of object references needs the same stable
            // handle as a `new Array(n)` allocation, so its own base pointer
            // (not just later reads of it) is promoted to a local. See
            // `collect_array_binding_names`'s matching check.
            let is_materialized_object_array =
                declarator_node.text.as_deref().is_some_and(|name| {
                    declarator_init_is_array_literal(nodes, init)
                        && matches!(
                            repr_table.array_element(function_name, name),
                            kali_common::Repr::Object(_)
                        )
                });
            // A factory-call initializer (`const q = mk(2.0)`) whose callee
            // returns an `Object` repr matching the binding's own repr needs
            // the same stable handle: without promotion, the const folds to
            // re-evaluating the call at every use site (`resolve_literal_aggregate`'s
            // `bindings` alias lane), silently calling the factory again on
            // each read/write instead of sharing the one materialized object
            // — a distinct-instances miscompile, not just a missed optimization.
            let is_materialized_factory_return =
                declarator_node.text.as_deref().is_some_and(|name| {
                    match repr_table.scalar(function_name, name) {
                        kali_common::Repr::Object(_) => {
                            declarator_init_call_callee_name(nodes, init).is_some_and(|callee| {
                                matches!(
                                    repr_table.return_repr(callee),
                                    kali_common::Repr::Object(_)
                                )
                            })
                        }
                        _ => false,
                    }
                });
            if !declarator_init_is_array_alloc(nodes, init)
                && !declarator_init_is_array_fill(nodes, init)
                && !declarator_init_is_array_read(nodes, init, array_names)
                && !is_materialized_object
                && !is_materialized_object_array
                && !is_materialized_factory_return
            {
                continue;
            }
            if let Some(name) = declarator_node.text.clone() {
                if !locals.contains(&name) {
                    locals.push(name);
                }
            }
        }
    }

    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        collect_function_locals_from_node(
            nodes,
            *child,
            array_names,
            repr_table,
            function_name,
            seen,
            locals,
        );
    }
}

/// Returns true if `init_id` unwraps to a dynamic array element read `base[index]`
/// whose `base` identifier is a known linear-memory array binding.
/// A two-child `LirNodeKind::Value` node is a binary expression when its text is
/// an operator, and a computed member access (`a[<expr>]`) otherwise — computed
/// indices never stringify to a bare operator, so `text` cleanly separates the
/// two shapes that both lower to a two-child `Value` node.
///
/// Known limitation: a computed member whose index is a string LITERAL equal to
/// an operator (e.g. `obj["+"]`, `obj["in"]`) stringifies to that bare operator
/// and would be misclassified as a binary expression. This is unreachable in the
/// current integer-only slice (no general object with operator-named string keys
/// is expressible/evaluable); if a richer object model is added, disambiguate on
/// node kind (member vs binary) rather than on `text` here.
pub(crate) fn is_binary_operator_text(text: &str) -> bool {
    matches!(
        text,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "=="
            | "==="
            | "!="
            | "!=="
            | "<"
            | "<="
            | ">"
            | ">="
            | "<<"
            | ">>"
            | ">>>"
            | "&"
            | "|"
            | "^"
            | "&&"
            | "||"
            | "??"
            | ","
            | "in"
            | "instanceof"
            | "="
            | "+="
            | "-="
            | "*="
            | "/="
            | "%="
            | "**="
            | "<<="
            | ">>="
            | ">>>="
            | "&="
            | "|="
            | "^="
            | "&&="
            | "||="
            | "??="
    )
}

/// Unary operator texts a 1-child `Value` node may carry. Used to distinguish a
/// unary expression (`-g`, `!g`) from a dot/`.length` member access (`o.x`) —
/// both lower to a 1-child `Value` with a `text`, but only the latter uses `o`
/// as a heap (object) base (see `collect_member_base_names`).
pub(crate) fn is_unary_operator_text(text: &str) -> bool {
    matches!(
        text,
        "-" | "+"
            | "~"
            | "!"
            | "void"
            | "delete"
            | "typeof"
            | "prefix++"
            | "postfix++"
            | "prefix--"
            | "postfix--"
    )
}

pub(crate) fn declarator_init_is_array_read(
    nodes: &[LirNode],
    init_id: LirNodeId,
    array_names: &HashSet<String>,
) -> bool {
    let member = unwrap_transparent_value(nodes, init_id);
    let Some(member_node) = nodes.get(member.0 as usize) else {
        return false;
    };
    // Literal/identifier index reads lower to a 1-child member node (`[base]`)
    // with the index in `text`; computed reads (`a[i + 1]`) lower to a 2-child
    // member node (`[base, index]`). Both must be promoted to an eager local.
    if member_node.kind != LirNodeKind::Value
        || !(member_node.children.len() == 1 || member_node.children.len() == 2)
    {
        return false;
    }
    // A two-child node whose text is an operator is a binary expression
    // (`const t = a + b`), not a computed member read.
    if member_node.children.len() == 2
        && is_binary_operator_text(member_node.text.as_deref().unwrap_or_default())
    {
        return false;
    }
    if member_node
        .text
        .as_deref()
        .is_none_or(|text| text.is_empty())
    {
        return false;
    }
    let base = unwrap_transparent_value(nodes, member_node.children[0]);
    let Some(base_node) = nodes.get(base.0 as usize) else {
        return false;
    };
    base_node.kind == LirNodeKind::Value
        && base_node.children.is_empty()
        && base_node
            .text
            .as_deref()
            .is_some_and(|name| array_names.contains(name))
}

fn unwrap_transparent_value(nodes: &[LirNode], mut id: LirNodeId) -> LirNodeId {
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return id;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref().is_none_or(|text| text.is_empty())
        {
            id = node.children[0];
            guard += 1;
            if guard > 64 {
                return id;
            }
            continue;
        }
        return id;
    }
}

/// Returns true if `init_id` (after unwrapping transparent value wrappers) is a
/// `new Array(n)` / `Array(n)` allocation call (callee identifier `Array`, 0 or 1 arg).
pub(crate) fn declarator_init_is_array_alloc(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref().is_none_or(|text| text.is_empty())
        {
            id = node.children[0];
            guard += 1;
            if guard > 64 {
                return false;
            }
            continue;
        }

        if node.kind != LirNodeKind::Call || node.children.len() > 2 {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = nodes.get(callee.0 as usize) else {
            return false;
        };
        return callee_node.text.as_deref() == Some("Array") && callee_node.children.is_empty();
    }
}

/// Returns true if `init_id` (after unwrapping genuine sequence wrappers) is
/// an object-literal expression: a `Value` node with no text whose every
/// child is a 2-child `init`/`get`/`set` property node with a `Literal` key.
/// Mirrors `FunctionEmitter::is_object_literal`, but as a free function
/// usable here, before a `FunctionEmitter` exists (local-slot collection runs
/// first, ahead of emission).
///
/// Deliberately does NOT reuse `unwrap_transparent_value`: that helper
/// unwraps any single-child node whose text is `None` OR empty, but an
/// object literal's own top node ALSO has `text: None` — a single-property
/// literal (`{n: 3}`, exactly one property child) has the identical
/// "one child, no text" shape, so that unwrap would wrongly descend into the
/// lone property node and report "not a literal". A genuine sequence
/// wrapper's text is the empty string (`Some("")`), never `None` — that
/// narrower condition (mirroring `resolve_literal_aggregate`'s own
/// sequence-wrapper check) is what's used to unwrap here.
pub(crate) fn declarator_init_is_object_literal(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("")
            && !node.children.is_empty()
        {
            id = *node.children.last().expect("sequence wrapper has a child");
            guard += 1;
            if guard > 64 {
                return false;
            }
            continue;
        }
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }
        return node.children.iter().all(|child| {
            let Some(child_node) = nodes.get(child.0 as usize) else {
                return false;
            };
            child_node.children.len() == 2
                && child_node
                    .text
                    .as_deref()
                    .is_some_and(|kind| matches!(kind, "init" | "get" | "set"))
                && child_node
                    .children
                    .first()
                    .and_then(|key| nodes.get(key.0 as usize))
                    .is_some_and(|key_node| key_node.kind == LirNodeKind::Literal)
        });
    }
}

/// Returns the callee name if `init_id` (after unwrapping genuine sequence
/// wrappers) is a plain call `name(...)` — a `Call` node whose callee is a
/// bare identifier (no receiver, i.e. not a method call). Used to detect a
/// factory-function initializer (`const q = mk(2.0)`) so its binding can be
/// promoted to a stable local when the factory's return repr is `Object`;
/// otherwise the const would fold to re-evaluating the call at every use
/// site, silently duplicating the returned object (see
/// `is_materialized_factory_return` below).
pub(crate) fn declarator_init_call_callee_name(
    nodes: &[LirNode],
    init_id: LirNodeId,
) -> Option<&str> {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let node = nodes.get(id.0 as usize)?;
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("")
            && !node.children.is_empty()
        {
            id = *node.children.last().expect("sequence wrapper has a child");
            guard += 1;
            if guard > 64 {
                return None;
            }
            continue;
        }
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = node.children.first()?;
        let callee_node = nodes.get(callee.0 as usize)?;
        if !callee_node.children.is_empty() {
            return None;
        }
        return callee_node.text.as_deref();
    }
}

/// Returns true if `init_id` (after unwrapping genuine sequence wrappers) is
/// an array-literal expression: a `Value` node with no text that is not an
/// object literal. Mirrors `FunctionEmitter::is_array_literal`, but as a free
/// function usable here, before a `FunctionEmitter` exists — see the doc
/// comment on `declarator_init_is_object_literal` for why the sequence-wrapper
/// unwrap can't reuse `unwrap_transparent_value`.
pub(crate) fn declarator_init_is_array_literal(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("")
            && !node.children.is_empty()
        {
            id = *node.children.last().expect("sequence wrapper has a child");
            guard += 1;
            if guard > 64 {
                return false;
            }
            continue;
        }
        if node.kind != LirNodeKind::Value || node.text.is_some() {
            return false;
        }
        return !declarator_init_is_object_literal(nodes, id);
    }
}

/// `<array-alloc>.fill(v)` — a `.fill(v)` member call whose receiver is a
/// `new Array(n)` / `Array(n)` allocation. Like a bare allocation, this both
/// produces a fresh linear-memory array (so the binding needs a stable local
/// handle) and registers an array binding, so it is collected the same way.
pub(crate) fn declarator_init_is_array_fill(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    let node = loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref().is_none_or(|text| text.is_empty())
        {
            id = node.children[0];
            guard += 1;
            if guard > 64 {
                return false;
            }
            continue;
        }
        break node;
    };

    if node.kind != LirNodeKind::Call || node.children.len() != 2 {
        return false;
    }
    let Some(callee) = node.children.first().copied() else {
        return false;
    };
    let Some(callee_node) = nodes.get(callee.0 as usize) else {
        return false;
    };
    // The callee is the member expression `<receiver>.fill`: `text` is the method
    // name and `children[0]` is the receiver.
    if callee_node.text.as_deref() != Some("fill") {
        return false;
    }
    let Some(receiver) = callee_node.children.first().copied() else {
        return false;
    };
    declarator_init_is_array_alloc(nodes, receiver)
}

/// Page layout shared by every page the pool hands out: an 8-byte header
/// (`next: i32 @ 0`, `span_pages: i32 @ 4`) followed by payload at offset 8.
/// `span_pages` is only meaningful on the first page of a (possibly
/// multi-page) span; interior pages of a span are never independently headed.
const PAGE: i32 = 65536;
const HEADER: i32 = 8;
const PAYLOAD: i32 = PAGE - HEADER;

fn page_mem_arg(offset: u64) -> MemArg {
    MemArg {
        offset,
        align: 2,
        memory_index: 0,
    }
}

/// Bump allocator against one arena trio (page-list head `g_page`, cursor
/// `g_cur`, limit `g_lim`) — the shared body for both `__alloc`
/// (`g_page/g_cur/g_lim` = g1/g2/g3) and `__alloc_global`
/// (g4/g5/g6). Param 0 = `size`; locals 1 = `cur`, 2 = `p`.
///
/// Three paths, tried in order:
///   1. **Fast path**: `cur + size <= g_lim` — bump `g_cur` and return `cur`
///      in place, no call.
///   2. **Span path** (`size > PAYLOAD`): the request doesn't fit in one
///      page's payload at all. Get `ceil((size+HEADER)/PAGE)` fresh pages
///      from `__page_get`, link the span onto this trio's page list, and
///      return the payload start — WITHOUT touching `g_cur`/`g_lim`: the
///      trio's in-progress page keeps filling from where it left off, and
///      the span is handed out fully consumed (it will never be bumped into
///      again, since nothing else fits in its wake within this same
///      allocation call).
///   3. **Fresh single page**: get one page from `__page_get`, link it onto
///      the page list, and install it as the new cursor/limit before bumping
///      for this allocation — this is also the path a genuinely-empty
///      all-zero boot trio takes on its very first call (`cur + size <= 0` is
///      false for any `size > 0`, so the fast path is correctly skipped, and
///      `g_page == 0` linked as this page's `.next` correctly terminates the
///      list at its first entry).
///
/// The caller (`lower_lir_to_wasm`) appends the trailing `Instruction::End`
/// uniformly for every function, so this does not emit one itself.
fn emit_bump_body(func: &mut Function, g_page: u32, g_cur: u32, g_lim: u32, page_get: u32) {
    // fast path: cur = g_cur; if cur+size <= g_lim { g_cur = cur+size; return cur }
    func.instruction(&Instruction::GlobalGet(g_cur));
    func.instruction(&Instruction::LocalTee(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalGet(g_lim));
    func.instruction(&Instruction::I32LeU);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_cur));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // span path: size > PAYLOAD
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(PAYLOAD));
    func.instruction(&Instruction::I32GtU);
    func.instruction(&Instruction::If(BlockType::Empty));
    //   n = (size + HEADER + PAGE - 1) / PAGE ; p = __page_get(n)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(HEADER + PAGE - 1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32DivU);
    func.instruction(&Instruction::Call(page_get));
    func.instruction(&Instruction::LocalSet(2));
    //   p.next = g_page; g_page = p; return p + HEADER  (cursor/limit untouched:
    //   the previous page keeps filling; the span is fully consumed)
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalGet(g_page));
    func.instruction(&Instruction::I32Store(page_mem_arg(0)));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalSet(g_page));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // fresh single page: p = __page_get(1); link; install cursor/limit; return p+HEADER
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::Call(page_get));
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalGet(g_page));
    func.instruction(&Instruction::I32Store(page_mem_arg(0)));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalSet(g_page));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_cur));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_lim));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    // falls through as the function result
}

/// Body of the synthetic `__page_get(pages: i32) -> i32` page supplier: the
/// thing both `__alloc` and `__alloc_global` (via `emit_bump_body`) fall back
/// to once their own cursor can't satisfy a request in-place. Returns a page
/// base pointer to a freshly-claimed contiguous page range, `n` pages long
/// (`span_pages == n` is recorded on that first page only).
///
/// Two disjoint sources, tried in order:
///   1. **Free-list pop**: if the shared free list's head (`g7`) has
///      `span_pages >= n`, pop it. An exact-size match (`span == n`) unlinks
///      the head outright; an oversized match is split — the caller gets the
///      first `n` pages (`head`, `span` forced to `n`) and the remainder
///      (`head + n*PAGE`, `span - n`) goes back onto the free list in
///      `head`'s place. Only the free list's HEAD is ever consulted (no
///      search across multiple free-list nodes for a better fit); if it
///      can't satisfy `n` this always falls through to the frontier below,
///      which is always correct, just possibly wasteful of a previously-freed
///      range that a search further down the list could have used.
///   2. **Frontier + geometric grow**: carve `n` fresh pages off `__heap`
///      (g0, the page frontier — see its updated doc comment where it's
///      declared), growing linear memory first if needed. This re-houses the
///      Phase-0 `__alloc` growth logic verbatim (same `max(deficit_pages,
///      cur_pages)` geometric sizing, same `Unreachable`-on-`memory.grow ==
///      -1` trap), just measured in whole pages (`need = n * PAGE`) instead
///      of an arbitrary byte count.
///
/// Locals: 0 = `pages` (`n`) param; 1 = `head` in the free-list branch,
/// REUSED as `cur_pages` in the frontier branch (their lifetimes never
/// overlap: the free-list branch always returns before the frontier branch
/// runs); 2 = `base` (the page pointer being carved/returned, live across the
/// whole frontier branch); 3 = `need` (`n * PAGE`); 4 = `deficit_pages`.
/// `new_top` is never stored to a local — cheap enough (`base + need`, two
/// `LocalGet`s and an `I32Add`) to recompute at each of its three uses
/// instead of keeping a fifth local live across the branch.
fn emit_page_get_body(func: &mut Function) {
    // ---- Free-list fast path: g7 != 0 && head.span >= n ----
    //
    // Task 6 finding (documented here since this function is Task 5's, not
    // Task 6's, but the gap it closes was only DISCOVERED by Task 6's first
    // real exercise of page recycling): the ORIGINAL Task-5 version of this
    // branch only ever matched `n == 1`, so a multi-page span (`n > 1`, e.g.
    // any `new Array(n)` past `PAYLOAD` bytes) returned to the free list by
    // `__arena_reset` could NEVER be popped back off it — every subsequent
    // same-size span request fell through to the frontier/grow path and grew
    // linear memory further, unboundedly, regardless of how correctly a loop
    // arena around it opened/reset/released. This generalizes the same
    // pop-or-split logic from "`span == 1`" to "`span >= n`" (splitting off
    // exactly `n` pages and returning any remainder to the free list in the
    // popped node's place) — behavior-IDENTICAL to before for every `n == 1`
    // call site (every existing fixture/test only ever requests single
    // pages except the span path itself), and additionally correct for
    // `n > 1`: a page range popped from the free list was legitimately
    // freed by an earlier `__arena_reset`, so reusing it is exactly as sound
    // as the `n == 1` case.
    func.instruction(&Instruction::GlobalGet(7));
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::I32Ne);
    func.instruction(&Instruction::If(BlockType::Empty));
    {
        func.instruction(&Instruction::GlobalGet(7));
        func.instruction(&Instruction::LocalSet(1)); // head = g7 (no stray value left on the stack)

        // if head.span >= n
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I32Load(page_mem_arg(4)));
        func.instruction(&Instruction::LocalGet(0)); // n
        func.instruction(&Instruction::I32GeU);
        func.instruction(&Instruction::If(BlockType::Empty));
        {
            // if head.span == n { g7 = head.next; return head }
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::I32Load(page_mem_arg(4)));
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32Eq);
            func.instruction(&Instruction::If(BlockType::Empty));
            func.instruction(&Instruction::LocalGet(1)); // head.next value source addr
            func.instruction(&Instruction::I32Load(page_mem_arg(0)));
            func.instruction(&Instruction::GlobalSet(7));
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::Return);
            func.instruction(&Instruction::End);

            // span > n: split off the first n pages; return the remainder to
            // the free list. rem = head + n*PAGE
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::LocalGet(0)); // n
            func.instruction(&Instruction::I32Const(PAGE));
            func.instruction(&Instruction::I32Mul);
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalSet(2)); // base(local2) reused as `rem`, no residue

            // rem.next = head.next
            func.instruction(&Instruction::LocalGet(2)); // addr = rem
            func.instruction(&Instruction::LocalGet(1)); // head
            func.instruction(&Instruction::I32Load(page_mem_arg(0)));
            func.instruction(&Instruction::I32Store(page_mem_arg(0)));

            // rem.span = head.span - n
            func.instruction(&Instruction::LocalGet(2)); // addr = rem
            func.instruction(&Instruction::LocalGet(1)); // head
            func.instruction(&Instruction::I32Load(page_mem_arg(4)));
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32Sub);
            func.instruction(&Instruction::I32Store(page_mem_arg(4)));

            // g7 = rem
            func.instruction(&Instruction::LocalGet(2));
            func.instruction(&Instruction::GlobalSet(7));

            // head.span = n
            func.instruction(&Instruction::LocalGet(1)); // addr = head
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32Store(page_mem_arg(4)));

            // return head
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::Return);
        }
        func.instruction(&Instruction::End); // end "head.span >= n" check
    }
    func.instruction(&Instruction::End); // end free-list branch

    // ---- Frontier + geometric grow path ----
    // base = g0; need = n * PAGE
    func.instruction(&Instruction::GlobalGet(0));
    func.instruction(&Instruction::LocalSet(2)); // base
    func.instruction(&Instruction::LocalGet(0)); // n
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32Mul);
    func.instruction(&Instruction::LocalSet(3)); // need

    // cur_pages = memory.size (reuses local1 — dead after the free-list branch)
    func.instruction(&Instruction::MemorySize(0));
    func.instruction(&Instruction::LocalSet(1));

    // if (base + need) > cur_pages * PAGE { grow } — Phase-0 logic moved
    // verbatim from the old `__alloc`, measured against `base + need`
    // instead of `__heap + size`.
    func.instruction(&Instruction::LocalGet(2)); // base
    func.instruction(&Instruction::LocalGet(3)); // need
    func.instruction(&Instruction::I32Add); // new_top (not stored)
    func.instruction(&Instruction::LocalGet(1)); // cur_pages
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32Mul);
    func.instruction(&Instruction::I32GtU);
    func.instruction(&Instruction::If(BlockType::Empty));
    {
        // deficit_pages = ceil((new_top - cur_pages*PAGE) / PAGE)
        func.instruction(&Instruction::LocalGet(2));
        func.instruction(&Instruction::LocalGet(3));
        func.instruction(&Instruction::I32Add); // new_top (recomputed)
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I32Const(PAGE));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::I32Const(PAGE - 1));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(PAGE));
        func.instruction(&Instruction::I32DivU);
        func.instruction(&Instruction::LocalSet(4)); // deficit_pages

        // grow_pages = max(deficit_pages, cur_pages) — geometric: at least doubling.
        func.instruction(&Instruction::LocalGet(4));
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I32GtU);
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        func.instruction(&Instruction::LocalGet(4));
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::End); // end grow_pages select

        // memory.grow(grow_pages); if it reports failure (-1), trap cleanly
        // rather than let the subsequent store go wild out-of-bounds.
        func.instruction(&Instruction::MemoryGrow(0));
        func.instruction(&Instruction::I32Const(-1));
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(BlockType::Empty));
        func.instruction(&Instruction::Unreachable);
        func.instruction(&Instruction::End); // end grow-failure check
    }
    func.instruction(&Instruction::End); // end growth-needed check

    // g0 = base + need
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(0));

    // base.next = 0
    func.instruction(&Instruction::LocalGet(2)); // addr = base
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::I32Store(page_mem_arg(0)));

    // base.span = n
    func.instruction(&Instruction::LocalGet(2)); // addr = base
    func.instruction(&Instruction::LocalGet(0)); // n (param, still valid)
    func.instruction(&Instruction::I32Store(page_mem_arg(4)));

    // return base (falls through as the function result)
    func.instruction(&Instruction::LocalGet(2));
}

/// Body of the synthetic `__arena_reset() -> ()` function/loop-arena
/// recycler: walks the current arena's page list (`g1`) onto the shared free
/// list (`g7`), then zeros the current-arena trio (`g1`/`g2`/`g3`) so the
/// next allocation from that arena starts fresh. Not yet called from
/// anywhere in this task (`ArenaTable` is still unconsumed by codegen; loop
/// and function arenas that call this land in Tasks 6-7) — Task 5 only
/// builds and unit-tests this machinery in isolation.
///
/// Locals: 0 = `p`, 1 = `next` (no params).
fn emit_arena_reset_body(func: &mut Function) {
    // p = g1
    func.instruction(&Instruction::GlobalGet(1));
    func.instruction(&Instruction::LocalSet(0));

    func.instruction(&Instruction::Block(BlockType::Empty));
    func.instruction(&Instruction::Loop(BlockType::Empty));
    {
        // if p == 0 { break out of the block, ending the loop }
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::I32Eqz);
        func.instruction(&Instruction::BrIf(1));

        // next = p.next
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::I32Load(page_mem_arg(0)));
        func.instruction(&Instruction::LocalSet(1));

        // p.next = g7
        func.instruction(&Instruction::LocalGet(0)); // addr = p
        func.instruction(&Instruction::GlobalGet(7));
        func.instruction(&Instruction::I32Store(page_mem_arg(0)));

        // g7 = p
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::GlobalSet(7));

        // p = next
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::LocalSet(0));

        func.instruction(&Instruction::Br(0)); // continue loop
    }
    func.instruction(&Instruction::End); // end loop
    func.instruction(&Instruction::End); // end block

    // g1 = 0; g2 = 0; g3 = 0
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::GlobalSet(1));
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::GlobalSet(2));
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::GlobalSet(3));
}

/// `__substring(h, s, e) -> i64`: zero-copy slice of a tagged string handle.
/// Locals: 0 = h, 1 = s, 2 = e (params), 3 = swap temp.
/// len is recomputed from `h` (2 instructions) rather than stored.
fn emit_substring_body(func: &mut Function) {
    // s = max(s, 0)
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(1));
    // s = min(s, len)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(1));
    // e = max(e, 0)
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(2));
    // e = min(e, len)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(2));
    // if s > e { t = s; s = e; e = t }   (JS substring swaps its bounds)
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalSet(1));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::End);
    // TAG | (off + s) << 32 | (e - s)   where off = (h >> 32) & 0x7FFF_FFFF
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Or);
}

/// `__join(arr, sep) -> i64`: copy every element string (i64 handles in the
/// array's slots) plus `sep` between them into ONE fresh __alloc_global
/// buffer; return `TAG | out<<32 | total`. Empty array returns bare TAG
/// (offset 0, len 0 — a zero-length handle is never dereferenced).
/// Locals: 0=arr 1=sep (params), 2=n 3=i 4=total 5=out 6=cur 7=h.
fn emit_join_body(func: &mut Function, alloc_global_index: u32) {
    // n = *(arr + 0)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(2));
    // if n == 0 return TAG
    // (Explicit `n == 0` via I64Const(0) + I64Eq rather than I64Eqz: this
    // synthetic's body is NOT excluded from
    // `control_flow_tests::pipeline_basics::boolean_branches_use_the_layout_fast_path`,
    // a whole-module printed-text assertion elsewhere in this crate that a
    // specialized boolean fast path never needs `i64.eqz` — since `__join` is
    // present in every module, any `I64Eqz` it emits would trip that
    // assertion for unrelated programs. `I64Eq` is semantically identical
    // and already used elsewhere in this crate for `==`/`===` lowering.)
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // total = 0; i = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // pass 1: total += len(elem_i) for each i
    func.instruction(&Instruction::Loop(BlockType::Empty));
    //   h = *(arr + (i<<3) + 8)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(7));
    //   total = total + (h & 0xFFFF_FFFF)
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    //   i += 1; continue while i < n
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);
    // total += (sep & 0xFFFF_FFFF) * (n - 1)
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    // out = zext(__alloc_global(wrap((total + 7) & !7)))
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(7));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Const(-8)); // !7 as two's-complement
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(5));
    // cur = out; i = 0
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalSet(6));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // pass 2: copy elements, separator between them
    func.instruction(&Instruction::Loop(BlockType::Empty));
    //   h = *(arr + (i<<3) + 8)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(7));
    //   memory.copy(dst=cur, src=(h>>32)&0x7FFF_FFFF, len=h&0xFFFF_FFFF)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
    //   cur += len(h)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(6));
    //   i += 1
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    //   if i < n { copy separator; continue }
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    //     memory.copy(dst=cur, src=sep off, len=sep len) — zero-len is a legal no-op
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
    //     cur += sep_len
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(6));
    //     continue the loop (br 1: label 0 = this If, label 1 = the Loop)
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // If
    func.instruction(&Instruction::End); // Loop — falls through when i == n
                                         // TAG | out << 32 | total
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Or);
    // NO trailing End — the dispatch loop appends it (lower.rs:631).
}

pub(crate) fn top_level_children(lir: &LirProgram) -> Vec<LirNodeId> {
    let mut children = Vec::new();
    if let Some(root) = lir.nodes.get(lir.root.0 as usize) {
        for child in &root.children {
            if !is_function_like(&lir.nodes, *child) {
                children.push(*child);
            }
        }
    }
    children
}

pub(crate) fn emit_literal(
    function: &mut Function,
    text: Option<&str>,
    strings: &mut StringPool,
) -> EmittedValue {
    match text {
        Some("true") => {
            function.instruction(&Instruction::I64Const(1));
            EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            }
        }
        Some("false") => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            }
        }
        Some("null") | Some("undefined") => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
        }
        Some(text) => {
            if let Some(number) = parse_number_literal(text) {
                // Exact integer numeric literal: keep the i64 constant path.
                function.instruction(&Instruction::I64Const(number));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            } else if let Some(value) =
                parse_numeric_literal_value(text).filter(|v| v.is_finite() && v.fract() != 0.0)
            {
                // Finite non-integer numeric literal (fractional part or
                // exponent): emit an f64 constant. The repr inference seeds
                // these as float (see `is_float_literal` in kali_types), so
                // float locals/params expect an f64 here; a mis-typed i64
                // string handle otherwise yields an invalid module (E4201).
                //
                // Non-finite values (NaN/Infinity) are never real numeric
                // source-literal tokens — they are identifiers — so the only
                // such text reaching here is a codegen-synthesized string-print
                // artifact (e.g. an out-of-range `charCodeAt` rendering "NaN").
                // Those keep the existing string-interning path unchanged.
                function.instruction(&Instruction::F64Const(value.into()));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Float,
                }
            } else {
                let normalized = strip_string_delimiters(text);
                let (offset, len) = strings.intern(normalized);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
        }
        None => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
        }
    }
}

pub(crate) fn encode_string_handle(offset: u32, len: u32) -> i64 {
    (crate::STRING_HANDLE_TAG | ((offset as u64) << 32) | u64::from(len)) as i64
}
