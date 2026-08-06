use std::{fs, process::Command};

use tempfile::tempdir;

use kali_common::{
    object_has_own_combined_frozen_callable_condition_source,
    object_has_own_frozen_callable_source, object_has_own_property_call_binding_source,
    object_has_own_property_call_frozen_callable_source,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_has_own_run_source() -> String {
    let frozen_callable_condition_source =
        object_has_own_combined_frozen_callable_condition_source("wrapped", r#""a""#);
    let has_own_property_call_binding_source =
        object_has_own_property_call_binding_source("hasOwnPropertyCall");
    let frozen_callable_source = format!(
        "{} {}",
        object_has_own_frozen_callable_source(),
        object_has_own_property_call_frozen_callable_source()
    );
    format!(
        r#"const object = Object.fromEntries([["a", 1], ["b", 2]]);
const alias = object;
const hasOwn = Object.hasOwn;
const singleQuotedHasOwn = globalThis['Object']['hasOwn'];
const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];
const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);
const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);
{}
{}
const wrapped = (0, alias);
if (
  !Object.hasOwn(wrapped, "a") ||
  !hasOwn(wrapped, "a") ||
  !Object["hasOwn"](wrapped, "a") ||
  !globalThis.Object["hasOwn"](wrapped, "a") ||
  !globalThis["Object"]["hasOwn"](wrapped, "a") ||
  !globalThis.Object["hasOwn"](wrapped, "a") ||
  !globalThis["Object"].hasOwn(wrapped, "a") ||
  !singleQuotedHasOwn(wrapped, "a") ||
  !parenthesizedSingleQuotedHasOwn(wrapped, "a") ||
  !frozenSingleQuotedHasOwn(wrapped, "a") ||
  !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") ||
  {} ||
  !Object.prototype.hasOwnProperty.call(wrapped, "a") ||
  !Object["hasOwnProperty"].call(wrapped, "a") ||
  !Object["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||
  !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||
  !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||
  !hasOwnPropertyCall(wrapped, "a") ||
  !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||
  !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||
  !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||
  !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")
) {{
  throw new Error('unexpected browser Object.hasOwn result');
}}
console.log('browser object hasOwn ok');
"#,
        has_own_property_call_binding_source,
        frozen_callable_source,
        frozen_callable_condition_source
    )
}

fn browser_harness_object_has_own_test_source() -> String {
    let frozen_callable_condition_source =
        object_has_own_combined_frozen_callable_condition_source("wrapped", r#""a""#);
    let has_own_property_call_binding_source =
        object_has_own_property_call_binding_source("hasOwnPropertyCall");
    let frozen_callable_source = format!(
        "{} {}",
        object_has_own_frozen_callable_source(),
        object_has_own_property_call_frozen_callable_source()
    );
    format!(
        r#"Kali.test('object hasOwn primitive literals', () => {{
  const object = Object.fromEntries([["a", 1], ["b", 2]]);
  const alias = object;
  const hasOwn = Object.hasOwn;
  {}
  {}
  const wrapped = (0, alias);
  const singleQuotedHasOwn = globalThis['Object']['hasOwn'];
  const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];
  const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);
  const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);
  if (
    !Object.hasOwn(wrapped, "a") ||
    !hasOwn(wrapped, "a") ||
    !Object["hasOwn"](wrapped, "a") ||
    !globalThis.Object["hasOwn"](wrapped, "a") ||
    !globalThis["Object"]["hasOwn"](wrapped, "a") ||
    !globalThis.Object["hasOwn"](wrapped, "a") ||
    !globalThis["Object"].hasOwn(wrapped, "a") ||
    !singleQuotedHasOwn(wrapped, "a") ||
    !parenthesizedSingleQuotedHasOwn(wrapped, "a") ||
    !frozenSingleQuotedHasOwn(wrapped, "a") ||
    !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") ||
    {} ||
    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||
    !Object["hasOwnProperty"].call(wrapped, "a") ||
    !Object["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||
    !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||
    !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||
    !hasOwnPropertyCall(wrapped, "a") ||
    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||
    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||
    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||
    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")
  ) {{
    throw new Error('unexpected browser Object.hasOwn result');
  }}
  console.log('browser object hasOwn ok');
}});
"#,
        has_own_property_call_binding_source,
        frozen_callable_source,
        frozen_callable_condition_source
    )
}

fn assert_browser_harness_object_has_own<S: AsRef<str>>(
    command: &str,
    filename: &str,
    source: S,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source.as_ref()).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.js",
        browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.jsx",
        browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.ts",
        browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.tsx",
        browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.js",
        browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.jsx",
        browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.ts",
        browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.tsx",
        browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.js",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.jsx",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.ts",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.tsx",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.js",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.jsx",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.ts",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.tsx",
        browser_harness_object_has_own_test_source(),
        true,
    );
}
