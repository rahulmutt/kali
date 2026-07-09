//! Interprocedural int-vs-float representation inference.
//!
//! Float is a **forward-flow** property of scalar program points: a node is
//! `f64` iff a float *seed* reaches it along directed "if source is float then
//! target is float" edges (`operand -> result`, `rhs -> lhs`, `expr -> return`,
//! `arg -> param`, `return -> callsite`, `value -> array_element`,
//! `array_element -> read_result`). Float seeds are division results,
//! non-integer literals, `Math.sqrt`/`Math.cbrt` results, `.toFixed` receivers,
//! and `/=` targets. The property is solved by BFS reachability from the seed
//! set — it is deliberately **directional** so that an integer operand binding
//! that merely feeds a float result (e.g. `i` in `(i + j) / 2`) is NOT floated;
//! codegen converts each i64 operand inline where a float is needed.
//!
//! Array *element storage*, by contrast, is a single shared property of an
//! array's memory. Element identity across aliases (interprocedural array
//! arg↔param passing) is modelled with a **bidirectional** [`UnionFind`]: the
//! two arrays share ONE element node. Stores and reads are still directed edges
//! into / out of that shared node (an int stored into a float array stays int
//! and is converted at the store).
//!
//! The solved decisions populate a [`ReprTable`]; only float decisions are
//! recorded, so an all-integer program yields an empty table. Two axes are
//! tracked per program point, both defaulting to `I64`: a scalar repr (per
//! binding/param/return) and an array *element* repr. Array handles themselves
//! are always `i64`; only their elements can become `f64`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use kali_ast::{
    AssignmentOperator, BlockStatement, Expression, ForInLefthand, ForInit, ForOfLefthand,
    LiteralValue, Statement,
};
use kali_common::{Repr, ReprTable, UnionFind};

/// Synthetic function name for top-level statements, matching codegen's entry.
const TOP_LEVEL: &str = "_start";

/// `process.argv[<int literal>]` element read (Spec 5 Task 5). Structural
/// mirror of `TypeContext::is_process_argv_element_expr` / codegen's
/// `is_process_argv_element`, over a `MemberExpression` so `visit_member` can
/// register its result as a runtime-string node. Only a static non-negative
/// integer index qualifies (codegen emits `0` for anything else).
fn member_is_process_argv_element(member: &kali_ast::MemberExpression) -> bool {
    let Some(index) = &member.computed_index else {
        return false;
    };
    expr_is_nonneg_int_literal(index) && expr_is_process_argv(&member.object)
}

fn expr_is_process_argv(expr: &Expression) -> bool {
    let Expression::MemberExpression(member) = expr else {
        return false;
    };
    if member.computed_index.is_some() || member.property.as_str() != "argv" {
        return false;
    }
    expr_is_process_root(&member.object)
}

fn expr_is_process_root(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(name) => name == "process",
        Expression::MemberExpression(member) => {
            member.computed_index.is_none()
                && member.property.as_str() == "process"
                && matches!(&member.object, Expression::Identifier(root) if root == "globalThis")
        }
        _ => false,
    }
}

/// Mirror of `TypeContext::expression_is_nonneg_int_literal`
/// (`resolve/expression.rs`) — bounded to `n <= 9007199254740991.0` (2^53 - 1,
/// `Number.MAX_SAFE_INTEGER`) rather than `i64::MAX as f64` so the accepted
/// set exactly round-trips through codegen's `str::parse::<i64>()`
/// (`is_process_argv_element`) with no residual boundary mismatch; see the
/// comment on the `resolve/expression.rs` twin for the full rationale. Keep
/// both copies in lockstep.
fn expr_is_nonneg_int_literal(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Literal(LiteralValue::Number(n))
            if n.fract() == 0.0 && *n >= 0.0 && *n <= 9007199254740991.0
    )
}

/// A deferred interprocedural call constraint, resolved after every function
/// body has been walked (so all param/return/element nodes already exist).
struct CallEdge {
    /// Bare-identifier callee name.
    callee: String,
    /// Result node of each positional argument (scalar flow).
    arg_nodes: Vec<usize>,
    /// For each positional argument, `Some((caller_func, name))` when the
    /// argument is a bare identifier (candidate array binding), else `None`.
    arg_array_names: Vec<Option<(String, String)>>,
    /// For each positional argument, the object slot the argument's value
    /// aliases, when one exists: a bare identifier (binding), `arr[i]`
    /// (array element), or a bare-identifier call (callee return).
    arg_obj_slots: Vec<Option<ObjSlot>>,
    /// For each positional argument, `true` when the argument is SYNTACTICALLY
    /// a fresh array value — an array literal `[..]`, `new Array(..)`, or
    /// `Array(..)` — passed directly (no bare identifier to route through the
    /// array-binding fixpoint). Lets `resolve_calls` taint the callee param as
    /// non-scalar for `f([1, 2])`-shaped calls.
    arg_array_literal: Vec<bool>,
    /// For each positional argument, `true` when the argument is SYNTACTICALLY a
    /// provably-scalar primitive expression — a number/string/boolean literal, a
    /// template literal, or a binary/unary/update expression (all of which
    /// evaluate to a primitive number/string/boolean, never a heap array or
    /// object). This is the sole positive scalar-inflow evidence
    /// `scalar_inflow_params` is derived from (see `resolve_calls`); a bare
    /// identifier argument is NEVER scalar evidence, even when it names a
    /// param already proven scalar (no pass-through: see `resolve_calls`).
    arg_scalar_syntactic: Vec<bool>,
    /// Result node of the call expression itself (target of the callee's
    /// return-flow edge).
    result_node: usize,
}

#[derive(Default)]
struct ReprInfer {
    /// Bidirectional union-find, used ONLY for array-element storage aliasing
    /// (interprocedural array arg↔param). Node ids are allocated here so the
    /// directed reachability graph and the array-aliasing forest share one id
    /// space; scalar/param/return/result nodes are never unioned, so their
    /// representative is always themselves.
    uf: UnionFind,
    /// Number of allocated nodes (== next node id). Tracked because
    /// [`UnionFind`] exposes no length accessor.
    node_count: usize,
    /// Directed float-flow edges `(from, to, in_string_axis)`.
    /// `float(from) ⇒ float(to)` always; the boolean records whether the edge
    /// also carries the STRING axis. Endpoints are canonicalised through `uf`
    /// at solve time so edges touching aliased array-element nodes follow the
    /// shared representative. Array-element and object-field *read* edges are
    /// added float-only (`in_string_axis == false`, via `add_edge_float_only`):
    /// codegen materialises those reads as raw i64/f64 and has no string lane
    /// for them, so a scalar capturing such a read must NOT be proven
    /// `Repr::String` (that would vouch for a runtime raw int — a miscompile).
    /// The float axis keeps the edge (an f64 element read still floats its
    /// captor).
    edges: Vec<(usize, usize, bool)>,
    /// Directly-float nodes (division results, float literals, `Math.sqrt`, …).
    seeds: Vec<usize>,
    /// Directed reachability seeds for the STRING axis (string/template literals).
    string_seeds: Vec<usize>,
    /// Candidate RUNTIME-string-producing nodes: `+` results (string concat),
    /// interpolated template-literal results, and string `+=` targets. All are
    /// FRESH runtime handles at execution time (not interned literal constants).
    /// The taint pass seeds those of these that are string-reachable and
    /// forward-propagates over the string adjacency; any scalar/param/return it
    /// reaches is a "runtime-concat-derived" string whose fresh handle must NOT
    /// be compared by identity (`==`/`!=`) or tested for truthiness — see the
    /// `string_concat_tainted*` sets on `ReprTable`.
    runtime_string_nodes: Vec<usize>,
    /// Element-store edges `(element node, stored-value node)` — one entry per
    /// `a[i] = v` / `.fill(v)` / array-literal-init element. Consulted at
    /// emit_table time to fail-close arrays mixing string and non-string
    /// stores (the element node itself unions both axes, so reachability
    /// alone cannot see the mix).
    element_store_sources: Vec<(usize, usize)>,
    /// Directed reachability seeds for the NON-ASCII provenance axis:
    /// non-ASCII string literals and interpolated template results (whose
    /// interpolations are not modeled, so their contents are unprovable).
    non_ascii_seeds: Vec<usize>,
    /// One node per scalar binding/param/local: `(func, name) -> node`.
    scalar_node: BTreeMap<(String, String), usize>,
    /// One node per array binding/param element repr: `(func, name) -> node`.
    array_elem_node: BTreeMap<(String, String), usize>,
    /// One node per function's return value: `func -> node`.
    return_node: BTreeMap<String, usize>,
    /// Ordered parameter names of every user `FunctionDeclaration`.
    functions: BTreeMap<String, Vec<String>>,
    /// Names locally bound within a given scope: a function's own parameters
    /// plus every `let`/`const`/`var` declarator reachable from its body
    /// without descending into a nested function (module scope uses the
    /// `TOP_LEVEL` key). Lets identifier resolution tell a local read from a
    /// module-scope read regardless of source order, mirroring codegen's
    /// `self.locals`/`self.bindings` shadow precedence (see
    /// `kali_codegen::emit::control_flow`'s identifier fallback and
    /// `kali_codegen::lower`'s `module_binding_names`).
    local_names: BTreeMap<String, BTreeSet<String>>,
    /// Deferred interprocedural call constraints.
    calls: Vec<CallEdge>,
    /// Ordered field names of each slot directly initialized by an object literal.
    obj_literal_fields: BTreeMap<ObjSlot, Vec<String>>,
    /// Bidirectional object-aliasing flows (assignment, array element,
    /// arg↔param, return↔call-site). Harmless for scalar slots: flows only
    /// take effect for slots proven to hold object literals.
    obj_flows: Vec<(ObjSlot, ObjSlot)>,
    /// Deferred member accesses (wired in `resolve_objects`).
    obj_accesses: Vec<ObjAccess>,
    /// Per-(slot, field) storage node, unioned across aliased slots.
    obj_field_node: BTreeMap<(ObjSlot, String), usize>,
    /// Slots that must lower as runtime heap objects (any write, any flow).
    obj_materialized: BTreeSet<ObjSlot>,
    /// Object slots with their propagated field lists (set by `resolve_objects`).
    obj_fields_of: BTreeMap<ObjSlot, Vec<String>>,
    /// Deferred *structural* gate messages, keyed by the slot whose literal is
    /// unsupported on the runtime object lane (non-identifier property name,
    /// getter/setter, nested object). Emitted ONLY if that slot later
    /// materializes; a read-only fold-lane literal (e.g. a string-keyed object
    /// consumed only by `Object.keys`) never materializes and so keeps today's
    /// fold-lane behavior byte-identically (fold-first invariant).
    obj_pending_conflicts: BTreeMap<ObjSlot, String>,
    /// Gate messages (unsupported or contradictory object usage).
    obj_conflicts: Vec<String>,
    /// Value-SELECTING merge points and their direct inputs: `a || b`,
    /// `a && b`, `cond ? a : b` (`(result_node, [input_nodes])`). Unlike `+`
    /// (whose result is a genuine string whenever either operand is), a
    /// selecting merge yields ONE of its inputs unchanged at runtime, so a
    /// string-reachable merge with a plain-`I64` input can hold a raw integer
    /// where the solved repr says `String` — checked fail-closed in
    /// `emit_table` (see `plain_write_targets` for the assignment-shaped twin).
    merge_nodes: Vec<(usize, Vec<usize>)>,
    /// Active `for..in` key bindings (Spec 4a Task 3): `(func, key_name,
    /// base_slot)` pushed while visiting a `for (var key in base)` body,
    /// popped on exit. Lets `visit_member` recognize a computed read
    /// `base[key]` as a uniform-shape object FIELD read (not an array element
    /// read) so its result carries the shape's element repr. The repr_infer
    /// twin of codegen's `for_in_key_shapes` and the resolve-phase
    /// `for_in_key_bindings` registry.
    for_in_key_bases: Vec<(String, String, ObjSlot)>,
    /// Grow-only `(func, key_name)` for EVERY `for..in` key ever seen in
    /// `func` — populated when a key is pushed onto `for_in_key_bases` but
    /// NEVER removed (unlike the lexical stack). Mirrors the resolve-phase
    /// grow-only `for_in_key_bindings` registry and codegen's grow-only
    /// `for_in_key_handle_tables`: a key's String-materialization provenance
    /// persists past the loop body, so a key stored into an array element
    /// AFTER the loop exits (the fasta `fastaRandom` shape: `for (c in t) ...
    /// break; line[i] = c`) is still recognized as a string sink. The lexical
    /// `for_in_key_bases` stack is kept separately for the `base[key]`
    /// uniform-object read gate, which MUST stay lexically scoped.
    for_in_key_names: BTreeSet<(String, String)>,
    /// Deferred computed uniform-object reads `base[key]` — `(base_slot,
    /// result_node)`, wired to the shape's field storage in `resolve_objects`
    /// so the read result carries the (uniform) field repr.
    uniform_computed_reads: Vec<(ObjSlot, usize)>,
    /// `(func, name)` for every `return <identifier>;` — the function `func`
    /// returns the bare binding `name`. At `emit_table` time, if `name`'s
    /// element node solves `Repr::String`, the return is a String-element
    /// array with NO codegen lowering (the caller captures a raw i64 handle;
    /// element reads / `join` on the captured value silently yield `0`), so it
    /// is FAIL-CLOSED with a shape conflict (I2). Recorded unconditionally and
    /// filtered at emit time, keeping the check monotone: int/float array
    /// returns (element node not String) add no conflict.
    array_binding_returns: Vec<(String, String)>,
    /// `(func, param)` params proven to receive a non-scalar (array) argument
    /// at some call site — copied verbatim into
    /// [`ReprTable::non_scalar_params`](kali_common::ReprTable) at emit time.
    /// The resolve-phase param compound/update gate uses it to fail closed.
    non_scalar_params: BTreeSet<(String, String)>,
    /// `(func, param)` parameters POSITIVELY proven to receive a scalar
    /// (numeric/string/boolean) value by at least one call-site edge. Computed
    /// by a single pass in `resolve_calls` (Step 1b) over syntactically-scalar
    /// arguments only — no propagation/fixpoint, since a bare-identifier
    /// argument is never scalar evidence. Every param NOT in this set is left at
    /// the default I64 by CONVENTION only (no scalar flow evidence — an array or
    /// object could have reached it via an indirect call shape the array taint
    /// cannot see), so the param compound/update gate must reject it. Copied
    /// (negated) into [`ReprTable::params_lacking_scalar_inflow`] at emit time.
    scalar_inflow_params: BTreeSet<(String, String)>,
    /// `(func, binding)` var/let/const locals whose declarator RHS is an
    /// object literal — copied verbatim into
    /// [`ReprTable::object_initialized_bindings`](kali_common::ReprTable) at
    /// emit time. Object shape inference only assigns a binding `Repr::Object`
    /// when the object is "materialized" (reached by a field read elsewhere in
    /// the program — see `obj_materialized`); an object-initialized binding
    /// that is never field-read (e.g. `var o = {x:1}; o += 1;`) stays at the
    /// default `Repr::I64`, so `target_repr_is_one_of`'s repr-allowlist check
    /// alone cannot see it. This taint is independent of materialization: it
    /// fires on the syntactic shape of the declarator RHS alone, so the
    /// resolve-phase compound/update gate can reject fail-closed regardless of
    /// whether the object ever gets a shape (fasta Spec 7 Task 2).
    object_initialized_bindings: BTreeSet<(String, String)>,
}

