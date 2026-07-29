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

// The exact denial reason each rule emits. A test that asserts only "some
// E5506" silently degrades the moment the cell it names starts denying for a
// DIFFERENT reason — most often Rule 1, since a discriminant proof regression
// makes every switch in the file deny — and it keeps passing while having lost
// its entire purpose. That failure mode has now been found twice on this plan,
// so it is closed structurally: every fail-closed test below pins the rule, via
// `assert_denied_by`, and a new one has an obvious shape to copy.
const RULE_1_DISCRIMINANT: &str = "the discriminant is not a proven integer or string";
const RULE_2_CASE_TEST: &str = "a `case` test that is not a literal in the discriminant's domain";
const RULE_4_TERMINATOR: &str = "a clause that does not end in `return`";
const RULE_5_BLOCK_BINDING: &str = "a `let`/`const` declaration in a clause body";
// Not a switch rule: `kali_types`'s `plain_write_targets` shape conflict, which
// refuses a string-reachable binding fed by an unbacked plain source BEFORE
// `switch_plan` ever runs. Tests that pin a SWITCH-side conjunct must assert
// this is ABSENT, or they cannot tell the two apart.
const REPR_MIXED_CONFLICT: &str = "is used as both a string and a number";

/// Assert `out` was denied by the named rule, not merely denied.
fn assert_denied_by(out: &str, rule: &str, what: &str) {
    assert!(out.contains("E5506"), "expected E5506 for {what}, got: {out}");
    assert!(
        out.contains(rule),
        "{what} must be denied by `{rule}`, not some other rule — a test that \
         accepted any E5506 would still pass if this cell silently degraded \
         into a different denial, losing the whole point of the test; got: {out}"
    );
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
    // Rule 4, not Rule 1: `x` here IS a proven i64, so a Rule 1 denial would
    // mean the discriminant proof had regressed and this cell had stopped
    // testing fallthrough at all.
    assert_denied_by(&out, RULE_4_TERMINATOR, "module-scope true fallthrough");
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
    assert_denied_by(&out, RULE_4_TERMINATOR, "function-scope true fallthrough");
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
    // Rule 2, not Rule 1: `s(1, 1)` gives BOTH parameters clean numeric-literal
    // inflow, so `x` is a proven discriminant and the only thing left to deny
    // is the `case y:` test itself.
    assert_denied_by(&out, RULE_2_CASE_TEST, "a bare-identifier case test");
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

// ===========================================================================
// R-35 Task 8: the STRING discriminant axis. Every test below is the string
// analogue of a leak Task 7 measured on the numeric axis, plus the two
// mismatched-domain directions. They are the regression suite for the claim
// "the allowlist widened by adding a proof, and no rejection was removed".
// ===========================================================================

// Step 6 (brief): a string `case` test against a NUMERIC discriminant. Rule 2
// is domain-matched, so this is DENIED rather than compiled into a comparison
// that can never be true. Node falls to `default` here; agreeing with node by
// accident is not a lowering proof, so kali refuses instead.
#[test]
fn a_string_case_against_a_numeric_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"1\": return \"A\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    // RULE 2 is the whole point. `s(1)` gives `x` clean numeric-literal inflow,
    // so the discriminant IS proven and Rule 1 has nothing to say; only a
    // domain-matched rule 2 can deny this. If the string proof ever regressed
    // and this started denying at Rule 1, the test would still "pass" while no
    // longer testing domain matching at all — hence the pin.
    assert_denied_by(&out, RULE_2_CASE_TEST, "a string case on a numeric disc");
}

// The OTHER direction: a numeric `case` test against a PROVEN STRING
// discriminant. Both directions must be denied or rule 2 is not actually
// domain-matched -- it would just be two independent one-way checks.
#[test]
fn a_numeric_case_against_a_string_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: return \"A\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"a\"));\n",
    );
    // RULE 2 again, and for the same reason mirrored: `s("a")` proves `x` a
    // string, so the discriminant is admitted and only the numeric `case 1:`
    // can deny. Pinning BOTH directions at Rule 2 is what makes this a claim
    // about domain MATCHING rather than two unrelated one-way checks.
    assert_denied_by(&out, RULE_2_CASE_TEST, "a numeric case on a string disc");
}

