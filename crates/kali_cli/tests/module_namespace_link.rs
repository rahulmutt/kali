//! End-to-end acceptance tests for the AST module-linking pass
//! (throw-fallout Stage 5). Unlike `kali_cli`'s in-crate `module_link.rs`
//! unit tests (which only assert the AST rename/append/order happened),
//! these tests actually RUN the built binary and assert real stdout — the
//! false-confidence gap a review finding on Task 7 identified: an existing
//! unit test (`append_linked_functions_still_renames_genuine_sibling_call_no_shadow`)
//! asserted the sibling-call rename happened but never checked the result
//! actually RESOLVED, so it stayed green over a defect where the linked
//! clones were appended in alphabetical (`BTreeMap` key) order instead of
//! dependency order — whenever a caller happened to sort before its callee,
//! the appended program failed (or, worse, silently produced a wrong value)
//! at the resolver, not at `append_linked_functions` itself.
//!
//! (Task 8 extends this file with the fuller Stage 5 acceptance suite; this
//! file starts minimal, with just the two dependency-order pins the Task 7
//! fix requires.)

use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// `helper` is declared BEFORE `f` in the linked module's source (`f` calls
/// `helper`). Node prints `inside helper`, `7`, `main loaded` (verified
/// against a real `node` run of this exact fixture). A `Number` return
/// (`return 7`, not `return 7n`/`String(...)`) and a call made DIRECTLY at
/// the `console.log` site (never bound to a `const` first) are deliberate:
/// two PRE-EXISTING, unrelated kali codegen bugs — `String(bigint)` folds to
/// `0`, and a `const` bound to a call re-evaluates it at every use,
/// duplicating side effects — would otherwise corrupt this test's expected
/// output for reasons that have nothing to do with the module-linking
/// dependency-order fix under test here.
#[test]
fn run_supports_namespace_linked_sibling_call_helper_declared_before_export() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"function helper() { console.log("inside helper"); return 7; }
export function f() { return helper(); }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
console.log(ns.f());
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "inside helper\n7\nmain loaded\n"
    );
}

/// The mirror declaration order: `f` (the export, and the caller) is
/// declared BEFORE `helper` (the private callee) in the linked module's
/// source. This is the plan's own mandated Task 4/5 fixture shape, and the
/// exact shape the pre-fix alphabetical (`BTreeMap` key) append order got
/// wrong: `"f" < "helper"` alphabetically, which happens to match THIS
/// source order too, so the pre-fix code appended the caller's clone before
/// its callee's regardless of which order the module's source actually
/// declared them in. Same expected stdout as the forward-order test above —
/// proving the fix is genuinely dependency-driven, not order-of-appearance
/// driven.
#[test]
fn run_supports_namespace_linked_sibling_call_export_declared_before_helper() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"export function f() { return helper(); }
function helper() { console.log("inside helper"); return 7; }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
console.log(ns.f());
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "inside helper\n7\nmain loaded\n"
    );
}
