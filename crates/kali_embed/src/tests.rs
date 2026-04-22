use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn compiles_standalone_artifacts_in_memory() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler.compile_file(&source_path).expect("compile file");

    assert!(!artifact.wasm_bytes().is_empty());
    assert_eq!(artifact.metadata().artifact_kind, "executable");
    assert_eq!(
        artifact.metadata().entrypoint,
        source_path.display().to_string()
    );
}

#[test]
fn compiles_library_artifacts_with_wit_sidecars() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler.compile_lib(&source_path).expect("compile lib");

    assert!(!artifact.wasm_bytes().is_empty());
    assert!(artifact.wit().contains("package kali:embed;"));
    assert!(artifact.wit().contains("export add: func();"));
    assert_eq!(artifact.metadata().artifact_kind, "lib");
    assert_eq!(artifact.metadata().exports.as_ref().unwrap().len(), 1);
}

#[test]
fn compile_lib_reports_missing_export_surfaces() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "const value = 1;").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let error = compiler
        .compile_lib(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_EXPORT_SURFACE as u32)),
        "expected E5011 diagnostic: {error}"
    );
}

#[test]
fn temporary_source_paths_are_unique_across_calls() {
    let first = temporary_source_path("first-module");
    let second = temporary_source_path("second/module");

    assert_ne!(first, second);
    assert!(first.display().to_string().contains("kali-embed-"));
    assert!(second.display().to_string().contains("kali-embed-"));
    assert!(first.display().to_string().contains("first-module"));
    assert!(second.display().to_string().contains("second_module"));
}

#[test]
fn compile_lib_from_raw_source_uses_a_stable_module_name() {
    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler
        .compile_lib_source(
            "math/embedded",
            "export function add(a, b) { return a + b; }",
        )
        .expect("compile lib source");

    assert!(!artifact.wasm_bytes().is_empty());
    assert_eq!(artifact.metadata().artifact_kind, "lib");
    assert_eq!(artifact.metadata().entrypoint, "math/embedded");
    assert!(artifact.wit().contains("// module: math/embedded"));
    assert!(artifact.wit().contains("export add: func();"));
}

#[test]
fn embedding_context_uses_the_stable_compiler_api() {
    let ctx = EmbeddingCtx::new();
    let wasm_bytes = ctx.build_library("export function add(a, b) { return a + b; }");

    assert!(!wasm_bytes.is_empty());
}

#[test]
fn embedding_layer_reexports_the_host_predicate_context() {
    let operation = kali_sandbox::HostOperation::Console;
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "effects.console");
    assert_eq!(context.subject, "stdout");
    assert_eq!(context.operation, operation);
    assert!(context.details.is_empty());

    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "deny-stdout", |ctx| {
        ctx.subject != "stdout"
    });
}

#[test]
fn embedding_layer_reexports_thread_spawn_context_details() {
    let operation = kali_sandbox::HostOperation::ThreadSpawn { active_threads: 5 };
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "resources.maxThreads");
    assert_eq!(context.subject, "5");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("activeThreads").map(String::as_str),
        Some("5")
    );
}

#[test]
fn compiler_rejects_threaded_runtime_profiles_in_the_current_phase() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["wasm-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected E5006 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_duplicate_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["wasm-threads".to_string(), "wasm-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5009 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_unknown_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["fiber-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5009 diagnostic: {error}"
    );
}
