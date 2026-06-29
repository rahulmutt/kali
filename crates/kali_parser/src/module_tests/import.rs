use super::*;

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
