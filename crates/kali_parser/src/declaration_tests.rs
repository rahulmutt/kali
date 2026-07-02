use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

fn assert_parse_class_method_modifiers_are_preserved(
    source: &str,
    is_async: bool,
    generator: bool,
) {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ClassDeclaration(class_decl) => {
            assert_eq!(class_decl.body.methods.len(), 1);
            let method = &class_decl.body.methods[0];
            assert_eq!(method.name, "main");
            assert_eq!(method.is_async, is_async);
            assert_eq!(method.generator, generator);
            assert!(
                method.body.is_some(),
                "expected class method body to be preserved"
            );
        }
        other => panic!("Expected ClassDeclaration, got {other:?}"),
    }
}

#[path = "declaration_tests/arrow.rs"]
mod arrow;

#[path = "declaration_tests/generator.rs"]
mod generator;

#[path = "declaration_tests/class_method.rs"]
mod class_method;

#[path = "declaration_tests/function.rs"]
mod function;

#[test]
fn test_parse_block_bodied_arrow_declarator_init_as_function_expression() {
    let tokens = lex("const bump = () => { console.log(\"bump\"); return 2; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1, "{:?}", output.statements);
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.kind, "const");
    assert_eq!(decl.declarations[0].id, "bump");
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    let Expression::FunctionExpression(function) = init else {
        panic!("expected function-expression init, got {init:?}");
    };
    assert_eq!(function.id, None);
    assert!(function.params.is_empty());
    assert!(!function.is_async);
    assert!(!function.generator);
    let body = function.body.as_ref().expect("function body");
    assert_eq!(body.body.len(), 2, "{:?}", body.body);
    assert!(matches!(body.body[1], Statement::ReturnStatement(_)));
}

#[test]
fn test_parse_block_bodied_arrow_declarator_init_with_params() {
    let tokens = lex("const add = (a, b) => { return a + b; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    let Expression::FunctionExpression(function) = init else {
        panic!("expected function-expression init, got {init:?}");
    };
    let names: Vec<&str> = function
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn test_parse_expression_bodied_arrow_declarator_init_stays_arrow() {
    let tokens = lex("const f = (x) => x + 1;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    assert!(
        matches!(init, Expression::ArrowFunctionExpression(_)),
        "expression-bodied arrows must keep their existing AST shape: {init:?}"
    );
}
