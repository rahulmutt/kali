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
}

impl CodegenCtx {
    pub fn new(target: TargetConfig) -> Self {
        Self {
            target,
            source_path: None,
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

pub(crate) struct StringPool {
    pub(crate) entries: Vec<(u32, String)>,
    pub(crate) offsets: BTreeMap<String, u32>,
    pub(crate) next_offset: u32,
}

impl StringPool {
    pub(crate) fn new(reserved_prefix: u32) -> Self {
        Self {
            entries: Vec::new(),
            offsets: BTreeMap::new(),
            next_offset: reserved_prefix,
        }
    }

    pub(crate) fn intern(&mut self, text: &str) -> (u32, u32) {
        if let Some(&offset) = self.offsets.get(text) {
            return (offset, text.len() as u32);
        }

        let offset = self.next_offset;
        let len = text.len() as u32;
        self.entries.push((offset, text.to_owned()));
        self.offsets.insert(text.to_owned(), offset);
        self.next_offset = self.next_offset.saturating_add(len);
        (offset, len)
    }
}
