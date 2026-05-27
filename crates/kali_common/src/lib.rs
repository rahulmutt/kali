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

/// Canonical feature-unavailable wording for generator-class-method yield-delegation slices.
pub const fn generator_class_method_yield_lowering_unavailable_message(
    is_async: bool,
    is_delegate: bool,
) -> &'static str {
    match (is_async, is_delegate) {
        (true, true) => {
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (true, false) => generator_class_method_lowering_unavailable_message(true),
        (false, true) => {
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (false, false) => generator_class_method_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator class-method lowering slices.
pub const fn generator_class_method_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
) -> &'static str {
    match (has_generator, has_async_generator) {
        (true, true) => "generator and async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path",
        (true, false) => generator_class_method_lowering_unavailable_message(false),
        (false, true) => generator_class_method_lowering_unavailable_message(true),
        (false, false) => generator_class_method_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator class-method yield-delegation slices.
pub const fn generator_class_method_yield_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
    is_delegate: bool,
) -> &'static str {
    match (has_generator, has_async_generator, is_delegate) {
        (true, true, true) => {
            "generator and async-generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (true, true, false) => {
            generator_class_method_lowering_unavailable_message_for_flavors(true, true)
        }
        (true, false, true) => generator_class_method_yield_lowering_unavailable_message(false, true),
        (true, false, false) => generator_class_method_lowering_unavailable_message(false),
        (false, true, true) => generator_class_method_yield_lowering_unavailable_message(true, true),
        (false, true, false) => generator_class_method_lowering_unavailable_message(true),
        (false, false, true) => generator_class_method_lowering_unavailable_message(false),
        (false, false, false) => generator_class_method_lowering_unavailable_message(false),
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

/// Canonical feature-unavailable wording for yield-delegation slices.
pub const fn generator_function_yield_lowering_unavailable_message(
    is_async: bool,
    is_delegate: bool,
) -> &'static str {
    match (is_async, is_delegate) {
        (true, true) => "async-generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path",
        (true, false) => generator_function_lowering_unavailable_message(true),
        (false, true) => "generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path",
        (false, false) => generator_function_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator function lowering slices.
pub const fn generator_function_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
) -> &'static str {
    match (has_generator, has_async_generator) {
        (true, true) => "generator and async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path",
        (true, false) => generator_function_lowering_unavailable_message(false),
        (false, true) => generator_function_lowering_unavailable_message(true),
        (false, false) => generator_function_lowering_unavailable_message(false),
    }
}

/// Canonical direct aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_direct_zero_aliases() -> &'static [&'static str] {
    &[
        r#"process.kill"#,
        r#"process["kill"]"#,
        r#"globalThis.process.kill"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process.kill(0)"#,
        r#"process.kill(+0)"#,
        r#"process["kill"](0)"#,
        r#"process["kill"](+0)"#,
        r#"globalThis.process.kill(0)"#,
        r#"globalThis.process.kill(+0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
    ]
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
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze((process).kill)(0)"#,
        r#"Object.freeze((process).kill)(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0)"#,
        r#"Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process).kill)(0)"#,
        r#"Object.freeze((globalThis.process).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        r#"((process.kill))(0)"#,
        r#"((process.kill))(+0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis.process.kill))(+0)"#,
        r#"((process["kill"]))(0)"#,
        r#"((process["kill"]))(+0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(+0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"].kill))(+0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze((process))["kill"](0)"#,
        r#"Object.freeze((process))["kill"](+0)"#,
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((globalThis.process))["kill"](0)"#,
        r#"Object.freeze((globalThis.process))["kill"](+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
    ]
}

/// Canonical receiver-freeze dot aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_freeze_aliases(
) -> &'static [&'static str] {
    &[
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` receiver-freeze dot aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_source() -> String {
    join_semicolon_terminated_segments(
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases(),
    )
}

/// Canonical transparent parenthesized receiver aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_aliases() -> &'static [&'static str] {
    &[
        r#"((process)).kill(0)"#,
        r#"((process)).kill(+0)"#,
        r#"((globalThis.process)).kill(0)"#,
        r#"((globalThis.process)).kill(+0)"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` transparent parenthesized receiver aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_source() -> String {
    join_semicolon_terminated_segments(process_kill_zero_probe_parenthesized_receiver_aliases())
}

/// Canonical parenthesized frozen-callable aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_frozen_callable_aliases(
) -> &'static [&'static str] {
    &[
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized frozen-callable aliases.
pub fn process_kill_zero_probe_parenthesized_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(
        process_kill_zero_probe_parenthesized_frozen_callable_aliases(),
    )
}

/// Canonical parenthesized receiver-freeze bracket aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases(
) -> &'static [&'static str] {
    &[
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
    ]
}

