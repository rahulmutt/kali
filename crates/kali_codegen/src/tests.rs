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
fn object_is_lowers_for_static_primitive_literals() {
    let program = parse_and_lower_lir(
        "const flag = true; const text = \"hello\"; const big = 1n; console.log(Object.is(flag, true)); console.log(Object.is(text, \"hello\")); console.log(Object.is(big, 1n)); console.log(Object.is(-1n, -1n)); console.log(Object.is(-0, +0)); console.log(Object.is(-0, -0)); console.log(Object.is(+0, +0)); console.log(Object.is(Infinity, Infinity)); console.log(Object.is(NaN, NaN)); console.log(Object.is(null, null)); console.log(Object.is(void 0, void 0)); console.log(Object.is(1, 0));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn object_is_lowers_for_same_static_reference() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; console.log(Object.is(alias, object)); console.log(Object.is(object, object));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_is_lowers_for_same_static_reference_through_object_freeze() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const frozen = Object.freeze(object); console.log(Object.is(frozen, object)); console.log(Object.is(Object.freeze(object), object)); console.log(globalThis[\"Object\"][\"is\"](frozen, object)); console.log(globalThis.Object[\"is\"](frozen, object)); console.log(globalThis[\"Object\"].is(frozen, object));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_is_lowers_for_bracketed_global_this_object_spellings() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; const frozen = Object.freeze(object); console.log(globalThis[\"Object\"][\"is\"](alias, object)); console.log(globalThis.Object[\"is\"](frozen, object)); console.log(globalThis[\"Object\"].is(alias, object)); console.log(globalThis.Object.is(frozen, object));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_is_lowers_through_parenthesized_same_reference_wrappers() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; console.log(Object.is((alias), (((object)))));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn for_of_object_enumeration_lowers_for_single_quoted_bracketed_aliases_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis['Object']['values'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for (const key of [...globalThis['Object']['keys'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(key); } for (const entry of [...globalThis['Object']['entries'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn object_has_own_lowers_for_single_quoted_bracketed_alias_over_frozen_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(globalThis['Object']['hasOwn'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(Object.hasOwn(Object.fromEntries([[\"b\", 1], [\"a\", 2]]), \"a\"));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_frozen_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(Object.hasOwn(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_bracketed_global_this_object_spellings() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Object\"][\"hasOwn\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_mixed_bracketed_global_this_object_spellings() {
    for program in [
        "console.log(globalThis.Object[\"hasOwn\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
        "console.log(globalThis[\"Object\"].hasOwn(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
    ] {
        let program = parse_and_lower_lir(program);
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

        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        assert!(printed.contains("i64.const 1"), "{printed}");
    }
}

#[test]
fn for_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_values_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of Object.values(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(value); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_entries_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_bracket_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis.Object[\"values\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(value); } for (const key of [...globalThis[\"Object\"].keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_frozen_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))) { console.log(key); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of [...Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_await_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn object_is_lowers_for_unary_plus_wrapped_numeric_literals() {
    let program = parse_and_lower_lir("console.log(Object.is(+1, 1));");
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
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
fn console_assert_member_lowering_uses_console_error_for_falsey_conditions() {
    let program = parse_and_lower_lir("console.assert(1, 'assert failed');");
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
    assert!(printed.contains("import \"kali:rt\" \"console_error\""));
    assert!(printed.contains("i64.eqz"));
    assert!(printed.contains("i32.eqz"));
}

#[test]
fn mutable_local_reassignment_keeps_runtime_reads() {
    let program = parse_and_lower_lir("let value = 1; value = 3; console.log(value);");
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn mutable_local_compound_assignment_accepts_wrapper_targets() {
    let program = parse_and_lower_lir("let value = 1; ((value)) += 2; console.log(value);");
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn compound_assignment_on_non_local_targets_reports_feature_unavailable() {
    let program = parse_and_lower_lir("let target = { value: 1 }; target.value += 2;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("compound assignment lowering is unavailable")
        }),
        "expected a compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn compound_assignment_on_immutable_bindings_reports_feature_unavailable() {
    let program = parse_and_lower_lir("const value = 1; value += 2;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("compound assignment lowering is unavailable for binding 'value'")
        }),
        "expected an immutable-binding compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn update_expression_lowering_keeps_prefix_and_postfix_local_reads() {
    let program = parse_and_lower_lir(
        "let value = 1; console.log(++value); console.log(value--); console.log(value);",
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
    assert!(printed.contains("i64.add"), "{printed}");
    assert!(printed.contains("i64.sub"), "{printed}");
}

#[test]
fn math_max_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("function max(value) { return Math.max(value, 2, 3); }");
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
fn math_max_member_calls_lower_to_math_host_imports_through_global_this_math() {
    let program =
        parse_and_lower_lir("function max(value) { return globalThis.Math.max(value, 2, 3); }");
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
fn math_floor_trunc_and_ceil_member_constant_folds_through_global_this_math() {
    let program = parse_and_lower_lir(
        "console.log(globalThis.Math.floor(1.6)); console.log(globalThis.Math.trunc(1.6)); console.log(globalThis.Math.ceil(1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_floor_trunc_and_ceil_member_constant_folds_through_bracketed_global_this_math() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Math\"][\"floor\"](1.6)); console.log(globalThis[\"Math\"][\"trunc\"](1.6)); console.log(globalThis[\"Math\"][\"ceil\"](1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_operand() {
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_operand_through_global_this_math() {
    let program = parse_and_lower_lir("console.log(globalThis.Math.max(1, 2, 3));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 3; const alias = value; console.log(globalThis.Math.max(alias, 2, 1));",
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_min_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("function min(value) { return Math.min(value, 3, 2); }");
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
fn math_min_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 3; const alias = value; console.log(Math.min(alias, 2, 1));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 8"), "{printed}");
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
fn math_abs_member_constant_folds_static_numeric_literal_alias_chains() {
    let program =
        parse_and_lower_lir("const value = -3; const alias = value; console.log(Math.abs(alias));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 9"), "{printed}");
}

#[test]
fn math_abs_member_constant_folds_static_numeric_literal_operand() {
    let program = parse_and_lower_lir("console.log(Math.abs(-3));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 9"), "{printed}");
}

#[test]
fn math_sign_member_constant_folds_static_numeric_literal_operand() {
    let program = parse_and_lower_lir("console.log(Math.sign(1.6));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 10"), "{printed}");
}

#[test]
fn math_sign_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.sign(alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 10"), "{printed}");
}

#[test]
fn math_abs_and_sign_member_calls_lower_through_bracketed_global_this_math_root() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Math\"][\"abs\"](3 - 6)); console.log(globalThis[\"Math\"][\"sign\"](3 - 6));",
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
    assert!(printed.contains(r#"import "kali:rt" "math_abs""#));
    assert!(printed.contains(r#"import "kali:rt" "math_sign""#));
}

#[test]
fn math_imul_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.imul(2147483647, 2));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_imul\""));
}

#[test]
fn math_imul_member_constant_folds_static_integer_literal_operands() {
    let program = parse_and_lower_lir("console.log(Math.imul(2147483647, 2));");
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
    assert!(printed.contains("i64.const -2"), "{printed}");
    assert!(!printed.contains("call 11"), "{printed}");
}

#[test]
fn math_imul_member_constant_folds_omitted_operands_to_zero() {
    let program = parse_and_lower_lir("console.log(Math.imul()); console.log(Math.imul(7));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 7"), "{printed}");
    assert!(!printed.contains("call 11"), "{printed}");
}

#[test]
fn math_clz32_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.clz32(1));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_clz32\""));
}

