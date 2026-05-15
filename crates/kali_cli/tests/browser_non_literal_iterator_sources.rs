use std::{fs, process::Command};

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

#[test]
fn build_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", false, "build", true);
}

#[test]
fn json_build_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", true, "build", true);
}

#[test]
fn build_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", false, "build", true);
}

#[test]
fn json_build_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", true, "build", true);
}

#[test]
fn build_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", false, "build", true);
}

#[test]
fn json_build_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", true, "build", true);
}

#[test]
fn build_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", false, "build", true);
}

#[test]
fn json_build_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", true, "build", true);
}

#[test]
fn check_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", true, "check", false);
}

#[test]
fn build_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", false, "build", true);
}

#[test]
fn build_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_values_source(), "main.js", false, "build", true);
}

#[test]
fn build_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.js",
        false,
        "build",
        true,
    );
}

#[test]
fn json_build_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", true, "build", true);
}

#[test]
fn json_build_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_values_source(), "main.js", true, "build", true);
}

#[test]
fn json_build_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_entries_source(), "main.js", true, "build", true);
}

#[test]
fn build_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", false, "build", true);
}

#[test]
fn build_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.ts",
        false,
        "build",
        true,
    );
}

#[test]
fn json_build_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", true, "build", true);
}

#[test]
fn json_build_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_entries_source(), "main.ts", true, "build", true);
}

#[test]
fn build_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", false, "build", true);
}

#[test]
fn build_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        false,
        "build",
        true,
    );
}

#[test]
fn json_build_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", true, "build", true);
}

#[test]
fn json_build_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        true,
        "build",
        true,
    );
}

#[test]
fn build_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", false, "build", true);
}

#[test]
fn build_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        false,
        "build",
        true,
    );
}

#[test]
fn json_build_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", true, "build", true);
}

#[test]
fn json_build_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        true,
        "build",
        true,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_values_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_values_source(), "main.js", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.js",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.ts",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.ts",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        true,
        "check",
        false,
    );
}

#[test]
fn build_rejects_non_literal_object_keys_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_keys_source(),
                filename,
                json_output,
                "build",
                true,
            );
        }
    }
}

#[test]
fn build_rejects_non_literal_object_values_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_values_source(),
                filename,
                json_output,
                "build",
                true,
            );
        }
    }
}

#[test]
fn build_rejects_non_literal_object_entries_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_entries_source(),
                filename,
                json_output,
                "build",
                true,
            );
        }
    }
}

#[test]
fn build_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        false,
        "build",
        true,
    );
}

#[test]
fn json_build_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        true,
        "build",
        true,
    );
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_keys_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_non_literal_object_values_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_values_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_entries_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}
