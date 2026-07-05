# Interprocedural Escape Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the arena gate's walk-order-sensitive heap judgment and the ownership engine's plain-ident escape blindness with one shared interprocedural dataflow pass (`escape_flow.rs`), flipping both xfail launder pins green.

**Architecture:** The ownership walk stops resolving heap-ness mid-walk; instead it records tri-state `ValueClass` judgments, flow edges (assignments, param binding, returns), and deferred veto sites into a `FlowCollector`. After the walk, one monotone worklist fixpoint (`solve`) computes may-heap (forward) and escape taint (backward) over all nodes, order-independently. Two consumers read the `FlowSolution`: `ArenaCollector::into_facts` resolves the deferred sites into the existing `FunctionArenaFacts` booleans (so `compute_arena_table` is untouched), and a post-pass flips `MirBinding.escapes` for bindings the fixpoint proves stored-outward (fixing `sink = p`).

**Tech Stack:** Rust, `kali_mir` crate only (verified: `escapes`/`OwnershipClass` have zero consumers outside `kali_mir`; LIR lowers only the structural node tree, so codegen output is unaffected).

**Spec:** `docs/superpowers/specs/2026-07-05-interprocedural-escape-flow-design.md`
**Root cause report:** `.superpowers/sdd/task-4-report.md` rounds 4–5b.

## Global Constraints

- GC-less invariant: this is escape analysis for region reclamation; never introduce tracing/GC concepts.
- Every ambiguity fails closed (toward veto / toward heap / toward escapes). Vetoing costs unreclaimed memory; a wrong grant is a use-after-reset miscompile.
- All new containers are `BTreeMap`/`BTreeSet` (determinism — matches existing collector convention).
- The round-3 asymmetry is preserved: call results / laundered values feed the VETO side (may-heap), never the GRANT side (`fresh_heap_bindings` fate classification stays literal-only).
- `compute_arena_table` and the walk's `UseContext` logic (caller-side escape contexts in the `CallExpr` arm) are NOT modified.
- Acceptance: both `#[ignore]` pins in `arena_gate_tests.rs` flip green with ALL assertions; the 24 existing gate pins pass unchanged; full `cargo test -p kali_mir` green; 5-crate gate (`cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`) exit 0; `cargo fmt --check` clean.
- Run all test/gate commands in the FOREGROUND (never backgrounded — standing SDD process rule).
- Work on branch `interproc-escape-flow` off `main`.

## File Structure

- Create: `crates/kali_mir/src/analysis/escape_flow.rs` — `FlowNode`, `ValueClass`, `FlowCollector` (edges/seeds/deferred sites), `FlowSolution`, `solve()`, `classify_value` impl on the analyzer, engine post-pass `apply_escape_verdicts`.
- Create: `crates/kali_mir/src/analysis/escape_flow_tests.rs` — pure fixpoint unit tests + solution-level tests on real sources.
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/plain_ident_escape.rs` — engine-level pins for the fixed `escapes` verdicts.
- Modify: `crates/kali_mir/src/analysis/mod.rs` — register module, add `flow` field to `OwnershipAnalyzer`, run solve + post-pass in `analyze_program_with_arena`.
- Modify: `crates/kali_mir/src/analysis/walk.rs` — record declarator edges and param nodes.
- Modify: `crates/kali_mir/src/analysis/arena_gate.rs` — rewrite the three note hooks to record classes/sites; `into_facts(flow, solution)`; delete `arena_is_heap_value`, `maybe_heap_bindings`, `arena_note_maybe_heap_binding`.
- Modify: `crates/kali_mir/src/analysis/arena_gate_tests.rs` — un-ignore the two pins; add hardening pins.
- Modify: `crates/kali_mir/src/analysis/ownership_analysis_tests.rs` — register the new test module.

Semantics cheat-sheet used throughout (from the approved spec):

- **may-heap (forward):** a node may hold a heap value. Seeds: `Heap`-class assignments/returns, ALL `Param` nodes (a function may be called from contexts the graph can't see — fail closed). Propagates along edges `from → to`.
- **escape taint (backward):** values reaching this node may be retained beyond the storing function's dynamic extent. Seeds: cross-scope assignment TARGETS (module or enclosing-function bindings stored into from a nested function), sources stored into object fields / passed to unknown callees. Propagates along REVERSED edges. Returns are deliberately NOT taint seeds (`returned` is already handled by the engine; a returned value's fate is the caller's site classification — that is what keeps `factory()` patterns and `itemCheck(bottomUpTree(d))` eligible).
- **deferred sites:** `has_global_site` / `loops[].has_outflow` marks recorded during the walk with a `ValueClass` instead of a resolved boolean; resolved against the solution in `into_facts`. Arg sites additionally gate on `param_escapes(callee, index)`.

---

### Task 1: `escape_flow.rs` core — nodes, classes, collector, fixpoint

**Files:**
- Create: `crates/kali_mir/src/analysis/escape_flow.rs`
- Create: `crates/kali_mir/src/analysis/escape_flow_tests.rs`
- Modify: `crates/kali_mir/src/analysis/mod.rs` (add `mod escape_flow;`)

**Interfaces:**
- Consumes: nothing (pure data structures + algorithm).
- Produces (used by Tasks 2–4):
  - `FlowNode::{Binding{owner: String, name: String}, Param{function: String, index: usize}, Return{function: String}}`
  - `ValueClass::{Scalar, Heap(BTreeSet<FlowNode>), DependsOn(BTreeSet<FlowNode>)}` with `join(self, other) -> ValueClass`, `is_scalar(&self) -> bool`, `take_nodes(self) -> BTreeSet<FlowNode>`
  - `FlowCollector` with `note_edge`, `note_value_into(target: FlowNode, class: &ValueClass)`, `note_taint_class(class: &ValueClass)`, `note_taint_node(node: FlowNode)`, `note_param(function: &str, index: usize, name: &str)`, `poison_function(name: &str)`, `push_global_site(function: &str, class: ValueClass)`, `push_outflow(function: &str, ordinal: u32, class: ValueClass)`, `push_arg_site(function: &str, callee: &str, index: usize, class: ValueClass, loop_ordinals: Vec<u32>)`, plus read accessors `global_sites()`, `outflow_sites()`, `arg_sites()`
  - `FlowSolution` with `class_may_heap(&ValueClass) -> bool`, `param_escapes(function: &str, index: usize) -> bool`, `binding_escapes(owner: &str, name: &str) -> bool`
  - `pub(crate) fn solve(flow: &FlowCollector) -> FlowSolution`

- [ ] **Step 1: Write the failing unit tests**

Create `crates/kali_mir/src/analysis/escape_flow_tests.rs`:

```rust
//! Unit tests for the interprocedural escape-flow fixpoint (pure graph level).

use std::collections::BTreeSet;

use super::*;

fn binding(owner: &str, name: &str) -> FlowNode {
    FlowNode::Binding {
        owner: owner.to_string(),
        name: name.to_string(),
    }
}

fn depends_on(nodes: &[FlowNode]) -> ValueClass {
    ValueClass::DependsOn(nodes.iter().cloned().collect())
}

#[test]
fn may_heap_propagates_forward_through_assign_chain() {
    // x <- Heap; y <- x; z <- y  =>  all three may-heap.
    let mut flow = FlowCollector::default();
    flow.note_value_into(binding("f", "x"), &ValueClass::Heap(BTreeSet::new()));
    flow.note_value_into(binding("f", "y"), &depends_on(&[binding("f", "x")]));
    flow.note_value_into(binding("f", "z"), &depends_on(&[binding("f", "y")]));
    let solution = solve(&flow);
    assert!(solution.class_may_heap(&depends_on(&[binding("f", "z")])));
    assert!(!solution.class_may_heap(&depends_on(&[binding("f", "unrelated")])));
    assert!(!solution.class_may_heap(&ValueClass::Scalar));
    assert!(solution.class_may_heap(&ValueClass::Heap(BTreeSet::new())));
}