/// Identity of an object-holding slot for shape/aliasing purposes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ObjSlot {
    /// `(func, binding)` — a named binding or parameter.
    Binding(String, String),
    /// `(func, array_binding)` — every element of the named array (elements
    /// of one array share one shape and one per-field storage cluster).
    ArrayElem(String, String),
    /// `func` — the function's return value.
    Return(String),
}

/// A recorded `<base>.field` access: read (`other` = the result node) or
/// write (`other` = the stored-value node). Wired to shared field storage
/// after object propagation (`resolve_objects`).
struct ObjAccess {
    base: ObjSlot,
    field: String,
    other: usize,
    is_write: bool,
}

/// Whole-program pass: allocate nodes, add seeds + intra/inter-procedural
/// directed float-flow edges (plus array-element aliasing unions), solve by
/// reachability, and emit the [`ReprTable`].
pub fn infer_reprs(statements: &[Statement]) -> ReprTable {
    let mut infer = ReprInfer::default();

    // Phase A: collect every function signature (recursively) and eagerly
    // create a scalar node per parameter so interprocedural edges have a
    // stable target even for params never mentioned in the body.
    infer.collect_functions(statements);

    // Phase A2: collect every locally-declared name per scope (module scope
    // plus each function's own params/declarators), so identifier resolution
    // in Phase B can distinguish a local read from a module-scope read
    // regardless of source order.
    infer.collect_local_names(TOP_LEVEL, statements);

    // Phase B: walk bodies. Top-level non-function statements run under the
    // synthetic `_start`; each `FunctionDeclaration` runs under its own name.
    for stmt in statements {
        infer.visit_stmt(TOP_LEVEL, stmt);
    }

    // Phase C: resolve deferred call edges (transitive array-param fixpoint +
    // directed scalar/return edges + bidirectional array-element unions).
    infer.resolve_calls();

    // Phase C2: object-shape propagation (field lists across flows, shared
    // field storage unions, deferred member-access wiring, materialization).
    infer.resolve_objects();

    // Phase D: solve → table.
    infer.emit_table()
}

impl ReprInfer {
    // ---- node / edge / seed constructors -------------------------------

    /// Allocate a fresh node id (kept in the `uf` id space).
    fn new_node(&mut self) -> usize {
        let n = self.uf.fresh();
        self.node_count = n + 1;
        n
    }

