//! Integration tests for the parser.
//! These tests verify the parser produces correct AST nodes for various constructs.

use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::{Parser, ParserOutput};

fn lex(source: &str) -> Vec<kali_lexer::Token> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let result = lexer.lex_all();
    result.tokens
}

fn parse(source: &str) -> ParserOutput {
    let tokens = lex(source);
    let mut parser = Parser::new(FileId::new(0), tokens);
    parser.parse(None)
}

/// Tests for variable/let/const declarations
mod variable_declarations {
    use super::*;

    #[test]
    fn test_parse_var_declaration() {
        let output = parse("var x = 1;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "var");
                assert_eq!(vd.declarations.len(), 1);
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_let_declaration() {
        let output = parse("let y = 2;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "let");
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_const_declaration() {
        let output = parse("const z = 3;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "const");
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_var_declaration_no_init() {
        let output = parse("var a;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                assert!(vd.declarations[0].init.is_none());
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}

/// Tests for block and function declarations
mod block_and_function {
    use super::*;

    #[test]
    fn test_parse_block_statement() {
        let output = parse("{ let x = 1; };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::BlockStatement(bs) => {
                assert_eq!(bs.body.len(), 1);
            }
            _ => panic!("Expected BlockStatement"),
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let output = parse("function foo() {};");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::FunctionDeclaration(fd) => {
                assert_eq!(fd.name, "foo");
            }
            _ => panic!("Expected FunctionDeclaration"),
        }
    }

    #[test]
    fn test_parse_function_with_params() {
        let output = parse("function bar(x, y) {};");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::FunctionDeclaration(fd) => {
                assert_eq!(fd.params.len(), 2);
            }
            _ => panic!("Expected FunctionDeclaration"),
        }
    }

    #[test]
    fn test_parse_function_with_body() {
        let output = parse("function baz() { return 1; };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::FunctionDeclaration(fd) => {
                assert!(matches!(
                    fd.body.body.first(),
                    Some(kali_ast::Statement::ReturnStatement(_))
                ));
            }
            _ => panic!("Expected FunctionDeclaration"),
        }
    }
}

/// Tests for class declarations
mod class_declarations {
    use super::*;

    #[test]
    fn test_parse_class_declaration() {
        let output = parse("class MyClass {};");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ClassDeclaration(cd) => {
                assert_eq!(cd.name, "MyClass");
            }
            _ => panic!("Expected ClassDeclaration"),
        }
    }

    #[test]
    fn test_parse_class_with_body() {
        let output = parse("class AnotherClass { method() {} };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ClassDeclaration(cd) => {
                assert!(!cd.body.methods.is_empty());
            }
            _ => panic!("Expected ClassDeclaration"),
        }
    }
}

/// Tests for expression statements
mod expression_statements {
    use super::*;

    #[test]
    fn test_parse_call_expression() {
        let output = parse("console.log('hello');");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::CallExpression(_) => {}
                _ => panic!("Expected CallExpression"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_parse_call_expression_with_args() {
        let output = parse("func(a, b, c);");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::CallExpression(ce) => {
                    assert_eq!(ce.args.len(), 3);
                }
                _ => panic!("Expected CallExpression with 3 args"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_parse_call_chain() {
        let output = parse("obj.method().other();");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => {
                // Should have nested call/member expressions
                match es.expression.as_ref() {
                    kali_ast::Expression::CallExpression(_) => {}
                    _ => panic!("Expected nested CallExpression"),
                }
            }
            _ => panic!("Expected ExpressionStatement"),
        }
    }
}

/// Tests for binary expressions - ORIGINALLY FAILING
mod binary_expressions {
    use super::*;

    #[test]
    fn test_parse_binary_expression() {
        let output = parse("let a = 1 + 2 * 3;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::BinaryExpression(be)) => {
                        // 1 + (2 * 3) - multiplication should bind tighter
                        assert_eq!(be.operator, "+");
                        match be.right.as_ref() {
                            kali_ast::Expression::BinaryExpression(inner) => {
                                assert_eq!(inner.operator, "*");
                            }
                            _ => panic!("Expected multiplication inside addition"),
                        }
                    }
                    _ => panic!("Expected BinaryExpression"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_binary_and_operator() {
        let output = parse("let x = a && b;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::BinaryExpression(be)) => {
                        assert_eq!(be.operator, "&&");
                    }
                    _ => panic!("Expected BinaryExpression with &&"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_binary_or_operator() {
        let output = parse("let y = a || b;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::BinaryExpression(be)) => {
                        assert_eq!(be.operator, "||");
                    }
                    _ => panic!("Expected BinaryExpression with ||"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_binary_comparison() {
        let output = parse("let z = a > b;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::BinaryExpression(be)) => {
                        assert_eq!(be.operator, ">");
                    }
                    _ => panic!("Expected BinaryExpression with >"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}

/// Tests for control flow statements - ORIGINALLY FAILING
mod control_flow_statements {
    use super::*;

    #[test]
    fn test_parse_if_statement() {
        let output = parse("if (x > 0) { console.log('positive'); };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::IfStatement(if_stmt) => {
                assert!(!if_stmt.consequent.body.is_empty());
            }
            _ => panic!("Expected IfStatement"),
        }
    }

    #[test]
    fn test_parse_if_with_else() {
        let output = parse("if (x) {} else {};");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::IfStatement(if_stmt) => {
                assert!(if_stmt.alternate.is_some());
            }
            _ => panic!("Expected IfStatement with alternate"),
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let output = parse("while (x < 10) { x++; };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::WhileStatement(ws) => {
                assert!(!ws.body.body.is_empty());
            }
            _ => panic!("Expected WhileStatement"),
        }
    }

    #[test]
    fn test_parse_do_while_statement() {
        let output = parse("do { x++; } while (x < 10);");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::DoWhileStatement(dws) => {
                assert!(!dws.body.body.is_empty());
            }
            _ => panic!("Expected DoWhileStatement"),
        }
    }

    #[test]
    fn test_parse_for_statement() {
        let output = parse("for (let i = 0; i < 10; i++) { console.log(i); };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ForStatement(fs) => {
                assert!(fs.init.is_some());
                assert!(fs.test.is_some());
                assert!(fs.update.is_some());
            }
            _ => panic!("Expected ForStatement"),
        }
    }

    #[test]
    fn test_parse_break_statement() {
        let output = parse("break;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::BreakStatement(_) => {}
            _ => panic!("Expected BreakStatement"),
        }
    }

    #[test]
    fn test_parse_continue_statement() {
        let output = parse("continue;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ContinueStatement(_) => {}
            _ => panic!("Expected ContinueStatement"),
        }
    }

    #[test]
    fn test_parse_throw_statement() {
        let output = parse("throw new Error('test');");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ThrowStatement(ts) => {
                assert!(ts.argument != kali_ast::Expression::Identifier(String::new()));
            }
            _ => panic!("Expected ThrowStatement"),
        }
    }
}

/// Tests for return and try statements
mod return_try {
    use super::*;

    #[test]
    fn test_parse_return_statement() {
        let output = parse("return 42;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ReturnStatement(rs) => {
                assert!(rs.argument.is_some());
            }
            _ => panic!("Expected ReturnStatement"),
        }
    }

    #[test]
    fn test_parse_try_statement() {
        let output = parse("try { } catch (e) { };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::TryStatement(ts) => {
                assert!(ts.handler.is_some());
            }
            _ => panic!("Expected TryStatement"),
        }
    }
}

/// Tests for debugger statement
mod debugger {
    use super::*;

    #[test]
    fn test_parse_debugger_statement() {
        let output = parse("debugger;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::DebuggerStatement(_) => {}
            _ => panic!("Expected DebuggerStatement"),
        }
    }
}

/// Tests for switch statement
mod switch {
    use super::*;

    #[test]
    fn test_parse_switch_statement() {
        let output = parse("switch(x) { case 1: break; default: break; };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::SwitchStatement(ss) => {
                assert!(!ss.cases.is_empty());
            }
            _ => panic!("Expected SwitchStatement"),
        }
    }
}

/// Tests for expression constants
mod constants {
    use super::*;

    #[test]
    fn test_parse_constant() {
        let output = parse("let a = 123;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::Literal(kali_ast::LiteralValue::Number(n))) => {
                        assert_eq!(*n, 123.0);
                    }
                    _ => panic!("Expected Number literal"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_string_constant() {
        let output = parse("let s = 'hello';");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::Literal(kali_ast::LiteralValue::String(s))) => {
                        assert_eq!(s, "'hello'");
                    }
                    _ => panic!("Expected String literal"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_boolean_constant() {
        let output = parse("let b = true;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::Literal(kali_ast::LiteralValue::Boolean(b))) => {
                        assert!(*b);
                    }
                    _ => panic!("Expected Boolean literal"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}

/// Tests for property access expressions
mod member_expressions {
    use super::*;

    #[test]
    fn test_parse_member_expression() {
        let output = parse("obj.property;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::MemberExpression(_) => {}
                _ => panic!("Expected MemberExpression"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_parse_bracketed_string_literal_member_expression() {
        let output = parse("globalThis[\"Deno\"][\"exit\"];");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::MemberExpression(me) => {
                    assert_eq!(me.property, "exit");
                    match &me.object {
                        kali_ast::Expression::MemberExpression(inner) => {
                            assert_eq!(inner.property, "Deno");
                            match &inner.object {
                                kali_ast::Expression::Identifier(name) => {
                                    assert_eq!(name, "globalThis");
                                }
                                other => panic!("Expected globalThis identifier, got {other:?}"),
                            }
                        }
                        other => panic!("Expected nested MemberExpression, got {other:?}"),
                    }
                }
                _ => panic!("Expected MemberExpression"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_parse_bracketed_string_literal_pid_member_expression() {
        let output = parse("globalThis[\"Deno\"][\"pid\"];");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::MemberExpression(me) => {
                    assert_eq!(me.property, "pid");
                    match &me.object {
                        kali_ast::Expression::MemberExpression(inner) => {
                            assert_eq!(inner.property, "Deno");
                            match &inner.object {
                                kali_ast::Expression::Identifier(name) => {
                                    assert_eq!(name, "globalThis");
                                }
                                other => panic!("Expected globalThis identifier, got {other:?}"),
                            }
                        }
                        other => panic!("Expected nested MemberExpression, got {other:?}"),
                    }
                }
                _ => panic!("Expected MemberExpression"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_parse_array_access() {
        let output = parse("arr[index];");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                kali_ast::Expression::MemberExpression(me) => {
                    assert_eq!(me.property, "index");
                }
                _ => panic!("Expected MemberExpression"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }
}

/// Tests for parenthesized expressions
mod parenthesized {
    use super::*;

    #[test]
    fn test_parse_parenthesized_expression() {
        let output = parse("let x = (1 + 2);");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::ParenthesizedExpression(_)) => {}
                    _ => panic!("Expected ParenthesizedExpression"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}

/// Tests for function expressions
mod function_expressions {
    use super::*;

    #[test]
    fn test_parse_function_expression() {
        let output = parse("let fn = function() {}; ");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::FunctionExpression(_)) => {}
                    _ => panic!("Expected FunctionExpression"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}

/// Tests for complex nested structures
mod complex_structures {
    use super::*;

    #[test]
    fn test_parse_nested_for() {
        let output = parse("for(let i = 0; i < 10; i++) { for(let j = 0; j < 5; j++) {} };");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::ForStatement(fs) => match fs.body.body.first() {
                Some(kali_ast::Statement::ForStatement(_)) => {}
                _ => panic!("Expected nested ForStatement"),
            },
            _ => panic!("Expected ForStatement"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let output = parse("let result = a + b * c - d / e;");
        assert_eq!(output.statements.len(), 1);

        match &output.statements[0] {
            kali_ast::Statement::VariableDeclaration(vd) => {
                match vd.declarations[0].init.as_ref() {
                    Some(kali_ast::Expression::BinaryExpression(_)) => {}
                    _ => panic!("Expected BinaryExpression"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }
}
