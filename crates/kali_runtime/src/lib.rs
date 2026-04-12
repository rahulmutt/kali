//! Runtime execution for Kali-generated WASM modules.

use kali_error::{Diagnostic, _error_codes::e4};
use kali_sandbox::SandboxPolicy;
use wasmtime::{Engine, Linker, Module, Store};

/// Runtime context.
#[derive(Clone, Debug, Default)]
pub struct RuntimeCtx {
    /// Sandbox policy.
    pub policy: Option<SandboxPolicy>,
}

/// Host-side state owned by the runtime.
#[derive(Clone, Debug, Default)]
pub struct KaliHostState {
    /// Sandbox policy.
    pub policy: Option<SandboxPolicy>,
}

/// Result of executing a WASM module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOutcome {
    /// Process exit code.
    pub exit_code: i32,
}

impl RuntimeCtx {
    pub fn new(policy: Option<SandboxPolicy>) -> Self {
        Self { policy }
    }

    /// Execute a WASM module.
    pub fn execute(&self, wasm_bytes: &[u8]) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, wasm_bytes).map_err(|error| {
            vec![Diagnostic::error(
                e4::IO_ERROR as u32,
                format!("failed to load WASM module: {}", error),
            )]
        })?;

        let mut store = Store::new(
            &engine,
            KaliHostState {
                policy: self.policy.clone(),
            },
        );
        let linker = Linker::new(&engine);

        let instance = linker.instantiate(&mut store, &module).map_err(|error| {
            vec![Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("failed to instantiate WASM module: {}", error),
            )]
        })?;

        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|error| {
                vec![Diagnostic::error(
                    e4::UNCAUGHT_ERROR as u32,
                    format!("missing _start export: {}", error),
                )]
            })?;

        start.call(&mut store, ()).map_err(|error| {
            vec![Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("runtime trap: {}", error),
            )]
        })?;

        Ok(RuntimeOutcome { exit_code: 0 })
    }
}
