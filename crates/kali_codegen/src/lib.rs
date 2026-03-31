//! WASM code generation for the Kali compiler.

use kali_lir::LirNodeId;
use kali_error::Diagnostic;
use serde::{Deserialize, Serialize};

/// WASM code generator context.
pub struct CodegenCtx {
    /// Target configuration.
    pub target: TargetConfig,
}

impl CodegenCtx {
    pub fn new(target: TargetConfig) -> Self {
        Self { target }
    }
}

/// Target configuration for code generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Whether to enable optimization passes.
    pub optimize: bool,
}

/// Code generation result containing the WASM output.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodegenResult {
    /// WASM bytes.
    pub wasm_bytes: Vec<u8>,
    /// Diagnostics collected during codegen.
    pub diagnostics: Vec<Diagnostic>,
}

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(_ctx: &mut CodegenCtx, _lir: LirNodeId) -> CodegenResult {
    CodegenResult {
        wasm_bytes: Vec::new(),
        diagnostics: Vec::new(),
    }
}
