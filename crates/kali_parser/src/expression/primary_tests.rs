use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, ExpressionOrSpread, Statement};

#[test]
fn test_parse_array_expression_with_spread_element() {
    let tokens = lex("const values = [...items, 1];");
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
    let Some(Expression::ArrayExpression(array)) = vd.declarations[0].init.as_ref() else {
        panic!(
            "Expected ArrayExpression initializer, got {:?}",
            vd.declarations[0].init
        );
    };
    assert_eq!(array.elements.len(), 2);
    match &array.elements[0] {
        Some(ExpressionOrSpread::Spread(spread)) => match &spread.argument {
            Expression::Identifier(name) => assert_eq!(name, "items"),
            other => panic!("Expected spread identifier, got {other:?}"),
        },
        other => panic!("Expected spread element, got {other:?}"),
    }
    match &array.elements[1] {
        Some(ExpressionOrSpread::Expression(Expression::Literal(
            kali_ast::LiteralValue::Number(value),
        ))) => {
            assert_eq!(*value, 1.0)
        }
        other => panic!("Expected literal expression element, got {other:?}"),
    }
}

#[test]
fn test_parse_bigint_literal_expression() {
    let tokens = lex("const value = 42n;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            let init = vd.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::BigIntLiteral(value) => assert_eq!(value, "42n"),
                other => panic!("Expected BigIntLiteral, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}
