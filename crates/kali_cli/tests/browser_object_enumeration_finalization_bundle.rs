use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_object_enumeration_finalization_source() -> &'static str {
    r#"// kali-tree-shake: browserObjectEnumerationFinalizationWrapper
async function browserObjectEnumerationFinalizationWrapper() {
  const values = { "b": 1, "a": 2 };
  let returnFinally = false;
  function returnProbe() {
    try {
      for (const key of Object.keys(values)) {
        return key;
      }
      throw new Error('unexpected empty Object.keys iteration');
    } finally {
      returnFinally = true;
    }
  }
  const returnValue = returnProbe();
  if (returnValue !== 'b' || !returnFinally) {
    throw new Error('unexpected Object.keys return/finally semantics');
  }

  let throwFinally = false;
  function throwProbe() {
    try {
      for (const entry of Object.entries(values)) {
        if (entry[0] === 'b') {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Object.entries iteration');
    } finally {
      throwFinally = true;
    }
  }
  let threw = false;
  try {
    throwProbe();
  } catch {
    threw = true;
  }
  if (!threw || !throwFinally) {
    throw new Error('unexpected Object.entries throw/finally semantics');
  }

  const asyncValues = { "b": 1, "a": 2 };
  let asyncFinallySeen = false;
  let asyncThrew = false;
  try {
    for await (const key of Object.keys(asyncValues)) {
      if (key === 'b') {
        throw new Error('boom');
      }
    }
    throw new Error('unexpected empty async Object.keys iteration');
  } catch {
    asyncThrew = true;
  } finally {
    asyncFinallySeen = true;
  }
  if (!asyncThrew || !asyncFinallySeen) {
    throw new Error('unexpected async Object.keys throw/finally semantics');
  }

  console.log('browser object enumeration finalization ok');
}
"#
}

fn assert_browser_object_enumeration_finalization(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_object_enumeration_finalization_source(),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.browserObjectEnumerationFinalizationWrapper();\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("browser object enumeration finalization ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_object_enumeration_finalization_in_js_input() {
    assert_browser_object_enumeration_finalization("app.js", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_js_input() {
    assert_browser_object_enumeration_finalization("app.js", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_ts_input() {
    assert_browser_object_enumeration_finalization("app.ts", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_ts_input() {
    assert_browser_object_enumeration_finalization("app.ts", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_jsx_input() {
    assert_browser_object_enumeration_finalization("app.jsx", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_jsx_input() {
    assert_browser_object_enumeration_finalization("app.jsx", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_tsx_input() {
    assert_browser_object_enumeration_finalization("app.tsx", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_tsx_input() {
    assert_browser_object_enumeration_finalization("app.tsx", true);
}
