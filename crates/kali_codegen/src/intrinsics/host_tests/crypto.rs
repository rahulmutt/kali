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

/// Stage P5 T-new-A: `.length` / `.byteLength` read off the RESULT of
/// `crypto.getRandomValues(buf)` lower to the i64 length-header load at `+0` of
/// the handle the result binding holds — the same lane the receiver binding's
/// own `.length` uses, since the call returns the argument handle unchanged.
/// End-to-end node parity is pinned by the `runtime_smoke` target
/// `run_supports_browser_web_crypto_get_random_values_result_length_*`.
#[test]
fn crypto_get_random_values_result_length_reads_the_buffer_length_header() {
    let program = parse_and_lower_lir(
        "const rb = new Uint8Array(8);\nconst fb = crypto.getRandomValues(rb);\nconsole.log(fb.length);\nconsole.log(fb.byteLength);\n",
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
        .expect("validate");
}

/// Stage P5 T-new-A: the remainder DENIES. An INLINE, unbound receiver
/// (`crypto.getRandomValues(rb).length`) is invisible to every name-keyed lane
/// and would otherwise fall through to a placeholder zero; admitting it would
/// additionally drop the buffer-filling side effect. Element reads of a result
/// binding deny for the same reason.
#[test]
fn crypto_get_random_values_result_outside_the_proven_path_fails_closed() {
    for source in [
        "const rb = new Uint8Array(8);\nconsole.log(crypto.getRandomValues(rb).length);\n",
        "const rb = new Uint8Array(8);\nconst fb = crypto.getRandomValues(rb);\nconsole.log(fb[0]);\n",
        "const o = { a: 1 };\nconst fb = crypto.getRandomValues(o);\nconsole.log(fb.length);\n",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);
        assert!(
            result.diagnostics.iter().any(|diag| diag.code
                == Some(e5::FEATURE_UNAVAILABLE as u32)
                && diag.message.contains("crypto.getRandomValues(...) result")),
            "{source}: {:?}",
            result.diagnostics
        );
    }
}
