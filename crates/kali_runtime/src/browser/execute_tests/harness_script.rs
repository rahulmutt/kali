use super::*;

#[test]
fn browser_runtime_harness_script_executes_wasm_and_bridges_console_output() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (import "kali:rt" "console_error" (func $console_error (param i64)))
                (func (export "_start")
                    call $args_len
                    i64.extend_i32_s
                    call $console_log
                    call $args_len
                    i64.extend_i32_s
                    call $console_error))
        "#,
    );
    let script =
        browser_runtime_harness_script(&wasm, &["alpha".to_string(), "beta".to_string()], false);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path =
        kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime.mjs", &script);

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert!(outcome.stdout.contains('2'), "stdout: {}", outcome.stdout);
    assert!(outcome.stderr.contains('2'), "stderr: {}", outcome.stderr);
}

#[test]
fn browser_runtime_harness_script_executes_registered_callbacks_and_reports_zero_failures() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (func (export "__kali_callback_7")
                    i64.const 11
                    call $console_log)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["gamma".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-tests.mjs",
        &script,
    );

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[\"7\"]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(outcome.stdout.contains("11"), "stdout: {}", outcome.stdout);
    assert!(
        outcome.stdout.contains("\"args\":[\"gamma\"]"),
        "stdout: {}",
        outcome.stdout
    );
}

#[test]
fn browser_runtime_harness_script_reports_failed_callbacks_and_nonzero_exit() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["delta".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-tests-failed.mjs",
        &script,
    );

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_ne!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[\"7\"]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stderr.contains("browser runtime test failures: 1"),
        "stderr: {}",
        outcome.stderr
    );
}

#[test]
fn browser_runtime_harness_script_reports_an_empty_test_summary_when_no_callbacks_register() {
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["epsilon".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-empty-tests.mjs",
        &script,
    );

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"args\":[\"epsilon\"]"),
        "stdout: {}",
        outcome.stdout
    );
}

#[test]
fn browser_runtime_harness_summary_file_capture_is_deterministic() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (func (export "__kali_callback_7")
                    i64.const 11
                    call $console_log)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["zeta".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary-file.mjs",
        &script,
    );
    let summary_path = tempdir.path().join("browser-runtime-summary.json");

    let outcome = browser_harness_run_checked_with_env(
        Some("node"),
        &script_path,
        &[],
        tempdir.path(),
        &[(
            crate::BROWSER_HARNESS_SUMMARY_FILE_ENV,
            summary_path.as_os_str(),
        )],
    )
    .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        !outcome.stdout.contains("\"tests\":"),
        "stdout: {}",
        outcome.stdout
    );
    let summary = fs::read_to_string(&summary_path).expect("summary file");
    assert!(
        summary.contains("\"args\":[\"zeta\"]"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"tests\":[\"7\"]"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"testsFailed\":0"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"hostContract\":\"browser-requested\""),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"runtimeBackend\":\"browser-harness\""),
        "summary: {}",
        summary
    );
}
