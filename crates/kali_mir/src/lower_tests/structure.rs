use super::*;

#[test]
fn test_mir_lowering_preserves_program_shape() {
    let hir = parse_and_lower_hir("const answer = 40 + 2;");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    assert_eq!(mir.nodes[mir.root.0 as usize].kind, MirNodeKind::Program);
    assert_eq!(mir.nodes[mir.root.0 as usize].children.len(), 1);
    assert_eq!(
        mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize].kind,
        MirNodeKind::Decl
    );
    assert!(mir.validate().is_ok());
}

#[test]
fn test_call_expressions_lower_to_call_nodes() {
    let hir = parse_and_lower_hir("foo(bar, 1);");
    let mir = MirLowerer::new().lower_hir_result(&hir);
    let expr_stmt = &mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize];
    let call = expr_stmt
        .children
        .iter()
        .map(|child| &mir.nodes[child.0 as usize])
        .find(|node| node.kind == MirNodeKind::Call)
        .expect("call node");
    assert_eq!(call.children.len(), 3);
}

#[test]
fn test_mir_validation_rejects_out_of_bounds_children() {
    let mir = MirProgram {
        root: MirNodeId::new(0),
        nodes: vec![MirNode {
            kind: MirNodeKind::Program,
            text: None,
            children: vec![MirNodeId::new(1)],
            function_flavor: None,
        }],
        functions: Vec::new(),
        arena_facts: Vec::new(),
        parent_labels: std::collections::BTreeMap::new(),
    };

    let error = mir
        .validate()
        .expect_err("invalid MIR should fail validation");
    assert!(error.contains("MIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}
