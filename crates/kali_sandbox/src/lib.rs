//! Sandbox and policy system for the Kali compiler.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod effects;

pub use effects::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, EffectInference, EffectLocation,
    EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate, PackageEffectsReport,
};

use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic,
};
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

/// Host operations checked against the policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostOperation {
    Console,
    Random,
    FileRead {
        path: PathBuf,
    },
    FileWrite {
        path: PathBuf,
    },
    NetworkFetch {
        url: String,
    },
    NetworkConnect {
        target: String,
    },
    NetworkListen {
        target: String,
    },
    EnvironmentRead {
        key: String,
    },
    EnvironmentWrite {
        key: String,
    },
    TimerSchedule {
        delay_ms: u64,
        active_timers: usize,
    },
    ProcessSpawn {
        executable: String,
    },
    /// Thread creation request with the current active thread count.
    ThreadSpawn {
        active_threads: usize,
    },
    ProcessEnvWrite {
        key: String,
    },
    Eval,
}

/// Canonical context payload observed by host-registered narrowing predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPredicateContext {
    /// Canonical capability name from the sandbox vocabulary.
    pub capability: String,
    /// Subject string associated with the host operation.
    pub subject: String,
    /// Host operation being evaluated.
    pub operation: HostOperation,
    /// Deterministic extra details for host-specific predicate logic.
    pub details: BTreeMap<String, String>,
}

impl PolicyPredicateContext {
    /// Create the canonical predicate context for one host operation.
    pub fn from_operation(operation: &HostOperation) -> Self {
        let mut details = BTreeMap::new();
        let (capability, subject) = match operation {
            HostOperation::Console => ("effects.console", "stdout".to_string()),
            HostOperation::Random => ("effects.random", "random".to_string()),
            HostOperation::FileRead { path } => {
                ("effects.fileSystem.read", path.display().to_string())
            }
            HostOperation::FileWrite { path } => {
                ("effects.fileSystem.write", path.display().to_string())
            }
            HostOperation::NetworkFetch { url } => ("effects.network.fetch", url.clone()),
            HostOperation::NetworkConnect { target } => ("effects.network.connect", target.clone()),
            HostOperation::NetworkListen { target } => ("effects.network.listen", target.clone()),
            HostOperation::EnvironmentRead { key } => ("effects.process.envRead", key.clone()),
            HostOperation::EnvironmentWrite { key } => ("effects.process.envWrite", key.clone()),
            HostOperation::TimerSchedule {
                delay_ms,
                active_timers,
            } => {
                details.insert("activeTimers".to_string(), active_timers.to_string());
                details.insert("delayMs".to_string(), delay_ms.to_string());
                ("effects.timer.schedule", delay_ms.to_string())
            }
            HostOperation::ProcessSpawn { executable } => {
                details.insert("executable".to_string(), executable.clone());
                ("effects.process.spawn", executable.clone())
            }
            HostOperation::ThreadSpawn { active_threads } => {
                details.insert("activeThreads".to_string(), active_threads.to_string());
                ("resources.maxThreads", active_threads.to_string())
            }
            HostOperation::ProcessEnvWrite { key } => ("effects.process.envWrite", key.clone()),
            HostOperation::Eval => ("effects.eval", "eval".to_string()),
        };

        Self {
            capability: capability.to_string(),
            subject,
            operation: operation.clone(),
            details,
        }
    }
}

/// Deterministic host-registered narrowing predicate registry.
#[derive(Clone)]
pub struct PolicyPredicateRegistry {
    enabled: bool,
    predicates: BTreeMap<String, Vec<RegisteredPredicate>>,
}

#[derive(Clone)]
struct RegisteredPredicate {
    name: String,
    predicate: HostPredicate,
}

/// Host predicate function used by the embedding narrowing layer.
pub type HostPredicate = Arc<dyn Fn(&PolicyPredicateContext) -> bool + Send + Sync + 'static>;

impl Default for PolicyPredicateRegistry {
    fn default() -> Self {
        Self::enabled()
    }
}

