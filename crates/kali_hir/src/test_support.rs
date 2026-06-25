//! Shared test helpers for the HIR test modules.
use kali_ast::Statement;
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

pub(crate) fn parse(source: &str) -> Vec<Statement> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    parser.parse(None).statements
}
