//! Program-level driver and LIR-walking analysis functions.
use crate::*;

/// Maps a representation decision to the wasm value type used for the matching
/// param/result/local slot. `I64` is the integer default; `F64` is an IEEE double.
pub(crate) fn wasm_type(repr: kali_common::Repr) -> wasm_encoder::ValType {
    match repr {
        kali_common::Repr::F64 => wasm_encoder::ValType::F64,
        kali_common::Repr::I64 | kali_common::Repr::Object(_) => wasm_encoder::ValType::I64,
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

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, lir: &LirProgram) -> CodegenResult {
    let mut diagnostics = Vec::new();
    let function_plans = collect_functions(lir, &ctx.repr_table);
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
    let mut all_functions = Vec::new();
    all_functions.push(FunctionPlan {
        name: "_start".to_string(),
        params: Vec::new(),
        locals: collect_function_locals(&lir.nodes, lir.root, &ctx.repr_table, "_start"),
        body: lir.root,
        result: false,
        is_entry: true,
        flavor: None,
    });
    // Synthetic bump allocator `__alloc(size: i32) -> i32`, occupying a fixed
    // slot right after `_start` and before any named (source-defined)
    // function. Its body is hand-emitted by `emit_alloc_body` below, not
    // lowered from LIR — `body` is unused (set to `lir.root` as an inert
    // placeholder) and `locals`/`flavor` are left at their inert defaults.
    // Object/array allocation sites resolve its index through
    // `function_name_to_index["__alloc"]` (see `FunctionEmitter::alloc_fn_index`)
    // exactly like any other named function, so inserting it here shifts every
    // later function's index by exactly one — safe because every call site in
    // this crate resolves callee indices through that same map (verified: the
    // only hardcoded `Instruction::Call(..)` sites are fixed *import* indices,
    // which live in a separate index space unaffected by `all_functions`).
    all_functions.push(FunctionPlan {
        name: "__alloc".to_string(),
        params: vec!["size".to_string()],
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
        // `__alloc` is not a repr-directed user function (it has no `ReprTable`
        // entries at all), so its `(i32) -> i32` signature is fixed here rather
        // than derived from `ctx.repr_table.param`/`return_repr`, which would
        // otherwise default it to `(i64) -> i64`.
        let (params, results) = if function.name == "__alloc" {
            (vec![ValType::I32], vec![ValType::I32])
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
        let mut local_decls: Vec<(u32, ValType)> = Vec::new();
        for local_name in &function.locals {
            let val_type = wasm_type(ctx.repr_table.scalar(&function.name, local_name));
            match local_decls.last_mut() {
                Some((count, last_type)) if *last_type == val_type => *count += 1,
                _ => local_decls.push((1, val_type)),
            }
        }
        match local_decls.last_mut() {
            Some((count, ValType::I64)) => *count += 2,
            _ => local_decls.push((2, ValType::I64)),
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
            &function.name,
            &module_const_inits,
            &module_binding_names,
        );
        let coverage_id = ctx.target.coverage.then_some(coverage_id as u32);
        if function.name == "__alloc" {
            // Hand-emitted: not lowered from LIR (there is no source-level
            // function body for the synthetic allocator), and deliberately
            // uninstrumented (no `emit_coverage_hit`) since it is not a
            // source-defined function.
            emit_alloc_body(&mut body);
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

    // Heap base: first 8-aligned byte after interned string data. The `__heap` bump
    // pointer starts here and grows upward as `new Array(n)` allocations are made.
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
        module.section(&CustomSection {
            name: Cow::Borrowed("kali:coverage"),
            data: Cow::Owned((all_functions.len() as u32).to_le_bytes().to_vec()),
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
) -> Vec<FunctionPlan> {
    let mut plans = Vec::new();
    let mut visited = HashSet::new();
    collect_functions_from_node(lir, lir.root, &mut visited, &mut plans, repr_table);
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
) {
    if !visited.insert(id) {
        return;
    }

    if let Some(plan) = function_plan(&lir.nodes, id, repr_table) {
        plans.push(plan);
    }

    let Some(node) = lir.nodes.get(id.0 as usize) else {
        return;
    };

    for child in &node.children {
        collect_functions_from_node(lir, *child, visited, plans, repr_table);
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
) -> Option<FunctionPlan> {
    let (name, flavor, body_id, params) = function_shape(nodes, id)?;
    let locals = collect_function_locals(nodes, body_id, repr_table, &name);

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

pub(crate) fn collect_function_locals(
    nodes: &[LirNode],
    body_id: LirNodeId,
    repr_table: &kali_common::ReprTable,
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
    locals
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

/// Body of the synthetic `__alloc(size: i32) -> i32` bump allocator: `ptr =
/// __heap; __heap = ptr + size; return ptr` — byte-for-byte the same
/// computation every inline bump-allocation site used to perform. Local 0 is
/// the `size` param; the caller (`lower_lir_to_wasm`) appends the trailing
/// `Instruction::End` uniformly for every function, so this does not emit one
/// itself. Phase 0 only (no `memory.grow` check yet); Task 3 inserts that
/// check ahead of the final `GlobalSet`, and this function is the only place
/// it will need to touch.
fn emit_alloc_body(func: &mut Function) {
    // Leave the old `__heap` value on the stack (the function result)...
    func.instruction(&Instruction::GlobalGet(0));
    // ...then advance the global by `size`.
    func.instruction(&Instruction::GlobalGet(0));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(0));
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
