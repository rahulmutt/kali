use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_map_iteration_run_source() -> &'static str {
    r##"function browserMapIteration() {
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

  assertMapIteration(direct);
  assertMapIteration(alias);
  assertMapIteration(wrappedAlias);
  assertMapIteration(globalDirect);
  assertMapIteration(bracketed);
  assertMapIteration(singleBracketed);
  console.log('browser map constructor iteration ok');
}

browserMapIteration();
"##
}

fn browser_harness_map_iteration_test_source() -> &'static str {
    r##"Kali.test('map constructor iteration', () => {
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

  assertMapIteration(direct);
  assertMapIteration(alias);
  assertMapIteration(wrappedAlias);
  assertMapIteration(globalDirect);
  assertMapIteration(bracketed);
  assertMapIteration(singleBracketed);
  console.log('browser map constructor iteration ok');
});
"##
}

fn assert_browser_harness_map_iteration(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_harness_map_iteration_test_source()
    } else {
        browser_harness_map_iteration_run_source()
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
                .contains("browser map constructor iteration ok"));
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("browser map constructor iteration ok"));
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser map constructor iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_map_iteration("run", "main.js", false);
}

#[test]
fn test_supports_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_map_iteration("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_map_iteration("run", "main.js", true);
}

#[test]
fn json_test_supports_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_map_iteration("test", "smoke.test.js", true);
}

#[test]
fn supports_map_constructor_iteration_in_browser_api_surface_with_harness_ts_jsx_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        let filename = format!("main.{extension}");
        for (command, json_output) in [
            ("run", false),
            ("test", false),
            ("run", true),
            ("test", true),
        ] {
            assert_browser_harness_map_iteration(command, &filename, json_output);
        }
    }
}