#[test]
fn taint_propagates_backward_through_assign_chain() {
    // a -> b -> cache(tainted)  =>  a and b tainted.
    let mut flow = FlowCollector::default();
    flow.note_value_into(binding("f", "b"), &depends_on(&[binding("f", "a")]));
    flow.note_value_into(binding("<module>", "cache"), &depends_on(&[binding("f", "b")]));
    flow.note_taint_node(binding("<module>", "cache"));
    let solution = solve(&flow);
    assert!(solution.binding_escapes("f", "a"));
    assert!(solution.binding_escapes("f", "b"));
    assert!(!solution.binding_escapes("f", "other"));
}

#[test]
fn param_nodes_seed_may_heap_and_link_to_param_binding() {
    // note_param registers Param{f,0} -> Binding{f,p}; params are may-heap.
    let mut flow = FlowCollector::default();
    flow.note_param("f", 0, "p");
    let solution = solve(&flow);
    assert!(solution.class_may_heap(&depends_on(&[binding("f", "p")])));
}

#[test]
fn param_escapes_via_taint_on_param_binding() {
    // retain(p) { sink = p; }: Param{retain,0} -> p -> sink(tainted).
    let mut flow = FlowCollector::default();
    flow.note_param("retain", 0, "p");
    flow.note_value_into(
        binding("<module>", "sink"),
        &depends_on(&[binding("retain", "p")]),
    );
    flow.note_taint_node(binding("<module>", "sink"));
    let solution = solve(&flow);
    assert!(solution.param_escapes("retain", 0));
    assert!(!solution.param_escapes("retain", 1));
    assert!(!solution.param_escapes("other_fn", 0));
}

#[test]
fn poisoned_function_params_always_escape() {
    let mut flow = FlowCollector::default();
    flow.poison_function("h");
    let solution = solve(&flow);
    assert!(solution.param_escapes("h", 0));
    assert!(solution.param_escapes("h", 7));
}

#[test]
fn cycles_terminate_and_stay_sound() {
    // a <-> b cycle with a tainted heap source entering it.
    let mut flow = FlowCollector::default();
    flow.note_value_into(binding("f", "a"), &depends_on(&[binding("f", "b")]));
    flow.note_value_into(binding("f", "b"), &depends_on(&[binding("f", "a")]));
    flow.note_value_into(binding("f", "a"), &ValueClass::Heap(BTreeSet::new()));
    flow.note_value_into(binding("<module>", "g"), &depends_on(&[binding("f", "b")]));
    flow.note_taint_node(binding("<module>", "g"));
    let solution = solve(&flow);
    assert!(solution.class_may_heap(&depends_on(&[binding("f", "b")])));
    assert!(solution.binding_escapes("f", "a"));
}

#[test]
fn value_class_join_lattice() {
    let heap_with = |nodes: &[FlowNode]| ValueClass::Heap(nodes.iter().cloned().collect());
    let a = binding("f", "a");
    let b = binding("f", "b");
    // Scalar is the identity.
    assert_eq!(
        ValueClass::Scalar.join(depends_on(&[a.clone()])),
        depends_on(&[a.clone()])
    );
    assert_eq!(ValueClass::Scalar.join(ValueClass::Scalar), ValueClass::Scalar);
    // Heap absorbs, unioning source nodes.
    assert_eq!(
        heap_with(&[a.clone()]).join(depends_on(&[b.clone()])),
        heap_with(&[a.clone(), b.clone()])
    );
    assert_eq!(
        depends_on(&[a.clone()]).join(depends_on(&[b.clone()])),
        depends_on(&[a, b])
    );
}

#[test]
fn insertion_order_does_not_change_the_solution() {
    // Same graph, edges recorded in opposite orders: identical verdicts.
    let build = |reverse: bool| {
        let mut flow = FlowCollector::default();
        let mut notes: Vec<(FlowNode, ValueClass)> = vec![
            (binding("f", "b"), depends_on(&[binding("f", "a")])),
            (binding("f", "c"), depends_on(&[binding("f", "b")])),
            (binding("f", "a"), ValueClass::Heap(BTreeSet::new())),
            (binding("<module>", "g"), depends_on(&[binding("f", "c")])),
        ];
        if reverse {
            notes.reverse();
        }
        for (target, class) in notes {
            flow.note_value_into(target, &class);
        }
        flow.note_taint_node(binding("<module>", "g"));
        solve(&flow)
    };
    let forward = build(false);
    let backward = build(true);
    for name in ["a", "b", "c"] {
        assert_eq!(
            forward.binding_escapes("f", name),
            backward.binding_escapes("f", name)
        );
        assert_eq!(
            forward.class_may_heap(&depends_on(&[binding("f", name)])),
            backward.class_may_heap(&depends_on(&[binding("f", name)]))
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cargo test -p kali_mir escape_flow 2>&1 | tail -5`
Expected: compile error — `escape_flow` module does not exist.

- [ ] **Step 3: Implement the module**

Create `crates/kali_mir/src/analysis/escape_flow.rs`:

```rust
//! Interprocedural escape-flow analysis: the shared, walk-order-independent
//! answer to "may this value be heap?" and "is this value retained beyond a
//! dynamic extent?".
//!
//! The ownership walk RECORDS tri-state value judgments ([`ValueClass`]),
//! plain-ident/param/return flow edges, and deferred veto sites into a
//! [`FlowCollector`] instead of resolving heap-ness against possibly-stale
//! layouts mid-walk. [`solve`] then runs one monotone worklist fixpoint:
//! may-heap propagates FORWARD along value flow; escape taint propagates
//! BACKWARD (if `b = a` and `b`'s contents escape, `a`'s do too). Bits only
//! flip false→true, so termination is structural and the solution cannot
//! depend on walk order — this is what closes the hoisted-function launder.
//!
//! Two consumers: `ArenaCollector::into_facts` resolves the deferred sites
//! (gate vetoes), and `apply_escape_verdicts` feeds `MirBinding.escapes`
//! (fixing the engine's `sink = p` param-escape blindness).
//!
//! Every unknown fails closed: unresolved identifiers and unknown callees
//! classify as heap; unknown callees taint their arguments; params seed
//! may-heap unconditionally (a function may be called from contexts the
//! graph cannot see); name-collided functions poison their param summaries.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A dataflow node. Bindings are keyed (owner function label, name) — the
/// same name-keyed granularity as the rest of the gate, with the same
/// collision conservatism (see [`FlowCollector::poison_function`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FlowNode {
    Binding { owner: String, name: String },
    /// Positional param slot: call-site arg edges land here even when the
    /// callee's declaration has not been walked yet (hoisting).
    Param { function: String, index: usize },
    Return { function: String },
}

/// Tri-state RHS judgment. `Heap(embeds)` carries the identifier sources
/// embedded in the value (literal property values, member-read bases, ...)
/// so structures keep the identity of their contents for taint purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueClass {
    Scalar,
    Heap(BTreeSet<FlowNode>),
    DependsOn(BTreeSet<FlowNode>),
}

impl ValueClass {
    pub(crate) fn heap() -> Self {
        ValueClass::Heap(BTreeSet::new())
    }

    pub(crate) fn is_scalar(&self) -> bool {
        matches!(self, ValueClass::Scalar)
    }

    pub(crate) fn take_nodes(self) -> BTreeSet<FlowNode> {
        match self {
            ValueClass::Scalar => BTreeSet::new(),
            ValueClass::Heap(nodes) | ValueClass::DependsOn(nodes) => nodes,
        }
    }

    /// Join for branch-producing positions (ternary, logical, sequence):
    /// Heap absorbs (a possibly-heap branch makes the whole expression
    /// possibly-heap), node sets union.
    pub(crate) fn join(self, other: ValueClass) -> ValueClass {
        match (self, other) {
            (ValueClass::Scalar, x) | (x, ValueClass::Scalar) => x,
            (ValueClass::Heap(a), ValueClass::Heap(b))
            | (ValueClass::Heap(a), ValueClass::DependsOn(b))
            | (ValueClass::DependsOn(b), ValueClass::Heap(a)) => {
                ValueClass::Heap(a.into_iter().chain(b).collect())
            }
            (ValueClass::DependsOn(a), ValueClass::DependsOn(b)) => {
                ValueClass::DependsOn(a.into_iter().chain(b).collect())
            }
        }
    }
}

/// Deferred "this function has a Global-fate site if the stored value is
/// may-heap" record.
#[derive(Debug, Clone)]
pub(crate) struct GlobalSiteIf {
    pub(crate) function: String,
    pub(crate) class: ValueClass,
}

/// Deferred "this loop has outflow if the value is may-heap" record.
#[derive(Debug, Clone)]
pub(crate) struct OutflowIf {
    pub(crate) function: String,
    pub(crate) ordinal: u32,
    pub(crate) class: ValueClass,
}

/// Deferred call-argument record: fires (global site + outflow for the open
/// loops) iff the callee's param summary says the slot escapes AND the
/// argument is may-heap.
#[derive(Debug, Clone)]
pub(crate) struct ArgEscapeIf {
    pub(crate) function: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) class: ValueClass,
    pub(crate) loop_ordinals: Vec<u32>,
}