#[test]
fn math_clz32_member_constant_folds_zero_arguments() {
    let program = parse_and_lower_lir("console.log(Math.clz32());");
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
    assert!(printed.contains("i64.const 32"), "{printed}");
    assert!(!printed.contains("call 15"), "{printed}");
}

#[test]
fn math_clz32_member_constant_folds_static_integer_literal_alias_chain() {
    let program = parse_and_lower_lir(
        "const value = 1; const alias = value; console.log(Math.clz32(alias));",
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
    assert!(printed.contains("i64.const 31"), "{printed}");
    assert!(!printed.contains("call 14"), "{printed}");
}

#[test]
fn math_clz32_member_constant_folds_static_non_integer_literal() {
    let program = parse_and_lower_lir("console.log(Math.clz32(1.6));");
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
    assert!(printed.contains("i64.const 31"), "{printed}");
    assert!(!printed.contains("call 14"), "{printed}");
}

#[test]
fn math_pow_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, 3));");
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
    assert!(printed.contains(r#"import "kali:rt" "math_pow""#));
}

#[test]
fn math_pow_member_constant_folds_zero_exponent_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, 0));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_zero_exponent_identity_for_non_integer_base_literals() {
    let program = parse_and_lower_lir("console.log(Math.pow(1.6, 0));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_zero_base_positive_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(0, alias));",
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
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_exponent_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(7, 1));");
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
    assert!(printed.contains("i64.const 7"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(1, 7));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_identity_through_bracketed_global_this_math() {
    let program = parse_and_lower_lir("console.log(globalThis[\"Math\"][\"pow\"](1, 7));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_uses_integer_exponent_alias_chain() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias));",
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_pow""#),
        "{printed}"
    );
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn math_pow_member_uses_negative_integer_base_with_integer_exponent_alias_chain() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(-2, alias));",
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_pow""#),
        "{printed}"
    );
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn math_round_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.round(1));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_round\""));
}

