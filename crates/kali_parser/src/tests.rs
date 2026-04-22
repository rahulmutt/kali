use super::*;
use kali_lexer::Lexer;

fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let result = lexer.lex_all();
    result.tokens
}

#[test]
fn test_parse_var_declaration() {
    let tokens = lex("var x = 1;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            assert_eq!(vd.kind, "var");
            assert_eq!(vd.declarations.len(), 1);
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_parse_side_effect_import_declaration() {
    let tokens = lex("import \"mod\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ImportDeclaration(decl) => {
            assert_eq!(decl.source, "mod");
            assert_eq!(decl.specifiers, vec![ImportSpecifier::SideEffect]);
        }
        _ => panic!("Expected ImportDeclaration"),
    }
}

#[test]
fn test_parse_default_import_declaration() {
    let tokens = lex("import value from \"mod\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ImportDeclaration(decl) => {
            assert_eq!(decl.source, "mod");
            assert_eq!(
                decl.specifiers,
                vec![ImportSpecifier::Default("value".to_string())]
            );
        }
        _ => panic!("Expected ImportDeclaration"),
    }
}

#[test]
fn test_parse_dynamic_import_expression() {
    let tokens = lex("const mod = import(\"./lazy\");");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            let init = vd.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ImportExpression(expr) => match &expr.source {
                    Expression::Literal(kali_ast::LiteralValue::String(source)) => {
                        assert_eq!(source, "\"./lazy\"")
                    }
                    other => panic!("unexpected import source: {other:?}"),
                },
                other => panic!("Expected ImportExpression, got {other:?}"),
            }
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_parse_bigint_literal_expression() {
    let tokens = lex("const value = 42n;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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

#[test]
fn test_parse_optional_chain_member_expression() {
    let tokens = lex("minVersion(\"^1.2.3\")?.version;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::OptionalChainExpression(_) => {}
            other => panic!("Expected OptionalChainExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_optional_chain_index_expression() {
    let tokens = lex("call()?.[expr];");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::OptionalChainExpression(_) => {}
            other => panic!("Expected OptionalChainExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}
