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

/// R-11 T4 review round 4: the SINGLE resolution `emit_value`'s bare-
/// identifier arm (`emit/control_flow.rs`'s `resolve_identifier_kind`, which
/// owns this enum's full ordering doc) reaches for a name — the shared
/// classifier both that function's dispatch and the bitwise compound-assign
/// captured-cell shadow guard (`emit/closure_access.rs`) `match`
/// EXHAUSTIVELY, so the two cannot independently drift. See
/// `resolve_identifier_kind`'s doc for the full arm-by-arm mapping and why
/// this replaces three prior rounds of hand-mirrored denylists.
#[derive(Clone, Debug)]
pub(crate) enum IdentifierResolution {
    EventTargetHandle,
    AbortHandleDenied,
    ModuleScopeAbortHandle,
    UrlHandleDenied,
    BytesHandleDenied,
    EventMarker,
    ModuleScopeOrCapturedEventMarker,
    ModuleScopeUrlHandle,
    CapturedUrlHandle,
    /// `(data-segment table base, ordinal local index)`.
    ForInKeyStringHandle(u32, u32),
    /// `(WASM global index, repr)`.
    ModuleGlobal(u32, kali_common::Repr),
    /// WASM local index.
    Local(u32),
    /// Compile-time bound alias node.
    Binding(LirNodeId),
    /// Module `const` compile-time-pure initializer node.
    ModuleConstPureInline(LirNodeId),
    ModuleBindingDenied,
    NumericLiteral(i64),
    KeywordTrue,
    KeywordFalsyBoolean,
    KeywordSetOrMap,
    /// Not resolved by any earlier lane — eligible for the captured-scalar
    /// lane, or the terminal placeholder fallback.
    CapturedCellOrPlaceholder,
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
    /// Stage P3: bindings proven to hold an abort handle (an i64 pointer to the
    /// never-reclaimed global abort cell) in THIS emitter's scope —
    /// `const c = new AbortController()` declarators the intercept fired for.
    /// Mirrors `event_target_locals`. Captured handles are proven separately
    /// via the env plan + owner-keyed repr (see `is_abort_handle`).
    pub(crate) abort_handle_locals: BTreeSet<String>,
    /// Stage P3 position allowlist flag for abort-handle reads (the
    /// `admit_growable_field_read` pattern): a bare read of an abort-handle
    /// binding is E5506 unless an allowlisted consumer set this while emitting
    /// its receiver (`emit_abort_receiver_handle`).
    pub(crate) admit_abort_handle_read: bool,
    /// Stage P4: bindings proven to hold a URL handle (an i64 pointer to the
    /// arena-allocated fixed 6-slot URL struct) in THIS emitter's scope —
    /// `const u = new URL(<string-literal>)` declarators the intercept fired
    /// for. Mirrors `abort_handle_locals`.
    pub(crate) url_locals: std::collections::BTreeSet<String>,
    /// Stage P4: bindings proven to hold a URLSearchParams handle (a tagged
    /// growable pair-store handle) in THIS emitter's scope — a
    /// `const q = new URLSearchParams(<string-literal>)` declarator the
    /// intercept fired for.
    pub(crate) usp_locals: std::collections::BTreeSet<String>,
    /// Every declarator NAME already emitted in THIS emitter (source order).
    /// Stage-review C-4: URL/USP provenance is name-keyed and FLAT (no block
    /// scoping), so a block-scoped redeclaration of a provenance-carrying name
    /// desyncs name→value. Two uses at the declarator choke: (1) declaring a
    /// name already in `url_locals`/`usp_locals` is denied E5506; (2) the
    /// URL/USP construction intercept refuses a name that was ALREADY declared
    /// (the init then falls to the generic path, where the F10 ctor deny fails
    /// it closed).
    pub(crate) declared_binding_names: std::collections::BTreeSet<String>,
    /// One position-allowlist flag covering BOTH URL/USP handle classes (both
    /// are escape-restricted identically). A bare read of a URL/USP binding is
    /// E5506 unless an allowlisted consumer set this while emitting its
    /// receiver (`emit_url_receiver_handle`). Mirrors `admit_abort_handle_read`.
    pub(crate) admit_url_handle_read: bool,
    /// Stage P5: bindings proven to hold a `TextEncoder().encode(...)` byte
    /// handle (an i64 handle to the zero-copy byte buffer) in THIS emitter's
    /// scope — a `const b = enc.encode(<string>)` / `new TextEncoder().encode`
    /// declarator the intercept fired for. Repr::Bytes provenance; the raw
    /// handle must never escape as an observable value — reads are denied at the
    /// identifier choke unless `admit_bytes_handle_read`. Mirrors `usp_locals`.
    pub(crate) bytes_locals: std::collections::BTreeSet<String>,
    /// Stage P5: bindings proven to hold a stateless `new TextEncoder()` marker
    /// (the encoder is stateless, so the marker carries no value — the binding
    /// exists only to recognize a bound `enc.encode(...)` receiver). Mirrors
    /// `usp_locals`.
    pub(crate) text_encoder_locals: std::collections::BTreeSet<String>,
    /// Stage P5 Task 4: bindings proven to hold a stateless `new TextDecoder()`
    /// marker — the decoder twin of `text_encoder_locals`. The decoder is
    /// stateless (kali only supports the default `utf-8`, non-fatal decoder), so
    /// the marker carries no value: the binding exists only to recognize a bound
    /// `dec.decode(...)` receiver. Reads are denied at the SAME identifier choke
    /// as the encoder marker / byte handle.
    pub(crate) text_decoder_locals: std::collections::BTreeSet<String>,
    /// Stage P5 position-allowlist flag for byte-handle reads (the
    /// `admit_url_handle_read` pattern): a bare read of a `bytes_locals` /
    /// `text_encoder_locals` binding is E5506 unless an allowlisted consumer set
    /// this while emitting its operand (`crypto.subtle.digest` operand; later
    /// `TextDecoder().decode` receiver-arg).
    pub(crate) admit_bytes_handle_read: bool,
    /// Stage P5 T-new-C: bindings proven to hold an `Event`/`CustomEvent`
    /// COMPILE-TIME marker in THIS emitter's scope, mapped to the event's type
    /// TEXT (the constructor's string-literal argument). Recorded by the
    /// declarator intercept for a `const` whose init is
    /// `new Event(<string literal>)` with the constructor unshadowed. The marker
    /// carries no runtime value: `<ident>.type` materializes the interned text
    /// directly from this map, and EVERY other read of the name is denied at the
    /// identifier choke. Mirrors `text_encoder_locals` (a marker side-table),
    /// with the type text as the payload.
    pub(crate) event_marker_locals: std::collections::BTreeMap<String, String>,
    /// Stage P5 Task 4 review fix (C-4): the PRODUCE-side twin of
    /// `admit_bytes_handle_read`. The read choke only guards BOUND handles
    /// (`bytes_locals` identifiers); an INLINE, unbound
    /// `new TextEncoder().encode('hi')` never passes through an identifier read,
    /// so `console.log(new TextEncoder().encode('hi'))` escaped the choke and
    /// printed `hi` where node prints `Uint8Array(2) [ 104, 105 ]` — exactly the
    /// divergence the read choke exists to prevent. The encode arm therefore
    /// emits its handle ONLY while an allowlisted consumer set this flag: the
    /// `const b = <enc>.encode(<string>)` declarator intercept, the
    /// `TextDecoder().decode` operand, and the `crypto.subtle.digest` operand.
    /// Every other position fails closed.
    pub(crate) admit_bytes_handle_produce: bool,
    /// Stage P5 T-new-A: every binding in THIS emitter's scope whose
    /// initializer is structurally a `crypto.getRandomValues(...)` CALL RESULT
    /// — the DENY DOMAIN. Recorded unconditionally at the declarator/assignment
    /// choke, BEFORE any admission test, so a result that is not provably
    /// array-backed can never fall through to the silent-zero placeholder
    /// (`fb.length` read `0` where node reads `8`, and the crypto bundle
    /// trapped on its own self-check).
    pub(crate) crypto_random_result_bindings: std::collections::BTreeSet<String>,
    /// Stage P5 T-new-A: the ADMITTED SUBSET of
    /// `crypto_random_result_bindings` — those whose recorded init additionally
    /// proved (a) the callee lowered through the `crypto_get_random_values`
    /// import arm, which returns the argument handle UNCHANGED (JS identity),
    /// (b) exactly one argument, which is a bare identifier RECORDED in
    /// `array_bindings` (a `new Array(n)` / `new Uint8Array(n)` allocation or a
    /// repr-table-proven array param — positive recorded evidence, never a
    /// default), and (c) the binding itself owns a WASM local slot, so its
    /// `.length` reads the header off the handle IT holds (not off the
    /// argument's binding, which may be reassigned in between).
    pub(crate) crypto_random_result_array_bindings: std::collections::BTreeSet<String>,
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
    /// `const` names promoted to a local slot by the stability allowlist — the
    /// ones that get a `bindings` denotation entry DESPITE having a slot. A
    /// handle-promoted `const` (`ConstPromotion::Handle`) is deliberately
    /// absent: its lanes key on provenance sets and a denotation entry would
    /// re-resolve the name to its initializer.
    pub(crate) allowlist_promoted_consts: HashSet<String>,
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
    /// Fix 5 (call-through-a-first-class-function-value) choke-point caches.
    /// Both are whole-PROGRAM structural facts, so they are computed once per
    /// emitter and reused across every denial decision in this function body.
    /// See `call_target_keeps_placeholder_lowering` in `emit/call.rs`.
    pub(crate) program_bound_names_cache: std::cell::OnceCell<HashSet<String>>,
    pub(crate) program_fn_valued_property_names_cache: std::cell::OnceCell<HashSet<String>>,
    pub(crate) program_stores_function_in_aggregate_cache: std::cell::OnceCell<bool>,
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
    /// Stage P2 review C-2: `true` only while a growable-aware recognizer
    /// (push/join/length/index/for-of receiver, or the Lane-3 `===` field pair)
    /// is deliberately reading a `GrowableArrayI64` FIELD receiver's tagged
    /// handle — set/restored by `emit_growable_receiver_handle`. `emit_unary`'s
    /// generic member-read arm admits an `object_field_is_growable_array` read
    /// ONLY when this flag is set (an allowlisted SAFE position — the Spec-4a
    /// headline lesson: allowlist safe positions at the single read site, don't
    /// denylist sinks). Every other value position (`console.log(o.values)`,
    /// `o.values + 1`, `const a = o.values`) fails closed E5506 so the raw
    /// ARRAY_HANDLE_TAG handle never escapes as an observable scalar.
    pub(crate) admit_growable_field_read: bool,
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
    /// R-11 T3 review Important 1: names in `module_global_slots` whose
    /// declarator init OR some reassignment RHS is a raw BigInt literal
    /// (optionally negated, e.g. `let n = 6n;` or `n = -3n;` reached from any
    /// scope). `is_numeric_expr` (the promotion proof in
    /// `collect_module_scalar_globals`/`scan_numeric_assignments`) strips the
    /// trailing `n` before checking, so such a name is "provably numeric" and
    /// promotes exactly like a genuine integer — a pre-existing, deferred gap
    /// for the ten arithmetic/plain-assignment operators (BigInt truncation)
    /// this task does not touch. The six bitwise ops' raw `I32WrapI64`
    /// combiner cannot share that gap silently: consulted ONLY by the
    /// bitwise compound-assign arm (`literal.rs`) to fail closed instead of
    /// computing a wrong value from a truncated BigInt. See
    /// `kali_codegen::lower::collect_bigint_tainted_module_scalars`.
    pub(crate) module_global_bigint_targets: &'a HashSet<String>,
    /// R-11 T4: whole-program BigInt-taint set for promoted SCALAR env
    /// cells — the identical class `module_global_bigint_targets` closes for
    /// module globals, reapplied to captured bindings. `numeric_bindings`
    /// (the proof `binding_is_proven_numeric` reads) admits a BigInt literal
    /// write exactly like a plain number, so it cannot by itself tell `let
    /// flags = 6n;` from `let flags = 6;`. Consulted ONLY by the bitwise
    /// compound-assign captured-cell arm (`closure_access.rs`'s
    /// `try_emit_captured_assign`), keyed by NAME ONLY (not `(owner, name)`)
    /// — see `kali_codegen::lower::collect_bigint_tainted_captured_cells`'s
    /// doc for why that is a deliberate, conservative over-denial rather
    /// than an under-taint.
    pub(crate) captured_cell_bigint_targets: &'a HashSet<String>,
    /// R-11 T4 review Important 1: whole-program FLOAT-taint set for promoted
    /// SCALAR env cells (`collect_float_tainted_captured_cells`), the sibling
    /// of `captured_cell_bigint_targets` on a different axis.
    /// `crate::closure::cell_is_promotable`'s owner-scoped `scalar(owner,
    /// name) == I64` check cannot be trusted to exclude every float write to
    /// a captured cell: `repr_infer`'s scalar-repr union-find resolves an
    /// off-scope write's node key via `binding_scope`, which cannot name the
    /// true owner when the write is reached from a THIRD function (neither
    /// the owner nor top-level module scope) — such a write is filed under a
    /// disconnected union-find node the owner-scoped query never sees. Also
    /// keyed by NAME ONLY, for the same reason
    /// `captured_cell_bigint_targets` is.
    pub(crate) captured_cell_float_targets: &'a HashSet<String>,
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
        module_global_bigint_targets: &'a HashSet<String>,
        captured_cell_bigint_targets: &'a HashSet<String>,
        captured_cell_float_targets: &'a HashSet<String>,
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
        // `const` names promoted to a local slot by the STABILITY allowlist.
        // These still need a compile-time denotation entry in `bindings` (the
        // alias/intrinsic analyses read it); their RUNTIME reads resolve
        // through `locals`, which the identifier path consults first.
        let allowlist_promoted_consts = crate::lower::allowlist_promoted_const_names(
            &program.nodes,
            body,
            repr_table,
            function_name,
        );
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
            abort_handle_locals: BTreeSet::new(),
            admit_abort_handle_read: false,
            url_locals: BTreeSet::new(),
            usp_locals: BTreeSet::new(),
            declared_binding_names: BTreeSet::new(),
            admit_url_handle_read: false,
            bytes_locals: BTreeSet::new(),
            text_encoder_locals: BTreeSet::new(),
            text_decoder_locals: BTreeSet::new(),
            admit_bytes_handle_read: false,
            event_marker_locals: std::collections::BTreeMap::new(),
            admit_bytes_handle_produce: false,
            crypto_random_result_bindings: BTreeSet::new(),
            crypto_random_result_array_bindings: BTreeSet::new(),
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
            allowlist_promoted_consts,
            fn_valued_locals: BTreeMap::new(),
            unstable_provenance_names,
            program_bound_names_cache: std::cell::OnceCell::new(),
            program_fn_valued_property_names_cache: std::cell::OnceCell::new(),
            program_stores_function_in_aggregate_cache: std::cell::OnceCell::new(),
            array_bindings,
            growable_array_bindings,
            growable_for_of_active: None,
            admit_growable_field_read: false,
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
            module_global_bigint_targets,
            captured_cell_bigint_targets,
            captured_cell_float_targets,
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

