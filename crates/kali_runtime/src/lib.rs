//! Runtime execution for Kali-generated WASM modules.

use kali_error::{Diagnostic, _error_codes::e4};
use kali_sandbox::SandboxPolicy;
use std::io::Write;
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
        let mut linker = Linker::new(&engine);
        register_default_host_imports(&mut linker).map_err(|diagnostic| vec![diagnostic])?;

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

fn register_default_host_imports(linker: &mut Linker<KaliHostState>) -> Result<(), Diagnostic> {
    linker
        .func_wrap("kali:rt", "console_log", |val: i64| {
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{}", format_tagged_val(val));
        })
        .map_err(|error| host_import_error("console_log", error))?;

    linker
        .func_wrap("kali:rt", "console_error", |val: i64| {
            let mut stderr = std::io::stderr().lock();
            let _ = writeln!(stderr, "{}", format_tagged_val(val));
        })
        .map_err(|error| host_import_error("console_error", error))?;

    linker
        .func_wrap("kali:rt", "console_warn", |val: i64| {
            let mut stderr = std::io::stderr().lock();
            let _ = writeln!(stderr, "[warn] {}", format_tagged_val(val));
        })
        .map_err(|error| host_import_error("console_warn", error))?;

    Ok(())
}

fn format_tagged_val(value: i64) -> String {
    value.to_string()
}

fn host_import_error(name: &str, error: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::error(
        e4::UNCAUGHT_ERROR as u32,
        format!("failed to register host import '{}': {}", name, error),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{
        CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
        ImportSection, Instruction, Module, TypeSection, ValType,
    };

    fn module_with_console_imports() -> Vec<u8> {
        let mut module = Module::new();

        let mut types = TypeSection::new();
        types.ty().function(vec![ValType::I64], Vec::new());
        types.ty().function(Vec::new(), Vec::new());
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("kali:rt", "console_log", EntityType::Function(0));
        imports.import("kali:rt", "console_error", EntityType::Function(0));
        imports.import("kali:rt", "console_warn", EntityType::Function(0));
        module.section(&imports);

        let mut functions = FunctionSection::new();
        functions.function(1);
        module.section(&functions);

        let mut exports = ExportSection::new();
        exports.export("_start", ExportKind::Func, 3);
        module.section(&exports);

        let mut code = CodeSection::new();
        let mut body = Function::new(Vec::new());
        body.instruction(&Instruction::I64Const(1));
        body.instruction(&Instruction::Call(0));
        body.instruction(&Instruction::I64Const(2));
        body.instruction(&Instruction::Call(1));
        body.instruction(&Instruction::I64Const(3));
        body.instruction(&Instruction::Call(2));
        body.instruction(&Instruction::End);
        code.function(&body);
        module.section(&code);

        module.finish()
    }

    #[test]
    fn runtime_executes_modules_with_console_host_imports() {
        let runtime = RuntimeCtx::default();
        let wasm = module_with_console_imports();

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }
}
