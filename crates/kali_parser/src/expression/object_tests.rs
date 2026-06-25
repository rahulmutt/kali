use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyName, Statement};

#[test]
fn test_parse_object_literal_expression() {
    let tokens = lex("const obj = { [\"a\"]: 1, [3]: 4, c };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let Expression::ObjectExpression(ObjectExpression { properties }) = init else {
        panic!("Expected ObjectExpression, got {init:?}");
    };
    assert_eq!(properties.len(), 3);

    let expected = [
        (
            PropertyName::String("a".to_string()),
            Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
        ),
        (
            PropertyName::Number(3.0),
            Expression::Literal(kali_ast::LiteralValue::Number(4.0)),
        ),
        (
            PropertyName::Identifier("c".to_string()),
            Expression::Identifier("c".to_string()),
        ),
    ];

    for (property, (expected_key, expected_value)) in properties.iter().zip(expected.iter()) {
        assert_eq!(property.kind, ObjectPropertyKind::Init);
        assert_eq!(&property.key, expected_key);
        assert_eq!(&property.value, expected_value);
    }
}

#[test]
fn test_parse_object_literal_expression_with_direct_numeric_property_names() {
    let tokens = lex("const obj = { 3: 4, 1: 2, c: 7 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let Expression::ObjectExpression(ObjectExpression { properties }) = init else {
        panic!("Expected ObjectExpression, got {init:?}");
    };
    assert_eq!(properties.len(), 3);

    let expected = [
        (
            PropertyName::Number(3.0),
            Expression::Literal(kali_ast::LiteralValue::Number(4.0)),
        ),
        (
            PropertyName::Number(1.0),
            Expression::Literal(kali_ast::LiteralValue::Number(2.0)),
        ),
        (
            PropertyName::Identifier("c".to_string()),
            Expression::Literal(kali_ast::LiteralValue::Number(7.0)),
        ),
    ];

    for (property, (expected_key, expected_value)) in properties.iter().zip(expected.iter()) {
        assert_eq!(property.kind, ObjectPropertyKind::Init);
        assert_eq!(&property.key, expected_key);
        assert_eq!(&property.value, expected_value);
    }
}

#[test]
fn test_parse_object_literal_expression_accepts_transparent_wrapper_computed_property_names() {
    let tokens = lex("const obj = { [(0, \"answer\")]: 1, [(\"value\" as Foo)]: 2 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let Some(Expression::ObjectExpression(obj)) = decl.declarations[0].init.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            decl.declarations[0].init
        );
    };
    assert_eq!(obj.properties.len(), 2);
    assert_eq!(
        obj.properties[0].key,
        PropertyName::String("answer".to_string())
    );
    assert_eq!(
        obj.properties[1].key,
        PropertyName::String("value".to_string())
    );
}

#[test]
fn test_parse_object_literal_expression_accepts_frozen_computed_property_names() {
    let tokens = lex(
        "const obj = { [Object.freeze(\"answer\")]: 1, [globalThis.Object.freeze((+2))]: 2 };\n",
    );
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let Some(Expression::ObjectExpression(obj)) = decl.declarations[0].init.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            decl.declarations[0].init
        );
    };
    assert_eq!(obj.properties.len(), 2);
    assert_eq!(
        obj.properties[0].key,
        PropertyName::String("answer".to_string())
    );
    assert_eq!(obj.properties[1].key, PropertyName::Number(2.0));
}

#[test]
fn test_parse_object_literal_expression_accepts_unary_numeric_computed_property_names() {
    let tokens = lex("const obj = { [-1]: 1, [+2]: 2, [(-0)]: 3 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let Some(Expression::ObjectExpression(obj)) = decl.declarations[0].init.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            decl.declarations[0].init
        );
    };
    assert_eq!(obj.properties.len(), 3);
    assert_eq!(obj.properties[0].key, PropertyName::Number(-1.0));
    assert_eq!(obj.properties[1].key, PropertyName::Number(2.0));
    assert_eq!(obj.properties[2].key, PropertyName::Number(-0.0));
}

#[test]
fn test_parse_object_literal_expression_accepts_await_wrapped_computed_property_names() {
    let tokens =
        lex("async function main() { const obj = { [await \"answer\"]: 1, [await (+2)]: 2 }; }\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(function) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(function.is_async, "expected async function context");
    let Some(Statement::VariableDeclaration(decl)) = function.body.body.first() else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            function.body.body.first()
        );
    };
    let Some(Expression::ObjectExpression(obj)) = decl.declarations[0].init.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            decl.declarations[0].init
        );
    };
    assert_eq!(obj.properties.len(), 2);
    assert_eq!(
        obj.properties[0].key,
        PropertyName::String("answer".to_string())
    );
    assert_eq!(obj.properties[1].key, PropertyName::Number(2.0));
}

#[test]
fn test_parse_object_literal_expression_accepts_nested_await_sequence_wrapped_computed_property_names(
) {
    let tokens = lex(
        "async function main() { const obj = { [(await \"ignored\", \"answer\")]: 1, [await ((0, \"value\"))]: 2 }; }\n",
    );
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(function) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(function.is_async, "expected async function context");
    let Some(Statement::VariableDeclaration(decl)) = function.body.body.first() else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            function.body.body.first()
        );
    };
    let Some(Expression::ObjectExpression(obj)) = decl.declarations[0].init.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            decl.declarations[0].init
        );
    };
    assert_eq!(obj.properties.len(), 2);
    assert_eq!(
        obj.properties[0].key,
        PropertyName::String("answer".to_string())
    );
    assert_eq!(
        obj.properties[1].key,
        PropertyName::String("value".to_string())
    );
}