    /// Stage P3: true when `name` is a proven abort handle in this function — a
    /// local provenance-set member, or a depth-1 captured binding whose OWNER is
    /// a REAL function (not `_start`) and whose repr-table entry is `AbortHandle`
    /// (the owner-keyed lookup pattern; no env-slot metadata needed — the cell
    /// holds the handle by value).
    ///
    /// The `owner != "_start"` guard is load-bearing: a module-scope (`_start`)
    /// `const c = new AbortController()` — including one declared inside a
    /// loop/block body, where the declarator intercept binds `c` as a plain
    /// `_start` LOCAL via `LocalSet` and never populates the captured env cell —
    /// must NOT be admitted here. Admitting it lets the abort-dispatch arm store
    /// through the wrong cell and a deferred callback read a stale/zero cell,
    /// silently miscompiling (`c.abort()` becomes a no-op).
    ///
    /// Such handles fail closed at the CHOKE POINTS (not at the capture site —
    /// the capture is admitted by host.rs ALLOWLIST 1 as a by-value scalar, so
    /// the whole-program deny is what the choke-point diagnostics produce):
    /// `is_module_scope_abort_handle` denies a method call on the receiver
    /// (`emit/call.rs`) AND a bare/member read of it (`emit/control_flow.rs`
    /// identifier + abort member-read arms). Do NOT revert this exclusion —
    /// re-admitting `_start` owners re-enables the wrong-cell store.
    pub(crate) fn is_abort_handle(&self, name: &str) -> bool {
        if self.abort_handle_locals.contains(name) {
            return true;
        }
        self.env_plan.captured.iter().any(|reference| {
            reference.name == name
                && reference.depth == 1
                && reference.owner != "_start"
                && self.repr_table.scalar(&reference.owner, &reference.name)
                    == kali_common::Repr::AbortHandle
        })
    }

