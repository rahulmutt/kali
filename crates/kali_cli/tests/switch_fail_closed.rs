use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js_expect_failure(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        !output.status.success(),
        "expected rejection but it ran\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

// A switch nested inside a `for` loop is its own risk surface (the loop can
// die at iteration 0 before the switch's behavior is ever observed), so this
// pins fail-closed on that shape too. Every iteration logs first, so a
// truncated loop is distinguishable from a mis-selected clause — if this ever
// ran instead of failing closed, the output would visibly reveal which defect
// occurred rather than passing by accident.
#[test]
fn switch_nested_in_for_loop_is_fail_closed_not_silently_wrong() {
    let out = run_js_expect_failure(
        "for (let i = 0; i < 3; i = i + 1) {\n\
           console.log(\"iter=\" + i);\n\
           switch (i) {\n\
             case 0: continue;\n\
             case 1: break;\n\
             default: continue;\n\
           }\n\
         }\n\
         console.log(\"done\");\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("switch"),
        "the diagnostic must name switch as the limit, got: {out}"
    );
}

// Scope is a required test axis. Module scope cannot use `return`, so this
// pins the DENIAL side at module scope (the admitted twin arrives with
// Task 9's `break`): true fallthrough (no terminator at all) must still fail
// closed here too.
#[test]
fn true_fallthrough_at_module_scope_is_fail_closed() {
    let out = run_js_expect_failure(
        "var v = \"?\";\n\
         var x = 20;\n\
         switch (x) {\n\
           case 10: v = \"A\";\n\
           case 20: v = \"B\";\n\
           default: v = \"D\";\n\
         }\n\
         console.log(\"v=\" + v);\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}

// True fallthrough (a clause with no `return`/`break`) is denied at function
// scope too — Task 7 admits ONLY all-`return` clauses; true fallthrough
// arrives nowhere in this plan (it is never a supported lowering shape).
#[test]
fn true_fallthrough_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var r = 0;\n\
           switch (x) {\n\
             case 1: r = 1;\n\
             case 2: r = 2;\n\
           }\n\
           return r;\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}

// A `case` test must be a numeric literal (Rule 2). A non-literal test (here,
// another parameter) is denied rather than silently compared as if it were.
#[test]
fn a_non_literal_case_test_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x, y) {\n\
           switch (x) {\n\
             case y: return \"A\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1, 1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}

// ADDITIVE (Task 4's re-derived boundary matrix, cell 15): a float
// discriminant is denied. The trap this test exists to close: matrix cell 15
// records that a float discriminant already fails today with a DIFFERENT
// diagnostic, `E4201` (an invalid-module error at load time, for an unrelated
// reason), so a test that only asserted "this is rejected" would pass for the
// wrong reason and prove nothing. This asserts E5506 specifically IS present
// and E4201 is NOT — i.e. `is_provable_i64_scalar`'s float check
// (`is_float_valued`) is what denies this, at the switch choke point, before
// any invalid module could ever be produced.
#[test]
fn a_float_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: return \"a\";\n\
             default: return \"b\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1.5));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        !out.contains("E4201"),
        "the float discriminant must be denied at the switch choke point \
         (E5506), not fall through to an invalid-module error (E4201); got: {out}"
    );
}

// ADDITIVE (Task 4's re-derived boundary matrix, cell 16): a boolean
// discriminant is denied. Cell 16 is one of only two cells where kali is
// otherwise CORRECT — this is a deliberate regression to fail-closed
// (Task 11 records it as such), accepted because fail-closed beats
// accidentally-right: `static_equality_class` proving `EqClass::Boolean` is
// the ONLY existing proof that a value is a JS boolean (a plain `Repr::I64`
// scalar carries no boolean axis of its own — see `emit/equality.rs`'s doc
// comment), and `is_provable_i64_scalar` denies on it explicitly.
#[test]
fn a_boolean_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x > 0) {\n\
             case 1: return \"pos\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "the boolean discriminant must be denied by is_provable_i64_scalar's \
         boolean check, not by a different rule (e.g. the numeric case-test \
         rule); got: {out}"
    );
}

// A `let`/`const` declaration in a clause body is denied (Rule 5): block
// shadowing across case labels is unmodeled (register R-10), so a
// case-scoped binding would build on a known-broken foundation.
#[test]
fn a_let_declaration_in_a_clause_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: { let a = 1; return \"A\" + a; }\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}
