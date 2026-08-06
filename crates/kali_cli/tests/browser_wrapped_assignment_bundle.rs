use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

// fasta Spec 7 Task 3: scalar `??=` rejects fail-closed (null and 0 are
// indistinguishable for a scalar), so the wrapped `??=` coverage rides the one
// surviving lowering — a for-in-key ALIAS binding (`-1` null sentinel). The
// scalar `||=`/`&&=` targets keep their falsy semantics and stay unchanged.
fn browser_bundle_wrapped_assignment_source() -> &'static str {
    r##"// kali-tree-shake: browserWrappedAssignmentTargets
export function browserWrappedAssignmentTargets() {
  var table = { a: 1, b: 2 };
  var last = null;
  for (var c in table) {
    last = c;
  }
  ((last)) ??= null;
  var nullishOk = 0;
  if (last) { nullishOk = 1; }

  let left = 0;
  ((left)) ||= 1;
  let right = 1;
  ((right)) &&= 2;
  if (nullishOk !== 1 || left !== 1 || right !== 2) {
    throw new Error(`unexpected wrapped assignment result ${nullishOk} ${left} ${right}`);
  }
}
"##
}
fn assert_browser_bundle_wrapped_assignment(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_wrapped_assignment_source()).expect("write source");

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
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserWrappedAssignmentTargets();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime_contract::browser_harness_command_parts_for(
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
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn build_emits_wrapped_nullish_and_logical_assignment_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_assignment(filename, false);
    }
}

#[test]
fn json_build_emits_wrapped_nullish_and_logical_assignment_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_assignment(filename, true);
    }
}
