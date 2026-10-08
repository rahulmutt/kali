//! Growable-runtime-arrays spec §3.1: what the walk records.

use super::super::elem_proof::ElemProof;
use super::super::flow::{GrowFacts, GrowNode, TempKind, UseKind};

fn facts(src: &str) -> GrowFacts {
    crate::repr_infer::growable_facts_for(&crate::test_support::parse_statements(src))
}

fn b(func: &str, name: &str) -> GrowNode {
    GrowNode::Binding(func.to_string(), name.to_string())
}

fn has_edge(facts: &GrowFacts, x: &GrowNode, y: &GrowNode) -> bool {
    facts
        .edges
        .iter()
        .any(|(a, c)| (a == x && c == y) || (a == y && c == x))
}

fn kinds_of(facts: &GrowFacts, node: &GrowNode) -> Vec<UseKind> {
    facts
        .uses
        .iter()
        .filter(|u| &u.node == node)
        .map(|u| u.kind.clone())
        .collect()
}

#[test]
fn a_literal_declarator_is_an_origin_and_its_push_a_demand_with_element_values() {
    let f = facts(
        "function main() { const out = [1]; out.push(2, x); out.push({a: 1}); out.push(true); }",
    );
    let out = b("main", "out");
    assert!(f.literal_origins.contains(&out));
    assert!(f.demands.contains(&out));
    assert_eq!(
        kinds_of(&f, &out),
        vec![UseKind::Push, UseKind::Push, UseKind::Push]
    );
    let values: Vec<ElemProof> = f
        .element_values
        .iter()
        .filter(|(n, _)| n == &out)
        .map(|(_, v)| v.clone())
        .collect();
    assert_eq!(
        values,
        vec![
            ElemProof::Yes,
            ElemProof::Yes,
            ElemProof::Binding {
                func: "main".to_string(),
                name: "x".to_string(),
            },
            ElemProof::No,
            ElemProof::No,
        ]
    );
}

#[test]
fn a_return_a_call_result_an_argument_and_an_alias_are_edges() {
    let f = facts(
        "function make() { const xs = []; xs.push(1); return xs; }\n\
         function add(a) { a.push(2); }\n\
         const ys = make();\n\
         add(ys);\n\
         const zs = ys;\n",
    );
    assert!(has_edge(
        &f,
        &GrowNode::Return("make".to_string()),
        &b("make", "xs")
    ));
    assert!(has_edge(
        &f,
        &b("_start", "ys"),
        &GrowNode::Return("make".to_string())
    ));
    assert!(has_edge(&f, &b("add", "a"), &b("_start", "ys")));
    assert!(has_edge(&f, &b("_start", "zs"), &b("_start", "ys")));
    assert_eq!(f.calls.len(), 1);
    assert_eq!(f.calls[0].callee, "add");
    assert_eq!(f.calls[0].index, 0);
    assert_eq!(f.calls[0].node, b("_start", "ys"));
}

#[test]
fn a_slice_result_is_a_temporary_joined_to_its_receiver() {
    let f = facts("const a = []; a.push(1); const t = a.slice(1);");
    let slice = f
        .temp_kinds
        .iter()
        .find(|(_, (kind, _))| *kind == TempKind::Slice)
        .map(|(n, _)| GrowNode::Temp(*n))
        .expect("a slice temporary");
    assert!(has_edge(&f, &slice, &b("_start", "a")));
    assert!(has_edge(&f, &b("_start", "t"), &slice));
    assert!(kinds_of(&f, &b("_start", "a")).contains(&UseKind::Slice));
}

#[test]
fn an_allocation_is_a_plain_origin_and_a_literal_argument_a_literal_temporary() {
    let f = facts("function g(a) { return a.length; } const p = new Array(3); g([1, 2]);");
    assert!(f.plain_origins.contains(&b("_start", "p")));
    let literal = f
        .temp_kinds
        .iter()
        .find(|(_, (kind, _))| *kind == TempKind::LiteralExpression)
        .map(|(n, _)| GrowNode::Temp(*n))
        .expect("a literal temporary");
    assert!(has_edge(&f, &b("g", "a"), &literal));
}

#[test]
fn a_function_naming_a_module_binding_records_a_module_read_at_its_own_site() {
    let f = facts("const out = []; out.push(1); function size() { return out.length; }");
    let reads: Vec<_> = f
        .uses
        .iter()
        .filter(|u| u.kind == UseKind::ModuleRead)
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].node, b("_start", "out"));
    assert_eq!(reads[0].site, "size");
}

#[test]
fn a_closure_naming_an_outer_binding_records_a_capture() {
    let f = facts("function m() { const o = []; o.push(1); const g = () => o.length; }");
    assert!(kinds_of(&f, &b("m", "o")).contains(&UseKind::Captured));
}