/// Canonical alias inventory for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases(
) -> Vec<&'static str> {
    ordered_unique_union(&[process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases()])
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source() -> String {
    join_semicolon_terminated_segments(
        &process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases(),
    )
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source() -> String {
    process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source()
}

/// Canonical alias inventory for the supported Node `process.kill(0)` parenthesized receiver-freeze slice.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases() -> Vec<&'static str>
{
    let bracket_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases();
    ordered_unique_union(&[
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases(),
        bracket_aliases.as_slice(),
    ])
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source() -> String {
    join_semicolon_terminated_segments(
        &process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases(),
    )
}

fn join_semicolon_terminated_segments(segments: &[&str]) -> String {
    let mut source = segments.join("; ");
    source.push(';');
    source
}

fn join_zero_probe_aliases(aliases: &[&'static str]) -> String {
    join_semicolon_terminated_segments(aliases)
}

fn join_const_binding_lines(bindings: &[(&'static str, &'static str)]) -> String {
    let lines = bindings
        .iter()
        .map(|(name, alias)| format!("const {name} = {alias}"))
        .collect::<Vec<_>>();
    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    join_semicolon_terminated_segments(&line_refs)
}

fn ordered_unique_union(slices: &[&[&'static str]]) -> Vec<&'static str> {
    let total_len = slices.iter().map(|slice| slice.len()).sum();
    let mut aliases = Vec::with_capacity(total_len);
    let mut seen = std::collections::HashSet::with_capacity(total_len);

    for alias in slices.iter().flat_map(|slice| slice.iter().copied()) {
        if seen.insert(alias) {
            aliases.push(alias);
        }
    }

    aliases
}

/// Canonical direct zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_direct_source() -> String {
    join_zero_probe_aliases(process_kill_zero_probe_direct_zero_aliases())
}

/// Canonical wrapped zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_wrapped_source() -> String {
    join_zero_probe_aliases(process_kill_zero_probe_wrapped_zero_aliases())
}

/// Canonical full alias inventory for the supported Node `process.kill(0)` zero-probe slice.
pub fn process_kill_zero_probe_aliases() -> Vec<&'static str> {
    ordered_unique_union(&[
        process_kill_zero_probe_direct_zero_aliases(),
        process_kill_zero_probe_wrapped_zero_aliases(),
    ])
}

/// Canonical call-target aliases for TS-wrapped supported Node `process.kill(0)` slices.
pub const fn process_kill_zero_probe_call_target_aliases() -> &'static [&'static str] {
    &[
        r#"process.kill"#,
        r#"globalThis.process.kill"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` TS-wrapped call-target aliases.
pub fn process_kill_zero_probe_call_target_inventory_source() -> String {
    join_semicolon_terminated_segments(process_kill_zero_probe_call_target_aliases())
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe call-target inventory wrapped in a typed expression.
pub fn process_kill_zero_probe_wrapped_call_target_source(argument_source: &str) -> String {
    let mut source = process_kill_zero_probe_call_target_aliases()
        .iter()
        .map(|alias| format!("{alias}{argument_source}"))
        .collect::<Vec<_>>()
        .join("; ");
    source.push(';');
    source
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe slices wrapped in a TS `satisfies` expression.
pub fn process_kill_zero_probe_satisfies_source() -> String {
    process_kill_zero_probe_wrapped_call_target_source("((0 satisfies number))")
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe slices wrapped in a TS type assertion.
pub fn process_kill_zero_probe_type_assertion_source() -> String {
    process_kill_zero_probe_wrapped_call_target_source("((0 as number))")
}

/// Canonical source text for the full supported Node `process.kill(0)` alias inventory.
///
/// This source composes the dedicated direct and wrapped zero-probe source helpers so the
/// inventory stays single-sourced.
pub fn process_kill_zero_probe_alias_inventory_source() -> String {
    format!(
        "{} {}",
        process_kill_zero_probe_direct_source().trim_end(),
        process_kill_zero_probe_wrapped_source().trim_end()
    )
}

/// Canonical zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_source() -> String {
    process_kill_zero_probe_alias_inventory_source()
}

/// Canonical binding inventory for the supported Node zero-probe sequence-callable-target bindings.
pub const fn process_kill_zero_probe_sequence_call_target_binding_lines(
) -> &'static [(&'static str, &'static str)] {
    &[
        ("sequenceKill", "(process.kill, process.kill)"),
        (
            "bracketedRootSequenceKill",
            "(process[\"kill\"], process[\"kill\"])",
        ),
        (
            "dotRootSequenceKill",
            "(globalThis.process.kill, globalThis.process.kill)",
        ),
        (
            "bracketedSequenceKill",
            "(globalThis[\"process\"][\"kill\"], globalThis[\"process\"][\"kill\"])",
        ),
        (
            "dotBracketSequenceKill",
            "(globalThis.process[\"kill\"], globalThis.process[\"kill\"])",
        ),
        (
            "bracketedDotSequenceKill",
            "(globalThis[\"process\"].kill, globalThis[\"process\"].kill)",
        ),
    ]
}

/// Canonical source text for the supported Node zero-probe sequence-callable-target bindings.
pub fn process_kill_zero_probe_sequence_call_target_bindings_source() -> String {
    join_const_binding_lines(process_kill_zero_probe_sequence_call_target_binding_lines())
}

/// Canonical binding inventory for the supported Node `process.kill(0)` direct call-target bindings.
pub const fn process_kill_zero_probe_call_target_binding_lines(
) -> &'static [(&'static str, &'static str)] {
    &[
        ("kill", "process.kill"),
        ("bracketedRootKill", "process[\"kill\"]"),
        ("dotRootKill", "globalThis.process.kill"),
        ("bracketedDotKill", "globalThis[\"process\"].kill"),
        ("dotBracketKill", "globalThis.process[\"kill\"]"),
        ("fullyBracketedKill", "globalThis[\"process\"][\"kill\"]"),
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` direct call-target bindings.
pub fn process_kill_zero_probe_call_target_bindings_source() -> String {
    join_const_binding_lines(process_kill_zero_probe_call_target_binding_lines())
}

/// Canonical `console.log(...)` source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_console_log_source() -> String {
    let statements = process_kill_zero_probe_aliases()
        .iter()
        .map(|alias| format!("console.log({alias})"))
        .collect::<Vec<_>>();
    format!("{};", statements.join("; "))
}

/// Canonical source text for the supported Node `process.kill(0)` node-API-surface
/// alias matrix used by the documented Node runtime regression.
pub fn process_kill_zero_probe_node_api_surface_run_source() -> String {
    format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} console.log(process.kill(zeroAlias)); console.log(dotRootKill(+zero)); console.log(globalThis[\"process\"][\"kill\"](zero)); console.log(process[\"kill\"](zero)); console.log(kill(0)); console.log(bracketedDotKill(+0)); console.log(globalThis[\"process\"].kill(+0)); console.log(dotBracketKill(0)); console.log(fullyBracketedKill(0)); console.log(sequenceKill(0)); console.log(bracketedRootSequenceKill(0)); console.log(dotRootSequenceKill(0)); console.log(bracketedSequenceKill(0)); console.log(dotBracketSequenceKill(0)); console.log(bracketedDotSequenceKill(0)); console.log(globalThis[\"process\"][\"kill\"](+0)); console.log(((globalThis[\"process\"][\"kill\"]))(+0));\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
    )
}

/// Canonical source text for the supported Node `process.kill(0)` node-API-surface
/// alias matrix used by the documented Node runtime regression.
pub fn process_kill_zero_probe_node_api_surface_test_source() -> String {
    format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} globalThis[\"process\"].kill(+0); globalThis[\"process\"][\"kill\"](+0); Kali.test('process kill alias', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
        process_kill_zero_probe_guard_source(),
    )
}

/// Canonical rejection-guard source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_guard_source() -> String {
    process_kill_zero_probe_aliases()
        .iter()
        .map(|alias| format!("!{alias}"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical frozen callable aliases for the supported `Object.hasOwn` helper slice.
pub const fn object_has_own_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Object.hasOwn)"#,
        r#"Object.freeze((Object.hasOwn))"#,
        r#"Object.freeze(Object["hasOwn"])"#,
        r#"Object.freeze((Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis.Object.hasOwn)"#,
        r#"Object.freeze((globalThis.Object.hasOwn))"#,
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object)["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis?.Object.hasOwn)"#,
        r#"Object.freeze((globalThis?.Object.hasOwn))"#,
        r#"Object.freeze((globalThis?.Object).hasOwn)"#,
        r#"Object.freeze((globalThis?.Object)["hasOwn"])"#,
        r#"Object.freeze(globalThis?.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis?.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwn)"#,
        r#"Object.freeze((globalThis["Object"].hasOwn))"#,
        r#"Object.freeze((globalThis["Object"]).hasOwn)"#,
        r#"Object.freeze((globalThis["Object"])["hasOwn"])"#,
        r#"Object.freeze(globalThis["Object"]["hasOwn"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwn"]))"#,
        r#"Object.freeze(globalThis['Object'].hasOwn)"#,
        r#"Object.freeze((globalThis['Object'].hasOwn))"#,
        r#"Object.freeze((globalThis['Object']).hasOwn)"#,
        r#"Object.freeze((globalThis['Object'])['hasOwn'])"#,
        r#"Object.freeze(globalThis['Object']['hasOwn'])"#,
        r#"Object.freeze((globalThis['Object']['hasOwn']))"#,
        r#"Object.freeze((null ?? Object.hasOwn))"#,
        r#"Object.freeze((true && Object.hasOwn))"#,
        r#"Object.freeze((false || Object.hasOwn))"#,
    ]
}

/// Canonical source text for the supported `Object.hasOwn` frozen callable aliases.
pub fn object_has_own_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_has_own_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Reflect.ownKeys` helper slice.
pub const fn reflect_own_keys_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Reflect.ownKeys)"#,
        r#"Object.freeze((Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect.ownKeys)"#,
        r#"Object.freeze((null ?? globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((true && globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((false || globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect["ownKeys"])"#,
        r#"Object.freeze(globalThis.Reflect['ownKeys'])"#,
        r#"Object.freeze((globalThis.Reflect['ownKeys']))"#,
        r#"Object.freeze(globalThis["Reflect"].ownKeys)"#,
        r#"Object.freeze(globalThis["Reflect"]['ownKeys'])"#,
        r#"Object.freeze(globalThis['Reflect']["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"]['ownKeys']))"#,
        r#"Object.freeze((globalThis['Reflect']["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect)["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"]).ownKeys)"#,
        r#"Object.freeze((globalThis['Reflect']).ownKeys)"#,
        r#"Object.freeze((globalThis["Reflect"])["ownKeys"])"#,
        r#"Object.freeze(globalThis["Reflect"]["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"].ownKeys))"#,
        r#"Object.freeze((globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((globalThis['Reflect'].ownKeys))"#,
        r#"Object.freeze((globalThis['Reflect']['ownKeys']))"#,
        r#"Object.freeze((globalThis['Reflect'])['ownKeys'])"#,
        r#"Object.freeze(globalThis['Reflect'].ownKeys)"#,
        r#"Object.freeze(globalThis['Reflect']['ownKeys'])"#,
        r#"Object.freeze((null ?? Reflect.ownKeys))"#,
        r#"Object.freeze((true && Reflect.ownKeys))"#,
        r#"Object.freeze((false || Reflect.ownKeys))"#,
        r#"Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))"#,
        r#"Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((true && globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((false || globalThis["Reflect"]["ownKeys"]))"#,
    ]
}

/// Canonical source text for the supported `Reflect.ownKeys` frozen callable aliases.
pub fn reflect_own_keys_frozen_callable_source(object_source: &str) -> String {
    let statements = [
        format!("const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)({object_source})"),
        format!("const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))({object_source})"),
        format!("const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)({object_source})"),
        format!(r#"const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"]['ownKeys']))({object_source})"#),
        format!(r#"const mixedSingleQuotedRootKeys = Object.freeze(globalThis['Reflect']["ownKeys"])({object_source})"#),
        format!(r#"const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']["ownKeys"]))({object_source})"#),
        format!(r#"const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])({object_source})"#),
        format!(r#"const frozenSingleQuotedMixedBracketedKeys = Object.freeze(globalThis.Reflect['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedMixedBracketedKeys = Object.freeze((globalThis.Reflect['ownKeys']))({object_source})"#),
        format!(r#"const nullishFrozenCallableKeys = Object.freeze((null ?? globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const logicalAndFrozenCallableKeys = Object.freeze((true && globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const logicalOrFrozenCallableKeys = Object.freeze((false || globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)({object_source})"#),
        format!(r#"const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])({object_source})"#),
        format!(r#"const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedBracketRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])({object_source})"#),
        format!(r#"const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))({object_source})"#),
        format!(r#"const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])({object_source})"#),
        format!(r#"const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))({object_source})"#),
        format!(r#"const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!("const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))({object_source})"),
        format!(r#"const frozenSingleQuotedRootKeys = Object.freeze(globalThis['Reflect'].ownKeys)({object_source})"#),
        format!(r#"const nullishFrozenBracketedKeys = Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const logicalAndFrozenBracketedKeys = Object.freeze((true && globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const logicalOrFrozenBracketedKeys = Object.freeze((false || globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])({object_source})"#),
        format!(r#"const frozenSingleQuotedBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedRootKeys = Object.freeze((globalThis['Reflect'].ownKeys))({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))({object_source})"#),
        format!("const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))({object_source})"),
        format!("const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))({object_source})"),
        format!("const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))({object_source})"),
        format!("const conditionalFrozenCallableKeys = Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))({object_source})"),
        format!("const conditionalFrozenGlobalCallableKeys = Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))({object_source})"),
    ];
    join_semicolon_terminated_segments(&statements.iter().map(String::as_str).collect::<Vec<_>>())
}

/// Canonical frozen callable aliases for the supported `Object.keys` / `Object.values` / `Object.entries` helper slice.
pub const fn object_enumeration_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis["Object"]).keys)"#,
        r#"Object.freeze((globalThis["Object"]).values)"#,
        r#"Object.freeze((globalThis["Object"]).entries)"#,
        r#"Object.freeze((globalThis['Object']).keys)"#,
        r#"Object.freeze((globalThis['Object']).values)"#,
        r#"Object.freeze((globalThis['Object']).entries)"#,
    ]
}

/// Canonical source text for the supported `Object.keys` / `Object.values` / `Object.entries` helper slice.
pub fn object_enumeration_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_enumeration_frozen_callable_aliases())
}

/// Canonical boolean-check source for the supported `Object.hasOwn` frozen callable aliases.
pub fn object_has_own_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    object_has_own_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("!{alias}({receiver_source}, {key_source})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical frozen callable aliases for the supported `Object.prototype.hasOwnProperty.call` helper slice.
pub const fn object_has_own_property_call_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis['Object']).prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty']['call'])"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty']['call']))"#,
        r#"Object.freeze((globalThis['Object']).prototype['hasOwnProperty']['call'])"#,
        r#"Object.freeze((globalThis['Object'])['prototype']['hasOwnProperty']['call'])"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty'].call))"#,
        r#"Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((Object.prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze((null ?? Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty["call"]))"#,
    ]
}

/// Canonical source text for the supported `Object.prototype.hasOwnProperty.call` frozen callable aliases.
pub fn object_has_own_property_call_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_has_own_property_call_frozen_callable_aliases())
}

/// Canonical boolean-check source for the supported `Object.prototype.hasOwnProperty.call` frozen callable aliases.
pub fn object_has_own_property_call_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    object_has_own_property_call_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("!{alias}({receiver_source}, {key_source})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical combined boolean-check source for the supported `Object.hasOwn` helper slice.
pub fn object_has_own_combined_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    format!(
        "{} || {}",
        object_has_own_frozen_callable_condition_source(receiver_source, key_source),
        object_has_own_property_call_frozen_callable_condition_source(receiver_source, key_source)
    )
}

/// Canonical source text for the supported late-compat `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice.
pub fn late_compat_object_has_own_source(receiver_source: &str, key_source: &str) -> String {
    let source = [
        format!("Object.hasOwn({receiver_source}, {key_source})"),
        format!("globalThis.Object.hasOwn({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwn({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['prototype']['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!("Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!("globalThis.Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
    ]
    .join("; ");
    format!("{source};")
}

/// Canonical source text for the supported Number predicate slice.
pub fn number_predicates_preamble_source(alias_literal: &str) -> String {
    format!(
        "const alias = {alias_literal}; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number[\"isFinite\"]); const frozenBracketedNaN = Object.freeze(Number[\"isNaN\"]); const frozenBracketedInteger = Object.freeze(Number[\"isInteger\"]); const frozenBracketedSafeInteger = Object.freeze(Number[\"isSafeInteger\"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis[\"Number\"])[\"isFinite\"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis[\"Number\"])[\"isNaN\"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis[\"Number\"])[\"isInteger\"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis[\"Number\"])[\"isSafeInteger\"]);"
    )
}

/// Canonical console-log body for the supported Number predicate slice.
pub fn number_predicates_console_log_body_source() -> String {
    join_semicolon_terminated_segments(&[
        r#"console.log(Number.isFinite(alias))"#,
        r#"console.log(integer(alias))"#,
        r#"console.log(Number.isSafeInteger(alias))"#,
        r#"console.log(integer(1.5))"#,
        r#"console.log(Number.isFinite("hello"))"#,
        r#"console.log(Number.isSafeInteger(1.5))"#,
        r#"console.log(globalThis["Number"]["isNaN"](NaN))"#,
        r#"console.log(globalThis.Number.isNaN(1))"#,
        r#"console.log(globalThis["Number"].isNaN(1))"#,
        r#"console.log(globalThis["Number"]["isFinite"](alias))"#,
        r#"console.log(globalThis["Number"]["isInteger"](alias))"#,
        r#"console.log(globalThis["Number"]["isSafeInteger"](alias))"#,
        r#"console.log(globalThis.Number["isNaN"](1))"#,
        r#"console.log(globalThis["Number"].isFinite(alias))"#,
        r#"console.log(globalThis.Number["isInteger"](alias))"#,
        r#"console.log(globalThis["Number"].isSafeInteger(alias))"#,
        r#"console.log(Number["isFinite"](alias))"#,
        r#"console.log(Number["isInteger"](alias))"#,
        r#"console.log(Number["isSafeInteger"](alias))"#,
        r#"console.log(Number["isNaN"](1))"#,
        r#"console.log(frozenFinite(alias))"#,
        r#"console.log(frozenNaN(NaN))"#,
        r#"console.log(frozenNaN(1))"#,
        r#"console.log(frozenInteger(alias))"#,
        r#"console.log(frozenSafeInteger(alias))"#,
        r#"console.log(frozenBracketedFinite(alias))"#,
        r#"console.log(frozenBracketedNaN(NaN))"#,
        r#"console.log(frozenBracketedNaN(1))"#,
        r#"console.log(frozenBracketedInteger(alias))"#,
        r#"console.log(frozenBracketedSafeInteger(alias))"#,
        r#"console.log(frozenParenthesizedBracketedFinite(alias))"#,
        r#"console.log(frozenParenthesizedBracketedNaN(NaN))"#,
        r#"console.log(frozenParenthesizedBracketedNaN(1))"#,
        r#"console.log(frozenParenthesizedBracketedInteger(alias))"#,
        r#"console.log(frozenParenthesizedBracketedSafeInteger(alias))"#,
        r#"console.log(finite(alias))"#,
        r#"console.log(integer(alias))"#,
        r#"console.log(safeInteger(alias))"#,
    ])
}

/// Canonical runtime source text for the supported Number predicate slice.
pub fn number_predicates_runtime_source() -> String {
    format!(
        "{} {}",
        number_predicates_preamble_source("1"),
        number_predicates_console_log_body_source()
    )
}

/// Canonical browser-bundle source text for the supported Number predicate slice.
pub fn number_predicates_browser_bundle_source(alias_literal: &str) -> String {
    format!(
        concat!(
            "// kali-tree-shake: browserNumberPredicates\n",
            "async function browserNumberPredicates() {{\n",
            "  const alias = {};\n",
            "  const finite = Number.isFinite;\n",
            "  const integer = Number.isInteger;\n",
            "  const safeInteger = Number.isSafeInteger;\n",
            "  const frozenFinite = Object.freeze(Number.isFinite);\n",
            "  const frozenNaN = Object.freeze(Number.isNaN);\n",
            "  const frozenInteger = Object.freeze(Number.isInteger);\n",
            "  const frozenSafeInteger = Object.freeze(Number.isSafeInteger);\n",
            "  const frozenBracketedFinite = Object.freeze(Number[\"isFinite\"]);\n",
            "  const frozenBracketedNaN = Object.freeze(Number[\"isNaN\"]);\n",
            "  const frozenBracketedInteger = Object.freeze(Number[\"isInteger\"]);\n",
            "  const frozenBracketedSafeInteger = Object.freeze(Number[\"isSafeInteger\"]);\n",
            "  const frozenParenthesizedBracketedFinite = Object.freeze((globalThis[\"Number\"])[\"isFinite\"]);\n",
            "  const frozenParenthesizedBracketedNaN = Object.freeze((globalThis[\"Number\"])[\"isNaN\"]);\n",
            "  const frozenParenthesizedBracketedInteger = Object.freeze((globalThis[\"Number\"])[\"isInteger\"]);\n",
            "  const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis[\"Number\"])[\"isSafeInteger\"]);\n",
            "  if (\n",
            "    Number.isFinite(alias) !== true ||\n",
            "    Number.isSafeInteger(await alias) !== true ||\n",
            "    integer(alias) !== true ||\n",
            "    Number.isSafeInteger(alias) !== true ||\n",
            "    integer(1.5) !== false ||\n",
            "    Number.isFinite(\"hello\") !== false ||\n",
            "    Number.isSafeInteger(1.5) !== false ||\n",
            "    globalThis[\"Number\"][\"isNaN\"](NaN) !== true ||\n",
            "    globalThis.Number.isNaN(1) !== false ||\n",
            "    globalThis[\"Number\"].isNaN(1) !== false ||\n",
            "    globalThis[\"Number\"][\"isFinite\"](alias) !== true ||\n",
            "    globalThis[\"Number\"][\"isInteger\"](alias) !== true ||\n",
            "    globalThis[\"Number\"][\"isSafeInteger\"](alias) !== true ||\n",
            "    globalThis.Number[\"isNaN\"](1) !== false ||\n",
            "    globalThis[\"Number\"].isFinite(alias) !== true ||\n",
            "    globalThis.Number[\"isInteger\"](alias) !== true ||\n",
            "    globalThis[\"Number\"].isSafeInteger(alias) !== true ||\n",
            "    Number[\"isFinite\"](alias) !== true ||\n",
            "    Number[\"isInteger\"](alias) !== true ||\n",
            "    Number[\"isSafeInteger\"](alias) !== true ||\n",
            "    Number[\"isNaN\"](1) !== false ||\n",
            "    frozenFinite(alias) !== true ||\n",
            "    frozenNaN(NaN) !== true ||\n",
            "    frozenNaN(1) !== false ||\n",
            "    frozenInteger(alias) !== true ||\n",
            "    frozenSafeInteger(alias) !== true ||\n",
            "    frozenBracketedFinite(alias) !== true ||\n",
            "    frozenBracketedNaN(NaN) !== true ||\n",
            "    frozenBracketedNaN(1) !== false ||\n",
            "    frozenBracketedInteger(alias) !== true ||\n",
            "    frozenBracketedSafeInteger(alias) !== true ||\n",
            "    frozenParenthesizedBracketedFinite(alias) !== true ||\n",
            "    frozenParenthesizedBracketedNaN(NaN) !== true ||\n",
            "    frozenParenthesizedBracketedNaN(1) !== false ||\n",
            "    frozenParenthesizedBracketedInteger(alias) !== true ||\n",
            "    frozenParenthesizedBracketedSafeInteger(alias) !== true ||\n",
            "    safeInteger(alias) !== true ||\n",
            "    finite(alias) !== true\n",
            "  ) {{\n",
            "    throw new Error('unexpected browser Number predicate result');\n",
            "  }}\n",
            "  console.log('browser number predicates ok');\n",
            "}}\n"
        ),
        alias_literal
    )
}

/// Canonical `Kali.test` source text for the supported Number predicate slice.
pub fn number_predicates_test_source() -> String {
    format!(
        "Kali.test('number predicates', () => {{ {} {} }});",
        number_predicates_preamble_source("1"),
        number_predicates_console_log_body_source()
    )
}

/// Canonical source text for the supported `Object.prototype.hasOwnProperty.call` helper.
pub const fn object_has_own_property_call_source() -> &'static str {
    "Object.prototype.hasOwnProperty.call"
}

/// Canonical binding source for the supported `Object.prototype.hasOwnProperty.call` helper.
pub fn object_has_own_property_call_binding_source(binding_name: &str) -> String {
    format!(
        "const {binding_name} = {};",
        object_has_own_property_call_source()
    )
}

/// Canonical frozen callable aliases for the supported `Math.abs` / `Math.sign` helper slice.
pub const fn math_abs_sign_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["abs"])"#,
        r#"Object.freeze((globalThis.Math["abs"]))"#,
        r#"Object.freeze(globalThis.Math['abs'])"#,
        r#"Object.freeze((globalThis.Math['abs']))"#,
        r#"Object.freeze(globalThis.Math.abs)"#,
        r#"Object.freeze((globalThis.Math.abs))"#,
        r#"Object.freeze(globalThis["Math"]["abs"])"#,
        r#"Object.freeze((globalThis["Math"]["abs"]))"#,
        r#"Object.freeze(globalThis["Math"]['abs'])"#,
        r#"Object.freeze((globalThis["Math"]['abs']))"#,
        r#"Object.freeze(globalThis["Math"].abs)"#,
        r#"Object.freeze((globalThis["Math"].abs))"#,
        r#"Object.freeze(globalThis['Math']['abs'])"#,
        r#"Object.freeze((globalThis['Math']['abs']))"#,
        r#"Object.freeze(globalThis['Math'].abs)"#,
        r#"Object.freeze((globalThis['Math'].abs))"#,
        r#"Object.freeze(Math.abs)"#,
        r#"Object.freeze((Math.abs))"#,
        r#"Object.freeze(Math["abs"])"#,
        r#"Object.freeze((Math["abs"]))"#,
        r#"Object.freeze(Math['abs'])"#,
        r#"Object.freeze((Math['abs']))"#,
        r#"Object.freeze(globalThis.Math["sign"])"#,
        r#"Object.freeze((globalThis.Math["sign"]))"#,
        r#"Object.freeze(globalThis.Math['sign'])"#,
        r#"Object.freeze((globalThis.Math['sign']))"#,
        r#"Object.freeze(globalThis.Math.sign)"#,
        r#"Object.freeze((globalThis.Math.sign))"#,
        r#"Object.freeze(globalThis["Math"]["sign"])"#,
        r#"Object.freeze((globalThis["Math"]["sign"]))"#,
        r#"Object.freeze(globalThis["Math"]['sign'])"#,
        r#"Object.freeze((globalThis["Math"]['sign']))"#,
        r#"Object.freeze(globalThis["Math"].sign)"#,
        r#"Object.freeze((globalThis["Math"].sign))"#,
        r#"Object.freeze(globalThis['Math']['sign'])"#,
        r#"Object.freeze((globalThis['Math']['sign']))"#,
        r#"Object.freeze(globalThis['Math'].sign)"#,
        r#"Object.freeze((globalThis['Math'].sign))"#,
        r#"Object.freeze(Math.sign)"#,
        r#"Object.freeze((Math.sign))"#,
        r#"Object.freeze(Math["sign"])"#,
        r#"Object.freeze((Math["sign"]))"#,
        r#"Object.freeze(Math['sign'])"#,
        r#"Object.freeze((Math['sign']))"#,
    ]
}

/// Canonical source text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_abs_sign_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_abs_sign_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_invocation_source() -> String {
    math_abs_sign_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_entries(indentation: &str) -> String {
    math_abs_sign_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(alias)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_entries_source() -> String {
    math_abs_sign_frozen_callable_entries("")
}

/// Canonical frozen callable aliases for the supported `Math.floor` / `Math.trunc` / `Math.ceil` helper slice.
pub const fn math_floor_trunc_ceil_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math['floor'])"#,
        r#"Object.freeze((globalThis.Math['floor']))"#,
        r#"Object.freeze(globalThis.Math.floor)"#,
        r#"Object.freeze((globalThis.Math.floor))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze((globalThis["Math"]))["floor"]"#,
        r#"Object.freeze((globalThis["Math"]))['floor']"#,
        r#"Object.freeze((globalThis.Math))["floor"]"#,
        r#"Object.freeze((globalThis.Math))['floor']"#,
        r#"Object.freeze((globalThis['Math']))["floor"]"#,
        r#"Object.freeze((globalThis['Math']))['floor']"#,
        r#"Object.freeze(globalThis["Math"]['floor'])"#,
        r#"Object.freeze((globalThis["Math"]['floor']))"#,
        r#"Object.freeze(globalThis["Math"].floor)"#,
        r#"Object.freeze((globalThis["Math"])["floor"])"#,
        r#"Object.freeze((globalThis['Math'])['floor'])"#,
        r#"Object.freeze(globalThis['Math'].floor)"#,
        r#"Object.freeze((globalThis['Math']).floor)"#,
        r#"Object.freeze((globalThis["Math"]).floor)"#,
        r#"Object.freeze((globalThis["Math"].floor))"#,
        r#"Object.freeze(Math["floor"])"#,
        r#"Object.freeze((Math["floor"]))"#,
        r#"Object.freeze(Math['floor'])"#,
        r#"Object.freeze((Math['floor']))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math['trunc'])"#,
        r#"Object.freeze((globalThis.Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math.trunc)"#,
        r#"Object.freeze((globalThis.Math.trunc))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze((globalThis["Math"]))["trunc"]"#,
        r#"Object.freeze((globalThis["Math"]))['trunc']"#,
        r#"Object.freeze((globalThis.Math))["trunc"]"#,
        r#"Object.freeze((globalThis.Math))['trunc']"#,
        r#"Object.freeze((globalThis['Math']))["trunc"]"#,
        r#"Object.freeze((globalThis['Math']))['trunc']"#,
        r#"Object.freeze(globalThis["Math"]['trunc'])"#,
        r#"Object.freeze((globalThis["Math"]['trunc']))"#,
        r#"Object.freeze(globalThis["Math"].trunc)"#,
        r#"Object.freeze((globalThis["Math"])["trunc"])"#,
        r#"Object.freeze((globalThis['Math'])['trunc'])"#,
        r#"Object.freeze(globalThis['Math'].trunc)"#,
        r#"Object.freeze((globalThis['Math']).trunc)"#,
        r#"Object.freeze((globalThis["Math"]).trunc)"#,
        r#"Object.freeze((globalThis["Math"].trunc))"#,
        r#"Object.freeze(Math["trunc"])"#,
        r#"Object.freeze((Math["trunc"]))"#,
        r#"Object.freeze(Math['trunc'])"#,
        r#"Object.freeze((Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis.Math['ceil'])"#,
        r#"Object.freeze((globalThis.Math['ceil']))"#,
        r#"Object.freeze(globalThis.Math.ceil)"#,
        r#"Object.freeze((globalThis.Math.ceil))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
        r#"Object.freeze((globalThis["Math"]))["ceil"]"#,
        r#"Object.freeze((globalThis["Math"]))['ceil']"#,
        r#"Object.freeze((globalThis.Math))["ceil"]"#,
        r#"Object.freeze((globalThis.Math))['ceil']"#,
        r#"Object.freeze((globalThis['Math']))["ceil"]"#,
        r#"Object.freeze((globalThis['Math']))['ceil']"#,
        r#"Object.freeze(globalThis["Math"]['ceil'])"#,
        r#"Object.freeze((globalThis["Math"]['ceil']))"#,
        r#"Object.freeze(globalThis["Math"].ceil)"#,
        r#"Object.freeze((globalThis["Math"])["ceil"])"#,
        r#"Object.freeze((globalThis['Math'])['ceil'])"#,
        r#"Object.freeze(globalThis['Math'].ceil)"#,
        r#"Object.freeze((globalThis['Math']).ceil)"#,
        r#"Object.freeze((globalThis["Math"]).ceil)"#,
        r#"Object.freeze((globalThis["Math"].ceil))"#,
        r#"Object.freeze(Math["ceil"])"#,
        r#"Object.freeze((Math["ceil"]))"#,
        r#"Object.freeze(Math['ceil'])"#,
        r#"Object.freeze((Math['ceil']))"#,
    ]
}
/// Canonical source text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_floor_trunc_ceil_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_floor_trunc_ceil_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_invocation_source() -> String {
    math_floor_trunc_ceil_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_entries(indentation: &str) -> String {
    math_floor_trunc_ceil_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(alias)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_entries_source() -> String {
    math_floor_trunc_ceil_frozen_callable_entries("")
}

/// Canonical frozen callable aliases for the supported `Math.round` helper slice.
pub const fn math_round_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["round"])"#,
        r#"Object.freeze((globalThis.Math["round"]))"#,
        r#"Object.freeze(globalThis.Math['round'])"#,
        r#"Object.freeze((globalThis.Math['round']))"#,
        r#"Object.freeze(globalThis.Math.round)"#,
        r#"Object.freeze((globalThis.Math.round))"#,
        r#"Object.freeze(globalThis?.Math.round)"#,
        r#"Object.freeze((globalThis?.Math.round))"#,
        r#"Object.freeze(globalThis["Math"]["round"])"#,
        r#"Object.freeze((globalThis["Math"]["round"]))"#,
        r#"Object.freeze(globalThis["Math"]['round'])"#,
        r#"Object.freeze((globalThis["Math"]['round']))"#,
        r#"Object.freeze(globalThis["Math"].round)"#,
        r#"Object.freeze((globalThis["Math"]).round)"#,
        r#"Object.freeze((globalThis["Math"].round))"#,
        r#"Object.freeze((globalThis["Math"])["round"])"#,
        r#"Object.freeze((globalThis['Math'])['round'])"#,
        r#"Object.freeze((globalThis['Math'])["round"])"#,
        r#"Object.freeze(globalThis['Math']['round'])"#,
        r#"Object.freeze((globalThis['Math']['round']))"#,
        r#"Object.freeze(globalThis['Math'].round)"#,
        r#"Object.freeze((globalThis['Math']).round)"#,
        r#"Object.freeze((globalThis['Math'].round))"#,
        r#"Object.freeze(Math.round)"#,
        r#"Object.freeze((Math.round))"#,
        r#"Object.freeze(Math["round"])"#,
        r#"Object.freeze((Math["round"]))"#,
        r#"Object.freeze(Math['round'])"#,
        r#"Object.freeze((Math['round']))"#,
        r#"Object.freeze((null ?? Math.round))"#,
        r#"Object.freeze((true && globalThis.Math.round))"#,
        r#"Object.freeze((false || globalThis["Math"]["round"]))"#,
    ]
}

