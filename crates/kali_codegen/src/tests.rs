use super::*;
use wasmparser::Validator;

fn node(kind: LirNodeKind, text: Option<&str>, children: Vec<LirNodeId>) -> LirNode {
    LirNode {
        kind,
        text: text.map(ToString::to_string),
        children,
    }
}

fn sample_program() -> LirProgram {
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

#[test]
fn generates_valid_wasm_for_simple_programs() {
    let program = sample_program();
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.add"));
    assert!(printed.contains("call"));
}

#[test]
fn function_plans_are_detected_from_instruction_shape() {
    let program = sample_program();
    let plans = collect_functions(&program);
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].name, "add");
    assert_eq!(plans[0].params, vec!["a", "b"]);
}

fn parse_and_lower_lir(source: &str) -> LirProgram {
    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut hir_lowerer = kali_hir::HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&statements);
    let mir = kali_mir::MirLowerer::new().lower_hir_result(&hir);
    kali_lir::LirLowerer::new().lower_program(&mir)
}

#[test]
fn boolean_branches_use_the_layout_fast_path() {
    let program = parse_and_lower_lir("if (1 == 1) { 7; } else { 9; }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty());
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i32.wrap_i64"));
    assert!(!printed.contains("i64.eqz"));
}

#[test]
fn console_member_calls_lower_to_console_host_imports() {
    let program = parse_and_lower_lir(
        "console.log(1); console.error(2); console.warn(3); console.info(4); console.debug(5);",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"console_log\""));
    assert!(printed.contains("import \"kali:rt\" \"console_error\""));
    assert!(printed.contains("import \"kali:rt\" \"console_warn\""));
    assert!(printed.contains("import \"kali:rt\" \"console_info\""));
    assert!(printed.contains("import \"kali:rt\" \"console_debug\""));
}

#[test]
fn math_max_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.max(1, 2, 3));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"math_max\""));
}

#[test]
fn math_min_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.min(3, 2, 1));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"math_min\""));
}

#[test]
fn math_abs_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.abs(3 - 6));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"math_abs\""));
}

#[test]
fn math_sign_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.sign(3 - 6));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"math_sign\""));
}

#[test]
fn process_argv_length_lowers_to_runtime_args_length_import() {
    let program = parse_and_lower_lir("console.log(process.argv.length);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"args_len\""));
}

#[test]
fn process_argv_slice_length_lowers_to_runtime_args_length_minus_start() {
    let program = parse_and_lower_lir("console.log(process.argv.slice(2).length);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"args_len\""));
    assert!(printed.contains("i64.const 2"));
    assert!(printed.contains("i64.sub"));
}

fn legacy_phase1_baseline(program: &LirProgram, mir: &kali_mir::MirProgram) -> LirProgram {
    let mut nodes = program.nodes.clone();
    let mut extra_nodes = Vec::new();
    let mut insertions = Vec::new();

    let mut ownership_by_name = std::collections::BTreeMap::new();
    for function in &mir.functions {
        for binding in &function.bindings {
            if binding.kind == kali_mir::MirBindingKind::Local {
                ownership_by_name
                    .entry(binding.name.clone())
                    .or_insert(binding.ownership);
            }
        }
    }

    insertions.push((
        program.root.0 as usize,
        vec!["phase1.alloc", "phase1.decref"],
    ));

    for (index, node) in program.nodes.iter().enumerate() {
        if node.kind != LirNodeKind::Instruction {
            continue;
        }

        let Some(name) = node.text.as_deref() else {
            continue;
        };

        if let Some(last_child) = node.children.last().copied() {
            if program
                .nodes
                .get(last_child.0 as usize)
                .is_some_and(|child| child.kind == LirNodeKind::Block)
            {
                insertions.push((last_child.0 as usize, vec!["phase1.alloc", "phase1.decref"]));
                continue;
            }
        }

        let Some(ownership) = ownership_by_name.get(name).copied() else {
            continue;
        };

        let markers: Vec<&'static str> = match ownership {
            kali_mir::OwnershipClass::OwnedHeap => vec!["phase1.alloc", "phase1.decref"],
            kali_mir::OwnershipClass::SharedHeap => {
                vec!["phase1.alloc", "phase1.incref", "phase1.decref"]
            }
            kali_mir::OwnershipClass::Stack | kali_mir::OwnershipClass::Borrowed => Vec::new(),
        };

        if markers.is_empty() {
            continue;
        }

        insertions.push((index, markers));
    }

    for (index, markers) in insertions {
        let mut synthetic_children = Vec::with_capacity(markers.len());
        for marker in markers {
            let id = LirNodeId((nodes.len() + extra_nodes.len()) as u32);
            extra_nodes.push(LirNode::with_text(LirNodeKind::Literal, marker));
            synthetic_children.push(id);
        }
        nodes[index].children.extend(synthetic_children);
    }

    nodes.extend(extra_nodes);
    LirProgram {
        root: program.root,
        nodes,
    }
}

fn compile_and_measure(program: &LirProgram) -> (Vec<u8>, usize) {
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
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let instruction_count = wasm_instruction_count(&result.wasm_bytes);
    (result.wasm_bytes, instruction_count)
}

fn wasm_instruction_count(bytes: &[u8]) -> usize {
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

#[test]
fn unresolved_identifier_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_value.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_value")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-identifier diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unresolved_call_target_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[10].text = Some("missing_fn".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_fn.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_fn")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-call diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn duplicate_unresolved_identifier_lowering_reports_one_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());
    program.nodes[8].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let matching_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic.message.contains("missing_value")
        })
        .count();
    assert_eq!(
        matching_diagnostics, 1,
        "expected the repeated unresolved identifier to be reported once: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn mir_backed_pipeline_reduces_legacy_overhead_on_escaping_locals() {
    let current_lir = sample_program();
    let mir = kali_mir::MirProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: Vec::new(),
    };
    let baseline_lir = legacy_phase1_baseline(&current_lir, &mir);

    let current_trace = current_lir
        .nodes
        .iter()
        .filter_map(|node| node.text.as_deref())
        .collect::<Vec<_>>();
    let baseline_trace = baseline_lir
        .nodes
        .iter()
        .filter_map(|node| node.text.as_deref())
        .collect::<Vec<_>>();

    assert!(!current_trace.contains(&"phase1.alloc"));
    assert!(!current_trace.contains(&"phase1.incref"));
    assert!(!current_trace.contains(&"phase1.decref"));
    assert!(baseline_trace.contains(&"phase1.alloc"));
    assert!(baseline_trace.contains(&"phase1.decref"));

    let (current_bytes, current_instructions) = compile_and_measure(&current_lir);
    let (baseline_bytes, baseline_instructions) = compile_and_measure(&baseline_lir);

    assert!(
        current_bytes.len() < baseline_bytes.len(),
        "MIR-backed pipeline should produce smaller WASM than the legacy baseline"
    );
    assert!(
        current_instructions < baseline_instructions,
        "MIR-backed pipeline should emit fewer instructions than the legacy baseline"
    );
}