#[test]
fn math_round_member_calls_constant_fold_floating_literal() {
    let program = parse_and_lower_lir("console.log(Math.round(1.6));");
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_round_member_calls_global_this_root_lower_to_math_host_imports() {
    let program =
        parse_and_lower_lir("function f(value) { console.log(globalThis.Math.round(value)); }");
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_round""#),
        "{printed}"
    );
}

#[test]
fn math_trunc_member_lowers_without_runtime_host_import() {
    let program = parse_and_lower_lir("console.log(Math.trunc(1));");
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
    assert!(!printed.contains("import \"kali:rt\" \"math_trunc\""));
}

#[test]
fn math_ceil_member_lowers_without_runtime_host_import() {
    let program = parse_and_lower_lir("console.log(Math.ceil(1));");
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
    assert!(!printed.contains("import \"kali:rt\" \"math_ceil\""));
}

#[test]
fn supported_math_ceil_member_constant_folds_non_integer_numeric_literals() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias));",
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn supported_math_trunc_member_constant_folds_non_integer_numeric_literals() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.trunc(alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_floor_member_lowering_is_available() {
    let program = parse_and_lower_lir("console.log(Math.floor(1));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_floor_member_constant_folding_is_available_for_non_integer_literal() {
    let program = parse_and_lower_lir("console.log(Math.floor(1.6));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_hypot_member_lowering_is_available_for_perfect_square_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.hypot(3, 4));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 5"), "{printed}");
}

#[test]
fn supported_math_hypot_member_lowering_is_available_for_zero_arguments() {
    let program = parse_and_lower_lir("console.log(Math.hypot());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_sqrt_member_lowering_is_available_for_perfect_square_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.sqrt(4));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn supported_math_cbrt_member_lowering_is_available_for_perfect_cube_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.cbrt(27));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_math_log2_member_lowering_is_available_for_positive_power_of_two_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.log2(8));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_math_log10_member_lowering_is_available_for_positive_power_of_ten_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.log10(1000));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn unsupported_math_sqrt_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.sqrt(1.6));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.sqrt is unavailable unless the argument is a statically-known perfect-square integer literal")
        }),
        "expected an unavailable Math-member diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_generator_function_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("function* main() { yield* []; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && (diagnostic.message.contains("generator function lowering")
                    || diagnostic.message.contains("yield expressions"))
        }),
        "expected an unavailable generator diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_async_generator_function_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("async function* main() { yield 1; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && (diagnostic.message.contains("generator function lowering")
                    || diagnostic.message.contains("yield expressions"))
        }),
        "expected an unavailable async-generator diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_math_exp_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.exp(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_exp2_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.exp2(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_global_this_bracketed_math_exp2_member_lowering_is_available_for_exact_zero_literals()
{
    let program =
        parse_and_lower_lir("const zero = 0; console.log(globalThis[\"Math\"][\"exp2\"](zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_log_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.log(one));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_exp_and_log_member_lowering_is_available_for_const_numeric_alias_chain_literals()
{
    let program = parse_and_lower_lir(
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.exp(zeroAlias)); console.log(Math.log(oneAlias));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_expm1_and_log1p_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir(
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_expm1_and_log1p_member_lowering_is_available_for_const_numeric_alias_chain_literals(
) {
    let program = parse_and_lower_lir(
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_tan_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.tan(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_sin_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.sin(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_cos_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.cos(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn unsupported_math_tan_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.tan(1));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.tan is unavailable unless the argument is a statically-known zero numeric literal")
        }),
        "expected an unavailable Math.tan diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_asin_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.asin(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_acos_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.acos(one));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.atan(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_asinh_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.asinh(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_acosh_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.acosh(one));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atanh_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.atanh(zero));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_hyperbolic_zero_identity_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir(
        "const zero = 0; console.log(Math.sinh(zero)); console.log(Math.cosh(zero)); console.log(Math.tanh(zero));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn unsupported_math_hyperbolic_zero_identity_member_reports_feature_unavailable() {
    for (source, expected_method) in [
        ("console.log(Math.sinh(1));", "Math.sinh"),
        ("console.log(Math.cosh(1));", "Math.cosh"),
        ("console.log(Math.tanh(1));", "Math.tanh"),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains("zero numeric literal")
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_inverse_hyperbolic_member_reports_feature_unavailable() {
    for (source, expected_method, expected_literal) in [
        (
            "console.log(Math.asinh(1));",
            "Math.asinh",
            "zero numeric literal",
        ),
        (
            "console.log(Math.acosh(0));",
            "Math.acosh",
            "one numeric literal",
        ),
        (
            "console.log(Math.atanh(1));",
            "Math.atanh",
            "zero numeric literal",
        ),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains(expected_literal)
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_inverse_trig_member_reports_feature_unavailable() {
    for (source, expected_method, expected_literal) in [
        (
            "console.log(Math.asin(1));",
            "Math.asin",
            "zero numeric literal",
        ),
        (
            "console.log(Math.acos(0));",
            "Math.acos",
            "one numeric literal",
        ),
        (
            "console.log(Math.atan(1));",
            "Math.atan",
            "zero numeric literal",
        ),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains(expected_literal)
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_atan2_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.atan2(1, 1));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path")
        }),
        "expected an unavailable Math.atan2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_atan2_member_is_available_for_zero_numerator_and_non_negative_denominator_literals(
) {
    let program =
        parse_and_lower_lir("const zero = 0; const one = 1; console.log(Math.atan2(zero, one));");
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
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan2_member_is_available_for_const_numeric_alias_chain() {
    let program = parse_and_lower_lir(
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.atan2(zeroAlias, oneAlias));",
    );
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
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan2_member_is_available_for_bracketed_global_this_aliases() {
    let program = parse_and_lower_lir(
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one)); console.log(globalThis.Math[\"atan2\"](zero, one)); console.log(globalThis[\"Math\"].atan2(zero, one));",
    );
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
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn unsupported_math_exp_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.exp(2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.exp is unavailable unless the argument is a statically-known zero numeric literal")
        }),
        "expected an unavailable Math.exp diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_exp2_member_lowering_is_available_for_non_negative_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.exp2(1)); console.log(Math.exp2(3));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 8"), "{printed}");
}

#[test]
fn unsupported_math_exp2_non_integer_literals_report_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.exp2(1.5));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.exp2 is unavailable unless the argument is a statically-known non-negative integer literal within the current integer-fold range")
        }),
        "expected an unavailable Math.exp2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_log_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log(2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.log is unavailable unless the argument is a statically-known one numeric literal")
        }),
        "expected an unavailable Math.log diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_expm1_and_log1p_member_reports_feature_unavailable() {
    for (source, expected_method) in [
        ("console.log(Math.expm1(1));", "Math.expm1"),
        ("console.log(Math.log1p(1));", "Math.log1p"),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains("zero numeric literal")
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_log2_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log2(12));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("positive power-of-two")
        }),
        "expected an unavailable Math.log2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_log10_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log10(12));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("positive power-of-ten")
        }),
        "expected an unavailable Math.log10 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_max_without_arguments_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.max());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("requires at least one argument")
        }),
        "expected an unavailable Math.max diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_with_single_argument_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.pow(2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(5506)
                && diagnostic
                    .message
                    .contains("requires at least two operands")
        }),
        "expected an unavailable Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_member_rejects_non_integer_const_alias_exponents() {
    let program = parse_and_lower_lir(
        "const exponent = 1.6; const alias = exponent; console.log(Math.pow(2, alias));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("non-integer numeric literals")
        }),
        "expected a non-integer Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_member_rejects_negative_exponents() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, -1));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Math.pow is unavailable for negative numeric literals")
        }),
        "expected a negative-exponent Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn math_pow_member_constant_folds_negative_one_base_positive_integer_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(-1, alias));",
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
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_negative_one_base_negative_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = -3; const alias = exponent; console.log(Math.pow(-1, alias));",
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
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_negative_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn supported_exponentiation_operator_lowering_is_available_for_integer_literals() {
    let program = parse_and_lower_lir("console.log(2 ** 3);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("call 16"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn unsupported_exponentiation_operator_rejects_negative_exponents() {
    let program = parse_and_lower_lir("console.log(2 ** -1);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("negative numeric literals")
        }),
        "expected a negative-exponentiation diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_const_alias_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; const values = ([1, (value)]); for (const item of (values)) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for ((item) of [1, 2]) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] as const)) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] as const)) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for await ((item) of [1, 2]) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for await (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys('ab')) { console.log(key); } for (const value of Object.values('ab')) { console.log(value); } for (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for await (const key of Object.keys('ab')) { console.log(key); } for await (const value of Object.values('ab')) { console.log(value); } for await (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...values]) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays()
{
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }",
    );
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_nullish_assignment_lowers(source: &str) {
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_logical_assignment_lowers(source: &str) {
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("if"), "{printed}");
    assert!(printed.contains("else"), "{printed}");
}

