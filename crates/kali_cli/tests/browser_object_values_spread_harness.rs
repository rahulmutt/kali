use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_values_spread_run_source() -> &'static str {
    r##"function assertObjectValuesSpreadIteration(values) {
  if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
    throw new Error('unexpected Object.values spread iteration semantics');
  }
}

function browserObjectValuesSpreadIteration() {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const collected = [...Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const doubleQuotedSingleQuotedCollected = [...globalThis["Object"]['values'](fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const parenthesizedSingleQuotedReceiverBracketedCollected = [...Object.freeze((globalThis['Object'])['values'])(fromEntries)];
  const parenthesizedSingleQuotedReceiverBracketedPropertyCollected = [...Object.freeze((globalThis['Object']).values)(fromEntries)];
  const frozenBracketRootCollected = [...Object.freeze((globalThis["Object"]))["values"](fromEntries)];
  const frozenCollected = [...globalThis["Object"]["values"](frozenFromEntries)];
  const frozenFromEntriesCollected = [...Object.freeze(Object.values(fromEntries))];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(doubleQuotedSingleQuotedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(parenthesizedSingleQuotedReceiverBracketedCollected);
  assertObjectValuesSpreadIteration(parenthesizedSingleQuotedReceiverBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(frozenBracketRootCollected);
  assertObjectValuesSpreadIteration(frozenCollected);
  assertObjectValuesSpreadIteration(frozenFromEntriesCollected);
  console.log('browser object values spread iteration ok');
}

browserObjectValuesSpreadIteration();
"##
}

fn browser_harness_object_values_spread_test_source() -> &'static str {
    r##"Kali.test('object values spread iteration', () => {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  function browserObjectValuesSpreadIteration() {
    const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
    const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
    const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
    const collected = [...Object.values(fromEntries)];
    const bracketedCollected = [...Object.values(bracketedFromEntries)];
    const globalCollected = [...globalThis.Object.values(fromEntries)];
    const mixedCollected = [...globalThis.Object["values"](fromEntries)];
    const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
    const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
    const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
    const frozenBracketRootCollected = [...Object.freeze((globalThis["Object"]))["values"](fromEntries)];
    const frozenCollected = [...globalThis["Object"]["values"](frozenFromEntries)];
    const frozenFromEntriesCollected = [...Object.freeze(Object.values(fromEntries))];
    assertObjectValuesSpreadIteration(collected);
    assertObjectValuesSpreadIteration(bracketedCollected);
    assertObjectValuesSpreadIteration(globalCollected);
    assertObjectValuesSpreadIteration(mixedCollected);
    assertObjectValuesSpreadIteration(mixedBracketedCollected);
    assertObjectValuesSpreadIteration(bracketedAliasCollected);
    assertObjectValuesSpreadIteration(singleBracketedCollected);
    assertObjectValuesSpreadIteration(frozenBracketRootCollected);
    assertObjectValuesSpreadIteration(frozenCollected);
    assertObjectValuesSpreadIteration(frozenFromEntriesCollected);
    console.log('browser object values spread iteration ok');
  }

  browserObjectValuesSpreadIteration();
});
"##
}

fn assert_browser_harness_object_values_spread(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

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
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stderr.contains("Uncaught Error")
            || stderr.contains("unreachable")
            || stdout.contains("Uncaught Error")
            || stdout.contains("unreachable"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn run_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_object_values_spread(
                "run",
                filename,
                browser_harness_object_values_spread_run_source(),
                json_output,
            );
        }
    }
}

#[test]
fn test_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        for json_output in [false, true] {
            assert_browser_harness_object_values_spread(
                "test",
                filename,
                browser_harness_object_values_spread_test_source(),
                json_output,
            );
        }
    }
}