impl PolicyPredicateRegistry {
    /// Create an enabled predicate registry.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            predicates: BTreeMap::new(),
        }
    }

    /// Create a disabled registry that deterministically rejects predicate evaluation.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            predicates: BTreeMap::new(),
        }
    }

    /// Return whether host-registered predicate evaluation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Register a deterministic narrowing predicate for one canonical capability name.
    pub fn register(
        &mut self,
        capability: impl Into<String>,
        name: impl Into<String>,
        predicate: impl Fn(&PolicyPredicateContext) -> bool + Send + Sync + 'static,
    ) -> &mut Self {
        let capability = capability.into();
        let entry = self.predicates.entry(capability).or_default();
        entry.push(RegisteredPredicate {
            name: name.into(),
            predicate: Arc::new(predicate),
        });
        self
    }

    fn evaluate(&self, context: &PolicyPredicateContext) -> Result<(), Diagnostic> {
        if !self.enabled {
            return Err(unavailable_capability("host-registered sandbox predicates"));
        }

        let Some(predicates) = self.predicates.get(&context.capability) else {
            return Ok(());
        };

        for predicate in predicates {
            if !(predicate.predicate)(context) {
                return Err(host_predicate_violation(
                    format!(
                        "host-registered predicate '{}' rejected {} for subject '{}'",
                        predicate.name, context.capability, context.subject
                    ),
                    context,
                ));
            }
        }

        Ok(())
    }
}

