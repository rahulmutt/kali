use super::*;

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