    /// Record a directed flow edge `from -> to` carried by BOTH axes.
    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to, true));
    }

    /// Record a directed flow edge carried by the FLOAT axis ONLY (excluded
    /// from the string axis). Used for array-element and object-field *read*
    /// edges: an f64 element/field read still floats its captor, but a string
    /// stored in an element/field must not prove the captor `Repr::String`
    /// (codegen has no string lane for element/field reads).
    fn add_edge_float_only(&mut self, from: usize, to: usize) {
        self.edges.push((from, to, false));
    }

    /// Mark `node` as a direct float seed.
    fn add_seed(&mut self, node: usize) {
        self.seeds.push(node);
    }

    /// Mark `node` as a direct string seed.
    fn add_string_seed(&mut self, node: usize) {
        self.string_seeds.push(node);
    }

    /// True when `name` is a currently-active `for..in` key of `func` (its
    /// enclosing `for (key in base)` body is being visited). Spec 4a Task 5.
    fn is_active_for_in_key(&self, func: &str, name: &str) -> bool {
        self.for_in_key_bases
            .iter()
            .any(|(f, k, _)| f == func && k == name)
    }

    /// Spec 4a Task 5: if `expr` is a bare identifier that is an active
    /// `for..in` key used HERE in a string sink (a `return`, a `console.log`
    /// argument, or a `+`/equality operand), lift its value axis to `String` by
    /// seeding its scalar node. This makes `return c` solve `Repr::String`
    /// (so callers/`console.log(selectRandom(...))` treat the result as a
    /// string) AND makes codegen's `is_string_valued`/materialization fire on
    /// the key at that sink. The key's ORDINAL role (`table[c]`) is unaffected:
    /// the ordinal counter local is `i64` regardless (`wasm_type(String) ==
    /// I64`) and codegen emits the raw ordinal at index sites.
    fn seed_for_in_key_string_use(&mut self, func: &str, expr: &Expression) {
        if let Expression::Identifier(name) = expr {
            if self.is_active_for_in_key(func, name) {
                let node = self.scalar_node_for(func, name);
                self.add_string_seed(node);
            }
        }
    }

    /// Spec 5 Task 2: like `seed_for_in_key_string_use`, but recognizes a
    /// `for..in` key by its PERSISTENT (grow-only) provenance rather than the
    /// lexical active-key stack. The fasta `fastaRandom` shape stores the key
    /// into an array element AFTER the `for..in` exits via `break`
    /// (`for (c in t) { ... break; } line[i] = c;`), so the store site is
    /// OUTSIDE the loop body where `is_active_for_in_key` is already false.
    /// The store is nonetheless a string-materialization sink: lifting the
    /// key's scalar to `String` makes both the resolve gate's materialized-key
    /// carve-out (`identifier_repr_is_string`) and codegen's grow-only
    /// `for_in_key_handle_tables` materialization (gated on the same String
    /// repr, keyed off the persistent ordinal local) fire on the post-loop
    /// read. Used ONLY at the array-element store sink (the resolve gate keeps
    /// every other post-loop key value use fail-closed).
    fn seed_persisted_for_in_key_string_use(&mut self, func: &str, expr: &Expression) {
        if let Expression::Identifier(name) = expr {
            if self
                .for_in_key_names
                .contains(&(func.to_string(), name.clone()))
            {
                let node = self.scalar_node_for(func, name);
                self.add_string_seed(node);
            }
        }
    }

    // ---- node accessors (get-or-create) --------------------------------

    fn scalar_node_for(&mut self, func: &str, name: &str) -> usize {
        let key = (func.to_string(), name.to_string());
        if let Some(&n) = self.scalar_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.scalar_node.insert(key, n);
        n
    }

    fn array_elem_node_for(&mut self, func: &str, name: &str) -> usize {
        let key = (func.to_string(), name.to_string());
        if let Some(&n) = self.array_elem_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.array_elem_node.insert(key, n);
        n
    }

    fn return_node_for(&mut self, func: &str) -> usize {
        if let Some(&n) = self.return_node.get(func) {
            return n;
        }
        let n = self.new_node();
        self.return_node.insert(func.to_string(), n);
        n
    }

    fn obj_field_node_for(&mut self, slot: &ObjSlot, field: &str) -> usize {
        let key = (slot.clone(), field.to_string());
        if let Some(&n) = self.obj_field_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.obj_field_node.insert(key, n);
        n
    }

    /// Record an object literal initializing `slot`: remember its ordered
    /// field names, visit each value, and wire `value -> field storage`
    /// float edges. Unsupported property forms (non-identifier key,
    /// getter/setter, nested object) record a *deferred* structural conflict
    /// keyed by `slot` and return WITHOUT a field list — the slot then never
    /// materializes on its own, so a read-only fold-lane literal keeps today's
    /// behavior. The deferred message is promoted to a real gate conflict only
    /// if the slot is later forced onto the object lane (`resolve_objects`).
    fn record_object_literal(
        &mut self,
        func: &str,
        slot: ObjSlot,
        obj: &kali_ast::ObjectExpression,
    ) {
        let mut names = Vec::new();
        for prop in &obj.properties {
            let kali_ast::PropertyName::Identifier(key) = &prop.key else {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!(
                        "object literal for {slot:?} uses a non-identifier property name, which is unavailable in the current phase"
                    ),
                );
                return;
            };
            if !matches!(prop.kind, kali_ast::ObjectPropertyKind::Init) {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!(
                        "object literal for {slot:?} uses a getter/setter, which is unavailable in the current phase"
                    ),
                );
                return;
            }
            if matches!(prop.value, Expression::ObjectExpression(_)) {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!("nested object field '{key}' is unavailable in the current phase"),
                );
                return;
            }
            let value_node = self.visit_expr(func, &prop.value);
            let field_node = self.obj_field_node_for(&slot, key);
            self.add_edge(value_node, field_node);
            names.push(key.clone());
        }
        match self.obj_literal_fields.entry(slot.clone()) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                if *existing.get() != names {
                    self.obj_conflicts
                        .push(format!("conflicting object shapes assigned to {slot:?}"));
                }
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(names);
            }
        }
    }

    /// Record an aliasing flow `dst ~ <expr>` when the expression can carry an
    /// object reference: identifier, `arr[i]`, or bare-identifier call.
    fn record_object_flow_from_expr(&mut self, func: &str, dst: ObjSlot, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => self
                .obj_flows
                .push((dst, ObjSlot::Binding(func.to_string(), name.clone()))),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Expression::Identifier(array) = &member.object {
                    self.obj_flows
                        .push((dst, ObjSlot::ArrayElem(func.to_string(), array.clone())));
                }
            }
            Expression::CallExpression(call) => {
                if let Expression::Identifier(callee) = &call.callee {
                    self.obj_flows.push((dst, ObjSlot::Return(callee.clone())));
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.record_object_flow_from_expr(func, dst, &inner.expression)
            }
            _ => {}
        }
    }

    /// Object slot aliased by a call argument, when the expression can carry
    /// an object reference (same recognized set as `record_object_flow_from_expr`).
    fn arg_obj_slot(&mut self, func: &str, arg: &Expression) -> Option<ObjSlot> {
        match arg {
            Expression::Identifier(name) => Some(ObjSlot::Binding(func.to_string(), name.clone())),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                match &member.object {
                    Expression::Identifier(array) => {
                        Some(ObjSlot::ArrayElem(func.to_string(), array.clone()))
                    }
                    _ => None,
                }
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => Some(ObjSlot::Return(callee.clone())),
                _ => None,
            },
            Expression::ParenthesizedExpression(inner) => {
                self.arg_obj_slot(func, &inner.expression)
            }
            _ => None,
        }
    }

    /// Slot for a member-access base: a bare identifier (binding) or a
    /// subscript of a bare identifier (array element). Registers the array's
    /// element node in the subscript case (the base is an array) and visits
    /// the index for its own edges.
    fn member_base_slot(&mut self, func: &str, base: &Expression) -> Option<ObjSlot> {
        match base {
            Expression::Identifier(name) => Some(ObjSlot::Binding(func.to_string(), name.clone())),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Some(index) = &member.computed_index {
                    self.visit_expr(func, index);
                }
                match &member.object {
                    Expression::Identifier(array) => {
                        self.array_elem_node_for(func, array);
                        Some(ObjSlot::ArrayElem(func.to_string(), array.clone()))
                    }
                    _ => None,
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.member_base_slot(func, &inner.expression)
            }
            _ => None,
        }
    }

    // ---- Phase A: signature collection ---------------------------------

    fn collect_functions(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.collect_functions_in_stmt(stmt);
        }
    }

    fn collect_functions_in_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                self.functions
                    .insert(func.name.clone(), func.params.clone());
                for param in &func.params {
                    // Eagerly allocate the param's scalar node.
                    self.scalar_node_for(&func.name, param);
                }
                self.collect_functions(&func.body.body);
            }
            Statement::BlockStatement(block) => self.collect_functions(&block.body),
            Statement::IfStatement(node) => {
                self.collect_functions(&node.consequent.body);
                if let Some(alt) = &node.alternate {
                    self.collect_functions(&alt.body);
                }
            }
            Statement::ForStatement(node) => self.collect_functions(&node.body.body),
            Statement::ForInStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::ForOfStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::WhileStatement(node) => self.collect_functions(&node.body.body),
            Statement::DoWhileStatement(node) => self.collect_functions(&node.body.body),
            Statement::LabeledStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::TryStatement(node) => {
                self.collect_functions(&node.block.body);
                if let Some(handler) = &node.handler {
                    self.collect_functions(&handler.body.body);
                }
                if let Some(finalizer) = &node.finalizer {
                    self.collect_functions(&finalizer.body);
                }
            }
            _ => {}
        }
    }

    // ---- Phase A2: local-name collection --------------------------------

    /// Populate `local_names` for `func`'s own scope (module scope when
    /// `func == TOP_LEVEL`): every `let`/`const`/`var` declarator reachable
    /// from `statements` without descending into a nested function, plus
    /// (via the `FunctionDeclaration` arm below) each nested function's own
    /// parameters.
    fn collect_local_names(&mut self, func: &str, statements: &[Statement]) {
        for stmt in statements {
            self.collect_local_names_in_stmt(func, stmt);
        }
    }

    fn collect_local_names_in_stmt(&mut self, func: &str, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(decl) => {
                let entry = self.local_names.entry(decl.name.clone()).or_default();
                for param in &decl.params {
                    entry.insert(param.clone());
                }
                self.collect_local_names(&decl.name, &decl.body.body);
            }
            Statement::VariableDeclaration(decl) => {
                let entry = self.local_names.entry(func.to_string()).or_default();
                for d in &decl.declarations {
                    entry.insert(d.id.clone());
                }
            }
            Statement::BlockStatement(block) => self.collect_local_names(func, &block.body),
            Statement::IfStatement(node) => {
                self.collect_local_names(func, &node.consequent.body);
                if let Some(alt) = &node.alternate {
                    self.collect_local_names(func, &alt.body);
                }
            }
            Statement::ForStatement(node) => {
                if let Some(ForInit::VariableDeclaration(decl)) = &node.init {
                    let entry = self.local_names.entry(func.to_string()).or_default();
                    for d in &decl.declarations {
                        entry.insert(d.id.clone());
                    }
                }
                self.collect_local_names(func, &node.body.body);
            }
            Statement::ForInStatement(node) => {
                if let ForInLefthand::VariableDeclaration(decl) = &node.left {
                    let entry = self.local_names.entry(func.to_string()).or_default();
                    for d in &decl.declarations {
                        entry.insert(d.id.clone());
                    }
                }
                self.collect_local_names_in_stmt(func, &node.body);
            }
            Statement::ForOfStatement(node) => {
                if let ForOfLefthand::VariableDeclaration(decl) = &node.left {
                    let entry = self.local_names.entry(func.to_string()).or_default();
                    for d in &decl.declarations {
                        entry.insert(d.id.clone());
                    }
                }
                self.collect_local_names_in_stmt(func, &node.body);
            }
            Statement::WhileStatement(node) => self.collect_local_names(func, &node.body.body),
            Statement::DoWhileStatement(node) => self.collect_local_names(func, &node.body.body),
            Statement::LabeledStatement(node) => self.collect_local_names_in_stmt(func, &node.body),
            Statement::TryStatement(node) => {
                self.collect_local_names(func, &node.block.body);
                if let Some(handler) = &node.handler {
                    let entry = self.local_names.entry(func.to_string()).or_default();
                    entry.insert(handler.param.clone());
                    self.collect_local_names(func, &handler.body.body);
                }
                if let Some(finalizer) = &node.finalizer {
                    self.collect_local_names(func, &finalizer.body);
                }
            }
            _ => {}
        }
    }

    /// True when `name` is locally bound in `func`'s own scope (a parameter
    /// or a `let`/`const`/`var` declarator anywhere in its body, ignoring
    /// nested functions) — mirrors codegen's local/binding precedence.
    fn is_locally_declared(&self, func: &str, name: &str) -> bool {
        self.local_names
            .get(func)
            .is_some_and(|names| names.contains(name))
    }

    // ---- Phase B: statement walk ---------------------------------------

    fn visit_block(&mut self, func: &str, block: &BlockStatement) {
        for stmt in &block.body {
            self.visit_stmt(func, stmt);
        }
    }

    fn visit_stmt(&mut self, func: &str, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(decl) => {
                // Walk the body under the function's own name.
                self.visit_block(&decl.name, &decl.body);
            }
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let Some(init) = &d.init {
                        self.visit_declarator_init(func, &d.id, init);
                    }
                }
            }
            Statement::ExpressionStatement(stmt) => {
                // A bare, top-level member-read statement (`p.field;`) never
                // observes its result — the value is unconditionally
                // discarded. This is also, incidentally, the exact AST shape
                // the parser currently produces for `delete p.field;` (the
                // parser silently drops the `delete` keyword rather than
                // building a `UnaryExpression` for it) — routing it through
                // the same "unobserved" helper fixes that case too, without
                // requiring dedicated `delete` parser support. See
                // `visit_unobserved_member_target`.
                if matches!(stmt.expression.as_ref(), Expression::MemberExpression(_)) {
                    self.visit_unobserved_member_target(func, &stmt.expression);
                    return;
                }
                self.visit_expr(func, &stmt.expression);
            }
            Statement::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    if let Expression::ObjectExpression(obj) = arg {
                        self.record_object_literal(func, ObjSlot::Return(func.to_string()), obj);
                    } else {
                        // `return <identifier>;` — record for the I2
                        // String-element-array-return fail-closed check.
                        if let Expression::Identifier(name) = arg {
                            self.array_binding_returns
                                .push((func.to_string(), name.clone()));
                        }
                        self.record_object_flow_from_expr(
                            func,
                            ObjSlot::Return(func.to_string()),
                            arg,
                        );
                        // Spec 4a Task 5: `return c` where `c` is an active
                        // for-in key lifts the key (hence the return) to String.
                        self.seed_for_in_key_string_use(func, arg);
                        let rn = self.visit_expr(func, arg);
                        let ret = self.return_node_for(func);
                        self.add_edge(rn, ret);
                    }
                }
            }
            Statement::IfStatement(stmt) => {
                self.visit_expr(func, &stmt.test);
                self.visit_block(func, &stmt.consequent);
                if let Some(alt) = &stmt.alternate {
                    self.visit_block(func, alt);
                }
            }
            Statement::ForStatement(stmt) => {
                if let Some(init) = &stmt.init {
                    match init {
                        ForInit::VariableDeclaration(decl) => {
                            for d in &decl.declarations {
                                if let Some(i) = &d.init {
                                    self.visit_declarator_init(func, &d.id, i);
                                }
                            }
                        }
                        ForInit::Expression(expr) => {
                            self.visit_expr(func, expr);
                        }
                    }
                }
                if let Some(test) = &stmt.test {
                    self.visit_expr(func, test);
                }
                if let Some(update) = &stmt.update {
                    self.visit_expr(func, update);
                }
                self.visit_block(func, &stmt.body);
            }
            Statement::ForInStatement(stmt) => {
                // `for (key in obj)` observes EVERY field of `obj` at
                // runtime (Spec 4a Task 1's counted-loop lowering reads the
                // object's shape field count) even though it never emits a
                // `.field` `ObjAccess` of its own — so a base that is
                // otherwise only ever read via a fold-lane compile-time
                // literal (never field-accessed, never aliased) must still
                // be forced onto the materialized runtime-object lane here,
                // or `kali_codegen`'s `object_shape_of_node` will find no
                // `Repr::Object` entry for it and fail closed (a "no known
                // shape" diagnostic) even for a perfectly fixed-shape object
                // literal. Mirrors `visit_assignment`'s whole-object
                // reassignment branch, which materializes the same way for
                // the same reason (observable outside the fold lane).
                let base_slot = self.member_base_slot(func, &stmt.right);
                if let Some(base) = &base_slot {
                    self.obj_materialized.insert(base.clone());
                }
                self.visit_expr(func, &stmt.right);
                // Seed a scalar node for the key binding so it has a stable
                // identity in the repr graph. Deliberately left on the
                // default (I64) axis — no float/string seed — so the ORDINAL
                // itself stays i64 (Spec 4a Task 2); the value read THROUGH it
                // (`base[key]`) is what carries the field repr (Task 3).
                let mut pushed_key = false;
                if let ForInLefthand::VariableDeclaration(decl) = &stmt.left {
                    for d in &decl.declarations {
                        let _ = self.scalar_node_for(func, &d.id);
                    }
                }
                // Record the key → base provenance for the loop body so
                // `base[key]` resolves to a uniform-object field read (Task 3)
                // and a string USE of the key materializes (Task 5). Supports
                // BOTH the single-declarator declaration form (`for (var c in
                // obj)`) and — Task 5 R1 — the bare-identifier Expression form
                // (`for (c in obj)`, the shape the capstone's `selectRandom`
                // uses). Destructuring / multi-declarator remain deferred.
                let key_name = match &stmt.left {
                    ForInLefthand::VariableDeclaration(decl) if decl.declarations.len() == 1 => {
                        Some(decl.declarations[0].id.clone())
                    }
                    ForInLefthand::Expression(Expression::Identifier(name)) => Some(name.clone()),
                    _ => None,
                };
                if let (Some(key), Some(base)) = (key_name, &base_slot) {
                    let _ = self.scalar_node_for(func, &key);
                    // Grow-only record for the persistent string-sink provenance
                    // (never popped), separate from the lexical stack below.
                    self.for_in_key_names
                        .insert((func.to_string(), key.clone()));
                    self.for_in_key_bases
                        .push((func.to_string(), key, base.clone()));
                    pushed_key = true;
                }
                self.visit_stmt(func, &stmt.body);
                if pushed_key {
                    self.for_in_key_bases.pop();
                }
            }
            Statement::ForOfStatement(stmt) => {
                self.visit_expr(func, &stmt.right);
                self.visit_stmt(func, &stmt.body);
            }
            Statement::WhileStatement(stmt) => {
                self.visit_expr(func, &stmt.test);
                self.visit_block(func, &stmt.body);
            }
            Statement::DoWhileStatement(stmt) => {
                self.visit_block(func, &stmt.body);
                self.visit_expr(func, &stmt.test);
            }
            Statement::BlockStatement(block) => self.visit_block(func, block),
            Statement::LabeledStatement(stmt) => self.visit_stmt(func, &stmt.body),
            Statement::SwitchStatement(stmt) => {
                self.visit_expr(func, &stmt.discriminant);
                for case in &stmt.cases {
                    if let Some(test) = &case.test {
                        self.visit_expr(func, test);
                    }
                    for s in &case.consequent {
                        self.visit_stmt(func, s);
                    }
                }
            }
            Statement::TryStatement(stmt) => {
                self.visit_block(func, &stmt.block);
                if let Some(handler) = &stmt.handler {
                    self.visit_block(func, &handler.body);
                }
                if let Some(finalizer) = &stmt.finalizer {
                    self.visit_block(func, finalizer);
                }
            }
            Statement::ThrowStatement(stmt) => {
                self.visit_expr(func, &stmt.argument);
            }
            _ => {}
        }
    }

    /// `let/const/var id = init` — array-producing inits create an element
    /// node for `id`; everything else flows the init into `id`'s scalar node
    /// (`init -> id`).
    fn visit_declarator_init(&mut self, func: &str, id: &str, init: &Expression) {
        if let Expression::ObjectExpression(obj) = init {
            // Syntactic taint, independent of materialization — see the field
            // doc on `object_initialized_bindings`. A compound/update on `id`
            // must reject even when the object literal is never field-read and
            // so never gets promoted to `Repr::Object` below.
            self.object_initialized_bindings
                .insert((func.to_string(), id.to_string()));
            self.record_object_literal(
                func,
                ObjSlot::Binding(func.to_string(), id.to_string()),
                obj,
            );
            return;
        }
        self.record_object_flow_from_expr(
            func,
            ObjSlot::Binding(func.to_string(), id.to_string()),
            init,
        );
        if self.init_is_array(init) {
            self.note_array_init(func, id, init);
            return;
        }
        let rn = self.visit_expr(func, init);
        let sn = self.scalar_node_for(func, id);
        // init -> id.
        self.add_edge(rn, sn);
    }

    /// True when `expr` produces a fresh array binding (`new Array(...)` or an
    /// array literal).
    fn init_is_array(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ArrayExpression(_) => true,
            Expression::NewExpression(new_expr) => {
                constructor_name(&new_expr.callee).as_deref() == Some("Array")
            }
            _ => false,
        }
    }

    /// Wire an array-producing init (`new Array(...)` or an array literal)
    /// into `name`'s element node. Shared by a declarator init (`let a = ...`)
    /// and a plain-identifier reassignment (`a = ...`), so both union into the
    /// SAME element node instead of the reassignment silently dropping the
    /// array-ness (extracted verbatim from the former declarator-only body —
    /// the declarator path's behavior is unchanged).
    fn note_array_init(&mut self, func: &str, name: &str, init: &Expression) {
        let elem = self.array_elem_node_for(func, name);
        // Array-literal elements flow (store direction) into the element.
        if let Expression::ArrayExpression(arr) = init {
            for element in arr.elements.iter().flatten() {
                if let kali_ast::ExpressionOrSpread::Expression(expr) = element {
                    if let Expression::ObjectExpression(obj) = expr {
                        self.record_object_literal(
                            func,
                            ObjSlot::ArrayElem(func.to_string(), name.to_string()),
                            obj,
                        );
                        continue;
                    }
                    self.record_object_flow_from_expr(
                        func,
                        ObjSlot::ArrayElem(func.to_string(), name.to_string()),
                        expr,
                    );
                    let en = self.visit_expr(func, expr);
                    self.add_edge(en, elem);
                    self.element_store_sources.push((elem, en));
                }
            }
        }
    }

    // ---- Phase B: expression walk --------------------------------------

    /// Walk `expr`, wiring seeds/edges, and return its result node.
    fn visit_expr(&mut self, func: &str, expr: &Expression) -> usize {
        match expr {
            Expression::Identifier(name) => {
                // A read of a name not locally bound in `func`'s own scope
                // but declared at module scope is a module-const/binding
                // read (see `kali_codegen`'s matching identifier fallback):
                // route it to the SAME node as the module declaration so its
                // float-ness (e.g. `const DPY = 365.24;`) reaches every
                // reader, instead of a fresh, permanently-unseeded node
                // private to this function.
                let scope = if func != TOP_LEVEL
                    && !self.is_locally_declared(func, name)
                    && self.is_locally_declared(TOP_LEVEL, name)
                {
                    TOP_LEVEL
                } else {
                    func
                };
                self.scalar_node_for(scope, name)
            }

            Expression::Literal(LiteralValue::Number(n)) => {
                let node = self.new_node();
                if is_float_literal(*n) {
                    self.add_seed(node);
                }
                node
            }
            Expression::Literal(LiteralValue::String(value)) => {
                let node = self.new_node();
                self.add_string_seed(node);
                if !value.is_ascii() {
                    self.non_ascii_seeds.push(node);
                }
                node
            }
            Expression::Literal(_) => self.new_node(),
            // NOTE: real source never reaches this arm — kali_parser desugars
            // interpolated backtick templates into `+` chains of string
            // Literals and the parsed interpolands (desugar_template_literal,
            // kali_parser/src/expression/primary.rs) BEFORE repr_infer runs,
            // and non-interpolated backticks parse as plain string Literals.
            // Only hand-built ASTs construct `TemplateLiteral`; the arm stays
            // fail-closed in case a future pipeline ever routes a raw
            // interpolated `TemplateLiteral` here.
            Expression::TemplateLiteral(template) => {
                // Visit interpolated expressions for their own edges.
                for expr in &template.expressions {
                    self.visit_expr(func, expr);
                }
                let node = self.new_node();
                self.add_string_seed(node);
                if !template.expressions.is_empty() {
                    // An interpolated template lowers to runtime concatenation
                    // (a fresh handle), exactly like a string `+`.
                    self.runtime_string_nodes.push(node);
                    // The interpolations' value flow is not wired into `node`
                    // on this arm, so the template's contents cannot be proven
                    // ASCII here. Fail closed.
                    self.non_ascii_seeds.push(node);
                } else if template.quasis.iter().any(|quasi| !quasi.value.is_ascii()) {
                    self.non_ascii_seeds.push(node);
                }
                node
            }

            Expression::ParenthesizedExpression(inner) => self.visit_expr(func, &inner.expression),

            Expression::BinaryExpression(bin) => {
                // Spec 4a Task 5: a for-in key as a `+` or equality operand is
                // used as a string (`c + x`, `c == "a"`), so lift it to String.
                if matches!(bin.operator.as_str(), "+" | "==" | "!=" | "===" | "!==") {
                    self.seed_for_in_key_string_use(func, &bin.left);
                    self.seed_for_in_key_string_use(func, &bin.right);
                }
                let left = self.visit_expr(func, &bin.left);
                let right = self.visit_expr(func, &bin.right);
                let result = self.new_node();
                match bin.operator.as_str() {
                    "/" => {
                        // Division always yields a float: SEED the result. Its
                        // operands are NOT floated (edges run operand ->
                        // result only), so integer operands stay integer.
                        self.add_seed(result);
                        self.add_edge(left, result);
                        self.add_edge(right, result);
                    }
                    "+" | "-" | "*" | "%" | "**" => {
                        // Forward-flow: a float operand floats the result; the
                        // result does NOT flow back to the operands.
                        self.add_edge(left, result);
                        self.add_edge(right, result);
                        if bin.operator == "+" {
                            // A string `+` lowers to runtime `string_concat`
                            // producing a FRESH handle (not an interned literal
                            // constant). Record it as a runtime-string node so
                            // the taint pass can forbid identity-comparing /
                            // truthiness-testing its downstream captors.
                            self.runtime_string_nodes.push(result);
                        }
                    }
                    // Comparisons / bitwise / shift ops yield i64 (boolean or
                    // int32); operands are visited for their own edges but no
                    // edge runs into the result.
                    _ => {}
                }
                result
            }

            Expression::UnaryExpression(unary) => {
                if unary.operator == "delete" {
                    // `delete <base>.field` / `delete <base>[i]` never OBSERVES
                    // the deleted slot's value (codegen lowers `delete` through
                    // its own dedicated path, independent of this object axis).
                    // Visiting it like an ordinary member read would wrongly
                    // register a deferred field-*read* access, which the
                    // pending-conflict promotion below treats as evidence that
                    // the slot's value is actually consumed — miscounting a
                    // `delete` as a read would over-promote a structural
                    // literal that is only ever deleted-from and reinserted
                    // into (never truly read through this axis), rejecting a
                    // program that today runs correctly. Still visit the base
                    // (and computed index) for their own housekeeping, just
                    // without creating the ObjAccess.
                    self.visit_unobserved_member_target(func, &unary.argument);
                    return self.new_node();
                }
                let arg = self.visit_expr(func, &unary.argument);
                let result = self.new_node();
                if unary.operator == "-" {
                    self.add_edge(arg, result);
                } else if unary.operator == "+" {
                    // fasta Spec 5 Task 6: unary `+` over a runtime-string
                    // operand (`+process.argv[i]`, or any other proven
                    // string) takes codegen's inline decimal-parse coercion
                    // (`emit_string_to_i64_parse`) and ALWAYS yields a
                    // NUMERIC value — mirroring JS `Number(x)`/`Math.trunc`
                    // semantics, never a string. A FLOAT-only edge (not the
                    // full `add_edge`) lets a float-seeded operand still
                    // float the result (`+x` where `x: f64` stays float),
                    // but — same discipline as the array-element/object-
                    // field `add_edge_float_only` uses — does NOT carry the
                    // STRING axis into `result`. Using the full `add_edge`
                    // here would let a genuine string-typed operand (e.g.
                    // `var s = "5"; var n = +s;`) incorrectly solve
                    // `n: Repr::String`: codegen's `is_string_valued` would
                    // then treat every LATER read of `n` as a live string
                    // handle (per the stale `ReprTable` entry) even though
                    // `n`'s WASM local actually holds the freshly-parsed
                    // integer written by the coercion — a real miscompile,
                    // not a diagnostic. Excluding the string axis here means
                    // `result` (and any scalar it flows into, like `n`) can
                    // only ever solve `F64` or the `I64` default — never
                    // `String` — regardless of how string-valued the operand
                    // is, which is exactly the "coerced to numeric" contract
                    // codegen's `"+"` arm now implements.
                    self.add_edge_float_only(arg, result);
                }
                result
            }

            Expression::UpdateExpression(update) => {
                // `x++` / `--x`: numeric identity on the operand.
                self.visit_expr(func, &update.argument)
            }

            Expression::AssignmentExpression(assign) => self.visit_assignment(func, assign),

            Expression::ConditionalExpression(cond) => {
                self.visit_expr(func, &cond.test);
                let cons = self.visit_expr(func, &cond.consequent);
                let alt = self.visit_expr(func, &cond.alternate);
                let result = self.new_node();
                // Either branch floats the result.
                self.add_edge(cons, result);
                self.add_edge(alt, result);
                // Value-selecting merge: a string branch mixed with a plain
                // int branch is a shape conflict (see `merge_nodes`).
                self.merge_nodes.push((result, vec![cons, alt]));
                result
            }

            Expression::LogicalExpression(logical) => {
                let left = self.visit_expr(func, &logical.left);
                let right = self.visit_expr(func, &logical.right);
                let result = self.new_node();
                self.add_edge(left, result);
                self.add_edge(right, result);
                // Value-selecting merge: a string operand mixed with a plain
                // int operand is a shape conflict (see `merge_nodes`).
                self.merge_nodes.push((result, vec![left, right]));
                result
            }

            Expression::SequenceExpression(seq) => {
                let mut last = self.new_node();
                for e in &seq.expressions {
                    last = self.visit_expr(func, e);
                }
                last
            }

            Expression::MemberExpression(member) => self.visit_member(func, member),

            Expression::CallExpression(call) => self.visit_call(func, call),

            Expression::NewExpression(new_expr) => {
                // Constructor arguments are visited for edges (e.g. `new Array`
                // length is an int). The handle itself is i64.
                for arg in &new_expr.args {
                    self.visit_expr(func, arg);
                }
                self.new_node()
            }

            // Any other expression kind is a fresh (int) node.
            _ => self.new_node(),
        }
    }

    fn visit_assignment(&mut self, func: &str, assign: &kali_ast::AssignmentExpression) -> usize {
        // Whole-object (re)assignment through a plain identifier target.
        if let Expression::Identifier(name) = &assign.left {
            if matches!(assign.operator, AssignmentOperator::Assign) {
                if let Expression::ObjectExpression(obj) = &assign.right {
                    let slot = ObjSlot::Binding(func.to_string(), name.clone());
                    self.record_object_literal(func, slot.clone(), obj);
                    // A reassigned literal is observable through the binding:
                    // the fold lane cannot represent it, so materialize.
                    self.obj_materialized.insert(slot);
                    return self.scalar_node_for(func, name);
                }
                self.record_object_flow_from_expr(
                    func,
                    ObjSlot::Binding(func.to_string(), name.clone()),
                    &assign.right,
                );
            }
        }

        // Array element store: `a[i] = v`.
        if let Expression::MemberExpression(member) = &assign.left {
            if let Some(index) = &member.computed_index {
                self.visit_expr(func, index); // index stays i64 (untouched).
                let rn = self.visit_expr(func, &assign.right);
                if let Expression::Identifier(name) = &member.object {
                    let elem = self.array_elem_node_for(func, name);
                    // Store is directed: value -> element (a float value floats
                    // the array; an int value into a float array stays int).
                    self.add_edge(rn, elem);
                    self.element_store_sources.push((elem, rn));
                    // Spec 5: a `for..in` key stored into an array element is a
                    // string-materialization sink, exactly like `return c` /
                    // `console.log(c)` / `+`/`==`. Seed the key's scalar node
                    // String so (a) the element axis lifts to String via the
                    // edge above (enabling `.join('')` and the
                    // `string_element_array_binding` gate) and (b) the resolve
                    // gate's materialized-key carve-out
                    // (`identifier_repr_is_string`) admits the `c` read after
                    // the loop. Uses the PERSISTENT (grow-only) provenance,
                    // not the lexical active-key stack, because the fasta
                    // shape stores the key AFTER the `for..in` exits via
                    // `break` (outside the loop body). No-ops unless the RHS
                    // was seen as a `for..in` key somewhere in `func`.
                    self.seed_persisted_for_in_key_string_use(func, &assign.right);
                } else {
                    self.visit_expr(func, &member.object);
                }
                return rn;
            }
            // Non-computed member store: `<base>.field = v` — deferred object
            // field access, wired after object propagation.
            let rn = self.visit_expr(func, &assign.right);
            if let Some(base) = self.member_base_slot(func, &member.object) {
                self.obj_accesses.push(ObjAccess {
                    base,
                    field: member.property.clone(),
                    other: rn,
                    is_write: true,
                });
            } else {
                self.visit_expr(func, &member.object);
            }
            return rn;
        }

        // Scalar assignment: `x = v`, `x += v`, `x /= v`, ...
        let rn = self.visit_expr(func, &assign.right);
        if let Expression::Identifier(name) = &assign.left {
            let sn = self.scalar_node_for(func, name);
            match assign.operator {
                AssignmentOperator::Assign
                | AssignmentOperator::AddAssign
                | AssignmentOperator::SubtractAssign
                | AssignmentOperator::MultiplyAssign
                | AssignmentOperator::ModuloAssign
                | AssignmentOperator::ExponentAssign => {
                    // rhs -> x.
                    self.add_edge(rn, sn);
                    if matches!(assign.operator, AssignmentOperator::AddAssign) {
                        // A string `+=` lowers to runtime `string_concat` into
                        // the target (a fresh handle). Record the target as a
                        // runtime-string node (seeded only if string-reachable).
                        self.runtime_string_nodes.push(sn);
                    }
                }
                AssignmentOperator::DivideAssign => {
                    // `x /= v` ⇒ x is float; the divisor keeps its own repr, so
                    // no `rhs -> x` edge.
                    self.add_seed(sn);
                }
                // Bitwise/logical compound assigns keep i64.
                _ => {}
            }
            if matches!(assign.operator, AssignmentOperator::Assign) {
                // `a = new Array(n)` / `a = [..]`: route the RHS through the
                // same element-node path as a declarator init, so
                // reassignment unions the element axes instead of silently
                // dropping the array-ness.
                if self.init_is_array(&assign.right) {
                    self.note_array_init(func, name, &assign.right);
                } else if let Expression::Identifier(rhs) = &assign.right {
                    // `a = b` between arrays: elements of b flow into elements
                    // of a.
                    if self.binding_has_element_node(func, rhs) {
                        let src = self.array_elem_node_for(func, rhs);
                        let dst = self.array_elem_node_for(func, name);
                        self.add_edge(src, dst);
                        self.element_store_sources.push((dst, src));
                    }
                }
            }
            return sn;
        }
        rn
    }

    /// Non-inserting lookup twin of [`Self::array_elem_node_for`]: true when
    /// `(func, name)` already has an element node, without allocating one.
    fn binding_has_element_node(&self, func: &str, name: &str) -> bool {
        self.array_elem_node
            .contains_key(&(func.to_string(), name.to_string()))
    }

    fn visit_member(&mut self, func: &str, member: &kali_ast::MemberExpression) -> usize {
        // Computed access `a[i]` → array element read.
        if let Some(index) = &member.computed_index {
            self.visit_expr(func, index); // index untouched (i64).
                                          // `process.argv[<int>]` (Spec 5 Task 5): a runtime string handle
                                          // (`args_get`), NOT an array element read. Its base is the
                                          // `process.argv` member (never a bare array binding), so this must
                                          // precede the Identifier-base element arm below. Register the read
                                          // result as a runtime-string node so a consuming binding
                                          // (`const s = process.argv[i]`) solves `Repr::String`, mirroring
                                          // the substring/join result registration.
            if member_is_process_argv_element(member) {
                self.visit_expr(func, &member.object);
                let result = self.new_node();
                self.runtime_string_nodes.push(result);
                return result;
            }
            // Task 3: a computed read `base[key]` where `key` is the active
            // `for..in` key over `base` is a uniform-object FIELD read, not an
            // array element read. Its result carries the shape's (uniform)
            // field repr — wired against field storage in `resolve_objects`.
            if let (Expression::Identifier(base_name), Expression::Identifier(key_name)) =
                (&member.object, index.as_ref())
            {
                let base = ObjSlot::Binding(func.to_string(), base_name.clone());
                if self
                    .for_in_key_bases
                    .iter()
                    .any(|(f, k, b)| f == func && k == key_name && *b == base)
                {
                    let result = self.new_node();
                    self.obj_materialized.insert(base.clone());
                    self.uniform_computed_reads.push((base, result));
                    return result;
                }
            }
            if let Expression::Identifier(name) = &member.object {
                let elem = self.array_elem_node_for(func, name);
                let result = self.new_node();
                // Read is directed: element -> read result. Element reads now
                // carry the STRING axis too (Spec 3 lifts Spec 1's float-only
                // exclusion): element STORES are gated (Spec 2's F1, re-keyed
                // in Spec 3), and a mixed string/number array fails closed at
                // emit_table (`element_store_sources`) — so a string can no
                // longer launder through an element read unseen the way it
                // could when only reachability was consulted. Object-FIELD
                // reads (`resolve_objects`) remain float-only and gated;
                // fields are a separate, still-excluded axis.
                self.add_edge(elem, result);
                return result;
            }
            self.visit_expr(func, &member.object);
            return self.new_node();
        }

        // `.length` on a bare identifier is an array-length access: register the
        // receiver as an array binding so its i64 handle is treated as a
        // linear-memory array (matching codegen's `.length` header-load path),
        // even when the identifier is never subscripted directly. This mirrors
        // the subscript/`.fill`/`new Array` seeds and lets interprocedural
        // propagation link pass-through callers. Other dot access carries no
        // array signal. Arrays that are also subscripted are already seeded, so
        // integer programs (whose arrays are always indexed) are unaffected.
        if member.property.as_str() == "length" {
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
            }
        }

        // Non-computed member read `<base>.field` — deferred object access.
        if let Some(base) = self.member_base_slot(func, &member.object) {
            let result = self.new_node();
            self.obj_accesses.push(ObjAccess {
                base,
                field: member.property.as_str().to_string(),
                other: result,
                is_write: false,
            });
            return result;
        }

        // `.length` and other dot access → i64 result.
        self.visit_expr(func, &member.object);
        self.new_node()
    }

    /// Visit `expr` WITHOUT recording a deferred object-field READ access,
    /// for positions whose member-access result is provably never observed:
    /// the target of `delete <expr>` (which never observes the deleted
    /// slot's value — codegen lowers `delete` through its own dedicated
    /// path), and a bare top-level `<expr>;` expression-statement (whose
    /// value is unconditionally discarded — see the `ExpressionStatement`
    /// arm of `visit_stmt`, which is *also* the exact AST shape the parser
    /// currently produces for `delete <expr>;`, since it drops the `delete`
    /// keyword rather than building a `UnaryExpression` for it; routing both
    /// spellings through this helper keeps them consistent regardless of
    /// which one the parser happens to use). Mirrors `visit_member`'s
    /// housekeeping (index visiting, array-binding registration, `.length`
    /// array signal) so array/float bookkeeping is unaffected; only the
    /// terminal `ObjAccess` push is skipped. Not observing this read matters
    /// for `resolve_objects`'s pending-conflict promotion: an unobserved
    /// read must NOT count as evidence that a structurally-unsupported
    /// literal's value is consumed (that would over-promote a literal that
    /// is only ever deleted-from/reinserted-into and enumerated via
    /// `Object.keys`-style builtins, rejecting a program that runs correctly
    /// today — see `pending_slots_reached_by_a_read`).
    fn visit_unobserved_member_target(&mut self, func: &str, expr: &Expression) {
        let Expression::MemberExpression(member) = expr else {
            self.visit_expr(func, expr);
            return;
        };
        if let Some(index) = &member.computed_index {
            self.visit_expr(func, index);
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
            } else {
                self.visit_expr(func, &member.object);
            }
            return;
        }
        if member.property.as_str() == "length" {
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
                return;
            }
        }
        // Still visit the base for its own housekeeping (e.g. a nested
        // member/array access reached along the way), but never create a
        // deferred field-read ObjAccess for the terminal `.field`.
        self.visit_expr(func, &member.object);
    }

    fn visit_call(&mut self, func: &str, call: &kali_ast::CallExpression) -> usize {
        match &call.callee {
            // Method call: `obj.method(args)`.
            Expression::MemberExpression(member) if member.computed_index.is_none() => {
                let method = member.property.as_str();
                match method {
                    "sqrt" | "cbrt" if is_math_object(&member.object) => {
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        self.add_seed(result);
                        result
                    }
                    "toFixed" => {
                        // The receiver is a float.
                        let recv = self.visit_expr(func, &member.object);
                        self.add_seed(recv);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        // `.toFixed` returns a string; result is a fresh i64.
                        self.new_node()
                    }
                    "substring" => {
                        let recv = self.visit_expr(func, &member.object);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        // A slice of a string is a string: receiver -> result.
                        self.add_edge(recv, result);
                        // A runtime substring result is a non-interned runtime
                        // string: taint-seed it (like `+` results). Static-
                        // foldable slices never consult the repr, so this
                        // over-approximation costs nothing there.
                        self.runtime_string_nodes.push(result);
                        result
                    }
                    "join" => {
                        // `a.join(sep)` implies `a`'s elements are strings.
                        // String-seed the receiver's element node so an
                        // otherwise-unstored array (e.g. `new Array(0)`) proves
                        // a String element axis and the join gate accepts it,
                        // while an array that ALSO stores a non-string element
                        // (`a[0] = 1`) becomes a mixed-store element conflict
                        // (E5506) — the number-element reject. Seeding the
                        // element node rather than a fresh one is what makes
                        // both facts fall out of the existing element solve
                        // (emit_table: mixed_store || float => conflict).
                        if let Expression::Identifier(name) = &member.object {
                            let elem = self.array_elem_node_for(func, name);
                            self.add_string_seed(elem);
                        } else {
                            self.visit_expr(func, &member.object);
                        }
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        // A bound join result must flow `Repr::String` to its
                        // captor: string-seed it directly. Unlike substring there
                        // is no receiver->result string edge — a join result is a
                        // FRESH runtime buffer, not a slice of the receiver.
                        self.add_string_seed(result);
                        // A join result is a non-interned runtime string (like
                        // `+`): taint it so identity `==` rejects.
                        self.runtime_string_nodes.push(result);
                        // No node-level non-ASCII seed on the RESULT: the Task 7
                        // join GATE (resolve_array_join_member_call) rejects any
                        // runtime join whose element axis is non-ASCII OR whose
                        // separator is not proven ASCII, so a non-ASCII join
                        // never reaches codegen — result-node non-ASCII
                        // propagation would be unreachable.
                        result
                    }
                    "fill" => {
                        // `a.fill(v)` is a store: value -> receiver element.
                        let vnode = call
                            .args
                            .first()
                            .map(|arg| self.visit_expr(func, arg))
                            .unwrap_or_else(|| self.new_node());
                        for arg in call.args.iter().skip(1) {
                            self.visit_expr(func, arg);
                        }
                        if let Expression::Identifier(name) = &member.object {
                            let elem = self.array_elem_node_for(func, name);
                            self.add_edge(vnode, elem);
                            self.element_store_sources.push((elem, vnode));
                        } else {
                            self.visit_expr(func, &member.object);
                        }
                        // `.fill` returns the array handle (i64).
                        self.new_node()
                    }
                    _ => {
                        // Spec 4a Task 5: `console.log(c)` (and siblings) print
                        // the for-in key as a string — lift it to String so
                        // codegen materializes the field-name handle instead of
                        // printing the raw ordinal.
                        if is_console_object(&member.object) {
                            for arg in &call.args {
                                self.seed_for_in_key_string_use(func, arg);
                            }
                        }
                        self.visit_expr(func, &member.object);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        self.new_node()
                    }
                }
            }

            // Bare-identifier call: candidate user-function call. Record an
            // interprocedural edge (resolved after all bodies are walked).
            Expression::Identifier(callee) => {
                let mut arg_nodes = Vec::with_capacity(call.args.len());
                let mut arg_array_names = Vec::with_capacity(call.args.len());
                let mut arg_obj_slots = Vec::with_capacity(call.args.len());
                let mut arg_array_literal = Vec::with_capacity(call.args.len());
                let mut arg_scalar_syntactic = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    if matches!(arg, Expression::ObjectExpression(_)) {
                        self.obj_conflicts.push(
                            "an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first"
                                .to_string(),
                        );
                    }
                    arg_obj_slots.push(self.arg_obj_slot(func, arg));
                    arg_array_literal.push(self.init_is_array(arg));
                    arg_scalar_syntactic.push(Self::expr_is_syntactic_scalar(arg));
                    arg_nodes.push(self.visit_expr(func, arg));
                    arg_array_names.push(match arg {
                        Expression::Identifier(name) => Some((func.to_string(), name.clone())),
                        _ => None,
                    });
                }
                let result_node = self.new_node();
                self.calls.push(CallEdge {
                    callee: callee.clone(),
                    arg_nodes,
                    arg_array_names,
                    arg_obj_slots,
                    arg_array_literal,
                    arg_scalar_syntactic,
                    result_node,
                });
                result_node
            }

            other => {
                self.visit_expr(func, other);
                for arg in &call.args {
                    self.visit_expr(func, arg);
                }
                self.new_node()
            }
        }
    }

    /// True when `expr` SYNTACTICALLY evaluates to a primitive scalar
    /// (number/string/boolean) — never a heap array or object — and so is
    /// positive scalar-inflow evidence for a param that receives it as an
    /// argument. Conservative (fail-closed): only forms whose result is a
    /// primitive by construction return true. A bare identifier / call /
    /// member expression is NOT handled here (a bare identifier is never
    /// treated as scalar evidence — see `resolve_calls` Step 1b — and is
    /// simply left unproven), and `null`, regex, `delete`, array/object
    /// literals, spreads, etc. return false. Note this is a claim about JS
    /// primitive TYPE, not value correctness: e.g. `xs+0` where `xs` is an
    /// array is syntactically a `BinaryExpression` and so returns true here,
    /// even though adding an array to a number is a dubious program; that is
    /// a separate, pre-existing expression-site surface, not this gate's
    /// concern.
    fn expr_is_syntactic_scalar(expr: &Expression) -> bool {
        match expr {
            Expression::Literal(LiteralValue::Number(_))
            | Expression::Literal(LiteralValue::String(_))
            | Expression::Literal(LiteralValue::Boolean(_)) => true,
            // Interpolated/plain template — a string primitive.
            Expression::TemplateLiteral(_) => true,
            // Arithmetic/comparison/bitwise/`+` — a number/boolean/string
            // primitive; never a heap handle.
            Expression::BinaryExpression(_) => true,
            // `i++` / `i--` as an expression — a number primitive.
            Expression::UpdateExpression(_) => true,
            // `-x`, `+x`, `!x`, `typeof x`, `void x` — a number/boolean/string
            // primitive. `delete` (a boolean) is excluded only to keep the set
            // to obviously-numeric coercions; treating it as scalar would be
            // sound too, but it never feeds an arithmetic param.
            Expression::UnaryExpression(unary) if unary.operator != "delete" => true,
            Expression::ParenthesizedExpression(inner) => {
                Self::expr_is_syntactic_scalar(&inner.expression)
            }
            _ => false,
        }
    }

    // ---- Phase C: interprocedural resolution ---------------------------

    fn resolve_calls(&mut self) {
        // Step 1: compute the transitive "is an array param" set to a fixpoint.
        // Seed with every binding directly used as an array (subscript, read,
        // write, `.fill`, `new Array`, array literal — anything that created an
        // `array_elem_node`). Then propagate: a caller's identifier argument
        // that is passed at position `k` to a callee whose param `k` is an
        // array binding is itself an array binding. This links pass-through
        // params (e.g. `w` in `AtAu(u,v,w){ Au(u,w); Atu(w,v); }`) that never
        // subscript their param directly.
        let mut array_bindings: BTreeSet<(String, String)> =
            self.array_elem_node.keys().cloned().collect();
        loop {
            let mut changed = false;
            for edge in &self.calls {
                let Some(params) = self.functions.get(&edge.callee) else {
                    continue;
                };
                for (k, arg) in edge.arg_array_names.iter().enumerate() {
                    let Some((caller, argname)) = arg else {
                        continue;
                    };
                    let Some(param_name) = params.get(k) else {
                        continue;
                    };
                    if array_bindings.contains(&(edge.callee.clone(), param_name.clone()))
                        && array_bindings.insert((caller.clone(), argname.clone()))
                    {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Step 1b: compute the POSITIVE scalar-inflow set. A param is in this
        // set only when (a) SOME call edge supplies it a syntactically-scalar
        // argument, i.e. actual flow evidence its runtime value is a
        // number/string — NOT the default I64 every unconstrained param carries
        // — AND (b) NO call edge supplies it a non-scalar or unproven argument.
        // The param compound/update gate rejects any param NOT proven here, so
        // an array/object that reached the param through an INDIRECT call shape
        // (`f(g())`, a pass-through chain `f(a)->h(a)`, `f(o.a)`) — which the
        // syntactic `non_scalar_params` array taint cannot see — fails closed by
        // construction instead of silently miscompiling. This is the positive-
        // proof allowlist (mirror the Spec 4a lesson: prove safe positions, do
        // not enumerate indirect array sinks).
        //
        // Scalar evidence at position `k`, for callee param `param_name`, is
        // ONLY that the argument is SYNTACTICALLY a primitive
        // (`arg_scalar_syntactic`: a literal / arithmetic / unary / update /
        // template). A bare-identifier argument is NEVER scalar evidence —
        // not even when it names another param already proven to have scalar
        // inflow. Concretely, `function g(p){p+=1;return p;} function
        // f(n){return g(n);}` does NOT propagate proof from `f`'s `n` to `g`'s
        // `p`: `g`'s compound-assign on `p` rejects fail-closed even though
        // every call to `f` passes a number literal.
        //
        // The ∀ half (Spec 7 Task 1, "existential-laundering closure"): the
        // existential-only proof above is not enough. `f(5); f(g())` has a
        // scalar edge (`f(5)`) and an unproven edge (`f(g())`, an indirect
        // call-result that could deliver a heap handle) for the SAME param —
        // existential proof alone would admit `p += 1` in `f` on the strength
        // of the `f(5)` edge alone, even though the `f(g())` edge could hand
        // `p` a non-scalar at runtime. So a param is proven-scalar iff (some
        // edge is `arg_scalar_syntactic`) AND (no edge is a VETO). An edge is
        // a veto when its argument is an ARRAY (`arg_array_literal`, or a
        // bare identifier in `array_bindings`), an OBJECT (`arg_obj_slots` is
        // `Some`, which — see the historical note below — covers every bare
        // identifier), or is simply NOT syntactically scalar (an unproven
        // indirect form: a call-result, a member read, anything
        // `arg_scalar_syntactic` didn't recognize). Only a syntactically-
        // scalar argument is non-veto evidence; every other shape vetoes. A
        // self-recursive `h(p+1)` passes `p+1`, which IS syntactically scalar
        // (arithmetic) — not a veto — so a self-recursive proof chain still
        // closes over itself correctly.
        //
        // This requires two passes over `self.calls`, not one: the veto for
        // a given `(callee, param)` key can be discovered on any edge, in any
        // order, so both `has_scalar_evidence` and `has_veto` must be fully
        // accumulated across ALL edges before either can be consulted, unlike
        // the single existential pass this replaces (which could commit a key
        // the moment one qualifying edge was seen). There is still no fixpoint
        // needed beyond that: an identifier's presence in `scalar_inflow_params`
        // is never consulted, so no edge's admission can unblock another edge's
        // admission on a later round.
        //
        // (Historical note: an earlier version of this loop attempted a
        // "scalar pass-through" branch that treated a bare identifier as
        // scalar evidence when it named an already-proven-scalar param, and
        // ran to a fixpoint to let that propagate transitively. That branch
        // was dead code: `arg_obj_slot` returns `Some(ObjSlot::Binding(..))`
        // for EVERY bare-identifier argument, object-holding or not, so
        // `arg_is_object` below is true for every identifier argument and the
        // loop `continue`s before any identifier-based scalar check could
        // run. It was excised rather than made real. A real pass-through lane
        // is future work; it must consult actual object/shape info for the
        // identifier — not mere identifier-ness, since an identifier that
        // aliases an array/object must never be treated as scalar.)
        let mut scalar_evidence: BTreeSet<(String, String)> = BTreeSet::new();
        let mut veto: BTreeSet<(String, String)> = BTreeSet::new();
        for edge in &self.calls {
            let Some(params) = self.functions.get(&edge.callee) else {
                continue;
            };
            for (k, param_name) in params.iter().enumerate() {
                let key = (edge.callee.clone(), param_name.clone());
                let ident = edge.arg_array_names.get(k).cloned().flatten();
                let is_array = edge.arg_array_literal.get(k).copied().unwrap_or(false)
                    || ident.as_ref().is_some_and(|(caller, name)| {
                        array_bindings.contains(&(caller.clone(), name.clone()))
                    });
                let is_object = matches!(edge.arg_obj_slots.get(k), Some(Some(_)));
                let is_scalar = edge.arg_scalar_syntactic.get(k).copied().unwrap_or(false);
                // Veto: an array/object argument, OR an argument that is
                // neither proven scalar nor a known array/object — an
                // unproven indirect form (call-result, member read) that
                // could deliver a heap handle. Only a syntactically-scalar
                // argument is non-veto evidence.
                let is_veto = is_array || is_object || !is_scalar;
                if is_scalar {
                    scalar_evidence.insert(key.clone());
                }
                if is_veto {
                    veto.insert(key);
                }
            }
        }
        for key in scalar_evidence {
            if !veto.contains(&key) {
                self.scalar_inflow_params.insert(key);
            }
        }

        // Step 2: drain call edges and wire the interprocedural constraints.
        let calls = std::mem::take(&mut self.calls);
        for edge in calls {
            let Some(params) = self.functions.get(&edge.callee).cloned() else {
                continue; // Not a user function (builtin / undefined) — skip.
            };

            for (k, param_name) in params.iter().enumerate() {
                let is_array_param =
                    array_bindings.contains(&(edge.callee.clone(), param_name.clone()));
                let arg_identifier_name = edge.arg_array_names.get(k).cloned().flatten();
                // Non-scalar taint: this call passes an ARRAY value at position
                // `k` (a bare-identifier array binding, or a syntactic array
                // literal / `new Array` / `Array(..)`). The receiving param
                // therefore holds a heap handle at runtime, not a number — a
                // compound/update assignment on it must fail closed (the
                // resolve-phase param compound/update allowlist reads this).
                let arg_is_array_identifier =
                    arg_identifier_name.as_ref().is_some_and(|(caller, name)| {
                        array_bindings.contains(&(caller.clone(), name.clone()))
                    });
                let arg_is_array_literal = edge.arg_array_literal.get(k).copied().unwrap_or(false);
                if arg_is_array_identifier || arg_is_array_literal {
                    self.non_scalar_params
                        .insert((edge.callee.clone(), param_name.clone()));
                }
                if let Some((caller, name)) = arg_identifier_name.clone().filter(|_| is_array_param)
                {
                    // Array element flow is bidirectional shared storage: union
                    // the caller argument's element node with the param's. Only
                    // meaningful when the argument is a bare identifier — the
                    // only shape that can alias array storage.
                    let caller_elem = self.array_elem_node_for(&caller, &name);
                    let param_elem = self.array_elem_node_for(&edge.callee, param_name);
                    self.uf.union(caller_elem, param_elem);
                    // Elements of the two arrays are the same objects.
                    self.obj_flows.push((
                        ObjSlot::ArrayElem(caller, name),
                        ObjSlot::ArrayElem(edge.callee.clone(), param_name.clone()),
                    ));
                }
                // Independently of the array-element union above, ALSO wire the
                // ordinary scalar/string/float flow edge whenever the call site
                // supplies an argument node. This is NOT mutually exclusive with
                // the array-union branch: `is_array_param` can be true purely
                // because the callee's param is read through a `.length`-only
                // receiver (see `visit_member`'s "array-bias" registration) while
                // the param is ACTUALLY a runtime string (e.g. `fastaRepeat`'s
                // `seq`, which is both `seq.length`'d and `seq.substring`'d).
                // Skipping this edge whenever the arg happens to be a bare
                // identifier passed to such a param silently drops the only
                // signal that proves the param `Repr::String` — the callee
                // param never resolves as a string and the substring/`.length`
                // gates reject it, even though codegen would lower it
                // correctly. Wiring it unconditionally costs nothing for a
                // genuine array argument: its own scalar node carries no
                // string/float seed unless the SAME identifier is independently
                // used as a scalar elsewhere, in which case surfacing that flow
                // is correct (a real conflict), not a regression.
                if let Some(&arg_node) = edge.arg_nodes.get(k) {
                    // Scalar arg flow is directional: arg -> param.
                    let pnode = self.scalar_node_for(&edge.callee, param_name);
                    self.add_edge(arg_node, pnode);
                    // Object aliasing arg ~ param (no-op unless proven object).
                    if let Some(Some(slot)) = edge.arg_obj_slots.get(k) {
                        self.obj_flows.push((
                            slot.clone(),
                            ObjSlot::Binding(edge.callee.clone(), param_name.clone()),
                        ));
                    }
                }
            }

            // Return flow is directional: callee return -> call-site result.
            let ret = self.return_node_for(&edge.callee);
            self.add_edge(ret, edge.result_node);
        }
    }

    // ---- Phase C2: object-shape propagation -----------------------------

    fn resolve_objects(&mut self) {
        // 1. Propagate field lists across flows to a fixpoint (copy into
        //    unknown sides only; mismatches are flagged once, afterwards).
        let mut fields_of: BTreeMap<ObjSlot, Vec<String>> = self.obj_literal_fields.clone();
        loop {
            let mut changed = false;
            for (a, b) in &self.obj_flows {
                match (fields_of.contains_key(a), fields_of.get(b).cloned()) {
                    (false, Some(fields)) => {
                        fields_of.insert(a.clone(), fields);
                        changed = true;
                    }
                    (true, None) => {
                        fields_of.insert(b.clone(), fields_of[a].clone());
                        changed = true;
                    }
                    _ => {}
                }
            }
            if !changed {
                break;
            }
        }
        for (a, b) in &self.obj_flows {
            if let (Some(fa), Some(fb)) = (fields_of.get(a), fields_of.get(b)) {
                if fa != fb {
                    self.obj_conflicts.push(format!(
                        "conflicting object shapes flow between {a:?} and {b:?}"
                    ));
                }
            }
        }

        // 2. Union per-field storage across flows between object slots; both
        //    endpoints of an object flow are observable through an alias, so
        //    they materialize.
        let flows = self.obj_flows.clone();
        for (a, b) in &flows {
            let Some(names) = fields_of.get(a).cloned() else {
                continue;
            };
            if !fields_of.contains_key(b) {
                continue;
            }
            for name in &names {
                let x = self.obj_field_node_for(a, name);
                let y = self.obj_field_node_for(b, name);
                self.uf.union(x, y);
            }
            self.obj_materialized.insert(a.clone());
            self.obj_materialized.insert(b.clone());
        }

        // 2.5. Compute which *pending*-conflict slots (structurally-unsupported
        //      literals — non-identifier key / getter-setter / nested object;
        //      see `record_object_literal`) are observably READ somewhere they
        //      can be reached, and so must be promoted below: a purely
        //      write-only or purely aliased-but-never-read pending literal
        //      stays on the fold lane untouched (fold-first). See the doc
        //      comment on `pending_slots_reached_by_a_read` for the exact
        //      rule and why it is shaped this way.
        let promote_via_read = self.pending_slots_reached_by_a_read();

        // 2.6. Pre-mark materialization for every WRITE access on a
        //      known-shape base, BEFORE gating individual accesses in step 3
        //      below. Materialization is a whole-program property: a write
        //      appearing later in the source must still be visible to an
        //      earlier unknown-field READ's materialized-gate (step 3), not
        //      just to writes processed after it in this same pass.
        for access in &self.obj_accesses {
            if access.is_write {
                if let Some(names) = fields_of.get(&access.base) {
                    if names.contains(&access.field) {
                        self.obj_materialized.insert(access.base.clone());
                    }
                }
            }
        }

        // 3. Wire deferred member accesses through canonical field storage.
        let accesses = std::mem::take(&mut self.obj_accesses);
        for access in accesses {
            let Some(names) = fields_of.get(&access.base) else {
                continue; // not an object: fold lane / existing behavior
            };
            if !names.contains(&access.field) {
                // Unknown-field access on a KNOWN shape: a real conflict only
                // when the base is observable outside the fold lane — a
                // write, or a base materialized elsewhere. A read-only
                // unknown-field access on a literal that never materializes
                // matches JS (`undefined`) and must NOT reject (fold-first).
                if access.is_write || self.obj_materialized.contains(&access.base) {
                    self.obj_conflicts.push(format!(
                        "unknown field '{}' on fixed-shape object {:?}",
                        access.field, access.base
                    ));
                }
                continue;
            }
            let field_node = self.obj_field_node_for(&access.base, &access.field);
            if access.is_write {
                self.add_edge(access.other, field_node);
                self.obj_materialized.insert(access.base.clone());
            } else {
                // FLOAT-ONLY: an object field read is materialised as a raw
                // i64/f64 (object fields are I64/F64 only); a string flowing
                // into a field must not prove the read result `Repr::String`
                // (Finding 2).
                self.add_edge_float_only(field_node, access.other);
            }
        }

        // 3b. Wire deferred computed uniform-object reads `base[key]` (Task
        //     3) through EVERY field's storage: the read result is float iff
        //     the (uniform) field repr is float. A non-object base has no
        //     field list — left untouched (fold lane / existing behavior).
        //     `add_edge_float_only` keeps object fields on the I64/F64 axis
        //     (no string can prove through a field read).
        let uniform_reads = std::mem::take(&mut self.uniform_computed_reads);
        for (base, result) in uniform_reads {
            let Some(names) = fields_of.get(&base).cloned() else {
                continue;
            };
            for name in &names {
                let field_node = self.obj_field_node_for(&base, name);
                self.add_edge_float_only(field_node, result);
            }
        }

        // 4. Promote deferred structural conflicts for any slot that was
        //    forced onto the object lane: reassigned (already recorded in
        //    `obj_materialized` eagerly by `visit_assignment`'s whole-object
        //    reassignment branch), or reached by an observable read per 2.5
        //    above. A structurally-unsupported literal that stayed read-only
        //    and non-aliased, or that is only written-to and consumed via
        //    generic enumeration (`Object.keys`-style builtins, which never
        //    create a deferred field access here), never materializes and so
        //    keeps its fold-lane behavior byte-identically (fold-first
        //    invariant) — this is the whole point of the deferral.
        let pending = std::mem::take(&mut self.obj_pending_conflicts);
        for (slot, message) in pending {
            if self.obj_materialized.contains(&slot) || promote_via_read.contains(&slot) {
                self.obj_conflicts.push(message);
            }
        }

        self.obj_fields_of = fields_of;
    }

    /// Which `obj_pending_conflicts` slots are observably READ somewhere
    /// reachable from them, and so must be promoted to a real (rejected)
    /// conflict rather than staying deferred.
    ///
    /// A pending (structurally-unsupported) literal hits the buggy fold-lane
    /// `.field` read codegen this gate exists to guard against precisely when
    /// its value can be read from somewhere OTHER than a compile-time fold of
    /// its own literal text:
    ///
    /// - **Same-slot write + read**: the slot is both the base of a WRITE
    ///   `ObjAccess` and a READ `ObjAccess` — the write makes a same-slot
    ///   fold-to-literal-text read stale (`const p = {"a-b":1}; p.c = 2;
    ///   console.log(p.c);` must print `2`, not the folded/default value).
    /// - **Cross-slot via any flow**: the slot participates in ANY
    ///   object-aliasing flow (assignment, array-element sharing,
    ///   arg↔param, return↔call-site — `obj_flows`, regardless of whether
    ///   either endpoint has a known field list) AND *some* slot in that
    ///   flow-connected component has a READ `ObjAccess` anywhere. Crossing a
    ///   flow means the reading site has no visibility into the original
    ///   literal text to fold from at all (`function f(o){return o.c;}
    ///   f(structuralLiteral)` must reject, not silently fold to `0`) — a
    ///   write on that same component is not required.
    ///
    /// A literal that is only ever read locally with no write (the classic
    /// read-only fold-lane case), or only ever written-to/aliased but never
    /// read through this axis (e.g. deleted-from and reinserted, then only
    /// consumed via `Object.keys`/`Object.values`/`Object.entries`/
    /// `Reflect.ownKeys`, none of which create a deferred `ObjAccess`), is
    /// NOT promoted — that would re-break fold-first, the whole point of the
    /// pending-conflict deferral. `delete <base>.field` (and any bare,
    /// top-level `<base>.field;` expression statement, whose result is
    /// unconditionally discarded either way) is deliberately excluded from
    /// counting as a read — see `visit_unobserved_member_target`.
    fn pending_slots_reached_by_a_read(&self) -> BTreeSet<ObjSlot> {
        // Union-find over `ObjSlot`s via `obj_flows` (undirected aliasing).
        // `ObjSlot`s are never merged in the *field-storage* union-find
        // (`self.uf`, which only unions field nodes of slots that already
        // have BOTH field lists known — step 2 above); this is a separate,
        // slot-identity-level grouping used only to decide promotion.
        fn find(parent: &mut BTreeMap<ObjSlot, ObjSlot>, x: &ObjSlot) -> ObjSlot {
            let p = parent.entry(x.clone()).or_insert_with(|| x.clone()).clone();
            if &p == x {
                return p;
            }
            let root = find(parent, &p);
            parent.insert(x.clone(), root.clone());
            root
        }

        let mut parent: BTreeMap<ObjSlot, ObjSlot> = BTreeMap::new();
        for (a, b) in &self.obj_flows {
            let ra = find(&mut parent, a);
            let rb = find(&mut parent, b);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        let mut members: BTreeMap<ObjSlot, BTreeSet<ObjSlot>> = BTreeMap::new();
        for (a, b) in &self.obj_flows {
            let root = find(&mut parent, a);
            members.entry(root.clone()).or_default().insert(a.clone());
            members.entry(root).or_default().insert(b.clone());
        }

        let mut comp_has_read: BTreeSet<ObjSlot> = BTreeSet::new();
        let mut comp_has_write: BTreeSet<ObjSlot> = BTreeSet::new();
        for access in &self.obj_accesses {
            let root = find(&mut parent, &access.base);
            if access.is_write {
                comp_has_write.insert(root);
            } else {
                comp_has_read.insert(root);
            }
        }

        let mut promoted = BTreeSet::new();
        for slot in self.obj_pending_conflicts.keys() {
            let root = find(&mut parent, slot);
            let component_size = members.get(&root).map(BTreeSet::len).unwrap_or(1);
            let has_read = comp_has_read.contains(&root);
            let has_write = comp_has_write.contains(&root);
            if has_read && (has_write || component_size > 1) {
                promoted.insert(slot.clone());
            }
        }
        promoted
    }

    // ---- Phase D: solve → table ----------------------------------------

    /// BFS reachability over the directed edge graph from `seed_nodes`,
    /// endpoints canonicalised through the array-element union-find. Shared by
    /// the float and string axes. Consumes nothing (adjacency rebuilt by caller).
    fn solve_reach(&mut self, adj: &[Vec<usize>], seed_nodes: &[usize]) -> Vec<bool> {
        let n = self.node_count;
        let mut hit = vec![false; n];
        let mut queue: VecDeque<usize> = VecDeque::new();
        for &s in seed_nodes {
            let r = self.uf.find(s);
            if !hit[r] {
                hit[r] = true;
                queue.push_back(r);
            }
        }
        while let Some(u) = queue.pop_front() {
            for &v in &adj[u] {
                if !hit[v] {
                    hit[v] = true;
                    queue.push_back(v);
                }
            }
        }
        hit
    }

    /// Build the canonicalised FLOAT adjacency list (all edges; consumes
    /// `self.edges`).
    fn build_adjacency(&mut self) -> Vec<Vec<usize>> {
        let n = self.node_count;
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let edges = std::mem::take(&mut self.edges);
        for (from, to, _in_string) in edges {
            let f = self.uf.find(from);
            let t = self.uf.find(to);
            adj[f].push(t);
        }
        adj
    }

    fn emit_table(mut self) -> ReprTable {
        let n = self.node_count;
        let float_seeds = std::mem::take(&mut self.seeds);
        let string_seeds = std::mem::take(&mut self.string_seeds);
        // No string seeds ⇒ no string decisions, no plain-write fixpoint, no
        // taint — skip the extra edge snapshot/clone and the fixpoint entirely
        // (the all-integer/float fast path stays byte-identical). Programs with
        // strings are the only ones that pay for the string adjacency + taint.
        let has_strings = !string_seeds.is_empty();
        // Snapshot the raw edges BEFORE `build_adjacency` consumes them —
        // needed both to build the string-only adjacency and to detect a DIRECT
        // write into a scalar/return node from an unbacked source (see
        // `plain_write_targets`). Only cloned when strings are present.
        let edges_snapshot: Vec<(usize, usize, bool)> = if has_strings {
            self.edges.clone()
        } else {
            Vec::new()
        };
        let adj = self.build_adjacency();
        let float = self.solve_reach(&adj, &float_seeds);
        // String reachability runs over the STRING adjacency: edges added
        // `add_edge_float_only` (array-element / object-field reads) are
        // excluded so a captor of such a read is never proven `Repr::String`
        // (Finding 2). The float axis above keeps every edge.
        // `tainted[node]` marks nodes carrying a FRESH runtime string handle
        // (reachable from a `+`/interpolated-template/`+=` result), as opposed
        // to an interned literal constant. Codegen may `==`/`!=`-compare or
        // truthiness-test an interned handle correctly (identity == value), but
        // NOT a fresh concat handle — so tainted string operands are rejected in
        // those positions (fail-closed) while interned ones stay allowed.
        let (string, tainted, non_ascii) = if has_strings {
            let mut string_adj: Vec<Vec<usize>> = vec![Vec::new(); n];
            for &(from, to, in_string) in &edges_snapshot {
                if in_string {
                    let f = self.uf.find(from);
                    let t = self.uf.find(to);
                    string_adj[f].push(t);
                }
            }
            let string = self.solve_reach(&string_adj, &string_seeds);
            // Taint seeds: runtime-string nodes that are actually string-typed.
            let taint_seeds: Vec<usize> = std::mem::take(&mut self.runtime_string_nodes)
                .into_iter()
                .filter(|&node| string[self.uf.find(node)])
                .collect();
            let tainted = self.solve_reach(&string_adj, &taint_seeds);
            // Non-ASCII provenance: same string adjacency, seeded by non-ASCII
            // literals/templates (see the `Literal`/`TemplateLiteral` arms of
            // `visit_expr`).
            let non_ascii_seeds = std::mem::take(&mut self.non_ascii_seeds);
            let non_ascii = self.solve_reach(&string_adj, &non_ascii_seeds);
            (string, tainted, non_ascii)
        } else {
            (vec![false; n], vec![false; n], vec![false; n])
        };

        // Nodes fed by a write whose source is UNBACKED as a string handle —
        // i.e. codegen materializes a raw `i64` there, but the solved repr may
        // say `String`. Two kinds of unbacked source:
        //
        //   (a) A plain integer: string-unreachable AND float-unreachable —
        //       exactly the sources the `(false, false) => {}` arms below leave
        //       as the default `I64` repr (an integer literal, or any
        //       expression that never touches a string/float seed). An unseeded
        //       integer literal does NOT itself seed EITHER axis, so
        //       `let x = "a"; x = 5;` would otherwise be classified by
        //       `(string[x], float[x])` alone as plain `Repr::String` (string-
        //       reachable via `"a"`, no float seed to conflict with) — and
        //       codegen's `is_string_valued` would read the raw `5` as a handle.
        //
        //   (b) The call-RESULT of a DOWNGRADED mixed return (string branch +
        //       plain/int branch). The returns loop below leaves such a return
        //       at the default I64 repr (no `Repr::String` entry), so codegen's
        //       call arm (`return_repr(callee) != String`) materializes its call
        //       result as a raw `i64`. String-reachability, however, still flows
        //       from the return node into that result node (the
        //       `add_edge(ret, result_node)` in `resolve_calls`) and onward into
        //       any scalar/param/return that captures the call — which would
        //       then classify `String` over a runtime int (Finding 1: silent
        //       miscompile). Treating the result node as an unbacked source
        //       forces those captors into `plain_write_targets` so they FAIL
        //       CLOSED (conflict) instead. A downgraded return is itself fed by
        //       such a result in a chain (`return g(...)`), so its own result
        //       becomes unbacked too — hence the fixpoint below.
        //
        // A NEVER-called downgraded return has NO result-node edge, adds no
        // unbacked source, and stays a silent downgrade with no downstream
        // conflict — the `kali check`-only benchmark fixtures (never-called
        // `dead*` mixed-return functions) depend on this and stay green.
        //
        // REJECT-DON'T-MISCOMPILE: a scalar/return node fed by an unbacked
        // source that is ALSO string-reachable is exactly as unsound as the
        // existing string+float conflict, and is reported identically.
        let plain_write_targets: BTreeSet<usize> = if !has_strings {
            // No strings ⇒ nothing can be a mislabelled string write; skip the
            // whole fixpoint (empty-table fast path stays byte-identical).
            BTreeSet::new()
        } else {
            // Canonicalise once; scalar/return nodes are never unioned, so their
            // raw ids equal their roots (the match arms below index by raw id).
            let canon_edges: Vec<(usize, usize)> = edges_snapshot
                .iter()
                .map(|(f, t, _in_string)| (self.uf.find(*f), self.uf.find(*t)))
                .collect();
            let value_targets: BTreeSet<usize> = self
                .scalar_node
                .values()
                .chain(self.return_node.values())
                .map(|&v| self.uf.find(v))
                .collect();
            let return_roots: BTreeSet<usize> = self
                .return_node
                .values()
                .map(|&v| self.uf.find(v))
                .collect();

            let mut targets: BTreeSet<usize> = BTreeSet::new();
            let mut unbacked_results: BTreeSet<usize> = BTreeSet::new();
            loop {
                // (1) Value targets fed by ANY unbacked source (plain int, or a
                //     downgraded return's call-result node).
                let mut next_targets: BTreeSet<usize> = BTreeSet::new();
                for &(from, to) in &canon_edges {
                    if !value_targets.contains(&to) {
                        continue;
                    }
                    let plain_int = !string[from] && !float[from];
                    if plain_int || unbacked_results.contains(&from) {
                        next_targets.insert(to);
                    }
                }
                // (2) Returns that WILL be downgraded under `next_targets`
                //     (string-reachable, not float, and a plain_write_target):
                //     their downstream neighbour is the call-result node codegen
                //     leaves as a raw i64 — a fresh unbacked source.
                let mut next_unbacked: BTreeSet<usize> = BTreeSet::new();
                for &(from, to) in &canon_edges {
                    if return_roots.contains(&from)
                        && string[from]
                        && !float[from]
                        && next_targets.contains(&from)
                    {
                        next_unbacked.insert(to);
                    }
                }
                if next_targets == targets && next_unbacked == unbacked_results {
                    break;
                }
                targets = next_targets;
                unbacked_results = next_unbacked;
            }
            targets
        };

        let mut table = ReprTable::default();

        // Scalars (BTreeMap ⇒ deterministic order). Scalar nodes are never
        // unioned, so the flag can be read directly.
        let scalars: Vec<((String, String), usize)> = self
            .scalar_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for ((func, name), node) in scalars {
            match (string[node], float[node]) {
                (true, true) => table.add_shape_conflict(scope_conflict_message(&func, &name)),
                (true, false) if plain_write_targets.contains(&node) => {
                    table.add_shape_conflict(scope_conflict_message(&func, &name))
                }
                (true, false) => {
                    table.set_scalar(&func, &name, Repr::String);
                    if tainted[node] {
                        table.mark_string_concat_tainted(&func, &name);
                    }
                    if non_ascii[node] {
                        table.mark_string_non_ascii(&func, &name);
                    }
                }
                (false, true) => table.set_scalar(&func, &name, Repr::F64),
                (false, false) => {}
            }
        }

        // Array elements — read via the union-find representative.
        let elems: Vec<((String, String), usize)> = self
            .array_elem_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for ((func, name), node) in elems {
            // Every binding/param with an element node is an array (i64 or f64),
            // including subscripted params and pass-through array params linked
            // only via the transitive array-param fixpoint (which unions their
            // element node into the callee's in `resolve_calls`).
            table.set_array_binding(&func, &name);
            let rep = self.uf.find(node);
            if string[rep] {
                // The element node unions every store AND every read into one
                // shared node, so plain reachability cannot see a mix of
                // string and non-string stores — consult the recorded store
                // sources directly. A float store into a string-reachable
                // element is exactly as unsound (mixed_store handles the
                // "int literal stored + string stored" shape; `float[rep]`
                // catches the "float stored + string stored" shape).
                let mixed_store = self
                    .element_store_sources
                    .iter()
                    .any(|(e, s)| self.uf.find(*e) == rep && !string[self.uf.find(*s)]);
                if mixed_store || float[rep] {
                    table.add_shape_conflict(element_conflict_message(&func, &name));
                } else {
                    table.set_array_element(&func, &name, Repr::String);
                    if non_ascii[rep] {
                        table.mark_array_element_non_ascii(&func, &name);
                    }
                    if tainted[rep] {
                        table.mark_array_element_concat_tainted(&func, &name);
                    }
                }
            } else if float[rep] {
                table.set_array_element(&func, &name, Repr::F64);
            }
        }

        // I2: returning a String-element array binding has NO codegen lowering.
        // The caller captures a raw i64 handle (`return_repr` is not proven
        // String for an array), and every downstream element read / `join` on
        // that captured value falls through to a silent `0` — the whole
        // call-result-captor family (`const c = mk(s); c[0]` / `c.join(...)`),
        // which no gate catches at the READ site. Reject at the choke point
        // (the return) with an element-style shape conflict. Monotone: only a
        // return whose element node SOLVES `Repr::String` (string-reachable,
        // no mixed/float store) conflicts — int/float array returns are
        // untouched.
        for (func, name) in &self.array_binding_returns {
            if let Some(&node) = self.array_elem_node.get(&(func.clone(), name.clone())) {
                let rep = self.uf.find(node);
                if !string[rep] {
                    continue;
                }
                let mixed_store = self
                    .element_store_sources
                    .iter()
                    .any(|(e, s)| self.uf.find(*e) == rep && !string[self.uf.find(*s)]);
                if !mixed_store && !float[rep] {
                    table.add_shape_conflict(returning_string_array_message(func, name));
                }
            }
        }

        // Returns.
        let returns: Vec<(String, usize)> = self
            .return_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for (func, node) in returns {
            match (string[node], float[node]) {
                (true, true) => table.add_shape_conflict(format!(
                    "return value of `{func}` is used as both a string and a number"
                )),
                // Mixed string + plain returns (`return \`s\`` on one path,
                // `return unprovenParam` on another): the repr axis cannot
                // claim `Repr::String` — a call site could receive a raw
                // integer — but unlike the scalar-binding case above it does
                // NOT hard-reject either: leaving the return unproven keeps
                // both codegen (`is_string_valued`'s call arm) and the E3200
                // gate (`operand_repr_is_string`) on the pre-string-flow I64
                // lane, byte-identical to the behavior before this axis was
                // consumed (pinned by the template-literal-concatenation
                // benchmark fixture, whose never-called `dead*` functions
                // have exactly this shape).
                (true, false) if plain_write_targets.contains(&node) => {}
                (true, false) => {
                    table.set_return(&func, Repr::String);
                    if tainted[node] {
                        table.mark_string_concat_tainted_return(&func);
                    }
                    if non_ascii[node] {
                        table.mark_string_non_ascii_return(&func);
                    }
                }
                (false, true) => table.set_return(&func, Repr::F64),
                (false, false) => {}
            }
        }

        // Params (aliased to each param's scalar node, indexed by position).
        let functions: Vec<(String, Vec<String>)> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (func, params) in functions {
            for (index, name) in params.iter().enumerate() {
                if let Some(&node) = self.scalar_node.get(&(func.clone(), name.clone())) {
                    match (string[node], float[node]) {
                        (true, true) => table.add_shape_conflict(format!(
                            "param {index} of `{func}` is used as both a string and a number"
                        )),
                        (true, false) if plain_write_targets.contains(&node) => table
                            .add_shape_conflict(format!(
                                "param {index} of `{func}` is used as both a string and a number"
                            )),
                        (true, false) => {
                            table.set_param(&func, index, Repr::String);
                            // A param aliases its binding's scalar node; taint is
                            // keyed by (func, name) so codegen's identifier arm
                            // (which looks up by name) sees it.
                            if tainted[node] {
                                table.mark_string_concat_tainted(&func, name);
                            }
                            if non_ascii[node] {
                                table.mark_string_non_ascii(&func, name);
                            }
                        }
                        (false, true) => table.set_param(&func, index, Repr::F64),
                        (false, false) => {}
                    }
                }
            }
        }

        // Value-SELECTING merges (`a || b`, `a && b`, `cond ? a : b`). Unlike
        // `+` (whose result is a genuine string whenever either operand is), a
        // selecting merge yields ONE input UNCHANGED at runtime. If the solved
        // result is string-reachable but SOME input is a plain `i64` (neither
        // string- nor float-reachable), the runtime value can be a raw integer
        // where the repr says `String` — the same unsoundness the assignment-
        // shaped `plain_write_targets` guards. FAIL CLOSED with a conflict.
        //
        // Behaviour-neutral today: the parser lowers `&&`/`||`/`??` to
        // `BinaryExpression` (no string inflow reaches these merge results, so
        // no `String` claim to trip on) and `?:` parses degenerately, so no
        // current program reaches this arm — but it makes the guard REAL (the
        // field is now consumed, closing Finding 2) and protects
        // programmatically-built ASTs / any future parser change. It can only
        // reject, never widen.
        let merge_nodes = std::mem::take(&mut self.merge_nodes);
        for (result, inputs) in merge_nodes {
            let result = self.uf.find(result);
            if !string[result] {
                continue;
            }
            let has_plain_input = inputs.iter().any(|&i| {
                let i = self.uf.find(i);
                !string[i] && !float[i]
            });
            if has_plain_input {
                table.add_shape_conflict(
                    "value-selecting expression is used as both a string and a number".to_string(),
                );
            }
        }

        // Object shapes: one interned Shape per materialized object slot.
        // Unmaterialized (write-free, non-flowing) literals get NO entry —
        // codegen keeps its compile-time fold lane for them.
        let fields_of = std::mem::take(&mut self.obj_fields_of);
        let materialized = std::mem::take(&mut self.obj_materialized);
        for (slot, names) in &fields_of {
            if !materialized.contains(slot) {
                continue;
            }
            let fields: Vec<(String, Repr)> = names
                .iter()
                .map(|name| {
                    let node = self.obj_field_node_for(slot, name);
                    let rep = self.uf.find(node);
                    let repr = if float[rep] { Repr::F64 } else { Repr::I64 };
                    (name.clone(), repr)
                })
                .collect();
            let shape = table.intern_shape(fields);
            match slot {
                ObjSlot::Binding(func, name) => {
                    // A binding both object and float-unified is contradictory.
                    if let Some(&node) = self.scalar_node.get(&(func.clone(), name.clone())) {
                        if float[node] {
                            self.obj_conflicts.push(format!(
                                "binding '{name}' in '{func}' is used both as an object and as a number"
                            ));
                            continue;
                        }
                    }
                    table.set_scalar(func, name, Repr::Object(shape));
                }
                ObjSlot::ArrayElem(func, name) => {
                    table.set_array_binding(func, name);
                    table.set_array_element(func, name, Repr::Object(shape));
                }
                ObjSlot::Return(func) => table.set_return(func, Repr::Object(shape)),
            }
        }
        // Object params mirror the binding entry positionally.
        let functions: Vec<(String, Vec<String>)> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (func, params) in functions {
            for (index, name) in params.iter().enumerate() {
                if let Repr::Object(shape) = table.scalar(&func, name) {
                    table.set_param(&func, index, Repr::Object(shape));
                } else if table.is_array_binding(&func, name) {
                    // Array-of-objects param: the param itself has no scalar
                    // Object entry (only its elements do), but codegen still
                    // needs the shape at the param position to lower element
                    // accesses without re-deriving it from the array binding.
                    if let Repr::Object(shape) = table.array_element(&func, name) {
                        table.set_param(&func, index, Repr::Object(shape));
                    }
                }
            }
        }
        for message in std::mem::take(&mut self.obj_conflicts) {
            table.add_shape_conflict(message);
        }

        // Non-scalar (array-argument) param taint: copy verbatim for the
        // resolve-phase param compound/update allowlist.
        for (func, name) in std::mem::take(&mut self.non_scalar_params) {
            table.mark_non_scalar_param(&func, &name);
        }

        // Object-initialized binding taint: copy verbatim for the
        // resolve-phase compound/update allowlist (fasta Spec 7 Task 2). See
        // the field doc on `object_initialized_bindings` — this fires on the
        // declarator's syntactic shape alone, independent of whether the
        // object literal is ever materialized into a `Repr::Object` shape.
        for (func, name) in std::mem::take(&mut self.object_initialized_bindings) {
            table.mark_object_initialized_binding(&func, &name);
        }

        // Positive scalar-inflow proof: every PARAM not proven to receive a
        // scalar argument by an actual call edge is left at the default I64 by
        // convention only, so the param compound/update gate must reject it
        // (`params_lacking_scalar_inflow`). Recorded as the NEGATION of the
        // proven set, keyed per-parameter, so the gate can query it without
        // re-deriving param-ness. Never-called functions add all their params.
        for (func, params) in &self.functions {
            for name in params {
                if !self
                    .scalar_inflow_params
                    .contains(&(func.clone(), name.clone()))
                {
                    table.mark_param_lacking_scalar_inflow(func, name);
                }
            }
        }

        table
    }
}

/// Shape-conflict message for a scalar binding, rendering the synthetic
/// top-level function name `_start` as the user-facing "at module scope"
/// (a plain binding declared at module scope has no function to name).
fn scope_conflict_message(func: &str, name: &str) -> String {
    if func == TOP_LEVEL {
        format!("binding `{name}` at module scope is used as both a string and a number")
    } else {
        format!("binding `{name}` in `{func}` is used as both a string and a number")
    }
}

/// Shape-conflict message for an array's ELEMENT axis (mirrors
/// `scope_conflict_message`'s module-scope phrasing convention).
fn element_conflict_message(func: &str, name: &str) -> String {
    if func == TOP_LEVEL {
        format!("elements of `{name}` at module scope are used as both strings and numbers")
    } else {
        format!("elements of `{name}` in `{func}` are used as both strings and numbers")
    }
}

/// Shape-conflict message for a function returning a String-element array
/// binding (I2), following `element_conflict_message`'s module-scope phrasing.
fn returning_string_array_message(func: &str, name: &str) -> String {
    if func == TOP_LEVEL {
        format!(
            "returning `{name}` whose elements are strings from module scope is unavailable in the current direct-runtime path"
        )
    } else {
        format!(
            "returning `{name}` whose elements are strings from `{func}` is unavailable in the current direct-runtime path"
        )
    }
}

/// Classify a numeric literal value as a float seed. The AST stores literals as
/// `f64` (the raw token text is not retained), so a literal seeds float iff it
/// is not an exact finite integer (has a fractional part, or is non-finite).
fn is_float_literal(n: f64) -> bool {
    !(n.is_finite() && n.fract() == 0.0)
}

/// True when `expr` is the `Math` object (`Math` identifier).
fn is_math_object(expr: &Expression) -> bool {
    matches!(expr, Expression::Identifier(name) if name == "Math")
}

fn is_console_object(expr: &Expression) -> bool {
    matches!(expr, Expression::Identifier(name) if name == "console")
}

/// Extract the constructor identifier name from a `new`-expression callee,
/// which the parser may shape as `Identifier("Array")` or
/// `CallExpression { callee: Identifier("Array"), .. }` (`new Array(n)`).
fn constructor_name(callee: &Expression) -> Option<String> {
    match callee {
        Expression::Identifier(name) => Some(name.clone()),
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        },
        _ => None,
    }
}
