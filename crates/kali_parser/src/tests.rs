use super::*;
use kali_ast::{
    AssignmentOperator, ObjectExpression, ObjectPropertyKind, PropertyName, UpdateOperator,
};
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
fn test_parse_prefix_update_expression() {
    let tokens = lex("++value;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UpdateExpression(update) = expr_stmt.expression.as_ref() else {
        panic!("Expected UpdateExpression");
    };
    assert!(update.prefix);
    assert!(matches!(update.operator, UpdateOperator::Increment));
    assert!(matches!(update.argument, Expression::Identifier(_)));
}

#[test]
fn test_parse_postfix_update_expression() {
    let tokens = lex("value--;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UpdateExpression(update) = expr_stmt.expression.as_ref() else {
        panic!("Expected UpdateExpression");
    };
    assert!(!update.prefix);
    assert!(matches!(update.operator, UpdateOperator::Decrement));
    assert!(matches!(update.argument, Expression::Identifier(_)));
}

#[test]
fn test_parse_for_of_statement() {
    let tokens = lex("for (const value of items) { console.log(value); }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                kali_ast::ForOfLefthand::VariableDeclaration(decl) => {
                    assert_eq!(decl.kind, "const");
                    assert_eq!(decl.declarations.len(), 1);
                    assert_eq!(decl.declarations[0].id, "value");
                    assert!(decl.declarations[0].init.is_none());
                }
                other => panic!("Expected variable declaration left-hand, got {other:?}"),
            }
            match &stmt.right {
                Expression::Identifier(name) => assert_eq!(name, "items"),
                other => panic!("Expected identifier right-hand, got {other:?}"),
            }
        }
        other => panic!("Expected ForOfStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_array_expression_with_spread_element() {
    let tokens = lex("const values = [...items, 1];");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
fn test_parse_named_export_declaration() {
    let tokens = lex("export { quadruple } from \"./helper.ts\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportNamed(decl) => {
            assert_eq!(decl.source.as_deref(), Some("./helper.ts"));
            assert_eq!(
                decl.specifiers,
                vec![kali_ast::ExportSpecifier {
                    local: "quadruple".to_string(),
                    exported: "quadruple".to_string(),
                }]
            );
        }
        other => panic!("Expected ExportNamedDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_export_all_declaration() {
    let tokens = lex("export * from \"./helper.ts\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportAll(decl) => {
            assert_eq!(decl.source, "./helper.ts");
        }
        other => panic!("Expected ExportAllDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_function_declaration() {
    let tokens = lex("export default function main() { return 1; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(function) => {
                assert_eq!(function.name, "main");
                assert!(!function.is_async);
                assert!(!function.generator);
            }
            other => panic!("Expected function declaration export, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_export_async_function_declaration() {
    let tokens = lex("export async function main() { await value; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert_eq!(decl.name, "main");
            assert!(decl.is_async);
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_await_expression() {
    let tokens = lex("async function main() { await Promise.resolve(7); }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(decl.is_async, "expected async flag to be preserved");
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &decl.body.body[0] else {
        panic!("Expected ExpressionStatement, got {:?}", decl.body.body[0]);
    };
    let Expression::AwaitExpression(await_expr) = expr_stmt.expression.as_ref() else {
        panic!("Expected AwaitExpression, got {:?}", expr_stmt.expression);
    };
    assert!(matches!(await_expr.argument, Expression::CallExpression(_)));
}

#[test]
fn test_parse_function_declaration_stops_before_following_statement() {
    let tokens = lex("function add(a, b) { return a + b; } add(1, 2);");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 2);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ReturnStatement(return_stmt) = &decl.body.body[0] else {
        panic!("Expected ReturnStatement, got {:?}", decl.body.body[0]);
    };
    assert!(return_stmt.argument.is_some());

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[1] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[1]
        );
    };
    assert!(matches!(
        expr_stmt.expression.as_ref(),
        Expression::CallExpression(_)
    ));
}

#[test]
fn test_parse_for_await_of_statement() {
    let tokens = lex("async function main() { for await (const item of items) { item; } }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(decl.is_async, "expected async flag to be preserved");
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ForOfStatement(stmt) = &decl.body.body[0] else {
        panic!("Expected ForOfStatement, got {:?}", decl.body.body[0]);
    };
    assert!(stmt.is_await, "expected for-await-of flag to be preserved");
    match &stmt.left {
        kali_ast::ForOfLefthand::VariableDeclaration(decl) => {
            assert_eq!(decl.kind, "const");
            assert_eq!(decl.declarations[0].id, "item");
        }
        other => panic!("Expected variable declaration left-hand, got {other:?}"),
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
fn test_parse_nullish_coalescing_expression() {
    let tokens = lex("const value = null ?? 1;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let Expression::BinaryExpression(expr) = init else {
        panic!("Expected BinaryExpression, got {init:?}");
    };
    assert_eq!(expr.operator, "??");
}

#[test]
fn test_parse_exponentiation_expression() {
    let tokens = lex("const value = 2 ** 3 ** 2;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let Expression::BinaryExpression(expr) = init else {
        panic!("Expected BinaryExpression, got {init:?}");
    };
    assert_eq!(expr.operator, "**");
    let Expression::BinaryExpression(right_expr) = expr.right.as_ref() else {
        panic!(
            "Expected nested BinaryExpression on the right, got {:?}",
            expr.right
        );
    };
    assert_eq!(right_expr.operator, "**");
}

#[test]
fn test_parse_compound_assignment_expression() {
    let tokens = lex("value += 1; value **= 2; value %= 3; value &&= 4; value ||= 5;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 5);

    let Statement::ExpressionStatement(first) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::AssignmentExpression(first_assign) = first.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", first.expression);
    };
    assert!(matches!(
        first_assign.operator,
        AssignmentOperator::AddAssign
    ));

    let Statement::ExpressionStatement(second) = &output.statements[1] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[1]
        );
    };
    let Expression::AssignmentExpression(second_assign) = second.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", second.expression);
    };
    assert!(matches!(
        second_assign.operator,
        AssignmentOperator::ExponentAssign
    ));

    let Statement::ExpressionStatement(third) = &output.statements[2] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[2]
        );
    };
    let Expression::AssignmentExpression(third_assign) = third.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", third.expression);
    };
    assert!(matches!(
        third_assign.operator,
        AssignmentOperator::ModuloAssign
    ));

    let Statement::ExpressionStatement(fourth) = &output.statements[3] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[3]
        );
    };
    let Expression::AssignmentExpression(fourth_assign) = fourth.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", fourth.expression);
    };
    assert!(matches!(
        fourth_assign.operator,
        AssignmentOperator::AndAssign
    ));

    let Statement::ExpressionStatement(fifth) = &output.statements[4] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[4]
        );
    };
    let Expression::AssignmentExpression(fifth_assign) = fifth.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", fifth.expression);
    };
    assert!(matches!(
        fifth_assign.operator,
        AssignmentOperator::OrAssign
    ));
}

