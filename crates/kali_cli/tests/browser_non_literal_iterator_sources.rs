use std::{fs, process::Command};

use kali_runtime::BROWSER_HARNESS_COMMAND_ENV;
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_of_source() -> &'static str {
    r#"function main() {
  let values = [1, 2];
  values = values;
  for (const item of values) {
    console.log(item);
  }
}
main();
"#
}

fn for_await_source() -> &'static str {
    r#"async function main() {
  let values = [1, 2];
  values = values;
  for await (const item of values) {
    console.log(item);
  }
}
main();
"#
}

fn object_keys_source() -> &'static str {
    r#"function main() {
  let values = { a: 1 };
  values = values;
  for (const key of Object.keys(values)) {
    console.log(key);
  }
}
main();
"#
}

fn object_values_source() -> &'static str {
    r#"function main() {
  let values = { a: 1 };
  values = values;
  for (const value of Object.values(values)) {
    console.log(value);
  }
}
main();
"#
}

fn object_entries_source() -> &'static str {
    r#"function main() {
  let values = { a: 1 };
  values = values;
  for (const entry of Object.entries(values)) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
main();
"#
}

fn array_callback_iteration_source() -> &'static str {
    r#"function main() {
  const values = [1, 2];
  for (const item of values.filter((value) => value + 1)) {
    console.log(item);
  }
}
main();
"#
}

fn set_constructor_call_expression_source() -> &'static str {
    r#"function main() {
  let values = [1, 2];
  values = values;
  for (const value of new Set(values.filter(Boolean))) {
    console.log(value);
  }
}
main();
"#
}

fn map_constructor_call_expression_source() -> &'static str {
    r#"function main() {
  let values = [[1, 2], [3, 4]];
  values = values;
  for (const entry of new Map(values.filter(Boolean))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
main();
"#
}

fn assert_browser_iterator_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
    bundle: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cmd = Command::new(kali_bin());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    cmd.current_dir(dir.path());
    cmd.arg(command);
    if bundle {
        cmd.arg("--bundle");
    }
    let output = cmd
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], if bundle { "build" } else { "check" });
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_inherited_browser_iterator_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
    bundle: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let mut cmd = Command::new(kali_bin());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    cmd.current_dir(dir.path());
    cmd.arg(command);
    if bundle {
        cmd.arg("--bundle");
    }
    let output = cmd.arg(&source_path).output().expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], if bundle { "build" } else { "check" });
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_browser_requested_iterator_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cmd = Command::new(kali_bin());
    cmd.env(BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    let output = cmd
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_browser_array_callback_iteration_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
    bundle: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cmd = Command::new(kali_bin());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    cmd.current_dir(dir.path());
    cmd.arg(command);
    if bundle {
        cmd.arg("--bundle");
    }
    let output = cmd
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], if bundle { "build" } else { "check" });
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("array callback-produced iterables")
                || message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("array callback-produced iterables")
                || stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_inherited_browser_array_callback_iteration_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
    bundle: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let mut cmd = Command::new(kali_bin());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    cmd.current_dir(dir.path());
    cmd.arg(command);
    if bundle {
        cmd.arg("--bundle");
    }
    let output = cmd.arg(&source_path).output().expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], if bundle { "build" } else { "check" });
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("array callback-produced iterables")
                || message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("array callback-produced iterables")
                || stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_browser_requested_array_callback_iteration_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cmd = Command::new(kali_bin());
    cmd.env(BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    let output = cmd
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("array callback-produced iterables")
                || message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("array callback-produced iterables")
                || stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

fn assert_map_constructor_iteration_from_call_expression_source_rejects(
    command: &str,
    bundle: bool,
) {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_browser_iterator_source_rejects(
                map_constructor_call_expression_source(),
                filename,
                json_output,
                command,
                bundle,
            );
        }
    }
}

#[path = "browser_non_literal_iterator_sources/run.rs"]
mod run;

#[path = "browser_non_literal_iterator_sources/build.rs"]
mod build;

#[path = "browser_non_literal_iterator_sources/check.rs"]
mod check;

#[path = "browser_non_literal_iterator_sources/test.rs"]
mod test;
