use super::*;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

fn write_source_fixture(source: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("kali-sandbox-{unique}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    fs::write(&path, source).expect("write source fixture");
    path
}

fn valid_policy() -> SandboxPolicy {
    SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: EffectsPolicy {
            file_system: FileSystemPolicy {
                read: AccessRule::AllowList(vec!["/data/**".into()]),
                write: AccessRule::Deny(false),
            },
            network: NetworkPolicy {
                fetch: AccessRule::AllowList(vec!["https://example.com/**".into()]),
                connect: AccessRule::Deny(false),
                listen: AccessRule::Deny(false),
                max_connections: Some(4),
            },
            process: ProcessPolicy {
                spawn: AccessRule::Deny(false),
                env_read: AccessRule::AllowList(vec!["PATH".into()]),
                env_write: AccessRule::Deny(false),
            },
            timer: TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(5_000),
                max_active_timers: Some(8),
            },
            eval: false,
            random: true,
            console: true,
        },
        resources: ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(10_000),
            max_open_files: Some(16),
            max_spawned_processes: Some(1),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("/workspace"),
        serialized_source: None,
    }
}

#[test]
fn policy_validates_and_serializes() {
    let policy = valid_policy();
    assert!(policy.validate().is_ok());
    let json = policy.to_canonical_json().expect("canonical json");
    assert!(json.contains("\"schemaVersion\":1"));
    assert!(json.contains("\"effects\""));
}

#[test]
fn policy_thread_budget_helper_preserves_zero_cap_tightening() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(4);

    assert_eq!(policy.effective_thread_budget(None), Some(4));
    assert_eq!(policy.effective_thread_budget(Some(0)), Some(0));
    assert_eq!(policy.effective_thread_budget(Some(2)), Some(2));
}

#[test]
fn policy_rejects_thread_spawn_when_no_budget_is_available() {
    let mut policy = valid_policy();
    policy.resources.max_threads = None;

    let diagnostic = policy
        .check_operation(HostOperation::ThreadSpawn { active_threads: 0 })
        .expect_err("thread creation should remain gated without a budget");

    assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(diagnostic.message.contains("resources.maxThreads"));
}

#[test]
fn policy_rejects_thread_spawn_when_the_budget_is_zero() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(0);

    let diagnostic = policy
        .check_operation(HostOperation::ThreadSpawn { active_threads: 0 })
        .expect_err("zero-cap thread budgets should deny thread creation");

    assert_eq!(diagnostic.code, Some(e4::RESOURCE_LIMIT_EXCEEDED as u32));
    assert!(diagnostic.message.contains("active thread count 1"));
}

#[test]
fn policy_spawn_budget_helper_combines_policy_and_override() {
    let mut policy = valid_policy();
    policy.resources.max_spawned_processes = Some(4);

    assert_eq!(policy.effective_spawn_budget(None), Some(4));
    assert_eq!(policy.effective_spawn_budget(Some(0)), Some(0));
    assert_eq!(policy.effective_spawn_budget(Some(2)), Some(2));
}

#[test]
fn policy_rejects_unavailable_capabilities() {
    let mut policy = valid_policy();
    policy.effects.process.env_write = AccessRule::Deny(true);
    policy.effects.network.connect = AccessRule::Deny(true);
    policy.effects.eval = true;
    policy.resources.max_threads = Some(1);

    let validation = policy.validate_policy();
    assert!(!validation.valid);
    assert!(validation
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
}

#[test]
fn predicate_registry_rejects_when_disabled() {
    let policy = valid_policy();
    let registry = PolicyPredicateRegistry::disabled();

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("disabled predicate registry should reject evaluation");

    assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(diagnostic
        .message
        .contains("host-registered sandbox predicates"));
}

#[test]
fn registered_predicates_run_after_declarative_allowance() {
    let policy = valid_policy();
    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "deny-stdout", |context| {
        context.subject != "stdout"
    });

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("predicate should narrow console access");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-stdout'"));
    assert!(diagnostic.message.contains("effects.console"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: effects.console"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "subject: stdout"));
}

#[test]
fn predicate_context_records_process_spawn_details() {
    let operation = HostOperation::ProcessSpawn {
        executable: "deno".to_string(),
    };
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "effects.process.spawn");
    assert_eq!(context.subject, "deno");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("executable").map(String::as_str),
        Some("deno")
    );
}

