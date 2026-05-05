use super::*;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    thread,
};

#[cfg(unix)]
use std::os::unix::fs::symlink;

fn compile_wat(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap_or_else(|error| panic!("valid wat error: {error}\n{wat}"))
}

fn wat_assert_buffer_eq(start: i32, expected: &str) -> String {
    let mut checks = String::new();
    for (index, byte) in expected.as_bytes().iter().enumerate() {
        checks.push_str(&format!(
                "                i32.const {}\n                i32.load8_u\n                i32.const {}\n                i32.ne\n                if\n                    unreachable\n                end\n",
                start + index as i32,
                byte
            ));
    }
    checks
}

#[cfg(unix)]
fn browser_exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code << 8)
}

#[cfg(windows)]
fn browser_exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code as u32)
}

#[test]
fn runtime_executes_modules_with_console_host_imports() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (import "kali:rt" "console_error" (func $console_error (param i64)))
                (import "kali:rt" "console_warn" (func $console_warn (param i64)))
                (import "kali:rt" "console_info" (func $console_info (param i64)))
                (import "kali:rt" "console_debug" (func $console_debug (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log
                    i64.const 2
                    call $console_error
                    i64.const 3
                    call $console_warn
                    i64.const 4
                    call $console_info
                    i64.const 5
                    call $console_debug))
            "#,
    );

    let runtime = RuntimeCtx::default();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "1\n4\n5\n");
    assert_eq!(outcome.stderr, "2\n[warn] 3\n");
}

#[test]
fn runtime_exposes_arguments() {
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        capture_env(),
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (func (export "_start")
                    call $args_len
                    i32.const 2
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_records_guest_process_exit_codes() {
    let runtime = RuntimeCtx::default();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "process_exit" (func $process_exit (param i64)))
                (func (export "_start")
                    i64.const 7
                    call $process_exit))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 7);
}

#[test]
fn runtime_context_carries_process_identity() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));

    assert_eq!(runtime.process_id(), std::process::id());
    assert_eq!(KaliHostState::default().process_id(), std::process::id());
}

#[test]
fn runtime_context_exposes_deterministic_env_snapshots() {
    let mut runtime = RuntimeCtx::with_host_context(
        None,
        Vec::new(),
        BTreeMap::from([
            (String::from("BETA"), String::from("2")),
            (String::from("ALPHA"), String::from("1")),
        ]),
        PathBuf::from("."),
    );

    let snapshot = runtime.env_snapshot();
    let snapshot_keys = snapshot.keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        snapshot_keys,
        vec![String::from("ALPHA"), String::from("BETA")]
    );
    assert!(runtime.env_has("ALPHA"));
    assert!(runtime.has("BETA"));
    assert!(!runtime.env_has("GAMMA"));
    assert_eq!(snapshot.get("ALPHA"), Some(&String::from("1")));
    assert_eq!(snapshot.get("BETA"), Some(&String::from("2")));
    assert_eq!(runtime.env_to_object(), snapshot);
    assert_eq!(runtime.snapshot(), snapshot);
    assert_eq!(runtime.env_snapshot_object_value(), snapshot);

    let json_snapshot = runtime.env_snapshot_value();
    let json_snapshot = json_snapshot.as_object().expect("json object");
    assert_eq!(
        json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );
    assert_eq!(
        runtime.env_snapshot_json_value(),
        runtime.env_snapshot_value()
    );
    assert_eq!(runtime.snapshot_value(), runtime.env_snapshot_value());
    assert_eq!(
        runtime.snapshot_object_value(),
        runtime.env_snapshot_value()
    );
    assert_eq!(runtime.snapshot_json_value(), runtime.env_snapshot_value());
    assert_eq!(runtime.env_to_json_value(), runtime.env_snapshot_value());

    runtime.env.insert(String::from("GAMMA"), String::from("3"));
    assert!(!snapshot.contains_key("GAMMA"));
    assert!(!json_snapshot.contains_key("GAMMA"));

    let host_state = KaliHostState {
        env: BTreeMap::from([
            (String::from("BETA"), String::from("2")),
            (String::from("ALPHA"), String::from("1")),
        ]),
        ..KaliHostState::default()
    };

    let host_snapshot = host_state.env_snapshot();
    assert_eq!(
        host_snapshot.keys().cloned().collect::<Vec<_>>(),
        vec![String::from("ALPHA"), String::from("BETA")]
    );
    assert!(host_state.env_has("ALPHA"));
    assert!(host_state.has("BETA"));
    assert!(!host_state.env_has("GAMMA"));
    assert_eq!(host_state.env_to_object(), host_snapshot);
    assert_eq!(host_state.env_snapshot_object_value(), host_snapshot);

    let host_json_snapshot = host_state.env_snapshot_value();
    let host_json_snapshot = host_json_snapshot.as_object().expect("json object");
    assert_eq!(
        host_json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        host_json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );
    assert_eq!(
        host_state.env_snapshot_json_value(),
        host_state.env_snapshot_value()
    );
    assert_eq!(host_state.snapshot(), host_state.thread_topology_snapshot());
    assert_eq!(
        host_state.snapshot_object_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.snapshot_json_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.snapshot_value(),
        host_state.thread_topology_snapshot_value()
    );
    assert_eq!(
        host_state.env_to_json_value(),
        host_state.env_snapshot_value()
    );
}

#[test]
fn runtime_context_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    assert_eq!(runtime.api_surface, "deno");
    assert_eq!(
        runtime.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn runtime_reports_browser_host_contract_for_browser_api_surface() {
    let runtime = RuntimeCtx::with_api_surface(None, "browser");

    assert_eq!(
        runtime.host_contract(),
        RuntimeHostContract::BrowserRequested
    );
    assert!(runtime
        .host_contract()
        .canonical_label()
        .contains("browser"));
}

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
    let tempdir = tempfile::tempdir().expect("tempdir");
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
fn browser_runtime_contract_documents_the_future_execution_surface() {
    let descriptor = BrowserRuntimeContract::descriptor();

    assert_eq!(descriptor.host_label, "browser-requested");
    assert_eq!(descriptor.host_description, "real browser host");
    assert_eq!(
        descriptor.host_description_note,
        "browser runtime host description: real browser host"
    );
    assert_eq!(descriptor.supported_commands, &["run", "test"]);
    assert_eq!(
        descriptor.supported_commands_note,
        "supported browser runtime commands: run, test"
    );
    assert_eq!(
        descriptor.summary_note,
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    );
    assert_eq!(
        descriptor.contract_scope_note,
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    );
    assert_eq!(
        BrowserRuntimeContract::summary_file_fallback_note(),
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    );
    assert_eq!(
        BrowserRuntimeContract::diagnostic_notes(),
        &[
            BrowserRuntimeContract::supported_commands_note(),
            BrowserRuntimeContract::summary_note(),
            BrowserRuntimeContract::contract_scope_note(),
            BrowserRuntimeContract::summary_file_fallback_note(),
            BrowserRuntimeContract::host_description_note(),
        ]
    );
    assert!(descriptor
        .diagnostic_hint
        .contains("kali check --api browser"));
    assert!(descriptor
        .diagnostic_hint
        .contains("kali build --bundle --api browser"));
}

#[test]
fn split_command_spec_supports_shell_like_quoting() {
    let parts = split_command_spec(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    )
    .expect("split valid browser harness command");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );
}

#[test]
fn browser_harness_command_parts_exposes_override_and_default_selection() {
    let override_parts = browser_harness_command_parts_for(Some(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    ));
    assert_eq!(
        override_parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );

    let default_parts = browser_harness_command_parts();
    assert!(
        !default_parts.is_empty(),
        "default browser harness command should not be empty"
    );
    assert!(
        matches!(default_parts[0].as_str(), "bun" | "node")
            || browser_harness_uses_html_entrypoint(&default_parts[0]),
        "default browser harness command should prefer a browser executable when one is available"
    );
}

