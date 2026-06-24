use crate::*;


#[test]
fn runtime_context_carries_process_identity() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));

    assert_eq!(runtime.process_id(), std::process::id());
    assert_eq!(KaliHostState::default().process_id(), std::process::id());
}


#[test]
fn runtime_context_exposes_deterministic_env_snapshots() {
    let mut runtime = RuntimeCtx::with_host_context(
        None,
        Vec::new(),
        BTreeMap::from([
            (String::from("BETA"), String::from("2")),
            (String::from("ALPHA"), String::from("1")),
        ]),
        PathBuf::from("."),
    );

    let snapshot = runtime.env_snapshot();
    let snapshot_keys = snapshot.keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        snapshot_keys,
        vec![String::from("ALPHA"), String::from("BETA")]
    );
    assert!(runtime.env_has("ALPHA"));
    assert!(runtime.has("BETA"));
    assert!(!runtime.env_has("GAMMA"));
    assert_eq!(snapshot.get("ALPHA"), Some(&String::from("1")));
    assert_eq!(snapshot.get("BETA"), Some(&String::from("2")));
    assert_eq!(runtime.env_to_object(), snapshot);
    assert_eq!(runtime.snapshot(), snapshot);
    assert_eq!(runtime.env_snapshot_object_value(), snapshot);

    let json_snapshot = runtime.env_snapshot_value();
    let json_snapshot = json_snapshot.as_object().expect("json object");
    assert_eq!(
        json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );
    assert_eq!(
        runtime.env_snapshot_json_value(),
        runtime.env_snapshot_value()
    );
    assert_eq!(runtime.snapshot_value(), runtime.env_snapshot_value());
    assert_eq!(
        runtime.snapshot_object_value(),
        runtime.env_snapshot_value()
    );
    assert_eq!(runtime.snapshot_json_value(), runtime.env_snapshot_value());
    assert_eq!(runtime.env_to_json_value(), runtime.env_snapshot_value());

    runtime.env.insert(String::from("GAMMA"), String::from("3"));
    assert!(!snapshot.contains_key("GAMMA"));
    assert!(!json_snapshot.contains_key("GAMMA"));

    let host_state = KaliHostState {
        env: BTreeMap::from([
            (String::from("BETA"), String::from("2")),
            (String::from("ALPHA"), String::from("1")),
        ]),
        ..KaliHostState::default()
    };

    let host_snapshot = host_state.env_snapshot();
    assert_eq!(
        host_snapshot.keys().cloned().collect::<Vec<_>>(),
        vec![String::from("ALPHA"), String::from("BETA")]
    );
    assert!(host_state.env_has("ALPHA"));
    assert!(host_state.has("BETA"));
    assert!(!host_state.env_has("GAMMA"));
    assert_eq!(host_state.env_to_object(), host_snapshot);
    assert_eq!(host_state.env_snapshot_object_value(), host_snapshot);

    let host_json_snapshot = host_state.env_snapshot_value();
    let host_json_snapshot = host_json_snapshot.as_object().expect("json object");
    assert_eq!(
        host_json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        host_json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );
    assert_eq!(
        host_state.env_snapshot_json_value(),
        host_state.env_snapshot_value()
    );
    assert_eq!(host_state.snapshot(), host_state.thread_topology_snapshot());
    assert_eq!(
        host_state.snapshot_object_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.snapshot_json_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.snapshot_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.env_to_json_value(),
        host_state.env_snapshot_value()
    );
}


#[test]
fn runtime_context_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    assert_eq!(runtime.api_surface, "deno");
    assert_eq!(
        runtime.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}


#[test]
fn runtime_context_carries_thread_budget_override() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(0));

    assert_eq!(runtime.max_threads, Some(0));
}


#[test]
fn runtime_context_resolves_effective_thread_budget() {
    let policy_path = tempfile::NamedTempFile::new().expect("policy temp file");
    std::fs::write(
        policy_path.path(),
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy fixture");
    let policy = SandboxPolicy::from_file(policy_path.path()).expect("load policy");

    let runtime = RuntimeCtx::with_api_surface(Some(policy), "deno").with_max_threads(Some(2));
    assert_eq!(runtime.effective_thread_budget(), Some(0));

    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(0));
    assert_eq!(runtime.effective_thread_budget(), Some(0));
}


#[test]
fn runtime_context_resolves_positive_thread_budget_against_policy_cap() {
    let policy_path = tempfile::NamedTempFile::new().expect("policy temp file");
    std::fs::write(
        policy_path.path(),
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy fixture");
    let runtime_profiles = vec!["wasm-threads".to_string()];
    let policy =
        SandboxPolicy::from_file_with_runtime_profiles(policy_path.path(), &runtime_profiles)
            .expect("load policy");

    let runtime =
        RuntimeCtx::with_api_surface(Some(policy.clone()), "deno").with_max_threads(Some(2));
    assert_eq!(runtime.effective_thread_budget(), Some(1));

    let runtime = RuntimeCtx::with_api_surface(Some(policy), "deno").with_max_threads(Some(1));
    assert_eq!(runtime.effective_thread_budget(), Some(1));
}
