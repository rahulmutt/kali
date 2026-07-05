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
