use super::*;

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_missing_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("missing-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'process.stdout.write("browser missing summary\n");'"#,
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
        stdout.contains("browser missing summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.jsx");
    write_jsx_source(&source_path);

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
fn json_run_canonicalizes_whitespace_padded_host_labels_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("padded-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser padded labels\"],\"testsFailed\":0,\"hostContract\":\" browser-requested \",\"runtimeBackend\":\" browser-harness \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser padded labels\"],\"testsFailed\":0,\"hostContract\":\" browser-requested \",\"runtimeBackend\":\" browser-harness \"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser padded labels"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_whitespace_only_host_labels_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("whitespace-labels-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":0,\"hostContract\":\"   \",\"runtimeBackend\":\" \t \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser whitespace labels"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_whitespace_only_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("whitespace-summary.jsx");
    write_jsx_source(&source_path);

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
fn run_falls_back_to_stdout_when_browser_summary_file_is_empty_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("empty-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, ""); process.stdout.write("browser empty summary\n");'"#,
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
    assert!(stdout.contains("browser empty summary"), "stdout: {stdout}");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_failed_type_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-summary.jsx");
    write_jsx_source(&source_path);

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
fn run_falls_back_to_stdout_when_browser_summary_file_has_null_args_and_tests_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("null-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":null,\"tests\":null,\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("browser null summary\n");'"#,
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
    assert!(stdout.contains("browser null summary"), "stdout: {stdout}");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_has_invalid_numeric_tests_failed_value_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-numeric-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid numeric summary\"],\"testsFailed\":-1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("browser invalid numeric summary\n");'"#,
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
        stdout.contains("browser invalid numeric summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.jsx");
    write_jsx_source(&source_path);

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

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_is_missing_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("missing-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'process.stdout.write("browser missing summary\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser missing summary"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_is_empty_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("empty-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, ""); process.stdout.write("browser empty summary\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser empty summary"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[cfg(unix)]
#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.jsx");
    write_jsx_source(&source_path);

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

#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_thread_topology_script_url_is_whitespace_padded_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("thread-topology-summary.jsx");
    write_jsx_source(&source_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\" https://example.com/thread.js \",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("browser thread topology summary\n");'"#,
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
        stdout.contains("browser thread topology summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
