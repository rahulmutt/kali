//! Embedding interfaces for Kali.

/// Embedding context.
pub struct EmbeddingCtx;

impl EmbeddingCtx {
    pub fn new() -> Self {
        Self
    }

    /// Build a library artifact.
    pub fn build_library(&self, _source: &str) -> Result<Vec<u8>, ()> {
        Ok(vec![])
    }
}