/// Policy validation results.
#[derive(Debug, Clone)]
pub struct PolicyValidation {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl SandboxPolicy {
    /// Load, parse, and validate a policy file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|error| {
            vec![Diagnostic::error(
                e5::INVALID_POLICY as u32,
                format!(
                    "failed to read sandbox policy '{}': {}",
                    path.display(),
                    error
                ),
            )]
        })?;

        let mut policy: SandboxPolicy = serde_json::from_str(&source).map_err(|error| {
            vec![Diagnostic::error(
                e5::INVALID_POLICY as u32,
                format!(
                    "failed to parse sandbox policy '{}': {}",
                    path.display(),
                    error
                ),
            )]
        })?;

        policy.base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        policy.serialized_source = Some(source.into_bytes());
        policy.validate().map(|_| policy)
    }

    /// Serialize the policy to deterministic canonical JSON.
    pub fn to_canonical_json(&self) -> Result<String, Diagnostic> {
        serde_json::to_string(self).map_err(|error| {
            Diagnostic::error(
                e5::INVALID_POLICY as u32,
                format!("failed to serialize sandbox policy: {}", error),
            )
        })
    }

    /// Validate the policy against the schema v1 contract and current Phase-1 availability.
    pub fn validate(&self) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        if self.schema_version != 1 {
            diagnostics.push(Diagnostic::error(
                e5::INVALID_POLICY as u32,
                format!(
                    "unsupported sandbox policy schemaVersion {}; expected 1",
                    self.schema_version
                ),
            ));
        }

        validate_positive_u64(
            &mut diagnostics,
            self.resources.max_memory_mb,
            "resources.maxMemoryMB",
        );
        validate_positive_u64(
            &mut diagnostics,
            self.resources.max_cpu_time_ms,
            "resources.maxCpuTimeMs",
        );
        validate_positive_u64(
            &mut diagnostics,
            self.resources.max_open_files,
            "resources.maxOpenFiles",
        );
        validate_zero_capable_u64(
            &mut diagnostics,
            self.resources.max_spawned_processes,
            "resources.maxSpawnedProcesses",
        );
        validate_zero_capable_u64(
            &mut diagnostics,
            self.resources.max_threads,
            "resources.maxThreads",
        );

        validate_positive_u64(
            &mut diagnostics,
            self.network_max_connections(),
            "effects.network.maxConnections",
        );
        validate_positive_u64(
            &mut diagnostics,
            self.effects.timer.max_timeout_ms,
            "effects.timer.maxTimeoutMs",
        );
        validate_positive_u64(
            &mut diagnostics,
            self.effects.timer.max_active_timers,
            "effects.timer.maxActiveTimers",
        );

        // Phase-1 availability checks.
        if self.effects.network.connect.is_enabled() {
            diagnostics.push(unavailable_capability("effects.network.connect"));
        }
        if self.effects.network.listen.is_enabled() {
            diagnostics.push(unavailable_capability("effects.network.listen"));
        }
        if self.effects.process.spawn.is_enabled() {
            diagnostics.push(unavailable_capability("effects.process.spawn"));
        }
        if self.effects.process.env_write.is_enabled() {
            diagnostics.push(unavailable_capability("effects.process.envWrite"));
        }
        if self.effects.eval {
            diagnostics.push(unavailable_capability("effects.eval"));
        }
        if self.resources.max_threads.unwrap_or(0) > 0 {
            diagnostics.push(unavailable_capability("resources.maxThreads"));
        }

        // `console`, `random`, `fetch`, `fileSystem`, `envRead`, and `timer.schedule` are all
        // available in Phase 1; no additional availability gating needed here.

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    /// Resolve the effective thread budget after combining the attached policy cap
    /// with an invocation override.
    ///
    /// The current phase still rejects positive runtime thread requests before execution,
    /// but the resolver keeps the shared zero-cap tightening rule explicit so later
    /// threaded-profile work has a single canonical limit path.
    pub fn effective_thread_budget(&self, invocation_override: Option<u64>) -> Option<u64> {
        match (self.resources.max_threads, invocation_override) {
            (Some(policy_limit), Some(requested_limit)) => Some(policy_limit.min(requested_limit)),
            (Some(policy_limit), None) => Some(policy_limit),
            (None, Some(requested_limit)) => Some(requested_limit),
            (None, None) => None,
        }
    }

    /// Resolve the effective spawned-process budget after combining the attached policy cap
    /// with an invocation override.
    pub fn effective_spawn_budget(&self, invocation_override: Option<u64>) -> Option<u64> {
        match (self.resources.max_spawned_processes, invocation_override) {
            (Some(policy_limit), Some(requested_limit)) => Some(policy_limit.min(requested_limit)),
            (Some(policy_limit), None) => Some(policy_limit),
            (None, Some(requested_limit)) => Some(requested_limit),
            (None, None) => None,
        }
    }

    /// Validate the policy and wrap the result in a `PolicyValidation` helper.
    pub fn validate_policy(&self) -> PolicyValidation {
        match self.validate() {
            Ok(()) => PolicyValidation {
                valid: true,
                diagnostics: Vec::new(),
            },
            Err(diagnostics) => PolicyValidation {
                valid: false,
                diagnostics,
            },
        }
    }

    /// Check a host operation against the current policy.
    pub fn check_operation(&self, op: HostOperation) -> Result<(), Diagnostic> {
        match op {
            HostOperation::Console => {
                if self.effects.console {
                    Ok(())
                } else {
                    Err(sandbox_violation(
                        "Console output is not allowed by the current policy",
                    ))
                }
            }
            HostOperation::Random => {
                if self.effects.random {
                    Ok(())
                } else {
                    Err(sandbox_violation(
                        "Random byte generation is not allowed by the current policy",
                    ))
                }
            }
            HostOperation::FileRead { path } => {
                self.check_path_access(&self.effects.file_system.read, &path, "FileSystem.Read")
            }
            HostOperation::FileWrite { path } => {
                self.check_path_access(&self.effects.file_system.write, &path, "FileSystem.Write")
            }
            HostOperation::NetworkFetch { url } => {
                self.check_url_access(&self.effects.network.fetch, &url, "Network.Fetch")
            }
            HostOperation::NetworkConnect { target } => {
                self.check_url_access(&self.effects.network.connect, &target, "Network.Connect")
            }
            HostOperation::NetworkListen { target } => {
                self.check_url_access(&self.effects.network.listen, &target, "Network.Listen")
            }
            HostOperation::EnvironmentRead { key } => {
                self.check_exact_access(&self.effects.process.env_read, &key, "Process.EnvRead")
            }
            HostOperation::EnvironmentWrite { key } => {
                self.check_exact_access(&self.effects.process.env_write, &key, "Process.EnvWrite")
            }
            HostOperation::TimerSchedule {
                delay_ms,
                active_timers,
            } => {
                if !self.effects.timer.schedule {
                    return Err(sandbox_violation(
                        "Timer creation is not allowed by the current policy",
                    ));
                }
                if let Some(limit) = self.effects.timer.max_timeout_ms {
                    if delay_ms > limit {
                        return Err(resource_limit_violation(format!(
                            "timer delay {}ms exceeds policy limit of {}ms",
                            delay_ms, limit
                        )));
                    }
                }
                if let Some(limit) = self.effects.timer.max_active_timers {
                    if active_timers.saturating_add(1) > limit as usize {
                        return Err(resource_limit_violation(format!(
                            "active timer count {} exceeds policy limit of {}",
                            active_timers.saturating_add(1),
                            limit
                        )));
                    }
                }
                Ok(())
            }
            HostOperation::ProcessSpawn { executable } => {
                self.check_exact_access(&self.effects.process.spawn, &executable, "Process.Spawn")
            }
            HostOperation::ThreadSpawn { active_threads } => match self.resources.max_threads {
                Some(limit) => {
                    if active_threads.saturating_add(1) > limit as usize {
                        Err(resource_limit_violation(format!(
                            "active thread count {} exceeds policy limit of {}",
                            active_threads.saturating_add(1),
                            limit
                        )))
                    } else {
                        Ok(())
                    }
                }
                None => Err(unavailable_capability("resources.maxThreads")),
            },
            HostOperation::ProcessEnvWrite { key } => {
                self.check_exact_access(&self.effects.process.env_write, &key, "Process.EnvWrite")
            }
            HostOperation::Eval => {
                if self.effects.eval {
                    Ok(())
                } else {
                    Err(sandbox_violation(
                        "Eval is not allowed by the current policy",
                    ))
                }
            }
        }
    }

    /// Check a host operation against the current policy and a host-registered predicate registry.
    ///
    /// Declarative policy remains primary: this first applies the declarative allow/deny decision
    /// and then runs any registered narrowing predicates against the canonical operation context.
    /// Predicates may reject additional operations but cannot authorize a declaratively denied one.
    pub fn check_operation_with_predicates(
        &self,
        op: HostOperation,
        predicates: &PolicyPredicateRegistry,
    ) -> Result<(), Diagnostic> {
        let context = PolicyPredicateContext::from_operation(&op);
        self.check_operation(op)?;
        predicates.evaluate(&context)
    }

    /// Return the policy as exact input bytes when available, or canonical JSON otherwise.
    pub fn to_embedded_json_bytes(&self) -> Result<Vec<u8>, Diagnostic> {
        match &self.serialized_source {
            Some(bytes) => Ok(bytes.clone()),
            None => self.to_canonical_json_bytes(),
        }
    }

    /// Return the policy as canonical JSON bytes for artifact embedding.
    pub fn to_canonical_json_bytes(&self) -> Result<Vec<u8>, Diagnostic> {
        self.to_canonical_json().map(|json| json.into_bytes())
    }

    fn check_path_access(
        &self,
        rule: &AccessRule,
        candidate: &Path,
        capability: &str,
    ) -> Result<(), Diagnostic> {
        if rule.allows_path(candidate, &self.base_dir) {
            Ok(())
        } else {
            Err(sandbox_violation(format!(
                "{} is not allowed for path '{}'",
                capability,
                candidate.display()
            )))
        }
    }

    fn check_url_access(
        &self,
        rule: &AccessRule,
        candidate: &str,
        capability: &str,
    ) -> Result<(), Diagnostic> {
        if rule.allows_candidate(candidate, &self.base_dir, PatternKind::Url) {
            Ok(())
        } else {
            Err(sandbox_violation(format!(
                "{} is not allowed for target '{}'",
                capability, candidate
            )))
        }
    }

    fn check_exact_access(
        &self,
        rule: &AccessRule,
        candidate: &str,
        capability: &str,
    ) -> Result<(), Diagnostic> {
        if rule.allows_candidate(candidate, &self.base_dir, PatternKind::Exact) {
            Ok(())
        } else {
            Err(sandbox_violation(format!(
                "{} is not allowed for value '{}'",
                capability, candidate
            )))
        }
    }

    fn network_max_connections(&self) -> Option<u64> {
        self.effects.network.max_connections
    }
}

