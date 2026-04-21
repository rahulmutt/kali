use super::*;
use std::{
    fs,
    path::PathBuf,
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
fn effect_reports_normalize_analysis_context_axes() {
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
            dynamic_reasons: Vec::new(),
        },
    );

    assert_eq!(
        report.analysis_context.runtime_profiles,
        vec!["alpha", "wasm-threads"]
    );
    assert_eq!(
        report.analysis_context.compat_features,
        vec!["alpha", "beta"]
    );
}
