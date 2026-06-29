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
