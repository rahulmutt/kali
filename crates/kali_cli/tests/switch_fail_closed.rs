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

// Fix round 1 (Critical 2 companion, cell 16 closed for real): a bare
// identifier bound to a boolean is a DIFFERENT trap than `x > 0` above.
// `static_equality_class` proves boolean-ness only for a SYNTACTIC form
// (a literal, a comparison, `!`, `delete`) — a bare identifier like `d` here
// carries no such proof, so this must be denied by the identifier arm's
// positive-evidence requirement (`name_is_declared_parameter(name) ||
// binding_is_proven_numeric(name)`) instead: `d` is not a parameter, and its
// only write (`var d = true;`) is not `write_value_is_numeric`, so
// `binding_is_proven_numeric` is false and this fails Rule 1. Before fix
// round 1 this ran to completion and printed `v=one` where node prints
// `v=other` (measured, exit 0 both sides) — a silent miscompile, not a
// rejection.
#[test]
fn a_boolean_identifier_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s() {\n\
           var d = true;\n\
           switch (d) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s());\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 2 (human ruling): a boolean-valued PARAMETER discriminant is the
// residual fix round 1 disclosed but did not close. `kali_common::Repr` has
// no `Boolean` variant, so nothing distinguishes a boolean-passed parameter
// from a numeric one by repr alone -- the fix is narrowing WHICH parameters
// the identifier arm trusts at all: `identifier_is_provable_i64_scalar` now
// requires `param_has_numeric_literal_inflow`, positive proof that EVERY
// enumerated call site of `s` passes a numeric literal for `b`. `s(true)` is
// the (only) call site and its argument is not a numeric literal, so `b`
// fails this proof and Rule 1 denies. Before fix round 2 this ran to
// completion and printed `v=one` where node prints `v=other` (measured, exit
// 0 both sides) -- a silent miscompile, not a rejection.
#[test]
fn a_boolean_parameter_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(b) {\n\
           switch (b) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(true));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 2 positive-direction coverage: a parameter fed a COMPUTED
// (non-literal) argument at its only call site must ALSO be denied --
// `param_has_numeric_literal_inflow` requires the argument to be
// `expr_is_nonneg_int_literal`-grade specifically, not merely
// arithmetic/scalar. `n + 1` is syntactically scalar (it would satisfy the
// weaker `scalar_inflow_params`/`binding_is_proven_numeric`-style proof) but
// is NOT a numeric literal, so this must still fail Rule 1 -- proving the new
// rule actually denies computed arguments and is not vacuously satisfied by
// anything scalar-shaped.
#[test]
fn a_computed_parameter_argument_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         var n = 0;\n\
         console.log(\"v=\" + s(n + 1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 3: the inflow predicate was widened to accept unary `+`/`-`
// DIRECTLY over a numeric literal (so `s(-1)` admits again -- see
// `numeric_switch_selects_correctly_with_a_negative_case_test` in
// `switch_runtime.rs`). This pins the float axis specifically, not just the
// boolean axis: `-1.5` is unary `-` over a LITERAL, syntactically the same
// shape as `-1`, but `expr_is_nonneg_int_literal`'s `n.fract() == 0.0` check
// must still catch the fractional magnitude on both sides of zero. If the
// widening had been done by loosening that check instead of by peeling the
// unary wrapper around the UNCHANGED nonneg-int check, this is the fixture
// that would have caught it (Task 4's float requirement depends on this
// staying closed).
#[test]
fn a_negative_float_parameter_argument_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(-1.5));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        !out.contains("E4201"),
        "must be denied at the switch choke point (E5506), not fall through to an \
         invalid-module error (E4201); got: {out}"
    );
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// A `let`/`const` declaration in a clause body is denied (Rule 5): block
// shadowing across case labels is unmodeled (register R-10), so a
// case-scoped binding would build on a known-broken foundation.
//
// NOTE (fix round 1): this braced form is denied by RULE 4 (the clause's
// last statement is a `Block`, not a `Branch("return")`), not Rule 5 —
// `declares_block_scoped_binding` is never even consulted for it. Kept as a
// pin on its own right (a braced clause with an inner `let` must still
// fail), but see the unbraced test below for actual Rule-5 coverage.
#[test]
fn a_let_declaration_in_a_braced_clause_is_fail_closed_via_rule_4() {
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
    // Fix round 4: this test's NAME claims Rule 4, so pin the Rule-4 reason —
    // otherwise nothing distinguishes it from the Rule-5 test below (or from a
    // Rule-1 denial that would make both of them pass for the wrong reason).
    assert!(
        out.contains("a clause that does not end in `return`"),
        "the braced form must be denied by Rule 4 (the clause's last statement \
         is a Block, not a return), as this test's name claims; got: {out}"
    );
}

