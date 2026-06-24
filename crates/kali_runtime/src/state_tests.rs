use crate::test_support::*;
use crate::*;

#[test]
fn runtime_host_state_tracks_thread_budget_bookkeeping() {
    let mut state = KaliHostState::default();

    assert!(
        state.begin_thread().is_err(),
        "thread creation should be gated without the threaded profile opt-in"
    );

    state.runtime_profiles = vec!["wasm-threads".to_string()];
    assert!(
        state.begin_thread().is_err(),
        "thread creation should still be gated without a budget"
    );

    state.max_threads = Some(0);
    assert!(
        state.begin_thread().is_err(),
        "zero-cap budgets should deny thread creation"
    );

    state.max_threads = Some(2);
    assert!(state.begin_thread().is_ok());
    assert!(state.begin_thread().is_ok());
    assert!(state.begin_thread().is_err());
    state.finish_thread();
    assert!(state.begin_thread().is_ok());
}

#[test]
fn runtime_host_state_accepts_trimmed_threaded_runtime_profile() {
    let mut state = KaliHostState {
        runtime_profiles: vec![" wasm-threads ".to_string()],
        max_threads: Some(1),
        ..Default::default()
    };

    assert!(state.begin_thread().is_ok());
    assert_eq!(state.active_threads, 1);
}

#[test]
fn runtime_host_state_spawns_and_releases_thread_instances() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(2),
        ..Default::default()
    };

    let first = state
        .spawn_thread_instance("https://e.co/t.js")
        .expect("first thread instance");
    assert_eq!(first, 0);
    assert_eq!(state.active_threads, 1);
    assert_eq!(state.thread_topology.total_instances(), 1);

    let second = state
        .spawn_thread_instance("https://e.co/u.js")
        .expect("second thread instance");
    assert_eq!(second, 1);
    assert_eq!(state.active_threads, 2);
    assert_eq!(state.thread_topology.total_instances(), 2);

    let snapshot = state.thread_topology_snapshot();
    assert_eq!(snapshot.total_instances, 2);
    assert_eq!(snapshot.terminated_instances, 0);
    assert_eq!(snapshot.live_instances.len(), 2);
    assert_eq!(
        snapshot
            .live_instances
            .iter()
            .map(|entry| entry.instance_id)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_eq!(
        state.thread_topology_snapshot_value(),
        serde_json::json!({
            "totalInstances": 2,
            "terminatedInstances": 0,
            "liveInstances": [
                {
                    "instanceId": first,
                    "scriptUrl": "https://e.co/t.js",
                    "postedMessages": [],
                    "postedSharedBuffers": [],
                    "wasTerminated": false
                },
                {
                    "instanceId": second,
                    "scriptUrl": "https://e.co/u.js",
                    "postedMessages": [],
                    "postedSharedBuffers": [],
                    "wasTerminated": false
                }
            ]
        })
    );
    assert_eq!(
        state.thread_topology_snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.thread_topology_snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.thread_topology_snapshot().snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(state.snapshot(), state.thread_topology_snapshot());
    assert_eq!(snapshot.snapshot(), state.thread_topology_snapshot());
    assert_eq!(
        snapshot.thread_topology_snapshot_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );

    let diagnostic = state
        .spawn_thread_instance("https://e.co/v.js")
        .expect_err("thread budget should cap guest thread spawns");
    assert_eq!(
        state
            .pending_diagnostic
            .as_ref()
            .and_then(|diagnostic| diagnostic.code),
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(diagnostic.to_string().contains("KALI_E4003"));

    assert!(state.release_thread_instance(second));
    assert_eq!(state.active_threads, 1);
    assert!(state.release_thread_instance(first));
    assert_eq!(state.active_threads, 0);
    assert!(!state.release_thread_instance(first));

    let third = state
        .spawn_thread_instance("https://e.co/v.js")
        .expect("third thread instance after re-spawn");
    assert_eq!(third, 2);
    assert_eq!(state.active_threads, 1);
    assert_eq!(
        state.thread_topology.instance_ids(),
        vec![first, second, third]
    );
    assert_eq!(state.thread_topology.total_instances(), 3);

    let snapshot_after_respawn = state.thread_topology_snapshot();
    assert_eq!(snapshot_after_respawn.total_instances, 3);
    assert_eq!(snapshot_after_respawn.terminated_instances, 2);
    assert_eq!(snapshot_after_respawn.live_instances.len(), 1);
    assert_eq!(snapshot_after_respawn.live_instances[0].instance_id, third);
    assert_eq!(
        snapshot_after_respawn.live_instances[0].script_url,
        "https://e.co/v.js"
    );
    assert!(snapshot_after_respawn.live_instances[0]
        .posted_messages
        .is_empty());
    assert!(snapshot_after_respawn.live_instances[0]
        .posted_shared_buffers
        .is_empty());
    assert!(!snapshot_after_respawn.live_instances[0].was_terminated);
}

#[test]
fn runtime_host_state_trims_surrounding_whitespace_from_thread_script_urls() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(1),
        ..Default::default()
    };

    let instance_id = state
        .spawn_thread_instance("  https://e.co/padded.js \n")
        .expect("thread instance with trimmed script URL");
    assert_eq!(instance_id, 0);
    assert_eq!(state.active_threads, 1);
    assert_eq!(state.thread_topology.total_instances(), 1);

    let snapshot = state.thread_topology_snapshot();
    assert_eq!(snapshot.total_instances, 1);
    assert_eq!(snapshot.terminated_instances, 0);
    assert_eq!(snapshot.live_instances.len(), 1);
    assert_eq!(snapshot.live_instances[0].instance_id, instance_id);
    assert_eq!(
        snapshot.live_instances[0].script_url,
        "https://e.co/padded.js"
    );
    assert_eq!(
        snapshot.live_instances[0].posted_messages,
        Vec::<serde_json::Value>::new()
    );
    assert_eq!(
        snapshot.live_instances[0].posted_shared_buffers,
        Vec::<Vec<u8>>::new()
    );
    assert!(!snapshot.live_instances[0].was_terminated);
    assert_eq!(
        state.thread_topology_snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [
                {
                    "instanceId": instance_id,
                    "scriptUrl": "https://e.co/padded.js",
                    "postedMessages": [],
                    "postedSharedBuffers": [],
                    "wasTerminated": false
                }
            ]
        })
    );
}

