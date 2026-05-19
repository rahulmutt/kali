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

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source() -> String {
    join_semicolon_terminated_segments(
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases(),
    )
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source() -> String {
    format!(
        "{} {}",
        process_kill_zero_probe_parenthesized_receiver_freeze_source().trim_end(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source().trim_end()
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

fn join_wrapped_zero_probe_call_targets(argument_source: &str) -> String {
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
    join_wrapped_zero_probe_call_targets("((0 satisfies number))")
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe slices wrapped in a TS type assertion.
pub fn process_kill_zero_probe_type_assertion_source() -> String {
    join_wrapped_zero_probe_call_targets("((0 as number))")
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

/// Canonical source text for the supported Node zero-probe sequence-callable-target bindings.
pub fn process_kill_zero_probe_sequence_call_target_bindings_source() -> String {
    join_semicolon_terminated_segments(&[
        r#"const sequenceKill = (process.kill, process.kill)"#,
        r#"const bracketedRootSequenceKill = (process["kill"], process["kill"])"#,
        r#"const dotRootSequenceKill = (globalThis.process.kill, globalThis.process.kill)"#,
        r#"const bracketedSequenceKill = (globalThis["process"]["kill"], globalThis["process"]["kill"])"#,
        r#"const dotBracketSequenceKill = (globalThis.process["kill"], globalThis.process["kill"])"#,
        r#"const bracketedDotSequenceKill = (globalThis["process"].kill, globalThis["process"].kill)"#,
        r#"const fullyBracketedSequenceKill = (globalThis["process"]["kill"], globalThis["process"]["kill"])"#,
    ])
}

/// Canonical source text for the supported Node `process.kill(0)` direct call-target bindings.
pub fn process_kill_zero_probe_call_target_bindings_source() -> String {
    join_semicolon_terminated_segments(&[
        r#"const kill = process.kill"#,
        r#"const bracketedRootKill = process["kill"]"#,
        r#"const dotRootKill = globalThis.process.kill"#,
        r#"const bracketedDotKill = globalThis["process"].kill"#,
        r#"const dotBracketKill = globalThis.process["kill"]"#,
        r#"const fullyBracketedKill = globalThis["process"]["kill"]"#,
    ])
}

/// Canonical `console.log(...)` source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_console_log_source() -> String {
    let statements = process_kill_zero_probe_aliases()
        .iter()
        .map(|alias| format!("console.log({alias})"))
        .collect::<Vec<_>>();
    format!("{};", statements.join("; "))
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
        r#"Object.freeze(globalThis.Object.hasOwn)"#,
        r#"Object.freeze((globalThis.Object.hasOwn))"#,
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwn)"#,
        r#"Object.freeze((globalThis["Object"].hasOwn))"#,
        r#"Object.freeze(globalThis["Object"]["hasOwn"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwn"]))"#,
    ]
}

/// Canonical source text for the supported `Object.hasOwn` frozen callable aliases.
pub fn object_has_own_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_has_own_frozen_callable_aliases())
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
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))"#,
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
        format!(r#"globalThis.Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!("Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!("globalThis.Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
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
        "const alias = {alias_literal}; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger;"
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

/// Canonical frozen callable aliases for the supported `Math.floor` / `Math.trunc` / `Math.ceil` helper slice.
pub const fn math_floor_trunc_ceil_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math.floor)"#,
        r#"Object.freeze((globalThis.Math.floor))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze(globalThis["Math"].floor)"#,
        r#"Object.freeze((globalThis["Math"].floor))"#,
        r#"Object.freeze(Math["floor"])"#,
        r#"Object.freeze((Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math.trunc)"#,
        r#"Object.freeze((globalThis.Math.trunc))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze(globalThis["Math"].trunc)"#,
        r#"Object.freeze((globalThis["Math"].trunc))"#,
        r#"Object.freeze(Math["trunc"])"#,
        r#"Object.freeze((Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis.Math.ceil)"#,
        r#"Object.freeze((globalThis.Math.ceil))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
        r#"Object.freeze(globalThis["Math"].ceil)"#,
        r#"Object.freeze((globalThis["Math"].ceil))"#,
        r#"Object.freeze(Math["ceil"])"#,
        r#"Object.freeze((Math["ceil"]))"#,
    ]
}

/// Canonical source text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_floor_trunc_ceil_frozen_callable_aliases())
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
        r#"globalThis["Math"].pow"#,
        r#"globalThis["Math"]["pow"]"#,
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

/// Canonical browser source text for the supported `Math.pow` alias inventory.
pub fn math_pow_browser_alias_inventory_source() -> String {
    format!(
        "{} {}",
        math_pow_alias_inventory_source().trim_end(),
        math_pow_bracketed_frozen_callable_source().trim_end()
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
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
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
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze((globalThis['Math'].pow))"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
        r#"Object.freeze((Math.pow))"#,
        r#"Object.freeze((Math['pow']))"#,
        r#"Object.freeze((Math["pow"]))"#,
    ]
}

/// Canonical bracketed-root frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_bracketed_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
    ]
}

