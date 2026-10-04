use kali_ast::Statement;
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

pub(crate) fn parse(source: &str) -> Vec<Statement> {
    let lexed = Lexer::new(FileId::new(0), source.to_string()).lex_all();
    assert!(
        lexed.diagnostics.is_empty(),
        "lex diagnostics: {:?}",
        lexed.diagnostics
    );
    let mut parser = Parser::new(FileId::new(0), lexed.tokens);
    let parsed = parser.parse(None);
    assert!(
        parsed.diagnostics.is_empty(),
        "parse diagnostics: {:?}",
        parsed.diagnostics
    );
    parsed.statements
}

pub(crate) fn table_of(source: &str) -> crate::build::block_scope_rename::table::ScopeTable {
    let mut statements = parse(source);
    let mut collector = crate::build::block_scope_rename::table::Collector::default();
    crate::build::block_scope_rename::walk::walk_program(&mut statements, &mut collector);
    collector.finish()
}
