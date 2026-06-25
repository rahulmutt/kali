use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

#[test]
fn test_parse_bracketed_member_expression_chain() {
    let tokens = lex(
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Proxy"]["revocable"]({}, {}); globalThis["Object"]["hasOwn"]({}, "a"); globalThis["Deno"]["exit"]; globalThis["Deno"]["pid"]; globalThis["Deno"]["env"]["get"]("HOME"); globalThis["Deno"]["permissions"]["query"]("read"); globalThis["Intl"]["Locale"]; globalThis["WeakRef"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["PluralRules"]; globalThis["process"]["cwd"]; globalThis["process"]["exit"];"#,
    );
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
fn test_parse_dot_from_member_expression_after_keyword_property() {
    let tokens = lex("Array.from([1, 2]);");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    assert_eq!(member.property, "from");
    let Expression::Identifier(array) = &member.object else {
        panic!("Expected Array root, got {:?}", member.object);
    };
    assert_eq!(array, "Array");
}

#[test]
fn test_parse_optional_chain_member_expression() {
    let tokens = lex("minVersion(\"^1.2.3\")?.version;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
