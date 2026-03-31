//! Runtime execution for Kali-generated WASM modules.

/// Runtime context.
pub struct RuntimeCtx {
    /// Sandbox policy.
    pub policy: Option<kali_sandbox::SandboxPolicy>,
}

impl RuntimeCtx {
    pub fn new(_policy: Option<kali_sandbox::SandboxPolicy>) -> Self {
        Self { policy }
    }

    /// Execute a WASM module.
    pub fn execute(
        &self,
        _wasm_bytes: &[u8],
    ) -> Result<(), ()> {
        Ok(())
    }
}
