use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn run_supports_static_array_is_array_slice_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
const arr = [1, 2];
const obj = { a: 1 };
const frozen = Object.freeze([3]);
console.log(Array.isArray(arr));
console.log(Array["isArray"](obj));
console.log(globalThis["Array"]["isArray"](frozen));
console.log(globalThis.Array.isArray(Array.from([4])));
console.log(Array.isArray(new Set([1, 2, 1])));
console.log(globalThis.Array.isArray(new globalThis["Map"]([[1, 2]])));
console.log(Array.isArray("x"));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n0\n1\n1\n0\n0\n0\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_rejects_dynamic_array_is_array_argument_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function accepts(value) { return Array.isArray(value); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "check unexpectedly succeeded");
    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("message")
        .contains("Array.isArray is unavailable unless the argument is a statically-known array"));
}