/// Canonical source text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_round_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_round_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(value));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_invocation_source() -> String {
    math_round_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_entries(indentation: &str) -> String {
    math_round_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(value)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_entries_source() -> String {
    math_round_frozen_callable_entries("")
}

/// Canonical direct aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_aliases() -> &'static [&'static str] {
    &[
        "Math.pow",
        r#"Math['pow']"#,
        r#"Math["pow"]"#,
        "globalThis.Math.pow",
        r#"globalThis.Math['pow']"#,
        r#"globalThis.Math["pow"]"#,
        r#"globalThis['Math'].pow"#,
        r#"globalThis['Math']['pow']"#,
        r#"globalThis['Math']["pow"]"#,
        r#"globalThis["Math"].pow"#,
        r#"globalThis["Math"]["pow"]"#,
        r#"globalThis["Math"]['pow']"#,
    ]
}

/// Canonical source text for the supported `Math.pow` helper aliases.
pub fn math_pow_source() -> String {
    join_semicolon_terminated_segments(math_pow_aliases())
}

/// Canonical source text for the supported `Math.pow` alias inventory.
pub fn math_pow_alias_inventory_source() -> String {
    format!(
        "{} {}",
        math_pow_source().trim_end(),
        math_pow_frozen_callable_source().trim_end()
    )
}

