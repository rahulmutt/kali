//! WASM code generation for the Kali compiler.

use kali_lir::LirNodeId;
use serde::{Deserialize, Serialize};

/// Code generation results.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodegenResult {
    /// WASM bytes.
    pub wasm_bytes: Vec<u8>,
    /// Diagnostics.
    pub diagnostics: Vec<kali_error::diagnostic::Diagnostic>,
}

/// WASM code generator.
pub struct CodegenCtx {
    /// Target configuration.
    pub target: TargetConfig,
}

impl CodegenCtx {
    pub fn new(_target: TargetConfig) -> Self {
        Self { target }
    }
}

/// Target configuration.
#[derive(Clone, Debug)]
pub struct TargetConfig {
    pub optimize: bool,
}

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, _lir: LirNodeId) -> CodegenResult {
    CodegenResult {
        wasm_bytes: Vec::new(),
        diagnostics: Vec::new(),
    }
}
