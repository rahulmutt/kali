//! Source file registry and mapping utilities.

use super::{FileId, SourceFile, SourceRegistry};
use std::path::Path;

/// Source map for tracking source file locations.
#[derive(Default)]
pub struct SourceMap {
    registry: SourceRegistry,
}

impl SourceMap {
    /// Create a new SourceMap with a fresh registry.
    pub fn new() -> Self {
        Self {
            registry: SourceRegistry::default(),
        }
    }

    /// Get or create a FileId for a given path.
    pub fn intern_path(&mut self, path: &Path) -> FileId {
        self.registry.intern_path(path)
    }

    /// Get a reference to a source file by ID.
    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.registry.get_file(id)
    }

    /// Format a file reference for diagnostics.
    pub fn format_file_ref(&self, file_id: FileId) -> String {
        self.registry
            .get_file(file_id)
            .map(|f| f.filename().to_string())
            .unwrap_or_else(|| format!("file_{}.ts", file_id.as_u32()))
    }
}

impl From<SourceRegistry> for SourceMap {
    fn from(registry: SourceRegistry) -> Self {
        Self { registry }
    }
}

/// Canonicalize a path to remove relative components.
pub fn canonicalize_path(path: &Path) -> String {
    // For now, just convert to string. Full canonicalization requires
    // filesystem access which complicates testing.
    path.to_string_lossy().to_string()
}

/// Get the filename from a path.
pub fn get_filename(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
}

#[cfg(test)]
#[path = "source_map_tests.rs"]
mod tests;
