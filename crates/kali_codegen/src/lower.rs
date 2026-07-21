//! Program-level driver and LIR-walking analysis functions.
use crate::*;

/// Maps a representation decision to the wasm value type used for the matching
/// param/result/local slot. `I64` is the integer default; `F64` is an IEEE double.
pub(crate) fn wasm_type(repr: kali_common::Repr) -> wasm_encoder::ValType {
    match repr {
        kali_common::Repr::F64 => wasm_encoder::ValType::F64,
        kali_common::Repr::I64
        | kali_common::Repr::Object(_)
        | kali_common::Repr::String
        // A growable-array binding is a tagged i64 handle into its header
        // (see `lower_lir_to_wasm`'s `__join_growable_*` doc comment), so it
        // occupies the same i64 param/result/local slot as every other
        // handle repr — no new storage width needed.
        | kali_common::Repr::GrowableArrayI64
        // AbortHandle is an i64 pointer to an abort cell (Stage P3); same slot.
        | kali_common::Repr::AbortHandle
        // URL struct pointer and USP growable handle — both one i64 slot.
        | kali_common::Repr::Url
        | kali_common::Repr::UrlSearchParams => wasm_encoder::ValType::I64,
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
/// helper (Spec 2), the runtime-join pair (Spec 3 / fasta Spec 7), and the
/// runtime string-equality helper (throw-fallout Stage 1). Used to exclude
/// them from coverage instrumentation (see the `kali:coverage` custom-section
/// count below) and, in later tasks, anywhere else code needs to distinguish
/// a real source-defined function from these fixed compiler-internal slots.
pub const SYNTHETIC_FUNCTIONS: &[&str] = &[
    "__alloc",
    "__alloc_global",
    "__page_get",
    "__arena_reset",
    "__substring",
    "__join",
    "__join_arena",
    "__join_growable_i64",
    "__join_growable_str",
    "__streq",
    "__usp_get",
    "__usp_has",
    "__usp_getall",
    "__usp_set",
    "__percent_encode",
    "__usp_tostring",
];

/// A synthetic function name is either an exact entry in `SYNTHETIC_FUNCTIONS`
/// or a shape-parameterized deep-clone synthetic `__clone_shape_<n>` (Stage P2
/// Lane 2, emitted on demand by `collect_requested_clone_shapes`). Every place
/// that used to consult `SYNTHETIC_FUNCTIONS.contains` to tell a compiler-
/// internal slot from a source-defined function goes through this helper so the
/// parameterized clone names are recognized too.
pub fn is_synthetic_function(name: &str) -> bool {
    SYNTHETIC_FUNCTIONS.contains(&name) || name.starts_with("__clone_shape_")
}

/// Shapes for which a `__clone_shape_<n>` deep-clone synthetic must be emitted
/// (Stage P2 Lane 2). The synthetic is currently unreachable from any source
/// program — Task 8's `structuredClone` dispatch is what will resolve a call to
/// `clone_shape_synthetic_name(shape)` and, in doing so, populate this set (by
/// scanning the LIR for the clone sites and their argument shapes). Until then
/// this returns empty, so no clone slot is emitted and the module stays
/// byte-identical. Kept as the single collection point so Task 8 wires the scan
/// here and the emission machinery below (plan + type + body dispatch) needs no
/// further change.
fn collect_requested_clone_shapes(
    lir: &LirProgram,
    repr_table: &kali_common::ReprTable,
) -> std::collections::BTreeSet<kali_common::ShapeId> {
    let mut requested = std::collections::BTreeSet::new();
    // Gate on the presence of ANY bare `structuredClone(...)` call — a SUPERSET
    // probe (shadowing unchecked here) exactly like `program_constructs_event_target`
    // gates the event-target import. If none exists the set stays empty and the
    // module is byte-identical.
    if !program_calls_bare_identifier(lir, "structuredClone") {
        return requested;
    }
    // Request a `__clone_shape_N` synthetic for EVERY interned envelope shape.
    // This is a sound SUPERSET of what the call-site dispatch can resolve: the
    // dispatch only ever clones an in-envelope object shape (Task 8's
    // `shape_is_clone_envelope`), and it resolves the callee index through
    // `function_name_to_index[&clone_shape_synthetic_name(shape)]` — so every
    // shape the dispatch could name is guaranteed present here. A shape that is
    // in-envelope but never actually cloned yields a dead synthetic (harmless);
    // an out-of-envelope shape is never requested (its clone body would
    // shallow-share a nested object — see `fields_are_clone_envelope`).
    for index in 0..repr_table.shape_count() {
        let shape = kali_common::ShapeId(index as u32);
        if crate::emit::clone::fields_are_clone_envelope(repr_table.shape_fields(shape))
            && repr_table.shape_is_clone_safe(shape)
        {
            requested.insert(shape);
        }
    }
    requested
}

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
    let uses_args_get = program_uses_args_get(lir);
    let uses_performance_now = program_uses_performance_now(lir);
    let uses_crypto_get_random_values = program_uses_crypto_get_random_values(lir);
    let uses_crypto_random_uuid = program_uses_crypto_random_uuid(lir);
    let uses_crypto_subtle_digest = program_uses_crypto_subtle_digest(lir);
    // Stage D: scheduling-surface conditional imports, appended LAST (after
    // crypto_subtle_digest) in declaration order queueMicrotask, setTimeout,
    // setInterval, clearTimeout, clearInterval — so no earlier import or
    // function index shifts.
    let uses_queue_microtask = program_calls_bare_identifier(lir, "queueMicrotask");
    let uses_set_timeout = program_calls_bare_identifier(lir, "setTimeout");
    let uses_set_interval = program_calls_bare_identifier(lir, "setInterval");
    let uses_clear_timeout = program_calls_bare_identifier(lir, "clearTimeout");
    let uses_clear_interval = program_calls_bare_identifier(lir, "clearInterval");
    // Stage D event-surface lane, appended LAST (after clearInterval) in
    // declaration order event_target_new, event_listener_add, event_dispatch —
    // so no earlier import or function index shifts.
    let uses_event_target_new = program_constructs_event_target(lir);
    let uses_event_listener_add = program_calls_member_named(lir, "addEventListener");
    let uses_event_dispatch = program_calls_member_named(lir, "dispatchEvent");
    let uses_env_access = uses_env_get || uses_env_has || uses_env_set || uses_env_delete;
    let function_index_offset = crate::FUNCTION_INDEX_OFFSET
        + if ctx.target.coverage { 1 } else { 0 }
        + if uses_env_set { 1 } else { 0 }
        + if uses_env_delete { 1 } else { 0 }
        + if uses_env_get { 1 } else { 0 }
        + if uses_env_has { 1 } else { 0 }
        + if uses_cwd_set { 1 } else { 0 }
        + if uses_process_exit { 1 } else { 0 }
        + if uses_stdout_write_bytes { 1 } else { 0 }
        + if uses_args_get { 1 } else { 0 }
        + if uses_performance_now { 1 } else { 0 }
        + if uses_crypto_get_random_values { 1 } else { 0 }
        + if uses_crypto_random_uuid { 1 } else { 0 }
        + if uses_crypto_subtle_digest { 1 } else { 0 }
        + if uses_queue_microtask { 1 } else { 0 }
        + if uses_set_timeout { 1 } else { 0 }
        + if uses_set_interval { 1 } else { 0 }
        + if uses_clear_timeout { 1 } else { 0 }
        + if uses_clear_interval { 1 } else { 0 }
        + if uses_event_target_new { 1 } else { 0 }
        + if uses_event_listener_add { 1 } else { 0 }
        + if uses_event_dispatch { 1 } else { 0 };
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
    // `args_get` is appended AFTER `stdout_write_bytes` in the import section
    // below, so its index sums every preceding conditional-import flag in the
    // same declaration order (coverage, env_set, env_delete, env_get, env_has,
    // cwd_set, process_exit, stdout_write_bytes).
    let args_get_import_index = if uses_args_get {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 },
        )
    } else {
        None
    };
    // `performance_now` is appended AFTER `args_get` in the import section below,
    // so its index sums every preceding conditional-import flag in the same
    // declaration order (coverage, env_set, env_delete, env_get, env_has,
    // cwd_set, process_exit, stdout_write_bytes, args_get).
    let performance_now_import_index = if uses_performance_now {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 },
        )
    } else {
        None
    };
    // `crypto_get_random_values` is appended AFTER `performance_now` in the import
    // section below, so its index sums every preceding conditional-import flag in
    // the same declaration order (coverage, env_set, env_delete, env_get, env_has,
    // cwd_set, process_exit, stdout_write_bytes, args_get, performance_now).
    let crypto_get_random_values_import_index = if uses_crypto_get_random_values {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 },
        )
    } else {
        None
    };
    // `crypto_random_uuid` is appended AFTER `crypto_get_random_values` in the
    // import section below, so its index additionally sums
    // `uses_crypto_get_random_values`.
    let crypto_random_uuid_import_index = if uses_crypto_random_uuid {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 },
        )
    } else {
        None
    };
    // `crypto_subtle_digest` is appended AFTER `crypto_random_uuid` in the import
    // section below, so its index additionally sums both preceding crypto flags
    // (`uses_crypto_get_random_values` + `uses_crypto_random_uuid`).
    let crypto_subtle_digest_import_index = if uses_crypto_subtle_digest {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 },
        )
    } else {
        None
    };
    // Stage D scheduling-surface imports, appended (in this order) AFTER
    // `crypto_subtle_digest`: queueMicrotask, setTimeout, setInterval,
    // clearTimeout, clearInterval. Each index sums every preceding
    // conditional-import flag, so each new block adds exactly one more term
    // (`+ if uses_<previous> {1} else {0}`) onto the block above it.
    let queue_microtask_import_index = if uses_queue_microtask {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 },
        )
    } else {
        None
    };
    let set_timeout_import_index = if uses_set_timeout {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 },
        )
    } else {
        None
    };
    let set_interval_import_index = if uses_set_interval {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 },
        )
    } else {
        None
    };
    let clear_timeout_import_index = if uses_clear_timeout {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 }
                + if uses_set_interval { 1 } else { 0 },
        )
    } else {
        None
    };
    let clear_interval_import_index = if uses_clear_interval {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 }
                + if uses_set_interval { 1 } else { 0 }
                + if uses_clear_timeout { 1 } else { 0 },
        )
    } else {
        None
    };
    // Stage D event-surface lane import indices — each chain is the previous
    // import's full chain plus one term, in declaration order (event_target_new,
    // event_listener_add, event_dispatch), appended after clearInterval.
    let event_target_new_import_index = if uses_event_target_new {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 }
                + if uses_set_interval { 1 } else { 0 }
                + if uses_clear_timeout { 1 } else { 0 }
                + if uses_clear_interval { 1 } else { 0 },
        )
    } else {
        None
    };
    let event_listener_add_import_index = if uses_event_listener_add {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 }
                + if uses_set_interval { 1 } else { 0 }
                + if uses_clear_timeout { 1 } else { 0 }
                + if uses_clear_interval { 1 } else { 0 }
                + if uses_event_target_new { 1 } else { 0 },
        )
    } else {
        None
    };
    let event_dispatch_import_index = if uses_event_dispatch {
        Some(
            crate::COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if uses_process_exit { 1 } else { 0 }
                + if uses_stdout_write_bytes { 1 } else { 0 }
                + if uses_args_get { 1 } else { 0 }
                + if uses_performance_now { 1 } else { 0 }
                + if uses_crypto_get_random_values { 1 } else { 0 }
                + if uses_crypto_random_uuid { 1 } else { 0 }
                + if uses_crypto_subtle_digest { 1 } else { 0 }
                + if uses_queue_microtask { 1 } else { 0 }
                + if uses_set_timeout { 1 } else { 0 }
                + if uses_set_interval { 1 } else { 0 }
                + if uses_clear_timeout { 1 } else { 0 }
                + if uses_clear_interval { 1 } else { 0 }
                + if uses_event_target_new { 1 } else { 0 }
                + if uses_event_listener_add { 1 } else { 0 },
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
    // Arena twin of `__join` (fasta Spec 7 Task 4c): byte-for-byte identical to
    // `__join` EXCEPT its result is allocated via the current-arena `__alloc`
    // (dispatch below) instead of `__alloc_global`, so a join whose result the
    // escape gate proved iteration-local (`arena_string_site`) lands in the
    // resettable per-iteration arena. Emitted unconditionally (it is tiny; DCE
    // is not run) — `emit_runtime_join` selects it per site, and any site not
    // positively granted keeps the global `__join`, so this is fail-closed.
    all_functions.push(FunctionPlan {
        name: "__join_arena".to_string(),
        params: vec!["arr".to_string(), "sep".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Growable-array join synthetics (throw-fallout Stage 4 Task 5):
    // `__join_growable_i64` / `__join_growable_str` (arr: i64, sep: i64) ->
    // i64 — the growable-layout analogue of `__join`. The receiver is a
    // TAGGED growable handle (header indirection: `n=*(hdr+0)`,
    // `data=*(hdr+16)`, elem `=*(data+i*8)`); the `_i64` variant additionally
    // renders each raw i64 slot to a decimal string via `int_to_string`
    // before measuring/copying. Both allocate their result into the global
    // heap (`__alloc_global`) — a join result must outlive any arena reset,
    // exactly as `__join` does. `emit_runtime_join` selects the pair member
    // by the growable binding's element repr. Same inert-placeholder pattern
    // as the synthetics above; bodies hand-emitted by `emit_join_growable_body`.
    all_functions.push(FunctionPlan {
        name: "__join_growable_i64".to_string(),
        params: vec!["arr".to_string(), "sep".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__join_growable_str".to_string(),
        params: vec!["arr".to_string(), "sep".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Synthetic runtime string equality `__streq(a: i64, b: i64) -> i64`
    // (throw-fallout Stage 1): content comparison of two tagged string
    // handles — 1 when equal, 0 when not. Handle-identity fast path, then a
    // string-tag guard (a 0/untagged operand — e.g. a missing `Deno.env.get`
    // — is unequal to every real string), then length pre-check, then a
    // byte-compare loop. Same inert-placeholder pattern as the synthetics
    // above; body hand-emitted by `emit_streq_body`.
    all_functions.push(FunctionPlan {
        name: "__streq".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Synthetic URLSearchParams scan/mutation helpers (Stage P4 Task 4). Each
    // takes the tagged growable pair-store handle (`store`) and scans the
    // `[k0,v0,…]` data block, calling `__streq` for key comparison (its index is
    // threaded into each body exactly as `alloc_global_index` is threaded into
    // `emit_join_body`). `__usp_getall`/`__usp_set` also allocate through
    // `__alloc_global` (a fresh result / a grown data block must not dangle
    // across an arena reset). Inert-placeholder pattern like the synthetics
    // above; bodies hand-emitted by `emit_usp_*_body` (dispatch below).
    all_functions.push(FunctionPlan {
        name: "__usp_get".to_string(),
        params: vec!["store".to_string(), "key".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__usp_has".to_string(),
        params: vec!["store".to_string(), "key".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__usp_getall".to_string(),
        params: vec!["store".to_string(), "key".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__usp_set".to_string(),
        params: vec!["store".to_string(), "key".to_string(), "val".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // `URLSearchParams.toString()` serialization pair (Stage P4 Task 5).
    // `__percent_encode(str) -> i64` re-encodes one component's bytes as
    // application/x-www-form-urlencoded (unreserved verbatim, space → `+`,
    // else `%XX` uppercase) into a fresh `__alloc_global` buffer and returns a
    // packed String handle. `__usp_tostring(store) -> i64` walks the pair
    // store and builds `enc(k0)=enc(v0)&enc(k1)=enc(v1)&…` in one
    // `__alloc_global` buffer (separator bytes stored directly; each encoded
    // component's bytes copied in — no `string_concat` import, no interned
    // literals). Same inert-placeholder pattern as the synthetics above;
    // bodies hand-emitted by `emit_percent_encode_body` /
    // `emit_usp_tostring_body`.
    all_functions.push(FunctionPlan {
        name: "__percent_encode".to_string(),
        params: vec!["str".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    all_functions.push(FunctionPlan {
        name: "__usp_tostring".to_string(),
        params: vec!["store".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
    // Per-shape deep-clone synthetics `__clone_shape_<n>` (Stage P2 Lane 2):
    // appended AFTER the fixed synthetics and BEFORE any source-defined function
    // so, like the fixed synthetics, they shift every later function's index by
    // a fixed amount — safe because every call site resolves callee indices
    // through `function_name_to_index`. Bodies are hand-emitted (dispatch below)
    // exactly like the other synthetics, so `body`/`locals`/`flavor` are inert
    // placeholders. The requested set is empty until Task 8 wires its scan, so
    // this loop currently adds nothing and the module stays byte-identical.
    for shape in collect_requested_clone_shapes(lir, &ctx.repr_table) {
        all_functions.push(FunctionPlan {
            name: crate::emit::clone::clone_shape_synthetic_name(shape),
            params: vec!["src".to_string()],
            locals: Vec::new(),
            body: lir.root,
            result: true,
            is_entry: false,
            flavor: None,
        });
    }
    all_functions.extend(function_plans);

    let mut function_param_counts: BTreeMap<u32, usize> = BTreeMap::new();
    // Owner-name -> its declared parameter names. Used by the deferred-callback
    // scalar-capture deny (Task 9 C-1) to tell a captured PARAMETER (a real
    // value node computes, silently placeholder-0 in the deferred lane — deny)
    // apart from a captured non-scalar placeholder binding (an unsupported
    // zero-placeholder construct with no real value either side — allow).
    let mut function_param_names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (idx, function) in all_functions.iter().enumerate() {
        let windex = idx as u32 + function_index_offset;
        function_name_to_index.insert(function.name.clone(), windex);
        function_param_counts.insert(windex, function.params.len());
        function_param_names.insert(function.name.clone(), function.params.clone());
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
    // Type 10: args_get `(index: i32, out_ptr: i32, out_cap: i32) -> i32`
    // (Spec 5 Task 5) — writes an argv element's UTF-8 bytes into guest memory,
    // returns the byte count or -1. Registered unconditionally so the type
    // index is stable; the import itself is conditional (see below).
    const ARGS_GET_TYPE_INDEX: u32 = 10;
    type_section.ty().function(
        vec![ValType::I32, ValType::I32, ValType::I32],
        vec![ValType::I32],
    );
    // Type 11: performance_now `() -> f64` (throw-fallout Stage 3 bucket #5) —
    // returns a monotonic millisecond timestamp. Registered unconditionally so
    // the type index is stable; the import itself is conditional (see below).
    const PERFORMANCE_NOW_TYPE_INDEX: u32 = 11;
    type_section.ty().function(vec![], vec![ValType::F64]);
    // Type 12: crypto_subtle_digest
    // `(algo_ptr: i32, algo_len: i32, in_ptr: i32, in_len: i32, out_ptr: i32,
    // out_cap: i32) -> i32` (throw-fallout Stage 3 bucket #6 part 2) — reads the
    // algorithm name + input bytes from guest memory, writes the raw digest bytes
    // at `out_ptr` (bounded by `out_cap`), and returns the digest byte length.
    // This is a NEW fixed signature (no existing type matches the 6-i32-arg
    // shape), registered unconditionally so the type index is stable; the import
    // itself is conditional (see below). Because it is the last fixed type, the
    // repr-directed function types start after the last fixed type (see the
    // dedup base below).
    const CRYPTO_SUBTLE_DIGEST_TYPE_INDEX: u32 = 12;
    type_section.ty().function(
        vec![
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
        ],
        vec![ValType::I32],
    );
    // Type 13: test_register `(callback_index: i32, env_ptr: i64) -> ()`
    // (Stage C C3). The trailing `env_ptr` carries the `current_env` active at
    // registration so a capturing `Kali.test(...)` callback resolves its
    // enclosing bindings when the host invokes it later. Type 14
    // (`SCHEDULING_TIMER_SET_TYPE_INDEX`, added below) is now the last fixed
    // type, so the repr-directed function types start at index 15.
    const TEST_REGISTER_TYPE_INDEX: u32 = 13;
    type_section
        .ty()
        .function(vec![ValType::I32, ValType::I64], Vec::new());
    // Type 14: setTimeout / setInterval
    // `(callback_index: i32, delay_ms: i32, env_ptr: i64) -> i32` (Stage D) —
    // registers a timer with the env active at the scheduling site and
    // returns the i32 timer id. Registered unconditionally so the type index
    // is stable; the imports are conditional.
    const SCHEDULING_TIMER_SET_TYPE_INDEX: u32 = 14;
    type_section.ty().function(
        vec![ValType::I32, ValType::I32, ValType::I64],
        vec![ValType::I32],
    );
    // Type 15: event_target_new `() -> i64` (Stage D event lane) — returns a
    // fresh opaque EventTarget handle.
    const EVENT_TARGET_NEW_TYPE_INDEX: u32 = 15;
    type_section.ty().function(vec![], vec![ValType::I64]);
    // Type 16: event_listener_add `(target: i64, name_ptr: i32, name_len: i32,
    // callback_index: i32, env_ptr: i64) -> ()`.
    const EVENT_LISTENER_ADD_TYPE_INDEX: u32 = 16;
    type_section.ty().function(
        vec![
            ValType::I64,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I64,
        ],
        vec![],
    );
    // Type 17: event_dispatch `(target: i64, name_ptr: i32, name_len: i32) -> i32`
    // — synchronously invokes the snapshot of listeners, returns 1 (true).
    // This is now the last fixed type: repr-directed function types start at 18.
    const EVENT_DISPATCH_TYPE_INDEX: u32 = 17;
    type_section.ty().function(
        vec![ValType::I64, ValType::I32, ValType::I32],
        vec![ValType::I32],
    );
    let mut import_section = ImportSection::new();
    import_section.import(
        "kali:rt",
        "test_register",
        EntityType::Function(TEST_REGISTER_TYPE_INDEX),
    );
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
    // Five unconditional runtime helpers occupy fixed import indices 17 through 21
    // (see INT_TO_STRING_IMPORT_INDEX / STRING_CONCAT_IMPORT_INDEX /
    // FLOAT_TO_FIXED_IMPORT_INDEX / FLOAT_TO_STRING_IMPORT_INDEX /
    // STRING_CONCAT_ARENA_IMPORT_INDEX). They are registered here, before the
    // conditional coverage/env/process imports, so the relative bookkeeping below
    // (all expressed against COVERAGE_HIT_IMPORT_INDEX = 22) stays consistent.
    // int_to_string is (i64) -> i64 (type 4); string_concat is (i64, i64) -> i64
    // (type 3); float_to_fixed is (f64, i32) -> i64 (type 8); float_to_string is
    // (f64) -> i64 (type 9). `string_concat_arena` (fasta Spec 7 Task 4d) is the
    // current-arena twin of `string_concat` and reuses its exact signature (type
    // 3); it is appended LAST among the always-present imports so no earlier fixed
    // import index shifts.
    import_section.import("kali:rt", "int_to_string", EntityType::Function(4));
    import_section.import("kali:rt", "string_concat", EntityType::Function(3));
    import_section.import("kali:rt", "float_to_fixed", EntityType::Function(8));
    import_section.import("kali:rt", "float_to_string", EntityType::Function(9));
    import_section.import("kali:rt", "string_concat_arena", EntityType::Function(3));
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
    if args_get_import_index.is_some() {
        // `(index: i32, out_ptr: i32, out_cap: i32) -> i32`: writes an argv
        // element's UTF-8 bytes into guest memory at `out_ptr` (bounded by
        // `out_cap`), returning the byte count or -1 (out-of-range index).
        import_section.import(
            "kali:rt",
            "args_get",
            EntityType::Function(ARGS_GET_TYPE_INDEX),
        );
    }
    if performance_now_import_index.is_some() {
        // `() -> f64`: returns a monotonic millisecond timestamp. Appended AFTER
        // `args_get`, so it takes the next import index (summing every preceding
        // conditional-import flag including `args_get`).
        import_section.import(
            "kali:rt",
            "performance_now",
            EntityType::Function(PERFORMANCE_NOW_TYPE_INDEX),
        );
    }
    if crypto_get_random_values_import_index.is_some() {
        // `(out_ptr: i32, out_len: i32) -> i32`: fills `out_len` random bytes at
        // `out_ptr` in guest memory (in place) and returns `out_len`. Reuses the
        // existing `(i32, i32) -> i32` signature (type 7); no new type is added.
        // Appended AFTER `performance_now`, so it takes the next import index.
        import_section.import(
            "kali:rt",
            "crypto_get_random_values",
            EntityType::Function(7),
        );
    }
    if crypto_random_uuid_import_index.is_some() {
        // `(out_ptr: i32, out_cap: i32) -> i32`: writes the UUID string's UTF-8
        // bytes at `out_ptr` (bounded by `out_cap`) and returns the byte count.
        // Reuses type 7. Appended AFTER `crypto_get_random_values`.
        import_section.import("kali:rt", "crypto_random_uuid", EntityType::Function(7));
    }
    if crypto_subtle_digest_import_index.is_some() {
        // `(algo_ptr, algo_len, in_ptr, in_len, out_ptr, out_cap) -> i32`: computes
        // the digest of the input bytes and writes the raw digest at `out_ptr`,
        // returning its length. Uses the new type 12. Appended AFTER
        // `crypto_random_uuid`, so it takes the next import index.
        import_section.import(
            "kali:rt",
            "crypto_subtle_digest",
            EntityType::Function(CRYPTO_SUBTLE_DIGEST_TYPE_INDEX),
        );
    }
    if queue_microtask_import_index.is_some() {
        // `(callback_index: i32, env_ptr: i64) -> ()` — same shape as
        // test_register: pushes the callback id + the scheduling-site
        // `current_env` onto the host microtask FIFO; drained after `_start`
        // (`kali_runtime::host::enforce::drain_event_loop`).
        import_section.import(
            "kali:rt",
            "queueMicrotask",
            EntityType::Function(TEST_REGISTER_TYPE_INDEX),
        );
    }
    if set_timeout_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "setTimeout",
            EntityType::Function(SCHEDULING_TIMER_SET_TYPE_INDEX),
        );
    }
    if set_interval_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "setInterval",
            EntityType::Function(SCHEDULING_TIMER_SET_TYPE_INDEX),
        );
    }
    if clear_timeout_import_index.is_some() {
        // `(timer_id: i32) -> ()` — same shape as coverage_hit (type 0).
        import_section.import("kali:rt", "clearTimeout", EntityType::Function(0));
    }
    if clear_interval_import_index.is_some() {
        import_section.import("kali:rt", "clearInterval", EntityType::Function(0));
    }
    if event_target_new_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_target_new",
            EntityType::Function(EVENT_TARGET_NEW_TYPE_INDEX),
        );
    }
    if event_listener_add_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_listener_add",
            EntityType::Function(EVENT_LISTENER_ADD_TYPE_INDEX),
        );
    }
    if event_dispatch_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_dispatch",
            EntityType::Function(EVENT_DISPATCH_TYPE_INDEX),
        );
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
        } else if matches!(
            function.name.as_str(),
            "__join" | "__join_arena" | "__join_growable_i64" | "__join_growable_str"
        ) {
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
            // Function-signature types begin right after the fixed types
            // (0..=EVENT_DISPATCH_TYPE_INDEX): the event-dispatch type (type 17)
            // is now the last fixed type, so repr-directed function types start
            // at index 18.
            let idx = function_types.len() as u32 + EVENT_DISPATCH_TYPE_INDEX + 1;
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
    // Export g8 (`current_env`, Stage C closures) under a stable name so the
    // host can set it to a deferred callback's captured `env_ptr` before the
    // nullary `__kali_callback_<idx>` call and restore it after (Phase C3,
    // `invoke_callback`). A guest that owns no promotable env still exports this
    // (harmless): the value stays 0 and the host's set/restore is a no-op.
    export_section.export(
        "__current_env",
        ExportKind::Global,
        crate::closure::CURRENT_ENV_GLOBAL,
    );
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

    // Stage C: C1 promotes ONLY scalar `i64` cells into the env record — the
    // shape it can lower soundly (a raw 8-byte i64 slot + i64 arithmetic).
    // Every other cell shape (heap/closure captures = the C2 surface, non-`i64`
    // scalars, multi-level chains) is left EXACTLY as pre-Stage-C: it keeps its
    // WASM local / const-fold / placeholder path, so those programs stay
    // byte-identical (no new E5506, no new machinery). For a function with >=1
    // promotable scalar-i64 cell we (a) drop those cell names from its locals —
    // the env cell IS their storage, so they get no WASM local slot — and
    // (b) reserve a dedicated i64 save local for the incoming `current_env`
    // (restored on every exit path). Keyed by the SAME name space as
    // `derive_env_plans` (declared / `__kali_fn_N`; `_start` is the module root
    // `""`, absent here and never an owner). This mutation must precede both the
    // `local_decls` build and `FunctionEmitter::new` below so the declared local
    // set and the emitter's `locals` map stay in lockstep.
    for function in all_functions.iter_mut() {
        if let Some(plan) = ctx.env_plans.get(&function.name) {
            let promoted: HashSet<&str> = plan
                .cells
                .iter()
                .filter(|cell| {
                    // Owner-keyed lockstep predicate (C1 scalar-i64 OR C2
                    // fixed-shape object). `function.name` IS the owner here —
                    // these are its OWN cells — so the owner namespace is this
                    // function's. Same predicate the access gate uses.
                    crate::closure::cell_is_promotable(
                        &ctx.repr_table,
                        &function.name,
                        &cell.name,
                        cell.is_scalar,
                    )
                })
                .map(|cell| cell.name.as_str())
                .collect();
            if !promoted.is_empty() {
                function
                    .locals
                    .retain(|name| !promoted.contains(name.as_str()));
                function.locals.push(crate::closure::env_save_local_name());
            }
        }
    }

    // Stage C stage-review CRITICAL fix: the dynamic-env safety gate. Capture
    // lowering resolves cells against the DYNAMIC `current_env`, but the
    // capture analysis is LEXICAL — a capturer invoked while a sibling
    // env-owner's record is active silently addresses the wrong cells (a
    // cross-binding memory corruption). Reject-don't-miscompile: every
    // call/registration edge into an engaged capturer must provably run with
    // the capturer's owner record in `current_env`, else E5506 (matching the
    // pre-Stage-C base, which rejected these shapes at the capture sites).
    // See `crate::env_safety` for the interprocedural fixpoint.
    diagnostics.extend(crate::env_safety::env_capture_safety_diagnostics(
        lir,
        &all_functions,
        &ctx.env_plans,
        &ctx.repr_table,
    ));

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
        //   `__streq` (`emit_streq_body`): 4 i64 — `len`, `i`, `pa`, `pb`
        //     (locals 2-5; locals 0-1 are its `a`/`b` params).
        let mut local_decls: Vec<(u32, ValType)> = Vec::new();
        if matches!(function.name.as_str(), "__alloc" | "__alloc_global") {
            local_decls.push((2, ValType::I32));
        } else if function.name == "__page_get" {
            local_decls.push((4, ValType::I32));
        } else if function.name == "__arena_reset" {
            local_decls.push((2, ValType::I32));
        } else if function.name == "__substring" {
            local_decls.push((1, ValType::I64));
        } else if function.name == "__streq" {
            local_decls.push((4, ValType::I64));
        } else if matches!(function.name.as_str(), "__usp_get" | "__usp_has") {
            // `emit_usp_get_body`/`emit_usp_has_body`: 3 i64 — `len`, `i`, `data`
            // (locals 2-4; locals 0-1 are `store`/`key`).
            local_decls.push((3, ValType::I64));
        } else if function.name == "__usp_getall" {
            // `emit_usp_getall_body`: 6 i64 — `len`, `i`, `data`, `count`,
            // `newhdr`, `newdata` (locals 2-7; locals 0-1 are `store`/`key`).
            local_decls.push((6, ValType::I64));
        } else if function.name == "__usp_set" {
            // `emit_usp_set_body`: 7 i64 — `write`, `found`, `i`, `data`, `len`,
            // `cap`, `newdata` (locals 3-9; locals 0-2 are `store`/`key`/`val`).
            local_decls.push((7, ValType::I64));
        } else if function.name == "__percent_encode" {
            // `emit_percent_encode_body`: 7 i64 — `len`, `src`, `out`, `w`,
            // `i`, `b`, `n` (locals 1-7; local 0 is the `str` param).
            local_decls.push((7, ValType::I64));
        } else if function.name == "__usp_tostring" {
            // `emit_usp_tostring_body`: 8 i64 — `len`, `i`, `data`, `w`,
            // `out`, `h`, `p`, `n` (locals 1-8; local 0 is the `store` param).
            local_decls.push((8, ValType::I64));
        } else if matches!(function.name.as_str(), "__join" | "__join_arena") {
            local_decls.push((6, ValType::I64));
        } else if matches!(
            function.name.as_str(),
            "__join_growable_i64" | "__join_growable_str"
        ) {
            // `emit_join_growable_body`: 7 i64 — `n`, `i`, `total`, `out`,
            // `cur`, `h`, `data` (locals 2-8; locals 0-1 are `arr`/`sep`). One
            // more than `__join` for the cached header→`data` pointer.
            local_decls.push((7, ValType::I64));
        } else if function.name.starts_with("__clone_shape_") {
            // Hand-emitted deep-clone synthetic (Stage P2 Lane 2): its i64
            // locals (1=dst, 2=srch, 3=new_hdr, 4=new_data, 5=len, 6=cap; local
            // 0 is the `src` param) — see `emit::clone::emit_clone_shape_body`.
            local_decls.push((crate::emit::clone::CLONE_SHAPE_LOCAL_COUNT, ValType::I64));
        } else {
            for local_name in &function.locals {
                // A `__arena_save_*` local (Step 2 of loop-arena provisioning)
                // holds a saved copy of an i32 global (`g1`/`g2`/`g3`) and has
                // no `ReprTable` entry of its own; `scalar()`'s default
                // (`Repr::I64`) would mistype the slot and fail wasm
                // validation the first time `GlobalGet(1..3)` is stored into
                // it, so it is forced to i32 here ahead of the repr lookup.
                let val_type = if is_arena_save_local_name(local_name)
                    || is_argv_scratch_local_name(local_name)
                {
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
            &function_param_counts,
            &function_param_names,
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
            ctx.env_plans
                .get(&function.name)
                .cloned()
                .unwrap_or_default(),
            &ctx.env_plans,
        );
        let coverage_id = ctx.target.coverage.then_some(coverage_id as u32);
        if is_synthetic_function(&function.name) {
            // Hand-emitted: not lowered from LIR (there is no source-level
            // function body for these synthetic page-pool functions), and
            // deliberately uninstrumented (no `emit_coverage_hit`) since none
            // is a source-defined function.
            let page_get_index = function_name_to_index["__page_get"];
            let alloc_global_index = function_name_to_index["__alloc_global"];
            let alloc_index = function_name_to_index["__alloc"];
            match function.name.as_str() {
                "__alloc" => emit_bump_body(&mut body, 1, 2, 3, page_get_index),
                "__alloc_global" => emit_bump_body(&mut body, 4, 5, 6, page_get_index),
                "__page_get" => emit_page_get_body(&mut body),
                "__arena_reset" => emit_arena_reset_body(&mut body),
                "__substring" => emit_substring_body(&mut body),
                // `__join` allocates its result into the global heap;
                // `__join_arena` is the same body allocating into the current
                // (resettable) arena via `__alloc` (fasta Spec 7 Task 4c).
                "__join" => emit_join_body(&mut body, alloc_global_index),
                "__join_arena" => emit_join_body(&mut body, alloc_index),
                // Growable-array join (Task 5): always `__alloc_global` (the
                // result must not dangle across an arena reset); the `_i64`
                // variant renders each raw slot via `int_to_string`.
                "__join_growable_i64" => {
                    emit_join_growable_body(&mut body, alloc_global_index, true)
                }
                "__join_growable_str" => {
                    emit_join_growable_body(&mut body, alloc_global_index, false)
                }
                "__streq" => emit_streq_body(&mut body),
                // URLSearchParams scan/mutation helpers (Stage P4 Task 4). The
                // `__streq` index is threaded for key comparison; getall/set also
                // take `__alloc_global` (fresh result / grown block must outlive
                // any arena reset), exactly as `__join` takes its allocator.
                "__usp_get" => emit_usp_get_body(&mut body, function_name_to_index["__streq"]),
                "__usp_has" => emit_usp_has_body(&mut body, function_name_to_index["__streq"]),
                "__usp_getall" => emit_usp_getall_body(
                    &mut body,
                    function_name_to_index["__streq"],
                    alloc_global_index,
                ),
                "__usp_set" => emit_usp_set_body(
                    &mut body,
                    function_name_to_index["__streq"],
                    alloc_global_index,
                ),
                // `URLSearchParams.toString()` pair (Stage P4 Task 5): both
                // allocate through `__alloc_global` (a toString result must
                // outlive any arena reset, like `__join`); the joiner builds
                // its output in ONE buffer — separator bytes stored directly,
                // components copied from `__percent_encode` results — so it
                // never touches the global `string_concat` import (whose
                // absence a fully-granted module asserts) and interns nothing.
                "__percent_encode" => emit_percent_encode_body(&mut body, alloc_global_index),
                "__usp_tostring" => emit_usp_tostring_body(
                    &mut body,
                    function_name_to_index["__percent_encode"],
                    alloc_global_index,
                ),
                // Per-shape deep-clone synthetic (Stage P2 Lane 2). Recover the
                // shape from the name and hand-emit its body: fresh object,
                // verbatim scalar slots, deep-copied growable-i64 handles.
                other if other.starts_with("__clone_shape_") => {
                    let shape = crate::emit::clone::clone_shape_id_from_name(other)
                        .expect("well-formed __clone_shape_<n> synthetic name");
                    let fields = ctx.repr_table.shape_fields(shape).to_vec();
                    // Task 8 (binding obligation 1): a `structuredClone` result
                    // almost always ESCAPES its arena (it is bound and read after
                    // the call), so allocate through the escape-safe GLOBAL heap
                    // — mirroring the `__join` (global) vs `__join_arena` (arena)
                    // split above. The arena variant would need a full escape
                    // proof (NOT implemented this task); global is the sound
                    // default and never dangles across an arena reset.
                    crate::emit::clone::emit_clone_shape_body(
                        &mut body,
                        &fields,
                        alloc_global_index,
                    );
                }
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
    // fasta Spec 7 Task 4g: for-in key handle tables — per-shape blobs of
    // compile-time-constant `i64` string handles — as module-constant data,
    // interleaved into the same address space as the string constants above
    // (their bases came from `StringPool::intern_key_table`, which advanced
    // `next_offset`). Emitting them here, before `heap_base` is derived from
    // `next_offset`, keeps them out of the runtime heap entirely (zero
    // per-execution allocation).
    for (offset, bytes) in &string_pool.key_table_entries {
        data_section.active(
            0,
            &ConstExpr::i32_const(*offset as i32),
            bytes.iter().copied(),
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
    // g8 = current_env (Stage C closures, Task 2): the active environment
    // record pointer, mutable i64, 0 = no env. Allocated immediately after
    // the arena trio and before any module-scope scalar global — see
    // `crate::closure::CURRENT_ENV_GLOBAL`. Reserved-but-unused this task
    // (behavior-neutral): nothing reads or writes it yet; Task 3 wires the
    // allocation/store/restore sequence.
    global_section.global(
        GlobalType {
            val_type: ValType::I64,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i64_const(0),
    );
    // Module-scope mutable scalar globals, appended after g0..g8 at indices
    // 9, 10, … (matching the ascending indices assigned in
    // `collect_module_scalar_globals`, which iterates the same sorted-by-name
    // `BTreeMap`). Each is zero-initialized (`var` hoisting semantics: the
    // binding reads `undefined`/0 until its declarator line runs `GlobalSet` in
    // `_start`); the declared wasm type follows the binding's repr.
    for (i, (index, repr)) in module_global_slots.values().enumerate() {
        // The map iterates in the same sorted order the indices were assigned
        // in `collect_module_scalar_globals`, so append position and stored
        // index MUST stay in lockstep — a future filter divergence that broke
        // this would silently desync every `GlobalGet`/`GlobalSet`.
        debug_assert_eq!(
            *index,
            RESERVED_GLOBAL_COUNT + i as u32,
            "module global index/append-order desync"
        );
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
            .filter(|f| !is_synthetic_function(&f.name))
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

/// Structural check: is node `id` a `Deno.env.<method>` member node
/// (`globalThis.Deno.env.<method>` also accepted)? Factors the shared shape the
/// env-* import probes recognize.
fn node_is_deno_env_member(lir: &LirProgram, id: LirNodeId, method: &str) -> bool {
    let Some(member_node) = lir.nodes.get(id.0 as usize) else {
        return false;
    };
    if member_node.text.as_deref() != Some(method) {
        return false;
    }
    let Some(object) = member_node.children.first() else {
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
}

/// Names bound (via a declarator) to a `Deno.env.<method>` member — e.g.
/// `const g = Deno.env.get` yields `g` for method `"get"`. A declarator lowers
/// to an `Instruction` node whose `text` is the bound name and whose
/// `children[1]` is the initializer (`children[0]` is the name value). Used so
/// the env-* import probe sees THROUGH a bound alias `g(...)` (F-Stage1-3),
/// matching the emitter's `resolve_bound_member_callable_node` at the call site.
fn deno_env_member_alias_names<'a>(lir: &'a LirProgram, method: &str) -> Vec<&'a str> {
    lir.nodes
        .iter()
        .filter_map(|node| {
            if node.kind != LirNodeKind::Instruction || node.children.len() < 2 {
                return None;
            }
            if node_is_deno_env_member(lir, node.children[1], method) {
                node.text.as_deref()
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn program_uses_env_get(lir: &LirProgram) -> bool {
    let alias_names = deno_env_member_alias_names(lir, "get");
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(callee) = node.children.first() else {
            return false;
        };
        // Direct `Deno.env.get(...)` call.
        if node_is_deno_env_member(lir, *callee, "get") {
            return true;
        }
        // Bound alias `const g = Deno.env.get; g(...)`: the call's callee is a
        // bare identifier whose name was bound to a `Deno.env.get` member. Only
        // an ACTUAL invocation of the alias flips the probe, so an unused alias
        // never emits the import.
        if let Some(callee_node) = lir.nodes.get(callee.0 as usize) {
            if callee_node.children.is_empty() {
                if let Some(name) = callee_node.text.as_deref() {
                    return alias_names.contains(&name);
                }
            }
        }
        false
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

/// Program-wide probe for a `process.argv[<int literal>]` element read (Spec 5
/// Task 5). Mirrors `FunctionEmitter::is_process_argv_element` /
/// `is_process_argv` structurally over raw nodes (the emitter method needs a
/// live `FunctionEmitter`). Kept a SUPERSET of the emit recognizer: were this
/// ever false where emit fires, the conditional `args_get` import would be
/// undeclared and emit fails closed with an E5506 (never a bad call), so
/// over-inclusiveness here is the safe side.
pub(crate) fn program_uses_args_get(lir: &LirProgram) -> bool {
    lir.nodes
        .iter()
        .any(|node| node_is_process_argv_element(&lir.nodes, node))
}

/// Program-wide probe for a `performance.now()` call (throw-fallout Stage 3
/// bucket #5). Mirrors `FunctionEmitter::performance_now_import_index`
/// structurally over raw nodes (callee text `"now"`, object text
/// `"performance"`). Kept a SUPERSET of the emit recognizer: were this ever
/// false where emit fires, the conditional `performance_now` import would be
/// undeclared and emitting a `Call` to it would be invalid wasm — so
/// over-inclusiveness here is the safe side.
pub(crate) fn program_uses_performance_now(lir: &LirProgram) -> bool {
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
        if callee_node.text.as_deref() != Some("now") {
            return false;
        }
        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        object_node.text.as_deref() == Some("performance")
    })
}

/// Program-wide probe for a `crypto.getRandomValues(buf)` call (throw-fallout
/// Stage 3 bucket #6). Mirrors
/// `FunctionEmitter::crypto_get_random_values_import_index` structurally over raw
/// nodes (callee text `"getRandomValues"`, object text `"crypto"`). Kept a
/// SUPERSET of the emit recognizer: were this ever false where emit fires, the
/// conditional `crypto_get_random_values` import would be undeclared and emitting
/// a `Call` to it would be invalid wasm — so over-inclusiveness here is the safe
/// side.
pub(crate) fn program_uses_crypto_get_random_values(lir: &LirProgram) -> bool {
    program_uses_crypto_method(lir, "getRandomValues")
}

/// Program-wide probe for a `crypto.randomUUID()` call (throw-fallout Stage 3
/// bucket #6). Mirrors `FunctionEmitter::crypto_random_uuid_import_index`
/// structurally over raw nodes (callee text `"randomUUID"`, object text
/// `"crypto"`). Kept a SUPERSET of the emit recognizer (same rationale as
/// `program_uses_crypto_get_random_values`).
pub(crate) fn program_uses_crypto_random_uuid(lir: &LirProgram) -> bool {
    program_uses_crypto_method(lir, "randomUUID")
}

/// Shared body for the two `crypto.<method>()` program-wide probes: a `Call`
/// whose callee is a member with `text == method` and object text `"crypto"`.
fn program_uses_crypto_method(lir: &LirProgram, method: &str) -> bool {
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
        if callee_node.text.as_deref() != Some(method) {
            return false;
        }
        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        object_node.text.as_deref() == Some("crypto")
    })
}

/// Program-wide probe for a `crypto.subtle.digest(algo, bytes)` call (throw-fallout
/// Stage 3 bucket #6 part 2). Mirrors
/// `FunctionEmitter::crypto_subtle_digest_import_index` structurally over raw
/// nodes (callee text `"digest"`, object text `"subtle"`, grand-object text
/// `"crypto"`). Kept a SUPERSET of the emit recognizer (same rationale as the
/// other crypto probes): were this false where emit fires, the conditional
/// `crypto_subtle_digest` import would be undeclared and emitting a `Call` to it
/// would be invalid wasm.
pub(crate) fn program_uses_crypto_subtle_digest(lir: &LirProgram) -> bool {
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
        if callee_node.text.as_deref() != Some("digest") {
            return false;
        }
        let Some(subtle) = callee_node.children.first() else {
            return false;
        };
        let Some(subtle_node) = lir.nodes.get(subtle.0 as usize) else {
            return false;
        };
        if subtle_node.text.as_deref() != Some("subtle") {
            return false;
        }
        let Some(crypto) = subtle_node.children.first() else {
            return false;
        };
        let Some(crypto_node) = lir.nodes.get(crypto.0 as usize) else {
            return false;
        };
        crypto_node.text.as_deref() == Some("crypto")
    })
}

/// Program-wide probe for a bare-identifier call to `name` (Stage D
/// scheduling surfaces). The callee is a PLAIN identifier, not a member
/// expression. Kept a SUPERSET of the emit-time recognizer
/// (`scheduling_surface` additionally requires the name be unshadowed): if
/// this were ever false where emit fires, the conditional import would be
/// undeclared and the emitted `Call` invalid wasm — over-inclusive is the
/// safe side.
pub(crate) fn program_calls_bare_identifier(lir: &LirProgram, name: &str) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(&callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        callee_node.text.as_deref() == Some(name)
    })
}

/// Program-wide probe for `new EventTarget(...)` (Stage D event lane).
/// New-expressions lower to a text-less `Value` whose `children[0]` is the
/// constructor identifier (`Value("EventTarget")`). SUPERSET of the emit-time
/// recognizer (`FunctionEmitter::is_event_target_new`), which additionally
/// requires the name unshadowed + ZERO args + a declarator-init position.
/// Used only to gate the conditional import + type registration.
pub(crate) fn program_constructs_event_target(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        node.text.is_none()
            && node.children.first().is_some_and(|&c| {
                lir.nodes
                    .get(c.0 as usize)
                    .is_some_and(|n| n.text.as_deref() == Some("EventTarget"))
            })
    })
}

/// Program-wide probe for a MEMBER call named `name` (Stage D event lane):
/// any node whose `text` is `name` and which has children (the receiver).
/// SUPERSET of the emit-time recognizer (receiver provenance unchecked here).
/// Used only to gate the conditional import + type registration.
pub(crate) fn program_calls_member_named(lir: &LirProgram, name: &str) -> bool {
    lir.nodes
        .iter()
        .any(|node| node.text.as_deref() == Some(name) && !node.children.is_empty())
}

/// Structural mirror of `FunctionEmitter::is_event_target_new` usable in
/// locals-provisioning (before a `FunctionEmitter` exists). EMPIRICALLY-VERIFIED
/// LIR shape (KALI_DUMP_LIR): `new EventTarget()` lowers to
/// `Value(None, [Call(None, [Value("EventTarget")])])` — the New wrapper's
/// single child is a text-less `Call` whose first child is the ctor identifier
/// (zero args → the Call has exactly one child). Deliberately does NOT unwrap
/// transparent wrappers — the New node is ITSELF a text-less single-child
/// `Value` (same shape as a grouping/single-element-array wrapper), so
/// unwrapping would strip it. The shadowing check is enforced at emit (this
/// side only decides local promotion); an unshadowed match here with a
/// shadowed name at emit simply keeps an unused local slot.
pub(crate) fn declarator_init_is_event_target_new(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let Some(node) = nodes.get(init_id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.len() != 1 {
        return false;
    }
    let Some(call) = nodes.get(node.children[0].0 as usize) else {
        return false;
    };
    if call.kind != LirNodeKind::Call || call.text.is_some() || call.children.len() != 1 {
        return false;
    }
    nodes
        .get(call.children[0].0 as usize)
        .is_some_and(|ctor| ctor.text.as_deref() == Some("EventTarget") && ctor.children.is_empty())
}

/// True when a declarator init is exactly `new AbortController()` (bare
/// callee, zero args): `Value(None, [Call(None, [Value("AbortController")])])`,
/// inspected RAW (unwrapping would strip the New wrapper). Stage P3 gives this
/// construct a real lowering (8-byte global abort cell); the emit side
/// additionally requires the `Repr::AbortHandle` proof and the five-namespace
/// shadow guard before intercepting. Structurally identical to
/// [`declarator_init_is_placeholder_construct`] — the two must agree on the LIR
/// shape — with the ctor text pinned to `"AbortController"` and zero call args.
pub(crate) fn declarator_init_is_abort_controller_new(
    nodes: &[LirNode],
    init_id: LirNodeId,
) -> bool {
    let Some(node) = nodes.get(init_id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.len() != 1 {
        return false;
    }
    let Some(call) = nodes.get(node.children[0].0 as usize) else {
        return false;
    };
    // Zero args → the Call has exactly one child (the ctor identifier).
    if call.kind != LirNodeKind::Call || call.text.is_some() || call.children.len() != 1 {
        return false;
    }
    nodes.get(call.children[0].0 as usize).is_some_and(|ctor| {
        ctor.text.as_deref() == Some("AbortController") && ctor.children.is_empty()
    })
}

/// Stage P4 (URL + URLSearchParams). Compile-time-parsed components in slot
/// order (0..5). Built once from the string-literal constructor arg via the
/// `url` crate; the emitted URL struct interns each of the five string
/// components + an embedded USP built from `query_pairs`.
pub(crate) struct UrlComponents {
    pub href: String,
    pub origin: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub query_pairs: Vec<(String, String)>,
}

/// True when `init_id` is STRUCTURALLY `new <ctor>(<any args>)` with the bare
/// constructor identifier `ctor` (arg-agnostic — used as the deny trigger so a
/// non-admittable `new URL(...)` is rejected E5506 rather than silently lowering
/// to the `0` placeholder). EMPIRICALLY-VERIFIED LIR shape (`new URL('x')`,
/// dumped): the New wrapper is `Value(None, [Call(None, [Value(ctor), <args…>])])`
/// — the OUTER new args are empty; the real args live in the callee
/// `CallExpression`'s children AFTER the ctor.
pub(crate) fn declarator_init_is_url_ctor(
    nodes: &[LirNode],
    init_id: LirNodeId,
    ctor: &str,
) -> bool {
    let Some(node) = nodes.get(init_id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.len() != 1 {
        return false;
    }
    let Some(call) = nodes.get(node.children[0].0 as usize) else {
        return false;
    };
    if call.kind != LirNodeKind::Call || call.text.is_some() || call.children.is_empty() {
        return false;
    }
    nodes
        .get(call.children[0].0 as usize)
        .is_some_and(|c| c.text.as_deref() == Some(ctor) && c.children.is_empty())
}

/// The single string-literal argument text (WITH delimiters) of a
/// `new <ctor>(<string-literal>)` construction, or `None` when the ctor does not
/// match, when there is not EXACTLY ONE argument, or when that argument is not a
/// string literal. Two-arg `new URL(rel, base)` and non-literal `new URL(s)`
/// both return `None` (not admitted → the declarator intercept denies). The arg
/// lives at `call.children[1]` (index 0 is the ctor identifier).
pub(crate) fn new_ctor_string_literal_arg(
    nodes: &[LirNode],
    init_id: LirNodeId,
    ctor: &str,
) -> Option<String> {
    let node = nodes.get(init_id.0 as usize)?;
    if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.len() != 1 {
        return None;
    }
    let call = nodes.get(node.children[0].0 as usize)?;
    // Exactly one arg → the Call has TWO children: [ctor, arg].
    if call.kind != LirNodeKind::Call || call.text.is_some() || call.children.len() != 2 {
        return None;
    }
    let ctor_node = nodes.get(call.children[0].0 as usize)?;
    if ctor_node.text.as_deref() != Some(ctor) || !ctor_node.children.is_empty() {
        return None;
    }
    let arg = nodes.get(call.children[1].0 as usize)?;
    if arg.kind != LirNodeKind::Literal {
        return None;
    }
    let text = arg.text.as_deref()?;
    let trimmed = text.trim();
    let first = trimmed.chars().next()?;
    let last = trimmed.chars().last()?;
    let is_string = (first == '"' && last == '"')
        || (first == '\'' && last == '\'')
        || (first == '`' && last == '`');
    is_string.then(|| text.to_string())
}

/// `true` iff `init_id` is `new URL(<string-literal>)`. The Task 3 declarator
/// intercept uses the arg-agnostic `declarator_init_is_url_ctor` (so a
/// non-admittable shape denies rather than silently placeholder-lowering);
/// this narrow admit-predicate is the interface Tasks 4-5 consume.
#[allow(dead_code)]
pub(crate) fn declarator_init_is_url_new(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    new_ctor_string_literal_arg(nodes, init_id, "URL").is_some()
}

/// `true` iff `init_id` is `new URLSearchParams(<string-literal>)`. Interface
/// for Tasks 4-5 (see `declarator_init_is_url_new`).
#[allow(dead_code)]
pub(crate) fn declarator_init_is_url_search_params_new(
    nodes: &[LirNode],
    init_id: LirNodeId,
) -> bool {
    new_ctor_string_literal_arg(nodes, init_id, "URLSearchParams").is_some()
}

/// Parse a URL string literal into its emitted components via the `url` crate.
/// Returns `None` when the literal does not parse as an absolute URL — the
/// declarator intercept then denies (a non-parseable literal is not admitted).
pub(crate) fn parse_url_literal(text: &str) -> Option<UrlComponents> {
    let u = url::Url::parse(text).ok()?;
    Some(UrlComponents {
        href: u.as_str().to_string(),
        origin: u.origin().ascii_serialization(),
        pathname: u.path().to_string(),
        search: u.query().map(|q| format!("?{q}")).unwrap_or_default(),
        hash: u.fragment().map(|f| format!("#{f}")).unwrap_or_default(),
        query_pairs: u
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect(),
    })
}

/// `application/x-www-form-urlencoded` decode (`+`→space, `%XX`) of a
/// URLSearchParams literal via `form_urlencoded`. Always succeeds (an empty or
/// malformed body yields whatever pairs decode, matching the WHATWG parser).
pub(crate) fn parse_query_literal(text: &str) -> Vec<(String, String)> {
    form_urlencoded::parse(text.as_bytes())
        .into_owned()
        .collect()
}

/// True when `init_id` is a PROVABLE ZERO-PLACEHOLDER construct — a
/// `new X()` whose constructor `X` has no real lowering, so the whole
/// construction lowers to the drop-and-push-`0` aggregate placeholder (the
/// "unsupported `new` returns an empty object" fallback; e.g.
/// `const c = new AbortController()`). Same New-wrapper LIR shape as
/// [`declarator_init_is_event_target_new`] — a text-less single-child `Value`
/// wrapping a text-less `Call` whose `children[0]` is the bare constructor
/// identifier — inspected RAW (the New node is itself the wrapper; unwrapping
/// transparent wrappers would strip it).
///
/// This is the ONE allowlist exception the deferred-callback choke point keeps
/// for a captured binding without closure lowering (Task 9 C-1 final): a
/// zero-placeholder construct reads `0` in its owner's own body too, so a
/// deferred read of the same `0` introduces NO divergence (unlike a real object
/// / scalar, whose value node computes and the deferred lane loses). The
/// constructors EXCLUDED here are exactly those a bound `new X()` declarator
/// lowers to a REAL value for — `Array`/`Uint8Array` (real linear-memory arrays;
/// see `is_array_like_constructor`) and `EventTarget` (a real host handle);
/// capturing one of those in a deferred callback WOULD diverge, so they stay
/// denied. (`CustomEvent`/`Event`/`TextEncoder` never reach here as a bare bound
/// `new X()` — they only appear inline in `dispatchEvent`/`.encode` chains.)
pub(crate) fn declarator_init_is_placeholder_construct(
    nodes: &[LirNode],
    init_id: LirNodeId,
) -> bool {
    let Some(node) = nodes.get(init_id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.len() != 1 {
        return false;
    }
    let Some(call) = nodes.get(node.children[0].0 as usize) else {
        return false;
    };
    if call.kind != LirNodeKind::Call || call.text.is_some() || call.children.is_empty() {
        return false;
    }
    let Some(ctor) = nodes.get(call.children[0].0 as usize) else {
        return false;
    };
    if !ctor.children.is_empty() {
        return false;
    }
    match ctor.text.as_deref() {
        // A real-value construct (see doc comment): NOT a zero placeholder.
        // `AbortController` joined this list in Stage P3 (real global abort
        // cell); its `const`-declarator lowering is intercepted at emit under
        // the `Repr::AbortHandle` proof + shadow guard.
        Some("Array" | "Uint8Array" | "EventTarget" | "AbortController") => false,
        // Any other bare `new X()` lowers to the drop-and-push-0 placeholder.
        Some(_) => true,
        None => false,
    }
}

/// True when `init_id` is a `dispatchEvent(...)` MEMBER call (Stage D event
/// lane). EMPIRICALLY-VERIFIED shape (KALI_DUMP_LIR, `const ok =
/// t.dispatchEvent(...)`): the init is the raw `Call(None, [Value("dispatchEvent",
/// [receiver]), event])`. A `const ok = t.dispatchEvent(...)` binding MUST be a
/// REAL local — the fold-alias tunnel (`FunctionEmitter::bindings`) would
/// re-emit the dispatch at every read of `ok`, re-invoking EVERY listener
/// synchronously each time — an observable duplicate-dispatch miscompile, not a
/// missed optimization. (The recognizer is receiver-agnostic: promoting an
/// out-of-lane dispatch binding to a local is a harmless slot reservation.)
pub(crate) fn declarator_init_is_event_dispatch(nodes: &[LirNode], init_id: LirNodeId) -> bool {
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
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(&callee) = node.children.first() else {
            return false;
        };
        return nodes.get(callee.0 as usize).is_some_and(|callee_node| {
            callee_node.text.as_deref() == Some("dispatchEvent") && !callee_node.children.is_empty()
        });
    }
}

/// Follows empty-text single-child `Value` wrapper nodes, mirroring
/// `FunctionEmitter::unwrap_transparent_value_node` over raw nodes.
pub(crate) fn unwrap_transparent_value_node_raw(nodes: &[LirNode], mut id: LirNodeId) -> LirNodeId {
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return id;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            id = node.children[0];
            continue;
        }
        return id;
    }
}

/// Raw-node mirror of `FunctionEmitter::is_process_argv`.
fn node_is_process_argv_raw(nodes: &[LirNode], id: LirNodeId) -> bool {
    let id = unwrap_transparent_value_node_raw(nodes, id);
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    if node.text.as_deref() != Some("argv") || node.children.len() != 1 {
        return false;
    }
    let root = unwrap_transparent_value_node_raw(nodes, node.children[0]);
    is_process_root(nodes, root)
}

/// Raw-node mirror of `FunctionEmitter::is_process_argv_element` (the two-child
/// computed-read shape `[process.argv, index]`, with the stringified integer
/// index carried in the second child's `text`).
fn node_is_process_argv_element(nodes: &[LirNode], node: &LirNode) -> bool {
    if node.kind != LirNodeKind::Value || node.children.len() != 2 {
        return false;
    }
    if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
        return false;
    }
    if !node_is_process_argv_raw(nodes, node.children[0]) {
        return false;
    }
    let Some(index) = nodes
        .get(node.children[1].0 as usize)
        .and_then(|child| child.text.as_deref())
        .and_then(parse_number_literal)
    else {
        return false;
    };
    index >= 0
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

/// Names whose binding PROVENANCE is unstable within `body`: assigned outside
/// their declarator (`name = …`, any compound/logical-assignment form) or
/// declared by MORE THAN ONE declarator (a block-level `let` re-declaration /
/// shadow). The scheduling-surface default-deny guard refuses to resolve such
/// a name through `fn_valued_locals` (recorded once, at declarator-emit time)
/// — a reassignment or shadow leaves that mapping stale, which is exactly the
/// fail-open the stage review's reassignment tripwire pinned. Deliberately
/// walks INTO nested function subtrees: a nested function reassigning an
/// outer binding invalidates the outer provenance just the same. Update
/// expressions (`++`/`--`) are excluded on purpose — they cannot make a
/// binding hold a (capturing) function value.
pub(crate) fn unstable_provenance_names(nodes: &[LirNode], body: LirNodeId) -> HashSet<String> {
    let mut unstable: HashSet<String> = HashSet::new();
    let mut declarator_counts: HashMap<String, u32> = HashMap::new();
    let mut stack = vec![body];
    let mut seen: HashSet<LirNodeId> = HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = nodes.get(id.0 as usize) else {
            continue;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 2
            && matches!(
                node.text.as_deref(),
                Some("=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "??=" | "&&=" | "||=")
            )
        {
            if let Some(name) = bare_assignment_target_name(nodes, node.children[0]) {
                unstable.insert(name);
            }
        }
        if node.kind == LirNodeKind::Instruction
            && matches!(node.text.as_deref(), Some("const" | "let" | "var"))
        {
            for declarator_id in &node.children {
                if let Some(name) = nodes
                    .get(declarator_id.0 as usize)
                    .and_then(|declarator| declarator.text.clone())
                {
                    *declarator_counts.entry(name).or_default() += 1;
                }
            }
        }
        stack.extend(node.children.iter().copied());
    }
    for (name, count) in declarator_counts {
        if count > 1 {
            unstable.insert(name);
        }
    }
    unstable
}

/// The bare-identifier assignment target under transparent single-child
/// `Value` wrappers, or `None` for member/element/complex targets (static
/// mirror of `FunctionEmitter::unwrap_transparent` + bare-identifier check).
fn bare_assignment_target_name(nodes: &[LirNode], mut id: LirNodeId) -> Option<String> {
    let mut guard = 0;
    loop {
        let node = nodes.get(id.0 as usize)?;
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            id = node.children[0];
            guard += 1;
            if guard > 64 {
                return None;
            }
            continue;
        }
        return if node.kind == LirNodeKind::Value && node.children.is_empty() {
            node.text.clone()
        } else {
            None
        };
    }
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

/// Name of the dedicated i32 scratch local holding the `__alloc_global` buffer
/// pointer for a `process.argv[<int>]` element read (Spec 5 Task 5). Reserved
/// once per function that contains any such read; shared by
/// `collect_function_locals` (reserve) and `emit_unary`'s element-read emit
/// (resolve by name). The `#argv` suffix is unrepresentable as a source
/// identifier, so it never collides with a real binding.
pub(crate) fn argv_buf_local_name() -> String {
    "__argv_buf".to_string()
}

/// Name of the dedicated i32 scratch local holding the byte length returned by
/// `args_get` for a `process.argv[<int>]` element read (Spec 5 Task 5). Sibling
/// of `argv_buf_local_name`; same reserve/resolve discipline.
pub(crate) fn argv_len_local_name() -> String {
    "__argv_len".to_string()
}

/// True for a scratch local synthesized for a `process.argv` element read —
/// both hold i32 values (`__alloc_global` pointer / `args_get` byte count) and
/// must be declared as i32 locals, like `is_arena_save_local_name`, rather than
/// the `Repr::I64` default an unrecorded name would otherwise get.
pub(crate) fn is_argv_scratch_local_name(name: &str) -> bool {
    name == argv_buf_local_name() || name == argv_len_local_name()
}

/// Names of the three dedicated i64 scratch locals `emit_string_to_i64_parse`
/// uses to decode a tagged runtime-string handle and accumulate a base-10
/// parse (fasta Spec 5 Task 6, `+<runtime string>`): `__coerce_ptr` (byte
/// cursor), `__coerce_end` (one past the last byte), `__coerce_acc` (handle
/// scratch, then running accumulator). All three are `i64` — the default
/// `wasm_type(repr_table.scalar(...))` an unrecorded name gets already
/// resolves to `Repr::I64`, so (unlike `argv_buf_local_name`/
/// `argv_len_local_name`, which need i32 and so need `is_argv_scratch_local_name`
/// to force that) no special-casing is needed in the local-decl `val_type`
/// lookup. Shared by `collect_function_locals` (reserve) and
/// `emit_string_to_i64_parse` (resolve by name), so the two cannot disagree on
/// naming. The `__coerce_*` names are unrepresentable as source identifiers,
/// so they can never collide with a real binding.
pub(crate) fn coerce_ptr_local_name() -> String {
    "__coerce_ptr".to_string()
}

/// Sibling of `coerce_ptr_local_name`; see its doc comment.
pub(crate) fn coerce_end_local_name() -> String {
    "__coerce_end".to_string()
}

/// Sibling of `coerce_ptr_local_name`; see its doc comment.
pub(crate) fn coerce_acc_local_name() -> String {
    "__coerce_acc".to_string()
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

/// Pre-order, function-scoped ordinal for each *string-producing site* in the
/// LIR tree rooted at `body`: the `k`-th such node (in child-array pre-order,
/// numbered BEFORE descending its own children) gets ordinal `k` (0-based); a
/// nested function body is an opaque leaf (never descended into — each function
/// numbers its own sites independently).
///
/// ## THE STRING-SITE ORDINAL RULE (codegen mirror of the 4b oracle)
///
/// This is the **codegen half of the both-sides oracle** whose analysis half —
/// which RECORDS the grants this walk QUERIES — is
/// `kali_mir::analysis::arena_gate::OwnershipAnalyzer::arena_collect_string_sites`.
/// Read that function's "THE STRING-SITE ORDINAL RULE" doc comment: it is the
/// authoritative definition; this walk MUST enumerate the identical node set in
/// the identical order or `arena_string_site(fn, ord)` lookups desync and a
/// join escaping into the resettable arena becomes a use-after-reset (fail
/// OPEN). Correspondence rests on every HIR→MIR→LIR lowering being a 1:1
/// structural copy (same node count, same child order, same `text` — see
/// `kali_mir::lower` and `kali_lir::lower`), exactly as `loop_preorder_ordinals`
/// relies on for loop ordinals.
///
/// A **string-producing site** (`is_string_site`) is EITHER:
///   1. `recv.join(sep)` — a `Call` whose callee (first child) is a MemberExpr
///      with property text `"join"`, OR
///   2. ANY `+` `BinaryExpr` (numeric `+` included — the site set is purely
///      syntactic so both sides agree without re-deriving string types here).
///
/// LIR collapses many distinct HIR expression kinds into a single `Value` node
/// carrying the operator/property `text`, so the two HIR shapes 4b matches on
/// `kind` are reconstructed from `(LirNodeKind, text, child-arity)`:
///   - `+` `BinaryExpr` → `Value` text `"+"` with **exactly 2 children**
///     (operands). Unary plus `+x` (`UnaryExpr`, e.g. `+process.argv[2]`) is
///     also `Value` text `"+"` but has **1 child**, so the arity check excludes
///     it — mirroring 4b's `kind == BinaryExpr`.
///   - `.join` MemberExpr callee → the `Call`'s first child is a `Value` text
///     `"join"` with **>= 1 children** (a MemberExpr always has its receiver as
///     a child; a bare-identifier callee named `join` is a childless `Value`
///     and must NOT match) — mirroring 4b's `callee.kind == MemberExpr`.
///
/// Template literals are deliberately NOT sites (out of 4b's scope) — they lower
/// to their own `Value` node, never to `+`, so neither side numbers them.
///
/// KNOWN RESIDUAL (fail-closed in practice, unreachable in the supported
/// subset): a *computed* member access `a["+"]` — property literally `"+"` —
/// also lowers to a 2-child `Value` text `"+"` and would be over-counted here
/// but not by 4b (`kind == MemberExpr`). No `"+"`-named field exists in kali's
/// fixed-shape object model, so this cannot occur in any supported program; the
/// only realistic 2-child `Value("+")` is a `BinaryExpr`.
pub(crate) fn string_site_preorder_ordinals(
    nodes: &[LirNode],
    body: LirNodeId,
) -> HashMap<LirNodeId, u32> {
    let mut ordinals = HashMap::new();
    let mut next = 0u32;
    string_site_preorder_ordinals_walk(nodes, body, &mut next, &mut ordinals);
    ordinals
}

/// Whether `id` is a string-producing site — see the ordinal rule on
/// [`string_site_preorder_ordinals`]. Mirrors 4b's `is_string_producing_site`.
fn is_string_site(nodes: &[LirNode], id: LirNodeId) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    match node.kind {
        LirNodeKind::Call => node
            .children
            .first()
            .and_then(|c| nodes.get(c.0 as usize))
            .is_some_and(|callee| {
                callee.text.as_deref() == Some("join") && !callee.children.is_empty()
            }),
        LirNodeKind::Value => node.text.as_deref() == Some("+") && node.children.len() == 2,
        _ => false,
    }
}

fn string_site_preorder_ordinals_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    next: &mut u32,
    ordinals: &mut HashMap<LirNodeId, u32>,
) {
    if is_string_site(nodes, id) {
        ordinals.insert(id, *next);
        *next += 1;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        string_site_preorder_ordinals_walk(nodes, *child, next, ordinals);
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

    // Names rebound anywhere in this function (including inside nested
    // closures): a `const` initializer that reads one cannot stay on the
    // re-emitting fold lane. Computed once for the whole body.
    let reassigned = program_reassigned_names(nodes);

    let mut locals = Vec::new();
    let mut seen = HashSet::new();
    collect_function_locals_from_node(
        nodes,
        body_id,
        &array_names,
        &reassigned,
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
        // OR the emit-only `string_arena_loop` channel (fasta Spec 7 Task 4f)
        // into the SAME save-locals reservation gate `emit_loop`'s `is_arena_loop`
        // reads: a loop opens a per-iteration arena if EITHER the object/array
        // `loop_arena` channel OR the string-site channel grants it. Both key on
        // this identical pre-order loop ordinal, so a loop granted by both still
        // reserves exactly one save-locals triple (single-open by construction).
        if arena_table.loop_arena(function_name, ordinal)
            || arena_table.string_arena_loop(function_name, ordinal)
        {
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
        // fasta Spec 7 Task 4g: no per-for-in key-table BASE local is reserved
        // anymore — the table is module-constant data referenced by a fixed
        // offset (`StringPool::intern_key_table`), not a bump-allocated base
        // held in a runtime local.
    }

    // Reserve the two i32 scratch locals for a `process.argv[<int>]` element
    // read (Spec 5 Task 5): the `__alloc_global` buffer pointer and the
    // `args_get` byte length. One shared pair per function suffices — the
    // element-read emit sequence consumes both before any later read could
    // overwrite them. Guarded by the SAME argv-element shape walk the emit
    // recognizer keys on, so the slots exist iff the emit path can fire.
    if body_contains_process_argv_element(nodes, body_id) {
        locals.push(argv_buf_local_name());
        locals.push(argv_len_local_name());
    }

    // Reserve the three i64 scratch locals `emit_string_to_i64_parse` needs
    // (fasta Spec 5 Task 6, `+<runtime string>`). Guarded by a STRUCTURAL
    // "does this body contain any syntactic unary `+`" walk rather than a
    // precise "is the operand provably string-valued" oracle: the precise
    // oracle (`FunctionEmitter::is_string_valued`) depends on `self.bindings`
    // (the fold-lane const-alias map), which does not exist yet at this
    // locals-provisioning stage — collect_function_locals only has the raw
    // LIR + `ReprTable`. Reserving on the coarser superset (every unary `+`,
    // not just the string-operand subset the emitter will actually route
    // through `emit_string_to_i64_parse`) means the reservation can only ever
    // be too generous, never too stingy: the emit-time `self.locals[&name]`
    // lookups in `emit_string_to_i64_parse` can never miss their slot and
    // panic. The cost is a few unused-but-harmless locals declared for a
    // `+<numeric>` in the same function — WASM validation does not care about
    // unused locals.
    if body_contains_unary_plus(nodes, body_id) {
        locals.push(coerce_ptr_local_name());
        locals.push(coerce_end_local_name());
        locals.push(coerce_acc_local_name());
    }

    // Reserve the dedicated i64 scratch local the growable-array emit
    // helpers need (throw-fallout Stage 4): `emit_growable_alloc` holds the
    // header pointer across seed-element emission and `emit_growable_push`
    // across value emission — both of which may internally clobber the two
    // generic trailing scratch slots. One slot per function suffices (the
    // helpers never nest their own use of it across an `emit_node` call).
    // Guarded on the function actually having a growable binding, so
    // functions without one are byte-identical.
    //
    // Stage P2 Lane 1 Task 5 adds a SECOND trigger: a growable-array FIELD push
    // (`o.values.push(v)`) also flows through `emit_growable_push` and so needs
    // the same scratch — but the enclosing function has no growable BINDING to
    // key on. Reserve on a COARSE structural superset (`body_contains_field_push`
    // — any `.push` on a member-expression receiver), matching the
    // argv/unary-plus reservation philosophy: over-reserving a harmless unused
    // i64 local is always safe; under-reserving would panic in
    // `growable_scratch_local`.
    // ALSO reserve when this function allocates an object literal carrying a
    // `GrowableArrayI64` field: `emit_growable_field_value` (object.rs) uses the
    // dedicated scratch to allocate+seed the field's growable array. Detected
    // precisely off the shape table (a binding or return repr that is
    // `Object(shape)` with a growable field), so unrelated functions stay
    // byte-identical.
    let allocates_growable_object_field = locals.iter().any(|name| {
        matches!(
            repr_table.scalar(function_name, name),
            kali_common::Repr::Object(shape) if shape_has_growable_field(repr_table, shape)
        )
    }) || matches!(
        repr_table.return_repr(function_name),
        kali_common::Repr::Object(shape) if shape_has_growable_field(repr_table, shape)
    );
    // Stage P4: a URL or URLSearchParams construction builds an embedded /
    // standalone growable `[k0,v0,…]` pair-store via `emit_growable_alloc`,
    // which requires the dedicated growable scratch. Reserve it whenever this
    // function owns a `Repr::Url`/`Repr::UrlSearchParams` binding (over-reserving
    // an unused i64 local is harmless; under-reserving panics in
    // `growable_scratch_local`).
    let constructs_url_or_usp = locals.iter().any(|name| {
        matches!(
            repr_table.scalar(function_name, name),
            kali_common::Repr::Url | kali_common::Repr::UrlSearchParams
        )
    });
    if locals
        .iter()
        .any(|name| repr_table.is_growable_array_binding(function_name, name))
        || body_contains_field_push(nodes, body_id)
        || allocates_growable_object_field
        || constructs_url_or_usp
    {
        locals.push(growable_scratch_local_name());
    }

    // Reserve a real i64 loop-variable local for every runtime `for..of` over a
    // growable array (throw-fallout Stage 4 Task 4), plus the shared
    // index/length counter pair — but only when at least one such loop exists,
    // so functions without one stay byte-identical. The static-unroll lane
    // binds its loop var to a compile-time node; the runtime counted loop needs
    // a wasm LOCAL so the body's reads of the loop var resolve to it.
    let for_of_growable_vars =
        for_of_growable_loop_var_names(nodes, body_id, repr_table, function_name);
    if !for_of_growable_vars.is_empty() {
        for var in for_of_growable_vars {
            if !locals.contains(&var) {
                locals.push(var);
            }
        }
        locals.push(growable_foreach_index_local_name());
        locals.push(growable_foreach_len_local_name());
    }

    locals
}

/// Name of the dedicated i64 scratch local shared by the growable-array emit
/// helpers (throw-fallout Stage 4). Shared by `collect_function_locals`
/// (reserve) and `crate::emit::growable` (resolve) — same discipline as
/// `for_in_ord_local_name` and the argv scratch pair. Default-typed i64 (no
/// `ReprTable` entry), which is exactly what the helpers store in it.
pub(crate) fn growable_scratch_local_name() -> String {
    "__growable_scratch".to_string()
}

/// Name of the shared i64 loop-index counter local for the runtime `for..of`
/// over a growable array (throw-fallout Stage 4 Task 4). ONE shared slot per
/// function suffices: the counted-loop lane fails closed on a growable
/// `for..of` lexically NESTED inside another (see the emit guard), so two
/// growable `for..of` loops in the same function never overlap in time and can
/// reuse the same counter/length scratch. Reserved by `collect_function_locals`
/// and resolved by `emit_for_of_array_iteration`'s runtime branch.
pub(crate) fn growable_foreach_index_local_name() -> String {
    "__growable_foreach_index".to_string()
}

/// Name of the shared i64 snapshot-length local for the runtime `for..of` over
/// a growable array (throw-fallout Stage 4 Task 4). The iteration count is
/// snapshotted ONCE before the loop (per the design), so a body that pushes to
/// a DIFFERENT array is unaffected; see `growable_foreach_index_local_name` for
/// why one shared slot per function is sound.
pub(crate) fn growable_foreach_len_local_name() -> String {
    "__growable_foreach_len".to_string()
}

/// Loop-variable names of every `for..of` (`for-await-of` excluded — it fails
/// closed) whose iterable is a bare-identifier GROWABLE array binding of
/// `function_name`, reachable from `body_id` without descending into nested
/// function bodies (throw-fallout Stage 4 Task 4). Each such loop var must get
/// a real wasm LOCAL so the body's reads resolve to it (the static-unroll lane
/// binds the loop var to a compile-time node instead). Structural twin of the
/// emit-side runtime-lane guard in `emit_for_of_array_iteration`: both key on
/// "bare identifier iterable + `is_growable_array_binding`", so the reservation
/// exists exactly where the runtime branch resolves a slot. Element repr is NOT
/// consulted here — a String-element growable `for..of` never reaches codegen
/// (the resolve gate aborts the compile), and an over-reserved unused local is
/// harmless.
/// True iff `id` is a dot member `base.field` (1-child `Value`, `base` a bare
/// identifier) whose `base` has an `Object(shape)` repr with a
/// `GrowableArrayI64` field named `field` (Stage P2 Lane 1). The
/// provisioning-stage free-function mirror of the emitter's
/// `object_field_is_growable_array` (which resolves through the same
/// scalar/shape tables); the direct identifier-base form is all the growable
/// field-receiver surface admits.
fn node_is_growable_i64_field(
    nodes: &[LirNode],
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    id: LirNodeId,
) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Value || node.children.len() != 1 {
        return false;
    }
    let Some(field) = node.text.as_deref().filter(|t| !t.is_empty()) else {
        return false;
    };
    let Some(base) = nodes.get(node.children[0].0 as usize) else {
        return false;
    };
    if base.kind != LirNodeKind::Value || !base.children.is_empty() {
        return false;
    }
    let Some(base_name) = base.text.as_deref().filter(|t| !t.is_empty()) else {
        return false;
    };
    match repr_table.scalar(function_name, base_name) {
        kali_common::Repr::Object(shape) => matches!(
            repr_table.shape_field(shape, field),
            Some((_, kali_common::Repr::GrowableArrayI64))
        ),
        _ => false,
    }
}

fn for_of_growable_loop_var_names(
    nodes: &[LirNode],
    body_id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    for_of_growable_loop_var_names_walk(nodes, body_id, repr_table, function_name, &mut names);
    names
}

/// Raw-node twin of `FunctionEmitter::for_of_binding_name_from_node`: the
/// loop-variable name of a `for..of` left node, whether a `const`/`let`/`var`
/// declaration or a bare identifier, transparently unwrapping empty-text /
/// `await` `Value` wrappers. Kept in lockstep with the emitter version so the
/// reserved local and the resolved slot always name the same binding.
fn for_of_loop_var_name_of(nodes: &[LirNode], id: LirNodeId) -> Option<String> {
    let node = nodes.get(id.0 as usize)?;
    if node.children.is_empty() {
        return node.text.clone().filter(|t| !t.is_empty());
    }
    if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
        let declarator = *node.children.first()?;
        return nodes
            .get(declarator.0 as usize)
            .and_then(|n| n.text.clone())
            .filter(|t| !t.is_empty());
    }
    if node.text.as_deref().is_some_and(|t| t.is_empty()) && !node.children.is_empty() {
        return for_of_loop_var_name_of(nodes, *node.children.last()?);
    }
    if (node.text.is_none() || node.text.as_deref() == Some("await")) && node.children.len() == 1 {
        return for_of_loop_var_name_of(nodes, node.children[0]);
    }
    None
}

fn for_of_growable_loop_var_names_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    names: &mut Vec<String>,
) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("for-of") {
        let iterable_is_growable = node.children.get(1).is_some_and(|&iterable| {
            bare_identifier_name_of(nodes, iterable)
                .is_some_and(|name| repr_table.is_growable_array_binding(function_name, &name))
                // Stage P2 Lane 1 Task 5: a `for (const x of o.values)` over a
                // `GrowableArrayI64` object field also runs the counted growable
                // loop, so its loop var needs a real wasm local reserved here.
                || node_is_growable_i64_field(nodes, repr_table, function_name, iterable)
        });
        if iterable_is_growable {
            if let Some(var) = node
                .children
                .first()
                .and_then(|&left| for_of_loop_var_name_of(nodes, left))
            {
                if !names.contains(&var) {
                    names.push(var);
                }
            }
        }
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        for_of_growable_loop_var_names_walk(nodes, *child, repr_table, function_name, names);
    }
}

/// True iff any bare identifier reachable from `init` (not descending into
/// nested function bodies) names a GROWABLE array binding of `function_name`
/// (throw-fallout Stage 4). Coarse ON PURPOSE (any mention, not just
/// `.length`/index read shapes): it only ever PROMOTES a const to an
/// eagerly-evaluated local, which is the semantically-correct JS evaluation
/// order for every init.
fn declarator_init_mentions_growable(
    nodes: &[LirNode],
    id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    if node.kind == LirNodeKind::Value && node.children.is_empty() {
        if let Some(text) = node.text.as_deref() {
            if repr_table.is_growable_array_binding(function_name, text) {
                return true;
            }
        }
    }
    node.children.iter().any(|child| {
        !is_function_like(nodes, *child)
            && declarator_init_mentions_growable(nodes, *child, repr_table, function_name)
    })
}

/// True iff any node reachable from `body_id` (not descending into nested
/// function bodies) is a syntactic unary `+` (a single-child `Value` node
/// whose `text` is exactly `"+"`) — the same node shape `emit_unary`
/// dispatches on. See the call site in `collect_function_locals` for why this
/// is deliberately a coarse superset of "unary `+` over a provably
/// string-valued operand" rather than a precise mirror of it.
/// True iff any node reachable from `body_id` (not descending into nested
/// function bodies) is a `.push` member whose receiver is a member-expression
/// (field-read) shape — the coarse superset that guards reserving the growable
/// scratch local for a `GrowableArrayI64` FIELD push (Stage P2 Lane 1 Task 5).
/// Deliberately imprecise (fires for any `x.y.push(...)`, growable or not):
/// over-reserving one unused i64 local is harmless; the precise shape proof
/// (`object_field_is_growable_array`) is unavailable at this provisioning stage.
/// True iff `shape` has any `GrowableArrayI64` field (Stage P2 Lane 1 Task 5) —
/// the signal that allocating this object shape needs the growable scratch local
/// (`emit_growable_field_value`).
fn shape_has_growable_field(
    repr_table: &kali_common::ReprTable,
    shape: kali_common::ShapeId,
) -> bool {
    repr_table
        .shape_fields(shape)
        .iter()
        .any(|(_, repr)| matches!(repr, kali_common::Repr::GrowableArrayI64))
}

fn body_contains_field_push(nodes: &[LirNode], body_id: LirNodeId) -> bool {
    fn is_field_read_shape(nodes: &[LirNode], id: LirNodeId) -> bool {
        nodes.get(id.0 as usize).is_some_and(|n| {
            n.kind == LirNodeKind::Value
                && n.children.len() == 1
                && n.text.as_deref().is_some_and(|t| !t.is_empty())
        })
    }
    fn walk(nodes: &[LirNode], id: LirNodeId) -> bool {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        // `.push` member callee: Value{text:"push", children:[receiver]} whose
        // receiver is itself a member-expression (field read).
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("push")
            && node.children.len() == 1
            && is_field_read_shape(nodes, node.children[0])
        {
            return true;
        }
        node.children.iter().any(|child| {
            if is_function_like(nodes, *child) {
                return false;
            }
            walk(nodes, *child)
        })
    }
    walk(nodes, body_id)
}

fn body_contains_unary_plus(nodes: &[LirNode], body_id: LirNodeId) -> bool {
    fn walk(nodes: &[LirNode], id: LirNodeId) -> bool {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref() == Some("+")
        {
            return true;
        }
        node.children.iter().any(|child| {
            if is_function_like(nodes, *child) {
                return false;
            }
            walk(nodes, *child)
        })
    }
    walk(nodes, body_id)
}

/// True iff any node reachable from `body_id` (not descending into nested
/// function bodies) is a `process.argv[<int literal>]` element read. Mirrors
/// `node_is_process_argv_element` used by the program-wide import probe, but
/// scoped to one function body so its scratch locals are reserved only where
/// needed.
fn body_contains_process_argv_element(nodes: &[LirNode], body_id: LirNodeId) -> bool {
    fn walk(nodes: &[LirNode], id: LirNodeId) -> bool {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node_is_process_argv_element(nodes, node) {
            return true;
        }
        node.children.iter().any(|child| {
            if is_function_like(nodes, *child) {
                return false;
            }
            walk(nodes, *child)
        })
    }
    walk(nodes, body_id)
}

/// WASM globals reserved before any module-scope mutable scalar global:
/// g0 (heap/page frontier) + g1..g7 (arena page-pool state) + g8
/// (`current_env`, Stage C closures — see `crate::closure::CURRENT_ENV_GLOBAL`).
/// Module scalar globals are appended AFTER these, at indices
/// `RESERVED_GLOBAL_COUNT`, +1, … — raising this count keeps their indices
/// contiguous ABOVE the newly reserved global with no shift to g0..g7 or to
/// any existing module-scalar index's *relative* order (only the base moves).
pub(crate) const RESERVED_GLOBAL_COUNT: u32 = 9;

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

    // POSITIVE scalar proof (the load-bearing gate). A candidate's repr may be
    // a *default* `I64` for an object whose shape was never proven (e.g. a
    // module object reassigned `o = {…}` but never member-accessed), so the
    // I64/`is_array_binding`/member-base checks above are not sufficient — they
    // are a NEGATIVE heuristic an unproven heap value evades. Promote ONLY when
    // the binding is PROVABLY numeric: its initializer AND every reassignment
    // RHS is a numeric expression (numeric literal, arithmetic/bitwise over
    // numerics, a proven-numeric call, `.length`, …). Any object/array literal,
    // string, template, `new`, or non-numeric-return call as an init/RHS leaves
    // the name unpromoted → the existing E5506 module-binding gate = fail-closed
    // (the safe pre-feature behavior). The capstone's `var rngLast = 42`
    // (reassigned only to `(rngLast*3877+29573)%139968`) is provably numeric.
    let mut numeric_ok: HashSet<String> = candidates.keys().cloned().collect();
    {
        let mut seen = HashSet::new();
        scan_numeric_assignments(
            &lir.nodes,
            lir.root,
            "_start",
            repr_table,
            &candidates,
            &mut seen,
            &mut numeric_ok,
        );
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
        if referenced.contains(&name)
            && !heap_base_names.contains(&name)
            && numeric_ok.contains(&name)
        {
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

/// A binary operator whose RESULT is always a number (arithmetic, bitwise,
/// relational, and equality). Used by the positive-scalar promotion proof:
/// `&&`/`||`/`??` (yield one operand, possibly heap), `,` (yields the RHS),
/// and `in`/`instanceof` are deliberately NOT here, so a candidate whose RHS
/// uses them is left unproven (fail-closed).
fn is_numeric_result_binary_operator(text: &str) -> bool {
    matches!(
        text,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "&"
            | "|"
            | "^"
            | "<<"
            | ">>"
            | ">>>"
            | "<"
            | "<="
            | ">"
            | ">="
            | "=="
            | "==="
            | "!="
            | "!=="
    )
}

/// An assignment operator (`=` and the compound forms). LHS-targeted; used to
/// find every reassignment of a promotion candidate.
fn is_assignment_operator_text(text: &str) -> bool {
    matches!(
        text,
        "=" | "+="
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

/// POSITIVE proof that the expression at `id` yields a plain NUMBER (so a module
/// binding whose init and every reassignment RHS is numeric can be backed by an
/// i64/f64 global). Conservative: anything not recognized as numeric returns
/// `false` (→ the binding is left unpromoted → fail-closed E5506). Rejects the
/// heap shapes — object/array literal (a childless-text `Value` with children),
/// string/other non-numeric literal, `.field` member access (except `.length`),
/// computed member, and a call whose return repr is not proven numeric.
/// `func` scopes identifier-operand repr lookups to the enclosing function.
fn is_numeric_expr(
    nodes: &[LirNode],
    id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    func: &str,
    depth: usize,
) -> bool {
    if depth > 32 {
        return false;
    }
    let id = unwrap_transparent_value(nodes, id);
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    match node.kind {
        LirNodeKind::Literal => node.text.as_deref().is_some_and(|t| {
            parse_numeric_literal_value(t).is_some() || matches!(t, "true" | "false")
        }),
        LirNodeKind::Call => {
            // Only a plain named call (`f(...)`) whose return repr is proven
            // numeric. A member call (`o.m()`) or an unproven return is rejected.
            let Some(&callee_id) = node.children.first() else {
                return false;
            };
            let callee = unwrap_transparent_value(nodes, callee_id);
            let Some(callee_node) = nodes.get(callee.0 as usize) else {
                return false;
            };
            if callee_node.kind == LirNodeKind::Value && callee_node.children.is_empty() {
                if let Some(name) = callee_node.text.as_deref() {
                    return matches!(
                        repr_table.return_repr(name),
                        kali_common::Repr::I64 | kali_common::Repr::F64
                    );
                }
            }
            false
        }
        LirNodeKind::Value => {
            // Object/array literal: no text + children → heap aggregate.
            if node.text.is_none() && !node.children.is_empty() {
                return false;
            }
            match node.children.len() {
                0 => {
                    let Some(t) = node.text.as_deref() else {
                        return false;
                    };
                    if parse_numeric_literal_value(t).is_some() || matches!(t, "true" | "false") {
                        return true;
                    }
                    // Bare identifier operand: numeric iff its scalar repr is
                    // numeric AND it is not an array binding. An object/string
                    // binding (repr `Object`/`String`) is rejected here.
                    matches!(
                        repr_table.scalar(func, t),
                        kali_common::Repr::I64 | kali_common::Repr::F64
                    ) && !repr_table.is_array_binding(func, t)
                }
                1 => {
                    let t = node.text.as_deref().unwrap_or_default();
                    if t == "length" {
                        // `a.length` / `s.length` is a number.
                        return true;
                    }
                    if matches!(
                        t,
                        "-" | "+" | "~" | "!" | "prefix++" | "postfix++" | "prefix--" | "postfix--"
                    ) {
                        return is_numeric_expr(
                            nodes,
                            node.children[0],
                            repr_table,
                            func,
                            depth + 1,
                        );
                    }
                    // `o.field` / `typeof` / `void` / `delete` → not proven numeric.
                    false
                }
                2 => {
                    let t = node.text.as_deref().unwrap_or_default();
                    is_numeric_result_binary_operator(t)
                        && is_numeric_expr(nodes, node.children[0], repr_table, func, depth + 1)
                        && is_numeric_expr(nodes, node.children[1], repr_table, func, depth + 1)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Walk the whole program (tracking the enclosing function so identifier-operand
/// reprs resolve in the right scope) and, for every declarator init and every
/// reassignment RHS of a promotion `candidate`, drop the name from `numeric_ok`
/// if that init/RHS is not provably numeric (`is_numeric_expr`).
#[allow(clippy::too_many_arguments)]
fn scan_numeric_assignments(
    nodes: &[LirNode],
    id: LirNodeId,
    func: &str,
    repr_table: &kali_common::ReprTable,
    candidates: &BTreeMap<String, kali_common::Repr>,
    seen: &mut HashSet<LirNodeId>,
    numeric_ok: &mut HashSet<String>,
) {
    if !seen.insert(id) {
        return;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };

    // Function boundary: descend into the body under the function's own name so
    // a reassignment's identifier operands resolve in that scope.
    if let Some((name, _, body_id, _)) = function_shape(nodes, id) {
        scan_numeric_assignments(
            nodes, body_id, &name, repr_table, candidates, seen, numeric_ok,
        );
        return;
    }

    // Declarator init (`var`/`let name = init`).
    if node.kind == LirNodeKind::Instruction && matches!(node.text.as_deref(), Some("let" | "var"))
    {
        for declarator_id in &node.children {
            let Some(declarator) = nodes.get(declarator_id.0 as usize) else {
                continue;
            };
            let Some(name) = declarator.text.as_deref() else {
                continue;
            };
            if candidates.contains_key(name) && numeric_ok.contains(name) {
                if let Some(&init) = declarator.children.get(1) {
                    if !is_numeric_expr(nodes, init, repr_table, func, 0) {
                        numeric_ok.remove(name);
                    }
                }
            }
        }
    }

    // Reassignment (`name = rhs`, or a compound `name op= rhs`): its RHS must
    // also be numeric (a compound over a numeric global stays numeric iff the
    // RHS is numeric).
    if node.kind == LirNodeKind::Value
        && node.children.len() == 2
        && is_assignment_operator_text(node.text.as_deref().unwrap_or_default())
    {
        let lhs = unwrap_transparent_value(nodes, node.children[0]);
        if let Some(lhs_node) = nodes.get(lhs.0 as usize) {
            if lhs_node.kind == LirNodeKind::Value && lhs_node.children.is_empty() {
                if let Some(name) = lhs_node.text.as_deref() {
                    if candidates.contains_key(name)
                        && numeric_ok.contains(name)
                        && !is_numeric_expr(nodes, node.children[1], repr_table, func, 0)
                    {
                        numeric_ok.remove(name);
                    }
                }
            }
        }
    }

    for child in &node.children {
        scan_numeric_assignments(
            nodes, *child, func, repr_table, candidates, seen, numeric_ok,
        );
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

/// How a `const` declarator's initializer must be lowered — the SINGLE
/// promotion decision, shared by the local-slot collector
/// (`collect_function_locals_from_node`) and the emitter's denotation map
/// (`allowlist_promoted_const_names`). Keeping one function rather than two
/// mirrored predicates is deliberate: this codebase has repeatedly fail-opened
/// where a codegen oracle and its twin drifted apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConstPromotion {
    /// Stays on the compile-time fold lane: no local slot, and the init node is
    /// recorded in `FunctionEmitter::bindings` so reads re-emit it. Sound only
    /// because `const_init_is_stable` proved re-emission observationally
    /// identical.
    Fold,
    /// Promoted because the value is a runtime HANDLE needing stable storage (a
    /// fresh allocation, a host registration, a materialized object). These
    /// lanes key on their own provenance sets and must NOT get a `bindings`
    /// entry — recording one re-resolves the name to its init node and defeats
    /// the handle lane (this regressed the `TextEncoder`/`crypto` family once).
    Handle,
    /// Promoted because re-emitting the initializer is NOT observationally
    /// identical — it would repeat a side effect or read state that has since
    /// changed. Gets both a local slot (the runtime value, bound exactly once at
    /// the declaration) and a `bindings` entry (compile-time denotation only;
    /// the identifier read path consults `locals` first).
    Binding,
}

/// Decides how one `const` declarator is lowered. `declarator` is the
/// declarator node id; returns `None` when it has no initializer.
pub(crate) fn const_declarator_promotion(
    nodes: &[LirNode],
    declarator: LirNodeId,
    array_names: &HashSet<String>,
    reassigned: &HashSet<String>,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
) -> Option<(String, ConstPromotion)> {
    let declarator_node = nodes.get(declarator.0 as usize)?;
    let init = declarator_node.children.get(1).copied()?;
    let name = declarator_node.text.clone()?;
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
    let is_materialized_object_array = declarator_node.text.as_deref().is_some_and(|name| {
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
    let is_materialized_factory_return = declarator_node.text.as_deref().is_some_and(|name| {
        match repr_table.scalar(function_name, name) {
            kali_common::Repr::Object(_) => declarator_init_call_callee_name(nodes, init)
                .is_some_and(|callee| {
                    matches!(repr_table.return_repr(callee), kali_common::Repr::Object(_))
                }),
            _ => false,
        }
    });
    // A growable (push-accumulated) array binding needs a stable
    // local slot for its tagged handle regardless of `const`
    // (throw-fallout Stage 4) — push/length/index all read it back.
    let is_growable_array = declarator_node
        .text
        .as_deref()
        .is_some_and(|name| repr_table.is_growable_array_binding(function_name, name));
    // A const whose init READS a growable array (`const n = o.length`
    // / `o[i]`) must be evaluated EAGERLY into a local: the fold-lane
    // alias would re-emit the read at every use site, observing a
    // LATER length/element after more pushes — a stale-alias
    // miscompile (the growable twin of `declarator_init_is_array_read`).
    let reads_growable_array =
        declarator_init_mentions_growable(nodes, init, repr_table, function_name);
    // Scheduling registration (`const t = setTimeout(...)` /
    // `setInterval(...)`, Stage D task D2; also `queueMicrotask(...)`,
    // Stage D task D2's earlier microtask lane) is a SIDE-EFFECTING
    // host call — exactly like `declarator_init_is_performance_now`/
    // `_is_crypto_call` above, just a bare-identifier call rather than
    // a member call, so `declarator_init_call_callee_name` (already
    // used for the factory-return check above) recognizes its shape
    // directly. Before each surface's registration emit landed, it
    // lowered through a dropped zero-placeholder fallback, so
    // re-emitting the call at every read site of a bound name was a
    // harmless no-op; now each is a REAL host call, and without
    // promotion the `const` fold-alias tunnel (`FunctionEmitter::
    // bindings`) re-emits the ORIGINAL call at each later use site of
    // the bound name, registering a SECOND independent
    // timer/microtask — a duplicate-registration miscompile (for
    // setTimeout/setInterval: the pending first timer is never
    // cancelled if the read site is a `clearTimeout`/`clearInterval`
    // call; for queueMicrotask: its callback runs TWICE, reproduced
    // via `const m = queueMicrotask(fn); console.log(m);` — `fn` ran
    // twice on the pre-fix HEAD), not merely a missed optimization.
    // `queueMicrotask` is included here even though task D2's own
    // fixtures never bind its (always-`undefined`) return value,
    // because it is the exact same mechanism at the exact same choke
    // point — closing the class, not just the setTimeout/setInterval
    // instances of it.
    let is_scheduling_registration_call = matches!(
        declarator_init_call_callee_name(nodes, init),
        Some("setTimeout") | Some("setInterval") | Some("queueMicrotask")
    );
    // Stage D event lane: `const t = new EventTarget()` is a
    // SIDE-EFFECTING host construction (it allocates a fresh opaque
    // handle host-side). Like the scheduling registration calls above,
    // its `const` binding must be a REAL local — the fold-alias tunnel
    // would re-emit `event_target_new()` at every read site, minting a
    // DISTINCT handle each time (a different listener registry) rather
    // than sharing the one target. Promotion here; the handle store +
    // provenance recording is in the emitter's declarator branch.
    let is_event_target_construction = declarator_init_is_event_target_new(nodes, init);
    // Stage D event lane: `const ok = t.dispatchEvent(...)` is a
    // SIDE-EFFECTING host dispatch (it synchronously re-invokes every
    // registered listener). Its `const` binding must be a REAL local —
    // the fold-alias tunnel would re-emit the dispatch at each read of
    // `ok`, re-running all listeners again (a duplicate-dispatch
    // miscompile).
    let is_event_dispatch_result = declarator_init_is_event_dispatch(nodes, init);
    // Stage P2 Lane 2b: `const cloned = structuredClone(src)` is a FRESH
    // deep allocation whose result MUST be held in a stable local. Without
    // promotion the `const` fold-alias tunnel (`FunctionEmitter::bindings`)
    // re-emits the clone CALL at every read of `cloned` — a NEW allocation
    // each time (and the declaration-site result is dropped, so the first
    // read re-runs it) — a distinct-instances miscompile, the exact twin of
    // `is_materialized_factory_return` for a callee (`structuredClone`) that
    // is a compiler builtin with no `return_repr` entry. Gated on the
    // binding's own repr being `Object` (only the in-envelope object lane
    // reaches here; the placeholder/fail-closed lanes never bind an Object).
    let is_structured_clone_result = declarator_node.text.as_deref().is_some_and(|name| {
        matches!(
            repr_table.scalar(function_name, name),
            kali_common::Repr::Object(_)
        ) && declarator_init_call_callee_name(nodes, init) == Some("structuredClone")
    });
    // Stage P3 abort lane: a binding inference proved `AbortHandle`
    // (`const c = new AbortController()` or the `const s = c.signal`
    // alias) holds an i64 pointer to the shared global abort cell that
    // MUST live in a stable local slot. Without promotion the emitter's
    // abort declarator/alias arms fall to the drop branch (no
    // `self.locals` entry), silently discarding the handle — every
    // `.abort()`/`.aborted` then reads a zero handle and aliases address
    // 0, so DISTINCT controllers share one cell (a latent hole exposed
    // once `.aborted` can read the cell back). Repr-keyed so it covers
    // both admitted seeding shapes uniformly.
    let is_abort_handle_binding = declarator_node.text.as_deref().is_some_and(|name| {
        matches!(
            repr_table.scalar(function_name, name),
            kali_common::Repr::AbortHandle
        )
    });
    // The shapes above force promotion because each needs a stable
    // RUNTIME handle (a fresh allocation, a host registration, a
    // materialized object) — properties an allowlist over the init's
    // syntax cannot see. They are kept as an explicit force-promote
    // union rather than folded into `const_init_is_stable`, which
    // decides the general case: promote UNLESS re-emitting the
    // initializer is provably observationally identical. Default-deny,
    // so an initializer shape nobody enumerated is bound eagerly
    // (correct) instead of textually substituted (a silent wrong
    // value).
    let force_promote = declarator_init_is_array_alloc(nodes, init)
        || declarator_init_is_array_fill(nodes, init)
        || declarator_init_is_array_read(nodes, init, array_names)
        || is_materialized_object
        || is_materialized_object_array
        || is_materialized_factory_return
        || is_growable_array
        || reads_growable_array
        || declarator_init_is_performance_now(nodes, init)
        || declarator_init_is_crypto_call(nodes, init)
        || declarator_init_contains_mutation(nodes, init)
        || is_scheduling_registration_call
        || is_event_target_construction
        || is_event_dispatch_result
        || is_structured_clone_result
        || is_abort_handle_binding;
    if force_promote {
        return Some((name, ConstPromotion::Handle));
    }
    if const_init_is_stable(nodes, init, reassigned, 0) {
        return Some((name, ConstPromotion::Fold));
    }
    Some((name, ConstPromotion::Binding))
}

/// Every `const` in this function body promoted by the STABILITY allowlist
/// (`ConstPromotion::Binding`) — the names that need a compile-time denotation
/// entry alongside their local slot. Walks the same body the local collector
/// does and defers to the same `const_declarator_promotion`.
pub(crate) fn allowlist_promoted_const_names(
    nodes: &[LirNode],
    body_id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
) -> HashSet<String> {
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
    let reassigned = program_reassigned_names(nodes);

    let mut out = HashSet::new();
    let mut seen = HashSet::new();
    allowlist_promoted_const_names_walk(
        nodes,
        body_id,
        &array_names,
        &reassigned,
        repr_table,
        function_name,
        &mut seen,
        &mut out,
    );
    out
}

#[allow(clippy::too_many_arguments)]
fn allowlist_promoted_const_names_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    array_names: &HashSet<String>,
    reassigned: &HashSet<String>,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    seen: &mut HashSet<LirNodeId>,
    out: &mut HashSet<String>,
) {
    if !seen.insert(id) {
        return;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some("const") {
        for declarator in &node.children {
            if let Some((name, ConstPromotion::Binding)) = const_declarator_promotion(
                nodes,
                *declarator,
                array_names,
                reassigned,
                repr_table,
                function_name,
            ) {
                out.insert(name);
            }
        }
    }
    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        allowlist_promoted_const_names_walk(
            nodes,
            *child,
            array_names,
            reassigned,
            repr_table,
            function_name,
            seen,
            out,
        );
    }
}

// One recursive walk threading two precomputed name sets (`array_names`,
// `reassigned`) plus the repr context; bundling them into a struct would add a
// type for a single call site's benefit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_function_locals_from_node(
    nodes: &[LirNode],
    id: LirNodeId,
    array_names: &HashSet<String>,
    reassigned: &HashSet<String>,
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
            let Some((name, promotion)) = const_declarator_promotion(
                nodes,
                *declarator,
                array_names,
                reassigned,
                repr_table,
                function_name,
            ) else {
                continue;
            };
            if promotion == ConstPromotion::Fold {
                continue;
            }
            if !locals.contains(&name) {
                locals.push(name);
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
            reassigned,
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
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
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
/// `new Array(n)` / `Array(n)` allocation call (callee identifier `Array`, 0 or 1
/// arg) OR a `Uint8Array` typed-array constructor (bare `new Uint8Array(n)` or
/// `globalThis["Uint8Array"]` form) — the raw-node mirror of
/// `FunctionEmitter::is_array_like_constructor` (throw-fallout Stage 3 bucket #6).
/// This MUST stay in lockstep with that emit recognizer: it is what grants the
/// declarator its stable array-handle local slot, so a `Uint8Array` binding whose
/// alloc emit fires but whose local is not collected would read/write through an
/// undefined identifier.
pub(crate) fn declarator_init_is_array_alloc(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
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
        return match callee_node.text.as_deref() {
            Some("Array") => callee_node.children.is_empty(),
            Some("Uint8Array") => {
                callee_node.children.is_empty()
                    || callee_node.children.first().is_some_and(|obj| {
                        nodes.get(obj.0 as usize).and_then(|n| n.text.as_deref())
                            == Some("globalThis")
                    })
            }
            _ => false,
        };
    }
}

/// True iff the operator text mutates a binding when evaluated: the four
/// update forms (`++x` / `x++` / `--x` / `x--`) and every assignment operator.
/// `==`/`===` are comparisons, NOT mutations, and must stay out of this list.
pub(crate) fn is_mutating_operator_text(text: &str) -> bool {
    matches!(
        text,
        "prefix++"
            | "postfix++"
            | "prefix--"
            | "postfix--"
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

/// Returns true if the init expression subtree contains a mutating operator
/// (update expression or assignment). A `const` bound to such an init MUST be
/// promoted to an eager local: the default `const` fold lane re-emits the
/// bound init node at every read site, which re-applies the side effect —
/// `let b = 5; const x = ++b;` used to increment `b` once at the declaration
/// (value dropped) and once more per read of `x` (observed `x == 7, b == 7`;
/// node says `6, 6`). Same class as the factory-return promotion above.
/// Calls are deliberately NOT promoted here: structural recognizer lanes
/// (e.g. `resolve_literal_aggregate`) resolve const-bound call nodes by
/// shape, and impure user calls are a separate, wider gap tracked outside
/// this predicate.
///
/// The walk must NOT descend into function-like subtrees (mirroring every
/// sibling recursion in this file): a mutation inside a const-bound arrow's
/// BODY runs at call time in its own scope — it is not an init-time side
/// effect, so it creates no double-eval hazard. Descending would promote
/// `const mk = (n) => { let r = n; r = r + 1; return r; };` to an eager
/// local, but a function-like init produces no value: a zero placeholder
/// got stored and calls resolved through the phantom local (`mk(5)` printed
/// `0`; node says `6`) — a silent miscompile.
pub(crate) fn declarator_init_contains_mutation(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    fn walk(nodes: &[LirNode], id: LirNodeId, seen: &mut HashSet<LirNodeId>) -> bool {
        if !seen.insert(id) {
            return false;
        }
        if is_function_like(nodes, id) {
            return false;
        }
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.text.as_deref().is_some_and(is_mutating_operator_text) {
            return true;
        }
        node.children.iter().any(|&child| walk(nodes, child, seen))
    }
    walk(nodes, init_id, &mut HashSet::new())
}

/// Global namespaces whose members are compile-time-constant intrinsics. A
/// member read off one of these is a snapshot of nothing mutable, which is what
/// makes it safe to leave on the `const` fold lane (see
/// `const_init_is_stable`'s member-read arm).
///
/// This is an ALLOWLIST of RECEIVERS on purpose. The property name is not a
/// sound discriminator: host state reached through a member can be mutated by a
/// method call (`c.abort()` mutates `s.aborted`) with no assignment to that
/// property anywhere in the program. Only restricting the receiver excludes
/// that class by construction.
///
/// Add a namespace here only if reading any of its members can never observe
/// state that a method call elsewhere in the program can change.
pub(crate) const INTRINSIC_NAMESPACES: &[&str] = &[
    "globalThis",
    "Object",
    "Math",
    "Number",
    "String",
    "Array",
    "Boolean",
    "BigInt",
    "JSON",
    "Reflect",
    "Symbol",
    "Date",
];

/// True when `id` is an intrinsic namespace identifier, or a member chain that
/// bottoms out at one (`globalThis.Math`, `globalThis["Math"]`). The root must
/// also be absent from `reassigned`, so a program that shadows or reassigns
/// `Math` does not get its reads folded against the intrinsic.
fn member_chain_roots_at_intrinsic_namespace(
    nodes: &[LirNode],
    id: LirNodeId,
    reassigned: &HashSet<String>,
    depth: usize,
) -> bool {
    if depth > 32 {
        return false;
    }
    let id = unwrap_transparent_value(nodes, id);
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    let Some(text) = node.text.as_deref().filter(|text| !text.is_empty()) else {
        return false;
    };
    if node.children.is_empty() {
        return INTRINSIC_NAMESPACES.contains(&text) && !reassigned.contains(text);
    }
    // An inner member read (`globalThis.Math` inside `globalThis.Math.round`):
    // recurse on ITS base. The property name is checked by the caller's arm.
    if node.children.len() == 1 {
        return member_chain_roots_at_intrinsic_namespace(
            nodes,
            node.children[0],
            reassigned,
            depth + 1,
        );
    }
    false
}

/// Sentinel recorded in the reassigned-name set when a COMPUTED member target
/// (`a[i] = …`) is assigned: it names no single property, so every member read
/// in a `const` initializer must be denied the fold lane. Not a legal
/// identifier, so it can never collide with a source name.
pub(crate) const COMPUTED_MEMBER_ASSIGN_SENTINEL: &str = "\0computed-member-assign";

/// Every name REBOUND anywhere in the PROGRAM: the target of a
/// mutating operator (`=`, `+=`, `++`, `--`, …) or a `for-of`/`for-await-of`/
/// `for-in` loop variable. A read of such a name is NOT stable — re-emitting
/// it at a later program point can observe a different value than the one in
/// scope where the read was written.
///
/// Deliberately descends INTO function-like children, unlike most walks in
/// this module: a nested closure that assigns a captured name makes reads of
/// that name unstable in the enclosing function too. Over-collecting here only
/// costs an extra promoted local slot; under-collecting is a silent wrong
/// value, so the walk errs toward collecting.
pub(crate) fn program_reassigned_names(nodes: &[LirNode]) -> HashSet<String> {
    let mut out = HashSet::new();
    for node in nodes {
        record_reassigned_from_node(nodes, node, &mut out);
    }
    out
}

fn record_reassigned_from_node(nodes: &[LirNode], node: &LirNode, out: &mut HashSet<String>) {
    if node.text.as_deref().is_some_and(is_mutating_operator_text) {
        // The assignment/update TARGET is the first child.
        if let Some(&target) = node.children.first() {
            if let Some(name) = bare_identifier_name_of(nodes, target) {
                out.insert(name);
            } else {
                // A MEMBER target (`o.x = …`, `a[i] = …`). Record the property
                // name in the same set, so a `const` initializer that reads
                // `<anything>.x` is denied the fold lane. Sharing one namespace
                // with variable names only over-denies (a variable `x` assigned
                // somewhere also blocks reads of `.x`), which costs a local
                // slot rather than correctness.
                let target = unwrap_transparent_value(nodes, target);
                match nodes
                    .get(target.0 as usize)
                    .and_then(|n| n.text.as_deref())
                    .filter(|text| !text.is_empty())
                {
                    Some(property) => {
                        out.insert(property.to_string());
                    }
                    // A COMPUTED member target whose property is an expression
                    // (`a[i] = …`) names no single property. Deny every member
                    // read via a sentinel that cannot collide with a source
                    // identifier.
                    None => {
                        out.insert(COMPUTED_MEMBER_ASSIGN_SENTINEL.to_string());
                    }
                }
            }
        }
    }
    if node.kind == LirNodeKind::Branch {
        match node.text.as_deref() {
            Some("for-of" | "for-await-of") => {
                if let Some(name) = node
                    .children
                    .first()
                    .and_then(|&left| for_of_loop_var_name_of(nodes, left))
                {
                    out.insert(name);
                }
            }
            Some("for-in") => {
                if let Some(name) = node
                    .children
                    .first()
                    .and_then(|&left| for_in_loop_key_name(nodes, left))
                {
                    out.insert(name);
                }
            }
            _ => {}
        }
    }
}

/// True when re-emitting `id` at an arbitrary later program point is
/// OBSERVATIONALLY IDENTICAL to evaluating it at the declaration site — i.e.
/// the initializer of a `const` may safely stay on the compile-time fold lane
/// (`FunctionEmitter::bindings`, which re-emits the init AST node at every
/// read of the bound name) instead of being promoted to an eager local slot.
///
/// This is the ALLOWLIST half of the `const`-binding choke point. Its
/// predecessor was a denylist of ~15 initializer shapes known to break under
/// re-emission (allocations, host calls, factory returns, mutating inits, …),
/// each added after a miscompile was found in the field. A denylist cannot
/// close this class: every initializer shape NOT enumerated silently kept the
/// fold lane, so `const` was a textual substitution rather than a binding —
/// `const tmp = a; a = b; b = tmp;` lost a value with exit 0 and no
/// diagnostic, and `const c = f()` ran `f` once per read. Only an allowlist of
/// provably stable shapes closes it by construction: anything unrecognized is
/// promoted, which is always semantically safe (eager evaluation at the
/// declaration is exactly what the language specifies).
///
/// Stable shapes:
/// - a literal, or a bare token that is not a reassigned name;
/// - operators over stable operands (assignment/update operators excluded:
///   `is_binary_operator_text` includes them, and they are side effects);
/// - object/array-literal aggregates whose element expressions are stable
///   (these carry no operator text) and their property nodes;
/// - a function-like init, which stays on the fold lane BY DESIGN — promoting
///   it produces no value and calls then resolve through a phantom zero local
///   (pinned by `soundness_const_fold_side_effects::
///   const_bound_arrow_with_mutating_body_is_not_promoted`).
///
/// Everything else — calls, `new`, member reads, template literals, `await`ed
/// values, ternaries — is unstable and gets a local slot.
pub(crate) fn const_init_is_stable(
    nodes: &[LirNode],
    id: LirNodeId,
    reassigned: &HashSet<String>,
    depth: usize,
) -> bool {
    if depth > 32 {
        return false;
    }
    if is_function_like(nodes, id) {
        return true;
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    let all_children_stable = |nodes: &[LirNode]| {
        node.children
            .iter()
            .all(|&child| const_init_is_stable(nodes, child, reassigned, depth + 1))
    };
    match node.kind {
        LirNodeKind::Literal => true,
        LirNodeKind::Value => {
            let text = node.text.as_deref();
            if node.children.is_empty() {
                // A bare token: a literal spelling (`42`, `true`, `"hi"`) is
                // stable; an identifier is stable only when never reassigned.
                return text.is_some_and(|name| !reassigned.contains(name));
            }
            match text {
                // Sequence/paren/`await` wrappers and object/array-literal
                // aggregates all carry no operator text.
                None | Some("") => all_children_stable(nodes),
                // Object-literal property node: the key is a literal, the
                // value is the second child.
                Some("init" | "get" | "set") if node.children.len() == 2 => {
                    const_init_is_stable(nodes, node.children[1], reassigned, depth + 1)
                }
                // `is_binary_operator_text` also matches `=`/`+=`/… — those
                // are side effects, never stable.
                Some(operator) if is_mutating_operator_text(operator) => false,
                Some(operator) if node.children.len() == 2 && is_binary_operator_text(operator) => {
                    all_children_stable(nodes)
                }
                Some("-" | "+" | "!" | "~" | "void" | "typeof") if node.children.len() == 1 => {
                    all_children_stable(nodes)
                }
                // A MEMBER READ off an INTRINSIC NAMESPACE (`Object.is`,
                // `globalThis.Math.round`). This arm exists for exactly one
                // reason — keeping intrinsic ALIASES on the fold lane, where
                // the analyses that read `FunctionEmitter::bindings` can still
                // see what the name denotes (promoting one binds the runtime
                // value correctly but un-resolves the alias, so the intrinsic
                // is never recognized) — and it is scoped to precisely that.
                //
                // It is deliberately NOT a test on the property name. An
                // earlier form admitted any member read whose property was
                // never an ASSIGNMENT target, which is unsound for host state
                // mutated by a METHOD CALL: `c.abort()` mutates `s.aborted`
                // with no assignment to `aborted` anywhere in the program, so
                // `const before = s.aborted` folded and re-read the mutated
                // cell — one `const`, two values. Only a receiver allowlist
                // closes that: an intrinsic namespace is a compile-time
                // constant whose members are functions, so a read off one is a
                // snapshot of nothing mutable. A read off a program-bound
                // receiver may snapshot mutable state and is denied, which
                // promotes it to a slot and binds it once.
                //
                // The property-name and computed-assign checks are kept as
                // defense in depth (`Math.round = f` records `round`), but the
                // receiver allowlist is what makes the arm sound.
                Some(property)
                    if node.children.len() == 1
                        && !reassigned.contains(property)
                        && !reassigned.contains(COMPUTED_MEMBER_ASSIGN_SENTINEL)
                        && member_chain_roots_at_intrinsic_namespace(
                            nodes,
                            node.children[0],
                            reassigned,
                            0,
                        ) =>
                {
                    all_children_stable(nodes)
                }
                _ => false,
            }
        }
        // `Object.freeze(x)` is an IDENTITY intrinsic: it returns its argument
        // (freezing an object in place is idempotent), so re-emitting it is
        // observationally identical exactly when re-emitting the argument is.
        // Recognized here rather than left to the generic call deny because the
        // freeze wrapper is the idiomatic spelling of an intrinsic alias
        // throughout the browser corpus (`Object.freeze(Math.round)(v)`), and
        // promoting it both un-resolves the intrinsic and — when the argument
        // is a float the repr table did not infer as `F64` — allocates an i64
        // slot for an f64 value, emitting invalid wasm.
        LirNodeKind::Call
            if node.children.len() == 2
                && nodes
                    .get(node.children[0].0 as usize)
                    .is_some_and(|callee| {
                        callee.text.as_deref() == Some("freeze")
                            && callee
                                .children
                                .first()
                                .and_then(|&o| nodes.get(o.0 as usize))
                                .and_then(|n| n.text.as_deref())
                                == Some("Object")
                    }) =>
        {
            const_init_is_stable(nodes, node.children[1], reassigned, depth + 1)
        }
        _ => false,
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
/// True iff `init_id` (after unwrapping sequence wrappers) is a
/// `performance.now()` call (callee text `"now"`, object text `"performance"`).
/// Such a const initializer is IMPURE (each call returns a different monotonic
/// timestamp), so — like `is_materialized_factory_return` — it must be promoted
/// to a local slot and evaluated ONCE at its declaration, never fold-inlined and
/// re-called at each use site (which would call `performance.now()` again and
/// silently reorder `a`/`b` in a `b < a` comparison — a nondeterminism
/// miscompile, not just a missed optimization). Mirrors
/// `program_uses_performance_now` / the codegen recognizer's shape.
pub(crate) fn declarator_init_is_performance_now(nodes: &[LirNode], init_id: LirNodeId) -> bool {
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
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("now") {
            return false;
        }
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object_node) = nodes.get(object.0 as usize) else {
            return false;
        };
        return object_node.text.as_deref() == Some("performance");
    }
}

/// True iff `init_id` (after unwrapping transparent value / `await` wrappers) is
/// a `crypto.getRandomValues(...)`, `crypto.randomUUID()`,
/// `crypto.subtle.digest(...)`, or `new TextEncoder().encode(...)` call
/// (throw-fallout Stage 3 bucket #6). The two random calls are IMPURE (each
/// `Call` yields fresh bytes / a fresh UUID); `digest`/`encode` are deterministic
/// but produce a fresh RUNTIME STRING handle whose `String` repr the binding must
/// record. In every case — exactly like `declarator_init_is_performance_now` — a
/// `const` initializer of this shape must be PROMOTED to a local slot and
/// evaluated ONCE at its declaration, never fold-inlined and re-emitted at each
/// use site (for the random calls: a distinct-value nondeterminism miscompile
/// AND host re-call per use; for `digest`: a redundant host call + a fresh
/// `__alloc_global` buffer per use; for all of them a use-site bare-call result
/// whose String repr the binding never records — so `.byteLength` / `.length` /
/// `typeof` misresolve). `digest` arrives `await`-wrapped
/// (`const d = await crypto.subtle.digest(...)`), so the unwrap loop also tunnels
/// the `"await"` marker (Stage 3 Task 4). Mirrors the codegen recognizers'
/// `getRandomValues`/`randomUUID`/`subtle.digest`/`TextEncoder().encode` shapes.
pub(crate) fn declarator_init_is_crypto_call(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    let mut id = init_id;
    let mut guard = 0;
    loop {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        // Tunnel transparent value/sequence wrappers (text None or empty) and the
        // synchronously-settled `await` marker (text "await", one child).
        if node.kind == LirNodeKind::Value
            && !node.children.is_empty()
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            id = *node.children.last().expect("wrapper has a child");
            guard += 1;
            if guard > 64 {
                return false;
            }
            continue;
        }
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = nodes.get(callee.0 as usize) else {
            return false;
        };
        // `crypto.getRandomValues`/`crypto.randomUUID` (object `crypto`);
        // `crypto.subtle.digest` (object `subtle` -> grand-object `crypto`);
        // `new TextEncoder().encode` (object is a `new TextEncoder()` Call).
        return match callee_node.text.as_deref() {
            Some("getRandomValues") | Some("randomUUID") => {
                callee_node
                    .children
                    .first()
                    .and_then(|&o| nodes.get(o.0 as usize))
                    .and_then(|n| n.text.as_deref())
                    == Some("crypto")
            }
            Some("digest") => {
                let Some(subtle_node) = callee_node
                    .children
                    .first()
                    .and_then(|&o| nodes.get(o.0 as usize))
                else {
                    return false;
                };
                subtle_node.text.as_deref() == Some("subtle")
                    && subtle_node
                        .children
                        .first()
                        .and_then(|&o| nodes.get(o.0 as usize))
                        .and_then(|n| n.text.as_deref())
                        == Some("crypto")
            }
            Some("encode") => {
                let Some(ctor_call) = callee_node
                    .children
                    .first()
                    .and_then(|&o| nodes.get(o.0 as usize))
                else {
                    return false;
                };
                ctor_call.kind == LirNodeKind::Call
                    && ctor_call
                        .children
                        .first()
                        .and_then(|&c| nodes.get(c.0 as usize))
                        .and_then(|n| n.text.as_deref())
                        == Some("TextEncoder")
            }
            _ => false,
        };
    }
}

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
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
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

/// `__streq(a, b) -> i64`: content equality of two tagged string handles —
/// 1 when equal, 0 when not (throw-fallout Stage 1). Locals: 0 = a, 1 = b
/// (params), 2 = len, 3 = i, 4 = pa, 5 = pb.
///
/// Order of checks:
///   1. identical handles → 1 (interned-vs-interned and aliased handles);
///   2. string-tag guard: unless BOTH operands carry `STRING_HANDLE_TAG`,
///      they are not two live strings (e.g. a missing `Deno.env.get` is 0)
///      → 0 (the identical case already returned);
///   3. length mismatch (low 32 bits) → 0;
///   4. len == 0 → 1 (two empty strings are equal at ANY offsets);
///   5. byte loop over the two decoded offsets — first mismatch → 0, loop
///      completion → 1.
///
/// Offsets are decoded exactly as the runtime does (`(h >> 32) & 0x7FFF_FFFF`
/// — masked, mirroring `read_guest_string_handle` in
/// kali_runtime/src/host/memory.rs), matching `emit_substring_body`.
///
/// NO `i64.eqz` anywhere in this body: like `__join` (see the comment in
/// `emit_join_body`), `__streq` is present in every module and
/// `boolean_branches_use_the_layout_fast_path` asserts module-wide printed
/// text contains no `i64.eqz`. Zero-tests use `i64.const 0` + `i64.eq`.
fn emit_streq_body(func: &mut Function) {
    // 1. if a == b return 1
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 2. if (a & b & TAG) == 0 return 0  — not two tagged strings
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 3. len = a & 0xFFFF_FFFF; if len != (b & 0xFFFF_FFFF) return 0
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Ne);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 4. if len == 0 return 1
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // pa = (a >> 32) & 0x7FFF_FFFF; pb = (b >> 32) & 0x7FFF_FFFF
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(5));
    // i = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // 5. loop: if *(pa+i) != *(pb+i) return 0; i += 1; continue while i < len
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::I64Ne);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);
    // all len bytes equal
    func.instruction(&Instruction::I64Const(1));
    // NO trailing End — the dispatch loop appends it (same as every synthetic).
}

// === URLSearchParams scan/mutation synthetic bodies (Stage P4 Task 4) ========
//
// Shared store layout: `store` is the tagged growable handle
// (`hdr | ARRAY_HANDLE_TAG`); `hdr_ptr = store & GROWABLE_HANDLE_MASK`;
// `len = hdr[+0]`, `cap = hdr[+8]`, `data = hdr[+16]`; the data block is
// `[k0,v0,k1,v1,…]` of interned i64 string handles. Key comparison is the
// present-in-every-module `__streq` synthetic (index threaded in). No
// `i64.eqz` anywhere (module-wide boolean-fast-path assertion) — zero tests use
// `i64.const 0; i64.eq`.

/// Push `hdr_ptr` (i32) of the tagged growable store handle held in `store_local`.
fn usp_emit_hdr(func: &mut Function, store_local: u32) {
    func.instruction(&Instruction::LocalGet(store_local));
    func.instruction(&Instruction::I64Const(
        crate::emit::growable::GROWABLE_HANDLE_MASK,
    ));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
}

/// Push element base address `data + index*8` (i32); the caller's i64 load/store
/// uses `offset` 0 for the key/first slot or 8 for the value/second slot.
/// `data_local` holds the zero-extended `data_ptr` (i64); `index_local` the i64
/// pair index.
fn usp_emit_elem_addr(func: &mut Function, data_local: u32, index_local: u32) {
    func.instruction(&Instruction::LocalGet(data_local));
    func.instruction(&Instruction::LocalGet(index_local));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
}

/// Push element value `data[index]` (`off` 0) or `data[index+1]` (`off` 8) as i64.
fn usp_emit_load_elem(func: &mut Function, data_local: u32, index_local: u32, off: u64) {
    usp_emit_elem_addr(func, data_local, index_local);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: off,
        align: 3,
        memory_index: 0,
    }));
}

/// `__usp_get(store, key) -> i64`: `i=0; while i<len { if __streq(data[i],key)
/// return data[i+1]; i+=2 } return 0`. Locals 0-1 = `store`/`key`; 2 = `len`,
/// 3 = `i`, 4 = `data`.
fn emit_usp_get_body(func: &mut Function, streq_index: u32) {
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(2)); // len
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(4)); // data
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3)); // i = 0
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    // if __streq(data[i], key) return data[i+1]
    usp_emit_load_elem(func, 4, 3, 0);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(streq_index));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    usp_emit_load_elem(func, 4, 3, 8);
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // i += 2; continue
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
    func.instruction(&Instruction::I64Const(0)); // not found → null sentinel
                                                 // NO trailing End.
}

/// `__usp_has(store, key) -> i64`: the `__usp_get` scan returning `1`/`0`.
/// Locals identical to `__usp_get`.
fn emit_usp_has_body(func: &mut Function, streq_index: u32) {
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(2)); // len
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(4)); // data
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3)); // i = 0
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    usp_emit_load_elem(func, 4, 3, 0);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(streq_index));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
    func.instruction(&Instruction::I64Const(0)); // no match
                                                 // NO trailing End.
}

/// `__usp_getall(store, key) -> i64`: two-pass — count matches, allocate a fresh
/// growable of exactly that length via `__alloc_global`, then fill it with each
/// matching value. Returns the fresh tagged growable handle (its `.length`
/// reads its own header). Two-pass avoids any grow-if-full inside the scan.
/// Locals 0-1 = `store`/`key`; 2 = `len`, 3 = `i`, 4 = `data` (source), 5 =
/// `count`/write-cursor, 6 = `newhdr`, 7 = `newdata`.
fn emit_usp_getall_body(func: &mut Function, streq_index: u32, alloc_global_index: u32) {
    // len / data (source)
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(2));
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(4));
    // Pass 1: count = number of key matches.
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(5)); // count = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3)); // i = 0
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    usp_emit_load_elem(func, 4, 3, 0);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(streq_index));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(5));
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
                                         // Allocate the result growable: header(24) + data(count*8).
    func.instruction(&Instruction::I32Const(24));
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(6)); // newhdr (zero-extended ptr)
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(7)); // newdata
                                                 // newhdr.len = count
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    // newhdr.cap = count
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    // newhdr.data_ptr = newdata
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    // Pass 2: fill newdata[count++] = data[i+1] for each match.
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(5)); // reuse as write cursor
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3)); // i = 0
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    usp_emit_load_elem(func, 4, 3, 0);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(streq_index));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    // newdata[count] = data[i+1]
    usp_emit_elem_addr(func, 7, 5);
    usp_emit_load_elem(func, 4, 3, 8);
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(5));
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
                                         // return newhdr | ARRAY_HANDLE_TAG
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(crate::ARRAY_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    // NO trailing End.
}

/// `__usp_set(store, key, val) -> i64` (WHATWG): overwrite the FIRST matching
/// key's value, remove subsequent matches (in-place compaction with a write
/// cursor), else append `[key,val]` (grow-if-full via `__alloc_global`). Returns
/// the store handle. Locals 0-2 = `store`/`key`/`val`; 3 = `write`, 4 = `found`,
/// 5 = `i`, 6 = `data`, 7 = `len`, 8 = `cap`, 9 = `newdata`.
fn emit_usp_set_body(func: &mut Function, streq_index: u32, alloc_global_index: u32) {
    // data / len / cap
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(6)); // data
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(7)); // len
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(8)); // cap
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3)); // write = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(4)); // found = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(5)); // i = 0
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    // matched = __streq(data[i], key)
    usp_emit_load_elem(func, 6, 5, 0);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(streq_index));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    // matched: keep first (overwrite value), drop the rest.
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    // data[write] = key
    usp_emit_elem_addr(func, 6, 3);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    // data[write+1] = val
    usp_emit_elem_addr(func, 6, 3);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    // write += 2; found = 1
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::End); // found==0 if (else: drop)
    func.instruction(&Instruction::Else);
    // not matched: copy [data[i], data[i+1]] down to write.
    usp_emit_elem_addr(func, 6, 3);
    usp_emit_load_elem(func, 6, 5, 0);
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    usp_emit_elem_addr(func, 6, 3);
    usp_emit_load_elem(func, 6, 5, 8);
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::End); // matched if/else
                                         // i += 2; continue
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(5));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
                                         // if found == 0: append [key, val], growing the data block if needed.
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    // if write + 2 > cap: grow (cap*2; write==len here so this is len+2 vs cap).
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    // newdata = __alloc_global(cap * 2 * 8)
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64Const(16));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(9));
    // memory.copy(newdata, data, write*8)
    func.instruction(&Instruction::LocalGet(9));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
    // hdr.data_ptr = newdata
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::LocalGet(9));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    // hdr.cap = cap * 2
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    // data = newdata
    func.instruction(&Instruction::LocalGet(9));
    func.instruction(&Instruction::LocalSet(6));
    func.instruction(&Instruction::End); // grow if
                                         // data[write] = key
    usp_emit_elem_addr(func, 6, 3);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    // data[write+1] = val
    usp_emit_elem_addr(func, 6, 3);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 8,
        align: 3,
        memory_index: 0,
    }));
    // write += 2
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::End); // found==0 append if
                                         // hdr.len = write
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Store(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    // return store
    func.instruction(&Instruction::LocalGet(0));
    // NO trailing End.
}

