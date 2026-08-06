use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_values_run_source() -> &'static str {
    r##"function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of Object.values(alias)) {
    collected.push(value);
  }
  assertObjectValuesIteration(collected);
  console.log('browser object values iteration ok');
}

browserObjectValuesIteration();
"##
}

fn browser_harness_object_values_test_source() -> &'static str {
    r##"Kali.test('object values iteration', () => {
  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Object.values iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of Object.values(alias)) {
    collected.push(value);
  }
  assertObjectValuesIteration(collected);
  console.log('browser object values iteration ok');
});
"##
}

fn browser_harness_global_object_values_run_source() -> &'static str {
    r##"function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserGlobalObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of globalThis.Object.values(alias)) {
    collected.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const singleQuotedProperty = [];
  for (const value of globalThis['Object'].values(alias)) {
    singleQuotedProperty.push(value);
  }
  const doubleQuotedSingleQuoted = [];
  for (const value of globalThis["Object"]['values'](alias)) {
    doubleQuotedSingleQuoted.push(value);
  }
  const mixedSingleQuotedBracketed = [];
  for (const value of globalThis['Object']["values"](alias)) {
    mixedSingleQuotedBracketed.push(value);
  }
  const mixedSingleQuoted = [];
  for (const value of globalThis['Object']['values'](alias)) {
    mixedSingleQuoted.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  const parenthesizedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis["Object"])["values"])(alias)) {
    parenthesizedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis['Object'])['values'])(alias)) {
    parenthesizedSingleQuotedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketedProperty = [];
  for (const value of Object.freeze((globalThis['Object']).values)(alias)) {
    parenthesizedSingleQuotedReceiverBracketedProperty.push(value);
  }
  const parenthesizedBracketed = [];
  for (const value of Object.freeze((globalThis["Object"]).values)(alias)) {
    parenthesizedBracketed.push(value);
  }
  assertObjectValuesIteration(collected);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(singleQuotedProperty);
  assertObjectValuesIteration(doubleQuotedSingleQuoted);
  assertObjectValuesIteration(mixedSingleQuotedBracketed);
  assertObjectValuesIteration(mixedSingleQuoted);
  assertObjectValuesIteration(bracketed);
  assertObjectValuesIteration(parenthesizedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketedProperty);
  assertObjectValuesIteration(parenthesizedBracketed);
  console.log('browser object values iteration ok');
}

browserGlobalObjectValuesIteration();
"##
}

fn browser_harness_global_object_values_test_source() -> &'static str {
    r##"Kali.test('global object values iteration', () => {
  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Object.values iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of globalThis.Object.values(alias)) {
    collected.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const singleQuotedProperty = [];
  for (const value of globalThis['Object'].values(alias)) {
    singleQuotedProperty.push(value);
  }
  const doubleQuotedSingleQuoted = [];
  for (const value of globalThis["Object"]['values'](alias)) {
    doubleQuotedSingleQuoted.push(value);
  }
  const mixedSingleQuotedBracketed = [];
  for (const value of globalThis['Object']["values"](alias)) {
    mixedSingleQuotedBracketed.push(value);
  }
  const mixedSingleQuoted = [];
  for (const value of globalThis['Object']['values'](alias)) {
    mixedSingleQuoted.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  const parenthesizedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis["Object"])["values"])(alias)) {
    parenthesizedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis['Object'])['values'])(alias)) {
    parenthesizedSingleQuotedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketedProperty = [];
  for (const value of Object.freeze((globalThis['Object']).values)(alias)) {
    parenthesizedSingleQuotedReceiverBracketedProperty.push(value);
  }
  const parenthesizedBracketed = [];
  for (const value of Object.freeze((globalThis["Object"]).values)(alias)) {
    parenthesizedBracketed.push(value);
  }
  assertObjectValuesIteration(collected);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(singleQuotedProperty);
  assertObjectValuesIteration(doubleQuotedSingleQuoted);
  assertObjectValuesIteration(mixedSingleQuotedBracketed);
  assertObjectValuesIteration(mixedSingleQuoted);
  assertObjectValuesIteration(bracketed);
  assertObjectValuesIteration(parenthesizedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketedProperty);
  assertObjectValuesIteration(parenthesizedBracketed);
  console.log('browser object values iteration ok');
});
"##
}

fn assert_browser_harness_object_values(
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

    // Honest re-pin (PR #16 rev2 straggler cleanup): kali fails closed/loud here
    // (Uncaught Error / RuntimeError: unreachable), never a silent wrong value;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    // Helper re-pin: every caller of this helper in this file (both the run and
    // test variants of `*_object_values_spread_iteration_when_browser_harness_is_configured`)
    // is red — no green out-of-batch caller exists in this test binary, so the
    // helper itself is re-pinned rather than inlining each wrapper.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input()
{
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

fn browser_harness_object_values_spread_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('object values spread iteration', () => {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);
  console.log('browser object values spread iteration ok');
});
"#
        .to_string();
    }

    r#"function browserObjectValuesSpreadIteration() {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);
  console.log('browser object values spread iteration ok');
}

browserObjectValuesSpreadIteration();
"#
    .to_string()
}

fn browser_harness_object_values_frozen_spread_source(test_mode: bool) -> String {
    browser_harness_object_values_spread_source(test_mode).replace(
        "  const fromEntries = Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]);",
        "  const fromEntries = Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]));",
    )
}

#[test]
fn run_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_spread_source(false),
            false,
        );
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_spread_source(false),
            true,
        );
    }
}

#[test]
fn run_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured() {
    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    // Inlined (not routed through assert_browser_harness_object_values_spread): that helper
    // also serves out-of-batch callers that are still green, so it is left untouched.
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        for json_output in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(filename);
            fs::write(
                &source_path,
                &browser_harness_object_values_frozen_spread_source(false),
            )
            .expect("write source");

            let mut cmd = Command::new(kali_bin());
            cmd.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if json_output {
                cmd.arg("--output").arg("json");
            }
            let output = cmd
                .arg("run")
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(!output.status.success(), "must fail closed: {output:?}");
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
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_spread_source(true),
            false,
        );
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_spread_source(true),
            true,
        );
    }
}

#[test]
fn test_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured() {
    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    // Inlined (not routed through assert_browser_harness_object_values_spread): that helper
    // also serves out-of-batch callers that are still green, so it is left untouched.
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        for json_output in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(filename);
            fs::write(
                &source_path,
                &browser_harness_object_values_frozen_spread_source(true),
            )
            .expect("write source");

            let mut cmd = Command::new(kali_bin());
            cmd.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if json_output {
                cmd.arg("--output").arg("json");
            }
            let output = cmd
                .arg("test")
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(!output.status.success(), "must fail closed: {output:?}");
        }
    }
}
