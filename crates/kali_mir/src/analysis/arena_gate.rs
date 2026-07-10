//! Escape-gate analysis: decides WHERE arenas go, producing a name-keyed
//! [`kali_common::ArenaTable`] consumed by codegen (Tasks 6/7).
//!
//! This module extends the existing [`OwnershipAnalyzer`] walk (it does **not**
//! run a second traversal): during the one HIR walk that already computes
//! ownership, it also records, per function scope, the raw facts the gate needs
//! — allocation presence, per-site fate summary, call targets, and per-loop
//! (pre-order ordinal) facts. Heap-ness and escape resolution are no longer
//! judged immediately against possibly-stale walk-order state: the walk
//! defers global/outflow/arg-escape sites into
//! [`crate::analysis::escape_flow::FlowCollector`], and
//! [`ArenaCollector::into_facts`] resolves them against the interprocedural
//! [`crate::analysis::escape_flow::FlowSolution`] fixpoint before emission.
//! Those raw facts are surfaced on [`crate::MirProgram::arena_facts`];
//! [`compute_arena_table`] then applies the fate lattice, the transitive
//! reaches-allocation closure, and the loop opening rule over them.
//!
//! Every join **fails closed**: any ambiguity vetoes the arena (or sends a site
//! to the global heap). Vetoing is always sound — the only cost is unreclaimed
//! memory. Wrongly arena-ing an escaping value would be a use-after-reset bug.

use std::collections::{BTreeMap, BTreeSet};

use kali_common::ArenaTable;
use kali_hir::{HirNodeId, HirNodeKind};

use crate::analysis::escape_flow::{FlowCollector, FlowSolution, ValueClass};
use crate::{MirProgram, OwnershipAnalyzer};

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

/// The loop node kinds that receive a pre-order arena ordinal — the SINGLE
/// source of truth for "which loops are numbered", shared by the ownership
/// walk's `arena_enter_loop` sites (the `ForStmt|WhileStmt|DoWhileStmt` and
/// `ForOfStmt` arms in `walk.rs`) and the string-site walk's loop-ordinal
/// re-derivation (`string_site_walk`). `ForInStmt` is deliberately EXCLUDED:
/// codegen's `loop_preorder_ordinals` allowlist omits `"for-in"`, and both
/// numbering streams must skip it in lockstep or the ordinal streams desync
/// (see the `ForInStmt` arm's guardrail comment in `walk.rs`).
pub(crate) fn is_arena_ordinal_loop_kind(kind: &HirNodeKind) -> bool {
    matches!(
        kind,
        HirNodeKind::ForStmt
            | HirNodeKind::WhileStmt
            | HirNodeKind::DoWhileStmt
            | HirNodeKind::ForOfStmt
    )
}

/// Mutable accumulator threaded through [`OwnershipAnalyzer::string_site_walk`].
/// Bundles the two independent pre-order streams the walk maintains: the
/// string-site ordinal stream (`next_ordinal` → `local`, the 4b both-sides
/// oracle) and the loop-ordinal stream (`next_loop_ordinal` + `loop_stack`)
/// used only to credit each granted site's innermost enclosing loop into
/// `granted_loops` (the 4f `string_arena_loop` (T) input).
#[derive(Debug, Default)]
struct StringSiteWalk {
    /// Next string-site pre-order ordinal (the 4b/4c/4d oracle stream).
    next_ordinal: u32,
    /// Ordinals of proven iteration-local string sites.
    local: BTreeSet<u32>,
    /// Next loop pre-order ordinal (mirrors `arena_enter_loop`).
    next_loop_ordinal: u32,
    /// Enclosing arena-ordinal loops, innermost last.
    loop_stack: Vec<u32>,
    /// Loop ordinals crediting ≥1 granted (innermost) string site.
    granted_loops: BTreeSet<u32>,
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
    /// At least one allocation site has `Returned` fate (neither `Global` nor
    /// `ScopeLocal`: it flows out via a bare `return`). Such a site must NOT
    /// be routed into a per-call function arena that gets reset on exit, so
    /// this fact vetoes `opens_arena` even when a `ScopeLocal` site is also
    /// present (see `opens_arena` doc comment).
    pub has_returned_site: bool,
    /// Resolved bare-identifier call targets invoked in the function body.
    pub calls: BTreeSet<String>,
    /// The function contains a closure/indirect/unresolved or non-whitelisted
    /// host call (taints reaches-allocation and loop gating).
    pub has_unknown_call: bool,
    /// Per-loop facts in pre-order.
    pub loops: Vec<LoopArenaFacts>,
    /// Pre-order ordinals of the string-producing sites in this function
    /// (`.join(..)` calls and `+` binaries) proven iteration-local — every
    /// consumer drops or copies the result (see
    /// [`OwnershipAnalyzer::arena_collect_string_sites`]). Emptied when the
    /// function is name-collision poisoned.
    pub arena_string_sites: BTreeSet<u32>,
    /// Pre-order loop ordinals whose body contains a granted `arena_string_site`
    /// (crediting the innermost enclosing loop) — the (T) predicate for
    /// `string_arena_loop_qualifies` (fasta Spec 7 Task 4f). Emptied for
    /// name-collision-poisoned functions.
    pub string_arena_loops: BTreeSet<u32>,
}

