use crate::test_support::parse;
use crate::*;
use kali_ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyName};

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

#[test]
fn test_numeric_object_property_names_lower_as_string_literals() {
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(&Expression::ObjectExpression(ObjectExpression {
        properties: vec![ObjectProperty {
            key: PropertyName::Number(3.0),
            value: Expression::Identifier("value".to_string()),
            kind: ObjectPropertyKind::Init,
        }],
    }));

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"3\""));
}

#[test]
fn test_numeric_object_property_names_lower_from_parsed_source_as_string_literals() {
    let statements = parse("const obj = { 3: value };\n");
    let kali_ast::Statement::VariableDeclaration(vd) = &statements[0] else {
        panic!("Expected VariableDeclaration, got {:?}", statements[0]);
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(init);

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"3\""));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}

#[test]
fn test_numeric_object_property_names_lower_negative_zero_as_string_literal_zero() {
    let statements = parse("const obj = { [-0]: value };\n");
    let kali_ast::Statement::VariableDeclaration(vd) = &statements[0] else {
        panic!("Expected VariableDeclaration, got {:?}", statements[0]);
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(init);

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"0\""));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}
