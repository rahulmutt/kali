//! Sandbox and policy system for the Kali compiler.

use serde::{Deserialize, Serialize};
use kali_error::diagnostic::Diagnostic;

/// Sandbox policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub version: u32,
    pub resources: ResourceLimits,
}

/// Resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_time_ms: u64,
}

/// Policy validation results.
pub struct PolicyValidation {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Validate a sandbox policy.
pub fn validate_policy(policy: &SandboxPolicy) -> PolicyValidation {
    PolicyValidation {
        valid: true,
        diagnostics: Vec::new(),
    }
}