// String analogue of Task 7 fix round 4 LEAK 2: a WRITE launders a non-string
// value into a parameter with clean string inflow. On the numeric axis this
// silently miscompiled; on the string axis `repr_infer`'s own
// `plain_write_targets` rule already turns "string-reachable node fed by an
// unbacked plain source" into a shape conflict, so the program is refused
// before the switch proof is even consulted. Pinned here so a future change to
// that rule cannot quietly reopen the switch cell.
#[test]
fn a_boolean_written_into_a_string_parameter_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           x = true;\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"b\"));\n",
    );
    // Denied UPSTREAM of the switch, by `repr_infer`'s `plain_write_targets`
    // shape conflict — pinned on that message specifically, because the claim
    // this test makes is "the write is caught at the source, program-wide",
    // not merely "this switch was refused".
    assert_denied_by(&out, REPR_MIXED_CONFLICT, "a boolean written into a string param");
}

// Same leak through a SECOND PARAMETER rather than a literal -- the exact
// shape (`x = y` with `s(<good>, <bad>)`) that defeated the numeric proof's
// `||` disjunction in fix round 3.
#[test]
fn a_write_from_another_parameter_into_a_string_parameter_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x, y) {\n\
           x = y;\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"b\", true));\n",
    );
    assert_denied_by(&out, REPR_MIXED_CONFLICT, "a write from another parameter");
}

// Same leak through a comparison RESULT copied via a local -- the shape
// `binding_is_proven_numeric` itself refuses but the numeric proof's
// disjunction admitted anyway.
#[test]
fn a_write_from_a_comparison_result_into_a_string_parameter_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var t = 1 > 0;\n\
           x = t;\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"b\"));\n",
    );
    assert_denied_by(&out, REPR_MIXED_CONFLICT, "a write from a comparison result");
}

// A write of a STRING into the parameter is denied too. Nothing about the
// value is wrong here -- node prints `v=9` and a string discriminant would
// still be a string. It is denied because `readonly_params` is a POSITIVE
// enumeration of read-only forms and a written parameter simply is not in it.
// Fail-closed, deliberately: this pins that the parameter arm's evidence is
// "never written", not "never written with something bad", so no future edit
// can turn it into a denylist of bad write RHSs.
#[test]
fn a_written_string_parameter_is_fail_closed_even_when_written_with_a_string() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           x = \"z\";\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             case \"z\": return 9;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"b\"));\n",
    );
    // This is the SOLE pin on `param_is_readonly` being load-bearing, so it has
    // to isolate that conjunct rather than accept any denial. A string-RHS
    // write builds a string-reachable edge, so `plain_write_targets` does NOT
    // fire here (measured: the same function without the switch,
    // `function s(x) { x = "z"; return x + "!"; } s("b")`, compiles and runs at
    // exit 0). With the mixed-conflict message asserted ABSENT, the only thing
    // left that can produce this Rule 1 denial is `param_is_readonly`.
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "a string-written string param");
    assert!(
        !out.contains(REPR_MIXED_CONFLICT),
        "this cell must isolate `param_is_readonly`: a string-RHS write is NOT \
         a mixed string/number flow, so if the shape conflict fires here the \
         test is measuring `plain_write_targets` instead and proves nothing \
         about the readonly conjunct; got: {out}"
    );
}

// `new s(true)` builds NO call edge, so the call-edge quantifier cannot see
// it; it is visible only as an ESCAPE (`mark_new_callee_escapes`). The string
// parameter arm conjoins the SAME `escaping_function_names` set Task 7's
// numeric proof does, so this denies even though the only enumerable call site
// (`s("b")`) supplies a clean string.
#[test]
fn a_new_expression_call_site_denies_a_string_parameter_discriminant() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         new s(true);\n\
         console.log(\"v=\" + s(\"b\"));\n",
    );
    // Isolates `!function_escapes` the same way the readonly cell above
    // isolates its conjunct: `new s(true)` builds no call edge and writes
    // nothing, so no shape conflict is raised (measured: the same program
    // without the switch, `function s(x) { return x + "!"; } new s(true);
    // s("b")`, compiles and runs at exit 0). With the conflict asserted absent
    // and `x` never written, the escape gate is the only remaining conjunct.
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "a `new` call site");
    assert!(
        !out.contains(REPR_MIXED_CONFLICT),
        "this cell must isolate the escape gate, not a shape conflict; got: {out}"
    );
}

