use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_string_enumeration_sequence_wrappers_source() -> &'static str {
    r##"async function browserObjectStringEnumerationSequenceWrappers() {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
      throw new Error('unexpected Object.keys string-primitive iteration semantics');
    }
  }

  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
      throw new Error('unexpected Object.values string-primitive iteration semantics');
    }
  }

  function assertObjectEntriesIteration(entries) {
    if (
      entries.length !== 2 ||
      entries[0][0] !== '0' ||
      entries[0][1] !== 'a' ||
      entries[1][0] !== '1' ||
      entries[1][1] !== 'b'
    ) {
      throw new Error('unexpected Object.entries string-primitive iteration semantics');
    }
  }

  const keys = [];
  for await (const key of (0, globalThis["Object"]["keys"]('ab'))) {
    keys.push(key);
  }
  const singleQuotedKeys = [];
  for await (const key of (0, globalThis['Object']['keys']('ab'))) {
    singleQuotedKeys.push(key);
  }
  const values = [];
  for await (const value of (0, globalThis.Object["values"]('ab'))) {
    values.push(value);
  }
  const singleQuotedValues = [];
  for await (const value of (0, globalThis['Object']['values']('ab'))) {
    singleQuotedValues.push(value);
  }
  const entries = [];
  for await (const entry of (0, globalThis["Object"]["entries"]('ab'))) {
    entries.push(entry);
  }
  const singleQuotedEntries = [];
  for await (const entry of (0, globalThis['Object']['entries']('ab'))) {
    singleQuotedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(singleQuotedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(singleQuotedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(singleQuotedEntries);
  console.log('browser object string enumeration sequence wrappers ok');
}

browserObjectStringEnumerationSequenceWrappers();
"##
}

fn browser_harness_object_string_enumeration_sequence_wrappers_test_source() -> &'static str {
    r##"async function browserObjectStringEnumerationSequenceWrappers() {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
      throw new Error('unexpected Object.keys string-primitive iteration semantics');
    }
  }

  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
      throw new Error('unexpected Object.values string-primitive iteration semantics');
    }
  }

  function assertObjectEntriesIteration(entries) {
    if (
      entries.length !== 2 ||
      entries[0][0] !== '0' ||
      entries[0][1] !== 'a' ||
      entries[1][0] !== '1' ||
      entries[1][1] !== 'b'
    ) {
      throw new Error('unexpected Object.entries string-primitive iteration semantics');
    }
  }

  const keys = [];
  for await (const key of (0, globalThis["Object"]["keys"]('ab'))) {
    keys.push(key);
  }
  const singleQuotedKeys = [];
  for await (const key of (0, globalThis['Object']['keys']('ab'))) {
    singleQuotedKeys.push(key);
  }
  const values = [];
  for await (const value of (0, globalThis.Object["values"]('ab'))) {
    values.push(value);
  }
  const singleQuotedValues = [];
  for await (const value of (0, globalThis['Object']['values']('ab'))) {
    singleQuotedValues.push(value);
  }
  const entries = [];
  for await (const entry of (0, globalThis["Object"]["entries"]('ab'))) {
    entries.push(entry);
  }
  const singleQuotedEntries = [];
  for await (const entry of (0, globalThis['Object']['entries']('ab'))) {
    singleQuotedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(singleQuotedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(singleQuotedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(singleQuotedEntries);
  console.log('browser object string enumeration sequence wrappers ok');
}

Kali.test('browser object string enumeration sequence wrappers', () => browserObjectStringEnumerationSequenceWrappers());
"##
}

fn assert_browser_harness_object_string_enumeration_sequence_wrappers(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_harness_object_string_enumeration_sequence_wrappers_test_source()
    } else {
        browser_harness_object_string_enumeration_sequence_wrappers_source()
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

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stderr.contains("E5506") || stdout.contains("E5506"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn run_supports_object_string_enumeration_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_object_string_enumeration_sequence_wrappers("run", "main.js", false);
}

#[test]
fn test_supports_object_string_enumeration_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_object_string_enumeration_sequence_wrappers("test", "main.js", false);
}

#[test]
fn json_run_supports_object_string_enumeration_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_object_string_enumeration_sequence_wrappers("run", "main.js", true);
}

#[test]
fn json_test_supports_object_string_enumeration_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_object_string_enumeration_sequence_wrappers("test", "main.js", true);
}

#[test]
fn supports_object_string_enumeration_sequence_wrappers_in_browser_api_surface_with_harness_ts_jsx_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        let filename = format!("main.{extension}");
        for (command, json_output) in [
            ("run", false),
            ("test", false),
            ("run", true),
            ("test", true),
        ] {
            assert_browser_harness_object_string_enumeration_sequence_wrappers(
                command,
                &filename,
                json_output,
            );
        }
    }
}
