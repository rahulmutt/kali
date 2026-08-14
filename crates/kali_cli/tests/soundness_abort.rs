//! Stage P3 (AbortController/AbortSignal lane) residual: the ONE test this
//! family could not migrate to a case file.
//!
//! Every other `#[test]` from the original 58 in this file is migrated to
//! `tests/cases/soundness/abort.toml` (57 cases, audited clean). This file
//! keeps exactly `acceptance_web_baseline_prefix_matches_node_byte_for_byte`,
//! because it asserts kali's stdout is BYTE-FOR-BYTE IDENTICAL to a live
//! `node <file>` invocation on the same fixture -- a genuine dual-process
//! COMPARISON the case-runner format cannot express. Confirmed directly
//! against `crates/kali_case_runner/src/{steps,assertions,model}.rs`: a
//! step (whichever of the three kinds -- `cli` runs the fixed
//! `config.kali_bin`; `browser_bundle_harness` runs a second, separately
//! resolved executable via `browser_harness_command_parts_checked`, itself
//! `env`-overridable; `file_json` runs nothing at all) is captured and
//! checked exactly once, against ONE `Captured { code, success, stdout,
//! stderr }` (`steps.rs`'s `capture`/`run_cli`/`run_browser_bundle_harness`,
//! each followed by a single `check`/`check_json` call). No step spawns two
//! processes and compares their outputs against EACH OTHER the way this
//! test compares kali against `node` -- that is the capability gap this
//! file exists to document, not "no code here ever spawns a process."
//! Kept hand-written per spec 5.11's "outliers" bucket (the `starts_with`/
//! `lines()` sites' sibling category: "if they do not fit S5.4"), trimmed
//! to just this test and the two helpers it needs.

use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Runs `node <main_path>` with `dir` as the working directory. The acceptance
/// fixture is valid, unmodified ES that node executes directly, so a straight
/// `node` run is a faithful oracle for "what should this program print".
/// (Copied from `module_namespace_link.rs:36-42`.)
fn node_output(dir: &std::path::Path, main_path: &std::path::Path) -> std::process::Output {
    Command::new("node")
        .current_dir(dir)
        .arg(main_path)
        .output()
        .expect("run node")
}

/// Stage P3 acceptance: the web-baseline smoke PREFIX — structuredClone (P2) +
/// AbortController/`instanceof AbortSignal` (P3) + EventTarget/CustomEvent
/// dispatch with a capturing listener that calls `controller.abort()` (Stage D)
/// — composed into one program and proven byte-for-byte against a real `node`
/// oracle. This is the stage's integration evidence: the abort surface works in
/// composition, not just in the isolated per-lane pins above.
///
/// FIXTURE PROVENANCE (two deliberate divergences from the live web-baseline
/// fixture `structured_clone_and_event_primitives_source` in `runtime_smoke.rs`):
///   1. MINUS the `new Event('tick')` / `event.type` self-check block. That
///      block trips a PRE-EXISTING, out-of-P3-scope gap (`Event.type` reads 0,
///      not 'tick'); it is the fixture's current fail-closed point and belongs
///      to a P-series follow-up, not this acceptance run.
///   2. MINUS everything from `URLSearchParams` onward (P4 URL / P5
///      TextEncoder scope — not yet implemented).
/// So: acceptance prefix = web-baseline fixture − Event-type block − (P4/P5 tail).
///
/// FIXTURE ADAPTATION (recorded for the Task-8 doc entry): the program body is
/// wrapped in `function main() { ... } main();` rather than run at module top
/// level. `controller` is captured by the `addEventListener` arrow, and the
/// ratified capture lane (Task 3 allowlist entry 3, pinned by
/// `captured_handle_full_roundtrip_in_listener`) is FUNCTION-scoped: the handle
/// is a function-local i64 cell pointer restored by-value into the callback env.
/// Reading a MODULE-scope binding from a closure is separately fail-closed
/// (E5506, "reading module binding from a function is only available for
/// compile-time-constant const initializers") — and in the write position
/// (`controller.abort()` inside the closure) it currently fails OPEN through the
/// E3100 zero-placeholder fallback (silent no-op → `aborted` stays 0). That
/// module-scope-capture asymmetry is a pre-existing gap outside P3's ratified
/// lanes; the function-scope wrap keeps this fixture squarely inside the
/// ratified shape. node prints identical stdout either way, so byte-for-byte
/// equality is preserved.
#[test]
fn acceptance_web_baseline_prefix_matches_node_byte_for_byte() {
    let src = r#"function main() {
  const original = { count: 1, values: [1, 2, 3] };
  const cloned = structuredClone(original);
  if (cloned === original || cloned.values === original.values) {
    throw new Error('structuredClone should deep-clone object graphs');
  }
  original.values.push(4);
  if (cloned.count !== 1 || cloned.values.join(',') !== '1,2,3') {
    throw new Error('unexpected structuredClone result');
  }
  const controller = new AbortController();
  if (!(controller.signal instanceof AbortSignal)) {
    throw new Error('expected AbortSignal from AbortController');
  }
  const target = new EventTarget();
  let count = 0;
  target.addEventListener('tick', () => {
    count += 1;
    controller.abort();
  });
  const dispatched = target.dispatchEvent(new CustomEvent('tick'));
  if (!dispatched || count !== 1 || !controller.signal.aborted) {
    throw new Error('unexpected event primitive behavior');
  }
  console.log('web baseline prefix ok');
}
main();
"#;
    let dir = tempdir().expect("tempdir");
    let main_path = dir.path().join("main.js");
    fs::write(&main_path, src).expect("write");
    let kali = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");
    assert!(
        kali.status.success(),
        "kali stderr: {}",
        String::from_utf8_lossy(&kali.stderr)
    );
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout),
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout).trim(),
        "web baseline prefix ok"
    );
}
