use super::*;

#[test]
fn test_parse_dynamic_import_expression() {
    let tokens = lex("const mod = import(\"./lazy\");");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
