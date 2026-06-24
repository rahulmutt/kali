//! kali_codegen-specific test builders and macros (compiled under cfg(test)).
use crate::*;

// --- helpers migrated from tests.rs (each made pub(crate)) ---

pub(crate) fn node(kind: LirNodeKind, text: Option<&str>, children: Vec<LirNodeId>) -> LirNode {
    LirNode {
        kind,
        text: text.map(ToString::to_string),
        children,
        function_flavor: None,
    }
}

pub(crate) fn sample_program() -> LirProgram {
    let mut nodes = Vec::new();

    let root = LirNodeId(0);
    let add = LirNodeId(1);
    let add_param_a = LirNodeId(2);
    let add_param_b = LirNodeId(3);
    let add_block = LirNodeId(4);
    let add_return = LirNodeId(5);
    let add_expr = LirNodeId(6);
    let add_left = LirNodeId(7);
    let add_right = LirNodeId(8);
    let call_expr = LirNodeId(9);
    let call_callee = LirNodeId(10);
    let lit_one = LirNodeId(11);
    let lit_two = LirNodeId(12);

    nodes.push(node(LirNodeKind::Program, None, vec![add, call_expr]));
    nodes.push(node(
        LirNodeKind::Instruction,
        Some("add"),
        vec![add_param_a, add_param_b, add_block],
    ));
    nodes.push(node(LirNodeKind::Value, Some("a"), vec![]));
    nodes.push(node(LirNodeKind::Value, Some("b"), vec![]));
    nodes.push(node(LirNodeKind::Block, None, vec![add_return]));
    nodes.push(node(
        LirNodeKind::Instruction,
        Some("return"),
        vec![add_expr],
    ));
    nodes.push(node(
        LirNodeKind::Value,
        Some("+"),
        vec![add_left, add_right],
    ));
    nodes.push(node(LirNodeKind::Value, Some("a"), vec![]));
    nodes.push(node(LirNodeKind::Value, Some("b"), vec![]));
    nodes.push(node(
        LirNodeKind::Call,
        None,
        vec![call_callee, lit_one, lit_two],
    ));
    nodes.push(node(LirNodeKind::Value, Some("add"), vec![]));
    nodes.push(node(LirNodeKind::Literal, Some("1"), vec![]));
    nodes.push(node(LirNodeKind::Literal, Some("2"), vec![]));

    LirProgram { root, nodes }
}

pub(crate) fn parse_and_lower_lir(source: &str) -> LirProgram {
    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut hir_lowerer = kali_hir::HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&statements);
    let mir = kali_mir::MirLowerer::new().lower_hir_result(&hir);
    kali_lir::LirLowerer::new().lower_program(&mir)
}

pub(crate) fn assert_nullish_assignment_lowers(source: &str) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

pub(crate) fn assert_logical_assignment_lowers(source: &str) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("if"), "{printed}");
    assert!(printed.contains("else"), "{printed}");
}

pub(crate) fn assert_nullish_coalescing_lowers(source: &str) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

pub(crate) fn compile_and_measure(program: &LirProgram) -> (Vec<u8>, usize) {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, program);
    assert!(
        result.diagnostics.is_empty(),
        "codegen diagnostics: {:?}",
        result.diagnostics
    );
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let instruction_count = wasm_instruction_count(&result.wasm_bytes);
    (result.wasm_bytes, instruction_count)
}

pub(crate) fn wasm_instruction_count(bytes: &[u8]) -> usize {
    use wasmparser::{Parser as WasmParser, Payload};

    let mut count = 0;
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let Ok(Payload::CodeSectionEntry(body)) = payload {
            let mut operators = body.get_operators_reader().expect("operators");
            while !operators.eof() {
                operators.read().expect("operator");
                count += 1;
            }
        }
    }
    count
}

/// Build a `LirProgram` from a root id and a flat list of `(kind, text, children)`
/// tuples — collapses the repeated `LirNodeId(n)` + `nodes.push(node(...))` pattern.
///
/// `lir!(root: 0, nodes: [ (LirNodeKind::Program, None, vec![1]), (LirNodeKind::Value, Some("a"), vec![]) ])`
macro_rules! lir {
    (root: $root:expr, nodes: [ $( ($kind:expr, $text:expr, $children:expr) ),* $(,)? ]) => {{
        let mut nodes = Vec::new();
        $( nodes.push($crate::test_support::node($kind, $text, $children)); )*
        $crate::LirProgram { root: $crate::LirNodeId($root), nodes }
    }};
}
pub(crate) use lir;
