//! Stage P4 (URL + URLSearchParams lane) residual: the ONE test this family
//! could not migrate to a case file.
//!
//! Every other `#[test]` from the original 59 in this file is migrated to
//! `tests/cases/soundness/url.toml` (63 cases, audited clean). This file
//! keeps exactly `acceptance_web_baseline_with_url_matches_node_byte_for_byte`,
//! because it asserts kali's stdout is BYTE-FOR-BYTE IDENTICAL to a live
//! `node <file>` invocation on the same fixture -- a genuine dual-process
//! comparison the case-runner format cannot express (see
//! `soundness_abort.rs`'s matching residual for the full confirmation against
//! `crates/kali_case_runner/src/{steps,assertions,model}.rs`). Kept
//! hand-written per spec 5.11's "outliers" bucket, trimmed to just this test
//! and the two helpers it needs.

use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Runs `node <main_path>` with `dir` as the working directory. The acceptance
/// fixture is valid, unmodified ES that node executes directly, so a straight
/// `node` run is a faithful oracle for "what should this program print".
/// (Copied from `soundness_abort.rs` / `module_namespace_link.rs`.)
fn node_output(dir: &std::path::Path, main_path: &std::path::Path) -> std::process::Output {
    Command::new("node")
        .current_dir(dir)
        .arg(main_path)
        .output()
        .expect("run node")
}

/// Stage P4 acceptance: the web-baseline smoke prefix (P2 structuredClone + P3
/// abort + Stage D events) EXTENDED with the URL/USP block, byte-for-byte
/// against a real `node` oracle. This is the stage's integration evidence: the
/// URL/USP surface works in composition (dynamic values through `append`/`set`,
/// `get`/`getAll().length`/`has` in taken-path if-conditions, and the
/// `u.searchParams.get` composition), not just in the isolated per-lane pins.
///
/// FIXTURE PROVENANCE: the live web-baseline fixture
/// `structured_clone_and_event_primitives_source` (runtime_smoke.rs) MINUS the
/// `new Event('tick')`/`event.type` block (pre-existing out-of-scope gap — see
/// `soundness_abort.rs` acceptance provenance note) and MINUS the TextEncoder
/// tail (P5 scope), wrapped `function main() { ... } main();` (module-scope
/// capture stays fail-closed by design; node prints identically — see the P3
/// acceptance's recorded wrap adaptation).
///
/// FIXTURE ADAPTATIONS (recorded for the Task-8 doc entry; each keeps
/// node-identical semantics — node takes the same branches and prints the same
/// bytes for the adapted shapes):
///   1. `String(count)` (3 sites in the brief's text) — `String(x)` is
///      FAIL-CLOSED on this branch (G6 value-builtin deny-set). In the two
///      ARGUMENT sites (`append`/`set`) the Task-4-ratified dynamic-string
///      shape `'' + count` substitutes (same runtime-computed "1").
///   2. The COMPARISON site `query.get('beta') !== String(count)` becomes
///      `query.get('beta') !== '1'`: a runtime-string vs DYNAMIC-string
///      compare (`!== ('' + count)`) is E3200 fail-closed by design (Stage 1
///      lowered only literal/proven operands into `__streq`), and `count` is
///      deterministically `1` here (single dispatched listener), so the
///      literal-RHS compare — the Stage-1-ratified shape — is value-identical
///      in node.
#[test]
fn acceptance_web_baseline_with_url_matches_node_byte_for_byte() {
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
  const query = new URLSearchParams('alpha=1&beta=two+words');
  query.append('gamma', '' + count);
  query.set('beta', '' + count);
  if (query.get('alpha') !== '1' || query.get('beta') !== '1' || query.getAll('beta').length !== 1 || !query.has('gamma')) {
    throw new Error('unexpected URLSearchParams behavior ' + query.toString());
  }
  const browserUrl = new URL('https://example.com/browser?alpha=1#fragment');
  if (browserUrl.origin !== 'https://example.com' || browserUrl.pathname !== '/browser' || browserUrl.search !== '?alpha=1' || browserUrl.hash !== '#fragment' || browserUrl.searchParams.get('alpha') !== '1') {
    throw new Error('unexpected URL behavior ' + browserUrl.href);
  }
  console.log('web baseline url ok');
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
        "web baseline url ok"
    );
}