/// `__percent_encode(str) -> i64` (Stage P4 Task 5): re-encode a tagged string
/// handle's bytes as `application/x-www-form-urlencoded` — unreserved bytes
/// (`A-Z a-z 0-9 * - . _`) verbatim, space (0x20) → `+`, everything else →
/// `%` + two UPPERCASE hex digits — into a fresh buffer allocated through
/// `__alloc_global` (worst case `3*len`; a toString result must outlive any
/// arena reset, exactly as `__join` allocates globally). Returns the packed
/// handle `TAG | out<<32 | written` (`encode_string_handle` layout; offsets
/// decoded as `(h >> 32) & 0x7FFF_FFFF`, len as the low 32 bits — mirroring
/// `emit_streq_body`/`emit_substring_body`). A non-string input (no
/// `STRING_HANDLE_TAG`, e.g. a stray 0 sentinel) and the empty string both
/// return the bare TAG (a zero-length handle is never dereferenced).
/// Locals: 0 = `str` (param), 1 = `len`, 2 = `src`, 3 = `out`, 4 = `w`
/// (write cursor), 5 = `i`, 6 = `b` (current byte), 7 = `n` (nibble temp).
/// No `i64.eqz` anywhere (module-wide boolean-fast-path assertion — see the
/// comment in `emit_join_body`); zero tests use `i64.const 0` + `i64.eq`.
fn emit_percent_encode_body(func: &mut Function, alloc_global_index: u32) {
    // if (str & TAG) == 0 return TAG — not a live string handle.
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // len = str & 0xFFFF_FFFF; if len == 0 return TAG (empty stays empty).
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(1));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // src = (str >> 32) & 0x7FFF_FFFF
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(2));
    // out = zext(__alloc_global(3 * len)) — worst case, every byte → %XX.
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(3));
    // w = out; i = 0
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(5));
    // loop over input bytes
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    // b = *(src + i)
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(6));
    // unreserved: A-Z | a-z | 0-9 | '*' 42 | '-' 45 | '.' 46 | '_' 95
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(65));
    func.instruction(&Instruction::I64GeS);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(90));
    func.instruction(&Instruction::I64LeS);
    func.instruction(&Instruction::I32And);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(97));
    func.instruction(&Instruction::I64GeS);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(122));
    func.instruction(&Instruction::I64LeS);
    func.instruction(&Instruction::I32And);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(48));
    func.instruction(&Instruction::I64GeS);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(57));
    func.instruction(&Instruction::I64LeS);
    func.instruction(&Instruction::I32And);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(42));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(45));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(46));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(95));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::I32Or);
    func.instruction(&Instruction::If(BlockType::Empty));
    // unreserved: *w = b; w += 1
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::Else);
    // space → '+'
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Const(43));
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::Else);
    // else: '%' + two UPPERCASE hex digits (digit = n + (n>9 ? 55 : 48)).
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Const(37));
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    // n = b >> 4; *(w+1) = hexdigit(n)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(4));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::LocalSet(7));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(55));
    func.instruction(&Instruction::I64Const(48));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(9));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 1,
        align: 0,
        memory_index: 0,
    }));
    // n = b & 15; *(w+2) = hexdigit(n)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(15));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(7));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(55));
    func.instruction(&Instruction::I64Const(48));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(9));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 2,
        align: 0,
        memory_index: 0,
    }));
    // w += 3
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::End); // space/else if
    func.instruction(&Instruction::End); // unreserved if
                                         // i += 1; continue
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(5));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // loop
                                         // TAG | out<<32 | (w - out)
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Or);
    // NO trailing End.
}

