use super::*;

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
