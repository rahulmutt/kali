use super::*;

#[test]
fn unresolved_identifier_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_value.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_value")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-identifier diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unresolved_call_target_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[10].text = Some("missing_fn".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_fn.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_fn")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-call diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn duplicate_unresolved_identifier_lowering_reports_one_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());
    program.nodes[8].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let matching_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic.message.contains("missing_value")
        })
        .count();
    assert_eq!(
        matching_diagnostics, 1,
        "expected the repeated unresolved identifier to be reported once: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn source_path_in_temp_dir_attaches_to_unresolved_identifier_diagnostics() {
    // Use kali_test_support::fixtures to create a real temp directory with a
    // source file alongside a package.json; verify the source_path is threaded
    // through to diagnostics so downstream tooling knows where the file lives.
    let dir = tempdir();
    let src = write_file(dir.path(), "index.ts", "missing_var;");
    let _pkg = write_file(
        dir.path(),
        "package.json",
        r#"{"name":"smoke","version":"1.2.3"}"#,
    );

    let program = parse_and_lower_lir("missing_var;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(src.clone());
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // The unresolved identifier should produce a warning that embeds the
    // source path from the temp directory.
    let src_str = src.to_string_lossy();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| { d.notes.iter().any(|note| note.contains(src_str.as_ref())) }),
        "expected a diagnostic containing the tempdir source path {:?}; got: {:?}",
        src,
        result.diagnostics
    );
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
