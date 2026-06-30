use super::*;

#[test]
fn compiler_rejects_threaded_runtime_profiles_in_the_current_phase() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let config = CompilerConfig {
        runtime_profiles: vec!["wasm-threads".to_string()],
        ..CompilerConfig::default()
    };
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected E5506 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_duplicate_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let config = CompilerConfig {
        runtime_profiles: vec!["wasm-threads".to_string(), "wasm-threads".to_string()],
        ..CompilerConfig::default()
    };
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5509 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_unknown_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let config = CompilerConfig {
        runtime_profiles: vec!["fiber-threads".to_string()],
        ..CompilerConfig::default()
    };
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5509 diagnostic: {error}"
    );
}
