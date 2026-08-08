//! Task 18 batch 3 audit escalation: ONE of this file's 19 `#[test]` fns is
//! blocked from migration by the fixture self-inspection blind spot; the other
//! 18 are migrated to `tests/cases/browser/math_atan2_global_this_root.toml`
//! (69 named sibling `[[case]]` entries, no `[matrix]`, all green).
//!
//! BLOCKED TEST (1 of 19, NOT all -- U4's trim-and-keep applies here, this is
//! not a whole-file retention):
//! `browser_bundle_global_this_math_atan2_frozen_source_includes_direct_frozen_callable_aliases`
//! (`:399-414`). It has no helper: its whole body is three
//! `assert!(source.contains(<needle>))` self-checks -- one of them
//! itself an OR across two quoting spellings -- run against
//! `browser_bundle_global_this_math_atan2_frozen_source()`'s OWN TEXT
//! (`:402`, `:406`, `:407`, `:411` -- the whole blocking construct is the
//! three-`assert!` range `:401-413`), before
//! any command is built and without ever invoking `kali`. The four blocking
//! literals are `Object.freeze(globalThis.Math.atan2)`,
//! `Object.freeze(globalThis['Math']['atan2'])`, its double-quoted sibling
//! (identical but spelling both bracket keys with `"` instead of `'`) and
//! `Object.freeze(Math.atan2)`.
//!
//! WHY THE AUDIT CANNOT CARRY IT. `scripts/audit-case-migration.py` extracts
//! every `.contains(<literal>)` argument as a claim and searches only the
//! fields the case runner turns into assertions; `[source]` is excluded from
//! that search by construction. These four literals are *read*, not *asserted
//! on output*, so no honest migration can put them in an assertion field, and
//! the audit reports them absent regardless of what the migrated `[source]`
//! contains. Verified, not assumed: running
//! `python3 scripts/audit-case-migration.py
//! crates/kali_cli/tests/browser_math_atan2_global_this_root.rs
//! crates/kali_cli/tests/cases/browser/math_atan2_global_this_root.toml`
//! reports `AUDIT FAILED -- 4 claim(s) absent`, listing exactly those four
//! literals and nothing else.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs` and
//! batch 2's `browser_array_from_set_map_bundle.rs`; the controller has ruled
//! the script is NOT extended for it (ruling 4), so this is escalated per rule
//! 3/4 and the affected test is retained hand-written.
//!
//! SCOPE OF THE RETENTION. Only the one `#[test]` above is retained. The other
//! 18 fns route through `assert_browser_bundle_global_this_math_atan2`,
//! `assert_browser_bundle_global_this_math_atan2_source`,
//! `assert_browser_bundle_global_this_math_atan2_await_wrapped` or
//! `assert_browser_harness_global_this_math_atan2`; none reads fixture text and
//! all migrated cleanly (69 real invocations, expanded from their `for`
//! loops). Those 18 are still present in this file because batch 3's brief
//! forbids deleting or trimming any `.rs` in this increment -- deletion is one
//! family-wide operation after batch 8. At that sweep this file must be TRIMMED
//! to the single blocked test and
//! `browser_bundle_global_this_math_atan2_frozen_source`, NOT deleted outright.
//!
//! See `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch3-report.md` for the full account.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_global_this_math_atan2_source() -> &'static str {
    r##"// kali-tree-shake: globalThisMathAtan2ZeroSlice
function globalThisMathAtan2ZeroSlice() {
  const zero = 0;
  const one = 1;
  console.log(globalThis.Math.atan2(zero, one));
  console.log(globalThis.Math["atan2"](zero, one));
  return [globalThis.Math.atan2(zero, one), globalThis.Math["atan2"](zero, one)];
}
"##
}

fn browser_harness_global_this_math_atan2_run_source() -> &'static str {
    "const zero = 0; const one = 1; console.log(globalThis.Math.atan2(zero, one)); console.log(globalThis.Math[\"atan2\"](zero, one));\n"
}

fn browser_harness_global_this_math_atan2_test_source() -> &'static str {
    r#"Kali.test('globalThis math atan2 zero slice', () => {
  const zero = 0;
  const one = 1;
  console.log(globalThis.Math.atan2(zero, one));
  console.log(globalThis.Math["atan2"](zero, one));
});
"#
}