/// Emit a separator byte: `*w = byte; w += 1` (`w_local` holds the i64 write
/// cursor). Part of `emit_usp_tostring_body`'s single-buffer build.
fn usp_emit_sep_byte(func: &mut Function, w_local: u32, byte: i64) {
    func.instruction(&Instruction::LocalGet(w_local));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Const(byte));
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(w_local));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(w_local));
}

/// Emit `enc = __percent_encode(data[i + slot]); copy enc's bytes to w` for
/// `emit_usp_tostring_body`: calls the encoder on the element (`off` 0 = key,
/// 8 = value), decodes the returned handle into `p` (source cursor) / `n`
/// (remaining bytes), then byte-copies `n` bytes to the `w` cursor. Locals are
/// `emit_usp_tostring_body`'s: 2 = `i`, 3 = `data`, 4 = `w`, 6 = `h`, 7 = `p`,
/// 8 = `n`.
fn usp_emit_encoded_component_copy(func: &mut Function, percent_encode_index: u32, off: u64) {
    // h = __percent_encode(data[i]/data[i+1])
    usp_emit_load_elem(func, 3, 2, off);
    func.instruction(&Instruction::Call(percent_encode_index));
    func.instruction(&Instruction::LocalSet(6));
    // p = (h >> 32) & 0x7FFF_FFFF ; n = h & 0xFFFF_FFFF
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(7));
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(8));
    // while n > 0 { *w = *p; w += 1; p += 1; n -= 1 }
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::I64Store8(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(7));
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::LocalSet(8));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // n>0 if
    func.instruction(&Instruction::End); // copy loop
}