/// Canonical browser alias inventory for the supported `Math.pow` helper slice.
pub fn math_pow_browser_alias_inventory_aliases() -> Vec<&'static str> {
    let frozen_aliases = math_pow_frozen_callable_aliases();
    ordered_unique_union(&[
        math_pow_aliases(),
        frozen_aliases.as_slice(),
        math_pow_bracketed_frozen_callable_aliases(),
    ])
}

/// Canonical browser source text for the supported `Math.pow` alias inventory.
pub fn math_pow_browser_alias_inventory_source() -> String {
    join_semicolon_terminated_segments(&math_pow_browser_alias_inventory_aliases())
}

/// Canonical browser-invocation lines for the supported `Math.pow` browser alias inventory.
pub fn math_pow_browser_alias_inventory_invocation_lines(indentation: &str) -> String {
    math_pow_invocation_lines_for_aliases(
        math_pow_browser_alias_inventory_aliases().as_slice(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical browser-source invocation text for the supported `Math.pow` browser alias inventory.
pub fn math_pow_browser_alias_inventory_invocation_source() -> String {
    format!(
        "const exponent = 3; const alias = exponent;\n{}\n",
        math_pow_browser_alias_inventory_invocation_lines("")
    )
}

/// Canonical browser-bundle source text for the supported bracketed `globalThis["Math"].pow` alias chain.
pub const fn math_pow_bracketed_global_this_alias_chain_source() -> &'static str {
    r##"// kali-tree-shake: bracketedGlobalThisMathPowAliasChain
function bracketedGlobalThisMathPowAliasChain() {
  const exponent = 3;
  const alias = exponent;
  console.log(globalThis["Math"].pow(2, alias));
  return globalThis["Math"].pow(2, alias);
}
"##
}

/// Canonical browser smoke body for the supported `Promise.allSettled` slice.
pub const fn promise_all_settled_browser_body_source() -> &'static str {
    r#"  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"])["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis["Promise"]).allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis["Promise"]["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis["Promise"].allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"].allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const bracketedRootFrozenSettled = await Object.freeze(Promise["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const rootFrozenSettled = await Object.freeze(Promise.allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedRootFrozenSettled = await Object.freeze((Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  if (
    settled.length !== 2 ||
    settled[0].status !== 'fulfilled' ||
    settled[0].value !== 1 ||
    settled[1].status !== 'rejected' ||
    settled[1].reason !== 'boom' ||
    mixedSettled.length !== 2 ||
    mixedSettled[0].status !== 'fulfilled' ||
    mixedSettled[0].value !== 1 ||
    mixedSettled[1].status !== 'rejected' ||
    mixedSettled[1].reason !== 'boom' ||
    dottedSettled.length !== 2 ||
    dottedSettled[0].status !== 'fulfilled' ||
    dottedSettled[0].value !== 1 ||
    dottedSettled[1].status !== 'rejected' ||
    dottedSettled[1].reason !== 'boom' ||
    mixedDottedSettled.length !== 2 ||
    mixedDottedSettled[0].status !== 'fulfilled' ||
    mixedDottedSettled[0].value !== 1 ||
    mixedDottedSettled[1].status !== 'rejected' ||
    mixedDottedSettled[1].reason !== 'boom' ||
    mixedBracketedSettled.length !== 2 ||
    mixedBracketedSettled[0].status !== 'fulfilled' ||
    mixedBracketedSettled[0].value !== 1 ||
    mixedBracketedSettled[1].status !== 'rejected' ||
    mixedBracketedSettled[1].reason !== 'boom' ||
    bracketedSettled.length !== 2 ||
    bracketedSettled[0].status !== 'fulfilled' ||
    bracketedSettled[0].value !== 1 ||
    bracketedSettled[1].status !== 'rejected' ||
    bracketedSettled[1].reason !== 'boom' ||
    nullishRootSettled.length !== 2 ||
    nullishRootSettled[0].status !== 'fulfilled' ||
    nullishRootSettled[0].value !== 1 ||
    nullishRootSettled[1].status !== 'rejected' ||
    nullishRootSettled[1].reason !== 'boom' ||
    logicalAndRootSettled.length !== 2 ||
    logicalAndRootSettled[0].status !== 'fulfilled' ||
    logicalAndRootSettled[0].value !== 1 ||
    logicalAndRootSettled[1].status !== 'rejected' ||
    logicalAndRootSettled[1].reason !== 'boom' ||
    logicalOrRootSettled.length !== 2 ||
    logicalOrRootSettled[0].status !== 'fulfilled' ||
    logicalOrRootSettled[0].value !== 1 ||
    logicalOrRootSettled[1].status !== 'rejected' ||
    logicalOrRootSettled[1].reason !== 'boom' ||
    nullishDottedSettled.length !== 2 ||
    nullishDottedSettled[0].status !== 'fulfilled' ||
    nullishDottedSettled[0].value !== 1 ||
    nullishDottedSettled[1].status !== 'rejected' ||
    nullishDottedSettled[1].reason !== 'boom' ||
    logicalAndDottedSettled.length !== 2 ||
    logicalAndDottedSettled[0].status !== 'fulfilled' ||
    logicalAndDottedSettled[0].value !== 1 ||
    logicalAndDottedSettled[1].status !== 'rejected' ||
    logicalAndDottedSettled[1].reason !== 'boom' ||
    logicalOrDottedSettled.length !== 2 ||
    logicalOrDottedSettled[0].status !== 'fulfilled' ||
    logicalOrDottedSettled[0].value !== 1 ||
    logicalOrDottedSettled[1].status !== 'rejected' ||
    logicalOrDottedSettled[1].reason !== 'boom' ||
    wrappedBracketedDotRootFrozenSettled.length !== 2 ||
    wrappedBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||
    wrappedBracketedDotRootFrozenSettled[0].value !== 1 ||
    wrappedBracketedDotRootFrozenSettled[1].status !== 'rejected' ||
    wrappedBracketedDotRootFrozenSettled[1].reason !== 'boom' ||
    wrappedSingleBracketedDotRootFrozenSettled.length !== 2 ||
    wrappedSingleBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||
    wrappedSingleBracketedDotRootFrozenSettled[0].value !== 1 ||
    wrappedSingleBracketedDotRootFrozenSettled[1].status !== 'rejected' ||
    wrappedSingleBracketedDotRootFrozenSettled[1].reason !== 'boom' ||
    frozenBracketedSettled.length !== 2 ||
    frozenBracketedSettled[0].status !== 'fulfilled' ||
    frozenBracketedSettled[0].value !== 1 ||
    frozenBracketedSettled[1].status !== 'rejected' ||
    frozenBracketedSettled[1].reason !== 'boom' ||
    parenthesizedFrozenBracketedSettled.length !== 2 ||
    parenthesizedFrozenBracketedSettled[0].status !== 'fulfilled' ||
    parenthesizedFrozenBracketedSettled[0].value !== 1 ||
    parenthesizedFrozenBracketedSettled[1].status !== 'rejected' ||
    parenthesizedFrozenBracketedSettled[1].reason !== 'boom' ||
    mixedBracketedRootFrozenSettled.length !== 2 ||
    mixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    mixedBracketedRootFrozenSettled[0].value !== 1 ||
    mixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    mixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedMixedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedMixedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    singleMixedBracketedRootFrozenSettled.length !== 2 ||
    singleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    singleMixedBracketedRootFrozenSettled[0].value !== 1 ||
    singleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    singleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    fullyBracketedSingleRootFrozenSettled.length !== 2 ||
    fullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||
    fullyBracketedSingleRootFrozenSettled[0].value !== 1 ||
    fullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||
    fullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled.length !== 2 ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[0].value !== 1 ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    mixedRootFrozenSettled.length !== 2 ||
    mixedRootFrozenSettled[0].status !== 'fulfilled' ||
    mixedRootFrozenSettled[0].value !== 1 ||
    mixedRootFrozenSettled[1].status !== 'rejected' ||
    mixedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedMixedRootFrozenSettled.length !== 2 ||
    parenthesizedMixedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedMixedRootFrozenSettled[0].value !== 1 ||
    parenthesizedMixedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedMixedRootFrozenSettled[1].reason !== 'boom' ||
    bracketedRootFrozenSettled.length !== 2 ||
    bracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    bracketedRootFrozenSettled[0].value !== 1 ||
    bracketedRootFrozenSettled[1].status !== 'rejected' ||
    bracketedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    rootFrozenSettled.length !== 2 ||
    rootFrozenSettled[0].status !== 'fulfilled' ||
    rootFrozenSettled[0].value !== 1 ||
    rootFrozenSettled[1].status !== 'rejected' ||
    rootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedRootFrozenSettled.length !== 2 ||
    parenthesizedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedRootFrozenSettled[0].value !== 1 ||
    parenthesizedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedRootFrozenSettled[1].reason !== 'boom'
  ) {
    throw new Error('unexpected Promise.allSettled semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.race` slice.
pub const fn promise_race_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  const mixed = await Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);
  const dotted = await globalThis.Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  const bracketed = await globalThis["Promise"].race([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);
  const mixedDotted = await globalThis.Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);
  const bracketedBracketed = await globalThis["Promise"]["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketedBracketed = await Object.freeze((globalThis["Promise"]["race"]))([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketed = await Object.freeze(globalThis["Promise"].race)([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);
  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketedBracketed = await Object.freeze(globalThis["Promise"]["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);
  if (
    direct !== 1 ||
    mixed !== 1 ||
    singleMixed !== 1 ||
    dotted !== 1 ||
    bracketed !== 1 ||
    singleBracketed !== 1 ||
    mixedDotted !== 1 ||
    singleDotted !== 1 ||
    bracketedBracketed !== 1 ||
    singleBracketedBracketed !== 1 ||
    parenthesizedBracketed !== 1 ||
    parenthesizedSingleBracketed !== 1 ||
    parenthesizedDottedBracketed !== 1 ||
    parenthesizedSingleDottedBracketed !== 1 ||
    parenthesizedBracketedBracketed !== 1 ||
    parenthesizedSingleBracketedBracketed !== 1 ||
    frozenRoot !== 1 ||
    parenthesizedFrozenRoot !== 1 ||
    frozenBracketed !== 1 ||
    frozenSingleBracketed !== 1 ||
    frozenDottedBracketed !== 1 ||
    frozenSingleDottedBracketed !== 1 ||
    frozenBracketedBracketed !== 1 ||
    frozenSingleBracketedBracketed !== 1 ||
    frozenDotted !== 1 ||
    parenthesizedFrozenDotted !== 1
  ) {
    throw new Error('unexpected Promise.race semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.any` slice.
pub const fn promise_any_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  const mixed = await Promise["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleMixed = await Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const dotted = await globalThis.Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  const mixedDotted = await globalThis.Promise["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleDotted = await globalThis.Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const bracketed = await globalThis["Promise"].any([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const mixedBracketed = await globalThis["Promise"]["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleBracketed = await globalThis['Promise']['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const singleMixedBracketed = await globalThis['Promise'].any([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenBracketed = await Object.freeze(globalThis["Promise"].any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenMixedBracketed = await Object.freeze(globalThis["Promise"]["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleBracketRoot = await Object.freeze(globalThis['Promise']['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleMixedBracketed = await Object.freeze(globalThis["Promise"]['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenMixedBracketed = await Object.freeze((globalThis["Promise"]["any"]))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleBracketRoot = await Object.freeze((globalThis['Promise']['any']))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleMixedBracketed = await Object.freeze((globalThis["Promise"]['any']))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenReceiverWrappedDotted = await Object.freeze((globalThis["Promise"]).any)([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleReceiverWrappedDotted = await Object.freeze((globalThis['Promise']).any)([Promise.reject('boom'), Promise.resolve(1)]);
  const nullishRoot = await Object.freeze((null ?? Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const logicalAndRoot = await Object.freeze((true && Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const logicalOrRoot = await Object.freeze((false || Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenRoot = await Object.freeze(Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenDotted = await Object.freeze(globalThis.Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  if (
    direct !== 1 ||
    mixed !== 1 ||
    singleMixed !== 1 ||
    dotted !== 1 ||
    mixedDotted !== 1 ||
    singleDotted !== 1 ||
    bracketed !== 1 ||
    parenthesizedBracketed !== 1 ||
    mixedBracketed !== 1 ||
    singleBracketed !== 1 ||
    parenthesizedSingleBracketed !== 1 ||
    singleMixedBracketed !== 1 ||
    frozenBracketed !== 1 ||
    frozenSingleBracketed !== 1 ||
    parenthesizedFrozenMixedBracketed !== 1 ||
    parenthesizedFrozenSingleBracketRoot !== 1 ||
    parenthesizedFrozenReceiverWrappedDotted !== 1 ||
    parenthesizedFrozenSingleReceiverWrappedDotted !== 1 ||
    frozenMixedBracketed !== 1 ||
    frozenSingleBracketRoot !== 1 ||
    nullishRoot !== 1 ||
    logicalAndRoot !== 1 ||
    logicalOrRoot !== 1 ||
    frozenRoot !== 1 ||
    parenthesizedFrozenRoot !== 1 ||
    frozenDotted !== 1 ||
    parenthesizedDottedBracketed !== 1 ||
    parenthesizedSingleDottedBracketed !== 1 ||
    parenthesizedFrozenDotted !== 1
  ) {
    throw new Error('unexpected Promise.any semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.all` slice.
pub const fn promise_all_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);
  const mixed = await Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);
  const dotted = await globalThis.Promise.all([Promise.resolve(1), Promise.resolve(2)]);
  const mixedDotted = await globalThis.Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleDotted = await globalThis.Promise['all']([Promise.resolve(1), Promise.resolve(2)]);
  const bracketed = await globalThis["Promise"].all([Promise.resolve(1), Promise.resolve(2)]);
  const mixedBracketed = await globalThis["Promise"]["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketed = await globalThis['Promise']['all']([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedBracketed = await globalThis['Promise'].all([Promise.resolve(1), Promise.resolve(2)]);
  const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalOrRoot = await Object.freeze((false || Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalAndDotted = await Object.freeze((true && globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketedRoot = await Object.freeze(Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const mixedRoot = await Object.freeze(globalThis.Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedMixedRoot = await Object.freeze((globalThis.Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedRoot = await Object.freeze(globalThis.Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleMixedRoot = await Object.freeze((globalThis.Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const bracketedRoot = await Object.freeze(globalThis["Promise"].all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketedRoot = await Object.freeze((globalThis["Promise"].all))([Promise.resolve(1), Promise.resolve(2)]);
  const mixedBracketedRoot = await Object.freeze(globalThis["Promise"]["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedMixedBracketedRoot = await Object.freeze((globalThis["Promise"]["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedBracketedRoot = await Object.freeze(globalThis['Promise'].all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleMixedBracketedRoot = await Object.freeze((globalThis['Promise'].all))([Promise.resolve(1), Promise.resolve(2)]);
  const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFullyBracketedSingleRoot = await Object.freeze((globalThis['Promise']['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  if (
    direct.length !== 2 ||
    direct[0] !== 1 ||
    direct[1] !== 2 ||
    mixed.length !== 2 ||
    mixed[0] !== 1 ||
    mixed[1] !== 2 ||
    singleMixed.length !== 2 ||
    singleMixed[0] !== 1 ||
    singleMixed[1] !== 2 ||
    dotted.length !== 2 ||
    dotted[0] !== 1 ||
    dotted[1] !== 2 ||
    mixedDotted.length !== 2 ||
    mixedDotted[0] !== 1 ||
    mixedDotted[1] !== 2 ||
    singleDotted.length !== 2 ||
    singleDotted[0] !== 1 ||
    singleDotted[1] !== 2 ||
    bracketed.length !== 2 ||
    bracketed[0] !== 1 ||
    bracketed[1] !== 2 ||
    mixedBracketed.length !== 2 ||
    mixedBracketed[0] !== 1 ||
    mixedBracketed[1] !== 2 ||
    singleBracketed.length !== 2 ||
    singleBracketed[0] !== 1 ||
    singleBracketed[1] !== 2 ||
    singleMixedBracketed.length !== 2 ||
    singleMixedBracketed[0] !== 1 ||
    singleMixedBracketed[1] !== 2 ||
    nullishRoot.length !== 2 ||
    nullishRoot[0] !== 1 ||
    nullishRoot[1] !== 2 ||
    logicalAndRoot.length !== 2 ||
    logicalAndRoot[0] !== 1 ||
    logicalAndRoot[1] !== 2 ||
    logicalOrRoot.length !== 2 ||
    logicalOrRoot[0] !== 1 ||
    logicalOrRoot[1] !== 2 ||
    nullishDotted.length !== 2 ||
    nullishDotted[0] !== 1 ||
    nullishDotted[1] !== 2 ||
    logicalAndDotted.length !== 2 ||
    logicalAndDotted[0] !== 1 ||
    logicalAndDotted[1] !== 2 ||
    logicalOrDotted.length !== 2 ||
    logicalOrDotted[0] !== 1 ||
    logicalOrDotted[1] !== 2 ||
    frozenRoot.length !== 2 ||
    frozenRoot[0] !== 1 ||
    frozenRoot[1] !== 2 ||
    parenthesizedFrozenRoot.length !== 2 ||
    parenthesizedFrozenRoot[0] !== 1 ||
    parenthesizedFrozenRoot[1] !== 2 ||
    frozenBracketedRoot.length !== 2 ||
    frozenBracketedRoot[0] !== 1 ||
    frozenBracketedRoot[1] !== 2 ||
    parenthesizedFrozenBracketedRoot.length !== 2 ||
    parenthesizedFrozenBracketedRoot[0] !== 1 ||
    parenthesizedFrozenBracketedRoot[1] !== 2 ||
    frozenSingleBracketedRoot.length !== 2 ||
    frozenSingleBracketedRoot[0] !== 1 ||
    frozenSingleBracketedRoot[1] !== 2 ||
    parenthesizedFrozenSingleBracketedRoot.length !== 2 ||
    parenthesizedFrozenSingleBracketedRoot[0] !== 1 ||
    parenthesizedFrozenSingleBracketedRoot[1] !== 2 ||
    frozenGlobal.length !== 2 ||
    frozenGlobal[0] !== 1 ||
    frozenGlobal[1] !== 2 ||
    parenthesizedFrozenGlobal.length !== 2 ||
    parenthesizedFrozenGlobal[0] !== 1 ||
    parenthesizedFrozenGlobal[1] !== 2
  ) {
    throw new Error("unexpected Promise.all results");
  }
"#
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.pow` helper slice.
pub fn math_pow_invocation_lines(source: &str, indentation: &str) -> String {
    source
        .trim_end_matches(';')
        .split("; ")
        .map(|alias| format!("{indentation}console.log({alias}(2, alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation lines for an arbitrary `Math.pow` alias inventory.
pub fn math_pow_invocation_lines_for_aliases(
    aliases: &[&str],
    base: &str,
    argument: &str,
    indentation: &str,
) -> String {
    aliases
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}({base}, {argument}));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `return [...]` invocation entries for an arbitrary `Math.pow` alias inventory.
pub fn math_pow_invocation_entries_for_aliases(
    aliases: &[&str],
    base: &str,
    argument: &str,
    indentation: &str,
) -> String {
    aliases
        .iter()
        .map(|alias| format!("{indentation}{alias}({base}, {argument}),"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical direct frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_direct_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math['pow'])"#,
        r#"Object.freeze(globalThis.Math["pow"])"#,
        r#"Object.freeze(globalThis['Math']['pow'])"#,
        r#"Object.freeze(globalThis['Math']["pow"])"#,
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
        r#"Object.freeze(globalThis["Math"]['pow'])"#,
        r#"Object.freeze(globalThis.Math.pow)"#,
        r#"Object.freeze(globalThis['Math'].pow)"#,
        r#"Object.freeze(globalThis["Math"].pow)"#,
        r#"Object.freeze(Math.pow)"#,
        r#"Object.freeze(Math['pow'])"#,
        r#"Object.freeze(Math["pow"])"#,
    ]
}

/// Canonical parenthesized frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_parenthesized_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis.Math['pow']))"#,
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze((globalThis['Math']['pow']))"#,
        r#"Object.freeze((globalThis['Math']["pow"]))"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((globalThis["Math"]['pow']))"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze((globalThis['Math'].pow))"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
        r#"Object.freeze((Math.pow))"#,
        r#"Object.freeze((Math['pow']))"#,
        r#"Object.freeze((Math["pow"]))"#,
    ]
}

/// Canonical nullish/logical frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_nullish_logical_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((null ?? Math.pow))"#,
        r#"Object.freeze((true && Math.pow))"#,
        r#"Object.freeze((false || Math.pow))"#,
        r#"Object.freeze((null ?? globalThis.Math.pow))"#,
        r#"Object.freeze((true && globalThis.Math.pow))"#,
        r#"Object.freeze((false || globalThis.Math.pow))"#,
        r#"Object.freeze((null ?? globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((true && globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((false || globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((null ?? globalThis['Math']['pow']))"#,
        r#"Object.freeze((true && globalThis['Math']['pow']))"#,
        r#"Object.freeze((false || globalThis['Math']['pow']))"#,
    ]
}

/// Canonical bracketed-root frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_bracketed_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis.Math))["pow"]"#,
        r#"Object.freeze((globalThis.Math))['pow']"#,
        r#"Object.freeze((globalThis.Math).pow)"#,
        r#"Object.freeze((globalThis.Math)['pow'])"#,
        r#"Object.freeze((globalThis["Math"]))["pow"]"#,
        r#"Object.freeze((globalThis['Math']))['pow']"#,
        r#"Object.freeze((globalThis['Math'])["pow"])"#,
        r#"Object.freeze((globalThis['Math'])['pow'])"#,
        r#"Object.freeze((globalThis["Math"]).pow)"#,
        r#"Object.freeze((globalThis['Math']).pow)"#,
    ]
}

/// Canonical source text for the supported `Math.pow` bracketed-root frozen callable aliases.
pub fn math_pow_bracketed_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_pow_bracketed_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported bracketed-root frozen `Math.pow` aliases.
pub fn math_pow_bracketed_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_pow_invocation_lines_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical `return [...]` invocation entries for the supported bracketed-root frozen `Math.pow` aliases.
pub fn math_pow_bracketed_frozen_callable_invocation_entries(indentation: &str) -> String {
    math_pow_invocation_entries_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical frozen callable aliases for the supported `Math.pow` helper slice.
pub fn math_pow_frozen_callable_aliases() -> Vec<&'static str> {
    ordered_unique_union(&[
        math_pow_frozen_callable_direct_aliases(),
        math_pow_frozen_callable_parenthesized_aliases(),
        math_pow_frozen_callable_nullish_logical_aliases(),
    ])
}

/// Canonical source text for the supported `Math.pow` frozen callable aliases.
pub fn math_pow_frozen_callable_source() -> String {
    let aliases = math_pow_frozen_callable_aliases();
    join_semicolon_terminated_segments(&aliases)
}

/// Canonical frozen callable aliases for the supported `Math.cbrt` helper slice.
pub const fn math_cbrt_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["cbrt"])"#,
        r#"Object.freeze((globalThis.Math["cbrt"]))"#,
        r#"Object.freeze(globalThis.Math['cbrt'])"#,
        r#"Object.freeze((globalThis.Math['cbrt']))"#,
        r#"Object.freeze(globalThis.Math.cbrt)"#,
        r#"Object.freeze((globalThis.Math.cbrt))"#,
        r#"Object.freeze((globalThis.Math)["cbrt"])"#,
        r#"Object.freeze((globalThis.Math)['cbrt'])"#,
        r#"Object.freeze(globalThis["Math"]["cbrt"])"#,
        r#"Object.freeze((globalThis["Math"]["cbrt"]))"#,
        r#"Object.freeze(globalThis["Math"]['cbrt'])"#,
        r#"Object.freeze((globalThis["Math"]['cbrt']))"#,
        r#"Object.freeze((globalThis["Math"]))["cbrt"]"#,
        r#"Object.freeze((globalThis["Math"]))['cbrt']"#,
        r#"Object.freeze((globalThis.Math))["cbrt"]"#,
        r#"Object.freeze((globalThis.Math))['cbrt']"#,
        r#"Object.freeze((globalThis["Math"]).cbrt)"#,
        r#"Object.freeze((globalThis["Math"])["cbrt"])"#,
        r#"Object.freeze(globalThis["Math"].cbrt)"#,
        r#"Object.freeze((globalThis["Math"].cbrt))"#,
        r#"Object.freeze((globalThis['Math'])["cbrt"])"#,
        r#"Object.freeze((globalThis['Math'])['cbrt'])"#,
        r#"Object.freeze((globalThis['Math']))["cbrt"]"#,
        r#"Object.freeze((globalThis['Math']))['cbrt']"#,
        r#"Object.freeze(globalThis['Math'].cbrt)"#,
        r#"Object.freeze((globalThis['Math'].cbrt))"#,
        r#"Object.freeze(Math.cbrt)"#,
        r#"Object.freeze((Math.cbrt))"#,
        r#"Object.freeze(Math["cbrt"])"#,
        r#"Object.freeze((Math["cbrt"]))"#,
        r#"Object.freeze(Math['cbrt'])"#,
        r#"Object.freeze((Math['cbrt']))"#,
    ]
}

/// Canonical source text for the supported `Math.cbrt` frozen callable aliases.
pub fn math_cbrt_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_cbrt_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Math.hypot` helper slice.
pub const fn math_hypot_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["hypot"])"#,
        r#"Object.freeze((globalThis.Math["hypot"]))"#,
        r#"Object.freeze(globalThis.Math['hypot'])"#,
        r#"Object.freeze((globalThis.Math['hypot']))"#,
        r#"Object.freeze(globalThis.Math.hypot)"#,
        r#"Object.freeze((globalThis.Math.hypot))"#,
        r#"Object.freeze(globalThis["Math"]["hypot"])"#,
        r#"Object.freeze((globalThis["Math"]["hypot"]))"#,
        r#"Object.freeze(globalThis["Math"]['hypot'])"#,
        r#"Object.freeze((globalThis["Math"]['hypot']))"#,
        r#"Object.freeze((globalThis["Math"]).hypot)"#,
        r#"Object.freeze((globalThis["Math"])["hypot"])"#,
        r#"Object.freeze((globalThis["Math"])['hypot'])"#,
        r#"Object.freeze(globalThis["Math"].hypot)"#,
        r#"Object.freeze((globalThis["Math"].hypot))"#,
        r#"Object.freeze(globalThis['Math']['hypot'])"#,
        r#"Object.freeze((globalThis['Math']['hypot']))"#,
        r#"Object.freeze((globalThis['Math']).hypot)"#,
        r#"Object.freeze((globalThis['Math'])["hypot"])"#,
        r#"Object.freeze((globalThis['Math'])['hypot'])"#,
        r#"Object.freeze((globalThis["Math"]))["hypot"]"#,
        r#"Object.freeze((globalThis['Math']))["hypot"]"#,
        r#"Object.freeze((globalThis.Math))["hypot"]"#,
        r#"Object.freeze((globalThis.Math))['hypot']"#,
        r#"Object.freeze(globalThis['Math'].hypot)"#,
        r#"Object.freeze((globalThis['Math'].hypot))"#,
        r#"Object.freeze(Math.hypot)"#,
        r#"Object.freeze((Math.hypot))"#,
        r#"Object.freeze(Math["hypot"])"#,
        r#"Object.freeze((Math["hypot"]))"#,
        r#"Object.freeze(Math['hypot'])"#,
        r#"Object.freeze((Math['hypot']))"#,
    ]
}

/// Canonical source text for the supported `Math.hypot` frozen callable aliases.
pub fn math_hypot_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_hypot_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Math.exp2` helper slice.
pub const fn math_exp2_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["exp2"])"#,
        r#"Object.freeze((globalThis.Math["exp2"]))"#,
        r#"Object.freeze(globalThis.Math['exp2'])"#,
        r#"Object.freeze((globalThis.Math['exp2']))"#,
        r#"Object.freeze(globalThis.Math.exp2)"#,
        r#"Object.freeze((globalThis.Math.exp2))"#,
        r#"Object.freeze(globalThis?.Math.exp2)"#,
        r#"Object.freeze((globalThis?.Math.exp2))"#,
        r#"Object.freeze(globalThis["Math"]["exp2"])"#,
        r#"Object.freeze((globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze(globalThis["Math"]['exp2'])"#,
        r#"Object.freeze((globalThis["Math"]['exp2']))"#,
        r#"Object.freeze(globalThis["Math"].exp2)"#,
        r#"Object.freeze((globalThis["Math"]).exp2)"#,
        r#"Object.freeze((globalThis["Math"].exp2))"#,
        r#"Object.freeze((globalThis["Math"])["exp2"])"#,
        r#"Object.freeze((globalThis['Math'])['exp2'])"#,
        r#"Object.freeze((globalThis['Math'])["exp2"])"#,
        r#"Object.freeze(globalThis['Math']['exp2'])"#,
        r#"Object.freeze((globalThis['Math']['exp2']))"#,
        r#"Object.freeze(globalThis['Math'].exp2)"#,
        r#"Object.freeze((globalThis['Math']).exp2)"#,
        r#"Object.freeze((globalThis['Math'].exp2))"#,
        r#"Object.freeze(Math.exp2)"#,
        r#"Object.freeze((Math.exp2))"#,
        r#"Object.freeze(Math["exp2"])"#,
        r#"Object.freeze((Math["exp2"]))"#,
        r#"Object.freeze(Math['exp2'])"#,
        r#"Object.freeze((Math['exp2']))"#,
        r#"Object.freeze((null ?? globalThis.Math["exp2"]))"#,
        r#"Object.freeze((true && globalThis.Math["exp2"]))"#,
        r#"Object.freeze((false || globalThis.Math["exp2"]))"#,
        r#"Object.freeze((null ?? globalThis["Math"].exp2))"#,
        r#"Object.freeze((true && globalThis["Math"].exp2))"#,
        r#"Object.freeze((false || globalThis["Math"].exp2))"#,
        r#"Object.freeze((null ?? Math.exp2))"#,
        r#"Object.freeze((true && globalThis.Math.exp2))"#,
        r#"Object.freeze((false || globalThis.Math.exp2))"#,
        r#"Object.freeze((null ?? globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze((true && globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze((false || globalThis["Math"]["exp2"]))"#,
    ]
}

/// Canonical source text for the supported `Math.exp2` frozen callable aliases.
pub fn math_exp2_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_exp2_frozen_callable_aliases())
}

/// Canonical aliases for the supported `Array.from` helper slice.
pub const fn array_from_aliases() -> &'static [&'static str] {
    &[
        "Array.from",
        "globalThis.Array.from",
        r#"globalThis["Array"].from"#,
        r#"globalThis["Array"]["from"]"#,
        r#"globalThis["Array"]['from']"#,
        r#"globalThis['Array'].from"#,
        r#"globalThis['Array']['from']"#,
        r#"globalThis['Array']["from"]"#,
        r#"Array["from"]"#,
        r#"Array['from']"#,
        r#"globalThis.Array["from"]"#,
        r#"globalThis.Array['from']"#,
    ]
}

/// Canonical source text for the supported `Array.from` helper aliases.
pub fn array_from_source() -> String {
    join_semicolon_terminated_segments(array_from_aliases())
}

/// Canonical frozen callable aliases for the supported `Array.from` helper slice.
pub const fn array_from_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Array.from)"#,
        r#"Object.freeze((Array.from))"#,
        r#"Object.freeze(globalThis.Array.from)"#,
        r#"Object.freeze((globalThis.Array.from))"#,
        r#"Object.freeze(globalThis["Array"].from)"#,
        r#"Object.freeze((globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis["Array"]).from)"#,
        r#"Object.freeze((globalThis["Array"])["from"])"#,
        r#"Object.freeze(globalThis["Array"]["from"])"#,
        r#"Object.freeze((globalThis["Array"]["from"]))"#,
        r#"Object.freeze(globalThis['Array'].from)"#,
        r#"Object.freeze((globalThis['Array'].from))"#,
        r#"Object.freeze((globalThis['Array']).from)"#,
        r#"Object.freeze((globalThis['Array'])["from"])"#,
        r#"Object.freeze((globalThis["Array"]))["from"]"#,
        r#"Object.freeze((globalThis['Array']))["from"]"#,
        r#"Object.freeze((globalThis['Array']))['from']"#,
        r#"Object.freeze(globalThis['Array']['from'])"#,
        r#"Object.freeze((globalThis['Array']['from']))"#,
        r#"Object.freeze(globalThis["Array"]['from'])"#,
        r#"Object.freeze((globalThis["Array"]['from']))"#,
        r#"Object.freeze((globalThis['Array'])['from'])"#,
        r#"Object.freeze(globalThis['Array']["from"])"#,
        r#"Object.freeze((globalThis['Array']["from"]))"#,
        r#"Object.freeze(Array['from'])"#,
        r#"Object.freeze((Array['from']))"#,
        r#"Object.freeze(Array["from"])"#,
        r#"Object.freeze((Array["from"]))"#,
        r#"Object.freeze(globalThis.Array['from'])"#,
        r#"Object.freeze((globalThis.Array['from']))"#,
        r#"Object.freeze(globalThis.Array["from"])"#,
        r#"Object.freeze((null ?? globalThis.Array["from"]))"#,
        r#"Object.freeze((true && globalThis.Array["from"]))"#,
        r#"Object.freeze((false || globalThis.Array["from"]))"#,
        r#"Object.freeze((globalThis.Array["from"]))"#,
        r#"Object.freeze((globalThis.Array).from)"#,
        r#"Object.freeze((globalThis.Array)["from"])"#,
        r#"Object.freeze((globalThis.Array))["from"]"#,
        r#"Object.freeze((globalThis.Array))['from']"#,
        r#"Object.freeze((globalThis.Array)['from'])"#,
        r#"Object.freeze((null ?? globalThis.Array.from))"#,
        r#"Object.freeze((true && globalThis.Array.from))"#,
        r#"Object.freeze((false || globalThis.Array.from))"#,
        r#"Object.freeze((null ?? Array.from))"#,
        r#"Object.freeze((true && Array.from))"#,
        r#"Object.freeze((false || Array.from))"#,
        r#"Object.freeze((null ?? globalThis["Array"].from))"#,
        r#"Object.freeze((true && globalThis["Array"].from))"#,
        r#"Object.freeze((false || globalThis["Array"].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#,
        r#"Object.freeze((true && globalThis["Array"]["from"]))"#,
        r#"Object.freeze((false || globalThis["Array"]["from"]))"#,
        r#"Object.freeze((null ?? globalThis['Array']['from']))"#,
        r#"Object.freeze((true && globalThis['Array']['from']))"#,
        r#"Object.freeze((false || globalThis['Array']['from']))"#,
        r#"Object.freeze((null ?? globalThis['Array'].from))"#,
        r#"Object.freeze((true && globalThis['Array'].from))"#,
        r#"Object.freeze((false || globalThis['Array'].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]['from']))"#,
        r#"Object.freeze((true && globalThis["Array"]['from']))"#,
        r#"Object.freeze((false || globalThis["Array"]['from']))"#,
        r#"Object.freeze((null ?? globalThis.Array['from']))"#,
        r#"Object.freeze((true && globalThis.Array['from']))"#,
        r#"Object.freeze((false || globalThis.Array['from']))"#,
    ]
}

/// Canonical source text for the supported `Array.from` frozen callable aliases.
pub fn array_from_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(array_from_frozen_callable_aliases())
}

/// Canonical source text for the supported `Array.from` alias inventory.
pub fn array_from_alias_inventory_source() -> String {
    format!(
        "{} {}",
        array_from_source().trim_end(),
        array_from_frozen_callable_source().trim_end()
    )
}

/// Canonical `for`/`for await` loop lines for the supported `Array.from` helper slice.
pub fn array_from_loop_lines(source: &str, loop_header: &str, indentation: &str) -> String {
    source
        .trim_end_matches(';')
        .split("; ")
        .map(|alias| {
            format!(
                "{indentation}{loop_header}{alias}(values)) {{\n{indentation}  console.log(value);\n{indentation}}}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical body for the supported template-literal string iteration slice.
pub const fn template_literal_string_iteration_body_source() -> &'static str {
    r#"for (const ch of `hello`) { console.log(ch); }"#
}

/// Canonical browser body for the supported template-literal string iteration slice.
pub const fn browser_template_literal_string_iteration_body_source() -> &'static str {
    r#"const prefix = "he";
const suffix = "llo";
const syncChars = [];
for (const item of `${prefix}${suffix}`) {
  syncChars.push(item);
}
const asyncChars = [];
for await (const item of `${prefix}${suffix}`) {
  asyncChars.push(item);
}
if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {
  throw new Error('unexpected template literal iteration semantics');
}"#
}

/// Canonical root aliases for the supported `Set` constructor slice.
pub const fn set_constructor_aliases() -> &'static [&'static str] {
    &[
        "Set",
        "globalThis.Set",
        r#"globalThis["Set"]"#,
        r#"globalThis['Set']"#,
    ]
}

/// Canonical frozen callable aliases for the supported `Set` constructor slice.
pub const fn set_constructor_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Set)"#,
        r#"Object.freeze((Set))"#,
        r#"Object.freeze((null ?? Set))"#,
        r#"Object.freeze((true && Set))"#,
        r#"Object.freeze((false || Set))"#,
        r#"Object.freeze(globalThis.Set)"#,
        r#"Object.freeze((globalThis.Set))"#,
        r#"Object.freeze((null ?? globalThis.Set))"#,
        r#"Object.freeze((true && globalThis.Set))"#,
        r#"Object.freeze((false || globalThis.Set))"#,
        r#"Object.freeze(globalThis["Set"])"#,
        r#"Object.freeze((globalThis["Set"]))"#,
        r#"Object.freeze((null ?? globalThis["Set"]))"#,
        r#"Object.freeze((true && globalThis["Set"]))"#,
        r#"Object.freeze((false || globalThis["Set"]))"#,
        r#"Object.freeze(globalThis['Set'])"#,
        r#"Object.freeze((globalThis['Set']))"#,
        r#"Object.freeze((null ?? globalThis['Set']))"#,
        r#"Object.freeze((true && globalThis['Set']))"#,
        r#"Object.freeze((false || globalThis['Set']))"#,
    ]
}

/// Canonical source text for the supported `Set` constructor aliases.
pub fn set_constructor_source() -> String {
    join_semicolon_terminated_segments(set_constructor_aliases())
}

/// Canonical source text for the supported `Set` constructor iteration smoke.
pub fn set_constructor_iteration_source() -> String {
    concat!(
        "for (const value of new Set([1, 2, 1])) { console.log(value); } ",
        "for (const value of new Set(Object.freeze([1, 2, 1]))) { console.log(value); } ",
        "for (const value of new globalThis.Set([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis[\"Set\"]([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis['Set']([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis['Set'](Object.freeze([1, 2, 1]))) { console.log(value); } ",
        "for (const value of new (Object.freeze((Set)))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis.Set)))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis[\"Set\"])))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis['Set'])))([1, 2, 1])) { console.log(value); } ",
        "for (const value of Object.freeze(new Set([1, 2, 1]))) { console.log(value); } ",
        "for (const value of Object.freeze((new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((null ?? new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((true && new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((false || new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze(new globalThis[\"Set\"]([1, 2, 1]))) { console.log(value); } ",
        "for (const value of Object.freeze((new globalThis[\"Set\"]([1, 2, 1])))) { console.log(value); } ",
        "for (const value of new (null ?? Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (null ?? globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (null ?? globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || globalThis['Set'])([1, 2, 1])) { console.log(value); }"
    )
    .to_string()
}

/// Canonical source text for the supported `Set` frozen callable aliases.
pub fn set_constructor_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(set_constructor_frozen_callable_aliases())
}

/// Canonical root aliases for the supported `Map` constructor slice.
pub const fn map_constructor_aliases() -> &'static [&'static str] {
    &[
        "Map",
        "globalThis.Map",
        r#"globalThis["Map"]"#,
        r#"globalThis['Map']"#,
    ]
}

/// Canonical frozen callable aliases for the supported `Map` constructor slice.
pub const fn map_constructor_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Map)"#,
        r#"Object.freeze((Map))"#,
        r#"Object.freeze((null ?? Map))"#,
        r#"Object.freeze((true && Map))"#,
        r#"Object.freeze((false || Map))"#,
        r#"Object.freeze(globalThis.Map)"#,
        r#"Object.freeze((globalThis.Map))"#,
        r#"Object.freeze((null ?? globalThis.Map))"#,
        r#"Object.freeze((true && globalThis.Map))"#,
        r#"Object.freeze((false || globalThis.Map))"#,
        r#"Object.freeze(globalThis["Map"])"#,
        r#"Object.freeze((globalThis["Map"]))"#,
        r#"Object.freeze((null ?? globalThis["Map"]))"#,
        r#"Object.freeze((true && globalThis["Map"]))"#,
        r#"Object.freeze((false || globalThis["Map"]))"#,
        r#"Object.freeze(globalThis['Map'])"#,
        r#"Object.freeze((globalThis['Map']))"#,
        r#"Object.freeze((null ?? globalThis['Map']))"#,
        r#"Object.freeze((true && globalThis['Map']))"#,
        r#"Object.freeze((false || globalThis['Map']))"#,
    ]
}

/// Canonical source text for the supported `Map` constructor aliases.
pub fn map_constructor_source() -> String {
    join_semicolon_terminated_segments(map_constructor_aliases())
}

/// Canonical source text for the supported `Map` constructor iteration smoke.
pub fn map_constructor_iteration_source() -> String {
    concat!(
        "for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis.Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis['Map']([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis['Map'](Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis.Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis[\"Map\"])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis['Map'])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((null ?? new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((true && new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((false || new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze(new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }"
    )
    .to_string()
}

/// Canonical source text for the supported `Map` frozen callable aliases.
pub fn map_constructor_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(map_constructor_frozen_callable_aliases())
}

/// Canonical feature-unavailable wording for the supported Node `process.kill(0)` zero-probe slice.
pub fn process_kill_zero_probe_unavailable_message() -> String {
    let aliases = process_kill_zero_probe_aliases();
    format!(
        "process.kill is unavailable unless it is invoked as process.kill(0) or one of its supported Node zero-probe aliases: {}; use the zero liveness-probe subset or the later compatibility path",
        aliases.join(", ")
    )
}

/// Canonical late-process-control source prefix that groups the supported
/// Deno and process-control aliases before the shared zero-probe inventory.
const LATE_PROCESS_CONTROL_PREFIX_SEGMENTS: &[&str] = &[
    "Deno.pid",
    "globalThis.Deno.pid",
    "globalThis[\"Deno\"][\"pid\"]",
    "globalThis[\"Deno\"].cwd",
    "globalThis[\"Deno\"].chdir",
    "globalThis[\"Deno\"].exit",
    "Deno[\"pid\"]",
    "globalThis.Deno[\"pid\"]",
    "globalThis.Deno.cwd",
    "globalThis[\"Deno\"][\"cwd\"]",
    "globalThis.Deno[\"cwd\"]",
    "Deno[\"cwd\"]",
    "Deno.chdir",
    "globalThis.Deno.chdir",
    "globalThis[\"Deno\"][\"chdir\"]",
    "globalThis.Deno[\"chdir\"]",
    "Deno[\"chdir\"]",
    "globalThis.Deno.exit",
    "globalThis[\"Deno\"][\"exit\"]",
    "globalThis.Deno[\"exit\"]",
    "Deno[\"exit\"]",
    "process.pid",
    "globalThis.process.pid",
    "globalThis[\"process\"][\"pid\"]",
    "globalThis[\"process\"].pid",
    "process[\"pid\"]",
    "globalThis.process[\"pid\"]",
    "process.cwd",
    "globalThis.process.cwd",
    "globalThis[\"process\"].cwd",
    "globalThis[\"process\"][\"cwd\"]",
    "process[\"cwd\"]",
    "globalThis.process[\"cwd\"]",
    "process.chdir",
    "globalThis.process.chdir",
    "globalThis[\"process\"].chdir",
    "globalThis[\"process\"][\"chdir\"]",
    "process[\"chdir\"]",
    "globalThis.process[\"chdir\"]",
    "process.kill",
    "globalThis.process.kill",
    "globalThis[\"process\"].kill",
    "globalThis[\"process\"][\"kill\"]",
    "process[\"kill\"]",
    "globalThis.process[\"kill\"]",
    "const zero = 0",
    "const zeroAlias = zero",
    "process.kill(zeroAlias)",
];

const LATE_PROCESS_CONTROL_EXIT_SEGMENTS: &[&str] = &[
    "process.exit",
    "globalThis.process.exit",
    "globalThis.process[\"exit\"]",
    "globalThis[\"process\"].exit",
    "globalThis[\"process\"][\"exit\"]",
    "process[\"exit\"]",
];

/// Canonical exit alias inventory for the shared late-process-control slice.
pub const fn late_process_control_exit_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_EXIT_SEGMENTS
}

/// Canonical late-process-control exit source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_exit_aliases())
}

/// Canonical late-process-control preamble source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_prefix_source() -> String {
    format!(
        "{}; {}",
        join_semicolon_terminated_segments(LATE_PROCESS_CONTROL_PREFIX_SEGMENTS)
            .trim_end_matches(';'),
        late_process_control_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text that embeds the supported Node zero-probe slice.
pub fn late_process_control_source() -> String {
    let process_kill_zero_probe_source = process_kill_zero_probe_alias_inventory_source();
    let parenthesized_receiver_source = process_kill_zero_probe_parenthesized_receiver_source();
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    format!(
        "{} {} {} {}",
        late_process_control_prefix_source(),
        parenthesized_receiver_source.trim_end(),
        parenthesized_receiver_freeze_source.trim_end(),
        process_kill_zero_probe_source.trim_end()
    )
}

const LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS: &[&str] = &[
    r#"globalThis['process'].kill(0)"#,
    r#"globalThis['process'].kill(+0)"#,
    r#"globalThis['process']['kill'](0)"#,
    r#"globalThis['process']['kill'](+0)"#,
    r#"process['kill'](0)"#,
    r#"process['kill'](+0)"#,
    r#"process['kill']((0))"#,
    r#"globalThis.process['kill'](0)"#,
    r#"globalThis.process['kill'](+0)"#,
    r#"globalThis.process['kill']((0))"#,
    r#"globalThis['process'].kill((0))"#,
    r#"globalThis['process']['kill']((0))"#,
    r#"globalThis.process['kill']((0))"#,
    r#"Object.freeze(process['kill'])(0)"#,
    r#"Object.freeze(process['kill'])(+0)"#,
    r#"Object.freeze((process['kill']))(0)"#,
    r#"Object.freeze((process['kill']))(+0)"#,
    r#"Object.freeze(globalThis.process['kill'])(0)"#,
    r#"Object.freeze(globalThis.process['kill'])(+0)"#,
    r#"Object.freeze((globalThis.process['kill']))(0)"#,
    r#"Object.freeze((globalThis.process['kill']))(+0)"#,
    r#"Object.freeze(globalThis['process'].kill)(0)"#,
    r#"Object.freeze(globalThis['process'].kill)(+0)"#,
    r#"Object.freeze((globalThis['process']).kill)(0)"#,
    r#"Object.freeze((globalThis['process']).kill)(+0)"#,
    r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
    r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
    r#"Object.freeze((globalThis['process'].kill))(0)"#,
    r#"Object.freeze((globalThis['process'].kill))(+0)"#,
    r#"Object.freeze((globalThis['process']['kill']))(0)"#,
    r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
    r#"Object.freeze(globalThis['process']['kill'])(0)"#,
    r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
    r#"process['exit'](0)"#,
    r#"process['exit'](+0)"#,
    r#"process['exit']((0))"#,
    r#"Object.freeze(process['exit'])(0)"#,
    r#"Object.freeze(process['exit'])(+0)"#,
    r#"Object.freeze((process['exit']))(0)"#,
    r#"Object.freeze((process['exit']))(+0)"#,
    r#"Object.freeze((process)['exit'])(0)"#,
    r#"Object.freeze((process)['exit'])(+0)"#,
    r#"Object.freeze((globalThis.process)['exit'])(0)"#,
    r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
    r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
    r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
    r#"globalThis['process'].exit(0)"#,
    r#"globalThis['process'].exit(+0)"#,
    r#"globalThis['process'].exit((0))"#,
    r#"globalThis['process']['exit'](0)"#,
    r#"globalThis['process']['exit'](+0)"#,
    r#"globalThis['process']['exit']((0))"#,
    r#"globalThis.process['exit'](0)"#,
    r#"globalThis.process['exit'](+0)"#,
    r#"globalThis.process['exit']((0))"#,
    r#"Object.freeze(globalThis['process'].exit)(0)"#,
    r#"Object.freeze(globalThis['process'].exit)(+0)"#,
    r#"Object.freeze((globalThis['process'].exit))(0)"#,
    r#"Object.freeze((globalThis['process'].exit))(+0)"#,
    r#"Object.freeze(globalThis['process']['exit'])(0)"#,
    r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
    r#"Object.freeze((globalThis['process']['exit']))(0)"#,
    r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
];

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_kill_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[..33]
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_aliases_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_kill_aliases())
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_exit_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[33..]
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_exit_aliases())
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_aliases_source() -> String {
    late_process_control_single_quoted_exit_source()
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_source() -> String {
    late_process_control_single_quoted_kill_aliases_source()
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases_source() -> String {
    format!(
        "{} {}",
        late_process_control_single_quoted_kill_source().trim_end(),
        late_process_control_single_quoted_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_source() -> String {
    format!(
        "{} {}",
        late_process_control_source(),
        late_process_control_single_quoted_process_aliases_source().trim_end()
    )
}

const LATE_PROCESS_ENV_MUTATION_SEGMENTS: &[&str] = &[
    r#"process.env = {}"#,
    r#"process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process.env = {}"#,
    r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"] = {}"#,
    r#"process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"process['env'] = {}"#,
    r#"process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis.process["env"] = {}"#,
    r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis.process['env'] = {}"#,
    r#"globalThis.process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis["process"].env = {}"#,
    r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]["env"] = {}"#,
    r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env'] = {}"#,
    r#"globalThis["process"]['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]['env']["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']["env"] = {}"#,
    r#"globalThis['process']["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"delete globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis['process'].env = {}"#,
    r#"globalThis['process'].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process'].env['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']['env'] = {}"#,
    r#"globalThis['process']['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis['process'].env['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION']"#,
];

/// Canonical late-process-environment-mutation alias inventory used by the browser and runtime smoke.
pub fn late_process_env_mutation_aliases() -> &'static [&'static str] {
    LATE_PROCESS_ENV_MUTATION_SEGMENTS
}

/// Canonical late-process-environment-mutation source text used by the browser and runtime smoke.
pub fn late_process_env_mutation_source() -> String {
    join_semicolon_terminated_segments(late_process_env_mutation_aliases())
}

const BROADER_INTL_SEGMENTS: &[&str] = &[
    "Intl",
    "globalThis.Intl",
    r#"globalThis["Intl"]"#,
    "globalThis['Intl']",
    "globalThis.Intl.NumberFormat",
    r#"globalThis["Intl"].NumberFormat"#,
    r#"globalThis.Intl["NumberFormat"]"#,
    "globalThis.Intl.DateTimeFormat",
    r#"globalThis["Intl"].DateTimeFormat"#,
    r#"globalThis.Intl["DateTimeFormat"]"#,
    r#"globalThis["Intl"]["DateTimeFormat"]"#,
    "globalThis.Intl.PluralRules",
    r#"globalThis["Intl"].PluralRules"#,
    r#"globalThis.Intl["PluralRules"]"#,
    "globalThis.Intl.RelativeTimeFormat",
    r#"globalThis["Intl"].RelativeTimeFormat"#,
    r#"globalThis.Intl["RelativeTimeFormat"]"#,
    "globalThis.Intl.Collator",
    r#"globalThis["Intl"].Collator"#,
    r#"globalThis.Intl["Collator"]"#,
    "globalThis.Intl.DisplayNames",
    r#"globalThis["Intl"].DisplayNames"#,
    r#"globalThis.Intl["DisplayNames"]"#,
    "globalThis.Intl.Segmenter",
    r#"globalThis["Intl"].Segmenter"#,
    r#"globalThis.Intl["Segmenter"]"#,
    "globalThis.Intl.Locale",
    r#"globalThis["Intl"].Locale"#,
    r#"globalThis.Intl["Locale"]"#,
    "globalThis['Intl']['Segmenter']",
    "globalThis['Intl']['NumberFormat']",
    "globalThis['Intl']['DateTimeFormat']",
    "globalThis['Intl']['PluralRules']",
    "globalThis['Intl']['RelativeTimeFormat']",
    "globalThis['Intl']['Collator']",
    "globalThis['Intl']['DisplayNames']",
    "globalThis['Intl']['Locale']",
    "Intl.NumberFormat",
    "Intl.DateTimeFormat",
    "Intl.PluralRules",
    "Intl.RelativeTimeFormat",
    "Intl.Collator",
    "Intl.DisplayNames",
    "Intl.Locale",
];

/// Canonical broader `Intl` aliases used by the browser and runtime smoke.
pub fn broader_intl_aliases() -> &'static [&'static str] {
    BROADER_INTL_SEGMENTS
}

/// Canonical broader `Intl` source text used by the browser and runtime smoke.
pub fn broader_intl_source() -> String {
    join_semicolon_terminated_segments(broader_intl_aliases())
}

const LATE_OBJECT_MODEL_SEGMENTS: &[&str] = &[
    "Proxy",
    "globalThis.Proxy",
    r#"globalThis["Proxy"]"#,
    "globalThis['Proxy']",
    "new Proxy({}, {})",
    "new globalThis.Proxy({}, {})",
    r#"new globalThis["Proxy"]({}, {})"#,
    "new globalThis['Proxy']({}, {})",
    "new WeakMap()",
    "globalThis.WeakMap",
    r#"globalThis["WeakMap"]"#,
    r#"globalThis['WeakMap']"#,
    r#"globalThis["WeakMap"]()"#,
    r#"globalThis['WeakMap']()"#,
    "Object.freeze(new WeakMap())",
    "Object.freeze((new WeakMap()))",
    "Object.freeze(globalThis.WeakMap)",
    "Object.freeze((globalThis.WeakMap))",
    r#"Object.freeze(globalThis["WeakMap"])"#,
    r#"Object.freeze((globalThis["WeakMap"]))"#,
    r#"Object.freeze(globalThis['WeakMap'])"#,
    r#"Object.freeze((globalThis['WeakMap']))"#,
    "new WeakSet()",
    "globalThis.WeakSet",
    r#"globalThis["WeakSet"]"#,
    r#"globalThis['WeakSet']"#,
    r#"globalThis["WeakSet"]()"#,
    r#"globalThis['WeakSet']()"#,
    "Object.freeze(new WeakSet())",
    "Object.freeze((new WeakSet()))",
    "Object.freeze(globalThis.WeakSet)",
    "Object.freeze((globalThis.WeakSet))",
    r#"Object.freeze(globalThis["WeakSet"])"#,
    r#"Object.freeze((globalThis["WeakSet"]))"#,
    r#"Object.freeze(globalThis['WeakSet'])"#,
    r#"Object.freeze((globalThis['WeakSet']))"#,
    "globalThis.WeakRef",
    r#"globalThis["WeakRef"]"#,
    "globalThis['WeakRef']",
    "Object.freeze(globalThis.WeakRef)",
    "Object.freeze((globalThis.WeakRef))",
    r#"Object.freeze(globalThis["WeakRef"])"#,
    r#"Object.freeze((globalThis["WeakRef"]))"#,
    "Object.freeze(globalThis['WeakRef'])",
    "Object.freeze((globalThis['WeakRef']))",
    "new FinalizationRegistry(() => {})",
    "globalThis.FinalizationRegistry",
    r#"globalThis["FinalizationRegistry"](() => {})"#,
    r#"globalThis['FinalizationRegistry'](() => {})"#,
    "Object.freeze(new FinalizationRegistry(() => {}))",
    "Object.freeze((new FinalizationRegistry(() => {})))",
    "Object.freeze(globalThis.FinalizationRegistry)",
    "Object.freeze((globalThis.FinalizationRegistry))",
    r#"Object.freeze(globalThis["FinalizationRegistry"](() => {}))"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"](() => {})))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'](() => {}))"#,
    r#"Object.freeze((globalThis['FinalizationRegistry'](() => {})))"#,
    r#"Object.freeze(globalThis["FinalizationRegistry"])"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'])"#,
    r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
    "Proxy.revocable({}, {})",
    "globalThis.Proxy.revocable({}, {})",
    r#"globalThis["Proxy"]["revocable"]({}, {})"#,
    r#"globalThis['Proxy']['revocable']({}, {})"#,
    r#"globalThis["Proxy"].revocable({}, {})"#,
    r#"globalThis['Proxy'].revocable({}, {})"#,
    r#"globalThis.Proxy["revocable"]({}, {})"#,
    r#"globalThis.Proxy['revocable']({}, {})"#,
    "Object.freeze(Proxy.revocable)({}, {})",
    "Object.freeze((Proxy.revocable))({}, {})",
    "Object.freeze(globalThis.Proxy.revocable)({}, {})",
    "Object.freeze((globalThis.Proxy.revocable))({}, {})",
    r#"Object.freeze(globalThis["Proxy"]["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"]["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy']['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']['revocable']))({}, {})"#,
    r#"Object.freeze(globalThis["Proxy"].revocable)({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"].revocable))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy'].revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']).revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy'].revocable))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy['revocable']))({}, {})"#,
];

/// Canonical alias inventory for the shared late-object-model slice.
pub const fn late_object_model_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_SEGMENTS
}

/// Canonical late-object-model source text used by the browser and runtime smoke.
pub const fn late_object_model_source() -> &'static str {
    r#"Proxy; globalThis.Proxy; globalThis["Proxy"]; globalThis['Proxy']; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); new globalThis['Proxy']({}, {}); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; globalThis['WeakMap']; globalThis["WeakMap"](); globalThis['WeakMap'](); Object.freeze(new WeakMap()); Object.freeze((new WeakMap())); Object.freeze(globalThis.WeakMap); Object.freeze((globalThis.WeakMap)); Object.freeze(globalThis["WeakMap"]); Object.freeze((globalThis["WeakMap"])); Object.freeze(globalThis['WeakMap']); Object.freeze((globalThis['WeakMap'])); new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis['WeakSet']; globalThis["WeakSet"](); globalThis['WeakSet'](); Object.freeze(new WeakSet()); Object.freeze((new WeakSet())); Object.freeze(globalThis.WeakSet); Object.freeze((globalThis.WeakSet)); Object.freeze(globalThis["WeakSet"]); Object.freeze((globalThis["WeakSet"])); Object.freeze(globalThis['WeakSet']); Object.freeze((globalThis['WeakSet'])); globalThis.WeakRef; globalThis["WeakRef"]; globalThis['WeakRef']; Object.freeze(globalThis.WeakRef); Object.freeze((globalThis.WeakRef)); Object.freeze(globalThis["WeakRef"]); Object.freeze((globalThis["WeakRef"])); Object.freeze(globalThis['WeakRef']); Object.freeze((globalThis['WeakRef'])); new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"](() => {}); globalThis['FinalizationRegistry'](() => {}); Object.freeze(new FinalizationRegistry(() => {})); Object.freeze((new FinalizationRegistry(() => {}))); Object.freeze(globalThis.FinalizationRegistry); Object.freeze((globalThis.FinalizationRegistry)); Object.freeze(globalThis["FinalizationRegistry"](() => {})); Object.freeze((globalThis["FinalizationRegistry"](() => {}))); Object.freeze(globalThis['FinalizationRegistry'](() => {})); Object.freeze((globalThis['FinalizationRegistry'](() => {}))); Object.freeze(globalThis["FinalizationRegistry"]); Object.freeze((globalThis["FinalizationRegistry"])); Object.freeze(globalThis['FinalizationRegistry']); Object.freeze((globalThis['FinalizationRegistry'])); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis['Proxy']['revocable']({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis['Proxy'].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); globalThis.Proxy['revocable']({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze(globalThis['Proxy']['revocable'])({}, {}); Object.freeze((globalThis['Proxy']['revocable']))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze((globalThis["Proxy"].revocable))({}, {}); Object.freeze(globalThis['Proxy'].revocable)({}, {}); Object.freeze((globalThis['Proxy']).revocable)({}, {}); Object.freeze((globalThis['Proxy'].revocable))({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {}); Object.freeze((globalThis.Proxy["revocable"]))({}, {}); Object.freeze(globalThis.Proxy['revocable'])({}, {}); Object.freeze((globalThis.Proxy['revocable']))({}, {});"#
}

const LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS: &[&str] = &[
    r#"Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object["hasOwn"](globalThis, "a")"#,
    r#"globalThis["Object"].hasOwn(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwn"](globalThis, "a")"#,
    r#"Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis.Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwnProperty"].call(globalThis, "a")"#,
    r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
];

/// Canonical alias inventory for the shared late-object-model own-property slice.
pub fn late_object_model_own_property_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS
}

/// Canonical late-object-model own-property source text used by the browser and runtime smoke.
pub const fn late_object_model_own_property_source() -> &'static str {
    r#"Object.hasOwn(globalThis, "a"); globalThis.Object.hasOwn(globalThis, "a"); globalThis.Object["hasOwn"](globalThis, "a"); globalThis["Object"].hasOwn(globalThis, "a"); globalThis["Object"]["hasOwn"](globalThis, "a"); Object["hasOwnProperty"].call(globalThis, "a"); globalThis.Object["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]["hasOwnProperty"].call(globalThis, "a"); Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a"); globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a");"#
}

const LATE_THREADED_RUNTIME_SEGMENTS: &[&str] = &[
    "globalThis.SharedArrayBuffer",
    r#"globalThis["SharedArrayBuffer"]"#,
    "globalThis['SharedArrayBuffer']",
    "Object.freeze(globalThis.SharedArrayBuffer)",
    r#"Object.freeze(globalThis["SharedArrayBuffer"])"#,
    "Object.freeze(globalThis['SharedArrayBuffer'])",
    "Object.freeze((globalThis.SharedArrayBuffer))",
    r#"Object.freeze((globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((globalThis['SharedArrayBuffer']))",
    "Object.freeze((null ?? globalThis.SharedArrayBuffer))",
    "Object.freeze((null ?? globalThis['SharedArrayBuffer']))",
    r#"Object.freeze((true && globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((true && globalThis['SharedArrayBuffer']))",
    r#"Object.freeze((false || globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((false || globalThis['SharedArrayBuffer']))",
    "globalThis.Atomics",
    r#"globalThis["Atomics"]"#,
    "globalThis['Atomics']",
    "Object.freeze(globalThis.Atomics)",
    r#"Object.freeze(globalThis["Atomics"])"#,
    "Object.freeze(globalThis['Atomics'])",
    "Object.freeze((globalThis.Atomics))",
    r#"Object.freeze((globalThis["Atomics"]))"#,
    "Object.freeze((globalThis['Atomics']))",
    "Object.freeze((null ?? globalThis.Atomics))",
    "Object.freeze((null ?? globalThis['Atomics']))",
    r#"Object.freeze((true && globalThis["Atomics"]))"#,
    "Object.freeze((true && globalThis['Atomics']))",
    r#"Object.freeze((false || globalThis["Atomics"]))"#,
    "Object.freeze((false || globalThis['Atomics']))",
];

/// Canonical alias inventory for the shared late-threaded-runtime slice.
pub fn late_threaded_runtime_aliases() -> &'static [&'static str] {
    LATE_THREADED_RUNTIME_SEGMENTS
}

/// Canonical late-threaded-runtime source text used by the browser and runtime smoke.
pub const fn late_threaded_runtime_source() -> &'static str {
    "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; Object.freeze(globalThis.SharedArrayBuffer); Object.freeze(globalThis[\"SharedArrayBuffer\"]); Object.freeze(globalThis['SharedArrayBuffer']); Object.freeze((globalThis.SharedArrayBuffer)); Object.freeze((globalThis[\"SharedArrayBuffer\"])); Object.freeze((globalThis['SharedArrayBuffer'])); Object.freeze((null ?? globalThis.SharedArrayBuffer)); Object.freeze((null ?? globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis[\"SharedArrayBuffer\"])); Object.freeze((true && globalThis['SharedArrayBuffer'])); Object.freeze((false || globalThis[\"SharedArrayBuffer\"])); Object.freeze((false || globalThis['SharedArrayBuffer'])); globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics']; Object.freeze(globalThis.Atomics); Object.freeze(globalThis[\"Atomics\"]); Object.freeze(globalThis['Atomics']); Object.freeze((globalThis.Atomics)); Object.freeze((globalThis[\"Atomics\"])); Object.freeze((globalThis['Atomics'])); Object.freeze((null ?? globalThis.Atomics)); Object.freeze((null ?? globalThis['Atomics'])); Object.freeze((true && globalThis[\"Atomics\"])); Object.freeze((true && globalThis['Atomics'])); Object.freeze((false || globalThis[\"Atomics\"])); Object.freeze((false || globalThis['Atomics']));"
}

const LATE_PERMISSION_ESCALATION_SEGMENTS: &[&str] = &[
    "Deno.permissions.request()",
    "Deno.permissions.revoke()",
    r#"Deno.permissions["request"]()"#,
    r#"Deno.permissions["revoke"]()"#,
    "globalThis.Deno.permissions.request()",
    "globalThis.Deno.permissions.revoke()",
    r#"globalThis.Deno.permissions["request"]()"#,
    r#"globalThis.Deno.permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions.request()"#,
    r#"globalThis["Deno"].permissions.revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
    r#"globalThis["Deno"]["permissions"].request()"#,
    r#"globalThis["Deno"]["permissions"].revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis.Deno["permissions"]["request"]()"#,
    r#"globalThis.Deno["permissions"]["revoke"]()"#,
];

/// Canonical late permission-escalation alias inventory used by the browser and runtime smoke.
pub fn late_permission_escalation_aliases() -> &'static [&'static str] {
    LATE_PERMISSION_ESCALATION_SEGMENTS
}

/// Canonical late permission-escalation source text used by the browser and runtime smoke.
pub fn late_permission_escalation_source() -> String {
    join_semicolon_terminated_segments(late_permission_escalation_aliases())
}

/// Canonical late environment-materialization source text used by the browser and runtime smoke.
pub const fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis[\"Deno\"].env.toObject();"
}

/// Canonical late subprocess source text used by the browser and runtime smoke.
pub const fn late_subprocess_source() -> &'static str {
    "new Deno.Command('sh').spawn(); new Deno[\"Command\"]('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno[\"Command\"]('sh').spawn(); new globalThis[\"Deno\"].Command('sh').spawn(); new globalThis[\"Deno\"][\"Command\"]('sh').spawn();"
}

/// Canonical late network source text used by the browser and runtime smoke.
pub const fn late_network_source() -> &'static str {
    "Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno[\"connect\"]('127.0.0.1', 1); globalThis[\"Deno\"].connect('127.0.0.1', 1); globalThis[\"Deno\"][\"connect\"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno[\"listen\"]('127.0.0.1', 0); globalThis[\"Deno\"].listen('127.0.0.1', 0); globalThis[\"Deno\"][\"listen\"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno[\"serve\"]('127.0.0.1', 0); globalThis[\"Deno\"].serve('127.0.0.1', 0); globalThis[\"Deno\"][\"serve\"]('127.0.0.1', 0);"
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