/// Everything the walk records for the fixpoint. All containers are ordered
/// (determinism).
#[derive(Debug, Default)]
pub(crate) struct FlowCollector {
    /// Value-flow edges: the value of `from` may become the value of `to`.
    edges: BTreeSet<(FlowNode, FlowNode)>,
    /// Nodes assigned a definitely-heap value.
    heap_seeds: BTreeSet<FlowNode>,
    /// Nodes whose contents are retained beyond a dynamic extent.
    taint_seeds: BTreeSet<FlowNode>,
    /// Every registered param slot (seeded may-heap: fail closed).
    param_nodes: BTreeSet<FlowNode>,
    /// Name-collided functions: param summaries are worst-cased.
    poisoned_functions: BTreeSet<String>,
    global_sites: Vec<GlobalSiteIf>,
    outflow_sites: Vec<OutflowIf>,
    arg_sites: Vec<ArgEscapeIf>,
}

impl FlowCollector {
    pub(crate) fn note_edge(&mut self, from: FlowNode, to: FlowNode) {
        self.edges.insert((from, to));
    }

    /// Record that `class`'s value flows into `target`.
    pub(crate) fn note_value_into(&mut self, target: FlowNode, class: &ValueClass) {
        match class {
            ValueClass::Scalar => {}
            ValueClass::Heap(nodes) => {
                self.heap_seeds.insert(target.clone());
                for node in nodes {
                    self.note_edge(node.clone(), target.clone());
                }
            }
            ValueClass::DependsOn(nodes) => {
                for node in nodes {
                    self.note_edge(node.clone(), target.clone());
                }
            }
        }
    }

    /// The value's sources escape directly (stored into an object field, or
    /// handed to an unknown callee): taint them without a target node.
    pub(crate) fn note_taint_class(&mut self, class: &ValueClass) {
        match class {
            ValueClass::Scalar => {}
            ValueClass::Heap(nodes) | ValueClass::DependsOn(nodes) => {
                self.taint_seeds.extend(nodes.iter().cloned());
            }
        }
    }

    pub(crate) fn note_taint_node(&mut self, node: FlowNode) {
        self.taint_seeds.insert(node);
    }

    /// Register param slot `index`/`name` of `function`, linking the
    /// positional node call sites target to the named binding the body uses.
    pub(crate) fn note_param(&mut self, function: &str, index: usize, name: &str) {
        let param = FlowNode::Param {
            function: function.to_string(),
            index,
        };
        self.param_nodes.insert(param.clone());
        self.note_edge(
            param,
            FlowNode::Binding {
                owner: function.to_string(),
                name: name.to_string(),
            },
        );
    }

    pub(crate) fn poison_function(&mut self, name: &str) {
        self.poisoned_functions.insert(name.to_string());
    }

    pub(crate) fn push_global_site(&mut self, function: &str, class: ValueClass) {
        if class.is_scalar() {
            return;
        }
        self.global_sites.push(GlobalSiteIf {
            function: function.to_string(),
            class,
        });
    }

    pub(crate) fn push_outflow(&mut self, function: &str, ordinal: u32, class: ValueClass) {
        if class.is_scalar() {
            return;
        }
        self.outflow_sites.push(OutflowIf {
            function: function.to_string(),
            ordinal,
            class,
        });
    }

    pub(crate) fn push_arg_site(
        &mut self,
        function: &str,
        callee: &str,
        index: usize,
        class: ValueClass,
        loop_ordinals: Vec<u32>,
    ) {
        if class.is_scalar() {
            return;
        }
        self.arg_sites.push(ArgEscapeIf {
            function: function.to_string(),
            callee: callee.to_string(),
            index,
            class,
            loop_ordinals,
        });
    }

    pub(crate) fn global_sites(&self) -> &[GlobalSiteIf] {
        &self.global_sites
    }

    pub(crate) fn outflow_sites(&self) -> &[OutflowIf] {
        &self.outflow_sites
    }

    pub(crate) fn arg_sites(&self) -> &[ArgEscapeIf] {
        &self.arg_sites
    }
}

/// Fixpoint results. Both sets are complete (post-closure) when returned.
#[derive(Debug, Default)]
pub(crate) struct FlowSolution {
    may_heap: BTreeSet<FlowNode>,
    tainted: BTreeSet<FlowNode>,
    poisoned_functions: BTreeSet<String>,
}

impl FlowSolution {
    pub(crate) fn class_may_heap(&self, class: &ValueClass) -> bool {
        match class {
            ValueClass::Scalar => false,
            ValueClass::Heap(_) => true,
            ValueClass::DependsOn(nodes) => {
                nodes.iter().any(|node| self.may_heap.contains(node))
            }
        }
    }

    pub(crate) fn param_escapes(&self, function: &str, index: usize) -> bool {
        self.poisoned_functions.contains(function)
            || self.tainted.contains(&FlowNode::Param {
                function: function.to_string(),
                index,
            })
    }

    pub(crate) fn binding_escapes(&self, owner: &str, name: &str) -> bool {
        self.tainted.contains(&FlowNode::Binding {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }
}

/// One monotone worklist fixpoint: may-heap forward from
/// `heap_seeds ∪ param_nodes`, escape taint backward from `taint_seeds`.
/// Bits only flip false→true over a finite node set, so this terminates in
/// O(edges) enqueues per direction regardless of cycles.
pub(crate) fn solve(flow: &FlowCollector) -> FlowSolution {
    let mut forward: BTreeMap<&FlowNode, Vec<&FlowNode>> = BTreeMap::new();
    let mut backward: BTreeMap<&FlowNode, Vec<&FlowNode>> = BTreeMap::new();
    for (from, to) in &flow.edges {
        forward.entry(from).or_default().push(to);
        backward.entry(to).or_default().push(from);
    }

    let mut may_heap: BTreeSet<FlowNode> = flow.heap_seeds.clone();
    may_heap.extend(flow.param_nodes.iter().cloned());
    let mut queue: VecDeque<FlowNode> = may_heap.iter().cloned().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(successors) = forward.get(&node) {
            for succ in successors {
                if may_heap.insert((*succ).clone()) {
                    queue.push_back((*succ).clone());
                }
            }
        }
    }

    let mut tainted: BTreeSet<FlowNode> = flow.taint_seeds.clone();
    let mut queue: VecDeque<FlowNode> = tainted.iter().cloned().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(predecessors) = backward.get(&node) {
            for pred in predecessors {
                if tainted.insert((*pred).clone()) {
                    queue.push_back((*pred).clone());
                }
            }
        }
    }

    FlowSolution {
        may_heap,
        tainted,
        poisoned_functions: flow.poisoned_functions.clone(),
    }
}

