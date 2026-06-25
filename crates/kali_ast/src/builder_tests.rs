use crate::*;

#[test]
fn test_ast_builder() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let root = builder.get_node(root_id).unwrap();
    assert_eq!(root.kind, NodeKind::Program);

    assert!(builder.root().is_some());
}

#[test]
fn test_ast_conversion() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let ast: AST = builder.into();
    assert!(ast.root().is_some());
}