fn assert_nullish_coalescing_lowers(source: &str) {
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn nullish_coalescing_lowers_for_supported_input_shapes() {
    assert_nullish_coalescing_lowers("console.log(null ?? 1);");
    assert_nullish_coalescing_lowers("console.log(undefined ?? 1);");
}

#[test]
fn nullish_assignment_lowers_for_wrapped_mutable_local_binding_targets() {
    assert_nullish_assignment_lowers("let value = null; ((value)) ??= 1; console.log(value);");
}

#[test]
fn logical_assignment_lowers_for_wrapped_mutable_local_binding_targets() {
    assert_logical_assignment_lowers("let left = 0; ((left)) ||= 1; console.log(left); let right = 1; ((right)) &&= 2; console.log(right);");
}

#[test]
fn process_argv_slice_length_with_non_default_start_lowers_to_runtime_args_length_minus_start() {
    let program = parse_and_lower_lir("console.log(process.argv.slice(1).length);");
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
    assert!(printed.contains("i64.const 1"));
    assert!(printed.contains("i64.sub"));
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
fn deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(Deno.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn global_this_deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis.Deno.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn bracketed_global_this_deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"Deno\"][\"pid\"]);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(process.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn global_this_process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis.process.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn bracketed_global_this_process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"process\"][\"pid\"]);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn deno_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(Deno.cwd());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn deno_chdir_member_calls_lower_to_runtime_cwd_set_import() {
    let program = parse_and_lower_lir("Deno.chdir(\"nested\");");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd_set\""));
}

