use std::{
    fs,
    path::{Path, PathBuf},
};

use kali_error::{_error_codes::e5, Diagnostic};
use serde_json;

use crate::SandboxPolicy;

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
}
