use crate::test_support::lex;
use crate::*;
use kali_ast::{ExportSpecifier, ImportSpecifier, Statement};

#[test]
fn test_parse_side_effect_import_declaration() {
    let tokens = lex("import \"mod\";");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportNamed(decl) => {
            assert_eq!(decl.source.as_deref(), Some("./helper.ts"));
            assert_eq!(
                decl.specifiers,
                vec![ExportSpecifier {
                    local: "quadruple".to_string(),
                    exported: "quadruple".to_string(),
                }]
            );
        }
        other => panic!("Expected ExportNamedDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_named_export_declaration_allows_default_aliases() {
    let tokens = lex("export { default as bridged } from \"./helper.ts\";");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportNamed(decl) => {
            assert_eq!(decl.source.as_deref(), Some("./helper.ts"));
            assert_eq!(
                decl.specifiers,
                vec![ExportSpecifier {
                    local: "default".to_string(),
                    exported: "bridged".to_string(),
                }]
            );
        }
        other => panic!("Expected ExportNamedDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_export_all_declaration() {
    let tokens = lex("export * from \"./helper.ts\";");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
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
fn test_parse_default_export_generator_function_declaration() {
    let tokens = lex("export default function* main() { yield 1; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(function) => {
                assert_eq!(function.name, "main");
                assert!(!function.is_async);
                assert!(function.generator);
            }
            other => panic!("Expected function declaration export, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_async_generator_function_declaration() {
    let tokens = lex("export default async function* main() { yield 1; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(function) => {
                assert_eq!(function.name, "main");
                assert!(function.is_async);
                assert!(function.generator);
            }
            other => panic!("Expected function declaration export, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_anonymous_async_generator_function_declaration() {
    let tokens = lex("export default async function*() { yield 1; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(function) => {
                assert_eq!(function.name, "");
                assert!(function.is_async);
                assert!(function.generator);
            }
            other => panic!("Expected function declaration export, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_anonymous_generator_function_declaration() {
    let tokens = lex("export default function*() { yield* []; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(function) => {
                assert_eq!(function.name, "");
                assert!(!function.is_async);
                assert!(function.generator);
            }
            other => panic!("Expected function declaration export, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}
