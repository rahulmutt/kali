use super::*;

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
fn streq_synthetic_is_emitted_without_i64_eqz() {
    // `__streq` (throw-fallout Stage 1) is an unconditional synthetic like
    // `__join`: present in every module. Its byte loop is the module's only
    // `i64.load8_u` consumer, and — like `__join` (see the comment in
    // `emit_join_body`) — it must never emit `i64.eqz`, which
    // `boolean_branches_use_the_layout_fast_path` bans module-wide.
    let program = parse_and_lower_lir("console.log(1);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty());
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.load8_u"));
    assert!(!printed.contains("i64.eqz"));
}

#[test]
fn mir_backed_pipeline_reduces_legacy_overhead_on_escaping_locals() {
    let current_lir = sample_program();
    let mir = kali_mir::MirProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: Vec::new(),
        arena_facts: Vec::new(),
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
