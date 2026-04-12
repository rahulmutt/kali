//! Embedding interfaces for Kali.

/// Embedding context.
#[derive(Default)]
pub struct EmbeddingCtx;

impl EmbeddingCtx {
    pub fn new() -> Self {
        Self
    }

    /// Build a library artifact.
    pub fn build_library(&self, _source: &str) -> Vec<u8> {
        Vec::new()
    }
}