#[cfg(test)]
#[path = "escape_flow_tests.rs"]
mod escape_flow_tests;
```

In `crates/kali_mir/src/analysis/mod.rs`, add below the existing `pub mod arena_gate;` line:

```rust
pub(crate) mod escape_flow;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_mir escape_flow 2>&1 | tail -5`
Expected: `8 passed; 0 failed`

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir/src/analysis/escape_flow.rs crates/kali_mir/src/analysis/escape_flow_tests.rs crates/kali_mir/src/analysis/mod.rs
git commit -m "feat(mir): escape-flow fixpoint core — FlowNode/ValueClass/FlowCollector/solve (may-heap forward, taint backward, params fail-closed heap)"
```

---

### Task 2: `classify_value` + walk recording (additive — old gate logic untouched)

**Files:**
- Modify: `crates/kali_mir/src/analysis/escape_flow.rs` (add `classify_value` impl block)
- Modify: `crates/kali_mir/src/analysis/mod.rs` (analyzer `flow` field)
- Modify: `crates/kali_mir/src/analysis/walk.rs` (declarator edge + param registration)
- Modify: `crates/kali_mir/src/analysis/arena_gate.rs` (collision → `poison_function`; recording added ALONGSIDE existing logic in the three note hooks)
- Modify: `crates/kali_mir/src/analysis/escape_flow_tests.rs` (solution-level tests on real sources)

**Interfaces:**
- Consumes: Task 1's types; `OwnershipAnalyzer` fields `nodes`, `scope_stack`, and methods `resolve_binding(&str) -> Option<(usize, usize)>`, `resolve_function_target(&str) -> Option<String>`, `infer_layout(HirNodeId) -> LayoutDescriptor`, `current_scope_label() -> String`, `current_scope_index() -> usize`.
- Produces: `OwnershipAnalyzer::classify_value(&self, HirNodeId) -> ValueClass` (pure — no recording); analyzer field `pub(crate) flow: escape_flow::FlowCollector`; a fully-populated `FlowCollector` after any walk. Task 3 consumes both.

**IMPORTANT:** in this task the old immediate judgments (`arena_is_heap_value`, `maybe_heap_bindings`, immediate global-site marks) stay in place and keep passing the existing suite; the new recording runs beside them. Behavior is unchanged until Task 3 cuts over.

- [ ] **Step 1: Write the failing solution-level tests**

Append to `crates/kali_mir/src/analysis/escape_flow_tests.rs`:

```rust
// --- Solution-level tests on real sources (walk → collector → solve) --------

use crate::{MirFunctionKind, OwnershipAnalyzer, UseContext};

fn solution_for(source: &str) -> FlowSolution {
    let hir = crate::test_support::parse_and_lower_hir(source);
    let mut analyzer = OwnershipAnalyzer::new(&hir.nodes, &hir.function_flavors);
    analyzer.push_scope("<module>", MirFunctionKind::Module, None);
    analyzer.precollect_scope_bindings(hir.root);
    analyzer.walk_scope_node(hir.root, UseContext::Normal);
    analyzer.pop_scope_and_record();
    solve(&analyzer.flow)
}

#[test]
fn param_stored_to_module_binding_escapes_in_solution() {
    let solution = solution_for("let sink; function retain(p) { sink = p; }");
    assert!(solution.param_escapes("retain", 0));
}

#[test]
fn param_only_read_does_not_escape_in_solution() {
    let solution =
        solution_for("function itemCheck(t) { if (t.left === null) { return 1; } return 2; }");
    assert!(!solution.param_escapes("itemCheck", 0));
}

#[test]
fn call_result_binding_is_may_heap_regardless_of_walk_order() {
    // `helper` is hoisted below the read: the fixpoint still sees x may-heap.
    let solution = solution_for(
        "function f() {
           let x = 0;
           helper();
           function helper() { x = mk(); }
           return 0;
         }
         function mk() { return { v: 1 }; }",
    );
    let x = FlowNode::Binding {
        owner: "f".to_string(),
        name: "x".to_string(),
    };
    assert!(solution.class_may_heap(&ValueClass::DependsOn(
        std::iter::once(x).collect()
    )));
}

#[test]
fn transitive_plain_ident_chain_taints_all_links() {
    let solution = solution_for(
        "let cache;
         function f() {
           const a = { v: 1 };
           const b = a;
           cache = b;
           return 0;
         }",
    );
    assert!(solution.binding_escapes("f", "a"));
    assert!(solution.binding_escapes("f", "b"));
}

#[test]
fn param_escape_propagates_through_wrapper_functions() {
    // wrap passes its param straight to retain: both slots escape.
    let solution = solution_for(
        "let sink;
         function retain(p) { sink = p; }
         function wrap(q) { retain(q); }",
    );
    assert!(solution.param_escapes("retain", 0));
    assert!(solution.param_escapes("wrap", 0));
}

#[test]
fn param_returned_is_not_taint_only_returned() {
    // Returning a param is NOT stored-outward taint (the engine's `returned`
    // flag already covers it; the caller's own sites classify the result).
    let solution = solution_for("function id(p) { return p; }");
    assert!(!solution.param_escapes("id", 0));
}

#[test]
fn mutual_recursion_terminates() {
    let solution = solution_for(
        "function a(n) { return b(n); }
         function b(n) { return a(n); }",
    );
    assert!(!solution.param_escapes("a", 0));
    assert!(!solution.param_escapes("b", 0));
}

#[test]
fn param_embedded_in_literal_stored_outward_escapes() {
    // The literal carries p in its embeds set; storing the literal to a
    // module binding taints p.
    let solution = solution_for("let cache; function stash(p) { cache = { v: p }; }");
    assert!(solution.param_escapes("stash", 0));
}

#[test]
fn member_read_carries_base_identity_for_taint() {
    // cache = p.left stores part of p's structure outward: p escapes.
    let solution = solution_for("let cache; function grab(p) { cache = p.left; }");
    assert!(solution.param_escapes("grab", 0));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_mir escape_flow 2>&1 | tail -5`
Expected: compile error — `analyzer.flow` field and `classify_value` do not exist yet.

- [ ] **Step 3: Add the analyzer field and `classify_value`**

In `crates/kali_mir/src/analysis/mod.rs`, add the field to `OwnershipAnalyzer` (after `pub(crate) arena: arena_gate::ArenaCollector,`):

```rust
    pub(crate) flow: escape_flow::FlowCollector,
```

and initialize it in `OwnershipAnalyzer::new` (after `arena: arena_gate::ArenaCollector::default(),`):

```rust
            flow: escape_flow::FlowCollector::default(),
```

Append to `crates/kali_mir/src/analysis/escape_flow.rs` (before the test mod), together with the imports it needs at the top of the file (`use kali_hir::{HirNodeId, HirNodeKind};` and `use crate::{LayoutDescriptor, OwnershipAnalyzer};`):

