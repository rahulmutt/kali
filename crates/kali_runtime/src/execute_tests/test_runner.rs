use super::*;

#[test]
fn runtime_collects_and_runs_registered_tests() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_1")
                    i32.const 1
                    i32.const 1
                    i32.add
                    drop)
                (func (export "_start")
                    i32.const 1
                    call $test_register)
            )
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 0);
}

#[test]
fn runtime_reports_failed_registered_tests() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_2")
                    unreachable)
                (func (export "_start")
                    i32.const 2
                    call $test_register)
            )
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 1);
}
