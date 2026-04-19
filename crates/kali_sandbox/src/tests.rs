use super::*;

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
            max_spawned_processes: Some(0),
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
fn policy_rejects_unavailable_capabilities() {
    let mut policy = valid_policy();
    policy.effects.process.env_write = AccessRule::Deny(true);
    policy.effects.network.connect = AccessRule::Deny(true);
    policy.effects.eval = true;
    policy.resources.max_spawned_processes = Some(1);

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
