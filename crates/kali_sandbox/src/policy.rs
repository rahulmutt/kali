use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Declarative sandbox policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SandboxPolicy {
    /// Schema version. Schema v1 uses `1`.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Optional schema URI for tooling.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema_uri: Option<String>,
    /// Capability policy block.
    pub effects: EffectsPolicy,
    /// Cross-cutting runtime budgets.
    pub resources: ResourceLimits,
    /// Base directory used when resolving relative policy patterns.
    #[serde(default = "default_base_dir", skip_serializing)]
    pub base_dir: PathBuf,
    /// Original policy bytes as read from disk for deterministic artifact embedding.
    #[serde(default, skip_serializing)]
    pub serialized_source: Option<Vec<u8>>,
}

/// Sandbox capability policy block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectsPolicy {
    #[serde(rename = "fileSystem")]
    pub file_system: FileSystemPolicy,
    pub network: NetworkPolicy,
    pub process: ProcessPolicy,
    pub timer: TimerPolicy,
    pub eval: bool,
    pub random: bool,
    pub console: bool,
}

/// File-system related capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSystemPolicy {
    pub read: AccessRule,
    pub write: AccessRule,
}

/// Network related capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkPolicy {
    pub fetch: AccessRule,
    pub connect: AccessRule,
    pub listen: AccessRule,
    #[serde(rename = "maxConnections")]
    pub max_connections: Option<u64>,
}

/// Process-related capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessPolicy {
    pub spawn: AccessRule,
    #[serde(rename = "envRead")]
    pub env_read: AccessRule,
    #[serde(rename = "envWrite")]
    pub env_write: AccessRule,
}

/// Timer policy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimerPolicy {
    pub schedule: bool,
    #[serde(rename = "maxTimeoutMs")]
    pub max_timeout_ms: Option<u64>,
    #[serde(rename = "maxActiveTimers")]
    pub max_active_timers: Option<u64>,
}

/// Cross-cutting runtime budgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    #[serde(rename = "maxMemoryMB")]
    pub max_memory_mb: Option<u64>,
    #[serde(rename = "maxCpuTimeMs")]
    pub max_cpu_time_ms: Option<u64>,
    #[serde(rename = "maxOpenFiles")]
    pub max_open_files: Option<u64>,
    #[serde(rename = "maxSpawnedProcesses")]
    pub max_spawned_processes: Option<u64>,
    #[serde(rename = "maxThreads")]
    pub max_threads: Option<u64>,
}

/// Capability allow/deny shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AccessRule {
    Deny(bool),
    AllowList(Vec<String>),
}

fn default_schema_version() -> u32 {
    1
}

fn default_base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