/// `__usp_tostring(store) -> i64` (Stage P4 Task 5): serialize the pair store
/// as `enc(k0)=enc(v0)&enc(k1)=enc(v1)&…` — each component percent-encoded by
/// `__percent_encode` (index threaded in) — into ONE `__alloc_global` buffer
/// (like `__join`; the result must outlive any arena reset). Pass 1 sums a
/// worst-case size (`3 * Σ raw component len + len` — `len` slots is one more
/// than the `len-1` separator bytes `n×'=' + (n-1)×'&'`); pass 2 writes
/// separator bytes directly and byte-copies each encoded component from its
/// own `__percent_encode` buffer. Deliberately NO `string_concat` import call
/// (a fully-granted module asserts the global concat import is never called —
/// see `granted_concat_in_loop_routes_to_arena_import`) and NO interned
/// literals (key-table pins fix data-segment offsets). Empty store → bare TAG
/// (empty string handle, never dereferenced). Returns `TAG | out<<32 |
/// (w-out)`. Locals: 0 = `store` (param), 1 = `len`, 2 = `i`, 3 = `data`,
/// 4 = `w` (size accumulator, then write cursor), 5 = `out`, 6 = `h`, 7 = `p`,
/// 8 = `n`. No `i64.eqz` (see `emit_join_body`).
fn emit_usp_tostring_body(func: &mut Function, percent_encode_index: u32, alloc_global_index: u32) {
    // len = hdr[+0]; data = hdr[+16]
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(1));
    usp_emit_hdr(func, 0);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(3));
    // if len == 0 return TAG (empty string handle)
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // Pass 1: w = len + Σ 3*(data[i] & 0xFFFF_FFFF) — worst-case output size.
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    usp_emit_load_elem(func, 3, 2, 0);
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // pass-1 loop
                                         // out = zext(__alloc_global(w)); w = out
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(5));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalSet(4));
    // Pass 2: for i in (0..len).step_by(2): [&] enc(key) = enc(val)
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    // if i > 0: *w = '&' (38); w += 1
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    usp_emit_sep_byte(func, 4, 38);
    func.instruction(&Instruction::End);
    // encoded key bytes
    usp_emit_encoded_component_copy(func, percent_encode_index, 0);
    // *w = '=' (61); w += 1
    usp_emit_sep_byte(func, 4, 61);
    // encoded value bytes
    usp_emit_encoded_component_copy(func, percent_encode_index, 8);
    // i += 2; continue
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(2));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // i<len if
    func.instruction(&Instruction::End); // pass-2 loop
                                         // TAG | out<<32 | (w - out)
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Or);
    // NO trailing End.
}