#[test]
fn predicate_context_records_file_network_and_env_details() {
    let file_read = PolicyPredicateContext::from_operation(&HostOperation::FileRead {
        path: PathBuf::from("/workspace/input.txt"),
    });
    assert_eq!(file_read.capability, "effects.fileSystem.read");
    assert_eq!(file_read.subject, "/workspace/input.txt");
    assert_eq!(
        file_read.details.get("path").map(String::as_str),
        Some("/workspace/input.txt")
    );

    let network_fetch = PolicyPredicateContext::from_operation(&HostOperation::NetworkFetch {
        url: "https://example.com/api".to_string(),
    });
    assert_eq!(network_fetch.capability, "effects.network.fetch");
    assert_eq!(network_fetch.subject, "https://example.com/api");
    assert_eq!(
        network_fetch.details.get("url").map(String::as_str),
        Some("https://example.com/api")
    );

    let env_write = PolicyPredicateContext::from_operation(&HostOperation::EnvironmentWrite {
        key: "KALI_FLAG".to_string(),
    });
    assert_eq!(env_write.capability, "effects.process.envWrite");
    assert_eq!(env_write.subject, "KALI_FLAG");
    assert_eq!(
        env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );

    let process_env_write =
        PolicyPredicateContext::from_operation(&HostOperation::ProcessEnvWrite {
            key: "KALI_FLAG".to_string(),
        });
    assert_eq!(process_env_write.capability, "effects.process.envWrite");
    assert_eq!(process_env_write.subject, "KALI_FLAG");
    assert_eq!(
        process_env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );
}

#[test]
fn predicate_context_records_remaining_host_specific_details() {
    let file_write = PolicyPredicateContext::from_operation(&HostOperation::FileWrite {
        path: PathBuf::from("/workspace/output.txt"),
    });
    assert_eq!(file_write.capability, "effects.fileSystem.write");
    assert_eq!(file_write.subject, "/workspace/output.txt");
    assert_eq!(
        file_write.details.get("path").map(String::as_str),
        Some("/workspace/output.txt")
    );

    let network_connect = PolicyPredicateContext::from_operation(&HostOperation::NetworkConnect {
        target: "127.0.0.1:80".to_string(),
    });
    assert_eq!(network_connect.capability, "effects.network.connect");
    assert_eq!(network_connect.subject, "127.0.0.1:80");
    assert_eq!(
        network_connect.details.get("target").map(String::as_str),
        Some("127.0.0.1:80")
    );

    let network_listen = PolicyPredicateContext::from_operation(&HostOperation::NetworkListen {
        target: "127.0.0.1:0".to_string(),
    });
    assert_eq!(network_listen.capability, "effects.network.listen");
    assert_eq!(network_listen.subject, "127.0.0.1:0");
    assert_eq!(
        network_listen.details.get("target").map(String::as_str),
        Some("127.0.0.1:0")
    );

    let environment_read =
        PolicyPredicateContext::from_operation(&HostOperation::EnvironmentRead {
            key: "PATH".to_string(),
        });
    assert_eq!(environment_read.capability, "effects.process.envRead");
    assert_eq!(environment_read.subject, "PATH");
    assert_eq!(
        environment_read.details.get("key").map(String::as_str),
        Some("PATH")
    );

    let timer_schedule = PolicyPredicateContext::from_operation(&HostOperation::TimerSchedule {
        delay_ms: 250,
        active_timers: 2,
    });
    assert_eq!(timer_schedule.capability, "effects.timer.schedule");
    assert_eq!(timer_schedule.subject, "250");
    assert_eq!(
        timer_schedule
            .details
            .get("activeTimers")
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(
        timer_schedule.details.get("delayMs").map(String::as_str),
        Some("250")
    );

    let console = PolicyPredicateContext::from_operation(&HostOperation::Console);
    assert_eq!(console.capability, "effects.console");
    assert_eq!(console.subject, "stdout");
    assert!(console.details.is_empty());

    let random = PolicyPredicateContext::from_operation(&HostOperation::Random);
    assert_eq!(random.capability, "effects.random");
    assert_eq!(random.subject, "random");
    assert!(random.details.is_empty());

    let eval = PolicyPredicateContext::from_operation(&HostOperation::Eval);
    assert_eq!(eval.capability, "effects.eval");
    assert_eq!(eval.subject, "eval");
    assert!(eval.details.is_empty());
}

#[test]
fn predicate_context_records_thread_spawn_details() {
    let operation = HostOperation::ThreadSpawn { active_threads: 3 };
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "resources.maxThreads");
    assert_eq!(context.subject, "3");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("activeThreads").map(String::as_str),
        Some("3")
    );
}

