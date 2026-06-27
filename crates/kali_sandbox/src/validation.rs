use kali_error::{_error_codes::e5, Diagnostic};

use crate::diagnostics::unavailable_capability;
use crate::SandboxPolicy;

/// Policy validation results.
#[derive(Debug, Clone)]
pub struct PolicyValidation {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl SandboxPolicy {
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
