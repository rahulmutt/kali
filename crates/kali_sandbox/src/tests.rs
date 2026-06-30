use super::*;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

fn write_source_fixture(source: &str) -> PathBuf {
    write_source_fixture_with_extension(source, "ts")
}

fn write_source_fixture_with_extension(source: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("kali-sandbox-{unique}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("main.{extension}"));
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
            max_spawned_processes: Some(0),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("/workspace"),
        serialized_source: None,
    }
}

#[path = "tests/policy.rs"]
mod policy;

#[path = "tests/predicates.rs"]
mod predicates;

#[path = "tests/effect_analysis.rs"]
mod effect_analysis;

#[path = "tests/effect_reports.rs"]
mod effect_reports;
