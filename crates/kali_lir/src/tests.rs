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

#[test]
fn test_lir_lowering_preserves_root() {
    let mir = parse_and_lower("function add(a, b) { return a + b; }");
    let lir = LirLowerer::new().lower_program(&mir);

    assert_eq!(lir.nodes[lir.root.0 as usize].kind, LirNodeKind::Program);
    assert_eq!(lir.nodes[lir.root.0 as usize].children.len(), 1);
}
