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

    // Review finding M-1: `diagnostics.is_empty()` + "it validates" does NOT
    // pin this lane — a regression back to the original silent defect
    // (`i64.const 0` handed to `console_log`) keeps both green. Pin the emitted
    // instructions instead: each of the two reads must lower to the array-base
    // address followed by the i64 length-header load at `+0`
    // (`i32.wrap_i64` / `i64.load`, no `offset=`), and NO `console_log` argument
    // may be a baked `i64.const 0`. Whitespace is normalized so indentation
    // changes in `wasmprinter` output cannot break the pin.
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    // The trailing space matters: without it `call 1` also matches the prefix of
    // `call 17` (`__streq`) and the count is wrong.
    let normalized = format!(
        "{} ",
        printed.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    // `call 1` is the `kali:rt` `console_log` import (index 1).
    assert!(
        normalized.contains("import \"kali:rt\" \"console_log\" (func (;1;)"),
        "console_log is no longer import index 1; re-anchor this pin: {printed}"
    );
    assert_eq!(
        normalized.matches("i32.wrap_i64 i64.load call 1 ").count(),
        2,
        "expected `.length` and `.byteLength` to each load the i64 length header \
         at +0 of the result handle: {printed}"
    );
    assert!(
        !normalized.contains("i64.const 0 call 1 "),
        "a console.log argument regressed to the baked silent zero: {printed}"
    );
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

/// Stage P5 T-new-A, review finding I-1: the deny domain is per-`FunctionEmitter`
/// (per LIR function), so a CAPTURING closure — its own `__kali_fn_N` function,
/// emitted by a different emitter with an empty set — read the result's
/// `.length` / `.byteLength` straight off the silent-zero placeholder. That is
/// byte-for-byte the defect this task exists to kill, surviving one scope
/// inwards (measured pre-fix: kali `0`, node `4`, `"warnings":[]`, exit 0).
/// The capturer now inherits the enclosing function's DENY domain (never its
/// admissions) via `EnvPlan::captured`, so both reads fail closed.
#[test]
fn crypto_get_random_values_result_read_in_a_capturing_closure_fails_closed() {
    for property in ["length", "byteLength"] {
        let source = format!(
            "function outer() {{\n\
             \x20 const rb = new Uint8Array(4);\n\
             \x20 const fb = crypto.getRandomValues(rb);\n\
             \x20 const g = () => {{ return fb.{property}; }};\n\
             \x20 return g();\n\
             }}\n\
             console.log(outer());\n"
        );
        let (program, env_plans) = parse_and_lower_lir_with_env_plans(&source);
        assert!(
            env_plans
                .values()
                .any(|plan| plan.captured.iter().any(|reference| reference.name == "fb")),
            "the fixture must actually CAPTURE `fb`, else this test pins nothing: {env_plans:?}"
        );
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        ctx.env_plans = env_plans;
        let result = lower_lir_to_wasm(&mut ctx, &program);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("crypto.getRandomValues(...) result")),
            "{source}: {:?}",
            result.diagnostics
        );
    }
}

/// Stage P5 T-new-A, review finding I-3: the deny domain is NAME-keyed, so
/// copying the result handle into an aggregate slot LAUNDERS it — the later
/// read's receiver (`o.buf`, `holder[0]`) has no binding name, every gate in
/// this lane misses it, and the read falls through to the pre-existing
/// aggregate-provenance bug, printing the object's field count / the holder's
/// length (measured pre-fix: `1`, `2`, `0` where node reads `4`). Every store of
/// a deny-domain value into an object field or array element now fails closed.
/// (The GENERAL aggregate-provenance bug is untouched and out of scope — it is
/// silent with no crypto involved and deserves its own task.)
#[test]
fn crypto_get_random_values_result_stored_into_an_aggregate_fails_closed() {
    let prelude =
        "const rb = new Uint8Array(4);\nconst fb = crypto.getRandomValues(rb);\nconst holder = new Array(2);\n";
    for tail in [
        // object literal field, folded lane (repr never inferred as Object)
        "const o = { buf: fb };\nconsole.log(o.buf.length);\n",
        // object literal field, INLINE unbound result
        "const o2 = { buf: crypto.getRandomValues(rb) };\nconsole.log(o2.buf.length);\n",
        // array literal element
        "const a = [fb];\nconsole.log(a[0].length);\n",
        // array element store
        "holder[0] = fb;\nconsole.log(holder[0].length);\n",
        // object field store
        "const o3 = { buf: 0 };\no3.buf = fb;\nconsole.log(o3.buf.length);\n",
    ] {
        let source = format!("{prelude}{tail}");
        let program = parse_and_lower_lir(&source);
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
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag
                        .message
                        .contains("storing a crypto.getRandomValues(...) result")),
            "{source}: {:?}",
            result.diagnostics
        );
    }
}