// The escape gate proper: `s` passed as a callback can be invoked through a
// site no analysis in this compiler enumerates, so no per-parameter fact
// derived from the enumerated sites is a fact about every invocation.
#[test]
fn an_escaping_function_denies_a_string_parameter_discriminant() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         setTimeout(s, 0);\n\
         console.log(\"v=\" + s(\"b\"));\n",
    );
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "an escaping function");
    assert!(
        !out.contains(REPR_MIXED_CONFLICT),
        "this cell must isolate the escape gate, not a shape conflict; got: {out}"
    );
}

// Existential laundering through a PASS-THROUGH chain: `s`'s only argument is
// a bare identifier, so nothing about `s`'s parameter is proven unless `t`'s
// parameter is itself proven a string -- and `t(true)` proves it is not.
#[test]
fn a_pass_through_chain_carrying_a_boolean_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         function t(y) { return s(y); }\n\
         console.log(\"v=\" + t(true));\n",
    );
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "a boolean pass-through chain");
}

// No evidence is not vacuous truth: a function with ZERO call sites has no
// proof for its parameter and is denied.
#[test]
fn a_string_switch_in_a_never_called_function_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + \"no-call\");\n",
    );
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "a never-called function");
}

// Free globals against STRING cases. `globalThis`/`undefined`/`NaN` are not
// program-bound, so the identifier arm's first gate (`name_is_program_bound`,
// the check Task 7's identifier arm originally omitted) denies them. Pinned on
// the string axis too so the gate cannot be dropped from one arm only.
#[test]
fn free_globals_against_string_cases_are_fail_closed() {
    for disc in ["globalThis", "undefined", "NaN"] {
        let out = run_js_expect_failure(&format!(
            "function s() {{\n\
               switch ({disc}) {{\n\
                 case \"a\": return 1;\n\
                 default: return 3;\n\
               }}\n\
             }}\n\
             console.log(\"v=\" + s());\n"
        ));
        // Rule 1 specifically: this pins `name_is_program_bound` as the gate
        // that denies a free global, which is the check Task 7's identifier arm
        // originally omitted.
        assert_denied_by(&out, RULE_1_DISCRIMINANT, &format!("`switch ({disc})`"));
        assert!(
            !out.contains("E4201"),
            "{disc} must be denied at the switch choke point, not by an \
             invalid module, got: {out}"
        );
    }
}

// A `case` test that is a runtime string EXPRESSION rather than a literal is
// denied: rule 2 admits a string LITERAL only, so a concat -- which would
// materialize a fresh handle per clause evaluation -- never reaches the emit.
#[test]
fn a_concatenated_case_test_against_a_string_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\" + \"b\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"ab\"));\n",
    );
    // Rule 2: `s("ab")` proves the discriminant a string, so the only thing
    // left to deny is the case test's non-literal shape.
    assert_denied_by(&out, RULE_2_CASE_TEST, "a concatenated case test");
}

// An object / array discriminant against string cases. Both are i64-shaped
// wasm values exactly like a string handle, so only the proof stands between
// them and `__streq` reading a non-string handle's bits.
#[test]
fn aggregate_discriminants_against_string_cases_are_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         var o = { k: 1 };\n\
         console.log(\"v=\" + s(o));\n",
    );
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "an object discriminant");
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": return 1;\n\
             default: return 3;\n\
           }\n\
         }\n\
         var a = [1, 2];\n\
         console.log(\"v=\" + s(a));\n",
    );
    assert_denied_by(&out, RULE_1_DISCRIMINANT, "an array discriminant");
}

// Rules 4 and 5 are unchanged by the string widening: a non-`return` clause
// and a `let` in a clause body stay denied on the string axis too. Without
// these, "widened the discriminant domain" could silently have meant "widened
// the clause allowlist as well".
#[test]
fn string_switch_still_obeys_the_clause_rules() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var r = 0;\n\
           switch (x) {\n\
             case \"a\": r = 1;\n\
             default: return 3;\n\
           }\n\
           return r;\n\
         }\n\
         console.log(\"v=\" + s(\"a\"));\n",
    );
    // Rule 4 SPECIFICALLY, not Rule 1. `s("a")` proves the discriminant, so
    // this cell's whole claim is "the clause-terminator rule still applies on
    // the string axis". A Rule 1 denial here would mean the string proof had
    // regressed and the clause rules were no longer being exercised at all —
    // the exact silent degradation this pin exists to catch.
    assert_denied_by(&out, RULE_4_TERMINATOR, "a non-`return` string clause");
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"a\": let q = 1; return q;\n\
             default: return 3;\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(\"a\"));\n",
    );
    assert_denied_by(&out, RULE_5_BLOCK_BINDING, "a `let` in a string clause");
}
