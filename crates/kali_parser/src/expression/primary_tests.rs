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

fn parse_single_init_expression(
    source: &str,
) -> (Expression, Vec<kali_error::diagnostic::Diagnostic>) {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(
        output.statements.len(),
        1,
        "statements: {:?}",
        output.statements
    );
    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    (
        vd.declarations[0].init.clone().expect("initializer"),
        output.diagnostics,
    )
}

fn expect_string_literal(expression: &Expression, expected: &str) {
    match expression {
        Expression::Literal(kali_ast::LiteralValue::String(value)) => {
            assert_eq!(value, expected)
        }
        other => panic!("Expected string literal {expected:?}, got {other:?}"),
    }
}

fn expect_plus(expression: &Expression) -> (&Expression, &Expression) {
    match expression {
        Expression::BinaryExpression(expr) if expr.operator == "+" => (&expr.left, &expr.right),
        other => panic!("Expected `+` chain, got {other:?}"),
    }
}

#[test]
fn test_interpolated_template_desugars_to_string_plus_chain() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${7 / 2} end`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    // ((`v: ` + (7 / 2)) + ` end`)
    let (left, right) = expect_plus(&init);
    expect_string_literal(right, "` end`");
    let (quasi, division) = expect_plus(left);
    expect_string_literal(quasi, "`v: `");
    match division {
        Expression::BinaryExpression(expr) => {
            assert_eq!(expr.operator, "/");
            assert_eq!(
                expr.left,
                Expression::Literal(kali_ast::LiteralValue::Number(7.0))
            );
            assert_eq!(
                expr.right,
                Expression::Literal(kali_ast::LiteralValue::Number(2.0))
            );
        }
        other => panic!("Expected division, got {other:?}"),
    }
}

#[test]
fn test_adjacent_interpolations_get_leading_empty_quasi_and_skip_empty_rest() {
    let (init, diagnostics) = parse_single_init_expression("const m = `${a}${b}`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    // ((`` + a) + b) — leading empty quasi kept, later empty quasis skipped
    let (left, right) = expect_plus(&init);
    assert_eq!(*right, Expression::Identifier("b".to_string()));
    let (quasi, a) = expect_plus(left);
    expect_string_literal(quasi, "``");
    assert_eq!(*a, Expression::Identifier("a".to_string()));
}

#[test]
fn test_template_without_interpolation_stays_plain_literal() {
    let (init, diagnostics) = parse_single_init_expression("const m = `hello`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    expect_string_literal(&init, "`hello`");
}

#[test]
fn test_unterminated_interpolation_reports_e2004_and_falls_back_to_raw() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${7`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    expect_string_literal(&init, "`v: ${7`");
}

#[test]
fn test_trailing_tokens_in_interpolation_report_e2004() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${1 2}`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    // Still desugars with the parsed prefix expression: (`v: ` + 1)
    let (quasi, one) = expect_plus(&init);
    expect_string_literal(quasi, "`v: `");
    assert_eq!(
        *one,
        Expression::Literal(kali_ast::LiteralValue::Number(1.0))
    );
}

#[test]
fn test_escaped_interpolation_reports_e2004_and_falls_back_to_raw() {
    let (init, diagnostics) = parse_single_init_expression("const m = `cost: \\${5}`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    expect_string_literal(&init, "`cost: \\${5}`");
}

#[test]
fn test_empty_interpolation_reports_e2004() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${}`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    // Desugared shape still string-valued: (`v: ` + ``)
    let (quasi, empty) = expect_plus(&init);
    expect_string_literal(quasi, "`v: `");
    expect_string_literal(empty, "``");
}

fn expect_number_init(source: &str, expected: f64) {
    let (init, diagnostics) = parse_single_init_expression(source);
    assert!(diagnostics.is_empty(), "{source}: {diagnostics:?}");
    match init {
        Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
            assert_eq!(value, expected, "{source}")
        }
        other => panic!("{source}: expected a number literal, got {other:?}"),
    }
}

#[test]
fn prefixed_separated_and_legacy_octal_numbers_read_their_javascript_value() {
    expect_number_init("const v = 0xff;", 255.0);
    expect_number_init("const v = 0b101;", 5.0);
    expect_number_init("const v = 0o17;", 15.0);
    expect_number_init("const v = 1_000;", 1000.0);
    // Register entry R-58, expression position: `042` is 34, not 42.
    expect_number_init("const v = 042;", 34.0);
    expect_number_init("const v = 08;", 8.0);
}

#[test]
fn a_separated_bigint_keeps_only_its_digits() {
    let (init, diagnostics) = parse_single_init_expression("const v = 1_000n;");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    match init {
        Expression::BigIntLiteral(value) => assert_eq!(value, "1000n"),
        other => panic!("Expected BigIntLiteral, got {other:?}"),
    }
}

#[test]
fn malformed_and_non_decimal_bigint_numbers_are_refused_not_zero() {
    // The parser used to read an unparseable literal as `0.0`. A token it
    // cannot read is refused, whether or not the lexer's E1100 is in view.
    for source in [
        "const v = 0x;",
        "const v = 1__0;",
        "const v = 0xffn;",
        "const v = 1.5n;",
    ] {
        let (_, diagnostics) = parse_single_init_expression(source);
        assert!(!diagnostics.is_empty(), "{source}: expected a refusal");
    }
}