impl AccessRule {
    pub fn is_enabled(&self) -> bool {
        match self {
            AccessRule::Deny(false) => false,
            AccessRule::Deny(true) => true,
            AccessRule::AllowList(entries) => !entries.is_empty(),
        }
    }

    fn allows_path(&self, candidate: &Path, base_dir: &Path) -> bool {
        self.allows_candidate(&candidate.to_string_lossy(), base_dir, PatternKind::Path)
    }

    pub(crate) fn allows_candidate(
        &self,
        candidate: &str,
        base_dir: &Path,
        kind: PatternKind,
    ) -> bool {
        match self {
            AccessRule::Deny(false) => false,
            AccessRule::Deny(true) => true,
            AccessRule::AllowList(patterns) => {
                if patterns.is_empty() {
                    return false;
                }

                let candidate = normalize_text(candidate);
                patterns.iter().any(|pattern| {
                    let resolved = resolve_pattern(pattern, base_dir, kind);
                    glob_match(&resolved, &candidate)
                })
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum PatternKind {
    Path,
    Url,
    Exact,
}

fn default_schema_version() -> u32 {
    1
}

fn default_base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn validate_positive_u64(diagnostics: &mut Vec<Diagnostic>, value: Option<u64>, name: &str) {
    if let Some(value) = value {
        if value == 0 {
            diagnostics.push(Diagnostic::error(
                e5::INVALID_POLICY as u32,
                format!("{} must be a positive integer when present", name),
            ));
        }
    }
}

fn validate_zero_capable_u64(_diagnostics: &mut Vec<Diagnostic>, _value: Option<u64>, _name: &str) {
    // The policy shape is numeric; any value is syntactically valid here. Phase-gating is
    // handled separately by the availability check below.
}

fn unavailable_capability(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "{} is unavailable in the current phase or availability context",
            name
        ),
    )
}

fn host_predicate_violation(
    message: impl Into<String>,
    context: &PolicyPredicateContext,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message.into())
        .note(format!("capability: {}", context.capability))
        .note(format!("subject: {}", context.subject));

    for (key, value) in &context.details {
        diagnostic = diagnostic.note(format!("detail {}: {}", key, value));
    }

    diagnostic
}

fn sandbox_violation(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message.into())
}