fn browser_bundle_global_this_math_atan2_await_wrapped_source() -> &'static str {
    r##"// kali-tree-shake: globalThisMathAtan2AwaitWrappedZeroSlice
async function globalThisMathAtan2AwaitWrappedZeroSlice() {
  console.log(globalThis.Math.atan2(await 0, await 1));
  console.log(globalThis.Math["atan2"](await 0, await 1));
  return [globalThis.Math.atan2(await 0, await 1), globalThis.Math["atan2"](await 0, await 1)];
}
"##
}

fn browser_harness_global_this_math_atan2_await_wrapped_run_source() -> &'static str {
    "async function main() {\n  console.log(globalThis.Math.atan2(await 0, await 1));\n  console.log(globalThis.Math[\"atan2\"](await 0, await 1));\n}\n\nmain();\n"
}

fn browser_harness_global_this_math_atan2_await_wrapped_test_source() -> &'static str {
    r#"Kali.test('globalThis math atan2 await-wrapped zero slice', () => {
  async function main() {
    console.log(globalThis.Math.atan2(await 0, await 1));
    console.log(globalThis.Math["atan2"](await 0, await 1));
  }
  return main();
});
"#
}

fn browser_bundle_global_this_math_atan2_frozen_source() -> &'static str {
    r##"// kali-tree-shake: globalThisMathAtan2FrozenCallableAliases
function globalThisMathAtan2FrozenCallableAliases() {
  const zero = 0;
  const one = 1;
  const frozenDotRoot = Object.freeze(globalThis.Math.atan2);
  const frozenBracketedRoot = Object.freeze(globalThis["Math"]["atan2"]);
  const frozenSingleQuotedRoot = Object.freeze(globalThis['Math']['atan2']);
  const frozenDirect = Object.freeze(Math.atan2);
  console.log(frozenDotRoot(zero, one));
  console.log(frozenBracketedRoot(zero, one));
  console.log(frozenSingleQuotedRoot(zero, one));
  console.log(frozenDirect(zero, one));
  return [frozenDotRoot(zero, one), frozenBracketedRoot(zero, one), frozenSingleQuotedRoot(zero, one), frozenDirect(zero, one)];
}
"##
}

fn browser_harness_global_this_math_atan2_frozen_run_source() -> &'static str {
    "const zero = 0; const one = 1; const frozenDotRoot = Object.freeze(globalThis.Math.atan2); const frozenBracketedRoot = Object.freeze(globalThis[\"Math\"][\"atan2\"]); const frozenSingleQuotedRoot = Object.freeze(globalThis['Math']['atan2']); const frozenDirect = Object.freeze(Math.atan2); console.log(frozenDotRoot(zero, one)); console.log(frozenBracketedRoot(zero, one)); console.log(frozenSingleQuotedRoot(zero, one)); console.log(frozenDirect(zero, one));\n"
}

fn browser_harness_global_this_math_atan2_frozen_test_source() -> &'static str {
    r#"Kali.test('globalThis math atan2 frozen callable aliases', () => {
  const zero = 0;
  const one = 1;
  const frozenDotRoot = Object.freeze(globalThis.Math.atan2);
  const frozenBracketedRoot = Object.freeze(globalThis["Math"]["atan2"]);
  const frozenSingleQuotedRoot = Object.freeze(globalThis['Math']['atan2']);
  const frozenDirect = Object.freeze(Math.atan2);
  console.log(frozenDotRoot(zero, one));
  console.log(frozenBracketedRoot(zero, one));
  console.log(frozenSingleQuotedRoot(zero, one));
  console.log(frozenDirect(zero, one));
});
"#
}

fn assert_browser_bundle_global_this_math_atan2_await_wrapped(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_global_this_math_atan2_await_wrapped_source(),
    )
    .expect("write source");

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
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.globalThisMathAtan2AwaitWrappedZeroSlice();
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n"), "stdout: {stdout}");
}

fn assert_browser_bundle_global_this_math_atan2(filename: &str, json_output: bool) {
    assert_browser_bundle_global_this_math_atan2_source(
        browser_bundle_global_this_math_atan2_source(),
        "globalThisMathAtan2ZeroSlice",
        filename,
        json_output,
    );
}

