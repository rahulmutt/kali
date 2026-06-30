use super::*;

#[test]
fn test_lir_lowering_preserves_root() {
    let mir = parse_and_lower("function add(a, b) { return a + b; }");
    let lir = LirLowerer::new().lower_program(&mir);

    assert_eq!(lir.nodes[lir.root.0 as usize].kind, LirNodeKind::Program);
    assert_eq!(lir.nodes[lir.root.0 as usize].children.len(), 1);
    assert!(lir.validate().is_ok());
}

#[test]
fn test_lir_lowering_preserves_child_order_and_text_payloads() {
    let mir = parse_and_lower("const answer = 40 + 2; foo(answer);");
    let lir = LirLowerer::new().lower_program(&mir);
    let root = &lir.nodes[lir.root.0 as usize];

    assert_eq!(root.kind, LirNodeKind::Program);
    assert_eq!(root.children.len(), 2);
    assert!(lir
        .nodes
        .iter()
        .any(|node| node.text.as_deref() == Some("answer")));
    assert!(lir
        .nodes
        .iter()
        .any(|node| node.text.as_deref() == Some("foo")));
}
