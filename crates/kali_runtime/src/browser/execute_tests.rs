use crate::*;
use crate::test_support::*;
use std::{fs};
#[cfg(unix)]
use std::os::unix::fs::symlink;


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


#[test]
fn browser_harness_invocation_checked_builds_a_launch_plan() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser-harness.mjs");
    let args = vec!["alpha".to_string(), "beta".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some("node --experimental-fetch"),
        &script,
        &args,
        tempdir.path(),
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "node");
    assert_eq!(
        invocation.harness_args,
        vec!["--experimental-fetch".to_string()]
    );
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, tempdir.path());
    assert_eq!(
        invocation.command,
        vec![
            "node".to_string(),
            "--experimental-fetch".to_string(),
            script.display().to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
}


#[cfg(unix)]
#[test]
fn browser_harness_invocation_checked_uses_file_url_for_browser_executables() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser-harness.html");
    let args = vec!["alpha".to_string(), "beta".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some("chromium --headless"),
        &script,
        &args,
        tempdir.path(),
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "chromium");
    assert_eq!(invocation.harness_args, vec!["--headless".to_string()]);
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, tempdir.path());
    assert!(
        invocation.command[2].starts_with("file://"),
        "command: {:?}",
        invocation.command
    );
    assert!(
        invocation.command[2].contains("browser-harness.html"),
        "command: {:?}",
        invocation.command
    );
    assert_eq!(
        invocation.command,
        vec![
            invocation.executable.clone(),
            "--headless".to_string(),
            invocation.command[2].clone(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
}


#[test]
fn browser_harness_run_checked_launches_command_and_captures_output() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-harness.mjs",
        r#"
console.error('browser-harness-stderr');
console.log(JSON.stringify(process.argv.slice(2)));
process.exit(7);
"#,
    );

    let outcome = browser_harness_run_checked(
        Some("node"),
        &script,
        &["alpha".to_string(), "beta".to_string()],
        tempdir.path(),
    )
    .expect("launch browser harness");

    assert_eq!(
        outcome.command,
        vec![
            "node".to_string(),
            script.display().to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
    assert_eq!(outcome.status.code(), Some(7));
    assert!(
        outcome.stdout.contains(r#"["alpha","beta"]"#),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stderr.contains("browser-harness-stderr"),
        "stderr: {}",
        outcome.stderr
    );
}


#[test]
fn browser_harness_launch_failure_preserves_the_resolved_command_vector() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = kali_test_support::fixtures::write_file(tempdir.path(), "browser-harness.mjs", "console.log('unreachable');");

    let error = browser_harness_run_checked(
        Some("definitely-not-a-real-browser-runner"),
        &script,
        &["alpha".to_string(), "beta".to_string()],
        tempdir.path(),
    )
    .expect_err("launch should fail for a missing executable");

    match error {
        BrowserHarnessError::LaunchFailed {
            executable,
            script: error_script,
            command,
            message,
        } => {
            assert_eq!(executable, "definitely-not-a-real-browser-runner");
            assert_eq!(error_script, script);
            assert_eq!(
                command,
                vec![
                    "definitely-not-a-real-browser-runner".to_string(),
                    script.display().to_string(),
                    "alpha".to_string(),
                    "beta".to_string(),
                ]
            );
            assert!(
                message.contains("No such file") || message.contains("not found"),
                "message: {message}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}


#[test]
fn browser_runtime_unavailable_diagnostic_formats_command_context() {
    let command_diagnostic = browser_runtime_unavailable_diagnostic(Some("run"), None);
    assert!(
        command_diagnostic
            .message
            .contains("run does not support the browser API surface"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .message
            .contains("selected host contract: browser-requested"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .message
            .contains("Phase-1 browser-targeted command set"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "selected host contract: browser-requested"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "supported browser runtime commands: run, test"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::contract_scope_note()),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::host_description_note()),
        "diagnostic: {command_diagnostic:?}"
    );

    let test_diagnostic = browser_runtime_unavailable_diagnostic(Some("test"), None);
    assert!(
        test_diagnostic
            .message
            .contains("test does not support the browser API surface"),
        "diagnostic: {test_diagnostic:?}"
    );
    assert!(
        test_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {test_diagnostic:?}"
    );

    let runtime_diagnostic = browser_runtime_unavailable_diagnostic(None, None);
    assert!(
        runtime_diagnostic
            .message
            .contains("current runtime contract"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .message
            .contains("selected host contract: browser-requested"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .message
            .contains("Phase-1 browser-targeted command set"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "selected host contract: browser-requested"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "supported browser runtime commands: run, test"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::contract_scope_note()),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::host_description_note()),
        "diagnostic: {runtime_diagnostic:?}"
    );
}
