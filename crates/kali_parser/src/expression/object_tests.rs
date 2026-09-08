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

/// Parses `source` (an expression statement) and returns its `ObjectExpression`.
fn parse_object_literal(source: &str) -> ObjectExpression {
    let tokens = lex(&format!("{source};\n"));
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);
    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::ParenthesizedExpression(parenthesized) = expr_stmt.expression.as_ref() else {
        panic!(
            "Expected ParenthesizedExpression, got {:?}",
            expr_stmt.expression
        );
    };
    let Expression::ObjectExpression(obj) = parenthesized.expression.as_ref() else {
        panic!(
            "Expected ObjectExpression, got {:?}",
            parenthesized.expression
        );
    };
    obj.clone()
}

#[test]
fn bigint_object_property_keys_keep_their_digits() {
    let obj = parse_object_literal("({42n: 1})");
    assert_eq!(
        obj.properties[0].key,
        PropertyName::BigInt("42".to_string())
    );
}

#[test]
fn large_bigint_object_property_keys_are_exact() {
    // The whole reason the variant holds text: this value has no exact f64.
    let obj = parse_object_literal("({123456789012345678901234567890n: 1})");
    assert_eq!(
        obj.properties[0].key,
        PropertyName::BigInt("123456789012345678901234567890".to_string())
    );
}

#[test]
fn an_unreadable_numeric_property_key_is_refused_not_fabricated() {
    // Hex/binary/octal keys (`0x10`) do NOT exercise this refusal: the lexer
    // does not tokenize those prefixes at all (a pre-existing, unrelated
    // gap -- `0x10` lexes as the numeric literal `0` followed by the
    // identifier `x10`, two tokens, and never reaches `numeric_property_name`
    // as one). `1.5n` does reach it as a single NumericLiteral token (the
    // lexer accepts a trailing `n` after a decimal fraction with no syntax
    // check), and a decimal BigInt literal is invalid JavaScript, so
    // `numeric_property_name` must refuse it rather than storing a
    // fabricated key.
    let tokens = lex("const obj = { 1.5n: 1 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        !output.diagnostics.is_empty(),
        "expected a diagnostic refusing the unreadable key, got none"
    );
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
    assert_eq!(
        properties.len(),
        0,
        "the unreadable property must be dropped, not stored under a fabricated key"
    );
}

#[test]
fn parsing_resumes_after_refusing_an_unreadable_numeric_key() {
    // Pins the comma resynchronization the refusal arm's `continue` depends
    // on: the property after the refused one must still be parsed.
    let tokens = lex("const obj = { 1.5n: 1, a: 2 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(!output.diagnostics.is_empty());
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
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].key, PropertyName::Identifier("a".to_string()));
}

#[test]
fn a_leading_zero_bigint_property_key_is_refused() {
    // `042n` is a SyntaxError in JavaScript -- BigInt literals may not carry
    // a leading zero -- so admitting it here as the key "042" would accept a
    // program node refuses. `0n` itself (a single "0") stays legal.
    let tokens = lex("const obj = { 042n: 1 };\n");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        !output.diagnostics.is_empty(),
        "expected a diagnostic refusing the leading-zero bigint key, got none"
    );
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
    assert_eq!(
        properties.len(),
        0,
        "the leading-zero bigint key must be dropped, not stored under \"042\""
    );
}
