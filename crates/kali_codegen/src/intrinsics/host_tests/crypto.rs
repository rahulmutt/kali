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

#[test]
fn crypto_subtle_digest_lowers_to_kalirt_import() {
    // throw-fallout Stage 3 Task 7: `crypto.subtle.digest(algo, bytes)` lowers to
    // a conditional `kali:rt` `crypto_subtle_digest` host import + call. The input
    // comes from `new TextEncoder().encode(<string>)` (a contiguous byte buffer).
    // (`.byteLength` on the result reads the digest length via the String-repr
    // arm, which needs kali_types inference — exercised end-to-end by the
    // `runtime_smoke` `subtle_digest` node-parity target, not this repr-less
    // codegen-unit lowering.)
    let program = parse_and_lower_lir(
        "const b = new TextEncoder().encode('browser crypto'); crypto.subtle.digest('SHA-256', b);",
    );
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
        printed.contains("import \"kali:rt\" \"crypto_subtle_digest\""),
        "{printed}"
    );
}

#[test]
fn text_encoder_encode_is_a_pure_guest_side_reinterpret() {
    // throw-fallout Stage 3 Task 7 (TextEncoder scope expansion): `new
    // TextEncoder().encode(<string>)` is a thin reinterpret of the string handle
    // to a contiguous byte buffer — it emits NO host import (fully guest-side) and
    // lowers cleanly when the resulting buffer is consumed (here by
    // `crypto.subtle.digest`). `.byteLength == UTF-8 byte count` is verified
    // end-to-end by the `runtime_smoke` `subtle_digest` node-parity target.
    let program = parse_and_lower_lir(
        "const b = new TextEncoder().encode('hello'); crypto.subtle.digest('SHA-256', b);",
    );
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
    // No host import is minted for TextEncoder().encode itself (pure reinterpret).
    assert!(
        !printed.to_lowercase().contains("text_encoder") && !printed.contains("TextEncoder"),
        "{printed}"
    );
}
