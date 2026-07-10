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
    // node indices: 0=Program, 1=Instruction(add), 2=Value(a), 3=Value(b),
    //               4=Block, 5=Instruction(return), 6=Value(+), 7=Value(a),
    //               8=Value(b), 9=Call, 10=Value(add), 11=Literal(1), 12=Literal(2)
    lir!(root: 0, nodes: [
        (LirNodeKind::Program,     None,          vec![LirNodeId(1), LirNodeId(9)]),
        (LirNodeKind::Instruction, Some("add"),   vec![LirNodeId(2), LirNodeId(3), LirNodeId(4)]),
        (LirNodeKind::Value,       Some("a"),     vec![]),
        (LirNodeKind::Value,       Some("b"),     vec![]),
        (LirNodeKind::Block,       None,          vec![LirNodeId(5)]),
        (LirNodeKind::Instruction, Some("return"),vec![LirNodeId(6)]),
        (LirNodeKind::Value,       Some("+"),     vec![LirNodeId(7), LirNodeId(8)]),
        (LirNodeKind::Value,       Some("a"),     vec![]),
        (LirNodeKind::Value,       Some("b"),     vec![]),
        (LirNodeKind::Call,        None,          vec![LirNodeId(10), LirNodeId(11), LirNodeId(12)]),
        (LirNodeKind::Value,       Some("add"),   vec![]),
        (LirNodeKind::Literal,     Some("1"),     vec![]),
        (LirNodeKind::Literal,     Some("2"),     vec![]),
    ])
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

/// `setup` mirrors the real compiler driver's context priming (e.g. the
/// `kali_types`-produced `ReprTable` shape entries a `for..in` fixture needs —
/// `parse_and_lower_lir` runs no type inference; see
/// `computed_forin_key_access_uses_headerless_offset_zero` for the pattern).
pub(crate) fn assert_nullish_assignment_lowers(source: &str, setup: impl FnOnce(&mut CodegenCtx)) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    setup(&mut ctx);
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
        #[allow(clippy::vec_init_then_push)]
        let nodes = {
            let mut nodes = Vec::new();
            $( nodes.push($crate::test_support::node($kind, $text, $children)); )*
            nodes
        };
        $crate::LirProgram { root: $crate::LirNodeId($root), nodes }
    }};
}
pub(crate) use lir;
