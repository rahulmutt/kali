//! Context, configuration, result, and string-pool types for kali_codegen.

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticObjectIdentityValue {
    Boolean(bool),
    Number(f64),
    String(String),
    BigInt(i64),
    Null,
    Undefined,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticArraySearchResult {
    Value(LirNodeId),
    Index(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticArrayAtResult {
    Value(LirNodeId),
    OutOfRange,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticStringAtResult {
    Value(String),
    OutOfRange,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticIndexMemberResult {
    Node(LirNodeId),
    String(String),
    Undefined,
}

impl StaticObjectIdentityValue {
    pub(crate) fn same_value(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::BigInt(left), Self::BigInt(right)) => left == right,
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Number(left), Self::Number(right)) => {
                (left.is_nan() && right.is_nan())
                    || (left == right
                        && (left != &0.0 || left.is_sign_positive() == right.is_sign_positive()))
            }
            _ => false,
        }
    }

    pub(crate) fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::BigInt(left), Self::BigInt(right)) => left == right,
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Number(left), Self::Number(right)) => {
                !left.is_nan() && !right.is_nan() && left == right
            }
            _ => false,
        }
    }

    pub(crate) fn same_value_zero(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => {
                (left.is_nan() && right.is_nan()) || left == right
            }
            _ => self.strict_eq(other),
        }
    }

    pub(crate) fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    pub(crate) fn truthiness(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            Self::Number(value) => Some(!value.is_nan() && *value != 0.0),
            Self::String(value) => Some(!value.is_empty()),
            Self::BigInt(value) => Some(*value != 0),
            Self::Null | Self::Undefined => Some(false),
        }
    }
}

/// WASM code generator context.
pub struct CodegenCtx {
    /// Target configuration.
    pub target: TargetConfig,
    /// Source file path for context-sensitive static lowering.
    pub source_path: Option<PathBuf>,
    /// Integer-vs-float representation decisions from the resolver. Empty (all
    /// `I64`) by default, keeping the integer fast path byte-identical.
    pub repr_table: kali_common::ReprTable,
    /// Arena placement decisions from the `kali_mir` escape gate. Empty by
    /// default; misses fail closed (global allocation / no arena). Read by
    /// Tasks 6/7 — no codegen behavior depends on it yet.
    pub arena_table: kali_common::ArenaTable,
    /// Per-function closure environment plans (Stage C, `kali_mir::derive_env_plans`):
    /// the promoted env cells a function owns and the outer captures it reads
    /// through the parent chain, keyed by the function's declared / `__kali_fn_N`
    /// name (module root is `""`). Empty by default — an entry absence means a
    /// function owns no env and captures nothing, so integer/closure-free
    /// programs are byte-identical.
    pub env_plans: std::collections::BTreeMap<String, kali_mir::EnvPlan>,
}

impl CodegenCtx {
    pub fn new(target: TargetConfig) -> Self {
        Self {
            target,
            source_path: None,
            repr_table: kali_common::ReprTable::default(),
            arena_table: kali_common::ArenaTable::default(),
            env_plans: std::collections::BTreeMap::new(),
        }
    }
}

/// Target configuration for code generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Upper bound on specialization fan-out.
    pub max_specializations: usize,
    /// Whether compatibility eval source stubs were pre-resolved earlier in the pipeline.
    pub compat_eval: bool,
    /// Whether coverage instrumentation is enabled for this compilation.
    pub coverage: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        }
    }
}

/// Code generation result containing the WASM output.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodegenResult {
    /// WASM bytes.
    pub wasm_bytes: Vec<u8>,
    /// Diagnostics collected during codegen.
    pub diagnostics: Vec<Diagnostic>,
}

/// Decode the recognized single-character string escapes into their bytes.
/// The lexer has already rejected unrecognized escapes, so an unknown `\x`
/// sequence here is passed through verbatim (best-effort, never a panic).
pub(crate) fn decode_string_escapes(text: &str) -> String {
    if !text.contains('\\') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('`') => out.push('`'),
            Some('0') => out.push('\0'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('v') => out.push('\u{000B}'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

pub(crate) struct StringPool {
    pub(crate) entries: Vec<(u32, String)>,
    pub(crate) offsets: BTreeMap<String, u32>,
    pub(crate) next_offset: u32,
    /// fasta Spec 7 Task 4g: per-shape for-in KEY TABLES emitted as
    /// module-constant data (raw `i64` handle blobs), not bump-allocated per
    /// for-in execution. Each entry is `(base, little-endian bytes)` and is
    /// written into the data segment at finalize alongside `entries`. They ride
    /// the SAME `next_offset` address space as interned strings, so each table's
    /// base is a compile-time constant known at emit time (exactly how a string
    /// constant gets its offset). `key_table_offsets` dedups by shape across the
    /// whole module (one table per distinct shape, shared by every for-in site
    /// and every monomorphized twin over that shape).
    pub(crate) key_table_entries: Vec<(u32, Vec<u8>)>,
    pub(crate) key_table_offsets: BTreeMap<kali_common::ShapeId, u32>,
}

impl StringPool {
    pub(crate) fn new(reserved_prefix: u32) -> Self {
        Self {
            entries: Vec::new(),
            offsets: BTreeMap::new(),
            next_offset: reserved_prefix,
            key_table_entries: Vec::new(),
            key_table_offsets: BTreeMap::new(),
        }
    }

    pub(crate) fn intern(&mut self, text: &str) -> (u32, u32) {
        let text = decode_string_escapes(text);
        if let Some(&offset) = self.offsets.get(&text) {
            return (offset, text.len() as u32);
        }

        let offset = self.next_offset;
        let len = text.len() as u32;
        self.entries.push((offset, text.clone()));
        self.offsets.insert(text, offset);
        self.next_offset = self.next_offset.saturating_add(len);
        (offset, len)
    }

    /// Intern a for-in key handle table (fasta Spec 7 Task 4g): a per-shape,
    /// compile-time-constant array of `i64` string handles (slot `j` = the
    /// interned handle of the shape's `j`th field name). Returns the table's
    /// base offset — an 8-aligned data-segment address the read site references
    /// as an `i32.const`, so the table costs ZERO runtime allocation. Deduped
    /// by shape: the second for-in over the same shape reuses the first base.
    ///
    /// The blob is serialized little-endian to match wasm linear-memory byte
    /// order, so an `i64.load` at `base + ord*8` observes exactly the handle
    /// value the old per-execution `i64.store` bump produced. The base is
    /// 8-aligned (the bump allocator this replaces always returned 8-aligned
    /// addresses); the ≤7 padding bytes before it stay zero (never emitted).
    pub(crate) fn intern_key_table(&mut self, shape: kali_common::ShapeId, handles: &[i64]) -> u32 {
        if let Some(&base) = self.key_table_offsets.get(&shape) {
            return base;
        }
        let base = (self.next_offset + 7) & !7;
        let mut bytes = Vec::with_capacity(handles.len() * 8);
        for &handle in handles {
            bytes.extend_from_slice(&handle.to_le_bytes());
        }
        let size = bytes.len() as u32;
        self.key_table_entries.push((base, bytes));
        self.key_table_offsets.insert(shape, base);
        self.next_offset = base.saturating_add(size);
        base
    }
}

#[cfg(test)]
#[path = "ctx_tests.rs"]
mod ctx_tests;
