use super::*;

#[test]
fn test_parse_parenthesized_arrow_function_expression() {
    let tokens = lex("const add = (left, right) => left + right;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(!func.is_async, "expected async flag to be false");
                    assert_eq!(func.params.len(), 2);
                    assert_eq!(func.params[0].name, "left");
                    assert_eq!(func.params[1].name, "right");
                    assert!(matches!(func.body, Expression::BinaryExpression(_)));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_single_parameter_arrow_function_expression() {
    let tokens = lex("const identity = value => value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_arrow_function_expression() {
    let tokens = lex("const add = async (left, right) => left + right;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 2);
                    assert_eq!(func.params[0].name, "left");
                    assert_eq!(func.params[1].name, "right");
                    assert!(matches!(func.body, Expression::BinaryExpression(_)));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_arrow_function_return_type_annotation_with_multiple_params() {
    let tokens = lex("const add = async (left, right): number => left + right;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 2);
                    assert_eq!(func.params[0].name, "left");
                    assert_eq!(func.params[1].name, "right");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
                    assert!(matches!(func.body, Expression::BinaryExpression(_)));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_single_parameter_arrow_function_expression() {
    let tokens = lex("const identity = async value => value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_arrow_function_return_type_annotation() {
    let tokens = lex("const identity = async (value): number => value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_arrow_function_return_type_annotation() {
    let tokens = lex("const identity = (value): number => value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ArrowFunctionExpression(func) => {
                    assert!(!func.is_async, "expected async flag to be false");
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}
