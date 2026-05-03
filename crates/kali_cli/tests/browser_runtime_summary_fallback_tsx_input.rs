use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn write_tsx_source(path: &std::path::Path) {
    fs::write(
        path,
        "Kali.test('browser unparseable', () => { console.log('browser unparseable'); });\n",
    )
    .expect("write source");
}

fn assert_browser_summary_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    json
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.test.tsx");
    write_tsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unparseable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    let json = assert_browser_summary_json(&output);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.tsx");
    write_tsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("browser unparseable\n");'"#,
        )
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("browser unparseable"), "stdout: {stdout}");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
