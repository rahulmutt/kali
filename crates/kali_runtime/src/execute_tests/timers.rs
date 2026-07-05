use super::*;

#[test]
fn runtime_drains_microtasks_before_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    i32.const 1
                    global.set $state)
                (func (export "__kali_callback_2")
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "_start")
                    i32.const 1
                    call $queue_microtask
                    i32.const 2
                    i32.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_repeating_intervals_can_be_cleared_from_callbacks() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32) (result i32)))
                (import "kali:rt" "clearInterval" (func $clear_interval (param i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (global $timer_id (mut i32) (i32.const -1))
                (func (export "__kali_callback_3")
                    global.get $state
                    i32.const 1
                    i32.add
                    global.set $state
                    global.get $state
                    i32.const 2
                    i32.eq
                    if
                        global.get $timer_id
                        call $clear_interval
                    else
                        global.get $state
                        i32.const 2
                        i32.gt_s
                        if
                            unreachable
                        end
                    end)
                (func (export "_start")
                    i32.const 3
                    i32.const 0
                    call $set_interval
                    global.set $timer_id)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_traps_from_the_entrypoint() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    i64.const 0
                    i64.div_s
                    drop)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome.trap.expect("division by zero should trap");
    assert_eq!(diagnostic.code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(diagnostic.message.contains("runtime trap"));
}

#[test]
fn runtime_can_clear_scheduled_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (import "kali:rt" "clearTimeout" (func $clear_timeout (param i32)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i32.const 7
                    i32.const 0
                    call $set_timeout
                    call $clear_timeout)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_rejects_negative_timer_delays() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (func (export "__kali_callback_9")
                    unreachable)
                (func (export "_start")
                    i32.const 9
                    i32.const -1
                    call $set_timeout
                    drop)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome
        .trap
        .expect("negative timer delays should be rejected");
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostic.message.contains("runtime trap"));
}

#[test]
fn runtime_rejects_negative_interval_delays() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32) (result i32)))
                (func (export "__kali_callback_10")
                    unreachable)
                (func (export "_start")
                    i32.const 10
                    i32.const -1
                    call $set_interval
                    drop)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome
        .trap
        .expect("negative timer delays should be rejected");
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostic.message.contains("runtime trap"));
}
