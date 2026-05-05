use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn write_ts_source(path: &std::path::Path) {
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.test.ts");
    write_ts_source(&source_path);

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
fn run_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.ts");
    write_ts_source(&source_path);

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

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_whitespace_only_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("whitespace-summary.test.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
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
fn run_falls_back_to_stdout_when_browser_summary_file_is_whitespace_only_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("whitespace-summary.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("browser whitespace summary\n");'"#,
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
    assert!(
        stdout.contains("browser whitespace summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_empty_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("empty-summary.test.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, ""); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser empty summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_failed_type_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-summary.test.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
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
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 7);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout")
        .contains("\"testsFailed\":7"));
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_failed_type_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-summary.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("browser invalid summary\n");'"#,
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
    assert!(
        stdout.contains("browser invalid summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_args_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-args.test.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid args\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
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
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout")
        .contains("\"args\":[\"stdout\"]"));
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"stdout\"],\"tests\":[1],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("browser invalid tests array items\n");'"#,
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
    assert!(
        stdout.contains("browser invalid tests array items"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[cfg(unix)]
#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.test.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
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

#[cfg(unix)]
#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.ts");
    write_ts_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("browser unreadable summary\n");'"#,
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
    assert!(
        stdout.contains("browser unreadable summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