#[test]
fn browser_harness_invocation_checked_preserves_html_entrypoint_file_urls_for_paths_with_spaces() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script = tempdir.path().join("browser app.html");
    let current_dir = tempdir.path().join("browser cwd");
    std::fs::create_dir_all(&current_dir).expect("create browser cwd");
    let args = vec!["one".to_string(), "two words".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some(r#"chrome --headless --profile "real browser""#),
        &script,
        &args,
        &current_dir,
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "chrome");
    assert_eq!(
        invocation.harness_args,
        vec![
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
        ]
    );
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, current_dir);
    assert_eq!(invocation.command[0], "chrome");
    assert_eq!(invocation.command[1], "--headless");
    assert_eq!(invocation.command[2], "--profile");
    assert_eq!(invocation.command[3], "real browser");
    assert!(
        invocation.command[4].starts_with("file:"),
        "command: {:?}",
        invocation.command
    );
    assert!(
        invocation.command[4].contains("browser%20app.html"),
        "command: {:?}",
        invocation.command
    );
    assert_eq!(invocation.command[5], "one");
    assert_eq!(invocation.command[6], "two words");
}

#[test]
fn browser_harness_launch_failure_reports_the_resolved_command_and_script() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script = tempdir.path().join("browser app.html");
    let current_dir = tempdir.path().join("browser cwd");
    std::fs::create_dir_all(&current_dir).expect("create browser cwd");
    let args = vec!["one".to_string(), "two words".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some(r#"definitely-not-a-real-browser --headless --profile "real browser""#),
        &script,
        &args,
        &current_dir,
    )
    .expect("build browser harness invocation");
    let expected_command = invocation.command.clone();

    let error = invocation.launch().expect_err("launch should fail");
    let message = error.to_string();

    match error {
        BrowserHarnessError::LaunchFailed {
            executable,
            script: observed_script,
            command,
            message: launch_message,
        } => {
            assert_eq!(executable, "definitely-not-a-real-browser");
            assert_eq!(observed_script, script);
            assert_eq!(command, expected_command);
            assert!(!launch_message.is_empty());
        }
        other => panic!("unexpected browser harness error: {other:?}"),
    }

    assert!(message.contains("failed to launch browser harness command"));
    assert!(message.contains("browser app.html"));
    assert!(message.contains("definitely-not-a-real-browser"));
}

#[test]
fn browser_harness_recognizes_all_canonical_browser_executable_names() {
    for executable in BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES {
        let parts = match browser_harness_command_parts_for_browser_executable(executable) {
            Some(parts) => parts,
            None => panic!(
                "recognized browser alias should be treated as a browser executable: {executable}"
            ),
        };
        assert_eq!(
            parts,
            vec![executable.to_string(), "--headless".to_string()]
        );
        assert!(browser_harness_uses_html_entrypoint(executable));
    }
}

