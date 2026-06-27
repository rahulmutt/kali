//! Sandbox and policy system for the Kali compiler.

use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod effects;

pub use effects::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, EffectInference, EffectLocation,
    EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate, PackageEffectsReport,
};

mod policy;

pub use policy::{
    AccessRule, EffectsPolicy, FileSystemPolicy, NetworkPolicy, ProcessPolicy, ResourceLimits,
    SandboxPolicy, TimerPolicy,
};

mod operation;

pub use operation::{HostOperation, PolicyPredicateContext};

use kali_error::{
    _error_codes::e5,
    Diagnostic,
};

mod diagnostics;
mod matching;
mod predicate;

pub(crate) use matching::PatternKind;

pub use predicate::{HostPredicate, PolicyPredicateRegistry};

use crate::diagnostics::{
    resource_limit_violation, sandbox_violation, unavailable_capability,
};

#[cfg(test)]
use kali_error::_error_codes::e4;

/// Policy validation results.
#[derive(Debug, Clone)]
pub struct PolicyValidation {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl SandboxPolicy {
    /// Load, parse, and validate a policy file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        Self::from_file_with_runtime_profiles(path, &[])
    }

    /// Load, parse, and validate a policy file against the provided runtime-profile context.
    pub fn from_file_with_runtime_profiles(
        path: impl AsRef<Path>,
        runtime_profiles: &[String],
    ) -> Result<Self, Vec<Diagnostic>> {
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
        policy
            .validate_with_runtime_profiles(runtime_profiles)
            .map(|_| policy)
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
        self.validate_with_runtime_profiles(&[])
    }

    /// Validate a policy against the supplied runtime-profile context.
    pub fn validate_with_runtime_profiles(
        &self,
        runtime_profiles: &[String],
    ) -> Result<(), Vec<Diagnostic>> {
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
        if self
            .resources
            .max_spawned_processes
            .is_some_and(|count| count > 0)
        {
            diagnostics.push(unavailable_capability("resources.maxSpawnedProcesses"));
        }
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
        let has_threaded_profile = runtime_profiles
            .iter()
            .any(|profile| profile == "wasm-threads");
        if self.resources.max_threads.unwrap_or(0) > 0 && !has_threaded_profile {
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
            HostOperation::ProcessPid { .. } => Err(unavailable_capability("effects.process.pid")),
            HostOperation::ProcessCwd { .. } => Err(unavailable_capability("effects.process.cwd")),
            HostOperation::ProcessChdir { .. } => {
                Err(unavailable_capability("effects.process.chdir"))
            }
            HostOperation::ProcessExit { .. } => {
                Err(unavailable_capability("effects.process.exit"))
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

    pub(crate) fn network_max_connections(&self) -> Option<u64> {
        self.effects.network.max_connections
    }
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
