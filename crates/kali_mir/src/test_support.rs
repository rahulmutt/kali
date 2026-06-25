//! Shared helpers for kali_mir unit tests.

use kali_common::FileId;
use kali_hir::{HirLowerer, LoweringResult as HirLoweringResult};
use kali_lexer::Lexer;
use kali_parser::Parser;

use crate::{MirLowerer, MirProgram};

pub(crate) fn parse_and_lower_hir(source: &str) -> HirLoweringResult {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut lowerer = HirLowerer::new();
    lowerer.lower_statements(&statements)
}

pub(crate) fn analyze(source: &str) -> MirProgram {
    let hir = parse_and_lower_hir(source);
    MirLowerer::new().lower_hir_result(&hir)
}
