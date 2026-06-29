use std::{fs, path::PathBuf, process::Command};

use serde_json::{json, Value};
use tempfile::tempdir;

use kali_common::{
    process_kill_zero_probe_console_log_source, process_kill_zero_probe_guard_source,
    process_kill_zero_probe_node_api_surface_run_source,
    process_kill_zero_probe_node_api_surface_test_source, process_kill_zero_probe_satisfies_source,
    process_kill_zero_probe_source, process_kill_zero_probe_type_assertion_source,
};

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_kali not set")
}

fn assert_node_api_succeeds(name: &str, mut command: Command, expected_stdout: &str) {
    let output = command.output().expect("run kali");
    assert!(
        output.status.success(),
        "{name} stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(expected_stdout), "{name} stdout: {stdout}");
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn json_error_codes(errors: &[Value]) -> Vec<String> {
    errors
        .iter()
        .map(|error| error["code"].as_str().expect("error code").to_string())
        .collect()
}

fn assert_json_error_codes_contain(errors: &[Value], expected: &[&str]) {
    let codes = json_error_codes(errors);
    for expected in expected {
        assert!(
            codes.iter().any(|code| code == expected),
            "missing {expected} in {codes:?}"
        );
    }
}

fn assert_json_node_builtin_rejection(errors: &[Value], expected_message: &str) {
    assert_json_error_codes_contain(errors, &["E5506"]);
    assert!(
        errors
            .iter()
            .any(|error| { error["message"].as_str().expect("error message") == expected_message }),
        "missing node builtin rejection message `{expected_message}` in {errors:?}"
    );
}

fn process_kill_zero_probe_run_source() -> String {
    format!(
        "{} {}\n",
        process_kill_zero_probe_console_log_source(),
        process_kill_zero_probe_source(),
    )
}

fn process_kill_zero_probe_test_source() -> String {
    format!(
        "Kali.test('process kill', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
        process_kill_zero_probe_guard_source()
    )
}

fn process_kill_optional_chain_zero_probe_guard_source() -> String {
    [
        r#"!process?.kill(0)"#,
        r#"!process?.kill(+0)"#,
        r#"!globalThis.process?.kill(0)"#,
        r#"!globalThis.process?.kill(+0)"#,
        r#"!globalThis["process"]?.kill(0)"#,
        r#"!globalThis["process"]?.kill(+0)"#,
    ]
    .join(" || ")
}

fn process_kill_optional_chain_zero_probe_run_source() -> String {
    format!(
        "const zero = 0; const zeroAlias = zero; if ({}) {{ throw new Error('expected zero probe'); }} console.log(1);\n",
        process_kill_optional_chain_zero_probe_guard_source()
    )
}

fn process_kill_optional_chain_zero_probe_test_source() -> String {
    format!(
        "Kali.test('process kill optional chain', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
        process_kill_optional_chain_zero_probe_guard_source()
    )
}

#[path = "node_api_surface/core.rs"]
mod core;

#[path = "node_api_surface/explicit.rs"]
mod explicit;

#[path = "node_api_surface/inherited.rs"]
mod inherited;