fn assert_browser_bundle_global_this_math_atan2_source(
    source: &str,
    export_name: &str,
    filename: &str,
    json_output: bool,
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
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        &format!("const mod = await import(bundleJs.href);\nawait mod.{export_name}();\n"),
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n"), "stdout: {stdout}");
}

fn assert_browser_harness_global_this_math_atan2(
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
        .current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("0\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("0\n"), "stdout: {stdout}");
    }
}

#[test]
fn browser_bundle_global_this_math_atan2_frozen_source_includes_direct_frozen_callable_aliases() {
    let source = browser_bundle_global_this_math_atan2_frozen_source();
    assert!(
        source.contains("Object.freeze(globalThis.Math.atan2)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis[\"Math\"][\"atan2\"])")
            || source.contains("Object.freeze(globalThis['Math']['atan2'])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.atan2)"),
        "source: {source}"
    );
}

#[test]
fn build_emits_global_this_math_atan2_zero_slice_in_js_input() {
    assert_browser_bundle_global_this_math_atan2("app.js", false);
}

#[test]
fn build_emits_global_this_math_atan2_zero_slice_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2(filename, false);
    }
}

#[test]
fn json_build_emits_global_this_math_atan2_zero_slice_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2(filename, true);
    }
}

#[test]
fn build_emits_global_this_math_atan2_frozen_callable_aliases_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2_source(
            browser_bundle_global_this_math_atan2_frozen_source(),
            "globalThisMathAtan2FrozenCallableAliases",
            filename,
            false,
        );
    }
}

#[test]
fn json_build_emits_global_this_math_atan2_frozen_callable_aliases_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2_source(
            browser_bundle_global_this_math_atan2_frozen_source(),
            "globalThisMathAtan2FrozenCallableAliases",
            filename,
            true,
        );
    }
}

#[test]
fn run_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "main.js",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.ts",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.jsx",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.tsx",
            browser_harness_global_this_math_atan2_run_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("run", filename, source, false);
    }
}

#[test]
fn run_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "main.js",
            browser_harness_global_this_math_atan2_frozen_run_source(),
        ),
        (
            "main.ts",
            browser_harness_global_this_math_atan2_frozen_run_source(),
        ),
        (
            "main.jsx",
            browser_harness_global_this_math_atan2_frozen_run_source(),
        ),
        (
            "main.tsx",
            browser_harness_global_this_math_atan2_frozen_run_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("run", filename, source, false);
    }
}

#[test]
fn test_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, false);
    }
}

#[test]
fn test_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, false);
    }
}

#[test]
fn run_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_json_js_like_input(
) {
    for (filename, source) in [
        (
            "main.js",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.ts",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.jsx",
            browser_harness_global_this_math_atan2_run_source(),
        ),
        (
            "main.tsx",
            browser_harness_global_this_math_atan2_run_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("run", filename, source, true);
    }
}

#[test]
fn test_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_json_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, true);
    }
}

#[test]
fn test_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_json_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_frozen_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, true);
    }
}

#[test]
fn build_emits_global_this_math_atan2_await_wrapped_zero_slice_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2_await_wrapped(filename, false);
    }
}

#[test]
fn json_build_emits_global_this_math_atan2_await_wrapped_zero_slice_in_js_like_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_this_math_atan2_await_wrapped(filename, true);
    }
}

#[test]
fn run_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "main.js",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.ts",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.jsx",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.tsx",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("run", filename, source, false);
    }
}

#[test]
fn test_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, false);
    }
}

#[test]
fn run_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_json_js_like_input(
) {
    for (filename, source) in [
        (
            "main.js",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.ts",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.jsx",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
        (
            "main.tsx",
            browser_harness_global_this_math_atan2_await_wrapped_run_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("run", filename, source, true);
    }
}

#[test]
fn test_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_json_js_like_input(
) {
    for (filename, source) in [
        (
            "smoke.test.js",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.ts",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.jsx",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
        (
            "smoke.test.tsx",
            browser_harness_global_this_math_atan2_await_wrapped_test_source(),
        ),
    ] {
        assert_browser_harness_global_this_math_atan2("test", filename, source, true);
    }
}