```rust
// ---------------------------------------------------------------------------
// Tri-state value classification (walk-time, pure — records nothing).
// ---------------------------------------------------------------------------

/// Heap-typed layouts, fail-closed: `TaggedVal` AND `Scalar("unknown")` (the
/// layout of `null`/`undefined`) count as heap. Moved here from arena_gate
/// when the gate's own judgment was retired.
pub(crate) fn is_heap_layout(layout: &LayoutDescriptor) -> bool {
    match layout {
        LayoutDescriptor::Scalar(name) => name == "unknown",
        _ => true,
    }
}

impl<'a> OwnershipAnalyzer<'a> {
    /// Classify an expression's value without resolving binding heap-ness:
    /// identifiers become `DependsOn` nodes for the fixpoint. Operator-aware:
    /// `sum + itemCheck(tree)` is `Scalar` no matter what `tree` is — this is
    /// what keeps the binary-trees loop grant alive. Every unknown shape
    /// fails closed to `Heap`.
    pub(crate) fn classify_value(&self, node_id: HirNodeId) -> ValueClass {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr => {
                // A structure carries the identity of everything embedded in
                // it: if the structure escapes, its contents escape.
                let mut embeds = BTreeSet::new();
                for child in &node.children {
                    embeds.extend(self.classify_value(*child).take_nodes());
                }
                ValueClass::Heap(embeds)
            }
            HirNodeKind::ObjectProperty => match node.children.get(1) {
                Some(value) => self.classify_value(*value),
                None => ValueClass::heap(),
            },
            HirNodeKind::Literal => {
                if is_heap_layout(&self.infer_layout(node_id)) {
                    ValueClass::heap()
                } else {
                    ValueClass::Scalar
                }
            }
            // A ternary produces whichever branch runs (children[1..]).
            HirNodeKind::ConditionalExpr => {
                if node.children.len() < 2 {
                    return ValueClass::heap();
                }
                node.children
                    .iter()
                    .skip(1)
                    .fold(ValueClass::Scalar, |acc, child| {
                        acc.join(self.classify_value(*child))
                    })
            }
            // Logical ops return an operand; a sequence returns its last
            // expression — join everything (fail closed on malformed).
            HirNodeKind::LogicalExpr | HirNodeKind::SequenceExpr => {
                if node.children.is_empty() {
                    return ValueClass::heap();
                }
                node.children
                    .iter()
                    .fold(ValueClass::Scalar, |acc, child| {
                        acc.join(self.classify_value(*child))
                    })
            }
            HirNodeKind::BinaryExpr => match node.text.as_deref() {
                // Genuinely scalar-producing operators. String `+` concat is
                // scalar too in v1: runtime strings are global-arena host
                // values and never dangle across a reset.
                Some(
                    "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>"
                    | ">>>" | "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=",
                ) => ValueClass::Scalar,
                // `&&`/`||`/`??` parse as BinaryExpr and return an operand.
                Some("&&" | "||" | "??") => {
                    if node.children.is_empty() {
                        return ValueClass::heap();
                    }
                    node.children
                        .iter()
                        .fold(ValueClass::Scalar, |acc, child| {
                            acc.join(self.classify_value(*child))
                        })
                }
                _ => self.classify_children_as_heap(node_id),
            },
            HirNodeKind::UnaryExpr => {
                if matches!(node.text.as_deref(), Some("!" | "-" | "+" | "~" | "typeof")) {
                    ValueClass::Scalar
                } else {
                    self.classify_children_as_heap(node_id)
                }
            }
            HirNodeKind::Ident => {
                let name = node.text.as_deref().unwrap_or_default();
                match self.resolve_binding(name) {
                    Some((scope_index, _)) => {
                        let owner = self
                            .scope_stack
                            .get(scope_index)
                            .map(|scope| scope.label.clone())
                            .unwrap_or_else(|| "<module>".to_string());
                        ValueClass::DependsOn(
                            std::iter::once(FlowNode::Binding {
                                owner,
                                name: name.to_string(),
                            })
                            .collect(),
                        )
                    }
                    None => ValueClass::heap(),
                }
            }
            HirNodeKind::CallExpr => {
                let target = node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .filter(|callee| callee.kind == HirNodeKind::Ident)
                    .and_then(|callee| callee.text.as_deref())
                    .and_then(|name| self.resolve_function_target(name));
                match target {
                    Some(function) => ValueClass::DependsOn(
                        std::iter::once(FlowNode::Return { function }).collect(),
                    ),
                    None => ValueClass::heap(),
                }
            }
            // A member read shares its base's structure: if the read value
            // escapes, the base's contents escape with it.
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr => {
                match node.children.first() {
                    Some(base) => ValueClass::Heap(self.classify_value(*base).take_nodes()),
                    None => ValueClass::heap(),
                }
            }
            // Everything else (templates, new-expressions, unknown nodes):
            // heap, carrying any identifier sources found underneath.
            _ => self.classify_children_as_heap(node_id),
        }
    }

    fn classify_children_as_heap(&self, node_id: HirNodeId) -> ValueClass {
        let node = &self.nodes[node_id.0 as usize];
        let mut embeds = BTreeSet::new();
        for child in &node.children {
            embeds.extend(self.classify_value(*child).take_nodes());
        }
        ValueClass::Heap(embeds)
    }
}
```

- [ ] **Step 4: Wire recording into the walk and the gate hooks (additive)**

(4a) `crates/kali_mir/src/analysis/walk.rs` — VarDeclarator arm: directly after the existing `arena_note_fresh_binding(name)` `if` block (keep it), add:

```rust
                        let class = self.classify_value(init);
                        let owner = self.current_scope_label();
                        self.flow.note_value_into(
                            crate::analysis::escape_flow::FlowNode::Binding {
                                owner,
                                name: name.clone(),
                            },
                            &class,
                        );
```

(4b) `walk.rs` — in BOTH the `FunctionDecl` and `FunctionExpr` arms, change the param loop to enumerate and register each param. Replace (in both arms):

```rust
                for child in children.iter().take(params_end) {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
```

with:

```rust
                for (param_index, child) in children.iter().take(params_end).enumerate() {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
                        self.flow
                            .note_param(&function_name, param_index, param_name);
```

(the `scope.define(...)` body below stays; `param_name` is cloned there already).

(4c) `crates/kali_mir/src/analysis/arena_gate.rs` — in `arena_enter_function`, after `raw.name_collision = true;`, add:

```rust
            self.flow.poison_function(&label);
```

(4d) `arena_gate.rs` — `arena_note_assignment`: keep ALL existing logic; add recording at the top of the `HirNodeKind::Ident` arm (after `let name = ...`):

```rust
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
```

and at the top of the `MemberExpr | OptionalChain | ChainExpr` arm:

```rust
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
```

and at the top of the fallback `_` arm:

```rust
                let class = self.classify_value(right);
                let function = self.current_scope_label();
                self.flow.push_global_site(&function, class.clone());
                self.flow.note_taint_class(&class);
                let all_ordinals: Vec<u32> =
                    self.arena.loop_stack.iter().map(|l| l.ordinal).collect();
                for ordinal in all_ordinals {
                    self.flow.push_outflow(&function, ordinal, class.clone());
                }
```

Borrow-check note: `left_node`/`left_kind` are read before any `&mut self` call in the existing code — keep that ordering; the ordinal `Vec`s exist to end the `self.arena` borrow before `self.flow` methods run.

(4e) `arena_gate.rs` — `arena_note_return`: add at the top (keep the existing body after it, INCLUDING its early-return for the empty loop stack — that early return must now come AFTER the recording):

```rust
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
        let ordinals: Vec<u32> = self.arena.loop_stack.iter().map(|l| l.ordinal).collect();
        for ordinal in ordinals {
            self.flow.push_outflow(&function, ordinal, class.clone());
        }
```

(add `use crate::analysis::escape_flow::ValueClass;` to arena_gate.rs imports).

(4f) `arena_gate.rs` — `arena_note_call_expr`: capture the resolved target and record args. Change the `HirNodeKind::Ident` arm of the callee match to bind the target:

```rust
            HirNodeKind::Ident => {
                let name = callee_node.text.clone();
                match name.and_then(|n| self.resolve_function_target(&n)) {
                    Some(target) => {
                        self.arena_note_call(&target);
                        known_target = Some(target);
                    }
                    None => self.arena_note_unknown_call(),
                }
            }
```

with `let mut known_target: Option<String> = None;` declared beside `let mut whitelisted = false;`. Then REPLACE the existing trailing fresh-literal-argument block:

```rust
        // A fresh literal handed to a non-whitelisted call might be retained by
        // the callee ⇒ global (fail closed). Whitelisted host calls consume it.
        if !whitelisted {
            for arg in children.iter().skip(1).copied() {
                if self.arena_is_fresh_literal(arg) {
                    self.arena_note_global_site();
                }
            }
        }
```

with class-based recording (this is the summary-gated refinement — a may-heap
argument to a callee whose param slot never escapes no longer vetoes, which is
what keeps `itemCheck(bottomUpTree(d))` eligible):

```rust
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
                    self.flow.push_arg_site(
                        &function,
                        target,
                        index,
                        class,
                        open_ordinals.clone(),
                    );
                }
                None => {
                    // Unknown callee: assume it retains every argument.
                    self.flow.note_taint_class(&class);
                    self.flow.push_global_site(&function, class);
                }
            }
        }
```