#[test]
fn process_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(process.cwd());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn bracketed_global_this_process_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"process\"][\"cwd\"]());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn process_exit_member_calls_lower_to_runtime_process_exit_import() {
    let program = parse_and_lower_lir("process.exit(7);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"process_exit\""));
}

#[test]
fn deno_env_get_member_calls_lower_to_runtime_env_get_import() {
    let program = parse_and_lower_lir("console.log(Deno.env.get(\"HOME\"));");
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
    assert!(printed.contains("import \"kali:rt\" \"env_get\""));
    assert!(
        printed.contains("i32.const 4096"),
        "printed wasm: {printed}"
    );
}

#[test]
fn deno_env_has_member_calls_lower_to_runtime_env_has_import() {
    let program = parse_and_lower_lir("console.log(Deno.env.has(\"HOME\"));");
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
    assert!(printed.contains("import \"kali:rt\" \"env_has\""));
}

#[test]
fn deno_env_set_member_calls_lower_to_runtime_env_set_import() {
    let program =
        parse_and_lower_lir("Deno.env.set(\"KALI_ENV_SET_SMOKE\", \"hello-environment\");");
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
    assert!(printed.contains("import \"kali:rt\" \"env_set\""));
}

#[test]
fn bracketed_deno_env_set_member_calls_lower_to_runtime_env_set_import() {
    let program = parse_and_lower_lir(
        "Deno[\"env\"][\"set\"](\"KALI_ENV_SET_SMOKE\", \"hello-environment\");",
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
    assert!(printed.contains("import \"kali:rt\" \"env_set\""));
}

#[test]
fn deno_env_delete_member_calls_lower_to_runtime_env_delete_import() {
    let program = parse_and_lower_lir("Deno.env.delete(\"KALI_ENV_SET_SMOKE\");");
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
    assert!(printed.contains("import \"kali:rt\" \"env_delete\""));
}

#[test]
fn mixed_global_this_deno_env_delete_member_calls_lower_to_runtime_env_delete_import() {
    let program =
        parse_and_lower_lir("globalThis.Deno[\"env\"][\"delete\"](\"KALI_ENV_SET_SMOKE\");");
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
    assert!(printed.contains("import \"kali:rt\" \"env_delete\""));
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
