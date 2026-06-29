use super::*;

#[test]
fn runtime_reports_thread_topology_snapshot_for_spawned_threads() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(2));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/alpha.js")
                (data (i32.const 32) "https://e.co/beta.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    call $thread_spawn
                    drop
                    i32.const 32
                    i32.const 20
                    call $thread_spawn
                    drop))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("execute wasm");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.thread_topology.total_instances, 2);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 2);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://e.co/alpha.js"
    );
    assert_eq!(
        outcome.thread_topology.live_instances[0].posted_messages,
        Vec::<serde_json::Value>::new()
    );
    assert_eq!(
        outcome.thread_topology.live_instances[0].posted_shared_buffers,
        Vec::<Vec<u8>>::new()
    );
    assert!(!outcome.thread_topology.live_instances[0].was_terminated);
    assert_eq!(outcome.thread_topology.live_instances[1].instance_id, 1);
    assert_eq!(
        outcome.thread_topology.live_instances[1].script_url,
        "https://e.co/beta.js"
    );
    assert_eq!(
        outcome.thread_topology.live_instances[1].posted_messages,
        Vec::<serde_json::Value>::new()
    );
    assert_eq!(
        outcome.thread_topology.live_instances[1].posted_shared_buffers,
        Vec::<Vec<u8>>::new()
    );
    assert!(!outcome.thread_topology.live_instances[1].was_terminated);
}

#[test]
fn runtime_rejects_thread_spawn_host_imports_when_budget_is_zero() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(0));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("zero thread budgets should deny thread creation through the host import");
    assert_eq!(
        diagnostics[0].code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(diagnostics[0]
        .message
        .contains("active thread count 1 exceeds policy limit of 0"));
}

#[test]
fn runtime_executes_thread_spawn_host_imports() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("thread spawn host import");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);

    let instance = &outcome.thread_topology.live_instances[0];
    assert_eq!(instance.instance_id, 0);
    assert_eq!(instance.script_url, "https://e.co/t.js");
    assert!(instance.posted_messages.is_empty());
    assert!(instance.posted_shared_buffers.is_empty());
    assert!(!instance.was_terminated);

    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://e.co/t.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
}

#[test]
fn runtime_execute_tests_reports_thread_topology_from_thread_spawn() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let outcome = runtime
        .execute_tests(&wasm)
        .expect("thread spawn host import in test mode");
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://e.co/t.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
}

#[test]
fn runtime_rejects_second_thread_spawn_host_import_when_budget_is_exhausted() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (data (i32.const 32) "https://e.co/u.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop
                    i32.const 32
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("second thread spawn should exhaust the one-thread budget");
    assert_eq!(
        diagnostics[0].code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(diagnostics[0]
        .message
        .contains("active thread count 2 exceeds policy limit of 1"));
}
