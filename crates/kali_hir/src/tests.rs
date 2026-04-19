use super::*;
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

fn parse(source: &str) -> Vec<Statement> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    parser.parse(None).statements
}

#[test]
fn test_hir_builder() {
    let mut builder = HirBuilder::new();
    let root = builder.alloc(HirNodeKind::Program, None);
    assert_eq!(root.0, 0);
    assert_eq!(builder.next_id.0, 1);
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

    let var_decl = &result.nodes[result.nodes[result.root.0 as usize].children[0].0 as usize];
    assert_eq!(var_decl.kind, HirNodeKind::VarDecl);
    assert_eq!(var_decl.text.as_deref(), Some("const"));

    let func_decl = &result.nodes[result.nodes[result.root.0 as usize].children[1].0 as usize];
    assert_eq!(func_decl.kind, HirNodeKind::FunctionDecl);
    assert_eq!(func_decl.text.as_deref(), Some("add"));
}

#[test]
fn test_object_literal_lowers_to_stable_property_shape() {
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(&Expression::ObjectExpression(ObjectExpression {
        properties: vec![ObjectProperty {
            key: PropertyName::Identifier("answer".to_string()),
            value: Expression::Identifier("value".to_string()),
            kind: ObjectPropertyKind::Init,
        }],
    }));

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);
    assert_eq!(property.text.as_deref(), Some("init"));
    assert_eq!(property.children.len(), 2);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("answer"));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}