#[test]
fn a_for_of_records_its_iterable_body_mutations_and_body_calls() {
    let f = facts(
        "function add(arr, v) { arr.push(v); }\n\
         const a = []; a.push(1); const b = [];\n\
         for (const x of a) { b.push(x); add(a, x); }\n",
    );
    assert_eq!(f.loops.len(), 1);
    assert_eq!(f.loops[0].iterable, b("_start", "a"));
    assert_eq!(f.loops[0].mutations, vec![b("_start", "b")]);
    // `add(a, x)` contributes one fact per argument; the array is index 0.
    assert!(f.loops[0]
        .calls
        .iter()
        .any(|c| c.callee == "add" && c.index == 0 && c.node == b("_start", "a")));
    // The push inside `add` belongs to `add`'s body, not to the loop.
    assert_eq!(f.loops[0].mutations.len(), 1);
}

#[test]
fn console_arguments_plain_operands_and_other_methods_are_recorded_as_such() {
    let f = facts("const a = []; a.push(1); console.log(a, a.length); const s = typeof a; a.reverse(); a[\"push\"](2); a.indexOf(1, 1);");
    let kinds = kinds_of(&f, &b("_start", "a"));
    assert!(kinds.contains(&UseKind::Console));
    assert!(kinds.contains(&UseKind::LengthRead));
    assert!(kinds.contains(&UseKind::Plain));
    assert!(kinds.contains(&UseKind::Method("`.reverse()`".to_string())));
    assert!(kinds.contains(&UseKind::Method("`[\"push\"]()`".to_string())));
    assert!(kinds.contains(&UseKind::Search {
        method: "indexOf".to_string(),
        from_index: true
    }));
}

#[test]
fn a_missing_argument_a_bare_return_and_a_scalar_write_are_non_array_writes() {
    let f = facts(
        "function f(a, b) { if (a) return; return a; } const xs = []; f(xs); let y = xs; y = 5;",
    );
    assert!(f.non_array_writes.contains(&b("f", "b")));
    assert!(f
        .non_array_writes
        .contains(&GrowNode::Return("f".to_string())));
    assert!(f.non_array_writes.contains(&b("_start", "y")));
}

#[test]
fn an_index_write_is_a_demand_and_an_element_value_but_not_a_loop_mutation() {
    let f = facts("const a = [0]; for (const x of a) { a[0] = x; } a.length = 0;");
    let a = b("_start", "a");
    assert!(f.demands.contains(&a));
    assert!(f.loops[0].mutations.is_empty());
    let kinds = kinds_of(&f, &a);
    assert!(kinds.contains(&UseKind::IndexWrite));
    assert!(kinds.contains(&UseKind::LengthWrite));
}

#[test]
fn a_class_inside_a_function_is_an_opaque_site_carrying_the_frame_stack() {
    let f = facts("function m() { const o = []; o.push(1); class C {} }");
    assert_eq!(
        f.opaque_sites,
        vec![vec!["_start".to_string(), "m".to_string()]]
    );
}

// Final review C1: every index, `slice` bound and search value is recorded
// against its receiver with its proof.
#[test]
fn indices_bounds_and_search_values_are_recorded_as_operands() {
    use super::super::flow::OperandPosition;
    let f = facts(
        "const a = []; a.push(1); let i = 0; a[i] = a[true]; a.slice(1, null); a.indexOf(undefined); a.includes(2); a.indexOf();",
    );
    let got: Vec<(OperandPosition, ElemProof)> = f
        .operands
        .iter()
        .filter(|(node, _, _)| *node == b("_start", "a"))
        .map(|(_, position, proof)| (*position, proof.clone()))
        .collect();
    let binding = |name: &str| ElemProof::Binding {
        func: "_start".to_string(),
        name: name.to_string(),
    };
    assert_eq!(
        got,
        vec![
            (OperandPosition::Index, binding("i")),
            (OperandPosition::Index, ElemProof::No),
            (OperandPosition::SliceStart, ElemProof::Yes),
            (OperandPosition::SliceEnd, ElemProof::No),
            (OperandPosition::SearchValue, binding("undefined")),
            (OperandPosition::SearchValue, ElemProof::Yes),
        ]
    );
}

#[test]
fn a_literal_assigned_to_a_binding_is_a_literal_assignment_temporary() {
    let f = facts("let a = []; a.push(1); a = [];");
    let literal = f
        .temp_kinds
        .iter()
        .find(|(_, (kind, _))| *kind == TempKind::LiteralAssignment)
        .map(|(n, _)| GrowNode::Temp(*n))
        .expect("a literal-assignment temporary");
    assert!(has_edge(&f, &b("_start", "a"), &literal));
}
