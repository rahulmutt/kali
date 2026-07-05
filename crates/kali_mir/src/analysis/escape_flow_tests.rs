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
    flow.note_value_into(
        binding("<module>", "cache"),
        &depends_on(&[binding("f", "b")]),
    );
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
    assert_eq!(
        ValueClass::Scalar.join(ValueClass::Scalar),
        ValueClass::Scalar
    );
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
    assert!(solution.class_may_heap(&ValueClass::DependsOn(std::iter::once(x).collect())));
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
