use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_set_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserSetIteration
export async function browserSetIteration() {
  function assertSetIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Set constructor iteration semantics');
    }
  }

  const values = [1, 2, 1];
  const setAlias = Set;
  const wrappedSetAlias = (setAlias);
  const aliasValues = (values);
  const direct = [];
  for (const value of new Set(values)) {
    direct.push(value);
  }
  const alias = [];
  for (const value of new setAlias(aliasValues)) {
    alias.push(value);
  }
  const wrappedAlias = [];
  for (const value of new (wrappedSetAlias)(aliasValues)) {
    wrappedAlias.push(value);
  }
  const globalDirect = [];
  for (const value of new globalThis.Set(values)) {
    globalDirect.push(value);
  }
  const bracketed = [];
  for (const value of new globalThis["Set"](values)) {
    bracketed.push(value);
  }
  const singleBracketed = [];
  for (const value of new globalThis['Set'](values)) {
    singleBracketed.push(value);
  }
  const frozenValues = Object.freeze(aliasValues);
  const frozenSet = Object.freeze(Set);
  const frozenDirect = [];
  for (const value of new Set(frozenValues)) {
    frozenDirect.push(value);
  }
  const frozenAlias = [];
  for (const value of new (frozenSet)(values)) {
    frozenAlias.push(value);
  }

  assertSetIteration(direct);
  assertSetIteration(alias);
  assertSetIteration(wrappedAlias);
  assertSetIteration(globalDirect);
  assertSetIteration(bracketed);
  assertSetIteration(singleBracketed);
  assertSetIteration(frozenDirect);
  assertSetIteration(frozenAlias);
  console.log('browser set constructor iteration ok');
}
"##
}

fn browser_bundle_map_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserMapIteration
export async function browserMapIteration() {
  function assertMapIteration(values) {
    if (values.length !== 2 || values[0] !== '[1,3]' || values[1] !== '[4,5]') {
      throw new Error('unexpected Map constructor iteration semantics');
    }
  }

  const values = [[1, 2], [1, 3], [4, 5]];
  const mapAlias = Map;
  const wrappedMapAlias = (mapAlias);
  const aliasValues = (values);
  const direct = [];
  for (const entry of new Map(values)) {
    direct.push(JSON.stringify(entry));
  }
  const alias = [];
  for (const entry of new mapAlias(aliasValues)) {
    alias.push(JSON.stringify(entry));
  }
  const wrappedAlias = [];
  for (const entry of new (wrappedMapAlias)(aliasValues)) {
    wrappedAlias.push(JSON.stringify(entry));
  }
  const globalDirect = [];
  for (const entry of new globalThis.Map(values)) {
    globalDirect.push(JSON.stringify(entry));
  }
  const bracketed = [];
  for (const entry of new globalThis["Map"](values)) {
    bracketed.push(JSON.stringify(entry));
  }
  const singleBracketed = [];
  for (const entry of new globalThis['Map'](values)) {
    singleBracketed.push(JSON.stringify(entry));
  }
  const frozenMapValues = Object.freeze(aliasValues);
  const frozenMap = Object.freeze(Map);
  const frozenDirect = [];
  for (const entry of new Map(frozenMapValues)) {
    frozenDirect.push(JSON.stringify(entry));
  }
  const frozenAlias = [];
  for (const entry of new (frozenMap)(values)) {
    frozenAlias.push(JSON.stringify(entry));
  }

  assertMapIteration(direct);
  assertMapIteration(alias);
  assertMapIteration(wrappedAlias);
  assertMapIteration(globalDirect);
  assertMapIteration(bracketed);
  assertMapIteration(singleBracketed);
  assertMapIteration(frozenDirect);
  assertMapIteration(frozenAlias);
  console.log('browser map constructor iteration ok');
}
"##
}

fn assert_browser_bundle_iteration(
    filename: &str,
    json_output: bool,
    source: &str,
    exported_fn: &str,
    expected_stdout: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

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
        &format!("const mod = await import(bundleJs.href);\nawait mod.{exported_fn}();\n"),
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
    assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
}

fn assert_browser_bundle_set_iteration(filename: &str, json_output: bool) {
    assert_browser_bundle_iteration(
        filename,
        json_output,
        browser_bundle_set_iteration_source(),
        "browserSetIteration",
        "browser set constructor iteration ok",
    );
}

fn assert_browser_bundle_map_iteration(filename: &str, json_output: bool) {
    assert_browser_bundle_iteration(
        filename,
        json_output,
        browser_bundle_map_iteration_source(),
        "browserMapIteration",
        "browser map constructor iteration ok",
    );
}

#[test]
fn build_emits_set_constructor_iteration_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_bundle_set_iteration(&format!("app.{extension}"), false);
    }
}

#[test]
fn json_build_emits_set_constructor_iteration_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_bundle_set_iteration(&format!("app.{extension}"), true);
    }
}

#[test]
fn build_emits_map_constructor_iteration_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_bundle_map_iteration(&format!("app.{extension}"), false);
    }
}

#[test]
fn json_build_emits_map_constructor_iteration_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_bundle_map_iteration(&format!("app.{extension}"), true);
    }
}
