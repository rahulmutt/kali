//! Deterministic permission model for the Deno compatibility layer.

/// Canonical Deno permission descriptor subset used by the Phase-1 compatibility facade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenoPermissionKind {
    Read,
    Write,
    Net,
    Env,
}

/// Permission query result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenoPermissionStatus {
    Granted,
    Denied,
}

/// Errors produced by the compatibility permission facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoPermissionError {
    message: String,
}

impl DenoPermissionError {
    fn unavailable(member: &str) -> Self {
        Self {
            message: format!(
                "{} is not available in the Phase-1 Deno permission facade",
                member
            ),
        }
    }
}

impl std::fmt::Display for DenoPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DenoPermissionError {}

/// Query-only permission view for the Deno compatibility layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoPermissions {
    read: bool,
    write: bool,
    net: bool,
    env: bool,
}

impl DenoPermissions {
    /// Create a permission view from explicit capability flags.
    pub fn new(read: bool, write: bool, net: bool, env: bool) -> Self {
        Self {
            read,
            write,
            net,
            env,
        }
    }

    /// Create the default open view used by standalone execution in Phase 1.
    pub fn open() -> Self {
        Self::new(true, true, true, true)
    }

    /// Query the current permission state for one descriptor kind.
    pub fn query(
        &self,
        kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Ok(match kind {
            DenoPermissionKind::Read if self.read => DenoPermissionStatus::Granted,
            DenoPermissionKind::Write if self.write => DenoPermissionStatus::Granted,
            DenoPermissionKind::Net if self.net => DenoPermissionStatus::Granted,
            DenoPermissionKind::Env if self.env => DenoPermissionStatus::Granted,
            DenoPermissionKind::Read
            | DenoPermissionKind::Write
            | DenoPermissionKind::Net
            | DenoPermissionKind::Env => DenoPermissionStatus::Denied,
        })
    }

    /// Recognized-but-unavailable compatibility member.
    pub fn request(
        &self,
        _kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Err(DenoPermissionError::unavailable(
            "Deno.permissions.request()",
        ))
    }

    /// Recognized-but-unavailable compatibility member.
    pub fn revoke(
        &self,
        _kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Err(DenoPermissionError::unavailable(
            "Deno.permissions.revoke()",
        ))
    }
}

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod permissions_tests;
