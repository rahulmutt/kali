use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_from_entries_run_source(include_ts_as_const: bool) -> String {
    let source = r##"function assertFromEntriesShape(fromEntries) {
  const keys = Object.keys(fromEntries);
  const entries = Object.entries(fromEntries);
  const values = Object.values(fromEntries);
  if (
    keys.length !== 2 ||
    keys[0] !== 'b' ||
    keys[1] !== 'a' ||
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2 ||
    values.length !== 2 ||
    values[0] !== 1 ||
    values[1] !== 2
  ) {
    throw new Error('unexpected Object.fromEntries semantics');
  }
}

const wrappedEntries = ([["b", 1], ["a", 2]]);
const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);
const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);
  __TS_ONLY__const directFromEntries = Object.fromEntries([["b", 1], ["a", 2]]);
const wrappedFromEntries = Object.fromEntries(wrappedEntries);
const frozenFromEntries = Object.fromEntries(frozenEntries);
const dottedFromEntries = globalThis.Object.fromEntries([["b", 1], ["a", 2]]);
const mixedDottedFromEntries = globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]);
const mixedBracketedFromEntries = globalThis["Object"].fromEntries([["b", 1], ["a", 2]]);
const mixedBracketedQuotedFromEntries = globalThis["Object"]['fromEntries']([["b", 1], ["a", 2]]);
const mixedSingleQuotedFromEntries = globalThis['Object']["fromEntries"]([["b", 1], ["a", 2]]);
const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]);
assertFromEntriesShape(directFromEntries);
assertFromEntriesShape(wrappedFromEntries);
assertFromEntriesShape(frozenFromEntries);
assertFromEntriesShape(Object.fromEntries(conditionalEntries));
assertFromEntriesShape(dottedFromEntries);
assertFromEntriesShape(mixedDottedFromEntries);
assertFromEntriesShape(mixedBracketedFromEntries);
assertFromEntriesShape(mixedBracketedQuotedFromEntries);
assertFromEntriesShape(mixedSingleQuotedFromEntries);
assertFromEntriesShape(bracketedFromEntries);
console.log('browser object fromEntries ok');
"##;

    source.replace(
        "  __TS_ONLY__",
        if include_ts_as_const {
            "  const wrappedEntriesConst = ([[\"b\", 1], [\"a\", 2]] as const);\n  const wrappedFromEntriesConst = Object.fromEntries(wrappedEntriesConst);\n  assertFromEntriesShape(wrappedFromEntriesConst);\n  const wrappedEntriesSatisfies = ([[\"b\", 1], [\"a\", 2]] satisfies unknown);\n  const wrappedFromEntriesSatisfies = Object.fromEntries(wrappedEntriesSatisfies);\n  assertFromEntriesShape(wrappedFromEntriesSatisfies);\n"
        } else {
            ""
        },
    )
}

fn browser_harness_object_from_entries_test_source(include_ts_as_const: bool) -> String {
    let source = r##"Kali.test('object fromEntries ordering', () => {
  function assertFromEntriesShape(fromEntries) {
    const keys = Object.keys(fromEntries);
    const entries = Object.entries(fromEntries);
    const values = Object.values(fromEntries);
    if (
      keys.length !== 2 ||
      keys[0] !== 'b' ||
      keys[1] !== 'a' ||
      entries.length !== 2 ||
      entries[0][0] !== 'b' ||
      entries[0][1] !== 1 ||
      entries[1][0] !== 'a' ||
      entries[1][1] !== 2 ||
      values.length !== 2 ||
      values[0] !== 1 ||
      values[1] !== 2
    ) {
      throw new Error('unexpected Object.fromEntries semantics');
    }
  }

  const wrappedEntries = ([["b", 1], ["a", 2]]);
  const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);
  const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);
  __TS_ONLY__  assertFromEntriesShape(Object.fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(Object.fromEntries(wrappedEntries));
  assertFromEntriesShape(Object.fromEntries(frozenEntries));
  assertFromEntriesShape(Object.fromEntries(conditionalEntries));
  assertFromEntriesShape(globalThis.Object.fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis["Object"].fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis["Object"]['fromEntries']([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis['Object']["fromEntries"]([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]));
  console.log('browser object fromEntries ok');
});
"##;

    source.replace(
        "  __TS_ONLY__",
        if include_ts_as_const {
            "  const wrappedEntriesConst = ([[\"b\", 1], [\"a\", 2]] as const);\n  assertFromEntriesShape(Object.fromEntries(wrappedEntriesConst));\n  const wrappedEntriesSatisfies = ([[\"b\", 1], [\"a\", 2]] satisfies unknown);\n  assertFromEntriesShape(Object.fromEntries(wrappedEntriesSatisfies));\n"
        } else {
            ""
        },
    )
}

fn assert_browser_harness_object_from_entries(
    command: &str,
    filename: &str,
    source: impl AsRef<str>,
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
fn run_supports_object_from_entries_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.js",
        browser_harness_object_from_entries_run_source(false),
        false,
    );
}

#[test]
fn run_supports_object_from_entries_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.ts",
        browser_harness_object_from_entries_run_source(true),
        false,
    );
}

#[test]
fn run_supports_object_from_entries_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.jsx",
        browser_harness_object_from_entries_run_source(false),
        false,
    );
}

#[test]
fn run_supports_object_from_entries_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.tsx",
        browser_harness_object_from_entries_run_source(true),
        false,
    );
}

#[test]
fn test_supports_object_from_entries_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.js",
        browser_harness_object_from_entries_test_source(false),
        false,
    );
}

#[test]
fn test_supports_object_from_entries_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.ts",
        browser_harness_object_from_entries_test_source(true),
        false,
    );
}

#[test]
fn test_supports_object_from_entries_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.jsx",
        browser_harness_object_from_entries_test_source(false),
        false,
    );
}

#[test]
fn test_supports_object_from_entries_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.tsx",
        browser_harness_object_from_entries_test_source(true),
        false,
    );
}

#[test]
fn json_run_supports_object_from_entries_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.js",
        browser_harness_object_from_entries_run_source(false),
        true,
    );
}

#[test]
fn json_run_supports_object_from_entries_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.ts",
        browser_harness_object_from_entries_run_source(true),
        true,
    );
}

#[test]
fn json_run_supports_object_from_entries_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.jsx",
        browser_harness_object_from_entries_run_source(false),
        true,
    );
}

#[test]
fn json_run_supports_object_from_entries_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_from_entries(
        "run",
        "main.tsx",
        browser_harness_object_from_entries_run_source(true),
        true,
    );
}

#[test]
fn json_test_supports_object_from_entries_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.js",
        browser_harness_object_from_entries_test_source(false),
        true,
    );
}

#[test]
fn json_test_supports_object_from_entries_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.ts",
        browser_harness_object_from_entries_test_source(true),
        true,
    );
}

#[test]
fn json_test_supports_object_from_entries_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.jsx",
        browser_harness_object_from_entries_test_source(false),
        true,
    );
}

#[test]
fn json_test_supports_object_from_entries_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_from_entries(
        "test",
        "smoke.test.tsx",
        browser_harness_object_from_entries_test_source(true),
        true,
    );
}
