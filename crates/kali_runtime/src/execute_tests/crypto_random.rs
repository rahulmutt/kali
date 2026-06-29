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
