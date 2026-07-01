use kali_cli::build::ArtifactMetadata;

/// Compiled standalone artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArtifact {
    pub(crate) wasm_bytes: Vec<u8>,
    pub(crate) metadata: ArtifactMetadata,
}

impl CompiledArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}

/// Compiled library artifact with a WIT sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibArtifact {
    pub(crate) wasm_bytes: Vec<u8>,
    pub(crate) wit: String,
    pub(crate) metadata: ArtifactMetadata,
}

impl LibArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the generated WIT interface description.
    pub fn wit(&self) -> &str {
        &self.wit
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}