/// Canonical source text for the supported `Math.pow` bracketed-root frozen callable aliases.
pub fn math_pow_bracketed_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_pow_bracketed_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Math.pow` helper slice.
pub fn math_pow_frozen_callable_aliases() -> Vec<&'static str> {
    ordered_unique_union(&[
        math_pow_frozen_callable_direct_aliases(),
        math_pow_frozen_callable_parenthesized_aliases(),
    ])
}

/// Canonical source text for the supported `Math.pow` frozen callable aliases.
pub fn math_pow_frozen_callable_source() -> String {
    let aliases = math_pow_frozen_callable_aliases();
    join_semicolon_terminated_segments(&aliases)
}

/// Canonical aliases for the supported `Array.from` helper slice.
pub const fn array_from_aliases() -> &'static [&'static str] {
    &[
        "Array.from",
        "globalThis.Array.from",
        r#"globalThis["Array"].from"#,
        r#"globalThis["Array"]["from"]"#,
        r#"globalThis['Array'].from"#,
        r#"globalThis['Array']['from']"#,
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
        r#"Object.freeze(globalThis["Array"]["from"])"#,
        r#"Object.freeze((globalThis["Array"]["from"]))"#,
    ]
}

/// Canonical source text for the supported `Array.from` frozen callable aliases.
pub fn array_from_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(array_from_frozen_callable_aliases())
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
        r#"Object.freeze(globalThis.Set)"#,
        r#"Object.freeze((globalThis.Set))"#,
        r#"Object.freeze(globalThis["Set"])"#,
        r#"Object.freeze((globalThis["Set"]))"#,
        r#"Object.freeze(globalThis['Set'])"#,
        r#"Object.freeze((globalThis['Set']))"#,
    ]
}

/// Canonical source text for the supported `Set` constructor aliases.
pub fn set_constructor_source() -> String {
    join_semicolon_terminated_segments(set_constructor_aliases())
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
        r#"Object.freeze(globalThis.Map)"#,
        r#"Object.freeze((globalThis.Map))"#,
        r#"Object.freeze(globalThis["Map"])"#,
        r#"Object.freeze((globalThis["Map"]))"#,
        r#"Object.freeze(globalThis['Map'])"#,
        r#"Object.freeze((globalThis['Map']))"#,
    ]
}

/// Canonical source text for the supported `Map` constructor aliases.
pub fn map_constructor_source() -> String {
    join_semicolon_terminated_segments(map_constructor_aliases())
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
    "process.exit",
    "globalThis.process.exit",
    "globalThis[\"process\"].exit",
    "globalThis[\"process\"][\"exit\"]",
    "process[\"exit\"]",
    "globalThis.process[\"exit\"]",
];

/// Canonical late-process-control preamble source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_prefix_source() -> String {
    join_semicolon_terminated_segments(LATE_PROCESS_CONTROL_PREFIX_SEGMENTS)
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

const LATE_PROCESS_ENV_MUTATION_SEGMENTS: &[&str] = &[
    r#"process.env = {}"#,
    r#"process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process.env = {}"#,
    r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"] = {}"#,
    r#"process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis.process["env"] = {}"#,
    r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"].env = {}"#,
    r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]["env"] = {}"#,
    r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"delete process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
];

/// Canonical late-process-environment-mutation source text used by the browser and runtime smoke.
pub fn late_process_env_mutation_source() -> String {
    join_semicolon_terminated_segments(LATE_PROCESS_ENV_MUTATION_SEGMENTS)
}

/// Canonical broader `Intl` source text used by the browser and runtime smoke.
pub const fn broader_intl_source() -> &'static str {
    r#"Intl; globalThis.Intl; globalThis["Intl"]; globalThis.Intl.NumberFormat; globalThis["Intl"].NumberFormat; globalThis.Intl["NumberFormat"]; globalThis.Intl.DateTimeFormat; globalThis["Intl"].DateTimeFormat; globalThis.Intl["DateTimeFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis.Intl.PluralRules; globalThis["Intl"].PluralRules; globalThis.Intl["PluralRules"]; globalThis.Intl.RelativeTimeFormat; globalThis["Intl"].RelativeTimeFormat; globalThis.Intl["RelativeTimeFormat"]; globalThis.Intl.Collator; globalThis["Intl"].Collator; globalThis.Intl["Collator"]; globalThis.Intl.DisplayNames; globalThis["Intl"].DisplayNames; globalThis.Intl["DisplayNames"]; globalThis.Intl.Segmenter; globalThis["Intl"].Segmenter; globalThis.Intl["Segmenter"]; globalThis.Intl.Locale; globalThis["Intl"].Locale; globalThis.Intl["Locale"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["NumberFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Locale"]; Intl.NumberFormat; Intl.DateTimeFormat; Intl.PluralRules; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Locale;"#
}

/// Canonical late-object-model source text used by the browser and runtime smoke.
pub const fn late_object_model_source() -> &'static str {
    r#"Proxy; globalThis.Proxy; globalThis["Proxy"]; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"](); new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"](); globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"](() => {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {});"#
}

/// Canonical late-threaded-runtime source text used by the browser and runtime smoke.
pub const fn late_threaded_runtime_source() -> &'static str {
    "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis.Atomics; globalThis[\"Atomics\"];"
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
