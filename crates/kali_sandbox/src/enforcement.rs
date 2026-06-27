use std::path::Path;

use kali_error::Diagnostic;

use crate::diagnostics::{resource_limit_violation, sandbox_violation, unavailable_capability};
use crate::{AccessRule, HostOperation, PatternKind, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy};

impl SandboxPolicy {
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
