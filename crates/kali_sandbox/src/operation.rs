use std::{collections::BTreeMap, path::PathBuf};

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
    ProcessPid {
        pid: u32,
    },
    ProcessCwd {
        cwd: PathBuf,
    },
    ProcessChdir {
        path: PathBuf,
    },
    ProcessExit {
        code: i32,
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
                details.insert("path".to_string(), path.display().to_string());
                ("effects.fileSystem.read", path.display().to_string())
            }
            HostOperation::FileWrite { path } => {
                details.insert("path".to_string(), path.display().to_string());
                ("effects.fileSystem.write", path.display().to_string())
            }
            HostOperation::NetworkFetch { url } => {
                details.insert("url".to_string(), url.clone());
                ("effects.network.fetch", url.clone())
            }
            HostOperation::NetworkConnect { target } => {
                details.insert("target".to_string(), target.clone());
                ("effects.network.connect", target.clone())
            }
            HostOperation::NetworkListen { target } => {
                details.insert("target".to_string(), target.clone());
                ("effects.network.listen", target.clone())
            }
            HostOperation::EnvironmentRead { key } => {
                details.insert("key".to_string(), key.clone());
                ("effects.process.envRead", key.clone())
            }
            HostOperation::EnvironmentWrite { key } => {
                details.insert("key".to_string(), key.clone());
                ("effects.process.envWrite", key.clone())
            }
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
            HostOperation::ProcessPid { pid } => {
                details.insert("pid".to_string(), pid.to_string());
                ("effects.process.pid", pid.to_string())
            }
            HostOperation::ProcessCwd { cwd } => {
                let cwd = cwd.display().to_string();
                details.insert("cwd".to_string(), cwd.clone());
                ("effects.process.cwd", cwd)
            }
            HostOperation::ProcessChdir { path } => {
                let path = path.display().to_string();
                details.insert("path".to_string(), path.clone());
                ("effects.process.chdir", path)
            }
            HostOperation::ProcessExit { code } => {
                details.insert("code".to_string(), code.to_string());
                ("effects.process.exit", code.to_string())
            }
            HostOperation::ThreadSpawn { active_threads } => {
                details.insert("activeThreads".to_string(), active_threads.to_string());
                ("resources.maxThreads", active_threads.to_string())
            }
            HostOperation::ProcessEnvWrite { key } => {
                details.insert("key".to_string(), key.clone());
                ("effects.process.envWrite", key.clone())
            }
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