fn resource_limit_violation(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(e4::RESOURCE_LIMIT_EXCEEDED as u32, message.into())
}

fn resolve_pattern(pattern: &str, base_dir: &Path, kind: PatternKind) -> String {
    match kind {
        PatternKind::Exact => normalize_text(pattern),
        PatternKind::Url => normalize_text(pattern),
        PatternKind::Path => {
            let candidate = Path::new(pattern);
            let resolved = if candidate.is_absolute() || pattern.contains("://") {
                candidate.to_path_buf()
            } else {
                base_dir.join(candidate)
            };
            normalize_text(&resolved.to_string_lossy())
        }
    }
}

fn normalize_text(text: &str) -> String {
    text.replace('\\', "/")
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = normalize_text(pattern);
    let text = normalize_text(text);
    glob_match_inner(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_inner(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0usize;
    let mut ti = 0usize;
    let mut star: Option<(usize, usize, bool)> = None;

    while ti < text.len() {
        if pi < pattern.len() {
            match pattern[pi] {
                b'*' => {
                    let is_double = pi + 1 < pattern.len() && pattern[pi + 1] == b'*';
                    let next_pi = if is_double { pi + 2 } else { pi + 1 };
                    star = Some((next_pi, ti, is_double));
                    pi = next_pi;
                    continue;
                }
                ch if ch == text[ti] => {
                    pi += 1;
                    ti += 1;
                    continue;
                }
                _ => {}
            }
        }

        if let Some((next_pi, star_text, is_double)) = star {
            if !is_double && text[star_text] == b'/' {
                return false;
            }
            if star_text < text.len() {
                star = Some((next_pi, star_text + 1, is_double));
                ti = star_text + 1;
                pi = next_pi;
                continue;
            }
        }

        return false;
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        if pi + 1 < pattern.len() && pattern[pi + 1] == b'*' {
            pi += 2;
        } else {
            pi += 1;
        }
    }

    pi == pattern.len()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