// ---------------------------------------------------------------------------
// Raw collection state (lives on the analyzer during the walk).
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub(crate) struct FuncRaw {
    allocates: bool,
    has_global_site: bool,
    has_scope_local_site: bool,
    has_returned_site: bool,
    calls: BTreeSet<String>,
    has_unknown_call: bool,
    fresh_heap_bindings: BTreeSet<String>,
    loops: Vec<LoopArenaFacts>,
    /// Pre-order ordinals of proven iteration-local string sites (see
    /// [`OwnershipAnalyzer::arena_collect_string_sites`]).
    string_sites_local: BTreeSet<u32>,
    /// Pre-order loop ordinals whose body (excluding nested functions, and
    /// crediting the INNERMOST enclosing loop) contains ≥1 proven
    /// iteration-local string site — the (T) input to `string_arena_loop`
    /// (fasta Spec 7 Task 4f). Recorded by `arena_collect_string_sites`,
    /// re-deriving the loop-ordinal stream `arena_enter_loop` assigns. Emptied
    /// for poisoned (name-collided) functions, like `string_sites_local`.
    string_arena_loops: BTreeSet<u32>,
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
    pub(crate) fn into_facts(
        mut self,
        flow: &FlowCollector,
        solution: &FlowSolution,
    ) -> Vec<FunctionArenaFacts> {
        // Resolve the deferred veto sites against the fixpoint. Everything
        // here only ever ORs veto bits in — grants are untouched.
        for site in flow.global_sites() {
            if solution.class_may_heap(&site.class) {
                if let Some(raw) = self.functions.get_mut(&site.function) {
                    raw.has_global_site = true;
                }
            }
        }
        for site in flow.outflow_sites() {
            if solution.class_may_heap(&site.class) {
                if let Some(raw) = self.functions.get_mut(&site.function) {
                    if let Some(l) = raw.loops.iter_mut().find(|l| l.ordinal == site.ordinal) {
                        l.has_outflow = true;
                    }
                }
            }
        }
        // A may-heap argument handed to a param slot the callee stores
        // outward escapes the caller's frame AND every loop open at the call.
        for site in flow.arg_sites() {
            if solution.param_escapes(&site.callee, site.index)
                && solution.class_may_heap(&site.class)
            {
                if let Some(raw) = self.functions.get_mut(&site.function) {
                    raw.has_global_site = true;
                    for ordinal in &site.loop_ordinals {
                        if let Some(l) = raw.loops.iter_mut().find(|l| l.ordinal == *ordinal) {
                            l.has_outflow = true;
                        }
                    }
                }
            }
        }
        // A `return` whose value resolves may-heap under the fixpoint is a
        // Returned-fate site: it must veto `opens_arena` regardless of shape
        // (bare literal, name-bound literal, ternary, `new`, or a
        // transitively may-heap call-result). This generalizes round-1's two
        // shape-specific paths (`arena_is_fresh_literal` in
        // `arena_note_return`, `binding.returned` in
        // `arena_finalize_current_function`).
        for site in flow.returned_sites() {
            if solution.class_may_heap(&site.class) {
                if let Some(raw) = self.functions.get_mut(&site.function) {
                    raw.has_returned_site = true;
                }
            }
        }

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
                            has_returned_site: false,
                            calls: raw.calls.clone(),
                            has_unknown_call: true,
                            loops: Vec::new(),
                            // Poisoned: a name-keyed collision cannot tell the
                            // two instances' sites apart, so retain none.
                            arena_string_sites: BTreeSet::new(),
                            string_arena_loops: BTreeSet::new(),
                        };
                    }
                    FunctionArenaFacts {
                        name: label.clone(),
                        allocates: raw.allocates,
                        has_global_site: raw.has_global_site,
                        has_scope_local_site: raw.has_scope_local_site,
                        has_returned_site: raw.has_returned_site,
                        calls: raw.calls.clone(),
                        has_unknown_call: raw.has_unknown_call,
                        loops: raw.loops.clone(),
                        arena_string_sites: raw.string_sites_local.clone(),
                        string_arena_loops: raw.string_arena_loops.clone(),
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
            self.flow.poison_function(&label);
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
            let returned = !global && binding.returned;
            let scope_local = !global && !binding.returned;
            let f = self.arena.func(&label);
            if global {
                f.has_global_site = true;
            }
            if returned {
                f.has_returned_site = true;
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

    /// Classify a call expression's callee (and its argument literals).
    pub(crate) fn arena_note_call_expr(&mut self, children: &[HirNodeId]) {
        let Some(callee) = children.first().copied() else {
            return;
        };
        let callee_node = &self.nodes[callee.0 as usize];
        let mut whitelisted = false;
        let mut known_target: Option<String> = None;
        match callee_node.kind {
            HirNodeKind::Ident => {
                let name = callee_node.text.clone();
                match name
                    .as_deref()
                    .and_then(|n| self.resolve_function_target(n))
                {
                    Some(target) => {
                        self.arena_note_call(&target);
                        known_target = Some(target);
                    }
                    // Builtin `Array(n)` / `new Array(n)` allocation. The parser
                    // lowers `new Array(n)` to `NewExpr[ CallExpr[Array, n] ]`,
                    // so the array constructor surfaces here as an *unresolved*
                    // bare `Array` call. Codegen recognizes exactly this shape
                    // (`resolve_array_alloc_call`, callee text `"Array"` with no
                    // children) and lowers it to an array bump-allocation routed
                    // by `alloc_callee_index` — `__alloc_global` unless the
                    // enclosing function is `arena_eligible` — NOT a user-body
                    // call that could allocate-and-retain an argument into the
                    // current arena. Its escape is already modeled by
                    // outflow/eligibility (a `line = new Array(n)` reassignment
                    // to an outer binding still trips `has_outflow`), so it must
                    // NOT taint `has_unknown_call`. Left untreated it spuriously
                    // vetoes both `loop_arena` and `string_arena_loop` (fasta
                    // Spec 7 Task 4f: `fastaRandom`'s `while` reassigns
                    // `line = new Array(n)`, so without this correction its
                    // per-iteration string arena never opens and N=25M leaks —
                    // the investigation's Q5 census missed this taint). Not
                    // added to `calls` either, so `reaches_alloc_transitively`
                    // is unaffected. Mirrors the `substring`/`join` non-tainting
                    // member-call cases below.
                    None if name.as_deref() == Some("Array") => {}
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
                } else if method == "substring" {
                    // Pure-ALU slice: retains nothing, poisons nothing. The
                    // result's aliasing is modeled by `classify_value`
                    // (escape_flow.rs), not by arena unknown-call taint.
                } else if method == "join" {
                    // Runtime `join` copies element bytes into a fresh
                    // `__alloc_global` string: retains nothing from the
                    // receiver's arena, poisons nothing. The result's
                    // scalar-ness is modeled by `classify_value`
                    // (escape_flow.rs), not by arena unknown-call taint.
                } else {
                    self.arena_note_unknown_call();
                }
            }
            // IIFEs and every other computed/indirect callee ⇒ taint.
            _ => self.arena_note_unknown_call(),
        }

        // Whitelisted host calls consume their arguments (never retain).
        if whitelisted {
            return;
        }
        let function = self.current_scope_label();
        let open_ordinals: Vec<u32> = self.arena.loop_stack.iter().map(|l| l.ordinal).collect();
        for (index, arg) in children.iter().skip(1).copied().enumerate() {
            let class = self.classify_value(arg);
            if class.is_scalar() {
                continue;
            }
            match &known_target {
                Some(target) => {
                    // The value flows into the callee's positional param slot;
                    // the veto (if any) is decided by the callee's summary.
                    self.flow.note_value_into(
                        crate::analysis::escape_flow::FlowNode::Param {
                            function: target.clone(),
                            index,
                        },
                        &class,
                    );
                    self.flow
                        .push_arg_site(&function, target, index, class, open_ordinals.clone());
                }
                None => {
                    // Unknown callee: assume it retains every argument.
                    self.flow.note_taint_class(&class);
                    self.flow.push_global_site(&function, class);
                }
            }
        }
    }

    /// Classify an assignment for global-site (function) and outflow (loop).
    pub(crate) fn arena_note_assignment(&mut self, left: HirNodeId, right: HirNodeId) {
        let left_node = &self.nodes[left.0 as usize];
        let left_kind = left_node.kind.clone();

        match left_kind {
            HirNodeKind::Ident => {
                let name = left_node.text.clone().unwrap_or_default();
                let class = self.classify_value(right);
                let function = self.current_scope_label();
                if let Some((scope_index, _)) = self.resolve_binding(&name) {
                    let owner = self
                        .scope_stack
                        .get(scope_index)
                        .map(|scope| scope.label.clone())
                        .unwrap_or_else(|| "<module>".to_string());
                    let target = crate::analysis::escape_flow::FlowNode::Binding {
                        owner,
                        name: name.clone(),
                    };
                    self.flow.note_value_into(target.clone(), &class);
                    if scope_index < self.current_scope_index() {
                        self.flow.note_taint_node(target);
                        self.flow.push_global_site(&function, class.clone());
                    } else if self.arena_is_fresh_literal(right) {
                        self.arena_note_fresh_binding(&name);
                    }
                } else {
                    self.flow.push_global_site(&function, class.clone());
                }
                let outflow_ordinals: Vec<u32> = self
                    .arena
                    .loop_stack
                    .iter()
                    .filter(|l| !l.inner_bindings.contains(&name))
                    .map(|l| l.ordinal)
                    .collect();
                for ordinal in outflow_ordinals {
                    self.flow.push_outflow(&function, ordinal, class.clone());
                }
            }
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr => {
                let class = self.classify_value(right);
                let function = self.current_scope_label();
                self.flow.push_global_site(&function, class.clone());
                self.flow.note_taint_class(&class);
                let base_name = left_node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .filter(|base| base.kind == HirNodeKind::Ident)
                    .and_then(|base| base.text.clone());
                let outflow_ordinals: Vec<u32> = self
                    .arena
                    .loop_stack
                    .iter()
                    .filter(|l| match &base_name {
                        Some(base) => !l.inner_bindings.contains(base),
                        None => true,
                    })
                    .map(|l| l.ordinal)
                    .collect();
                for ordinal in outflow_ordinals {
                    self.flow.push_outflow(&function, ordinal, class.clone());
                }
            }
            _ => {
                let class = self.classify_value(right);
                let function = self.current_scope_label();
                self.flow.push_global_site(&function, class.clone());
                self.flow.note_taint_class(&class);
                let all_ordinals: Vec<u32> =
                    self.arena.loop_stack.iter().map(|l| l.ordinal).collect();
                for ordinal in all_ordinals {
                    self.flow.push_outflow(&function, ordinal, class.clone());
                }
            }
        }
    }

    /// A `return` of a heap value from inside a loop leaks it out of the
    /// arena; a `return` of a heap value anywhere in the function is a
    /// Returned-fate site (deferred: resolved against the escape-flow
    /// fixpoint in `ArenaCollector::into_facts` via `push_returned_site`,
    /// which is what lets `return factory()` — a call-result — resolve
    /// correctly through transitive may-heap chains, not just the two
    /// shape-specific literal forms round 1 detected).
    pub(crate) fn arena_note_return(&mut self, children: &[HirNodeId]) {
        let function = self.current_scope_label();
        let mut class = ValueClass::Scalar;
        for child in children {
            class = class.join(self.classify_value(*child));
        }
        self.flow.note_value_into(
            crate::analysis::escape_flow::FlowNode::Return {
                function: function.clone(),
            },
            &class,
        );
        self.flow.push_returned_site(&function, class.clone());
        let ordinals: Vec<u32> = self.arena.loop_stack.iter().map(|l| l.ordinal).collect();
        for ordinal in ordinals {
            self.flow.push_outflow(&function, ordinal, class.clone());
        }
    }

    fn arena_is_fresh_literal(&self, node_id: HirNodeId) -> bool {
        matches!(
            self.nodes[node_id.0 as usize].kind,
            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr
        )
    }

    // -----------------------------------------------------------------------
    // Per-string-site iteration-locality (the parallel string-site channel).
    // -----------------------------------------------------------------------

    /// Assign each string-producing site in the current function's `body` a
    /// pre-order ordinal and record those proven iteration-local onto the
    /// function's raw facts. Called once per function scope from `walk.rs`
    /// (the FunctionDecl/FunctionExpr arms), while the current scope IS that
    /// function — so `func(label)` keys the right entry (and a poisoned,
    /// name-collided entry drops these at `into_facts`, just like loops).
    ///
    /// ## THE STRING-SITE ORDINAL RULE (both-sides oracle — 4c/4d MUST mirror)
    ///
    /// A **string-producing site** is a HIR node that is EITHER:
    ///   1. a `CallExpr` whose callee (first child) is a `MemberExpr` with
    ///      method text `"join"` (`recv.join(sep)`), OR
    ///   2. a `BinaryExpr` with operator text `"+"` (ANY `+`; the walk does
    ///      NOT distinguish string concat from numeric add — see below).
    ///
    /// Sites are numbered **pre-order, function-body-scoped**: the ordinal is
    /// assigned when the walk first reaches the node, BEFORE descending into
    /// its children (so a `+` is numbered before its operands), counting up
    /// from 0 per function. Nested function-like subtrees (`FunctionDecl` /
    /// `FunctionExpr`) are opaque leaves — never descended into; each function
    /// numbers its own sites independently. This is the exact discipline of
    /// `kali_codegen::lower::loop_preorder_ordinals` for loops; 4c's codegen
    /// ordinal walk over the matching LIR nodes MUST enumerate the same node
    /// set in the same pre-order, or `arena_string_site(fn, ord)` lookups
    /// desync and misroute — a fail-OPEN hazard identical to the for-in
    /// loop-ordinal guardrail. Numeric `+` sites are numbered too (they cost
    /// an ordinal slot but are never QUERIED by codegen's numeric path); this
    /// keeps the site set purely syntactic so both sides agree by construction
    /// WITHOUT duplicating string-type inference into `kali_mir`. Template
    /// literals are deliberately NOT sites here (out of 4b's scope); 4c/4d
    /// must not number them either.
    ///
    /// ## LOCALITY (default-deny)
    ///
    /// A site is iteration-local iff its immediate syntactic consumer (its
    /// parent) DROPS or COPIES the result:
    ///
    /// - DROP: the parent is a whitelisted host call
    ///   (`console.*` / `Kali.writeStdoutBytes`, via
    ///   [`is_whitelisted_host_method`]) and the site sits in ARGUMENT position
    ///   (not the callee slot) — the host serializes and never retains.
    /// - COPY: the parent is a `+` `BinaryExpr` and the site is an operand —
    ///   `+` copies the operand's bytes into a fresh string, so the operand's
    ///   backing dies within the iteration.
    ///
    /// EVERY other consumer vetoes (returned, assigned, bound to a declarator,
    /// stored into a member/element, handed to a user/unknown callee, used as a
    /// join separator, a bare expression statement, ...) — the result may
    /// outlive the iteration, so it stays on the global heap. Vetoing is always
    /// sound; the only cost is unreclaimed memory.
    pub(crate) fn arena_collect_string_sites(&mut self, body: HirNodeId) {
        let mut acc = StringSiteWalk::default();
        // The function root is a veto (non-drop/copy) context.
        self.string_site_walk(body, false, &mut acc);
        if acc.local.is_empty() && acc.granted_loops.is_empty() {
            return;
        }
        let label = self.current_scope_label();
        let raw = self.arena.func(&label);
        raw.string_sites_local.extend(acc.local);
        raw.string_arena_loops.extend(acc.granted_loops);
    }

    /// Pre-order string-site walk. `consumer_drops_or_copies` is the locality
    /// context THIS node inherits from its parent; it governs only whether
    /// this node — if it is a site — is recorded local. The context handed to
    /// each child depends solely on THIS node's kind (a host-call passes DROP
    /// to its args, a `+` passes COPY to its operands, everything else passes
    /// VETO), never on the incoming context.
    ///
    /// ## LOOP CORRELATION (fasta Spec 7 Task 4f)
    ///
    /// This walk ALSO re-derives the pre-order loop-ordinal stream and records,
    /// for every GRANTED site, its INNERMOST enclosing loop ordinal
    /// (`acc.granted_loops`). The re-derivation MUST assign the same ordinals as
    /// the ownership walk's `arena_enter_loop` — count `for`/`while`/`do-while`/
    /// `for-of` (via [`is_arena_ordinal_loop_kind`], the single-sourced
    /// predicate the two `arena_enter_loop` arms in `walk.rs` encode), skip
    /// `for-in`, treat nested functions as opaque leaves — or the site→loop
    /// correlation desyncs from codegen's `loop_preorder_ordinals` (a fail-OPEN
    /// use-after-reset). This walk runs at function-scope entry, BEFORE the
    /// ownership walk numbers the loops, so the live `arena.loop_stack` is not
    /// yet populated and cannot be reused — hence the local re-derivation.
    fn string_site_walk(
        &self,
        node_id: HirNodeId,
        consumer_drops_or_copies: bool,
        acc: &mut StringSiteWalk,
    ) {
        let node = &self.nodes[node_id.0 as usize];
        // A nested function owns its own ordinal stream — opaque leaf here (for
        // both the string-site and the loop-ordinal streams).
        if matches!(
            node.kind,
            HirNodeKind::FunctionDecl | HirNodeKind::FunctionExpr
        ) {
            return;
        }
        if self.is_string_producing_site(node_id) {
            let ordinal = acc.next_ordinal;
            acc.next_ordinal += 1;
            if consumer_drops_or_copies {
                acc.local.insert(ordinal);
                // (T): credit the innermost enclosing (arena-ordinal) loop, if
                // any. A granted site outside every loop belongs to none.
                if let Some(&inner) = acc.loop_stack.last() {
                    acc.granted_loops.insert(inner);
                }
            }
        }
        // Assign this node its loop ordinal (if it is an arena-ordinal loop)
        // and push it as the new innermost context for its subtree.
        let is_loop = is_arena_ordinal_loop_kind(&node.kind);
        if is_loop {
            let loop_ordinal = acc.next_loop_ordinal;
            acc.next_loop_ordinal += 1;
            acc.loop_stack.push(loop_ordinal);
        }
        // How this node passes locality context down to its children.
        let is_plus = node.kind == HirNodeKind::BinaryExpr && node.text.as_deref() == Some("+");
        let is_host_call = self.is_whitelisted_host_call(node_id);
        let children = node.children.clone();
        for (index, child) in children.iter().enumerate() {
            // COPY for every `+` operand; DROP for host-call args (index >= 1,
            // never the callee at index 0); VETO otherwise.
            let child_ctx = is_plus || (is_host_call && index >= 1);
            self.string_site_walk(*child, child_ctx, acc);
        }
        if is_loop {
            acc.loop_stack.pop();
        }
    }

    /// Whether `node_id` is a string-producing site (see the ordinal rule on
    /// [`OwnershipAnalyzer::arena_collect_string_sites`]). Mirrors the `"join"`
    /// / `"+"` recognizers in `classify_value` (escape_flow.rs).
    fn is_string_producing_site(&self, node_id: HirNodeId) -> bool {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::CallExpr => node
                .children
                .first()
                .map(|id| &self.nodes[id.0 as usize])
                .is_some_and(|callee| {
                    callee.kind == HirNodeKind::MemberExpr && callee.text.as_deref() == Some("join")
                }),
            HirNodeKind::BinaryExpr => node.text.as_deref() == Some("+"),
            _ => false,
        }
    }

    /// Whether `node_id` is a `CallExpr` on a whitelisted, non-retaining host
    /// method (`console.*` / `Kali.writeStdoutBytes`). Reuses the exact
    /// receiver/method extraction and [`is_whitelisted_host_method`] recognizer
    /// used by `arena_note_call_expr`, so the drop-sink set is single-sourced.
    fn is_whitelisted_host_call(&self, node_id: HirNodeId) -> bool {
        let node = &self.nodes[node_id.0 as usize];
        if node.kind != HirNodeKind::CallExpr {
            return false;
        }
        let Some(callee) = node.children.first().map(|id| &self.nodes[id.0 as usize]) else {
            return false;
        };
        if callee.kind != HirNodeKind::MemberExpr {
            return false;
        }
        let method = callee.text.clone().unwrap_or_default();
        let base_object = callee
            .children
            .first()
            .map(|id| &self.nodes[id.0 as usize])
            .filter(|base| base.kind == HirNodeKind::Ident)
            .and_then(|base| base.text.clone());
        is_whitelisted_host_method(base_object.as_deref(), &method)
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
/// 2. `opens_arena(f)` ⇔ `arena_eligible(f)`, `f` has a `ScopeLocal` site, and
///    `f` has NO `Returned`-fate site. A function-body arena is reset on
///    every exit path; a `Returned` site's value must survive past that
///    reset into the caller, so ANY `Returned` site vetoes opening a
///    function arena for `f` even when a `ScopeLocal` site is also present
///    (every fresh-heap site in `f` currently routes into the same current
///    arena, so a mixed function cannot open one without also resetting the
///    returned value's backing page). `f` stays `arena_eligible` — its sites
///    still target the *caller's* current arena — only the per-call function
///    arena it would otherwise open for itself is withheld.
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
            if facts.has_scope_local_site && !facts.has_returned_site {
                table.set_opens_arena(&facts.name);
            }
        }

        for loop_facts in &facts.loops {
            if loop_arena_qualifies(loop_facts, &facts_by_name, &eligible) {
                table.set_loop_arena(&facts.name, loop_facts.ordinal);
            }
            // Independent emit-only channel (fasta Spec 7 Task 4f): open a
            // per-iteration arena for a loop whose only reclaimable allocation
            // is a granted string site. This sets NO routing fact and does NOT
            // set `loop_arena`; codegen OR-s both getters into one single-open
            // `is_arena_loop` decision.
            if string_arena_loop_qualifies(
                &facts.name,
                loop_facts,
                &facts.string_arena_loops,
                &facts_by_name,
                &eligible,
            ) {
                table.set_string_arena_loop(&facts.name, loop_facts.ordinal);
            }
        }

        // Parallel string-site channel: record every proven iteration-local
        // site (already emptied for poisoned functions at `into_facts`). This
        // is behavior-neutral until codegen (4c/4d) reads the channel.
        for ordinal in &facts.arena_string_sites {
            table.set_arena_string_site(&facts.name, *ordinal);
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

/// Whether the loop qualifies for the emit-only `string_arena_loop` channel
/// (fasta Spec 7 Task 4f) — it opens a per-iteration arena SOLELY because its
/// body contains a granted `arena_string_site`. All of (T) + (V1) + (V2) + (V3)
/// must hold. `has_outflow` is DELIBERATELY not consulted: outflow of a
/// `__alloc_global`-routed value is invisible to a string-only arena, and 4b's
/// default-deny grant means a granted string value can never outflow.
fn string_arena_loop_qualifies(
    function: &str,
    loop_facts: &LoopArenaFacts,
    string_arena_loops: &BTreeSet<u32>,
    facts_by_name: &BTreeMap<&str, &FunctionArenaFacts>,
    eligible: &impl Fn(&str) -> bool,
) -> bool {
    // (T) the loop body (crediting the innermost enclosing loop) contains ≥1
    // granted string site.
    if !string_arena_loops.contains(&loop_facts.ordinal) {
        return false;
    }
    // (V1) the enclosing function is NOT `arena_eligible`. If it were, its
    // object/array sites route to `__alloc` and this loop arena could capture
    // an unproven object; such functions keep `loop_arena` as their only arena
    // path.
    if eligible(function) {
        return false;
    }
    // (V2) no unknown/closure/indirect or non-whitelisted host call in the loop
    // body — an unknown callee could allocate-and-retain via `__alloc`. Same
    // veto `loop_arena_qualifies` consults.
    if loop_facts.has_unknown_call {
        return false;
    }
    // (V3) no KNOWN callee reachable from the loop body may allocate. Unlike
    // `loop_arena_qualifies` (which permits arena-eligible allocating callees,
    // because an object arena captures their objects safely), a string-only
    // arena must capture NO non-string `__alloc` at all, so ANY reachable
    // allocation vetoes. Unknown target ⇒ may-allocate (fail closed in the
    // helper).
    for target in &loop_facts.calls {
        if reaches_alloc_transitively(target, facts_by_name, &mut BTreeSet::new()) {
            return false;
        }
    }

    true
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
