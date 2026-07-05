//! Escape-gate analysis: decides WHERE arenas go, producing a name-keyed
//! [`kali_common::ArenaTable`] consumed by codegen (Tasks 6/7).
//!
//! This module extends the existing [`OwnershipAnalyzer`] walk (it does **not**
//! run a second traversal): during the one HIR walk that already computes
//! ownership, it also records, per function scope, the raw facts the gate needs
//! — allocation presence, per-site fate summary, call targets, and per-loop
//! (pre-order ordinal) facts. Those raw facts are surfaced on
//! [`crate::MirProgram::arena_facts`]; [`compute_arena_table`] then applies the
//! fate lattice, the transitive reaches-allocation closure, and the loop
//! opening rule over them.
//!
//! Every join **fails closed**: any ambiguity vetoes the arena (or sends a site
//! to the global heap). Vetoing is always sound — the only cost is unreclaimed
//! memory. Wrongly arena-ing an escaping value would be a use-after-reset bug.

use std::collections::{BTreeMap, BTreeSet};

use kali_common::ArenaTable;
use kali_hir::{HirNodeId, HirNodeKind};

use crate::{LayoutDescriptor, MirProgram, OwnershipAnalyzer};

/// Host calls that consume (do not retain) their arguments, so a loop that only
/// calls these does not leak heap values out of its per-iteration arena.
///
/// Matches the stdout-write family actually registered in codegen
/// (`kali_codegen/src/intrinsics/host.rs`): any `console.*` method (console
/// serializes, never retains) and the `Kali.writeStdoutBytes` binary-stdout
/// intrinsic. The receiver check on `writeStdoutBytes` mirrors codegen's
/// `is_kali_write_stdout_bytes_call` — a user method merely named
/// `writeStdoutBytes` may retain its argument and must NOT pass.
fn is_whitelisted_host_method(base_object: Option<&str>, method: &str) -> bool {
    base_object == Some("console") || (base_object == Some("Kali") && method == "writeStdoutBytes")
}

/// Raw per-loop facts collected during the walk (pre-order ordinal keyed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopArenaFacts {
    /// Pre-order ordinal of the loop within its function.
    pub ordinal: u32,
    /// A fresh object/array literal appears lexically inside the loop body.
    pub reaches_alloc_directly: bool,
    /// Resolved bare-identifier call targets invoked inside the loop body.
    pub calls: BTreeSet<String>,
    /// The loop contains a closure/indirect/unresolved call, or a
    /// non-whitelisted host call — any of which vetoes the arena.
    pub has_unknown_call: bool,
    /// A heap-typed value flows out of the loop (assignment to an
    /// outer-declared binding, store into an object that outlives the loop, or
    /// a `return` of a heap value from inside it).
    pub has_outflow: bool,
}

/// Raw per-function facts collected during the walk. [`compute_arena_table`]
/// turns these into the final [`ArenaTable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArenaFacts {
    /// Function name (module scope is not surfaced).
    pub name: String,
    /// The function body contains at least one fresh object/array literal.
    pub allocates: bool,
    /// At least one allocation site has `Global` fate (escapes the dynamic
    /// extent) — under v1 coarseness this poisons every site in the function.
    pub has_global_site: bool,
    /// At least one allocation site has `ScopeLocal` fate (dies inside `f`).
    pub has_scope_local_site: bool,
    /// Resolved bare-identifier call targets invoked in the function body.
    pub calls: BTreeSet<String>,
    /// The function contains a closure/indirect/unresolved or non-whitelisted
    /// host call (taints reaches-allocation and loop gating).
    pub has_unknown_call: bool,
    /// Per-loop facts in pre-order.
    pub loops: Vec<LoopArenaFacts>,
}

