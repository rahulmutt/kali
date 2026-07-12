use super::*;

#[test]
fn performance_now_lowers_to_kalirt_import_returning_float() {
    let program = parse_and_lower_lir("performance.now();");
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
        .expect("validate");
    assert!(
        printed.contains("import \"kali:rt\" \"performance_now\""),
        "{printed}"
    );
}
