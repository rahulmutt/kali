use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_object_has_own_source() -> &'static str {
    r#"const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]));
const alias = object;
const wrapped = (0, alias);
const frozenHasOwn = Object.freeze(globalThis.Object["hasOwn"]);
const frozenParenthesizedHasOwn = Object.freeze((globalThis.Object["hasOwn"]));
const frozenBracketedHasOwn = Object.freeze(globalThis["Object"]["hasOwn"]);
const frozenParenthesizedBracketedHasOwn = Object.freeze((globalThis["Object"]["hasOwn"]));
if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !frozenHasOwn(wrapped, "a") || !frozenParenthesizedHasOwn(wrapped, "a") || !frozenBracketedHasOwn(wrapped, "a") || !frozenParenthesizedBracketedHasOwn(wrapped, "a") || !Object.prototype.hasOwnProperty.call(wrapped, "a")) {
  throw new Error('unexpected frozen Object.hasOwn result');
}
console.log('frozen object hasOwn ok');
"#
}

fn frozen_object_has_own_test_source() -> &'static str {
    r#"Kali.test('frozen object hasOwn', () => {
  const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]));
  const alias = object;
  const wrapped = (0, alias);
  const frozenHasOwn = Object.freeze(globalThis.Object["hasOwn"]);
  const frozenParenthesizedHasOwn = Object.freeze((globalThis.Object["hasOwn"]));
  const frozenBracketedHasOwn = Object.freeze(globalThis["Object"]["hasOwn"]);
  const frozenParenthesizedBracketedHasOwn = Object.freeze((globalThis["Object"]["hasOwn"]));
  if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !frozenHasOwn(wrapped, "a") || !frozenParenthesizedHasOwn(wrapped, "a") || !frozenBracketedHasOwn(wrapped, "a") || !frozenParenthesizedBracketedHasOwn(wrapped, "a") || !Object.prototype.hasOwnProperty.call(wrapped, "a")) {
    throw new Error('unexpected frozen Object.hasOwn result');
  }
});
"#
}

fn assert_frozen_object_has_own(command: &str, filename: &str, source: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output.current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("frozen object hasOwn ok"));
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["payload"]["skipped"], 0);
        }
    } else if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("frozen object hasOwn ok"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn check_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_frozen_object_has_own("check", filename, frozen_object_has_own_source(), false);
    }
}

#[test]
fn run_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_frozen_object_has_own("run", filename, frozen_object_has_own_source(), false);
    }
}

#[test]
fn json_run_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_frozen_object_has_own("run", filename, frozen_object_has_own_source(), true);
    }
}

#[test]
fn test_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in [
        "main.test.js",
        "main.test.ts",
        "main.test.jsx",
        "main.test.tsx",
    ] {
        assert_frozen_object_has_own("test", filename, frozen_object_has_own_test_source(), false);
    }
}

#[test]
fn json_test_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in [
        "main.test.js",
        "main.test.ts",
        "main.test.jsx",
        "main.test.tsx",
    ] {
        assert_frozen_object_has_own("test", filename, frozen_object_has_own_test_source(), true);
    }
}