    /// True when `name` is a `_start`-OWNED binding proven `AbortHandle` (module
    /// repr is keyed under owner `"_start"`) and we are emitting a NON-`_start`
    /// function — i.e. a `_start`-scope abort handle referenced from inside a
    /// function/closure, which the ratified (function-scoped, owner-keyed)
    /// capture lane in `is_abort_handle` does NOT admit. Consulted at the
    /// method-call choke point to fail such calls closed (E5506) instead of
    /// letting them silently drop through the generic zero-placeholder fallback
    /// (the write-position twin of the read-position `module_binding_names`
    /// gate). Refuses when a local of the same name shadows the binding (that
    /// local already routes through `is_abort_handle` / normal lookup).
    ///
    /// Keyed directly on the `_start` repr (NOT `module_binding_names`) so it
    /// also covers a handle declared inside a `_start` loop/block body — such a
    /// `const c = new AbortController()` is a `_start` LOCAL, not a top-level
    /// module binding, so a `module_binding_names` restriction would let a
    /// deferred `c.abort()` fall through and silently no-op. The remaining
    /// not-a-current-fn-local guards keep a genuine same-named local of the
    /// emitting function on its own (`is_abort_handle`) lane.
    pub(crate) fn is_module_scope_abort_handle(&self, name: &str) -> bool {
        self.function_name != "_start"
            && !self.abort_handle_locals.contains(name)
            && !self.locals.contains_key(name)
            && self.repr_table.scalar("_start", name) == kali_common::Repr::AbortHandle
    }

