use super::*;

#[test]
fn runtime_exposes_performance_now() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "performance_now" (func $now (result f64)))
                (func (export "_start")
                    call $now
                    f64.const 0.0
                    f64.ge
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_fills_random_values() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "crypto_get_random_values" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 16
                    call $random
                    i32.const 16
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_exposes_crypto_random_uuid() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "cryptoRandomUUID" (func $uuid (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 36
                    call $uuid
                    i32.const 36
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_computes_subtle_digest() {
    // throw-fallout Stage 3 Task 7: the `kali:rt` `crypto_subtle_digest` host
    // import reads the algorithm name + input bytes from guest memory, computes
    // the digest via `kali_api_web`'s `SubtleCrypto::digest`, writes the raw
    // digest bytes to `out_ptr`, and returns the digest length. SHA-256 of "abc"
    // is `ba7816bf...` — assert the returned length is 32 and the first byte is
    // 0xBA.
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "crypto_subtle_digest"
                    (func $digest (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "SHA-256")
                (data (i32.const 16) "abc")
                (func (export "_start")
                    ;; digest("SHA-256", "abc") into out_ptr=32, out_cap=64
                    i32.const 0    ;; algo_ptr
                    i32.const 7    ;; algo_len
                    i32.const 16   ;; in_ptr
                    i32.const 3    ;; in_len
                    i32.const 32   ;; out_ptr
                    i32.const 64   ;; out_cap
                    call $digest
                    i32.const 32
                    i32.eq
                    if
                    else
                        unreachable
                    end
                    ;; first digest byte must be 0xBA (186)
                    i32.const 32
                    i32.load8_u
                    i32.const 186
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}