#[test]
fn split_command_spec_rejects_malformed_inputs() {
    assert_eq!(split_command_spec("   "), None);
    assert_eq!(split_command_spec(r#"" --flag"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper "unterminated"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper \"#), None);
}

#[test]
fn browser_harness_command_parts_checked_reports_malformed_overrides() {
    let empty_override = browser_harness_command_parts_checked(Some(""))
        .expect_err("empty override should be rejected");
    assert!(empty_override.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));

    let empty_executable = browser_harness_command_parts_checked(Some(r#"" --flag"#))
        .expect_err("empty executable token should be rejected");
    assert!(empty_executable.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(empty_executable.contains(r#"" --flag"#));

    let flag_only = browser_harness_command_parts_checked(Some("--headless"))
        .expect_err("flag-only command should be rejected");
    assert!(flag_only.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(flag_only.contains("--headless"));

    let unterminated =
        browser_harness_command_parts_checked(Some(r#"browser-wrapper "unterminated"#))
            .expect_err("unterminated quotes should be rejected");
    assert!(unterminated.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(unterminated.contains("browser-wrapper"));
    assert!(unterminated.contains("unterminated"));
}

#[test]
fn browser_runtime_harness_page_wraps_the_module_body_for_real_browser_hosts() {
    let page = browser_runtime_harness_page(
        &[0x00, 0x61, 0x73, 0x6d],
        &["alpha".to_string(), "beta".to_string()],
        true,
    );

    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const runtimeArgs = [\"alpha\",\"beta\"]"),
        "page: {page}"
    );
    assert!(
        page.contains("const runRegisteredTests = true;"),
        "page: {page}"
    );
    assert!(page.contains("decodeBase64(\""), "page: {page}");
}

#[test]
fn browser_harness_uses_html_entrypoint_for_browser_executables() {
    assert!(browser_harness_uses_html_entrypoint("chrome"));
    assert!(browser_harness_uses_html_entrypoint(
        "chrome-headless-shell"
    ));
    assert!(browser_harness_uses_html_entrypoint("chromium"));
    assert!(browser_harness_uses_html_entrypoint("chromium-browser"));
    assert!(browser_harness_uses_html_entrypoint("chromium-for-testing"));
    assert!(browser_harness_uses_html_entrypoint("chromium for testing"));
    assert!(browser_harness_uses_html_entrypoint(
        "/usr/bin/google-chrome-stable"
    ));
    assert!(browser_harness_uses_html_entrypoint("google chrome beta"));
    assert!(browser_harness_uses_html_entrypoint("google chrome canary"));
    assert!(browser_harness_uses_html_entrypoint("google chrome dev"));
    assert!(browser_harness_uses_html_entrypoint(
        "google chrome for testing"
    ));
    assert!(browser_harness_uses_html_entrypoint("google chrome stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "google chrome unstable"
    ));
    assert!(browser_harness_uses_html_entrypoint("google-chrome-stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome-headless-shell"
    ));
    assert!(browser_harness_uses_html_entrypoint("msedge.exe"));
    assert!(browser_harness_uses_html_entrypoint("msedge-beta"));
    assert!(browser_harness_uses_html_entrypoint("msedge-canary"));
    assert!(browser_harness_uses_html_entrypoint("msedge-dev"));
    assert!(browser_harness_uses_html_entrypoint("msedge-insider"));
    assert!(browser_harness_uses_html_entrypoint("msedge-stable"));
    assert!(browser_harness_uses_html_entrypoint("edge-beta"));
    assert!(browser_harness_uses_html_entrypoint("edge-canary"));
    assert!(browser_harness_uses_html_entrypoint("edge-dev"));
    assert!(browser_harness_uses_html_entrypoint("edge-insider"));
    assert!(browser_harness_uses_html_entrypoint("edge-stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge-stable"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft edge stable"
    ));
    assert!(browser_harness_uses_html_entrypoint("brave-browser.exe"));
    assert!(browser_harness_uses_html_entrypoint(
        "brave-browser-stable.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint("brave browser stable"));
    assert!(browser_harness_uses_html_entrypoint("chrome.cmd"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome.desktop"
    ));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.app"));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.command"
    ));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.lnk"));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.lnk.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.app.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "C:/Program Files/Google/Chrome/Application/google-chrome.desktop"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge.desktop.cmd"
    ));
    assert!(browser_harness_uses_html_entrypoint("chrome.ps1"));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.url"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome.url.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge.url.cmd"
    ));
    assert!(browser_harness_uses_html_entrypoint("google-chrome-dev"));
    assert!(browser_harness_uses_html_entrypoint("brave-browser-dev"));
    assert!(browser_harness_uses_html_entrypoint(
        "brave-browser-nightly"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge-insider"
    ));
    assert!(browser_harness_uses_html_entrypoint("firefox-beta"));
    assert!(browser_harness_uses_html_entrypoint("firefox-esr"));
    assert!(browser_harness_uses_html_entrypoint("opera-stable"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi-stable"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi-snapshot"));
    assert!(browser_harness_uses_html_entrypoint(
        "C:/Program Files/Google/Chrome/Application/chrome.bat"
    ));
    assert!(browser_harness_uses_html_entrypoint("firefox"));
    assert!(browser_harness_uses_html_entrypoint("firefox-nightly"));
    assert!(browser_harness_uses_html_entrypoint(
        "firefox-developer-edition"
    ));
    assert!(browser_harness_uses_html_entrypoint("librewolf"));
    assert!(browser_harness_uses_html_entrypoint("waterfox"));
    assert!(browser_harness_uses_html_entrypoint("mullvad-browser"));
    assert!(browser_harness_uses_html_entrypoint("mullvad browser"));
    assert!(browser_harness_uses_html_entrypoint("privacy-browser"));
    assert!(browser_harness_uses_html_entrypoint("privacy browser"));
    assert!(browser_harness_uses_html_entrypoint("opera"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi"));
    assert!(browser_harness_uses_html_entrypoint("Mullvad Browser.app"));
    assert!(browser_harness_uses_html_entrypoint("zen-browser"));
    assert!(browser_harness_uses_html_entrypoint("zen browser"));
    assert!(browser_harness_uses_html_entrypoint("thorium-browser"));
    assert!(browser_harness_uses_html_entrypoint("thorium browser"));
    assert!(!browser_harness_uses_html_entrypoint("node"));
    assert!(!browser_harness_uses_html_entrypoint("bun"));
}

#[test]
fn browser_harness_command_parts_for_browser_executables_use_headless_mode() {
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chrome"),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chrome-headless-shell"),
        Some(vec![
            "chrome-headless-shell".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium"),
        Some(vec!["chromium".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium-for-testing"),
        Some(vec![
            "chromium-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium for testing"),
        Some(vec![
            "chromium for testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("/usr/bin/google-chrome-stable"),
        Some(vec![
            "google-chrome-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.app"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.command"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.lnk"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.lnk.exe"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.url"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome.url.exe"),
        Some(vec!["google-chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser"),
        Some(vec!["brave-browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser-stable"),
        Some(vec![
            "brave-browser-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave browser stable"),
        Some(vec![
            "brave browser stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("opera"),
        Some(vec!["opera".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("vivaldi"),
        Some(vec!["vivaldi".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome stable"),
        Some(vec![
            "google chrome stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome beta"),
        Some(vec![
            "google chrome beta".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome canary"),
        Some(vec![
            "google chrome canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome dev"),
        Some(vec![
            "google chrome dev".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome for testing"),
        Some(vec![
            "google chrome for testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome unstable"),
        Some(vec![
            "google chrome unstable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome-dev"),
        Some(vec![
            "google-chrome-dev".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium-dev"),
        Some(vec!["chromium-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser-nightly"),
        Some(vec![
            "brave-browser-nightly".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("vivaldi-snapshot"),
        Some(vec![
            "vivaldi-snapshot".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-beta"),
        Some(vec!["firefox-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-developer-edition"),
        Some(vec![
            "firefox-developer-edition".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome-headless-shell"),
        Some(vec![
            "google-chrome-headless-shell".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-beta"),
        Some(vec!["msedge-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-canary"),
        Some(vec!["msedge-canary".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-dev"),
        Some(vec!["msedge-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-insider"),
        Some(vec!["msedge-insider".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-stable"),
        Some(vec!["msedge-stable".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-beta"),
        Some(vec!["edge-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-stable"),
        Some(vec!["edge-stable".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-stable"),
        Some(vec![
            "microsoft-edge-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft edge stable"),
        Some(vec![
            "microsoft edge stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-canary"),
        Some(vec!["edge-canary".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-dev"),
        Some(vec!["edge-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-insider"),
        Some(vec!["edge-insider".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-canary"),
        Some(vec![
            "microsoft-edge-canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-insider"),
        Some(vec![
            "microsoft-edge-insider".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Microsoft/Edge/Application/msedge.exe"
        ),
        Some(vec!["msedge".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome.cmd"
        ),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome.ps1"
        ),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome-for-testing.com"
        ),
        Some(vec![
            "chrome-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/google-chrome-for-testing.com"
        ),
        Some(vec![
            "google-chrome-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/google-chrome-canary.exe"
        ),
        Some(vec![
            "google-chrome-canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "/usr/share/applications/google-chrome.desktop"
        ),
        Some(vec!["google-chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "/opt/Thorium Browser/thorium-browser.com"
        ),
        Some(vec![
            "thorium-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("librewolf"),
        Some(vec!["librewolf".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("waterfox"),
        Some(vec!["waterfox".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("mullvad-browser"),
        Some(vec![
            "mullvad-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("mullvad browser"),
        Some(vec![
            "mullvad browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("zen-browser"),
        Some(vec!["zen-browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("zen browser"),
        Some(vec!["zen browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("thorium-browser"),
        Some(vec![
            "thorium-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("thorium browser"),
        Some(vec![
            "thorium browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox"),
        Some(vec!["firefox".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-nightly"),
        Some(vec![
            "firefox-nightly".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-developer-edition"),
        Some(vec![
            "firefox-developer-edition".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("node"),
        None
    );
}

#[test]
fn browser_bundle_harness_script_reuses_the_shared_fetch_prelude() {
    let script = browser_bundle_harness_script(
        "browser-app",
        false,
        "const mod = await import(bundleJs.href);\nconsole.log(typeof mod);\n",
    );
    assert!(script.contains("const bundleJs = new URL('./browser-app/browser-app.js'"));
    assert!(script.contains("const wasmUrl = new URL('./browser-app/browser-app.wasm'"));
    assert!(script.contains("console.log(typeof mod);"));
    assert!(script.contains("globalThis.fetch = async (input) => {"));
}

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script_path = tempdir.path().join("browser-runtime.mjs");
    fs::write(&script_path, script).expect("write browser runtime script");

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script_path = tempdir.path().join("browser-runtime-tests.mjs");
    fs::write(&script_path, script).expect("write browser runtime test script");

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script_path = tempdir.path().join("browser-runtime-tests-failed.mjs");
    fs::write(&script_path, script).expect("write browser runtime test script");

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script_path = tempdir.path().join("browser-runtime-empty-tests.mjs");
    fs::write(&script_path, script).expect("write browser runtime test script");

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script_path = tempdir.path().join("browser-runtime-summary-file.mjs");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(&script_path, script).expect("write browser runtime summary script");

    let outcome = browser_harness_run_checked_with_env(
        Some("node"),
        &script_path,
        &[],
        tempdir.path(),
        &[(
            super::BROWSER_HARNESS_SUMMARY_FILE_ENV,
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

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(summary.host_contract, None);
    assert_eq!(summary.runtime_backend, None);
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(&summary_path, "not-json").expect("write malformed summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(summary.host_contract, None);
    assert_eq!(summary.runtime_backend, None);
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(&summary_path, " \n\t\n").expect("write blank summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_empty() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(&summary_path, "").expect("write empty summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_unreadable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::create_dir(&summary_path).expect("create unreadable summary path");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_incomplete() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    )
    .expect("write incomplete summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_stdout_labels_when_summary_file_labels_are_invalid() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":4,"hostContract":"not-a-contract","runtimeBackend":"not-a-backend"}"#,
    )
    .expect("write invalid-label summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":9,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(4));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_stdout_metadata_when_summary_file_has_invalid_labels_and_invalid_tests_failed_type(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":"oops","hostContract":"not-a-contract","runtimeBackend":"not-a-backend"}"#,
    )
    .expect("write invalid-label summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":9,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(9));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":[1],"tests":["browser invalid array items"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    )
    .expect("write invalid-array summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser invalid array items"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(
        summary.tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_null_args_field() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":null,"tests":["browser null args"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    )
    .expect("write null-args summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser null args"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser null args".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_null_tests_field() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":["stdout"],"tests":null,"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    )
    .expect("write null-tests summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser null tests"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser null tests".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":["zeta"],"tests":["7"],"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    )
    .expect("write partial summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(1),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":1,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(1));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_uses_stdout_labels_when_summary_file_lacks_them() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#,
    )
    .expect("write summary file without labels");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_prefers_the_last_json_line_from_stdout() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(&summary_path, "still-not-json").expect("write malformed summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: [
            "guest log line\n",
            r#"{"args":["ignored"],"tests":["1"],"testsFailed":9,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#,
            "\ntrailing non-json noise\n",
            r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
        ]
        .concat(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_prefers_the_last_json_line_from_a_noisy_summary_file() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        [
            "summary log line\n",
            r#"{"args":["ignored"],"tests":["1"],"testsFailed":4,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#,
            "\nsummary trailing noise\n",
            r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
        ]
        .concat(),
    )
    .expect("write noisy summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":3,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_bundle_runtime_execute_checked_loads_bundle_exports_and_parses_summary() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

#[test]
fn browser_bundle_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_null_value() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":null,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_invalid_type()
{
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_non_integer_number(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":1.5,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_negative_number(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":-1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_null_value() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":null,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser missing\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser missing".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unparseable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser unparseable".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 4);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels_and_is_missing_tests_failed(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":9"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid array items\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":9"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels_and_invalid_args(
) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels and args".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser missing\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser missing".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[cfg(unix)]
#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unreadable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser unreadable".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":2,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[1],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 8);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":8"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_labels_when_summary_file_lacks_them() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"stdout\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_harness_page_wraps_the_module_body_for_real_browser_hosts() {
    let page = browser_bundle_runtime_harness_page(
        "browser-app",
        false,
        &["alpha".to_string(), "beta".to_string()],
        true,
    );

    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const runtimeArgs = [\"alpha\",\"beta\"]"),
        "page: {page}"
    );
    assert!(
        page.contains("const runRegisteredTests = true;"),
        "page: {page}"
    );
    assert!(page.contains("browser-app/browser-app.js"), "page: {page}");
}

#[cfg(unix)]
#[test]
fn browser_bundle_runtime_execute_checked_uses_html_entrypoint_for_browser_executables() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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
    let tempdir = tempfile::tempdir().expect("tempdir");
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
    let tempdir = tempfile::tempdir().expect("tempdir");
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
    let tempdir = tempfile::tempdir().expect("tempdir");
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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script = tempdir.path().join("browser-harness.mjs");
    fs::write(
        &script,
        r#"
console.error('browser-harness-stderr');
console.log(JSON.stringify(process.argv.slice(2)));
process.exit(7);
"#,
    )
    .expect("write browser harness script");

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
    let tempdir = tempfile::tempdir().expect("tempdir");
    let script = tempdir.path().join("browser-harness.mjs");
    fs::write(&script, "console.log('unreachable');").expect("write browser harness script");

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

#[test]
fn runtime_rejects_browser_api_surface() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        BTreeMap::new(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("browser runtime should be gated");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic
            .message
            .contains("standalone browser runtime contract"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::summary_note()),
        "diagnostic: {diagnostic:?}"
    );
    assert_eq!(
        diagnostic.context.as_deref(),
        Some(
            &DiagnosticContext::new(DiagnosticContextOrigin::Default)
                .with_requested_value("browser")
                .with_effective_value("browser")
        ),
        "diagnostic: {diagnostic:?}"
    );
}

#[test]
fn runtime_test_execution_rejects_browser_api_surface() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        BTreeMap::new(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute_tests(&wasm)
        .expect_err("browser runtime test path should be gated");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic
            .message
            .contains("standalone browser runtime contract"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::summary_note()),
        "diagnostic: {diagnostic:?}"
    );
    assert_eq!(
        diagnostic.context.as_deref(),
        Some(
            &DiagnosticContext::new(DiagnosticContextOrigin::Default)
                .with_requested_value("browser")
                .with_effective_value("browser")
        ),
        "diagnostic: {diagnostic:?}"
    );
}

#[test]
fn runtime_accepts_threaded_runtime_profile_requests() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()]);
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("threaded runtime profile");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
}

#[test]
fn runtime_rejects_positive_thread_budget_requests_without_threaded_profile() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute_tests(&wasm)
        .expect_err("positive thread budgets should remain gated without the threaded profile");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic.message.contains("resources.maxThreads"),
        "diagnostic: {diagnostic:?}"
    );
}

#[test]
fn runtime_accepts_positive_thread_budget_requests_with_threaded_profile() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime
        .execute_tests(&wasm)
        .expect("positive thread budgets should be accepted when the threaded profile is active");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
}

#[test]
fn runtime_outcome_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::KaliHosted);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::Wasmtime);
}

#[test]
fn runtime_execute_normalizes_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " beta ".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ];

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn runtime_execute_tests_normalizes_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " beta ".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ];

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime test outcome");
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn runtime_test_outcome_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime test outcome");
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::KaliHosted);
}

#[test]
fn runtime_exposes_canonical_runtime_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " wasm-threads ".to_string(),
        "alpha".to_string(),
        "wasm-threads".to_string(),
        "alpha".to_string(),
    ];

    assert_eq!(
        runtime.canonical_runtime_profiles(),
        vec!["alpha".to_string(), "wasm-threads".to_string()]
    );
}

#[test]
fn normalize_runtime_profiles_is_shared_between_callers() {
    assert_eq!(
        normalize_runtime_profiles(vec![
            " wasm-threads ".to_string(),
            "alpha".to_string(),
            "wasm-threads".to_string(),
            "alpha".to_string(),
        ]),
        vec!["alpha".to_string(), "wasm-threads".to_string()]
    );
}

#[test]
fn runtime_context_carries_thread_budget_override() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(0));

    assert_eq!(runtime.max_threads, Some(0));
}

#[test]
fn runtime_context_resolves_effective_thread_budget() {
    let policy_path = tempfile::NamedTempFile::new().expect("policy temp file");
    std::fs::write(
        policy_path.path(),
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy fixture");
    let policy = SandboxPolicy::from_file(policy_path.path()).expect("load policy");

    let runtime = RuntimeCtx::with_api_surface(Some(policy), "deno").with_max_threads(Some(2));
    assert_eq!(runtime.effective_thread_budget(), Some(0));

    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(0));
    assert_eq!(runtime.effective_thread_budget(), Some(0));
}

#[test]
fn runtime_host_state_tracks_thread_budget_bookkeeping() {
    let mut state = KaliHostState::default();

    assert!(
        state.begin_thread().is_err(),
        "thread creation should be gated without the threaded profile opt-in"
    );

    state.runtime_profiles = vec!["wasm-threads".to_string()];
    assert!(
        state.begin_thread().is_err(),
        "thread creation should still be gated without a budget"
    );

    state.max_threads = Some(0);
    assert!(
        state.begin_thread().is_err(),
        "zero-cap budgets should deny thread creation"
    );

    state.max_threads = Some(2);
    assert!(state.begin_thread().is_ok());
    assert!(state.begin_thread().is_ok());
    assert!(state.begin_thread().is_err());
    state.finish_thread();
    assert!(state.begin_thread().is_ok());
}

#[test]
fn runtime_host_state_spawns_and_releases_thread_instances() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(2),
        ..Default::default()
    };

    let first = state
        .spawn_thread_instance("https://e.co/t.js")
        .expect("first thread instance");
    assert_eq!(first, 0);
    assert_eq!(state.active_threads, 1);
    assert_eq!(state.thread_topology.total_instances(), 1);

    let second = state
        .spawn_thread_instance("https://e.co/u.js")
        .expect("second thread instance");
    assert_eq!(second, 1);
    assert_eq!(state.active_threads, 2);
    assert_eq!(state.thread_topology.total_instances(), 2);

    let snapshot = state.thread_topology_snapshot();
    assert_eq!(snapshot.total_instances, 2);
    assert_eq!(snapshot.terminated_instances, 0);
    assert_eq!(snapshot.live_instances.len(), 2);
    assert_eq!(
        snapshot
            .live_instances
            .iter()
            .map(|entry| entry.instance_id)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_eq!(
        state.thread_topology_snapshot_value(),
        serde_json::json!({
            "totalInstances": 2,
            "terminatedInstances": 0,
            "liveInstances": [
                {
                    "instanceId": first,
                    "scriptUrl": "https://e.co/t.js",
                    "postedMessages": [],
                    "postedSharedBuffers": [],
                    "wasTerminated": false
                },
                {
                    "instanceId": second,
                    "scriptUrl": "https://e.co/u.js",
                    "postedMessages": [],
                    "postedSharedBuffers": [],
                    "wasTerminated": false
                }
            ]
        })
    );
    assert_eq!(
        state.thread_topology_snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.thread_topology_snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        state.thread_topology_snapshot().snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(state.snapshot(), state.thread_topology_snapshot());
    assert_eq!(snapshot.snapshot(), state.thread_topology_snapshot());
    assert_eq!(
        snapshot.thread_topology_snapshot_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_json_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );
    assert_eq!(
        snapshot.snapshot_object_value(),
        state.thread_topology_snapshot_value()
    );

    let diagnostic = state
        .spawn_thread_instance("https://e.co/v.js")
        .expect_err("thread budget should cap guest thread spawns");
    assert_eq!(
        state
            .pending_diagnostic
            .as_ref()
            .and_then(|diagnostic| diagnostic.code),
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(diagnostic.to_string().contains("KALI_E4003"));

    assert!(state.release_thread_instance(second));
    assert_eq!(state.active_threads, 1);
    assert!(state.release_thread_instance(first));
    assert_eq!(state.active_threads, 0);
    assert!(!state.release_thread_instance(first));
}

#[test]
fn runtime_host_state_rolls_back_failed_thread_spawns() {
    let mut state = KaliHostState {
        runtime_profiles: vec!["wasm-threads".to_string()],
        max_threads: Some(1),
        ..Default::default()
    };

    let _error = state
        .spawn_thread_instance("not-a-valid-thread-url")
        .expect_err("invalid URLs should be rejected before they leak bookkeeping");
    assert_eq!(state.active_threads, 0);
    assert_eq!(state.thread_topology.total_instances(), 0);
}

#[test]
fn runtime_executes_thread_spawn_host_imports() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("thread spawn host import");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
}

#[test]
fn runtime_rejects_thread_spawn_host_imports_when_budget_is_zero() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(0));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://e.co/t.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 17
                    call $thread_spawn
                    drop))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("zero thread budgets should deny thread creation through the host import");
    assert_eq!(
        diagnostics[0].code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(diagnostics[0]
        .message
        .contains("active thread count 1 exceeds policy limit of 0"));
}

#[test]
fn runtime_exposes_environment_variables() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    i32.const 128
                    i32.const 64
                    call $env_get
                    i32.const 6
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_environment_variable_presence() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_has" (func $env_has (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    call $env_has
                    i32.const 1
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_current_working_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cwd = dir.path().to_path_buf();
    let expected_len = cwd.to_string_lossy().len() as i32;
    let runtime = RuntimeCtx::with_host_context(None, Vec::new(), BTreeMap::new(), cwd);

    let wasm = compile_wat(&format!(
        r#"
            (module
                (import "kali:rt" "cwd" (func $cwd (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 0
                    i32.const 128
                    i32.const 64
                    call $cwd
                    i32.const {expected_len}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#
    ));

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_deletes_environment_variables() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_delete" (func $env_delete (param i32 i32 i32 i32) (result i32)))
                (import "kali:rt" "env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    i32.const 0
                    i32.const 0
                    call $env_delete
                    drop
                    i32.const 0
                    i32.const 21
                    i32.const 128
                    i32.const 64
                    call $env_get
                    i32.eqz
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_writes_text_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), dir.path().to_path_buf());
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "fs_write_text_file" (func $write (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "./written.txt")
                (data (i32.const 64) "hello runtime")
                (func (export "_start")
                    i32.const 0
                    i32.const 13
                    i32.const 64
                    i32.const 13
                    call $write
                    i32.const 0
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);

    let written = fs::read_to_string(dir.path().join("written.txt")).expect("written file");
    assert_eq!(written, "hello runtime");
}

#[test]
fn runtime_fetches_http_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let body = "hello fetch";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(response.as_bytes());
    });

    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let url = format!("http://127.0.0.1:{}/", addr.port());
    let wat = format!(
        r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    i32.const {}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        url,
        url.len(),
        body.len()
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    server.join().expect("server thread");
}

#[test]
fn runtime_reports_mocked_fetch_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(response.as_bytes());
    });

    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let url = format!("http://127.0.0.1:{}/missing", addr.port());
    let wat = format!(
        r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    drop))
            "#,
        url,
        url.len()
    );

    let wasm = compile_wat(&wat);
    let diagnostics = runtime.execute(&wasm).expect_err("fetch should fail");
    assert_eq!(diagnostics[0].code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(
        diagnostics[0].message.contains("runtime trap"),
        "diagnostic: {:?}",
        diagnostics[0]
    );
    server.join().expect("server thread");
}

#[test]
fn runtime_rejects_math_pow_negative_exponents_without_panicking() {
    let runtime = RuntimeCtx::default();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "math_pow" (func $math_pow (param i64 i64) (result i64)))
                (func (export "_start")
                    i64.const 2
                    i64.const -1
                    call $math_pow
                    drop))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("negative Math.pow exponents should be rejected through the host import");
    assert_eq!(diagnostics[0].code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(
        diagnostics[0].message.contains("runtime trap"),
        "diagnostic: {:?}",
        diagnostics[0]
    );
}

#[test]
fn runtime_executes_node_fs_promises_host_imports() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        dir.path().to_path_buf(),
        "node",
    );
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:node" "fs_promises_write_text_file" (func $write (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "fs_promises_read_text_file" (func $read (param i32 i32 i32 i32) (result i32)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "./node-promises.txt")
                (data (i32.const 64) "hello node fs")
                (func (export "_start")
                    i32.const 0
                    i32.const 19
                    i32.const 64
                    i32.const 13
                    call $write
                    drop
                    i32.const 0
                    i32.const 19
                    i32.const 128
                    i32.const 64
                    call $read
                    i32.const 13
                    i32.eq
                    if
                        i64.const 13
                        call $console_log
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    let written = fs::read_to_string(dir.path().join("node-promises.txt")).expect("written file");
    assert_eq!(written, "hello node fs");
}

#[test]
fn runtime_executes_node_stream_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:node" "stream_concat" (func $concat (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "hello ")
                (data (i32.const 32) "node")
                (func (export "_start")
                    i32.const 0
                    i32.const 6
                    i32.const 32
                    i32.const 4
                    i32.const 64
                    i32.const 32
                    call $concat
                    i32.const 10
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_executes_node_http_host_imports() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let body = "hello node http";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(response.as_bytes());
    });

    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let url = format!("http://127.0.0.1:{}/", addr.port());
    let wat = format!(
        r#"
            (module
                (import "kali:node" "http_get" (func $http_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $http_get
                    i32.const {}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        url,
        url.len(),
        body.len()
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    server.join().expect("server thread");
}

#[test]
fn runtime_executes_node_process_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        vec!["node".into(), "script.ts".into()],
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))]),
        PathBuf::from("."),
        "node",
    );
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:node" "process_args_len" (func $args_len (result i32)))
                (import "kali:node" "process_args_get" (func $args_get (param i32 i32 i32) (result i32)))
                (import "kali:node" "process_env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "process_stdout_write" (func $stdout_write (param i32 i32) (result i32)))
                (import "kali:node" "process_stderr_write" (func $stderr_write (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "HOME")
                (data (i32.const 32) "script.ts")
                (data (i32.const 64) "node stdout")
                (data (i32.const 96) "node stderr")
                (func (export "_start")
                    call $args_len
                    i32.const 2
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 1
                    i32.const 160
                    i32.const 16
                    call $args_get
                    i32.const 9
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 160
                    i32.load8_u
                    i32.const 115
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 0
                    i32.const 4
                    i32.const 192
                    i32.const 16
                    call $env_get
                    i32.const 9
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 192
                    i32.load8_u
                    i32.const 47
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 64
                    i32.const 11
                    call $stdout_write
                    drop
                    i32.const 96
                    i32.const 11
                    call $stderr_write
                    drop))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "node stdout");
    assert_eq!(outcome.stderr, "node stderr");
}

#[cfg(not(windows))]
#[test]
fn runtime_executes_node_child_process_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let command = "sh";
    let args = "-lc|printf child-process";
    let expected_stdout = "child-process";
    let wat = format!(
        r#"
            (module
                (import "kali:node" "process_spawn" (func $spawn (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{command}")
                (data (i32.const 32) "{args}")
                (func (export "_start")
                    i32.const 0
                    i32.const {command_len}
                    i32.const 32
                    i32.const {args_len}
                    i32.const 96
                    i32.const 32
                    call $spawn
                    i32.const 0
                    i32.ne
                    if
                        unreachable
                    end
{stdout_checks}
                )
            )
            "#,
        command = command,
        command_len = command.len(),
        args = args,
        args_len = args.len(),
        stdout_checks = wat_assert_buffer_eq(96, expected_stdout),
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[cfg(not(windows))]
#[test]
fn runtime_enforces_node_child_process_budget() {
    let policy = SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: kali_sandbox::EffectsPolicy {
            file_system: kali_sandbox::FileSystemPolicy {
                read: kali_sandbox::AccessRule::Deny(false),
                write: kali_sandbox::AccessRule::Deny(false),
            },
            network: kali_sandbox::NetworkPolicy {
                fetch: kali_sandbox::AccessRule::Deny(false),
                connect: kali_sandbox::AccessRule::Deny(false),
                listen: kali_sandbox::AccessRule::Deny(false),
                max_connections: Some(1),
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::AllowList(vec!["sh".into()]),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(1000),
                max_active_timers: Some(1),
            },
            eval: false,
            random: true,
            console: true,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(1000),
            max_open_files: Some(8),
            max_spawned_processes: Some(1),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("."),
        serialized_source: None,
    };
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        Some(policy),
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let wat = r#"
            (module
                (import "kali:node" "process_spawn" (func $spawn (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "sh")
                (data (i32.const 32) "-lc|printf child-process")
                (func (export "_start")
                    i32.const 0
                    i32.const 2
                    i32.const 32
                    i32.const 24
                    i32.const 96
                    i32.const 32
                    call $spawn
                    i32.const 0
                    i32.ne
                    if
                        unreachable
                    end)
            )
            "#;

    let wasm = compile_wat(wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[cfg(not(windows))]
#[test]
fn runtime_rejects_node_child_processes_when_budget_is_zero() {
    let policy = SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: kali_sandbox::EffectsPolicy {
            file_system: kali_sandbox::FileSystemPolicy {
                read: kali_sandbox::AccessRule::Deny(false),
                write: kali_sandbox::AccessRule::Deny(false),
            },
            network: kali_sandbox::NetworkPolicy {
                fetch: kali_sandbox::AccessRule::Deny(false),
                connect: kali_sandbox::AccessRule::Deny(false),
                listen: kali_sandbox::AccessRule::Deny(false),
                max_connections: Some(1),
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::AllowList(vec!["sh".into()]),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(1000),
                max_active_timers: Some(1),
            },
            eval: false,
            random: true,
            console: true,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(1000),
            max_open_files: Some(8),
            max_spawned_processes: Some(0),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("."),
        serialized_source: None,
    };
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        Some(policy),
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let wat = r#"
            (module
                (import "kali:node" "process_spawn" (func $spawn (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "sh")
                (data (i32.const 32) "-lc|printf child-process")
                (func (export "_start")
                    i32.const 0
                    i32.const 2
                    i32.const 32
                    i32.const 24
                    i32.const 96
                    i32.const 32
                    call $spawn
                    drop)
            )
            "#;

    let wasm = compile_wat(wat);
    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("spawn budget should be denied");
    assert_eq!(
        diagnostics[0].code,
        Some(e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
}

#[test]
fn runtime_executes_node_util_buffer_and_assert_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let format_left = "node";
    let format_right = "compat";
    let buffer_input = "hello node";
    let buffer_hex = "68656c6c6f206e6f6465";
    let wat = format!(
        r#"
            (module
                (import "kali:node" "util_format" (func $format (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "buffer_to_hex" (func $buffer_to_hex (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "buffer_from_hex" (func $buffer_from_hex (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "assert_equal" (func $assert_equal (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{format_left}")
                (data (i32.const 32) "{format_right}")
                (data (i32.const 64) "{buffer_input}")
                (data (i32.const 128) "{buffer_hex}")
                (data (i32.const 192) "kali")
                (data (i32.const 224) "kali")
                (func (export "_start")
                    i32.const 0
                    i32.const {format_left_len}
                    i32.const 32
                    i32.const {format_right_len}
                    i32.const 246
                    i32.const 64
                    call $format
                    i32.const {format_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{format_checks}
                    i32.const 64
                    i32.const {buffer_input_len}
                    i32.const 320
                    i32.const 32
                    call $buffer_to_hex
                    i32.const {buffer_hex_len}
                    i32.ne
                    if
                        unreachable
                    end
{buffer_hex_checks}
                    i32.const 128
                    i32.const {buffer_hex_len}
                    i32.const 384
                    i32.const {buffer_input_len}
                    call $buffer_from_hex
                    i32.const {buffer_input_len}
                    i32.ne
                    if
                        unreachable
                    end
{buffer_round_trip_checks}
                    i32.const 192
                    i32.const 4
                    i32.const 224
                    i32.const 4
                    call $assert_equal
                    i32.const 0
                    i32.ne
                    if
                        unreachable
                    end)
            )
            "#,
        format_left = format_left,
        format_left_len = format_left.len(),
        format_right = format_right,
        format_right_len = format_right.len(),
        format_output_len = format!("{} {}", format_left, format_right).len(),
        format_checks = wat_assert_buffer_eq(246, &format!("{} {}", format_left, format_right)),
        buffer_input = buffer_input,
        buffer_input_len = buffer_input.len(),
        buffer_hex_len = buffer_hex.len(),
        buffer_hex_checks = wat_assert_buffer_eq(320, buffer_hex),
        buffer_round_trip_checks = wat_assert_buffer_eq(384, buffer_input),
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_executes_node_event_emitter_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let wat = r#"
            (module
                (import "kali:node" "event_on" (func $event_on (param i32 i32 i32) (result i32)))
                (import "kali:node" "event_listener_count" (func $listener_count (param i32 i32) (result i32)))
                (import "kali:node" "event_emit" (func $event_emit (param i32 i32) (result i32)))
                (import "kali:node" "process_stdout_write" (func $stdout_write (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "message")
                (data (i32.const 32) "event fired")
                (func (export "__kali_callback_1")
                    i32.const 32
                    i32.const 11
                    call $stdout_write
                    drop)
                (func (export "_start")
                    i32.const 0
                    i32.const 7
                    i32.const 1
                    call $event_on
                    drop
                    i32.const 0
                    i32.const 7
                    call $listener_count
                    i32.const 1
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 0
                    i32.const 7
                    call $event_emit
                    i32.const 1
                    i32.ne
                    if
                        unreachable
                    end)
            )
        "#;

    let wasm = compile_wat(wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "event fired");
}

#[test]
fn runtime_executes_node_path_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let normalize_input = "./foo/../bar//baz";
    let join_base = "/tmp";
    let join_segment = "project/src";
    let resolve_base = "/tmp/project";
    let resolve_input = "../lib/index.js";
    let dirname_input = "/tmp/project/src/main.ts";
    let basename_input = "main.ts";
    let extname_input = "main.ts";
    let relative_from = "/tmp/project/src";
    let relative_to = "/tmp/project/lib/index.js";
    let normalized_output = "bar/baz";
    let joined_output = "/tmp/project/src";
    let resolved_output = "/tmp/lib/index.js";
    let dirname_output = "/tmp/project/src";
    let basename_output = "main.ts";
    let extname_output = ".ts";
    let relative_output = "../lib/index.js";
    let wat = format!(
        r#"
            (module
                (import "kali:node" "path_normalize" (func $normalize (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_join" (func $join (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_resolve" (func $resolve (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_dirname" (func $dirname (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_basename" (func $basename (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_extname" (func $extname (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_relative" (func $relative (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{normalize_input}")
                (data (i32.const 64) "{join_base}")
                (data (i32.const 80) "{join_segment}")
                (data (i32.const 112) "{resolve_base}")
                (data (i32.const 144) "{resolve_input}")
                (data (i32.const 192) "{dirname_input}")
                (data (i32.const 224) "{basename_input}")
                (data (i32.const 246) "{extname_input}")
                (data (i32.const 768) "{relative_from}")
                (data (i32.const 832) "{relative_to}")
                (func (export "_start")
                    ;; normalize
                    i32.const 0
                    i32.const {normalize_len}
                    i32.const 320
                    i32.const 32
                    call $normalize
                    i32.const {normalized_len}
                    i32.ne
                    if
                        unreachable
                    end
{normalize_checks}
                    ;; join
                    i32.const 64
                    i32.const {join_base_len}
                    i32.const 80
                    i32.const {join_segment_len}
                    i32.const 384
                    i32.const 32
                    call $join
                    i32.const {joined_len}
                    i32.ne
                    if
                        unreachable
                    end
{join_checks}
                    ;; resolve
                    i32.const 112
                    i32.const {resolve_base_len}
                    i32.const 144
                    i32.const {resolve_input_len}
                    i32.const 448
                    i32.const 32
                    call $resolve
                    i32.const {resolved_len}
                    i32.ne
                    if
                        unreachable
                    end
{resolve_checks}
                    ;; dirname
                    i32.const 192
                    i32.const {dirname_input_len}
                    i32.const 512
                    i32.const 32
                    call $dirname
                    i32.const {dirname_len}
                    i32.ne
                    if
                        unreachable
                    end
{dirname_checks}
                    ;; basename
                    i32.const 224
                    i32.const {basename_input_len}
                    i32.const 576
                    i32.const 32
                    call $basename
                    i32.const {basename_len}
                    i32.ne
                    if
                        unreachable
                    end
{basename_checks}
                    ;; extname
                    i32.const 246
                    i32.const {extname_input_len}
                    i32.const 640
                    i32.const 32
                    call $extname
                    i32.const {extname_len}
                    i32.ne
                    if
                        unreachable
                    end
{extname_checks}
                    ;; relative
                    i32.const 768
                    i32.const {relative_from_len}
                    i32.const 832
                    i32.const {relative_to_len}
                    i32.const 704
                    i32.const 32
                    call $relative
                    i32.const {relative_len}
                    i32.ne
                    if
                        unreachable
                    end
{relative_checks}
                )
            )
            "#,
        normalize_input = normalize_input,
        normalize_len = normalize_input.len(),
        normalized_len = normalized_output.len(),
        normalize_checks = wat_assert_buffer_eq(320, normalized_output),
        join_base = join_base,
        join_base_len = join_base.len(),
        join_segment = join_segment,
        join_segment_len = join_segment.len(),
        joined_len = joined_output.len(),
        join_checks = wat_assert_buffer_eq(384, joined_output),
        resolve_base = resolve_base,
        resolve_base_len = resolve_base.len(),
        resolve_input = resolve_input,
        resolve_input_len = resolve_input.len(),
        resolved_len = resolved_output.len(),
        resolve_checks = wat_assert_buffer_eq(448, resolved_output),
        dirname_input = dirname_input,
        dirname_input_len = dirname_input.len(),
        dirname_len = dirname_output.len(),
        dirname_checks = wat_assert_buffer_eq(512, dirname_output),
        basename_input = basename_input,
        basename_input_len = basename_input.len(),
        basename_len = basename_output.len(),
        basename_checks = wat_assert_buffer_eq(576, basename_output),
        extname_input = extname_input,
        extname_input_len = extname_input.len(),
        extname_len = extname_output.len(),
        extname_checks = wat_assert_buffer_eq(640, extname_output),
        relative_from = relative_from,
        relative_from_len = relative_from.len(),
        relative_to = relative_to,
        relative_to_len = relative_to.len(),
        relative_len = relative_output.len(),
        relative_checks = wat_assert_buffer_eq(704, relative_output),
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_executes_node_url_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let parse_input = "https://example.com/path?query=1";
    let resolve_base = "https://example.com/base/";
    let resolve_input = "../child";
    let parse_output = "https://example.com/path?query=1";
    let resolve_output = "https://example.com/child";
    let wat = format!(
        r#"
            (module
                (import "kali:node" "url_parse" (func $parse (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "url_resolve" (func $resolve (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{parse_input}")
                (data (i32.const 64) "{resolve_base}")
                (data (i32.const 96) "{resolve_input}")
                (func (export "_start")
                    i32.const 0
                    i32.const {parse_len}
                    i32.const 246
                    i32.const 64
                    call $parse
                    i32.const {parse_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{parse_checks}
                    i32.const 64
                    i32.const {resolve_base_len}
                    i32.const 96
                    i32.const {resolve_input_len}
                    i32.const 320
                    i32.const 64
                    call $resolve
                    i32.const {resolve_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{resolve_checks}
                )
            )
            "#,
        parse_input = parse_input,
        parse_len = parse_input.len(),
        parse_output_len = parse_output.len(),
        parse_checks = wat_assert_buffer_eq(246, parse_output),
        resolve_base = resolve_base,
        resolve_base_len = resolve_base.len(),
        resolve_input = resolve_input,
        resolve_input_len = resolve_input.len(),
        resolve_output_len = resolve_output.len(),
        resolve_checks = wat_assert_buffer_eq(320, resolve_output),
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_executes_node_crypto_and_os_host_imports() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        capture_env(),
        PathBuf::from("."),
        "node",
    );
    let hash_input = "hello";
    let hmac_key = "key";
    let hmac_input = "The quick brown fox jumps over the lazy dog";
    let expected_hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    let expected_hmac = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";
    let expected_platform = std::env::consts::OS;
    let expected_arch = std::env::consts::ARCH;
    let expected_eol = if cfg!(windows) { "\r\n" } else { "\n" };
    let eol_checks = if expected_eol.len() == 1 {
        format!(
                "                    i32.const 288\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n",
                expected_eol.as_bytes()[0]
            )
    } else {
        format!(
                "                    i32.const 288\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n                    i32.const 289\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n",
                expected_eol.as_bytes()[0],
                expected_eol.as_bytes()[1]
            )
    };
    let wat = format!(
        r#"
            (module
                (import "kali:node" "crypto_create_hash" (func $hash (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "crypto_create_hmac" (func $hmac (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "crypto_random_uuid" (func $uuid (param i32 i32) (result i32)))
                (import "kali:node" "os_platform" (func $platform (param i32 i32) (result i32)))
                (import "kali:node" "os_arch" (func $arch (param i32 i32) (result i32)))
                (import "kali:node" "os_eol" (func $eol (param i32 i32) (result i32)))
                (import "kali:node" "os_cpus" (func $cpus (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{hash_input}")
                (data (i32.const 32) "sha256")
                (data (i32.const 64) "{hmac_key}")
                (data (i32.const 96) "{hmac_input}")
                (data (i32.const 160) "{expected_platform}")
                (data (i32.const 224) "{expected_arch}")
                (func (export "_start")
                    i32.const 32
                    i32.const 6
                    i32.const 0
                    i32.const {hash_input_len}
                    i32.const 320
                    i32.const 80
                    call $hash
                    i32.const {expected_hash_len}
                    i32.ne
                    if
                        unreachable
                    end
{hash_checks}
                    i32.const 32
                    i32.const 6
                    i32.const 64
                    i32.const {hmac_key_len}
                    i32.const 96
                    i32.const {hmac_input_len}
                    i32.const 416
                    i32.const 80
                    call $hmac
                    i32.const {expected_hmac_len}
                    i32.ne
                    if
                        unreachable
                    end
{hmac_checks}
                    i32.const 480
                    i32.const 36
                    call $uuid
                    i32.const 36
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 488
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 493
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 494
                    i32.load8_u
                    i32.const 52
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 498
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 503
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 160
                    i32.const {expected_platform_len}
                    call $platform
                    i32.const {expected_platform_len}
                    i32.ne
                    if
                        unreachable
                    end
{platform_checks}
                    i32.const 224
                    i32.const {expected_arch_len}
                    call $arch
                    i32.const {expected_arch_len}
                    i32.ne
                    if
                        unreachable
                    end
{arch_checks}
                    i32.const 288
                    i32.const {expected_eol_len}
                    call $eol
                    i32.const {expected_eol_len}
                    i32.ne
                    if
                        unreachable
                    end
{eol_checks}
                    call $cpus
                    i32.const 1
                    i32.lt_s
                    if
                        unreachable
                    end)
            )
            "#,
        hash_input = hash_input,
        hash_input_len = hash_input.len(),
        expected_hash_len = expected_hash.len(),
        hash_checks = wat_assert_buffer_eq(320, expected_hash),
        hmac_key = hmac_key,
        hmac_key_len = hmac_key.len(),
        hmac_input = hmac_input,
        hmac_input_len = hmac_input.len(),
        expected_hmac_len = expected_hmac.len(),
        hmac_checks = wat_assert_buffer_eq(416, expected_hmac),
        expected_platform = expected_platform,
        expected_platform_len = expected_platform.len(),
        platform_checks = wat_assert_buffer_eq(160, expected_platform),
        expected_arch = expected_arch,
        expected_arch_len = expected_arch.len(),
        arch_checks = wat_assert_buffer_eq(224, expected_arch),
        expected_eol_len = expected_eol.len(),
        eol_checks = eol_checks,
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_exposes_performance_now() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "performance_now" (func $now (result f64)))
                (func (export "_start")
                    call $now
                    f64.const 0.0
                    f64.ge
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_fills_random_values() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "crypto_get_random_values" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 16
                    call $random
                    i32.const 16
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_exposes_crypto_random_uuid() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "cryptoRandomUUID" (func $uuid (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 36
                    call $uuid
                    i32.const 36
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_rejects_console_calls_when_policy_denies_them() {
    let policy = SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: kali_sandbox::EffectsPolicy {
            file_system: kali_sandbox::FileSystemPolicy {
                read: kali_sandbox::AccessRule::Deny(false),
                write: kali_sandbox::AccessRule::Deny(false),
            },
            network: kali_sandbox::NetworkPolicy {
                fetch: kali_sandbox::AccessRule::Deny(false),
                connect: kali_sandbox::AccessRule::Deny(false),
                listen: kali_sandbox::AccessRule::Deny(false),
                max_connections: Some(1),
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::Deny(false),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(1000),
                max_active_timers: Some(1),
            },
            eval: false,
            random: true,
            console: false,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(1000),
            max_open_files: Some(8),
            max_spawned_processes: Some(0),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("."),
        serialized_source: None,
    };
    let runtime =
        RuntimeCtx::with_host_context(Some(policy), Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("console should be denied");
    assert_eq!(diagnostics[0].code, Some(e4::EFFECT_NOT_PERMITTED as u32));
}

#[test]
fn runtime_drains_microtasks_before_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    i32.const 1
                    global.set $state)
                (func (export "__kali_callback_2")
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "_start")
                    i32.const 1
                    call $queue_microtask
                    i32.const 2
                    i32.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_repeating_intervals_can_be_cleared_from_callbacks() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32) (result i32)))
                (import "kali:rt" "clearInterval" (func $clear_interval (param i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (global $timer_id (mut i32) (i32.const -1))
                (func (export "__kali_callback_3")
                    global.get $state
                    i32.const 1
                    i32.add
                    global.set $state
                    global.get $state
                    i32.const 2
                    i32.eq
                    if
                        global.get $timer_id
                        call $clear_interval
                    else
                        global.get $state
                        i32.const 2
                        i32.gt_s
                        if
                            unreachable
                        end
                    end)
                (func (export "_start")
                    i32.const 3
                    i32.const 0
                    call $set_interval
                    global.set $timer_id)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_traps_from_the_entrypoint() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    i64.const 0
                    i64.div_s
                    drop)
            )
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("division by zero should trap");
    assert_eq!(diagnostics[0].code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(diagnostics[0].message.contains("runtime trap"));
}

#[test]
fn runtime_can_clear_scheduled_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (import "kali:rt" "clearTimeout" (func $clear_timeout (param i32)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i32.const 7
                    i32.const 0
                    call $set_timeout
                    call $clear_timeout)
            )
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_rejects_negative_timer_delays() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (func (export "__kali_callback_9")
                    unreachable)
                (func (export "_start")
                    i32.const 9
                    i32.const -1
                    call $set_timeout
                    drop)
            )
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("negative timer delays should be rejected");
    assert_eq!(
        diagnostics[0].code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostics[0].message.contains("runtime trap"));
}

#[test]
fn runtime_rejects_negative_interval_delays() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32) (result i32)))
                (func (export "__kali_callback_10")
                    unreachable)
                (func (export "_start")
                    i32.const 10
                    i32.const -1
                    call $set_interval
                    drop)
            )
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("negative timer delays should be rejected");
    assert_eq!(
        diagnostics[0].code,
        Some(kali_error::_error_codes::e4::UNCAUGHT_ERROR as u32)
    );
    assert!(diagnostics[0].message.contains("runtime trap"));
}

#[test]
fn runtime_collects_and_runs_registered_tests() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_1")
                    i32.const 1
                    i32.const 1
                    i32.add
                    drop)
                (func (export "_start")
                    i32.const 1
                    call $test_register)
            )
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 0);
}

#[test]
fn runtime_reports_failed_registered_tests() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_2")
                    unreachable)
                (func (export "_start")
                    i32.const 2
                    call $test_register)
            )
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(outcome.tests_failed, 1);
}