    /// Stage P4: `name` is a proven URL binding in THIS emitter's scope.
    pub(crate) fn is_url(&self, name: &str) -> bool {
        self.url_locals.contains(name)
    }

    /// Stage P4: `name` is a proven URLSearchParams binding in THIS emitter's
    /// scope.
    pub(crate) fn is_url_search_params(&self, name: &str) -> bool {
        self.usp_locals.contains(name)
    }

    /// Stage P5: `name` is a proven `TextEncoder().encode(...)` byte handle
    /// (Repr::Bytes) in THIS emitter's scope.
    pub(crate) fn is_bytes_handle(&self, name: &str) -> bool {
        self.bytes_locals.contains(name)
    }

    /// Stage P5: `name` is a proven stateless `new TextEncoder()` marker in THIS
    /// emitter's scope.
    pub(crate) fn is_text_encoder_marker(&self, name: &str) -> bool {
        self.text_encoder_locals.contains(name)
    }

    /// Stage P5 Task 4: `name` is a proven stateless `new TextDecoder()` marker
    /// in THIS emitter's scope.
    pub(crate) fn is_text_decoder_marker(&self, name: &str) -> bool {
        self.text_decoder_locals.contains(name)
    }

    /// Stage P5 T-new-C: the BARE-identifier receiver name of a one-child
    /// member node `<ident>.<field>`. `None` when the receiver is anything other
    /// than a childless identifier (a nested member, a call, a literal), so a
    /// chained `o.e.type` never resolves to a marker name. Deliberately does NOT
    /// unwrap transparent wrappers: a textless one-child `Value` is also a
    /// single-element ARRAY literal, and tunneling would let `[e].type` claim the
    /// marker.
    pub(crate) fn bare_member_receiver_name(&self, node: &LirNode) -> Option<String> {
        if node.children.len() != 1 {
            return None;
        }
        let base = self.node(node.children[0]);
        if !base.children.is_empty() {
            return None;
        }
        base.text.clone().filter(|text| !text.is_empty())
    }

