use std::{fs, process::Command};

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
  const parenthesizedBracketed = [];
  for (const value of new (globalThis["Set"])(values)) {
    parenthesizedBracketed.push(value);
  }
  const parenthesizedSingleBracketed = [];
  for (const value of new (globalThis['Set'])(values)) {
    parenthesizedSingleBracketed.push(value);
  }
  const bracketed = [];
  for (const value of new globalThis["Set"](values)) {
    bracketed.push(value);
  }
  const singleBracketed = [];
  for (const value of new globalThis['Set'](values)) {
    singleBracketed.push(value);
  }
  const nullishValues = [];
  for (const value of new (null ?? Set)(aliasValues)) {
    nullishValues.push(value);
  }
  const logicalOrValues = [];
  for (const value of new (false || Set)(aliasValues)) {
    logicalOrValues.push(value);
  }
  assertSetIteration(nullishValues);
  assertSetIteration(logicalOrValues);
  const frozenValues = Object.freeze(aliasValues);
  const frozenSet = Object.freeze(Set);
  const frozenGlobalThisSet = Object.freeze(globalThis.Set);
  const frozenGlobalThisBracketedSet = Object.freeze(globalThis["Set"]);
  const wrappedFrozenSet = Object.freeze((Set));
  const wrappedFrozenGlobalThisSet = Object.freeze((globalThis.Set));
  const wrappedFrozenGlobalThisBracketedSet = Object.freeze((globalThis["Set"]));
  const frozenDirect = [];
  for (const value of new Set(frozenValues)) {
    frozenDirect.push(value);
  }
  const frozenAlias = [];
  for (const value of new (frozenSet)(values)) {
    frozenAlias.push(value);
  }
  const frozenGlobalDirect = [];
  for (const value of new (frozenGlobalThisSet)(values)) {
    frozenGlobalDirect.push(value);
  }
  const frozenGlobalBracketed = [];
  for (const value of new (frozenGlobalThisBracketedSet)(values)) {
    frozenGlobalBracketed.push(value);
  }
  const wrappedFrozenDirect = [];
  for (const value of new (wrappedFrozenSet)(values)) {
    wrappedFrozenDirect.push(value);
  }
  const wrappedFrozenGlobalDirect = [];
  for (const value of new (wrappedFrozenGlobalThisSet)(values)) {
    wrappedFrozenGlobalDirect.push(value);
  }
  const wrappedFrozenGlobalBracketed = [];
  for (const value of new (wrappedFrozenGlobalThisBracketedSet)(values)) {
    wrappedFrozenGlobalBracketed.push(value);
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
  assertSetIteration(parenthesizedBracketed);
  assertSetIteration(parenthesizedSingleBracketed);
  assertSetIteration(bracketed);
  assertSetIteration(singleBracketed);
  assertSetIteration(frozenDirect);
  assertSetIteration(frozenAlias);
  assertSetIteration(frozenGlobalDirect);
  assertSetIteration(frozenGlobalBracketed);
  assertSetIteration(wrappedFrozenDirect);
  assertSetIteration(wrappedFrozenGlobalDirect);
  assertSetIteration(wrappedFrozenGlobalBracketed);
  console.log('browser set constructor iteration ok');
}

browserSetIteration();
"##
}

fn browser_harness_set_iteration_test_source() -> &'static str {
    // See the matching comment in browser_map_iteration_harness.rs: the whole
    // check body is wrapped in its own named function (called once from the
    // `Kali.test` callback) so `returnFinally`/`throwFinally` stay local to
    // `setConstructorIterationCheck` instead of becoming module-scope
    // bindings read across a function boundary by the nested
    // `setReturnProbe`/`setThrowProbe` functions (unsupported; previously
    // silently miscompiled, now correctly rejected as E5506).
    r##"function setConstructorIterationCheck() {
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
  const parenthesizedBracketed = [];
  for (const value of new (globalThis["Set"])(values)) {
    parenthesizedBracketed.push(value);
  }
  const parenthesizedSingleBracketed = [];
  for (const value of new (globalThis['Set'])(values)) {
    parenthesizedSingleBracketed.push(value);
  }
  const bracketed = [];
  for (const value of new globalThis["Set"](values)) {
    bracketed.push(value);
  }
  const singleBracketed = [];
  for (const value of new globalThis['Set'](values)) {
    singleBracketed.push(value);
  }
  const nullishValues = [];
  for (const value of new (null ?? Set)(aliasValues)) {
    nullishValues.push(value);
  }
  const logicalOrValues = [];
  for (const value of new (false || Set)(aliasValues)) {
    logicalOrValues.push(value);
  }
  assertSetIteration(nullishValues);
  assertSetIteration(logicalOrValues);
  const frozenValues = Object.freeze(aliasValues);
  const frozenSet = Object.freeze(Set);
  const frozenGlobalThisSet = Object.freeze(globalThis.Set);
  const frozenGlobalThisBracketedSet = Object.freeze(globalThis["Set"]);
  const wrappedFrozenSet = Object.freeze((Set));
  const wrappedFrozenGlobalThisSet = Object.freeze((globalThis.Set));
  const wrappedFrozenGlobalThisBracketedSet = Object.freeze((globalThis["Set"]));
  const frozenDirect = [];
  for (const value of new Set(frozenValues)) {
    frozenDirect.push(value);
  }
  const frozenAlias = [];
  for (const value of new (frozenSet)(values)) {
    frozenAlias.push(value);
  }
  const frozenGlobalDirect = [];
  for (const value of new (frozenGlobalThisSet)(values)) {
    frozenGlobalDirect.push(value);
  }
  const frozenGlobalBracketed = [];
  for (const value of new (frozenGlobalThisBracketedSet)(values)) {
    frozenGlobalBracketed.push(value);
  }
  const wrappedFrozenDirect = [];
  for (const value of new (wrappedFrozenSet)(values)) {
    wrappedFrozenDirect.push(value);
  }
  const wrappedFrozenGlobalDirect = [];
  for (const value of new (wrappedFrozenGlobalThisSet)(values)) {
    wrappedFrozenGlobalDirect.push(value);
  }
  const wrappedFrozenGlobalBracketed = [];
  for (const value of new (wrappedFrozenGlobalThisBracketedSet)(values)) {
    wrappedFrozenGlobalBracketed.push(value);
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
  assertSetIteration(parenthesizedBracketed);
  assertSetIteration(parenthesizedSingleBracketed);
  assertSetIteration(bracketed);
  assertSetIteration(singleBracketed);
  assertSetIteration(frozenDirect);
  assertSetIteration(frozenAlias);
  assertSetIteration(frozenGlobalDirect);
  assertSetIteration(frozenGlobalBracketed);
  assertSetIteration(wrappedFrozenDirect);
  assertSetIteration(wrappedFrozenGlobalDirect);
  assertSetIteration(wrappedFrozenGlobalBracketed);
  console.log('browser set constructor iteration ok');
}

Kali.test('set constructor iteration', () => {
  setConstructorIterationCheck();
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
    cli.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
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

    // Honest re-pin (PR #16 rev2, family `mapset`): kali fails closed/loud here
    // (2 of this helper's 5 worklist callers were tagged class B by the automated
    // classifier, but direct verification shows every one of them panics on this
    // exact assertion too — a loud E5506 rejection (try/catch is unavailable), not
    // a silent wrong value; re-pinned as class A for all 5 callers — see
    // docs/superpowers/followups/pr16-honest-repin-inventory.md).
    assert!(!output.status.success(), "must fail closed: {output:?}");
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
