//! `FunctionEmitter` struct, support types, and lifecycle methods.

use super::*;

#[derive(Clone, Debug)]
pub(crate) struct FunctionPlan {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) locals: Vec<String>,
    pub(crate) body: LirNodeId,
    pub(crate) result: bool,
    pub(crate) is_entry: bool,
    pub(crate) flavor: Option<FunctionFlavor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueShape {
    Unknown,
    Scalar,
    Boolean,
    /// A tagged linear-memory string handle (`STRING_HANDLE_TAG | offset << 32 | len`).
    String,
    /// An IEEE-754 double left on the wasm stack as an `f64`.
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectEnumerationMode {
    Keys,
    Values,
    Entries,
    ReflectOwnKeys,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlFlowLabelKind {
    If,
    LoopBreak,
    LoopContinue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LoopFrame {
    pub(crate) break_index: usize,
    pub(crate) continue_index: usize,
}

/// A live per-loop arena scope: the three local slots holding the
/// current-arena trio (`g1`/`g2`/`g3`) as it was *before* this loop opened
/// (restored on release), and the index into `FunctionEmitter::loop_frames`
/// of the loop this arena belongs to (used by `break` to decide whether it is
/// exiting the loop that owns this frame).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArenaFrame {
    pub(crate) saved_page_local: u32,
    pub(crate) saved_cursor_local: u32,
    pub(crate) saved_limit_local: u32,
    pub(crate) loop_frame_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EmittedValue {
    pub(crate) produced: bool,
    pub(crate) shape: ValueShape,
}

pub(crate) struct FunctionEmitter<'a> {
    pub(crate) program: &'a LirProgram,
    pub(crate) node_lookup: &'a [LirNode],
    pub(crate) scratch_nodes: Vec<LirNode>,
    pub(crate) functions: &'a BTreeMap<String, u32>,
    /// Wasm function index -> declared parameter count. Threaded from
    /// `lower.rs` alongside `functions` so an emit arm can gate on a resolved
    /// callback's arity (Stage D event lane: zero-parameter listeners only).
    pub(crate) function_param_counts: &'a BTreeMap<u32, usize>,
    /// Function NAME -> its declared parameter names. Threaded from `lower.rs`
    /// alongside `function_param_counts`; the deferred-callback scalar-capture
    /// deny (Task 9 C-1) consults it to classify a captured binding as a
    /// PARAMETER (deny) vs. a non-scalar placeholder construct (allow).
    pub(crate) function_param_names: &'a BTreeMap<String, Vec<String>>,
    pub(crate) env_set_import_index: Option<u32>,
    pub(crate) env_delete_import_index: Option<u32>,
    pub(crate) env_get_import_index: Option<u32>,
    pub(crate) env_has_import_index: Option<u32>,
    pub(crate) cwd_set_import_index: Option<u32>,
    pub(crate) process_exit_import_index: Option<u32>,
    pub(crate) stdout_write_bytes_import_index: Option<u32>,
    pub(crate) args_get_import_index: Option<u32>,
    pub(crate) performance_now_import_index: Option<u32>,
    pub(crate) crypto_get_random_values_import_index: Option<u32>,
    pub(crate) crypto_random_uuid_import_index: Option<u32>,
    pub(crate) crypto_subtle_digest_import_index: Option<u32>,
    /// Stage D scheduling-surface host import indices — `Some` only when the
    /// matching `program_calls_bare_identifier` probe found a call; appended
    /// after `crypto_subtle_digest` in declaration order (queueMicrotask,
    /// setTimeout, setInterval, clearTimeout, clearInterval).
    pub(crate) queue_microtask_import_index: Option<u32>,
    pub(crate) set_timeout_import_index: Option<u32>,
    pub(crate) set_interval_import_index: Option<u32>,
    pub(crate) clear_timeout_import_index: Option<u32>,
    pub(crate) clear_interval_import_index: Option<u32>,
    /// Stage D event-lane host import indices (`Some` only when the matching
    /// program-wide probe fired; appended after clear_interval).
    pub(crate) event_target_new_import_index: Option<u32>,
    /// Consumed by the Task 4 addEventListener/dispatchEvent emit arms.
    pub(crate) event_listener_add_import_index: Option<u32>,
    pub(crate) event_dispatch_import_index: Option<u32>,
    /// Locals/module bindings with stable provenance to `new EventTarget()`
    /// (declarator-recorded, the `fn_valued_locals` pattern). Reads outside the
    /// lane's allowed positions fail closed (handle-escape discipline, spec
    /// §2.4): the raw i64 handle must never escape as an observable value.
    pub(crate) event_target_locals: BTreeSet<String>,
    pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
    pub(crate) strings: &'a mut StringPool,
    pub(crate) source_path: Option<PathBuf>,
    /// Program-wide representation decisions, consulted for repr-directed
    /// instruction selection and operand promotion.
    pub(crate) repr_table: &'a kali_common::ReprTable,
    /// Arena placement decisions from the `kali_mir` escape gate — read by
    /// `alloc_callee_index` (allocation-site routing) and `emit_loop`
    /// (loop-arena open/reset/release). Misses fail closed (global
    /// allocation / no arena).
    pub(crate) arena_table: &'a kali_common::ArenaTable,
    /// Name of the function currently being emitted, used to key `repr_table`
    /// lookups (the synthetic entry is `_start`).
    pub(crate) function_name: String,
    /// LIR body root of the function currently being emitted. Retained so a
    /// declarator walk (e.g. `binding_is_placeholder_construct`) can inspect
    /// this function's own bindings at emit time.
    pub(crate) body: LirNodeId,
    pub(crate) current_function_flavor: Option<FunctionFlavor>,
    pub(crate) locals: BTreeMap<String, u32>,
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
    /// Local names bound to a function VALUE (`let cb = function(){…}` /
    /// `const cb = () => …`), mapping the binding name to the initializer's
    /// `__kali_fn_N` plan key. Recorded at declaration-emit time (source order,
    /// so the binding is populated before any later use). This is the
    /// binding-PROVENANCE the scheduling-surface guard consults to resolve an
    /// indirectly-passed callback (`setTimeout(cb, 0)`) back to its closure plan
    /// — a callback identifier is name-matched to the fn it was DECLARED to
    /// hold, not guessed. See `call_has_capturing_closure_arg`.
    pub(crate) fn_valued_locals: BTreeMap<String, String>,
    /// Names whose binding provenance is UNSTABLE in this function body:
    /// assigned outside their declarator, or declared by more than one
    /// declarator (block-level shadowing). `fn_valued_locals` is recorded once
    /// at declarator-emit time, so for these names it can go stale; the
    /// scheduling-surface default-deny guard therefore refuses to resolve any
    /// unstable name and fails closed E5506 instead (stage-review IMPORTANT-1;
    /// closes the reassignment-stale-provenance fail-open the tripwire pinned).
    /// Computed structurally up front (`crate::lower::unstable_provenance_names`)
    /// so it does not depend on emission order.
    pub(crate) unstable_provenance_names: HashSet<String>,
    /// Names of locals that hold a linear-memory array handle (`new Array(n)`).
    pub(crate) array_bindings: HashSet<String>,
    /// Names of locals that hold a GROWABLE runtime-array tagged handle
    /// (throw-fallout Stage 4) — the codegen half of the growable both-sides
    /// oracle, populated at construction from
    /// `repr_table.is_growable_array_binding` for every param/local name.
    /// DISJOINT from `array_bindings` (a separate tagged-header layout; the
    /// two lanes must never conflate).
    pub(crate) growable_array_bindings: HashSet<String>,
    /// `Some(<iterated binding name>)` while emitting the body of a runtime
    /// `for..of` over a growable array (throw-fallout Stage 4 Task 4). Two
    /// fail-closed guards key on it: (1) a growable `for..of` lexically NESTED
    /// inside another rejects E5506 — the shared index/length scratch pair
    /// (`growable_foreach_index_local_name`) would otherwise be clobbered by
    /// the inner loop and silently miscompile the outer counter; (2) a `.push`
    /// on the SAME binding being iterated rejects E5506 in
    /// `emit_growable_push_call` — the by-construction mirror of the
    /// resolve-time self-push reject (node grows the iteration; the counted
    /// loop's once-snapshotted length does not). Per-function scoped (fresh
    /// emitter per function), so a growable `for..of` in a nested FUNCTION is
    /// a separate emitter and never blocked.
    pub(crate) growable_for_of_active: Option<String>,
    pub(crate) reported_placeholder_fallbacks: HashSet<String>,
    pub(crate) control_frames: Vec<ControlFlowLabelKind>,
    pub(crate) loop_frames: Vec<LoopFrame>,
    /// Pre-order loop ordinal of every loop-shaped node in this function's
    /// body (see `crate::lower::loop_preorder_ordinals`), resolved once at
    /// construction time and consulted by `emit_loop` to key `arena_table`
    /// and the `__arena_save_*` locals.
    pub(crate) loop_ordinals: HashMap<LirNodeId, u32>,
    /// Pre-order ordinal of every string-producing site (`.join(..)` call or
    /// `+` binary) in this function's body (see
    /// `crate::lower::string_site_preorder_ordinals`), resolved once at
    /// construction time — the codegen half of the string-site "both-sides
    /// oracle". `emit_runtime_join` consults it to key
    /// `arena_table.arena_string_site` and select the `__join_arena` twin.
    /// Threaded exactly like `loop_ordinals`: reset per function (each function
    /// gets a fresh emitter/body), so ordinals never leak across functions.
    pub(crate) string_site_ordinals: HashMap<LirNodeId, u32>,
    /// Pre-order ordinal of every `for-in` node in this function's body (see
    /// `crate::lower::for_in_preorder_ordinals`), resolved once at
    /// construction time and consulted by `emit_for_in` to find its dedicated
    /// counter local (`crate::lower::for_in_ord_local_name`). Wholly separate
    /// from `loop_ordinals`/`arena_table` — never consulted for arena
    /// placement.
    pub(crate) for_in_ordinals: HashMap<LirNodeId, u32>,
    /// `for..in` key binding name → the fixed shape its object enumerates,
    /// recorded by `emit_for_in` BEFORE it emits the loop body (Spec 4a Task
    /// 3). The codegen-side mirror of `kali_types`'s `for_in_key_bindings`
    /// registry: the computed-key access recognizer
    /// (`computed_forin_object_access`) consults it so `obj[c]` lowers to a
    /// dynamic field slot only when `c` is a for..in key over `obj`'s shape,
    /// never a same-named static field. Grow-only within this function's
    /// emitter (each function gets a fresh emitter, so keys never leak across
    /// functions).
    pub(crate) for_in_key_shapes: HashMap<String, kali_common::ShapeId>,
    /// Per-function structural set of "for-in-key provenance" binding names:
    /// every `for..in` loop key declared in this function, plus every binding
    /// aliased directly from such a key (`last = c`) — the Spec 4a Task 4
    /// null-sentinel alias family. Computed ONCE up front (unlike
    /// `for_in_key_shapes`, which is populated during emission) so the
    /// `var last = null` init — emitted BEFORE the loop — already recognizes
    /// `last`. The null-sentinel (`-1`) store and the truthiness (`>= 0`)
    /// lowering key off this set. Structural twin of the types-side `for..in`
    /// key + `last = c` provenance propagation (mirror binding provenance, not
    /// repr — the Spec-3 lesson).
    pub(crate) for_in_key_aliases: HashSet<String>,
    /// Spec 4a Task 5 / fasta Spec 7 Task 4g: per-for-in-key handle-table base
    /// OFFSETS. `name -> base` where `base` is the compile-time data-segment
    /// offset (an `i32.const`, NOT a runtime local) of an `N*8`-byte table of
    /// interned field-name string handles (slot `j` = the interned handle of the
    /// shape's `j`th field). The table is MODULE-CONSTANT data, interned once per
    /// shape via `StringPool::intern_key_table` — zero runtime allocation
    /// (previously it was bump-allocated on every for-in execution, which leaked
    /// for a for-in nested in a loop). A for-in key (or alias) emitted in a
    /// STRING-VALUE context materializes its field-name string by loading
    /// `base + ord*8` from this table instead of yielding the raw ordinal. The
    /// interned handles are compile-time data-segment offsets, so the table only
    /// stores immutable `i64` handle values — the strings NEVER dangle.
    /// Registered for the loop key AND its aliases (they enumerate the same
    /// shape, so one table serves all); materialization is additionally gated on
    /// the name's scalar repr being `String`, so an alias used only as an ordinal
    /// (`table[last]`) is never wrongly materialized.
    pub(crate) for_in_key_handle_tables: HashMap<String, u32>,
    /// Stack of currently-open loop arenas (innermost last) — see
    /// `emitter::ArenaFrame`.
    pub(crate) arena_frames: Vec<ArenaFrame>,
    /// Top-level `const name → init node` table (all `FunctionEmitter`s share
    /// one `LirProgram` node space, so a compile-time-pure module const's
    /// initializer node can be re-emitted at each function read site).
    pub(crate) module_const_inits: &'a BTreeMap<String, LirNodeId>,
    /// ALL top-level binding names (const, let, var), used to gate reads of
    /// module bindings that are not inlinable pure consts.
    pub(crate) module_binding_names: &'a BTreeSet<String>,
    /// Module-scope mutable SCALAR (`var`/`let` numeric) bindings promoted to
    /// persistent mutable WASM globals: `name -> (global_index, repr)`. A read
    /// of such a name (from a function OR module scope) lowers to `GlobalGet`; a
    /// write to `GlobalSet`; the declarator init in `_start` stores through
    /// `GlobalSet`. See `kali_codegen::lower::collect_module_scalar_globals`.
    pub(crate) module_global_slots: &'a BTreeMap<String, (u32, kali_common::Repr)>,
    /// This function's closure environment plan (Stage C, `derive_env_plans`):
    /// the promoted scalar/heap cells it OWNS in its own env record and the
    /// outer bindings it captures through the parent chain. Default (owns_env
    /// false, no cells/captures) for every closure-free function, so integer
    /// programs never touch `CURRENT_ENV_GLOBAL`. Cell names that are promoted
    /// locals are removed from `locals` before construction (they get no WASM
    /// local slot — the cell IS their storage); a cell name still present in
    /// `locals` is therefore a captured PARAMETER, which C1 does not lower
    /// (rejected in the prologue).
    pub(crate) env_plan: kali_mir::EnvPlan,
    /// ALL functions' closure environment plans, keyed by `__kali_fn_N` name
    /// (the whole `derive_env_plans` map). Unlike `env_plan` (this function's
    /// own plan), this lets a call site inspect ANOTHER function's plan — used
    /// to detect whether a callback argument CAPTURES an enclosing env cell
    /// (`!captured.is_empty()`) when it is passed to an un-emittable scheduling
    /// surface (Stage C Concern 2 fail-closed guard, `emit_call`).
    pub(crate) env_plans: &'a std::collections::BTreeMap<String, kali_mir::EnvPlan>,
}

impl<'a> FunctionEmitter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        program: &'a LirProgram,
        functions: &'a BTreeMap<String, u32>,
        function_param_counts: &'a BTreeMap<u32, usize>,
        function_param_names: &'a BTreeMap<String, Vec<String>>,
        env_set_import_index: Option<u32>,
        env_delete_import_index: Option<u32>,
        env_get_import_index: Option<u32>,
        env_has_import_index: Option<u32>,
        cwd_set_import_index: Option<u32>,
        process_exit_import_index: Option<u32>,
        stdout_write_bytes_import_index: Option<u32>,
        args_get_import_index: Option<u32>,
        performance_now_import_index: Option<u32>,
        crypto_get_random_values_import_index: Option<u32>,
        crypto_random_uuid_import_index: Option<u32>,
        crypto_subtle_digest_import_index: Option<u32>,
        queue_microtask_import_index: Option<u32>,
        set_timeout_import_index: Option<u32>,
        set_interval_import_index: Option<u32>,
        clear_timeout_import_index: Option<u32>,
        clear_interval_import_index: Option<u32>,
        event_target_new_import_index: Option<u32>,
        event_listener_add_import_index: Option<u32>,
        event_dispatch_import_index: Option<u32>,
        diagnostics: &'a mut Vec<Diagnostic>,
        strings: &'a mut StringPool,
        source_path: Option<PathBuf>,
        current_function_flavor: Option<FunctionFlavor>,
        params: &[String],
        local_names: &[String],
        repr_table: &'a kali_common::ReprTable,
        arena_table: &'a kali_common::ArenaTable,
        function_name: &str,
        body: LirNodeId,
        module_const_inits: &'a BTreeMap<String, LirNodeId>,
        module_binding_names: &'a BTreeSet<String>,
        module_global_slots: &'a BTreeMap<String, (u32, kali_common::Repr)>,
        env_plan: kali_mir::EnvPlan,
        env_plans: &'a std::collections::BTreeMap<String, kali_mir::EnvPlan>,
    ) -> Self {
        let loop_ordinals = crate::lower::loop_preorder_ordinals(&program.nodes, body);
        let string_site_ordinals =
            crate::lower::string_site_preorder_ordinals(&program.nodes, body);
        let for_in_ordinals = crate::lower::for_in_preorder_ordinals(&program.nodes, body);
        let for_in_key_aliases = crate::lower::for_in_key_alias_names(&program.nodes, body);
        let unstable_provenance_names =
            crate::lower::unstable_provenance_names(&program.nodes, body);
        let mut locals = BTreeMap::new();
        for (idx, name) in params.iter().enumerate() {
            locals.insert(name.clone(), idx as u32);
        }
        for (offset, name) in local_names.iter().enumerate() {
            locals.insert(name.clone(), (params.len() + offset) as u32);
        }

        // Register array PARAMETERS as array bindings. `new Array(...)`
        // declarators register themselves at their declaration site, but a param
        // has no declarator in this body — its i64 handle simply arrives in the
        // param slot. The inference already knows which params are arrays
        // (subscripted, `.length`/`.fill`, or a pass-through array param); mark
        // them here so the existing base-address emission, repr-directed
        // element load/store, `.length`, and `.fill` paths fire. Scalar params
        // are left untouched, so integer programs are byte-identical.
        let mut array_bindings = HashSet::new();
        for name in params {
            if repr_table.is_array_binding(function_name, name) {
                array_bindings.insert(name.clone());
            }
        }

        // Register GROWABLE array bindings (throw-fallout Stage 4) for every
        // param/local name the repr inference promoted. Unlike the plain
        // `array_bindings` set above (params only; declarators register at
        // their declaration site), growable membership is decided entirely by
        // the repr table (the types-side promotion), so both params and
        // declarator locals are seeded here — declarator emission relies on
        // membership BEFORE its own statement is reached (locals collection
        // promoted the binding to a slot for the same reason).
        let mut growable_array_bindings = HashSet::new();
        for name in params.iter().chain(local_names.iter()) {
            if repr_table.is_growable_array_binding(function_name, name) {
                growable_array_bindings.insert(name.clone());
            }
        }

        Self {
            program,
            node_lookup: &program.nodes,
            scratch_nodes: Vec::new(),
            functions,
            function_param_counts,
            function_param_names,
            env_set_import_index,
            env_delete_import_index,
            env_get_import_index,
            env_has_import_index,
            cwd_set_import_index,
            process_exit_import_index,
            stdout_write_bytes_import_index,
            args_get_import_index,
            performance_now_import_index,
            crypto_get_random_values_import_index,
            crypto_random_uuid_import_index,
            crypto_subtle_digest_import_index,
            queue_microtask_import_index,
            set_timeout_import_index,
            set_interval_import_index,
            clear_timeout_import_index,
            clear_interval_import_index,
            event_target_new_import_index,
            event_listener_add_import_index,
            event_dispatch_import_index,
            event_target_locals: BTreeSet::new(),
            diagnostics,
            strings,
            source_path,
            repr_table,
            arena_table,
            function_name: function_name.to_string(),
            body,
            current_function_flavor,
            locals,
            bindings: BTreeMap::new(),
            fn_valued_locals: BTreeMap::new(),
            unstable_provenance_names,
            array_bindings,
            growable_array_bindings,
            growable_for_of_active: None,
            reported_placeholder_fallbacks: HashSet::new(),
            control_frames: Vec::new(),
            loop_frames: Vec::new(),
            loop_ordinals,
            string_site_ordinals,
            for_in_ordinals,
            for_in_key_shapes: HashMap::new(),
            for_in_key_aliases,
            for_in_key_handle_tables: HashMap::new(),
            arena_frames: Vec::new(),
            module_const_inits,
            module_binding_names,
            module_global_slots,
            env_plan,
            env_plans,
        }
    }

    /// True when `id` is a compile-time-pure expression: a numeric literal, an
    /// identifier of another pure module const (recursively, cycle-bounded),
    /// or unary/binary arithmetic over pure operands. Guards module-const
    /// inlining — an impure initializer (call, member access, allocation)
    /// must not be re-emitted per use site.
    pub(crate) fn is_pure_module_const_init(&self, id: LirNodeId, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        let id = self.unwrap_transparent(id);
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value if node.children.is_empty() => {
                let Some(text) = node.text.as_deref() else {
                    return false;
                };
                parse_numeric_literal_value(text).is_some()
                    || self
                        .module_const_inits
                        .get(text)
                        .is_some_and(|&init| self.is_pure_module_const_init(init, depth + 1))
            }
            LirNodeKind::Value
                if node.children.len() == 2
                    && is_binary_operator_text(node.text.as_deref().unwrap_or_default()) =>
            {
                self.is_pure_module_const_init(node.children[0], depth + 1)
                    && self.is_pure_module_const_init(node.children[1], depth + 1)
            }
            LirNodeKind::Value
                if node.children.len() == 1
                    && matches!(node.text.as_deref(), Some("-") | Some("+")) =>
            {
                self.is_pure_module_const_init(node.children[0], depth + 1)
            }
            _ => false,
        }
    }

    /// Representation chosen for scalar binding `name` in the current function.
    pub(crate) fn scalar_repr(&self, name: &str) -> kali_common::Repr {
        self.repr_table.scalar(&self.function_name, name)
    }

    /// Representation chosen for the elements of array binding `name` in the
    /// current function.
    pub(crate) fn array_elem_repr(&self, name: &str) -> kali_common::Repr {
        self.repr_table.array_element(&self.function_name, name)
    }

    /// True when `name` is a GROWABLE runtime-array binding of the current
    /// function (throw-fallout Stage 4) — the codegen half of the growable
    /// both-sides oracle, mirroring `repr_table.is_growable_array_binding`.
    pub(crate) fn is_growable_array(&self, name: &str) -> bool {
        self.growable_array_bindings.contains(name)
    }

    /// Wasm function index of the allocator an allocation site in the
    /// CURRENTLY-EMITTING function should call: `__alloc` (the current
    /// arena) when the escape gate marked this function `arena_eligible`,
    /// else `__alloc_global` — fail-closed default for anything the gate
    /// didn't (or couldn't) grant. This is a per-FUNCTION gate (every
    /// allocation call site lexically inside a given function shares its
    /// verdict), not a per-loop one — whether any of those sites also sit
    /// inside a loop that itself opens/resets/releases an iteration arena is
    /// decided separately by `emit_loop`.
    pub(crate) fn alloc_callee_index(&self) -> u32 {
        if self.arena_table.arena_eligible(&self.function_name) {
            self.functions["__alloc"]
        } else {
            self.functions["__alloc_global"]
        }
    }

    /// Wasm function index of the persistent global allocator `__alloc_global`
    /// (the never-reset g4/g5/g6 arena twin of `__alloc`). Used for host-runtime
    /// buffers that must outlive any `__arena_reset` — e.g. the `process.argv`
    /// element buffer (Spec 5 Task 5), which backs a string handle that may
    /// escape the current arena. Unconditional (not gated on `arena_eligible`),
    /// unlike `alloc_callee_index`.
    pub(crate) fn alloc_global_fn_index(&self) -> u32 {
        self.functions["__alloc_global"]
    }

    /// Wasm function index of the synthetic `__arena_reset() -> ()` page-pool
    /// recycler (Task 5): walks the current arena's page list onto the
    /// shared free list and zeroes the current-arena trio. Called by
    /// `emit_loop`'s per-iteration reset and by `emit_arena_release`.
    pub(crate) fn arena_reset_fn_index(&self) -> u32 {
        self.functions["__arena_reset"]
    }

    /// Declared parameter count of the compiled function at wasm index
    /// `index`, or `None` if the index names no user/synthetic function
    /// (e.g. a raw import index). Stage D event lane gates listeners on
    /// `Some(0)`.
    pub(crate) fn function_param_count_by_index(&self, index: u32) -> Option<usize> {
        self.function_param_counts.get(&index).copied()
    }

    /// WASM global index of `current_env` (Stage C closures): the active
    /// environment record pointer (i64; 0 = no env). See
    /// `crate::closure::CURRENT_ENV_GLOBAL`.
    pub(crate) fn current_env_global(&self) -> u32 {
        crate::closure::CURRENT_ENV_GLOBAL
    }

    /// True when scalar cell/capture `name` is a C1-promotable env cell in
    /// `owner`'s namespace: a SCALAR cell whose chosen repr is the default `I64`
    /// (the env cell is a raw 8-byte i64 slot with i64 arithmetic; an
    /// `F64`/`String`/`Object`/bool repr would corrupt the value). This is the
    /// SCALAR half of the promotion predicate — the arithmetic write paths
    /// (compound-assign / update) gate on it so a captured OBJECT cell keeps its
    /// baseline write path. The unified read/declaration gate is
    /// [`crate::closure::cell_is_promotable`].
    ///
    /// C1 review Finding 1: a captured cell was promoted (and
    /// thus allocated) by its OWNER, so the capturer must gate on the OWNER's
    /// repr verdict — the SAME predicate `lower.rs` applied
    /// (`repr_table.scalar(&owner.name, &cell.name) == I64`) when it decided to
    /// drop the name from the owner's locals. Gating on the capturer's own
    /// namespace (where an outer name defaults to `I64`) diverges from the
    /// owner's decision: an owner F64 that did NOT promote leaves no cell, yet the
    /// capturer would read/write one — a silent miscompile of a shape that was
    /// E5506 pre-Stage-C.
    pub(crate) fn promotable_scalar_cell_in(
        &self,
        owner: &str,
        name: &str,
        is_scalar: bool,
    ) -> bool {
        is_scalar && self.repr_table.scalar(owner, name) == kali_common::Repr::I64
    }

    /// True when THIS function owns a promotable env — i.e. `lower.rs` reserved
    /// its `current_env` save local because it has >=1 promotable cell (a
    /// C1 scalar-i64 cell OR a C2 fixed-shape-object pointer cell — both
    /// halves of `crate::closure::cell_is_promotable`, the single shared
    /// promotion predicate). Only such a function runs the env
    /// prologue/epilogue (allocating a record and mutating
    /// `CURRENT_ENV_GLOBAL`); a function whose cells are all NON-promotable
    /// (F64/string scalars, closure/array heap cells) leaves `current_env`
    /// untouched, exactly like baseline.
    pub(crate) fn owns_promotable_env(&self) -> bool {
        self.locals
            .contains_key(&crate::closure::env_save_local_name())
    }

    /// Wasm function index of the synthetic runtime-substring helper
    /// (`__substring(h, s, e) -> i64`, Spec 2): pure-ALU clamp/swap +
    /// zero-copy handle re-tag over a tagged string handle.
    pub(crate) fn substring_fn_index(&self) -> u32 {
        self.functions["__substring"]
    }

    /// Wasm function index of the synthetic runtime-join helper
    /// (`__join(arr, sep) -> i64`, Spec 3): two-pass `memory.copy` of an
    /// all-string-element array into one fresh `__alloc_global` string.
    pub(crate) fn join_fn_index(&self) -> u32 {
        self.functions["__join"]
    }

    /// Wasm function index of the arena twin of the runtime-join helper
    /// (`__join_arena(arr, sep) -> i64`, fasta Spec 7 Task 4c): identical to
    /// `__join` but allocates its result into the current (resettable) arena
    /// via `__alloc`. Selected by `emit_runtime_join` only for a join site the
    /// escape gate proved iteration-local (`arena_string_site`).
    pub(crate) fn join_arena_fn_index(&self) -> u32 {
        self.functions["__join_arena"]
    }

    /// Wasm function index of the growable-array integer-join synthetic
    /// (`__join_growable_i64(arr, sep) -> i64`, Task 5): joins a tagged
    /// growable handle whose slots are raw i64 numbers, rendering each via
    /// `int_to_string` before the byte copy. Selected by `emit_runtime_join`
    /// for a growable receiver whose element repr is (default) `I64`.
    pub(crate) fn join_growable_i64_fn_index(&self) -> u32 {
        self.functions["__join_growable_i64"]
    }

    /// Wasm function index of the growable-array string-join synthetic
    /// (`__join_growable_str(arr, sep) -> i64`, Task 5): joins a tagged
    /// growable handle whose slots are already string handles (the byte
    /// `memory.copy` path over the header-indirected layout). Selected by
    /// `emit_runtime_join` for a growable receiver whose element repr is
    /// `String`.
    pub(crate) fn join_growable_str_fn_index(&self) -> u32 {
        self.functions["__join_growable_str"]
    }

    /// Wasm function index of the synthetic runtime string-equality helper
    /// (`__streq(a, b) -> i64`, throw-fallout Stage 1): content comparison of
    /// two tagged string handles (identity fast path, tag guard, length
    /// pre-check, byte loop). Called by `emit_binary`'s both-string equality
    /// arm.
    pub(crate) fn streq_fn_index(&self) -> u32 {
        self.functions["__streq"]
    }

    /// Selects the string-concat host import for the concat node `id` (fasta
    /// Spec 7 Task 4d) — the codegen half of the string-site "both-sides
    /// oracle", exactly mirroring `emit_runtime_join`'s join selection. Returns
    /// `STRING_CONCAT_ARENA_IMPORT_INDEX` (current-arena `__alloc`) iff the
    /// escape gate proved THIS site's result iteration-local, keyed by the
    /// site's pre-order string-site ordinal (`string_site_ordinals`, the shared
    /// stream numbered by `crate::lower::string_site_preorder_ordinals`). A miss
    /// — the node is not a numbered string site (e.g. a `+=` compound-assign
    /// node, whose text is `"+="` not `"+"`, so `is_string_site` never records
    /// it), or the site is numbered but ungranted — fails closed to the global
    /// `STRING_CONCAT_IMPORT_INDEX` (never-reset `__alloc_global`).
    pub(crate) fn string_concat_import_index(&self, id: LirNodeId) -> u32 {
        let use_arena = self
            .string_site_ordinals
            .get(&id)
            .is_some_and(|&ord| self.arena_table.arena_string_site(&self.function_name, ord));
        if use_arena {
            crate::STRING_CONCAT_ARENA_IMPORT_INDEX
        } else {
            crate::STRING_CONCAT_IMPORT_INDEX
        }
    }

    pub(crate) fn push_control_frame(&mut self, kind: ControlFlowLabelKind) -> usize {
        self.control_frames.push(kind);
        self.control_frames.len() - 1
    }

    pub(crate) fn pop_control_frame(&mut self, kind: ControlFlowLabelKind) {
        let popped = self.control_frames.pop();
        debug_assert_eq!(popped, Some(kind));
    }

    pub(crate) fn control_frame_depth(&self, target_index: usize) -> u32 {
        debug_assert!(target_index < self.control_frames.len());
        (self.control_frames.len() - 1 - target_index) as u32
    }

    pub(crate) fn node(&self, id: LirNodeId) -> &LirNode {
        let index = id.0 as usize;
        if index < self.node_lookup.len() {
            &self.node_lookup[index]
        } else {
            &self.scratch_nodes[index - self.node_lookup.len()]
        }
    }

    pub(crate) fn alloc_scratch_node(
        &mut self,
        kind: LirNodeKind,
        text: Option<String>,
        children: Vec<LirNodeId>,
    ) -> LirNodeId {
        let id = LirNodeId((self.node_lookup.len() + self.scratch_nodes.len()) as u32);
        self.scratch_nodes.push(LirNode {
            kind,
            text,
            children,
            function_flavor: None,
        });
        id
    }

    pub(crate) fn push_placeholder_fallback_diagnostic(&mut self, kind: &str, name: &str) {
        let fallback_key = format!("{kind}:{name}");
        if !self.reported_placeholder_fallbacks.insert(fallback_key) {
            return;
        }

        let message = match kind {
            "identifier" => format!(
                "undefined identifier '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                name
            ),
            "call target" => format!(
                "undefined call target '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                name
            ),
            _ => format!(
                "undefined {} '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                kind, name
            ),
        };

        let mut diagnostic = Diagnostic::warning(e3::UNDEFINED_IDENTIFIER as u32, message)
            .with_context(
                DiagnosticContext::new(DiagnosticContextOrigin::Source)
                    .with_requested_value(name)
                    .with_effective_value("zero placeholder compatibility fallback"),
            )
            .note(
                "name resolution should resolve this before codegen; the fallback emits a zero placeholder and should remain a compatibility escape hatch only",
            );
        if let Some(source_path) = &self.source_path {
            diagnostic = diagnostic.note(format!("source path: {}", source_path.display()));
        }
        self.diagnostics.push(diagnostic);
    }
}
