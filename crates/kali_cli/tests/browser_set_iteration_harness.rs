use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_set_iteration_run_source() -> &'static str {
    r##"function browserSetIteration() {
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

  let returnFinally = false;
  function setReturnProbe() {
    try {
      for (const value of new Set(values)) {
        return value;
      }
      throw new Error('unexpected empty Set constructor iteration');
    } finally {
      returnFinally = true;
    }
  }
  const returnValue = setReturnProbe();
  if (returnValue !== 1 || !returnFinally) {
    throw new Error('unexpected Set constructor return/finally semantics');
  }

  let throwFinally = false;
  function setThrowProbe() {
    try {
      for (const value of new Set(values)) {
        if (value === 1) {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Set constructor iteration');
    } finally {
      throwFinally = true;
    }
  }
  let threw = false;
  try {
    setThrowProbe();
  } catch {
    threw = true;
  }
  if (!threw || !throwFinally) {
    throw new Error('unexpected Set constructor throw/finally semantics');
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

browserSetIteration();
"##
}

fn browser_harness_set_iteration_test_source() -> &'static str {
    r##"Kali.test('set constructor iteration', () => {
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

  let returnFinally = false;
  function setReturnProbe() {
    try {
      for (const value of new Set(values)) {
        return value;
      }
      throw new Error('unexpected empty Set constructor iteration');
    } finally {
      returnFinally = true;
    }
  }
  const returnValue = setReturnProbe();
  if (returnValue !== 1 || !returnFinally) {
    throw new Error('unexpected Set constructor return/finally semantics');
  }

  let throwFinally = false;
  function setThrowProbe() {
    try {
      for (const value of new Set(values)) {
        if (value === 1) {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Set constructor iteration');
    } finally {
      throwFinally = true;
    }
  }
  let threw = false;
  try {
    setThrowProbe();
  } catch {
    threw = true;
  }
  if (!threw || !throwFinally) {
    throw new Error('unexpected Set constructor throw/finally semantics');
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
});
"##
}

fn assert_browser_harness_set_iteration(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_harness_set_iteration_test_source()
    } else {
        browser_harness_set_iteration_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        if command == "run" {
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("browser set constructor iteration ok"));
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("browser set constructor iteration ok"));
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser set constructor iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_set_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_set_iteration("run", "main.js", false);
}

#[test]
fn test_supports_set_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_set_iteration("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_set_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_set_iteration("run", "main.js", true);
}

#[test]
fn json_test_supports_set_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_set_iteration("test", "smoke.test.js", true);
}

#[test]
fn supports_set_constructor_iteration_in_browser_api_surface_with_harness_ts_jsx_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        let filename = format!("main.{extension}");
        for (command, json_output) in [
            ("run", false),
            ("test", false),
            ("run", true),
            ("test", true),
        ] {
            assert_browser_harness_set_iteration(command, &filename, json_output);
        }
    }
}
