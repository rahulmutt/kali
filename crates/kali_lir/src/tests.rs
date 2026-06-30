use super::*;
use kali_common::FileId;
use kali_hir::HirLowerer;
use kali_lexer::Lexer;
use kali_mir::MirLowerer;
use kali_parser::Parser;

fn parse_and_lower(source: &str) -> MirProgram {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&statements);
    MirLowerer::new().lower_hir_result(&hir)
}

#[path = "tests/flavor_metadata.rs"]
mod flavor_metadata;

#[path = "tests/validation.rs"]
mod validation;

#[path = "tests/structure.rs"]
mod structure;
