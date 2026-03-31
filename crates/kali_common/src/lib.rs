//! Common utilities shared across all Kali crates.
//! 
//! This crate provides:
//! - String interning for identifiers and literals
//! - Source file registry with compact FileId
//! - Span type for source positions
//! - SourceMap for human-readable diagnostics

pub mod interner;
pub mod source_map;
pub mod span;

use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub use interner::{InternedString, Interner};
pub use span::Span;

/// Global string interner used throughout the compiler.
/// Provides thread-safe string interning for identifiers and literals.
pub static GLOBAl_INTERNER: Lazy<Interner> = Lazy::new(Interner::default);

/// Global source file registry.
/// Assigns compact FileId to each loaded source file.
pub static SOURCE_REGISTRY: Lazy<Mutex<SourceRegistry>> =
    Lazy::new(|| Mutex::new(SourceRegistry::default()));

/// Registry of source files in memory.
#[derive(Default)]
pub struct SourceRegistry {
    files: AHashMap<FileId, SourceFile>,
    next_file_id: FileId,
}

impl SourceRegistry {
    /// Get or create a FileId for a given path.
    pub fn intern_path(&mut self, path: &Path) -> FileId {
        let path_buf = canonicalize_path(path);
        
        // Find existing file by path
        for (&fid, file) in &self.files {
            if PathBuf::from(&file.path) == path_buf {
                return fid;
            }
        }
        
        // Create new file
        let fid = self.next_file_id;
        self.next_file_id.0 += 1;
        
        let source_file = SourceFile {
            id: fid,
            path: path_buf.to_string_lossy().to_string(),
        };
        
        self.files.insert(fid, source_file);
        fid
    }

    /// Get a reference to a source file by ID.
    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(&id)
    }

    /// Create a new source file with given ID (for testing/benchmarks).
    pub fn create_file(&mut self, id: FileId) -> &SourceFile {
        let path = format!("file://unknown_{}.ts", id.0);
        let source_file = SourceFile {
            id,
            path,
        };
        self.files.insert(id, source_file);
        &self.files[&id]
    }
}

/// Unique identifier for a source file.
/// Compact 32-bit ID that is safe to copy and use in Span.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FileId(u32);

impl FileId {
    /// Create a new FileId from a u32 value.
    pub fn new(id: u32) -> Self {
        FileId(id)
    }

    /// Get the numeric value of this FileId.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.0)
    }
}

/// A single source file in the compilation unit.
#[derive(Clone, Debug)]
pub struct SourceFile {
    /// Unique identifier for this file.
    pub id: FileId,
    /// Filesystem path or virtual URL.
    pub path: String,
}

impl SourceFile {
    /// Create a new SourceFile with given ID and path.
    pub fn new(id: FileId, path: impl Into<String>) -> Self {
        SourceFile {
            id,
            path: path.into(),
        }
    }

    /// Get the filename of this source file.
    pub fn filename(&self) -> &str {
        Path::new(&self.path).file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get the directory containing this source file.
    pub fn directory(&self) -> &str {
        Path::new(&self.path).parent()
            .map(|d| d.to_string_lossy())
            .unwrap_or("")
    }

    /// Get the file extension of this source file.
    pub fn extension(&self) -> &str {
        Path::new(&self.path).extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    }
}

impl std::fmt::Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// SourceMap provides lookup from source positions to human-readable locations.
pub struct SourceMap {
    registry: SourceRegistry,
}

impl SourceMap {
    /// Create a new SourceMap with a fresh registry.
    pub fn new() -> Self {
        SourceMap {
            registry: SourceRegistry::default(),
        }
    }

    /// Intern a path and return the FileId.
    pub fn intern_path(&mut self, path: &Path) -> FileId {
        self.registry.intern_path(path)
    }

    /// Create a source file with a given FileId.
    pub fn create_file(&mut self, id: FileId) -> &SourceFile {
        self.registry.create_file(id)
    }

    /// Get a reference to a source file by FileId.
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

    /// Format a span as a human-readable location.
    pub fn format_location(&self, span: Span) -> String {
        let file = self
            .get_file(span.file_id)
            .unwrap_or_else(|| self.registry.create_file(span.file_id));

        format!("{}:{}:{}", file.filename(), span.line(), span.column())
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a file location for use in diagnostics.
pub fn format_file_ref(source_map: &SourceMap, file_id: FileId) -> String {
    source_map.format_file_ref(file_id)
}

/// Format a location from a span and source map.
pub fn format_location(source_map: &SourceMap, span: Span) -> String {
    source_map.format_location(span)
}

/// Canonicalize a path to remove relative components.
fn canonicalize_path(path: &Path) -> String {
    // For now, just convert to string. Full canonicalization requires
    // filesystem access which complicates testing.
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id_basic() {
        let fid = FileId::new(42);
        assert_eq!(fid.as_u32(), 42);
        assert_eq!(fid.to_string(), "f42");
    }

    #[test]
    fn test_source_file() {
        let sf = SourceFile::new(FileId::new(0), "/path/to/file.ts");
        assert_eq!(sf.filename(), "file.ts");
        assert_eq!(sf.extension(), "ts");
        assert_eq!(sf.directory(), "/path/to");
    }

    #[test]
    fn test_source_registry_interning() {
        let mut registry = SourceRegistry::default();
        
        let path = Path::new("/test/file.ts");
        let fid1 = registry.intern_path(path);
        let fid2 = registry.intern_path(path);
        
        // Same path should give same ID
        assert_eq!(fid1, fid2);
        
        // Different paths should give different IDs
        let fid3 = registry.intern_path(Path::new("/test/other.ts"));
        assert_ne!(fid1, fid3);
    }

    #[test]
    fn test_source_map() {
        let mut sm = SourceMap::new();
        let fid = sm.intern_path(Path::new("/test/file.ts"));
        
        assert_eq!(sm.format_file_ref(fid), "file.ts");
        assert_eq!(sm.format_file_ref(fid).contains("file.ts"), true);
    }
}