#[test]
fn runtime_summary_parser_rejects_whitespace_padded_thread_script_urls() {
    let value = serde_json::json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": " https://e.co/padded.js ",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }
        ]
    });

    assert!(
        parse_thread_runtime_shutdown_report_value(Some(&value)).is_none(),
        "whitespace-padded scriptUrl should be rejected"
    );
}

#[test]
fn runtime_summary_parser_rejects_relative_thread_script_urls() {
    let value = serde_json::json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "worker.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }
        ]
    });

    assert!(
        parse_thread_runtime_shutdown_report_value(Some(&value)).is_none(),
        "relative scriptUrl should be rejected"
    );
}

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
fn runtime_host_state_rolls_back_failed_thread_spawns() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(1),
        ..Default::default()
    };

    let _error = state
        .spawn_thread_instance("not-a-valid-thread-url")
        .expect_err("invalid URLs should be rejected before they leak bookkeeping");
    assert_eq!(state.active_threads, 0);
    assert_eq!(state.thread_topology.total_instances(), 0);
}

#[test]
fn runtime_host_state_rejects_whitespace_only_thread_script_urls() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(1),
        ..Default::default()
    };

    let error = state
        .spawn_thread_instance("   ")
        .expect_err("whitespace-only URLs should be rejected before spawn bookkeeping");
    assert!(
        error.to_string().contains("non-empty absolute URL"),
        "error: {error}"
    );
    assert_eq!(state.active_threads, 0);
    assert_eq!(state.thread_topology.total_instances(), 0);
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
