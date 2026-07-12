use super::*;

#[test]
fn browser_requested_runtime_can_execute_with_an_explicit_harness_command() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        capture_env(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    call $args_len
                    i64.extend_i32_s
                    call $console_log))
            "#,
    );

    let outcome = execute_browser_runtime(
        &runtime,
        &wasm,
        false,
        runtime.canonical_runtime_profiles(),
        "node",
    )
    .expect("browser runtime outcome");

    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.tests_run, 0);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.runtime_backend.canonical_label(), "browser-harness");
    assert!(outcome.stdout.contains('2'), "stdout: {}", outcome.stdout);
}

#[test]
fn browser_requested_test_runtime_can_execute_registered_callbacks() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        vec!["gamma".to_string()],
        capture_env(),
        PathBuf::from("."),
        "browser",
    );
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

    let outcome = execute_browser_runtime(
        &runtime,
        &wasm,
        true,
        runtime.canonical_runtime_profiles(),
        "node",
    )
    .expect("browser runtime test outcome");

    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.runtime_backend.canonical_label(), "browser-harness");
    assert!(outcome.stdout.contains("11"), "stdout: {}", outcome.stdout);
}

#[test]
fn browser_requested_runtime_threads_recorded_coverage_hits_into_runtime_outcome() {
    // Real end-to-end regression test for the beyond-scope fix in
    // `execute_browser_runtime` (crate-root `execute.rs`): `RuntimeOutcome.coverage_hits`
    // used to be hardcoded to `Vec::new()` for the browser lane, silently
    // discarding whatever `BrowserRuntimeExecutionOutcome.coverage_hits` the
    // harness reported. This drives the real `coverage_hit` `kali:rt` import
    // (host-wired in every browser harness list) through the real node
    // harness and asserts the recorded ids survive all the way out to
    // `RuntimeOutcome.coverage_hits` — a revert of that threading back to
    // `Vec::new()` must fail this test.
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        vec![],
        capture_env(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "coverage_hit" (func $coverage_hit (param i32)))
                (func (export "_start")
                    i32.const 3
                    call $coverage_hit
                    i32.const 7
                    call $coverage_hit))
            "#,
    );

    let outcome = execute_browser_runtime(
        &runtime,
        &wasm,
        false,
        runtime.canonical_runtime_profiles(),
        "node",
    )
    .expect("browser runtime outcome");

    assert_eq!(outcome.exit_code, 0);
    let mut coverage_hits = outcome.coverage_hits.clone();
    coverage_hits.sort_unstable();
    assert_eq!(
        coverage_hits,
        vec![3, 7],
        "RuntimeOutcome.coverage_hits must carry the coverage ids the guest reported through \
         `kali:rt` coverage_hit, threaded from BrowserRuntimeExecutionOutcome.coverage_hits; \
         got {:?} (stdout: {})",
        outcome.coverage_hits,
        outcome.stdout
    );
}

#[cfg(unix)]
#[test]
fn browser_runtime_execution_helper_uses_html_entrypoint_for_browser_executables() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let browser = tempdir.path().join("firefox");
    symlink("/bin/sh", &browser).expect("link browser executable shim to /bin/sh");

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );
    let browser_command = format!(
        r#"{} -c 'cat <<EOF > "$KALI_BROWSER_HARNESS_SUMMARY_FILE"
{{"args":[],"tests":["7"],"testsFailed":0}}
EOF
exit 0'"#,
        browser.display()
    );
    let outcome = browser_runtime_execute_checked(
        Some(browser_command.as_str()),
        &wasm,
        &[],
        tempdir.path(),
        true,
    )
    .expect("execute browser runtime harness through browser executable");

    assert_eq!(outcome.command[0], browser.display().to_string());
    assert_eq!(outcome.command[1], "-c");
    assert!(
        outcome.command[2].contains("cat <<EOF"),
        "command: {:?}",
        outcome.command
    );
    assert!(
        outcome.command[3].starts_with("file://"),
        "command: {:?}",
        outcome.command
    );
    assert!(
        outcome.command[3].contains("browser-runtime.html"),
        "command: {:?}",
        outcome.command
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.runtime_backend.canonical_label(), "browser-harness");
    assert_eq!(outcome.reported_args, Vec::<String>::new());
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.tests_run(), 1);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.status.code(), Some(0));
}

#[test]
fn browser_bundle_runtime_execute_checked_loads_bundle_exports_and_parses_summary() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (func (export "__kali_callback_7")
                    i64.const 11
                    call $console_log)
                (func (export "_start")
                    i64.const 7
                    call $test_register
                    call $args_len
                    i64.extend_i32_s
                    call $console_log))
        "#,
    );
    fs::write(bundle_root.join("browser-app.wasm"), &wasm).expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let outcome = browser_bundle_runtime_execute_checked(
        Some("node"),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert!(outcome.stdout.contains("11"), "stdout: {}", outcome.stdout);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.tests_run(), 1);
}

#[cfg(unix)]
#[test]
fn browser_bundle_runtime_execute_checked_uses_html_entrypoint_for_browser_executables() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let chromium = tempdir.path().join("chromium");
    symlink("/bin/sh", &chromium).expect("link browser executable shim to /bin/sh");

    let command = format!("{} -c true", chromium.display());
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command.as_str()),
        &bundle_root,
        &[],
        false,
        false,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], chromium.display().to_string());
    assert_eq!(outcome.command[1], "-c");
    assert_eq!(outcome.command[2], "true");
    assert!(
        outcome.command[3].starts_with("file://"),
        "command: {:?}",
        outcome.command
    );
    assert!(
        outcome.command[3].contains("browser-bundle-runtime.html"),
        "command: {:?}",
        outcome.command
    );
    assert_eq!(outcome.status.code(), Some(0));
}

#[test]
fn browser_runtime_execution_helper_launches_browser_harness_and_parses_summary() {
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
                    call $test_register
                    i64.const 5
                    call $console_log))
        "#,
    );
    let tempdir = kali_test_support::fixtures::tempdir();
    let outcome = browser_runtime_execute_checked(
        Some("node"),
        &wasm,
        &["delta".to_string()],
        tempdir.path(),
        true,
    )
    .expect("execute browser runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert!(outcome.stdout.contains('5'), "stdout: {}", outcome.stdout);
    assert_eq!(outcome.reported_args, vec!["delta".to_string()]);
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.tests_run(), 1);
}
