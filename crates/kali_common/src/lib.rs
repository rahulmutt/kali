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
pub mod template;

use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub use interner::{InternedString, Interner};
pub use span::Span;

/// Report whether the bytewise shared-memory helpers are lock-free on this target.
///
/// The helper is intentionally tiny and deterministic so browser/runtime compatibility layers
/// can share one capability probe without repeating target-specific atomic checks at each call
/// site.
pub const fn bytewise_shared_memory_is_lock_free() -> bool {
    cfg!(target_has_atomic = "8")
}

/// Global string interner used throughout the compiler.
/// Provides thread-safe string interning for identifiers and literals.
pub static GLOBAL_INTERNER: Lazy<Interner> = Lazy::new(Interner::default);

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
        let path_buf = Self::canonicalize_path(path);

        // Find existing file by path
        for (&fid, file) in &self.files {
            if Path::new(&file.path) == path_buf.as_path() {
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
        let source_file = SourceFile { id, path };
        self.files.insert(id, source_file);
        &self.files[&id]
    }

    /// Canonicalize a path to remove relative components.
    fn canonicalize_path(path: &Path) -> PathBuf {
        // For now, just return the path as-is. Full canonicalization requires
        // filesystem access which complicates testing.
        PathBuf::from(path)
    }
}

/// Unique identifier for a source file.
/// Compact 32-bit ID that is safe to copy and use in Span.
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
)]
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
        Path::new(&self.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get the directory containing this source file.
    pub fn directory(&self) -> &str {
        Path::new(&self.path)
            .parent()
            .and_then(|d| d.to_str())
            .unwrap_or("")
    }

    /// Get the file extension of this source file.
    pub fn extension(&self) -> &str {
        Path::new(&self.path)
            .extension()
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

/// Canonical feature-unavailable wording for the supported async class-method lowering slice.
pub const fn async_class_method_lowering_unavailable_message() -> &'static str {
    "async class method lowering is unavailable in the direct runtime path; use a plain method or the later compatibility path"
}

/// Canonical feature-unavailable wording for the supported generator class-method lowering slice.
pub const fn generator_class_method_lowering_unavailable_message(is_async: bool) -> &'static str {
    if is_async {
        "async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    } else {
        "generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    }
}

/// Canonical feature-unavailable wording for the supported generator-function lowering slice.
pub const fn generator_function_lowering_unavailable_message(is_async: bool) -> &'static str {
    if is_async {
        "async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    } else {
        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    }
}

/// Canonical wrapped-zero aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_wrapped_zero_aliases() -> &'static [&'static str] {
    &[
        r#"process.kill((0))"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process.kill((0))"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze(process.kill)(+0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0)"#,
        r#"Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"((process.kill))(0)"#,
        r#"((process.kill))(+0)"#,
    ]
}

/// Canonical feature-unavailable wording for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_unavailable_message() -> &'static str {
    "process.kill is unavailable unless it is invoked as process.kill(0) or one of its supported Node zero-probe aliases: process[\"kill\"](0), process[\"kill\"](+0), process[\"kill\"]((0)), globalThis.process.kill(0), globalThis.process.kill(+0), globalThis.process.kill((0)), globalThis.process[\"kill\"](0), globalThis.process[\"kill\"](+0), globalThis.process[\"kill\"]((0)), globalThis[\"process\"].kill(0), globalThis[\"process\"].kill(+0), globalThis[\"process\"].kill((0)), globalThis[\"process\"][\"kill\"](0), globalThis[\"process\"][\"kill\"](+0), globalThis[\"process\"][\"kill\"]((0)), Object.freeze(process.kill)(0), Object.freeze(process.kill)(+0), Object.freeze((process.kill))(0), Object.freeze((process.kill))(+0), Object.freeze(globalThis.process.kill)(0), Object.freeze(globalThis.process.kill)(+0), Object.freeze((globalThis.process.kill))(0), Object.freeze((globalThis.process.kill))(+0), Object.freeze(globalThis.process[\"kill\"])(0), Object.freeze(globalThis.process[\"kill\"])(+0), Object.freeze((globalThis.process[\"kill\"]))(+0), Object.freeze((globalThis.process[\"kill\"]))(0), Object.freeze(globalThis[\"process\"].kill)(0), Object.freeze(globalThis[\"process\"].kill)(+0), Object.freeze(globalThis[\"process\"][\"kill\"])(0), Object.freeze(globalThis[\"process\"][\"kill\"])(+0), Object.freeze((globalThis[\"process\"][\"kill\"]))(0), Object.freeze((globalThis[\"process\"][\"kill\"]))(+0), Object.freeze(process)[\"kill\"](0), Object.freeze(process)[\"kill\"](+0), Object.freeze(globalThis.process)[\"kill\"](0), Object.freeze(globalThis.process)[\"kill\"](+0), Object.freeze(globalThis[\"process\"])[\"kill\"](0), Object.freeze(globalThis[\"process\"])[\"kill\"](+0), Object.freeze(globalThis.process[\"kill\"])(0), Object.freeze(globalThis.process[\"kill\"])(+0), Object.freeze(globalThis[\"process\"].kill)(0), Object.freeze(globalThis[\"process\"].kill)(+0), Object.freeze((globalThis[\"process\"].kill))(0), Object.freeze((globalThis[\"process\"].kill))(+0), ((process[\"kill\"]))(0), ((process[\"kill\"]))(+0), ((globalThis.process[\"kill\"]))(0), ((globalThis.process[\"kill\"]))(+0), ((globalThis.process.kill))(0), ((globalThis.process.kill))(+0), ((process.kill))(0), ((process.kill))(+0), ((globalThis[\"process\"][\"kill\"]))(0), ((globalThis[\"process\"][\"kill\"]))(+0), ((globalThis[\"process\"].kill))(0), ((globalThis[\"process\"].kill))(+0); use the zero liveness-probe subset or the later compatibility path"
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
