use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn write_js_source(path: &std::path::Path) {
    fs::write(
        path,
        "Kali.test('browser unparseable', () => { console.log('browser unparseable'); });\n",
    )
    .expect("write source");
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    json
}

/// Twin of `parse_json_stdout` for a run whose summary reports failed tests:
/// under the honest Stage-0 predicate (`test_run_succeeded`), `kali test`
/// exits non-zero with `success:false` while still carrying the
/// fallback-parsed metadata. (throw-fallout Stage 0 re-pin.)
fn parse_failed_json_stdout(output: &std::process::Output) -> Value {
    assert!(
        !output.status.success(),
        "expected non-zero exit for a summary reporting failed tests\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    json
}

#[path = "browser_runtime_summary_fallback_js_input/run.rs"]
mod run;

#[path = "browser_runtime_summary_fallback_js_input/test.rs"]
mod test;
