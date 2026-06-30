use super::*;
use crate::compiler::temporary_source_path;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tempfile::tempdir;

fn permissive_policy() -> kali_sandbox::SandboxPolicy {
    kali_sandbox::SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: kali_sandbox::EffectsPolicy {
            file_system: kali_sandbox::FileSystemPolicy {
                read: kali_sandbox::AccessRule::Deny(false),
                write: kali_sandbox::AccessRule::Deny(false),
            },
            network: kali_sandbox::NetworkPolicy {
                fetch: kali_sandbox::AccessRule::Deny(false),
                connect: kali_sandbox::AccessRule::Deny(false),
                listen: kali_sandbox::AccessRule::Deny(false),
                max_connections: None,
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::Deny(false),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: false,
                max_timeout_ms: None,
                max_active_timers: None,
            },
            eval: false,
            random: false,
            console: true,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: None,
            max_cpu_time_ms: None,
            max_open_files: None,
            max_spawned_processes: None,
            max_threads: None,
        },
        base_dir: std::path::PathBuf::from("."),
        serialized_source: None,
    }
}

#[path = "tests/compiler.rs"]
mod compiler;

#[path = "tests/runtime_profiles.rs"]
mod runtime_profiles;

#[path = "tests/context.rs"]
mod context;

#[path = "tests/predicates.rs"]
mod predicates;