#[test]
fn test_parse_object_literal_expression() {
    let tokens = lex("const obj = { [\"a\"]: 1, [3]: 4, c };\n");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
fn test_parse_object_literal_expression_rejects_dynamic_computed_property_names() {
    let tokens = lex("const obj = { [value]: 1 };\n");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("computed object property names")),
        "expected computed object property names to be gated: {:?}",
        output.diagnostics
    );
}

#[test]
fn test_parse_bracketed_member_expression_chain() {
    let tokens = lex(
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Proxy"]["revocable"]({}, {}); globalThis["Object"]["hasOwn"]({}, "a"); globalThis["Deno"]["exit"]; globalThis["Deno"]["pid"]; globalThis["Deno"]["env"]["get"]("HOME"); globalThis["Deno"]["permissions"]["query"]("read"); globalThis["Intl"]["Locale"]; globalThis["WeakRef"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["PluralRules"]; globalThis["process"]["cwd"]; globalThis["process"]["exit"];"#,
    );
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 13);

    let Statement::ExpressionStatement(first_stmt) = &output.statements[0] else {
        panic!(
            "Expected first ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::MemberExpression(first_member) = first_stmt.expression.as_ref() else {
        panic!(
            "Expected first bracketed MemberExpression, got {:?}",
            first_stmt.expression
        );
    };
    assert_eq!(first_member.property, "DateTimeFormat");
    let Expression::MemberExpression(first_root) = &first_member.object else {
        panic!("Expected first member root, got {:?}", first_member.object);
    };
    assert_eq!(first_root.property, "Intl");
    assert!(matches!(first_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(second_stmt) = &output.statements[1] else {
        panic!(
            "Expected second ExpressionStatement, got {:?}",
            output.statements[1]
        );
    };
    let Expression::CallExpression(second_call) = second_stmt.expression.as_ref() else {
        panic!(
            "Expected bracketed call expression, got {:?}",
            second_stmt.expression
        );
    };
    assert_eq!(second_call.args.len(), 2);
    let Expression::MemberExpression(second_member) = &second_call.callee else {
        panic!(
            "Expected second bracketed callee, got {:?}",
            second_call.callee
        );
    };
    assert_eq!(second_member.property, "revocable");
    let Expression::MemberExpression(second_root) = &second_member.object else {
        panic!(
            "Expected second member root, got {:?}",
            second_member.object
        );
    };
    assert_eq!(second_root.property, "Proxy");
    assert!(matches!(second_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(third_stmt) = &output.statements[2] else {
        panic!(
            "Expected third ExpressionStatement, got {:?}",
            output.statements[2]
        );
    };
    let Expression::CallExpression(third_call) = third_stmt.expression.as_ref() else {
        panic!(
            "Expected third bracketed call expression, got {:?}",
            third_stmt.expression
        );
    };
    assert_eq!(third_call.args.len(), 2);
    let Expression::MemberExpression(third_member) = &third_call.callee else {
        panic!(
            "Expected third bracketed callee, got {:?}",
            third_call.callee
        );
    };
    assert_eq!(third_member.property, "hasOwn");
    let Expression::MemberExpression(third_root) = &third_member.object else {
        panic!("Expected third member root, got {:?}", third_member.object);
    };
    assert_eq!(third_root.property, "Object");
    assert!(matches!(third_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(fourth_stmt) = &output.statements[3] else {
        panic!(
            "Expected fourth ExpressionStatement, got {:?}",
            output.statements[3]
        );
    };
    let Expression::MemberExpression(fourth_member) = fourth_stmt.expression.as_ref() else {
        panic!(
            "Expected fourth bracketed member expression, got {:?}",
            fourth_stmt.expression
        );
    };
    assert_eq!(fourth_member.property, "exit");
    let Expression::MemberExpression(fourth_root) = &fourth_member.object else {
        panic!(
            "Expected fourth member root, got {:?}",
            fourth_member.object
        );
    };
    assert_eq!(fourth_root.property, "Deno");
    assert!(matches!(fourth_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(fifth_stmt) = &output.statements[4] else {
        panic!(
            "Expected fifth ExpressionStatement, got {:?}",
            output.statements[4]
        );
    };
    let Expression::MemberExpression(fifth_member) = fifth_stmt.expression.as_ref() else {
        panic!(
            "Expected fifth bracketed member expression, got {:?}",
            fifth_stmt.expression
        );
    };
    assert_eq!(fifth_member.property, "pid");
    let Expression::MemberExpression(fifth_root) = &fifth_member.object else {
        panic!("Expected fifth member root, got {:?}", fifth_member.object);
    };
    assert_eq!(fifth_root.property, "Deno");
    assert!(matches!(fifth_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(sixth_stmt) = &output.statements[5] else {
        panic!(
            "Expected sixth ExpressionStatement, got {:?}",
            output.statements[5]
        );
    };
    let Expression::CallExpression(sixth_call) = sixth_stmt.expression.as_ref() else {
        panic!(
            "Expected sixth bracketed call expression, got {:?}",
            sixth_stmt.expression
        );
    };
    assert_eq!(sixth_call.args.len(), 1);
    let Expression::MemberExpression(sixth_member) = &sixth_call.callee else {
        panic!(
            "Expected sixth callee member expression, got {:?}",
            sixth_call.callee
        );
    };
    assert_eq!(sixth_member.property, "get");
    let Expression::MemberExpression(sixth_env) = &sixth_member.object else {
        panic!("Expected sixth env member, got {:?}", sixth_member.object);
    };
    assert_eq!(sixth_env.property, "env");
    let Expression::MemberExpression(sixth_root) = &sixth_env.object else {
        panic!("Expected sixth root member, got {:?}", sixth_env.object);
    };
    assert_eq!(sixth_root.property, "Deno");
    assert!(matches!(sixth_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(seventh_stmt) = &output.statements[6] else {
        panic!(
            "Expected seventh ExpressionStatement, got {:?}",
            output.statements[6]
        );
    };
    let Expression::CallExpression(seventh_call) = seventh_stmt.expression.as_ref() else {
        panic!(
            "Expected seventh bracketed call expression, got {:?}",
            seventh_stmt.expression
        );
    };
    assert_eq!(seventh_call.args.len(), 1);
    let Expression::MemberExpression(seventh_member) = &seventh_call.callee else {
        panic!(
            "Expected seventh callee member expression, got {:?}",
            seventh_call.callee
        );
    };
    assert_eq!(seventh_member.property, "query");
    let Expression::MemberExpression(seventh_permissions) = &seventh_member.object else {
        panic!(
            "Expected seventh permissions member, got {:?}",
            seventh_member.object
        );
    };
    assert_eq!(seventh_permissions.property, "permissions");
    let Expression::MemberExpression(seventh_root) = &seventh_permissions.object else {
        panic!(
            "Expected seventh root member, got {:?}",
            seventh_permissions.object
        );
    };
    assert_eq!(seventh_root.property, "Deno");
    assert!(
        matches!(seventh_root.object, Expression::Identifier(ref name) if name == "globalThis")
    );

    let Statement::ExpressionStatement(eighth_stmt) = &output.statements[7] else {
        panic!(
            "Expected eighth ExpressionStatement, got {:?}",
            output.statements[7]
        );
    };
    let Expression::MemberExpression(eighth_member) = eighth_stmt.expression.as_ref() else {
        panic!(
            "Expected eighth bracketed member expression, got {:?}",
            eighth_stmt.expression
        );
    };
    assert_eq!(eighth_member.property, "Locale");
    let Expression::MemberExpression(eighth_root) = &eighth_member.object else {
        panic!(
            "Expected eighth member root, got {:?}",
            eighth_member.object
        );
    };
    assert_eq!(eighth_root.property, "Intl");
    assert!(matches!(eighth_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(ninth_stmt) = &output.statements[8] else {
        panic!(
            "Expected ninth ExpressionStatement, got {:?}",
            output.statements[8]
        );
    };
    let Expression::MemberExpression(ninth_member) = ninth_stmt.expression.as_ref() else {
        panic!(
            "Expected ninth bracketed member expression, got {:?}",
            ninth_stmt.expression
        );
    };
    assert_eq!(ninth_member.property, "WeakRef");
    assert!(
        matches!(ninth_member.object, Expression::Identifier(ref name) if name == "globalThis")
    );

    let Statement::ExpressionStatement(tenth_stmt) = &output.statements[9] else {
        panic!(
            "Expected tenth ExpressionStatement, got {:?}",
            output.statements[9]
        );
    };
    let Expression::MemberExpression(tenth_member) = tenth_stmt.expression.as_ref() else {
        panic!(
            "Expected tenth bracketed member expression, got {:?}",
            tenth_stmt.expression
        );
    };
    assert_eq!(tenth_member.property, "DisplayNames");
    let Expression::MemberExpression(tenth_root) = &tenth_member.object else {
        panic!("Expected tenth member root, got {:?}", tenth_member.object);
    };
    assert_eq!(tenth_root.property, "Intl");
    assert!(matches!(tenth_root.object, Expression::Identifier(ref name) if name == "globalThis"));

    let Statement::ExpressionStatement(eleventh_stmt) = &output.statements[10] else {
        panic!(
            "Expected eleventh ExpressionStatement, got {:?}",
            output.statements[10]
        );
    };
    let Expression::MemberExpression(eleventh_member) = eleventh_stmt.expression.as_ref() else {
        panic!(
            "Expected eleventh bracketed member expression, got {:?}",
            eleventh_stmt.expression
        );
    };
    assert_eq!(eleventh_member.property, "PluralRules");
    let Expression::MemberExpression(eleventh_root) = &eleventh_member.object else {
        panic!(
            "Expected eleventh member root, got {:?}",
            eleventh_member.object
        );
    };
    assert_eq!(eleventh_root.property, "Intl");
    assert!(
        matches!(eleventh_root.object, Expression::Identifier(ref name) if name == "globalThis")
    );

    let Statement::ExpressionStatement(twelfth_stmt) = &output.statements[11] else {
        panic!(
            "Expected twelfth ExpressionStatement, got {:?}",
            output.statements[11]
        );
    };
    let Expression::MemberExpression(twelfth_member) = twelfth_stmt.expression.as_ref() else {
        panic!(
            "Expected twelfth bracketed member expression, got {:?}",
            twelfth_stmt.expression
        );
    };
    assert_eq!(twelfth_member.property, "cwd");
    let Expression::MemberExpression(twelfth_root) = &twelfth_member.object else {
        panic!(
            "Expected twelfth member root, got {:?}",
            twelfth_member.object
        );
    };
    assert_eq!(twelfth_root.property, "process");
    assert!(
        matches!(twelfth_root.object, Expression::Identifier(ref name) if name == "globalThis")
    );

    let Statement::ExpressionStatement(thirteenth_stmt) = &output.statements[12] else {
        panic!(
            "Expected thirteenth ExpressionStatement, got {:?}",
            output.statements[12]
        );
    };
    let Expression::MemberExpression(thirteenth_member) = thirteenth_stmt.expression.as_ref()
    else {
        panic!(
            "Expected thirteenth bracketed member expression, got {:?}",
            thirteenth_stmt.expression
        );
    };
    assert_eq!(thirteenth_member.property, "exit");
    let Expression::MemberExpression(thirteenth_root) = &thirteenth_member.object else {
        panic!(
            "Expected thirteenth member root, got {:?}",
            thirteenth_member.object
        );
    };
    assert_eq!(thirteenth_root.property, "process");
    assert!(
        matches!(thirteenth_root.object, Expression::Identifier(ref name) if name == "globalThis")
    );
}

#[test]
fn test_parse_fully_bracketed_permission_escalation_member_expression_chain() {
    let tokens = lex(
        r#"Deno["permissions"]["request"](); Deno["permissions"]["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    );
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 4);
}

#[test]
fn test_parse_mixed_bracket_dot_late_object_model_member_expression_chain() {
    let tokens = lex(
        r#"globalThis["Proxy"].revocable({}, {}); globalThis["Object"].hasOwn({}, "a"); globalThis.Object["prototype"].hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a");"#,
    );
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 4);
}

#[test]
fn test_parse_dot_delete_member_expression_after_keyword_property() {
    let tokens = lex("Deno.env.delete('KALI_ENV_DELETE_SMOKE');");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(stmt) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::CallExpression(call) = stmt.expression.as_ref() else {
        panic!("Expected CallExpression, got {:?}", stmt.expression);
    };
    let Expression::MemberExpression(member) = &call.callee else {
        panic!("Expected member expression callee, got {:?}", call.callee);
    };
    assert_eq!(member.property, "delete");
    let Expression::MemberExpression(root) = &member.object else {
        panic!("Expected member root, got {:?}", member.object);
    };
    assert_eq!(root.property, "env");
    let Expression::Identifier(deno) = &root.object else {
        panic!("Expected Deno root, got {:?}", root.object);
    };
    assert_eq!(deno, "Deno");
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

#[test]
fn test_parse_type_assertion_expression() {
    let tokens = lex("value as Foo;");
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
            Expression::TypeAssertion(assertion) => {
                assert_eq!(assertion.type_name, "Foo");
                match assertion.expression.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "value"),
                    other => panic!("Expected Identifier, got {other:?}"),
                }
            }
            other => panic!("Expected TypeAssertion, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_satisfies_expression() {
    let tokens = lex("value satisfies Foo;");
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
            Expression::SatisfiesExpression(satisfies) => {
                assert_eq!(satisfies.type_name, "Foo");
                match satisfies.expression.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "value"),
                    other => panic!("Expected Identifier, got {other:?}"),
                }
            }
            other => panic!("Expected SatisfiesExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_function_declaration() {
    let tokens = lex("function* main() { yield 1; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(!decl.is_async, "expected async flag to be false");
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.name, "main");
            assert_eq!(decl.params.len(), 0);
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(!yield_expr.delegate);
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
                                assert_eq!(*value, 1.0)
                            }
                            other => panic!("unexpected yield argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_delegating_yield_expression() {
    let tokens = lex("function* main() { yield* other(); }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(
                            yield_expr.delegate,
                            "expected yield* delegation to be preserved"
                        );
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::CallExpression(call_expr) => {
                                assert_eq!(call_expr.args.len(), 0)
                            }
                            other => panic!("unexpected yield* argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_function_expression() {
    let tokens = lex("const make = function*() { yield 1; };");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
                Expression::FunctionExpression(func) => {
                    assert!(!func.is_async, "expected async flag to be false");
                    assert!(func.generator, "expected generator flag to be preserved");
                    assert!(func.body.as_ref().is_some());
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_parenthesized_arrow_function_expression() {
    let tokens = lex("const add = (left, right) => left + right;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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

#[test]
fn test_parse_async_generator_function_declaration() {
    let tokens = lex("async function* main() { yield 1; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(decl.is_async, "expected async flag to be preserved");
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.name, "main");
            assert_eq!(decl.params.len(), 0);
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(!yield_expr.delegate);
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
                                assert_eq!(*value, 1.0)
                            }
                            other => panic!("unexpected yield argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_function_expression() {
    let tokens = lex("const make = async function*() { yield* other(); };");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
                Expression::FunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert!(func.generator, "expected generator flag to be preserved");
                    assert!(func.body.as_ref().is_some());
                    let body = func.body.as_ref().expect("function body");
                    assert_eq!(body.body.len(), 1);
                    match &body.body[0] {
                        Statement::ExpressionStatement(expr_stmt) => {
                            match expr_stmt.expression.as_ref() {
                                Expression::YieldExpression(yield_expr) => {
                                    assert!(
                                        yield_expr.delegate,
                                        "expected yield* delegation to be preserved"
                                    );
                                    let argument =
                                        yield_expr.argument.as_ref().expect("yield argument");
                                    match argument {
                                        Expression::CallExpression(call_expr) => {
                                            assert_eq!(call_expr.args.len(), 0)
                                        }
                                        other => panic!("unexpected yield* argument: {other:?}"),
                                    }
                                }
                                other => panic!("Expected YieldExpression, got {other:?}"),
                            }
                        }
                        other => panic!("Expected ExpressionStatement, got {other:?}"),
                    }
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_function_expression() {
    let tokens = lex("const make = async function() { return 1; };");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
                Expression::FunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert!(!func.generator, "expected generator flag to be false");
                    assert!(func.body.as_ref().is_some());
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_yield_expression_outside_generator_remains_identifier() {
    let tokens = lex("yield;");
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
            Expression::Identifier(name) => assert_eq!(name, "yield"),
            other => panic!("Expected Identifier, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_try_finally_statement() {
    let tokens = lex("try { value; } finally { other; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::TryStatement(stmt) => {
            assert!(
                stmt.handler.is_none(),
                "unexpected catch clause: {:?}",
                stmt.handler
            );
            assert!(stmt.finalizer.is_some(), "expected finally block");
            assert_eq!(stmt.block.body.len(), 1);
            assert_eq!(stmt.finalizer.as_ref().unwrap().body.len(), 1);
        }
        other => panic!("Expected TryStatement, got {other:?}"),
    }
}
