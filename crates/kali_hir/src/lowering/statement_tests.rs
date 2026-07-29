use crate::test_support::parse;
use crate::*;
use kali_ast::AST;

#[test]
fn switch_and_its_case_blocks_are_text_tagged() {
    let statements = parse("function f(x) { switch (x) { case 1: return 1; default: return 2; } }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let switch = result
        .nodes
        .iter()
        .find(|node| node.kind == HirNodeKind::SwitchStmt)
        .expect("a SwitchStmt node");
    assert_eq!(switch.text.as_deref(), Some("switch"));

    // children[0] is the discriminant; children[1..] are the case blocks.
    let cases: Vec<_> = switch.children[1..]
        .iter()
        .map(|id| result.nodes[id.0 as usize].text.as_deref())
        .collect();
    assert_eq!(cases, vec![Some("case"), Some("default")]);
}

#[test]
fn test_lower_statements_to_hir() {
    let statements = parse("const answer = 40 + 2; function add(a, b) { return a + b; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.nodes[result.root.0 as usize].kind,
        HirNodeKind::Program
    );
    assert_eq!(result.nodes[result.root.0 as usize].children.len(), 2);
    assert!(result.validate().is_ok());

    let var_decl = &result.nodes[result.nodes[result.root.0 as usize].children[0].0 as usize];
    assert_eq!(var_decl.kind, HirNodeKind::VarDecl);
    assert_eq!(var_decl.text.as_deref(), Some("const"));

    let func_decl = &result.nodes[result.nodes[result.root.0 as usize].children[1].0 as usize];
    assert_eq!(func_decl.kind, HirNodeKind::FunctionDecl);
    assert_eq!(func_decl.text.as_deref(), Some("add"));
}

#[test]
fn test_lower_program_from_ast_matches_statement_lowering_for_empty_ast_shell() {
    let statements = parse("const answer = 40 + 2; function add(a, b) { return a + b; }");
    let mut lowerer = HirLowerer::new();
    let ast = AST::empty();

    let from_ast = lowerer.lower_program_from_ast(&ast, &statements);
    let from_statements = lowerer.lower_statements(&statements);

    assert!(from_ast.diagnostics.is_empty());
    assert_eq!(from_ast.root, from_statements.root);
    assert_eq!(from_ast.nodes, from_statements.nodes);
}

#[test]
fn test_lower_statements_records_export_all_nodes() {
    let statements = parse("export * from './helper.ts';");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert_eq!(result.nodes[result.root.0 as usize].children.len(), 1);
    let export_decl = &result.nodes[result.nodes[result.root.0 as usize].children[0].0 as usize];
    assert_eq!(export_decl.kind, HirNodeKind::ExportDecl);
    assert_eq!(export_decl.text.as_deref(), Some("./helper.ts"));
}
