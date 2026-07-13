use super::*;

#[test]
fn crypto_get_random_values_lowers_to_kalirt_import() {
    let program = parse_and_lower_lir("const b = new Uint8Array(8); crypto.getRandomValues(b);");
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
        printed.contains("import \"kali:rt\" \"crypto_get_random_values\""),
        "{printed}"
    );
}

#[test]
fn crypto_random_uuid_lowers_to_kalirt_import() {
    let program = parse_and_lower_lir("const u = crypto.randomUUID(); console.log(u.length);");
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
        printed.contains("import \"kali:rt\" \"crypto_random_uuid\""),
        "{printed}"
    );
}
