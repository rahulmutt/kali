//! Runtime execution outcome.

use crate::*;

/// Result of executing a WASM module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOutcome {
    /// Process exit code.
    pub exit_code: i32,
    /// Number of tests executed during `kali test`.
    pub tests_run: usize,
    /// Number of failing tests during `kali test`.
    pub tests_failed: usize,
    /// Captured guest stdout.
    pub stdout: String,
    /// Captured guest raw stdout byte sink (populated only by `Kali.writeStdoutBytes`).
    pub stdout_bytes: Vec<u8>,
    /// Captured guest stderr.
    pub stderr: String,
    /// Coverage hit ordinals recorded during the execution.
    pub coverage_hits: Vec<u32>,
    /// Canonical runtime profiles active for the execution.
    pub runtime_profiles: Vec<String>,
    /// High-level host contract selected for the execution.
    pub host_contract: RuntimeHostContract,
    /// Canonical runtime backend selected for the execution.
    pub runtime_backend: RuntimeBackend,
    /// Deterministic worker/thread shutdown snapshot captured for the execution.
    pub thread_topology: ThreadRuntimeShutdownReport,
    /// Set when execution ended in a wasm trap: the run produced the captured
    /// stdout/stderr up to the trap, `exit_code` is nonzero, and this holds the
    /// diagnostic to render. `None` for clean completion.
    pub trap: Option<Diagnostic>,
}