#[test]
fn registered_predicates_can_inspect_host_specific_context_details() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(2);

    let seen_details = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let mut registry = PolicyPredicateRegistry::enabled();
    let seen_details_clone = Arc::clone(&seen_details);
    registry.register(
        "resources.maxThreads",
        "deny-nonzero-threads",
        move |context| {
            seen_details_clone
                .lock()
                .expect("details mutex")
                .push(context.details.get("activeThreads").cloned());
            context
                .details
                .get("activeThreads")
                .is_some_and(|count| count == "0")
        },
    );

    let diagnostic = policy
        .check_operation_with_predicates(
            HostOperation::ThreadSpawn { active_threads: 1 },
            &registry,
        )
        .expect_err("predicate should narrow thread creation");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-nonzero-threads'"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: resources.maxThreads"));
    assert!(diagnostic.notes.iter().any(|note| note == "subject: 1"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "detail activeThreads: 1"));
    assert_eq!(
        *seen_details.lock().expect("details mutex"),
        vec![Some(String::from("1"))]
    );
}

#[test]
fn declarative_denials_stay_primary_over_predicates() {
    let mut policy = valid_policy();
    policy.effects.console = false;

    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "allow-all", |_| true);

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("declarative deny should win");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic.message.contains("Console output is not allowed"));
    assert!(!diagnostic.message.contains("host-registered predicate"));
}

#[test]
fn registered_predicates_run_in_registration_order() {
    let policy = valid_policy();
    let log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let mut registry = PolicyPredicateRegistry::enabled();

    let first_log = Arc::clone(&log);
    registry.register("effects.console", "first", move |_| {
        first_log.lock().expect("log mutex").push("first");
        true
    });

    let second_log = Arc::clone(&log);
    registry.register("effects.console", "second", move |_| {
        second_log.lock().expect("log mutex").push("second");
        false
    });

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("second predicate should reject after first passes");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert_eq!(*log.lock().expect("log mutex"), vec!["first", "second"]);
}

#[test]
fn access_rules_match_globs() {
    let policy = valid_policy();
    assert!(policy
        .effects
        .file_system
        .read
        .allows_path(Path::new("/data/input.txt"), &policy.base_dir));
    assert!(!policy
        .effects
        .file_system
        .read
        .allows_path(Path::new("/secret/input.txt"), &policy.base_dir));
    assert!(policy.effects.network.fetch.allows_candidate(
        "https://example.com/a/b",
        &policy.base_dir,
        PatternKind::Url
    ));
}

