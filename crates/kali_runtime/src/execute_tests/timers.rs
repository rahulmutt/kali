use super::*;

#[test]
fn runtime_drains_microtasks_before_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32 i64)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
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
                    i64.const 0
                    call $queue_microtask
                    i32.const 2
                    i32.const 0
                    i64.const 0
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
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
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
                    i64.const 0
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
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (import "kali:rt" "clearTimeout" (func $clear_timeout (param i32)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i32.const 7
                    i32.const 0
                    i64.const 0
                    call $set_timeout
                    call $clear_timeout)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_timers_fire_in_delay_order_not_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Registers delay-10 BEFORE delay-5; the delay-5 callback must fire first.
    // State machine: cb_5 moves state 0->1; cb_10 requires state==1 else traps.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1") ;; the delay-10 callback
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "__kali_callback_2") ;; the delay-5 callback
                    i32.const 1
                    global.set $state)
                (func (export "_start")
                    i32.const 1
                    i32.const 10
                    i64.const 0
                    call $set_timeout
                    drop
                    i32.const 2
                    i32.const 5
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_large_delays_complete_instantly_under_the_virtual_clock() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1"))
                (func (export "_start")
                    i32.const 1
                    i32.const 60000
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let started = std::time::Instant::now();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    // Real-time drain would sleep 60s; the virtual clock must not sleep at all.
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "drain slept on a virtual timer: {:?}",
        started.elapsed()
    );
}

#[test]
fn runtime_equal_due_times_fire_in_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Two delay-0 timers: first-registered must fire first (seq tiebreak).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $state
                    i32.const 0
                    i32.eq
                    if
                        i32.const 1
                        global.set $state
                    else
                        unreachable
                    end)
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
                    i32.const 0
                    i64.const 0
                    call $set_timeout
                    drop
                    i32.const 2
                    i32.const 0
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_negative_delays_clamp_and_fire_instead_of_trapping() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Node parity: setTimeout(fn, -1) clamps to the 1ms minimum and FIRES.
    // (Flips the old reject-negative-delay semantics — a deliberate Stage D
    // decision; the two old `runtime_rejects_negative_*` tests are retargeted
    // in Step 3.)
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    i32.const 1
                    global.set $state)
                (func (export "_start")
                    i32.const 1
                    i32.const -1
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_negative_timeout_delay_fires_its_callback() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Stage D node parity: a negative delay clamps to the 1ms minimum and
    // FIRES rather than being rejected at schedule time. This still traps
    // (exit 1) because the callback body itself is `unreachable` — the trap
    // now comes from the FIRED callback, proving the clamped timer ran.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (func (export "__kali_callback_9")
                    unreachable)
                (func (export "_start")
                    i32.const 9
                    i32.const -1
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );

    // A callback trap during event-loop drain (as opposed to a trap inside
    // `_start` itself) propagates as an `Err` from `execute()` rather than an
    // `Ok(RuntimeOutcome { trap: Some(_), .. })` — see `execute.rs`'s
    // `drain_event_loop` error branch. Under the old reject-at-schedule-time
    // semantics this test's trap happened inside `_start` (the host import
    // call itself failed); under clamp-and-fire it now happens inside the
    // FIRED callback during drain, so the assertion targets the `Err` path.
    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("the clamped, fired callback should trap");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostic.message.contains("runtime trap"));
    assert!(
        diagnostic.message.contains("__kali_callback_"),
        "expected the trap to come from the FIRED callback, got: {}",
        diagnostic.message
    );
}

#[test]
fn runtime_negative_interval_delay_fires_its_callback() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Stage D node parity: a negative interval delay clamps to the 1ms
    // minimum and FIRES rather than being rejected at schedule time. The
    // `unreachable` callback traps on its first tick, so the drain
    // terminates before any re-arm — no budget interaction.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
                (func (export "__kali_callback_10")
                    unreachable)
                (func (export "_start")
                    i32.const 10
                    i32.const -1
                    i64.const 0
                    call $set_interval
                    drop)
            )
            "#,
    );

    // See the sibling timeout test's comment: a callback trap during drain
    // surfaces as an `Err` from `execute()`, not `outcome.trap`.
    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("the clamped, fired callback should trap");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostic.message.contains("runtime trap"));
    assert!(
        diagnostic.message.contains("__kali_callback_"),
        "expected the trap to come from the FIRED callback, got: {}",
        diagnostic.message
    );
}

#[test]
fn runtime_uncleared_interval_trips_the_drain_budget() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // An interval that never clears would drain forever (node parity would
    // hang); the bounded drain must trap loudly instead (Stage D decision).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1"))
                (func (export "_start")
                    i32.const 1
                    i32.const 0
                    i64.const 0
                    call $set_interval
                    drop)
            )
            "#,
    );
    let started = std::time::Instant::now();
    // A budget-exhaustion diagnostic during drain propagates as an `Err` from
    // `execute()` (same wrapping as a mid-drain callback trap — see the
    // `runtime_negative_*_fires_its_callback` tests above), not as
    // `Ok(RuntimeOutcome { trap: Some(_), .. })`.
    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("budget exhaustion must surface a diagnostic");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(
        diagnostic.message.contains("event loop did not quiesce"),
        "got: {}",
        diagnostic.message
    );
    // 100k no-op invocations under a virtual clock must be fast.
    assert!(started.elapsed() < std::time::Duration::from_secs(60));
}

#[test]
fn runtime_self_requeueing_microtask_trips_the_drain_budget() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32 i64)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1")
                    i32.const 1
                    i64.const 0
                    call $queue_microtask)
                (func (export "_start")
                    i32.const 1
                    i64.const 0
                    call $queue_microtask)
            )
            "#,
    );
    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("budget exhaustion must surface a diagnostic");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
}

#[test]
fn runtime_zero_delay_interval_does_not_starve_later_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // interval(0) clamps to 1ms; a timeout at 5ms must still get scheduled
    // (the timeout's callback clears the interval, so the drain terminates).
    // If the clamp regressed to 0, the interval would re-arm at the same
    // virtual instant forever and the budget trap (Err) would fire instead.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (import "kali:rt" "clearInterval" (func $clear_interval (param i32)))
                (memory (export "memory") 1)
                (global $interval_id (mut i32) (i32.const -1))
                (func (export "__kali_callback_1")) ;; interval tick: no-op
                (func (export "__kali_callback_2") ;; timeout: clears the interval
                    global.get $interval_id
                    call $clear_interval)
                (func (export "_start")
                    i32.const 1
                    i32.const 0
                    i64.const 0
                    call $set_interval
                    global.set $interval_id
                    i32.const 2
                    i32.const 5
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}
