use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"const left = [1, 2];
const right = [3, 4];
console.log(left.concat(right, 5)[2]);
console.log([1].concat(Object.freeze(2), [3])[1]);
console.log([1].concat([2])[4]);
"#
}

#[test]
fn run_supports_static_array_concat_direct_index_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-concat.js");
    fs::write(&source_path, supported_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "3\n2\nundefined\n");
}

#[test]
fn json_check_rejects_dynamic_array_concat_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-concat.ts");
    fs::write(
        &source_path,
        "function join(value: number) { return [0].concat(value); }\n",
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

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert!(json["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .any(|error| {
            error["code"] == "E5506"
                && error["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("Array.prototype.concat"))
        }));
}