    /// Stage P5 T-new-C: the compile-time event TYPE text of a proven
    /// `Event`/`CustomEvent` marker in THIS emitter's scope, or `None` when
    /// `name` is not such a marker. This is RECORDED EVIDENCE (the declarator
    /// intercept fired and stored the constructor's literal argument), never a
    /// default.
    pub(crate) fn event_marker_type(&self, name: &str) -> Option<&str> {
        self.event_marker_locals.get(name).map(String::as_str)
    }

    /// Stage P5 T-new-C: `name` is a proven event marker in THIS emitter's scope.
    pub(crate) fn is_event_marker(&self, name: &str) -> bool {
        self.event_marker_locals.contains_key(name)
    }

    /// Stage P5 T-new-D: the ONE stale-provenance shadow test.
    ///
    /// Every opaque-handle / marker lane in this emitter keeps a NAME-KEYED,
    /// FLAT side-table (`url_locals`, `usp_locals`, `abort_handle_locals`,
    /// `bytes_locals`, `text_encoder_locals`, `text_decoder_locals`,
    /// `event_marker_locals`, `event_target_locals`,
    /// `crypto_random_result_bindings`). None of them is block-scoped, so when
    /// an INNER binding shadows a recorded name, a member read on the SHADOWING
    /// binding is answered from the STALE handle — measured across four lanes
    /// as a silent, exit-0 wrong value (`for (const u of ['aa']) u.pathname`
    /// printed the outer URL's `/p`; `for (const c of ['aa']) c.abort()` fired
    /// a real side effect through the shadow).
    ///
    /// There are exactly TWO binding-introduction chokes a shadow can enter
    /// through — the `const`/`let`/`var` DECLARATOR list and the FOR-OF loop
    /// binding — and before this helper each lane had to remember to add an arm
    /// at both (five of the lanes remembered neither or only one). A per-lane
    /// arm is a denylist that leaks on every new lane; this predicate is the
    /// single test both chokes call, so a lane added to the OR below is closed
    /// at both by construction.
    ///
    /// Returns the LANE LABEL (a noun phrase for the diagnostic) so each call
    /// site can phrase its own sentence while keeping the per-lane needle a
    /// test can key on; `None` when `name` carries no handle provenance (the
    /// overwhelmingly common case — this must not over-deny an ordinary
    /// binding).
    pub(crate) fn stale_provenance_shadow_lane(&self, name: &str) -> Option<&'static str> {
        // URL/USP share one label: the C-4 guard's original wording, which the
        // Stage P4 pins key on.
        if self.is_url(name) || self.is_url_search_params(name) {
            return Some("a URL/URLSearchParams");
        }
        if self.is_event_marker(name) {
            return Some("an Event/CustomEvent");
        }
        if self.event_target_locals.contains(name) {
            return Some("an EventTarget");
        }
        // The whole abort DENY domain, captured handles included (the read
        // sites use exactly this predicate, so any shadow they could answer
        // from is denied here).
        if self.is_abort_handle(name) {
            return Some("an AbortController/AbortSignal");
        }
        if self.is_bytes_handle(name) {
            return Some("a TextEncoder().encode() byte handle");
        }
        if self.is_text_encoder_marker(name) {
            return Some("a TextEncoder");
        }
        if self.is_text_decoder_marker(name) {
            return Some("a TextDecoder");
        }
        // The crypto DENY DOMAIN (admitted subset included): the admitted
        // `.length` loads a length header off whatever the local holds, so a
        // shadow would load off the shadowing value.
        if self.crypto_random_result_bindings.contains(name) {
            return Some("a crypto.getRandomValues(...) result");
        }
        None
    }

    /// Stage P5 T-new-C read-position twin of `is_module_scope_url_handle`: a
    /// `_start`-owned event marker reached from a non-`_start` emitter. The
    /// marker is a COMPILE-TIME side-table entry private to the emitter that
    /// declared it, so an inner function has no way to recover the type text —
    /// deny rather than fall through to the placeholder/module-binding lanes.
    pub(crate) fn is_module_scope_event_marker(&self, name: &str) -> bool {
        self.function_name != "_start"
            && !self.event_marker_locals.contains_key(name)
            && !self.locals.contains_key(name)
            && self.repr_table.scalar("_start", name) == kali_common::Repr::Event
    }

    /// Stage P5 T-new-C CAPTURED twin of `is_captured_url_handle`: an event
    /// marker owned by an ENCLOSING function, reached through the closure env
    /// plan. Keyed on the OWNER's recorded repr verdict. There is no captured
    /// lane for a marker (it has no runtime value to promote into an env cell),
    /// so absent this gate an inner `e.type` falls through to the silent
    /// zero-placeholder lane — the exact defect the preceding task shipped one
    /// scope inwards.
    pub(crate) fn is_captured_event_marker(&self, name: &str) -> bool {
        !self.event_marker_locals.contains_key(name)
            && !self.locals.contains_key(name)
            && self.env_plan.captured.iter().any(|reference| {
                reference.name == name
                    && self.repr_table.scalar(&reference.owner, &reference.name)
                        == kali_common::Repr::Event
            })
    }

    /// Stage P4 read-position twin of the abort `is_module_scope_abort_handle`
    /// gate: a `_start`-owned URL/USP binding reached from a non-`_start`
    /// emitter. Fail-closed — the arena struct's lifetime is unproven across
    /// frames, and the handle is never captured into the callee's env, so a
    /// deferred read would be stale. Refuses when a same-named local shadows
    /// the binding (that local routes through `is_url`/`is_url_search_params`).
    pub(crate) fn is_module_scope_url_handle(&self, name: &str) -> bool {
        self.function_name != "_start"
            && !self.url_locals.contains(name)
            && !self.usp_locals.contains(name)
            && !self.locals.contains_key(name)
            && matches!(
                self.repr_table.scalar("_start", name),
                kali_common::Repr::Url | kali_common::Repr::UrlSearchParams
            )
    }

    /// Stage P4 Task 6 (enumeration-wave close): `name` is a URL/USP handle
    /// OWNED BY AN ENCLOSING FUNCTION, reached through the closure env plan —
    /// the CAPTURED twin of `is_module_scope_url_handle`, keyed on the OWNER's
    /// repr verdict exactly like `is_abort_handle`'s captured clause. Unlike
    /// abort (whose captured handles have a ratified admitted lane), URL/USP
    /// handles have NO captured lane: the arena struct / pair store is never
    /// materialized into an env cell (`cell_is_promotable` refuses the repr),
    /// so `try_emit_captured_read` returns `None` and the read would fall
    /// through to the silent zero-placeholder lane — a leak
    /// (`setTimeout(function() { q.get('a') })` silently no-ops where node
    /// queries). Consulted as a DENY at the identifier choke
    /// (`emit/control_flow.rs`) and the method-call choke (`emit/call.rs`);
    /// any depth, any non-module owner (module owners take the
    /// `is_module_scope_url_handle` lane; they are never env-captured).
    pub(crate) fn is_captured_url_handle(&self, name: &str) -> bool {
        !self.url_locals.contains(name)
            && !self.usp_locals.contains(name)
            && !self.locals.contains_key(name)
            && self.env_plan.captured.iter().any(|reference| {
                reference.name == name
                    && matches!(
                        self.repr_table.scalar(&reference.owner, &reference.name),
                        kali_common::Repr::Url | kali_common::Repr::UrlSearchParams
                    )
            })
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