/// `__join(arr, sep) -> i64`: copy every element string (i64 handles in the
/// array's slots) plus `sep` between them into ONE fresh buffer allocated via
/// `alloc_index`; return `TAG | out<<32 | total`. Empty array returns bare TAG
/// (offset 0, len 0 — a zero-length handle is never dereferenced).
/// Locals: 0=arr 1=sep (params), 2=n 3=i 4=total 5=out 6=cur 7=h.
///
/// `alloc_index` is `__alloc_global` for the global `__join` and `__alloc`
/// (current arena) for the `__join_arena` twin (fasta Spec 7 Task 4c) — the
/// ONLY difference between the two synthetic bodies.
fn emit_join_body(func: &mut Function, alloc_index: u32) {
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
    func.instruction(&Instruction::Call(alloc_index));
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

/// `__join_growable_i64(arr, sep) -> i64` / `__join_growable_str(arr, sep) ->
/// i64` (throw-fallout Stage 4 Task 5): the growable-array analogue of
/// `emit_join_body`. Same two-pass `memory.copy` join into ONE fresh
/// `__alloc_global` string, but over the tagged-handle GROWABLE layout:
///
///   * the receiver `arr` is a TAGGED handle (`ARRAY_HANDLE_TAG`), so the
///     header pointer is `arr & !TAG` (masked), NOT `arr` directly;
///   * `n = *(hdr+0)`, `data = *(hdr+16)` (header indirection), and each
///     element handle is `*(data + i*8)` (offset 0, NOT the inline `+8`);
///   * when `render_int` is set the raw i64 slot is a NUMBER, not a string
///     handle — it is rendered to a decimal string handle by the runtime
///     `int_to_string` import (fixed index 17, always present) BEFORE its
///     bytes are measured/copied. `int_to_string` renders negatives with a
///     `-` sign, matching node; each call `__alloc`s a fresh guest string, so
///     the pass-1 (length) and pass-2 (copy) calls never alias. (For a large
///     array the double render leaks the pass-1 strings into the global heap —
///     GC-less by design, reclaimed only by arena scope; the target fixtures
///     are small.)
///
/// `render_int == false` is the String-element body (`__join_growable_str`):
/// the slot already IS a string handle, copied verbatim.
///
/// Locals: 0=arr 1=sep (params), 2=n 3=i 4=total 5=out 6=cur 7=h 8=data —
/// one more (`data`) than `emit_join_body` for the cached header→data pointer.
/// `alloc_index` is `__alloc_global` (a join result must not dangle across an
/// arena reset — same rule as the global `__join`).
fn emit_join_growable_body(func: &mut Function, alloc_index: u32, render_int: bool) {
    let mask = !(crate::ARRAY_HANDLE_TAG) as i64;
    // hdr = arr & !TAG (masked header pointer, reused below via I32WrapI64).
    // n = *(hdr + 0)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(mask));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(2));
    // data = *(hdr + 16)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(mask));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 16,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalSet(8));
    // if n == 0 return TAG (empty string). Explicit I64Const(0)+I64Eq (never
    // I64Eqz) — same whole-module `boolean_branches_use_the_layout_fast_path`
    // constraint as `emit_join_body`; these synthetics are present in every
    // module too.
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
    // pass 1: total += len(render(elem_i)) for each i
    func.instruction(&Instruction::Loop(BlockType::Empty));
    emit_growable_join_element_handle(func, render_int);
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
    // out = zext(alloc(wrap((total + 7) & !7)))
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(7));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Const(-8)); // !7
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(5));
    // cur = out; i = 0
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalSet(6));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // pass 2: copy elements, separator between them
    func.instruction(&Instruction::Loop(BlockType::Empty));
    emit_growable_join_element_handle(func, render_int);
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
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(6));
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // If
    func.instruction(&Instruction::End); // Loop
                                         // TAG | out << 32 | total
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Or);
    // NO trailing End — the dispatch loop appends it.
}

/// Push the string handle of growable element `i` onto the stack: load the raw
/// i64 slot `*(data + i*8)` (data = local 8, i = local 3), then — for the i64
/// body — coerce it to a decimal-string handle via `int_to_string`. The String
/// body's slot is already a handle, so it is left as-is.
fn emit_growable_join_element_handle(func: &mut Function, render_int: bool) {
    // raw = *(data + (i << 3))
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    if render_int {
        func.instruction(&Instruction::Call(crate::INT_TO_STRING_IMPORT_INDEX));
    }
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