// ---------------------------------------------------------------------------
// Raw collection state (lives on the analyzer during the walk).
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub(crate) struct FuncRaw {
    allocates: bool,
    has_global_site: bool,
    has_scope_local_site: bool,
    calls: BTreeSet<String>,
    has_unknown_call: bool,
    fresh_heap_bindings: BTreeSet<String>,
    /// Bindings reassigned from ANY may-hold-heap RHS (call results, member
    /// reads, ...) whose declarator layout is stale-scalar. Consulted ONLY by
    /// `arena_is_heap_value` — deliberately NOT fed into the fate
    /// classification: a call RESULT is not a fresh allocation site of this
    /// function, and classifying it ScopeLocal would wrongly grant
    /// `opens_arena` (dangling returned values after the exit reset).
    maybe_heap_bindings: BTreeSet<String>,
    loops: Vec<LoopArenaFacts>,
    /// Two same-named function scopes wrote into this entry (the table is
    /// name-keyed, so per-instance decisions are impossible) ⇒ poisoned at
    /// `into_facts`.
    name_collision: bool,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LoopRaw {
    ordinal: u32,
    inner_bindings: BTreeSet<String>,
    reaches_alloc_directly: bool,
    calls: BTreeSet<String>,
    has_unknown_call: bool,
    has_outflow: bool,
}

/// Arena-fact collector threaded through the ownership walk. All state is
/// additive; it never influences the ownership verdicts the walk already
/// produces.
#[derive(Debug, Default)]
pub(crate) struct ArenaCollector {
    functions: BTreeMap<String, FuncRaw>,
    /// Emission order of function labels (deterministic output).
    order: Vec<String>,
    /// Next pre-order loop ordinal for the current function.
    loop_ordinal: u32,
    /// Loops currently open in the current function (innermost last).
    loop_stack: Vec<LoopRaw>,
    /// Saved `(loop_ordinal, loop_stack)` for each enclosing function scope.
    saved: Vec<(u32, Vec<LoopRaw>)>,
}

impl ArenaCollector {
    fn func(&mut self, label: &str) -> &mut FuncRaw {
        if !self.functions.contains_key(label) {
            self.order.push(label.to_string());
        }
        self.functions.entry(label.to_string()).or_default()
    }

    /// NOTE: facts are name-keyed, so two same-named functions merge into one
    /// entry; the merge must stay conservative. Boolean facts OR together
    /// (veto-side), but `has_scope_local_site` and per-ordinal loop entries
    /// would GRANT across instances — so a detected collision poisons the
    /// entry outright (global site + unknown call, loops dropped).
    pub(crate) fn into_facts(self) -> Vec<FunctionArenaFacts> {
        let ArenaCollector {
            functions, order, ..
        } = self;
        order
            .into_iter()
            .filter(|label| label != "<module>")
            .filter_map(|label| {
                functions.get(&label).map(|raw| {
                    if raw.name_collision {
                        return FunctionArenaFacts {
                            name: label.clone(),
                            allocates: raw.allocates,
                            has_global_site: true,
                            has_scope_local_site: false,
                            calls: raw.calls.clone(),
                            has_unknown_call: true,
                            loops: Vec::new(),
                        };
                    }
                    FunctionArenaFacts {
                        name: label.clone(),
                        allocates: raw.allocates,
                        has_global_site: raw.has_global_site,
                        has_scope_local_site: raw.has_scope_local_site,
                        calls: raw.calls.clone(),
                        has_unknown_call: raw.has_unknown_call,
                        loops: raw.loops.clone(),
                    }
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Walk-extension hooks (invoked from scope.rs / walk.rs).
// ---------------------------------------------------------------------------

impl<'a> OwnershipAnalyzer<'a> {
    /// Called after a function/module scope is pushed: reset the per-function
    /// loop counter/stack, saving the enclosing function's.
    pub(crate) fn arena_enter_function(&mut self) {
        let label = self.current_scope_label();
        // Entries are created here, at scope push — so a pre-existing entry
        // means a second function scope with the same name (name-keyed facts
        // cannot tell instances apart ⇒ the merged entry is poisoned at
        // `into_facts`).
        let collision = self.arena.functions.contains_key(&label);
        let raw = self.arena.func(&label);
        if collision {
            raw.name_collision = true;
        }
        let ordinal = std::mem::take(&mut self.arena.loop_ordinal);
        let stack = std::mem::take(&mut self.arena.loop_stack);
        self.arena.saved.push((ordinal, stack));
    }

    /// Called before a function scope is popped: classify the fates of the
    /// function's fresh-heap bindings from the ownership verdicts the walk just
    /// finished computing.
    pub(crate) fn arena_finalize_current_function(&mut self) {
        let Some(scope) = self.scope_stack.last() else {
            return;
        };
        let label = scope.label.clone();
        let fresh: Vec<String> = self
            .arena
            .functions
            .get(&label)
            .map(|f| f.fresh_heap_bindings.iter().cloned().collect())
            .unwrap_or_default();
        for name in fresh {
            let Some(binding) = scope.bindings.iter().find(|b| b.name == name) else {
                continue;
            };
            // Fail-closed lattice: capture or flow-escape ⇒ Global; a bare
            // `return` ⇒ Returned (neither local nor global); otherwise the
            // value dies here ⇒ ScopeLocal.
            let global = !binding.captured_by.is_empty() || binding.escaped_via_flow;
            let scope_local = !global && !binding.returned;
            let f = self.arena.func(&label);
            if global {
                f.has_global_site = true;
            }
            if scope_local {
                f.has_scope_local_site = true;
            }
        }
    }

    /// Called after a function scope is popped: restore the enclosing
    /// function's loop counter/stack.
    pub(crate) fn arena_exit_function(&mut self) {
        if let Some((ordinal, stack)) = self.arena.saved.pop() {
            self.arena.loop_ordinal = ordinal;
            self.arena.loop_stack = stack;
        }
    }

    /// Enter a loop node: assign its pre-order ordinal and push a loop context.
    pub(crate) fn arena_enter_loop(&mut self) {
        let ordinal = self.arena.loop_ordinal;
        self.arena.loop_ordinal += 1;
        self.arena.loop_stack.push(LoopRaw {
            ordinal,
            ..LoopRaw::default()
        });
    }

    /// Exit a loop node: finalize its facts onto the current function.
    pub(crate) fn arena_exit_loop(&mut self) {
        let Some(loop_raw) = self.arena.loop_stack.pop() else {
            return;
        };
        let facts = LoopArenaFacts {
            ordinal: loop_raw.ordinal,
            reaches_alloc_directly: loop_raw.reaches_alloc_directly,
            calls: loop_raw.calls,
            has_unknown_call: loop_raw.has_unknown_call,
            has_outflow: loop_raw.has_outflow,
        };
        let label = self.current_scope_label();
        self.arena.func(&label).loops.push(facts);
    }

    /// A fresh object/array literal was reached in the current function body.
    pub(crate) fn arena_note_alloc(&mut self) {
        let label = self.current_scope_label();
        self.arena.func(&label).allocates = true;
        for loop_raw in &mut self.arena.loop_stack {
            loop_raw.reaches_alloc_directly = true;
        }
    }

    /// Record that `name` was declared in the currently-open loops (so outflow
    /// analysis knows it is loop-local).
    pub(crate) fn arena_note_declared_binding(&mut self, name: &str) {
        for loop_raw in &mut self.arena.loop_stack {
            loop_raw.inner_bindings.insert(name.to_string());
        }
    }

    /// Record that local `name` was initialized from a fresh object/array
    /// literal (its fate is classified at function finalization).
    pub(crate) fn arena_note_fresh_binding(&mut self, name: &str) {
        let label = self.current_scope_label();
        self.arena
            .func(&label)
            .fresh_heap_bindings
            .insert(name.to_string());
    }

    /// Record that local `name` was reassigned from a may-hold-heap RHS, so
    /// its (stale) declarator layout must no longer be trusted by
    /// `arena_is_heap_value`.
    fn arena_note_maybe_heap_binding(&mut self, name: &str) {
        let label = self.current_scope_label();
        self.arena
            .func(&label)
            .maybe_heap_bindings
            .insert(name.to_string());
    }

    fn arena_note_call(&mut self, target: &str) {
        let label = self.current_scope_label();
        self.arena.func(&label).calls.insert(target.to_string());
        for loop_raw in &mut self.arena.loop_stack {
            loop_raw.calls.insert(target.to_string());
        }
    }

    fn arena_note_unknown_call(&mut self) {
        let label = self.current_scope_label();
        self.arena.func(&label).has_unknown_call = true;
        for loop_raw in &mut self.arena.loop_stack {
            loop_raw.has_unknown_call = true;
        }
    }

    fn arena_note_global_site(&mut self) {
        let label = self.current_scope_label();
        self.arena.func(&label).has_global_site = true;
    }

    /// Classify a call expression's callee (and its argument literals).
    pub(crate) fn arena_note_call_expr(&mut self, children: &[HirNodeId]) {
        let Some(callee) = children.first().copied() else {
            return;
        };
        let callee_node = &self.nodes[callee.0 as usize];
        let mut whitelisted = false;
        match callee_node.kind {
            HirNodeKind::Ident => {
                let name = callee_node.text.clone();
                match name.and_then(|n| self.resolve_function_target(&n)) {
                    Some(target) => self.arena_note_call(&target),
                    // A bare identifier that is not a known function target is
                    // an indirect/closure call ⇒ taint.
                    None => self.arena_note_unknown_call(),
                }
            }
            HirNodeKind::MemberExpr => {
                let method = callee_node.text.clone().unwrap_or_default();
                let base_object = callee_node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .filter(|base| base.kind == HirNodeKind::Ident)
                    .and_then(|base| base.text.clone());
                if is_whitelisted_host_method(base_object.as_deref(), &method) {
                    whitelisted = true;
                } else {
                    self.arena_note_unknown_call();
                }
            }
            // IIFEs and every other computed/indirect callee ⇒ taint.
            _ => self.arena_note_unknown_call(),
        }

        // A fresh literal handed to a non-whitelisted call might be retained by
        // the callee ⇒ global (fail closed). Whitelisted host calls consume it.
        if !whitelisted {
            for arg in children.iter().skip(1).copied() {
                if self.arena_is_fresh_literal(arg) {
                    self.arena_note_global_site();
                }
            }
        }
    }

    /// Classify an assignment for global-site (function) and outflow (loop).
    pub(crate) fn arena_note_assignment(&mut self, left: HirNodeId, right: HirNodeId) {
        let left_node = &self.nodes[left.0 as usize];
        let left_kind = left_node.kind.clone();
        let rhs_fresh = self.arena_is_fresh_literal(right);
        let rhs_heap = self.arena_is_heap_value(right);

        match left_kind {
            HirNodeKind::Ident => {
                let name = left_node.text.clone().unwrap_or_default();
                // Store of a heap-holding value (fresh literal, an identifier
                // bound to a heap layout, or anything unknown — `rhs_heap`
                // treats `TaggedVal` as heap) into a binding that lives in a
                // scope strictly enclosing the current function ⇒ the value
                // outlives the function ⇒ Global. The plain-ident LHS never
                // sets `escaped_via_flow` in the ownership walk
                // (`is_heap_store_target` only covers member/chain targets),
                // so this is the only place that catches `cache = node`.
                // Unresolved names fail closed.
                if rhs_fresh || rhs_heap {
                    match self.resolve_binding(&name) {
                        Some((scope_index, _)) if scope_index < self.current_scope_index() => {
                            self.arena_note_global_site();
                            // The target binding lives in an ENCLOSING
                            // function: mark it may-heap in its OWNER's entry
                            // (scopes are function scopes here), so reads of
                            // it back in the owner stop trusting the stale
                            // declarator layout — closure-mediated writes
                            // must not launder heap values into scalars.
                            if let Some(owner) = self
                                .scope_stack
                                .get(scope_index)
                                .map(|scope| scope.label.clone())
                            {
                                self.arena
                                    .func(&owner)
                                    .maybe_heap_bindings
                                    .insert(name.clone());
                            }
                        }
                        Some(_) => {
                            // Same-scope reassignment: the binding's
                            // declarator layout is stale (frozen at
                            // declaration). A fresh LITERAL makes the binding
                            // a fresh allocation site of this function (feeds
                            // fate classification at finalize); ANY heap RHS
                            // (call result, member read, ...) additionally
                            // marks it may-hold-heap so `arena_is_heap_value`
                            // stops trusting the stale scalar layout — but
                            // does NOT enter fate classification (a call
                            // result is not this function's allocation site
                            // and must not flip `opens_arena`).
                            if rhs_fresh {
                                self.arena_note_fresh_binding(&name);
                            }
                            self.arena_note_maybe_heap_binding(&name);
                        }
                        None => self.arena_note_global_site(),
                    }
                }
                // Loop outflow: heap value assigned to a binding declared
                // outside the loop.
                if rhs_heap {
                    self.arena_note_outflow_to_binding(&name);
                }
            }
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr => {
                // Store into a field/element of a pre-existing object.
                if rhs_fresh {
                    self.arena_note_global_site();
                }
                if rhs_heap {
                    // Base object: if it outlives the loop, the heap value
                    // stored into it outlives the loop too.
                    if let Some(base) = left_node.children.first().copied() {
                        let base_node = &self.nodes[base.0 as usize];
                        if base_node.kind == HirNodeKind::Ident {
                            let base_name = base_node.text.clone().unwrap_or_default();
                            self.arena_note_outflow_to_binding(&base_name);
                        } else {
                            self.arena_note_outflow_all_loops();
                        }
                    } else {
                        self.arena_note_outflow_all_loops();
                    }
                }
            }
            _ => {
                // Unknown assignment target (no destructuring surface exists
                // today — anything unexpected lands here) ⇒ fail closed on
                // BOTH axes: the heap value may flow anywhere.
                if rhs_fresh || rhs_heap {
                    self.arena_note_global_site();
                    self.arena_note_outflow_all_loops();
                }
            }
        }
    }

    /// A `return` of a heap value from inside a loop leaks it out of the arena.
    pub(crate) fn arena_note_return(&mut self, children: &[HirNodeId]) {
        if self.arena.loop_stack.is_empty() {
            return;
        }
        let returns_heap = children
            .iter()
            .copied()
            .any(|child| self.arena_is_heap_value(child));
        if returns_heap {
            self.arena_note_outflow_all_loops();
        }
    }

    fn arena_note_outflow_to_binding(&mut self, name: &str) {
        for loop_raw in &mut self.arena.loop_stack {
            if !loop_raw.inner_bindings.contains(name) {
                loop_raw.has_outflow = true;
            }
        }
    }

    fn arena_note_outflow_all_loops(&mut self) {
        for loop_raw in &mut self.arena.loop_stack {
            loop_raw.has_outflow = true;
        }
    }

    fn arena_is_fresh_literal(&self, node_id: HirNodeId) -> bool {
        matches!(
            self.nodes[node_id.0 as usize].kind,
            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr
        )
    }

    /// The gate's OWN conservative may-hold-heap judgment. Deliberately NOT a
    /// raw `infer_layout` lookup: `infer_layout` resolves ambiguity OPEN
    /// (ternaries take only the last branch's layout, `||`/`&&` map to
    /// `Scalar("bool")` though they return an operand, binding layouts are
    /// frozen at the declarator), which is fine for layout inference but
    /// unsound as an escape gate. Every arm here errs toward heap.
    fn arena_is_heap_value(&self, node_id: HirNodeId) -> bool {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr => true,
            // Numbers/strings/bools are scalar; `null`/`undefined` infer
            // Scalar("unknown"), which the gate treats as heap (fail closed).
            HirNodeKind::Literal => is_heap_layout(&self.infer_layout(node_id)),
            // A ternary produces whichever branch runs: heap if ANY branch
            // (children[1..] = consequent, alternate) is heap. Malformed ⇒
            // heap.
            HirNodeKind::ConditionalExpr => {
                node.children.len() < 2
                    || node
                        .children
                        .iter()
                        .skip(1)
                        .any(|child| self.arena_is_heap_value(*child))
            }
            // Logical ops return an operand; a sequence returns its last
            // expression. Heap if ANY child is heap (malformed ⇒ heap).
            HirNodeKind::LogicalExpr | HirNodeKind::SequenceExpr => {
                node.children.is_empty()
                    || node
                        .children
                        .iter()
                        .any(|child| self.arena_is_heap_value(*child))
            }
            HirNodeKind::BinaryExpr => match node.text.as_deref() {
                // Genuinely scalar-producing operators. String `+` concat is
                // scalar too in v1: runtime strings are global-arena host
                // values and never dangle across a reset.
                Some(
                    "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>" | ">>>"
                    | "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=",
                ) => false,
                // `&&`/`||`/`??` parse as BinaryExpr and return an operand.
                Some("&&" | "||" | "??") => {
                    node.children.is_empty()
                        || node
                            .children
                            .iter()
                            .any(|child| self.arena_is_heap_value(*child))
                }
                _ => true,
            },
            HirNodeKind::UnaryExpr => {
                !matches!(node.text.as_deref(), Some("!" | "-" | "+" | "~" | "typeof"))
            }
            HirNodeKind::Ident => {
                let name = node.text.as_deref().unwrap_or_default();
                // A binding reassigned from a fresh heap literal OR any other
                // may-hold-heap RHS keeps its stale declarator layout, so
                // consult the sets recorded by `arena_note_assignment` first —
                // in the binding's OWNING function's entry (bindings are
                // capturable: the reassignment may have happened in a closure,
                // and this read may itself be in a different closure than the
                // write).
                if let Some((scope_index, _)) = self.resolve_binding(name) {
                    if let Some(owner) = self
                        .scope_stack
                        .get(scope_index)
                        .map(|scope| scope.label.as_str())
                    {
                        if self.arena.functions.get(owner).is_some_and(|f| {
                            f.fresh_heap_bindings.contains(name)
                                || f.maybe_heap_bindings.contains(name)
                        }) {
                            return true;
                        }
                    }
                }
                match self.resolve_binding_layout(name) {
                    Some(layout) => is_heap_layout(&layout),
                    None => true,
                }
            }
            // Calls, member reads, templates, everything else: unknown ⇒ heap.
            _ => true,
        }
    }
}

/// Heap-typed layouts for the gate. Fails closed: `TaggedVal` AND
/// `Scalar("unknown")` (the layout of `null`/`undefined` and unclassifiable
/// values) are treated as heap. Concrete scalars — including strings, which
/// are host-allocated into the global arena in v1 — are exempt from outflow.
fn is_heap_layout(layout: &LayoutDescriptor) -> bool {
    match layout {
        LayoutDescriptor::Scalar(name) => name == "unknown",
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// Decision pass: raw facts → ArenaTable.
// ---------------------------------------------------------------------------

/// Compute the name-keyed [`ArenaTable`] from a lowered [`MirProgram`].
///
/// Reads the raw per-function/per-loop facts collected during ownership
/// analysis (`mir.arena_facts`) and applies:
/// 1. `arena_eligible(f)` ⇔ `f` allocates and has no `Global`-fate site.
/// 2. `opens_arena(f)` ⇔ `arena_eligible(f)` and `f` has a `ScopeLocal` site.
/// 3. `loop_arena(f, ord)` ⇔ the loop reaches allocation through only known,
///    arena-eligible callees, has no heap outflow, and no unknown/non-whitelist
///    call.
pub fn compute_arena_table(mir: &MirProgram) -> ArenaTable {
    let facts_by_name: BTreeMap<&str, &FunctionArenaFacts> = mir
        .arena_facts
        .iter()
        .map(|facts| (facts.name.as_str(), facts))
        .collect();

    let eligible = |name: &str| -> bool {
        facts_by_name
            .get(name)
            .map(|f| f.allocates && !f.has_global_site)
            .unwrap_or(false)
    };

    let mut table = ArenaTable::default();

    for facts in &mir.arena_facts {
        if eligible(&facts.name) {
            table.set_arena_eligible(&facts.name);
            if facts.has_scope_local_site {
                table.set_opens_arena(&facts.name);
            }
        }

        for loop_facts in &facts.loops {
            if loop_arena_qualifies(loop_facts, &facts_by_name, &eligible) {
                table.set_loop_arena(&facts.name, loop_facts.ordinal);
            }
        }
    }

    table
}

fn loop_arena_qualifies(
    loop_facts: &LoopArenaFacts,
    facts_by_name: &BTreeMap<&str, &FunctionArenaFacts>,
    eligible: &impl Fn(&str) -> bool,
) -> bool {
    // (c) no unknown/closure/indirect call and no non-whitelisted host call.
    if loop_facts.has_unknown_call {
        return false;
    }
    // (b) no heap-typed value outflow.
    if loop_facts.has_outflow {
        return false;
    }

    // (a) reaches allocation transitively through only known, arena-eligible
    // callees. Any reachable-allocation callee that is NOT eligible vetoes
    // (fail closed).
    let mut reaches_alloc = loop_facts.reaches_alloc_directly;
    for target in &loop_facts.calls {
        if reaches_alloc_transitively(target, facts_by_name, &mut BTreeSet::new()) {
            if !eligible(target) {
                return false;
            }
            reaches_alloc = true;
        }
    }

    // An arena is only opened where there is reachable allocation to reclaim.
    reaches_alloc
}

/// Whether calling `name` can (transitively) reach a current-arena allocation.
/// An unknown call taints the callee as reaching allocation (it might allocate).
fn reaches_alloc_transitively(
    name: &str,
    facts_by_name: &BTreeMap<&str, &FunctionArenaFacts>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let Some(facts) = facts_by_name.get(name) else {
        // Unknown target ⇒ assume it may allocate (fail closed).
        return true;
    };
    if facts.allocates || facts.has_unknown_call {
        return true;
    }
    if !visiting.insert(name.to_string()) {
        // Recursion cycle: this frame's own allocation is already accounted
        // for above; treat the back-edge as non-reaching.
        return false;
    }
    let result = facts
        .calls
        .iter()
        .any(|callee| reaches_alloc_transitively(callee, facts_by_name, visiting));
    visiting.remove(name);
    result
}

#[cfg(test)]
#[path = "arena_gate_tests.rs"]
mod arena_gate_tests;
