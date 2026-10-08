//! Growable-runtime-arrays spec §3.1-§3.2, A-2, A-5, A-7, A-16: the solve.

use super::*;

fn b(func: &str, name: &str) -> GrowNode {
    GrowNode::Binding(func.to_string(), name.to_string())
}

fn r(func: &str) -> GrowNode {
    GrowNode::Return(func.to_string())
}

#[test]
fn a_literal_binding_that_is_pushed_is_growable_and_local_only() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("main", "out"));
    facts.demands.insert(b("main", "out"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("main", "out"));
    assert!(solution.is_local_only("main", "out"));
    assert!(solution.conflicts.is_empty());
}

#[test]
fn a_literal_that_nobody_mutates_is_not_growable() {
    // A-2: `const a = [1, 2, 3]; return a;` stays on the array-return lane.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "a"));
    facts.edges.push((r("f"), b("f", "a")));
    facts.edges.push((b("_start", "xs"), r("f")));
    let solution = solve(&facts);
    assert!(!solution.is_growable(&r("f")));
    assert!(!solution.is_growable_binding("_start", "xs"));
}

#[test]
fn a_push_on_the_call_result_makes_the_returned_literal_growable() {
    // A-2: the caller's `xs.push(4)` flows back through the return edge.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "a"));
    facts.edges.push((r("f"), b("f", "a")));
    facts.edges.push((b("_start", "xs"), r("f")));
    facts.demands.insert(b("_start", "xs"));
    let solution = solve(&facts);
    assert!(solution.is_growable(&r("f")));
    assert!(solution.is_growable_binding("f", "a"));
    assert!(solution.is_growable_binding("_start", "xs"));
    assert!(!solution.is_local_only("f", "a"));
    assert!(solution.same_component(&b("f", "a"), &b("_start", "xs")));
}

#[test]
fn a_push_on_a_parameter_makes_the_argument_literal_growable() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.edges.push((b("add", "a"), b("_start", "xs")));
    facts.demands.insert(b("add", "a"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("_start", "xs"));
    assert!(solution.is_growable_binding("add", "a"));
}

#[test]
fn a_module_scope_binding_is_never_local_only() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "out"));
    facts.demands.insert(b("_start", "out"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("_start", "out"));
    assert!(!solution.is_local_only("_start", "out"));
}

#[test]
fn a_parameter_fed_a_growable_and_an_allocation_is_a_mixed_layout_at_the_parameter() {
    // Review Focus 1: `total(xs); total(p)` with `p = new Array(2)`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.demands.insert(b("_start", "xs"));
    facts.plain_origins.insert(b("_start", "p"));
    facts.edges.push((b("total", "a"), b("_start", "xs")));
    facts.edges.push((b("total", "a"), b("_start", "p")));
    let solution = solve(&facts);
    assert_eq!(
        solution.conflicts,
        vec![GrowConflict::MixedLayout(b("total", "a"))]
    );
}

#[test]
fn an_allocation_without_any_growable_source_is_no_conflict() {
    let mut facts = GrowFacts::default();
    facts.plain_origins.insert(b("_start", "p"));
    facts.demands.insert(b("_start", "p"));
    facts.edges.push((b("total", "a"), b("_start", "p")));
    let solution = solve(&facts);
    assert!(!solution.is_growable_binding("_start", "p"));
    assert!(solution.conflicts.is_empty());
}

#[test]
fn a_literal_expression_in_a_growable_component_is_a_conflict_naming_its_function() {
    // A-5: `function f(c) { if (c) return []; const o = []; o.push(1); return o; }`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "o"));
    facts.demands.insert(b("f", "o"));
    facts.edges.push((r("f"), b("f", "o")));
    facts.edges.push((r("f"), GrowNode::Temp(0)));
    facts
        .temp_kinds
        .insert(0, (TempKind::LiteralExpression, "f".to_string()));
    let solution = solve(&facts);
    assert_eq!(
        solution.conflicts,
        vec![GrowConflict::LiteralExpression("f".to_string())]
    );
}

#[test]
fn a_non_array_write_in_a_growable_component_is_a_conflict_at_that_node() {
    // A-7: `function f(a) { a.push(1); } f(xs); f(5);`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.edges.push((b("f", "a"), b("_start", "xs")));
    facts.demands.insert(b("f", "a"));
    facts.non_array_writes.insert(b("f", "a"));
    let solution = solve(&facts);
    assert_eq!(
        solution.conflicts,
        vec![GrowConflict::NonArrayWrite(b("f", "a"))]
    );
}

#[test]
fn a_non_array_write_outside_any_growable_component_is_ignored() {
    let mut facts = GrowFacts::default();
    facts.non_array_writes.insert(b("main", "n"));
    facts.edges.push((b("main", "n"), b("main", "m")));
    assert!(solve(&facts).conflicts.is_empty());
}

#[test]
fn growable_members_lists_every_node_of_every_growable_component() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("main", "a"));
    facts.demands.insert(b("main", "a"));
    facts.edges.push((b("main", "b"), b("main", "a")));
    facts.literal_origins.insert(b("main", "c"));
    let solution = solve(&facts);
    let members: Vec<GrowNode> = solution.growable_members().cloned().collect();
    assert_eq!(members, vec![b("main", "a"), b("main", "b")]);
    assert_eq!(
        solution.members_of(&b("main", "b")),
        &[b("main", "a"), b("main", "b")][..]
    );
    assert!(solution.members_of(&b("main", "zzz")).is_empty());
}