NOTE: removing the fresh-literal-arg immediate veto in this task changes gate behavior for known-callee literal args, but the deferred `arg_sites` are not consumed until Task 3 — to keep the suite green across this commit, the deferred sites for KNOWN targets take over the veto in Task 3. Verify in Step 5 that no existing test covered "fresh literal arg to a KNOWN callee" (survey result while planning: none does — the only arg-veto pins use unknown callees, which keep their immediate `arena_note_unknown_call` + the new unconditional global site push resolves identically in Task 3; and `loop_veto_on_unknown_call`'s veto comes from `has_unknown_call`, which is untouched).

- [ ] **Step 5: Run the full kali_mir suite plus the new tests**

Run: `cargo test -p kali_mir 2>&1 | tail -5`
Expected: `69 passed; 0 failed; 2 ignored` (60 existing + 8 Task-1 + 9 Task-2 solution-level tests, minus 8 already counted — verify the exact number printed; ALL non-ignored tests must pass, and the two `#[ignore]` pins stay ignored).

- [ ] **Step 6: Commit**

```bash
git add -A crates/kali_mir
git commit -m "feat(mir): walk records tri-state ValueClass judgments, flow edges, and deferred veto sites alongside the existing gate logic (no behavior change)"
```

---

### Task 3: Gate cutover — resolve deferred sites, delete the old judgment, flip the xfail pins

**Files:**
- Modify: `crates/kali_mir/src/analysis/arena_gate_tests.rs` (remove both `#[ignore]` attributes)
- Modify: `crates/kali_mir/src/analysis/arena_gate.rs` (`into_facts(flow, solution)`; delete `arena_is_heap_value`, `is_heap_layout` (moved in Task 2), `maybe_heap_bindings`, `arena_note_maybe_heap_binding`, `arena_note_outflow_to_binding`, `arena_note_outflow_all_loops`, and the old immediate-judgment remnants inside the three hooks)
- Modify: `crates/kali_mir/src/analysis/mod.rs` (`analyze_program_with_arena` runs `solve` and passes results to `into_facts`)

**Interfaces:**
- Consumes: `FlowCollector`/`FlowSolution`/`solve` from Task 1; recording from Task 2.
- Produces: `ArenaCollector::into_facts(self, flow: &escape_flow::FlowCollector, solution: &escape_flow::FlowSolution) -> Vec<FunctionArenaFacts>`. `MirProgram` shape and `compute_arena_table` are UNCHANGED.

- [ ] **Step 1: Flip the acceptance tests to RED**

In `crates/kali_mir/src/analysis/arena_gate_tests.rs`, delete these two lines (keep the test functions and the boundary-statement comment block, updating its first paragraph as shown):

```rust
#[ignore = "known fail-open: walk-order launder via hoisted function; needs order-independent (fixpoint) classification — see task-4 round-5 report"]
```

```rust
#[ignore = "known fail-open: param-mediated escape (engine's param-escape flags are blind to plain-ident outer stores); needs interprocedural summaries — see task-4 round-5 report"]
```

Replace the pin block header comment (the lines from `// --- KNOWN FAIL-OPEN pins (round 5, BLOCKED) ---` through `// fix must clear ALL assertions in both pins.`) with:

```rust
// --- Interprocedural launder pins (round 5 xfails, closed by escape_flow) ----
// These two shapes corrupted arena_eligible itself under the old walk-order-
// sensitive judgment (task-4-report.md rounds 4-5b: plain-ident dataflow was
// unmodeled in both the gate and the engine; there was no axis containment,
// only pattern containment). The escape_flow fixpoint resolves heap-ness and
// param-escape summaries order-independently, so all assertions — loop_arena
// via the driver loop, opens_arena, arena_eligible — now hold.
```

- [ ] **Step 2: Run to verify both pins FAIL**

Run: `cargo test -p kali_mir arena_gate 2>&1 | tail -8`
Expected: `2 failed` — `ineligible_on_hoisted_function_launder` and `ineligible_on_param_mediated_escape`, first assertion `!table.loop_arena("g", 0)`.

- [ ] **Step 3: Cut `into_facts` over to the solution and run solve in the pipeline**

(3a) `crates/kali_mir/src/analysis/arena_gate.rs` — add to imports:

```rust
use crate::analysis::escape_flow::{FlowCollector, FlowSolution};
```

Change `into_facts` to take the flow data and resolve deferred sites before emission:

```rust
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

        let ArenaCollector {
            functions, order, ..
        } = self;
        // ... (the existing emission body from here on is UNCHANGED) ...
```

(3b) `crates/kali_mir/src/analysis/mod.rs` — `analyze_program_with_arena` becomes:

```rust
    pub(crate) fn analyze_program_with_arena(
        mut self,
        root: HirNodeId,
    ) -> (Vec<MirFunction>, Vec<arena_gate::FunctionArenaFacts>) {
        self.push_scope("<module>", MirFunctionKind::Module, None);
        self.precollect_scope_bindings(root);
        self.walk_scope_node(root, UseContext::Normal);
        self.pop_scope_and_record();
        let flow = std::mem::take(&mut self.flow);
        let solution = escape_flow::solve(&flow);
        let facts = std::mem::take(&mut self.arena).into_facts(&flow, &solution);
        (self.functions, facts)
    }
```

(3c) `arena_gate.rs` — delete the now-redundant OLD immediate logic, leaving the Task-2 recording as the only body of each hook:

- In `arena_note_assignment` (`Ident` arm): delete the entire old `if rhs_fresh || rhs_heap { match self.resolve_binding(...) ... }` block and the old `if rhs_heap { self.arena_note_outflow_to_binding(&name); }` — BUT keep the same-scope fresh-binding fate note by adding it into the Task-2 recording block: inside the `if let Some((scope_index, _))` branch, after the cross-scope `if`, add:

```rust
                    else if self.arena_is_fresh_literal(right) {
                        self.arena_note_fresh_binding(&name);
                    }
```

- In the `MemberExpr | OptionalChain | ChainExpr` arm: delete the old `if rhs_fresh { ... }` and `if rhs_heap { ... }` blocks (the Task-2 recording is the whole body now).
- In the `_` arm: delete the old `if rhs_fresh || rhs_heap { ... }` block.
- Delete the local `let rhs_fresh = ...;` / `let rhs_heap = ...;` at the top of `arena_note_assignment` (`arena_is_fresh_literal` is still called in the Ident arm — keep that fn).
- In `arena_note_return`: delete the old `if self.arena.loop_stack.is_empty() { return; }` early-return and the old `returns_heap` block (the Task-2 recording is the whole body).
- Delete fn `arena_is_heap_value` entirely, fn `arena_note_maybe_heap_binding`, fn `arena_note_outflow_to_binding`, fn `arena_note_outflow_all_loops`, the `maybe_heap_bindings` field of `FuncRaw` (and its doc comment), and the module-level `fn is_heap_layout` (Task 2 moved it to escape_flow.rs — fix the one remaining caller, `classify_value`'s Literal arm, which already imports it locally).
- Update the module doc comment's description of the judgment (first paragraph) to mention that heap-ness/escape resolution now lives in `escape_flow` and `into_facts` resolves deferred sites.

- [ ] **Step 4: Run the gate suite — all 26 pins green**

Run: `cargo test -p kali_mir arena_gate 2>&1 | tail -5`
Expected: `26 passed; 0 failed; 0 ignored` (24 old + 2 flipped pins).

Then the whole crate: `cargo test -p kali_mir 2>&1 | tail -5`
Expected: 0 failed. If an ownership/lower test fails here, STOP and diagnose — Task 3 must not change engine verdicts (only Task 4 does).

- [ ] **Step 5: Commit**

```bash
git add -A crates/kali_mir
git commit -m "fix(mir): arena gate resolves heap-ness through the escape-flow fixpoint — deferred global/outflow/arg sites replace the walk-order-sensitive judgment; both round-5 xfail launder pins flip green"
```

---

### Task 4: Engine post-pass — `binding.escapes` learns the fixpoint verdicts

**Files:**
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/plain_ident_escape.rs`
- Modify: `crates/kali_mir/src/analysis/ownership_analysis_tests.rs` (register module)
- Modify: `crates/kali_mir/src/analysis/escape_flow.rs` (add `apply_escape_verdicts`)
- Modify: `crates/kali_mir/src/analysis/mod.rs` (call it in `analyze_program_with_arena`)

**Interfaces:**
- Consumes: `FlowSolution::binding_escapes`; `MirFunction { name: Option<String>, kind: MirFunctionKind, bindings: Vec<MirBinding> }`; `MirBinding { escapes: bool, ownership: OwnershipClass, kind: MirBindingKind, captured_by: Vec<String> }`.
- Produces: `pub(crate) fn apply_escape_verdicts(functions: &mut [MirFunction], solution: &FlowSolution)` in escape_flow.rs.

- [ ] **Step 1: Write the failing engine pins**

Create `crates/kali_mir/src/analysis/ownership_analysis_tests/plain_ident_escape.rs`:

```rust
//! Engine-level pins for the interprocedural escape round: plain-ident
//! stores to enclosing bindings must mark `escapes` (the round-5 blindness —
//! `is_heap_store_target` covers only member/chain LHS, so `sink = p` left
//! `p.escapes == false` and callers passed heap values un-flagged).

use super::*;

#[test]
fn test_param_stored_to_module_binding_via_plain_ident_escapes() {
    let mir = analyze("let sink; function retain(p) { sink = p; }");
    let retain = mir.function("retain").expect("retain function");
    let binding = retain.binding("p").expect("p binding");

    assert_eq!(binding.kind, MirBindingKind::Parameter);
    assert!(binding.escapes);
    // Parameters keep their Borrowed ownership (finalise_binding's
    // returned/escaped arm leaves Parameter ownership untouched).
    assert_eq!(binding.ownership, OwnershipClass::Borrowed);
}

#[test]
fn test_local_stored_outward_through_alias_chain_escapes() {
    let mir = analyze(
        "let cache;
         function f() {
           const a = { v: 1 };
           const b = a;
           cache = b;
           return 0;
         }",
    );
    let f = mir.function("f").expect("f function");
    let a = f.binding("a").expect("a binding");
    let b = f.binding("b").expect("b binding");

    assert!(a.escapes);
    assert!(b.escapes);
    assert_eq!(a.ownership, OwnershipClass::OwnedHeap);
    assert_eq!(b.ownership, OwnershipClass::OwnedHeap);
}

#[test]
fn test_purely_local_binding_still_does_not_escape() {
    let mir = analyze("function f() { const o = { v: 1 }; let s = o.v; return s; }");
    let f = mir.function("f").expect("f function");
    let o = f.binding("o").expect("o binding");

    assert!(!o.escapes);
    assert_eq!(o.ownership, OwnershipClass::Stack);
}

#[test]
fn test_module_scope_bindings_are_not_blanket_flipped() {
    // Module bindings are storage roots, not escapees: the post-pass must
    // skip the module scope.
    let mir = analyze("const answer = 40 + 2;");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert!(!binding.escapes);
    assert_eq!(binding.ownership, OwnershipClass::Stack);
}
```

Register it in `crates/kali_mir/src/analysis/ownership_analysis_tests.rs` (append, matching the existing pattern):

```rust
#[path = "ownership_analysis_tests/plain_ident_escape.rs"]
mod plain_ident_escape;
```

- [ ] **Step 2: Run to verify the first two tests fail**

Run: `cargo test -p kali_mir plain_ident_escape 2>&1 | tail -6`
Expected: 2 failed (`test_param_stored_to_module_binding_via_plain_ident_escapes`, `test_local_stored_outward_through_alias_chain_escapes` — `assert!(binding.escapes)`), 2 passed.

- [ ] **Step 3: Implement the post-pass**

Append to `crates/kali_mir/src/analysis/escape_flow.rs` (add `use crate::{MirBindingKind, MirFunction, MirFunctionKind, OwnershipClass};` to its imports):

```rust
// ---------------------------------------------------------------------------
// Engine consumer: fold fixpoint escape verdicts into finalized bindings.
// ---------------------------------------------------------------------------

/// The fourth disjunct of the engine's escape judgment: a binding whose value
/// the fixpoint proves stored beyond a dynamic extent escapes, even when the
/// walk saw only plain-ident dataflow (`sink = p`, alias chains, hoisted
/// helpers). Mirrors `finalise_binding`'s ownership mapping; only ever flips
/// verdicts toward escaping. Module-scope bindings are storage roots, not
/// escapees — skipped.
pub(crate) fn apply_escape_verdicts(functions: &mut [MirFunction], solution: &FlowSolution) {
    for function in functions.iter_mut() {
        if function.kind == MirFunctionKind::Module {
            continue;
        }
        let Some(owner) = function.name.clone() else {
            continue;
        };
        for binding in &mut function.bindings {
            if binding.escapes {
                continue;
            }
            if !solution.binding_escapes(&owner, &binding.name) {
                continue;
            }
            binding.escapes = true;
            match binding.kind {
                MirBindingKind::Local | MirBindingKind::Function => {
                    binding.ownership = if binding.captured_by.is_empty() {
                        OwnershipClass::OwnedHeap
                    } else {
                        OwnershipClass::SharedHeap
                    };
                }
                // finalise_binding's returned/escaped arm keeps Parameter and
                // Import ownership untouched.
                MirBindingKind::Parameter | MirBindingKind::Import => {}
            }
        }
    }
}
```

In `crates/kali_mir/src/analysis/mod.rs`, call it in `analyze_program_with_arena` between `solve` and `into_facts`:

```rust
        let solution = escape_flow::solve(&flow);
        escape_flow::apply_escape_verdicts(&mut self.functions, &solution);
        let facts = std::mem::take(&mut self.arena).into_facts(&flow, &solution);
```

- [ ] **Step 4: Run the new pins, then the whole crate, and audit any flips**

Run: `cargo test -p kali_mir plain_ident_escape 2>&1 | tail -5`
Expected: `4 passed`.

Run: `cargo test -p kali_mir 2>&1 | tail -8`
Expected: 0 failed. Planning-time survey says no existing test asserts the buggy verdict, so no updates should be needed. IF a pre-existing test fails, apply this rule strictly: the test may be updated ONLY if its source shape stores a binding's value into an enclosing-scope target through plain-ident dataflow (directly, via an alias chain, or via a call to a function that does) — i.e., the old expectation is exactly the round-5-documented blindness. Update it with a comment citing this round (`// escape verdict corrected by the 2026-07-05 interprocedural escape-flow round: <one-line reason>`). ANY other failure means the post-pass over-taints — STOP and debug `solve`/recording instead of updating the test.

- [ ] **Step 5: Commit**

```bash
git add -A crates/kali_mir
git commit -m "fix(mir): binding.escapes learns fixpoint verdicts — plain-ident stores to enclosing bindings (sink = p, alias chains) now mark escapes; engine param-escape blindness closed"
```

---

### Task 5: Hardening pins — the shapes the fixpoint claims to close (and the grants it must preserve)

**Files:**
- Modify: `crates/kali_mir/src/analysis/arena_gate_tests.rs`

**Interfaces:**
- Consumes: `compute_arena_table`, `test_support::analyze` (as every existing pin does).
- Produces: six new gate pins; no source changes expected (each pin should pass against Tasks 1–4's implementation — any failure is a real bug to fix in `escape_flow.rs`/`arena_gate.rs`, not in the test).

- [ ] **Step 1: Add the pins**

Append to `crates/kali_mir/src/analysis/arena_gate_tests.rs`:

```rust
// --- Interprocedural round hardening pins ------------------------------------

#[test]
fn eligible_when_arg_passed_to_nonescaping_param() {
    // THE load-bearing precision grant: a may-heap call-result argument to a
    // callee whose param never escapes must NOT veto — this is the
    // `itemCheck(bottomUpTree(d))` shape at the heart of binary-trees.
    let mir = analyze(
        "function bottomUpTree(d) { return { left: null, right: null }; }
         function itemCheck(t) { if (t.left === null) { return 1; } return 2; }
         function f(n) {
           let sum = 0;
           for (let i = 0; i < n; i = i + 1) {
             sum = sum + itemCheck(bottomUpTree(3));
           }
           return sum;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
    assert!(table.arena_eligible("bottomUpTree"));
}

#[test]
fn ineligible_on_call_result_stored_into_member() {
    // A call result stored into a pre-existing object's field outlives the
    // frame. The old judgment only caught FRESH literals in member stores;
    // the class-based site catches laundered/call-result values too.
    let mir = analyze(
        "function mk() { return { v: 1 }; }
         function f(p) {
           const local = { w: 2 };
           const x = mk();
           p.left = x;
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_transitive_chain_across_hoisted_helpers() {
    // The round-4 transitive shape (x -> keep -> cache across hoisted
    // helpers): every link is plain-ident dataflow, the store is above the
    // helper declarations, and only the fixpoint sees the whole chain.
    let mir = analyze(
        "let cache;
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           let x = 0;
           let keep = 0;
           step1();
           step2();
           cache = keep;
           function step1() { x = mk(); }
           function step2() { keep = x; }
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_launder_through_returning_callee() {
    // Heap-ness survives a round trip through `id`: the returned value is
    // the module-stored value.
    let mir = analyze(
        "let cache;
         function id(p) { return p; }
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           const x = mk();
           const y = id(x);
           cache = y;
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_param_embedded_in_literal_stored_outward() {
    // The callee wraps its param in a fresh literal and stores THAT outward:
    // the embeds set must carry p so the arg site still fires in the caller.
    let mir = analyze(
        "let cache;
         function stash(p) { cache = { v: p }; }
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           const x = mk();
           stash(x);
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_heap_ident_arg_to_unknown_callee() {
    // Old behavior only vetoed FRESH LITERAL args to unknown callees; a
    // may-heap IDENT handed to an unknown callee must veto too.
    let mir = analyze(
        "function mk() { return { v: 1 }; }
         function f(cb) {
           const local = { w: 2 };
           const x = mk();
           cb(x);
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}
```

- [ ] **Step 2: Run the pins**

Run: `cargo test -p kali_mir arena_gate 2>&1 | tail -5`
Expected: `32 passed; 0 failed`. Any failure here is an implementation bug — debug the recording/fixpoint (likely suspects: embeds propagation for `stash`, member-read base identity for chains, `Param`-node edge direction), do NOT weaken the pin.

- [ ] **Step 3: Run the whole crate**

Run: `cargo test -p kali_mir 2>&1 | tail -5`
Expected: 0 failed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/analysis/arena_gate_tests.rs
git commit -m "test(mir): hardening pins for the interprocedural round — nonescaping-param grant preserved (itemCheck(bottomUpTree)); member-store call-result, transitive hoisted chain, launder-through-return, literal-embed, unknown-callee heap-ident all veto"
```

---

### Task 6: Full verification gate and PR

**Files:** none (verification + delivery).

**Interfaces:**
- Consumes: the standing 5-crate verification gate; the four CLBG runtime fixtures (covered by `kali_cli` tests); repo fmt/warnings baselines.
- Produces: a merged PR per the standing integration convention (push branch, open PR, merge after review passes).

- [ ] **Step 1: Full kali_mir suite (foreground)**

Run: `cargo test -p kali_mir 2>&1 | tail -5`
Expected: 0 failed, 0 ignored in the arena_gate suite (the xfails are gone).

- [ ] **Step 2: The 5-crate gate (foreground; this takes a while)**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli 2>&1 | tail -15`
Expected: every suite line ends `0 failed`; overall exit 0. This includes the CLBG runtime fixtures (nbody, spectral-norm, mandelbrot, fannkuch golden-output tests), which proves behavior-neutrality for shipped programs — nothing reads `ArenaTable` in codegen yet, and LIR consumes only the structural MIR tree, so any fixture diff means an unintended leak into lowering: STOP and bisect.

- [ ] **Step 3: Format check**

Run: `cargo fmt --check`
Expected: no output, exit 0. (If it fails: `cargo fmt`, re-run the gate's failing crate, amend.)

- [ ] **Step 4: Warning parity**

Run: `cargo build -p kali_mir 2>&1 | grep -c warning`
Expected: `0` (crate builds warning-free today; deletions must not leave dead code — if a warning appears, remove the dead item it points at).

- [ ] **Step 5: Push branch, open PR, merge**

```bash
git push -u origin interproc-escape-flow
gh pr create --title "Interprocedural escape flow: shared may-heap/escape fixpoint closes the launder family (both round-5 xfail pins green)" --body "$(cat <<'EOF'
## Summary
- New `escape_flow` module in kali_mir: tri-state ValueClass judgments, plain-ident/param/return flow edges, and deferred veto sites recorded during the ownership walk; one monotone worklist fixpoint (may-heap forward, escape taint backward) resolves them order-independently.
- Arena gate consumes the fixpoint: deferred global/outflow/arg sites replace the walk-order-sensitive `arena_is_heap_value` + maybe-heap machinery. Both round-5 xfail launder pins (hoisted-function walk-order launder; param-mediated escape) flip green with ALL assertions (loop_arena driver form included).
- Engine consumer: `binding.escapes` gains the fixpoint's stored-outward verdict, closing the pre-existing `sink = p` param-escape blindness (plain-ident stores to enclosing bindings).
- Precision win kept sound by summaries: may-heap args to callees whose param slots never escape no longer veto — `itemCheck(bottomUpTree(d))` stays arena-eligible (pinned).

Spec: docs/superpowers/specs/2026-07-05-interprocedural-escape-flow-design.md
Plan: docs/superpowers/plans/2026-07-05-interprocedural-escape-flow.md
Root cause: .superpowers/sdd/task-4-report.md rounds 4-5b.

## Test plan
- [x] cargo test -p kali_mir (arena_gate 32 pins incl. both flipped xfails + 6 hardening pins; escape_flow unit + solution-level tests; plain_ident_escape engine pins)
- [x] 5-crate gate: cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli (incl. CLBG fixture golden outputs — behavior-neutral for shipped programs)
- [x] cargo fmt --check
EOF
)"
```

Then request review per the standing review process, address findings, and merge (squash or merge per repo convention) once green. After merge, binary-trees Phase 1 Tasks 5–8 resume on a fresh branch.

---

## Self-Review (performed while writing)

1. **Spec coverage:** ValueClass tri-state → Task 1/2; FlowGraph edges (assign/declarator/param/return) → Tasks 1–2; FunctionSummary (param_escapes/returns_heap as solution queries; stores-outward as taint) → Task 1; fixpoint + order independence → Task 1 (`insertion_order_does_not_change_the_solution`) and Task 2 (`call_result_binding_is_may_heap_regardless_of_walk_order`); engine consumer fourth disjunct → Task 4; gate consumer → Task 3; conservatism policy (unknown callee taints args, params fail-closed heap, poisoning pre-fixpoint, unresolved-ident heap) → Tasks 1–2 code; round-3 asymmetry (fresh-only fate grants) → Task 3 keeps `arena_note_fresh_binding` literal-gated; acceptance pins → Task 3; hardening/new tests from the spec's list (tri-state table = Task 1 join test + Task 2 sources; transitive chain, launder-through-return, mutual recursion, unknown-callee, order-shuffle) → Tasks 1/2/5; regression bar → Task 6.
2. **Placeholder scan:** none — every step has complete code or an exact command with expected output.
3. **Type consistency:** `note_value_into(FlowNode, &ValueClass)`, `push_global_site(&str, ValueClass)`, `push_arg_site(&str, &str, usize, ValueClass, Vec<u32>)`, `into_facts(&FlowCollector, &FlowSolution)`, `apply_escape_verdicts(&mut [MirFunction], &FlowSolution)`, `param_escapes(&str, usize)`, `binding_escapes(&str, &str)` — used identically across Tasks 1–4. Test count arithmetic in Task 5 Step 2: 24 original + 2 flipped + 6 new = 32.