#[test]
fn effect_analysis_tracks_phase_three_deno_host_capabilities() {
    let source = write_source_fixture(
        r#"
Deno.env.set('KALI_CORPUS_FLAG', 'set');
new Deno.Command('sh').spawn();
Deno.connect('127.0.0.1', 1);
Deno.listen('127.0.0.1', 0);
Deno.serve('127.0.0.1', 0);
Deno.open('/workspace/input.txt');
Deno.create('/workspace/output.txt');
Deno.mkdir('/workspace/newdir');
Deno.remove('/workspace/old.txt');
Deno.rename('/workspace/from.txt', '/workspace/to.txt');
Deno.lstat('/workspace/input.txt');
"#,
    );

    let inference = infer_effects_from_roots(&[source], EffectAnalysisContext::new("deno"))
        .expect("infer effects");

    let kinds: Vec<_> = inference
        .effects
        .iter()
        .map(|effect| effect.kind.as_str())
        .collect();
    for kind in [
        "FileSystem.Read",
        "FileSystem.Write",
        "Network.Connect",
        "Network.Listen",
        "Process.EnvWrite",
        "Process.Spawn",
    ] {
        assert!(
            kinds.contains(&kind),
            "missing effect kind {kind:?}: {kinds:?}"
        );
    }

    let diagnostics = compare_effects_to_policy(&inference.effects, &valid_policy());
    assert!(
        diagnostics.len() >= 4,
        "expected policy diagnostics for the phase-three capability slice, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diag| diag.code == Some(9007)),
        "expected an E9007 policy mismatch diagnostic: {diagnostics:?}"
    );
    assert!(inference.dynamic_reasons.is_empty());
}

#[test]
fn effect_analysis_marks_computed_deno_host_access_as_dynamic() {
    let source = write_source_fixture(
        r#"
globalThis["Deno"]["env"]["set"]('KALI_CORPUS_FLAG', 'set');
"#,
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.EnvWrite"));
}

#[test]
fn effect_analysis_marks_proxy_constructor_and_revocable_calls_as_dynamic() {
    let source = write_source_fixture(
        r#"
new Proxy({}, {});
new globalThis.Proxy({}, {});
Proxy.revocable({}, {});
globalThis.Proxy.revocable({}, {});
"#,
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["proxy-traps"]);
    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(report.dynamic_effects);
    assert_eq!(report.dynamic_reasons, vec!["proxy-traps"]);
}

#[test]
fn effect_reports_normalize_dynamic_reasons_and_analysis_context_axes() {
    let mut context = EffectAnalysisContext::new("deno");
    context.runtime_profiles = vec![
        "wasm-threads".to_string(),
        "alpha".to_string(),
        "wasm-threads".to_string(),
    ];
    context.compat_features = vec!["beta".to_string(), "alpha".to_string(), "beta".to_string()];

    let report = effect_report_from_inference(
        vec!["main.ts".to_string()],
        context,
        EffectInference {
            effects: Vec::new(),
            dynamic_reasons: vec![
                "proxy-traps".to_string(),
                "eval".to_string(),
                "proxy-traps".to_string(),
            ],
        },
    );

    assert_eq!(report.dynamic_reasons, vec!["eval", "proxy-traps"]);
    assert!(report.dynamic_effects);
    assert_eq!(
        report.analysis_context.runtime_profiles,
        vec!["alpha", "wasm-threads"]
    );
    assert_eq!(
        report.analysis_context.compat_features,
        vec!["alpha", "beta"]
    );
}

#[test]
fn effect_reports_deduplicate_entry_points_while_preserving_first_seen_order() {
    let report = effect_report_from_inference(
        vec![
            "src/main.ts".to_string(),
            "src/helper.ts".to_string(),
            "src/main.ts".to_string(),
            "src/other.ts".to_string(),
            "src/helper.ts".to_string(),
        ],
        EffectAnalysisContext::new("deno"),
        EffectInference {
            effects: Vec::new(),
            dynamic_reasons: Vec::new(),
        },
    );

    assert_eq!(
        report.entry_points,
        vec!["src/main.ts", "src/helper.ts", "src/other.ts"]
    );
}

#[test]
fn effect_reports_treat_permissions_query_as_effect_free() {
    let source = write_source_fixture(r#"Deno.permissions.query({ name: "env" });"#);
    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );
    assert!(
        inference.dynamic_reasons.is_empty(),
        "unexpected dynamic reasons: {inference:?}"
    );

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(!report.dynamic_effects);
    assert!(report.dynamic_reasons.is_empty());
    assert!(report.effects.is_empty());
}