// The unbraced form: `let a = 1;` and `return ...;` are two SIBLING
// statements in the clause body (not nested inside a `Block`), so this
// clause's last statement genuinely IS a `return` (Rule 4 passes) and Rule 5
// (`declares_block_scoped_binding`) is what must catch the `let`. Without
// this fixture `declares_block_scoped_binding` had zero test coverage.
#[test]
fn a_let_declaration_in_an_unbraced_clause_is_fail_closed_via_rule_5() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: let a = 1; return \"A\" + a;\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    // Fix round 4: pin the Rule-5 reason this test's name claims — without it
    // the assertion above is satisfied by ANY denial, including the Rule-4 one
    // the test above pins, which would make this test's whole premise
    // (`declares_block_scoped_binding` has coverage) untrue.
    assert!(
        out.contains("a `let`/`const` declaration in a clause body"),
        "the unbraced form must be denied by Rule 5 \
         (declares_block_scoped_binding), as this test's name claims; got: {out}"
    );
}

// Fix round 4, LEAK 2 (Critical): `param_has_numeric_literal_inflow` proves
// what flowed IN at the call sites; it was being consumed as a proof of the
// discriminant's value AT THE SWITCH. Any write to the parameter between
// function entry and the switch laundered a non-numeric value through a clean
// numeric-literal call site. Measured at HEAD 83a401c311: `v=one` where node
// prints `v=other`, exit 0 on both sides, no diagnostic — a silent
// miscompile. Closed by conjoining `kali_types::repr_infer`'s
// `readonly_params` (a POSITIVE enumeration of the forms in which a name is
// provably only read) into `numeric_literal_inflow_params`, which makes the
// inflow proof a proof of VALUE.
#[test]
fn a_written_parameter_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           x = true;\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 4, LEAK 2, the sharpest variant. `t` is a `var` holding a boolean
// that `binding_is_proven_numeric` correctly REFUSES — `switch (t)` directly
// is already denied (`a_boolean_identifier_discriminant_is_fail_closed`). This
// pins that copying that same refused value into a parameter does not launder
// it past the refusal: the parameter's numeric-literal inflow from `s(1)` must
// not stand in for the value it actually holds at the switch. This is also the
// fixture that pins the OTHER half of the fix — the identifier arm's parameter
// branch is now an if/else, not an `||` with `binding_is_proven_numeric`, so a
// PARAMETER can no longer be rescued by the numeric-binding write proof (which
// accepts a write whose RHS is another parameter on SCALAR inflow alone).
#[test]
fn a_parameter_overwritten_from_a_boolean_binding_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var t = 1 > 0;\n\
           x = t;\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 4, LEAK 2 companion: a second PARAMETER copied into the
// discriminant parameter. `x = y` earns the numeric-binding write proof for
// `x` on nothing stronger than `y`'s SCALAR inflow, and a boolean literal
// argument is scalar-syntactic — so `s(1, true)` had a "proven numeric" `x`
// holding `true`. Measured at HEAD 83a401c311: `v=one` vs node's `v=other`,
// exit 0. Denied now because a declared parameter's ONLY admitted proof is
// numeric-literal inflow AND never-written.
#[test]
fn a_parameter_overwritten_from_another_parameter_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x, y) {\n\
           x = y;\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1, true));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}

// Fix round 4, LEAK 1 (Critical): a `new`-invocation site was invisible to
// BOTH halves of the parameter proof. `new s(true)` is not a `CallEdge` (only
// a bare-identifier CallExpression builds one), so Step 1c's ∀-over-enumerated
// -edges never saw it, AND `repr_infer`'s `visit_expr` never visited a
// NewExpression's CALLEE at all, so `s` never reached the identifier arm that
// populates `escaping_function_names` — the backstop whose entire job is to
// guarantee that "∀ enumerated edges" means "∀ invocation sites". The `s(1)`
// site supplied clean ∃-evidence and nothing vetoed. Measured at HEAD
// 83a401c311: kali printed `one`/`one` where node prints `one`/`other`, exit
// 0, no diagnostic. Note this fixture NEEDS the `s(1)` call: without an
// enumerated numeric-literal site there is no ∃-evidence and the switch would
// deny for an unrelated reason, proving nothing about the `new` site.
#[test]
fn a_new_invocation_site_of_the_enclosing_function_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: return \"one\";\n\
             default: return \"other\";\n\
           }\n\
         }\n\
         console.log(\"a=\" + s(1));\n\
         new s(true);\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("the discriminant is not a proven integer"),
        "must be denied by Rule 1 (the discriminant), not some other rule; got: {out}"
    );
}
