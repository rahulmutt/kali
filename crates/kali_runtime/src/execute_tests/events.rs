use super::*;

#[test]
fn runtime_event_dispatch_invokes_listeners_synchronously_in_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Two listeners registered for "tick"; dispatch must run them IN ORDER,
    // synchronously (the state checks after the dispatch call happen inside
    // _start, before it returns).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
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
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 2
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    i32.const 1
                    i32.ne
                    if unreachable end        ;; dispatch must return 1 (true)
                    global.get $state
                    i32.const 2
                    i32.ne
                    if unreachable end)       ;; both listeners already ran — SYNCHRONOUS
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_dispatch_snapshot_excludes_listeners_added_during_dispatch() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // callback_1 (fired by dispatch #1) registers callback_2. Dispatch #1 must
    // NOT invoke callback_2 (snapshot semantics); dispatch #2 invokes both.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (global $t (mut i64) (i64.const 0))
                (global $ones (mut i32) (i32.const 0))
                (global $twos (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $ones
                    i32.const 1
                    i32.add
                    global.set $ones
                    global.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 2
                    i64.const 0
                    call $el_add)
                (func (export "__kali_callback_2")
                    global.get $twos
                    i32.const 1
                    i32.add
                    global.set $twos)
                (func (export "_start")
                    call $et_new
                    global.set $t
                    global.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    global.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    ;; after dispatch #1: ones=1, twos=0 (snapshot excluded cb2)
                    global.get $twos
                    if unreachable end
                    global.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    ;; after dispatch #2: ones=2, twos=1
                    global.get $ones
                    i32.const 2
                    i32.ne
                    if unreachable end
                    global.get $twos
                    i32.const 1
                    i32.ne
                    if unreachable end)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_duplicate_registration_dedups_by_callback_and_env() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Same (callback_id, env_ptr) registered twice → fires ONCE per dispatch
    // (node dedups by listener identity). A different env_ptr is a different
    // listener and fires separately.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (global $count (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $count
                    i32.const 1
                    i32.add
                    global.set $count)
                (func (export "_start")
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add          ;; exact duplicate — dedup
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 7
                    call $el_add          ;; different env_ptr — distinct listener
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    global.get $count
                    i32.const 2
                    i32.ne
                    if unreachable end)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_dispatch_restores_current_env_and_zero_listeners_returns_true() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // The listener runs with __current_env = its registration env (42) and
    // _start's env (7) is restored afterward. Dispatching an event with no
    // listeners returns 1 (node: dispatchEvent with no listeners → true).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (data (i32.const 24) "none")
                (global $env (export "__current_env") (mut i64) (i64.const 0))
                (global $seen (mut i64) (i64.const -1))
                (func (export "__kali_callback_1")
                    global.get $env
                    global.set $seen)
                (func (export "_start")
                    (local $t i64)
                    i64.const 7
                    global.set $env
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 42
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    global.get $seen
                    i64.const 42
                    i64.ne
                    if unreachable end        ;; listener saw its registration env
                    global.get $env
                    i64.const 7
                    i64.ne
                    if unreachable end        ;; _start's env restored
                    local.get $t
                    i32.const 24
                    i32.const 4
                    call $ev_dispatch
                    i32.const 1
                    i32.ne
                    if unreachable end)       ;; zero listeners → still true
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_listener_trap_propagates_loudly() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (func (export "__kali_callback_1")
                    unreachable)
                (func (export "_start")
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    // The trap surfaces through the _start error path (the listener trapped
    // while _start was on the stack: event_dispatch's func_wrap re-enters the
    // guest synchronously, so wasmtime attributes the trap to the outer
    // `start.call()` in execute.rs, which produces `Ok(RuntimeOutcome {
    // trap: Some(diagnostic), exit_code: 1, .. })` — the SAME shape as
    // `runtime_reports_traps_from_the_entrypoint` in timers.rs, NOT the
    // drain-time `Err(Vec<Diagnostic>)` shape used by the negative-delay
    // timer trap tests (this dispatch never reaches `drain_event_loop`).
    // The listener's name only survives into `diagnostic.message` because
    // execute.rs's fallback trap arm renders `error` with `{:?}` (the full
    // anyhow "Caused by" chain) rather than `{}` (top context only) — the
    // extra host-import call boundary that a re-entrant dispatch crosses
    // pushes `invoke_callback_reentrant`'s message one level into that
    // chain, where plain `Display` can't see it. See execute.rs's comment
    // on that arm.
    let diagnostic_text = format!("{:?}", outcome);
    assert!(
        diagnostic_text.contains("__kali_callback_1"),
        "expected the trap to be attributed to the listener, got: {diagnostic_text}"
    );
}
