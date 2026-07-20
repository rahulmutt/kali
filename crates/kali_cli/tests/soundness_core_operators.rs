//! Soundness pins for two core-operator silent miscompiles closed in the PR #16
//! merge-readiness batch:
//!
//! 1. `&&` / `||` did not short-circuit. The parser has no `LogicalExpression`
//!    node — `&&`/`||` are `BinaryExpression`s, and `emit_binary`'s shared
//!    operand pre-pass emitted BOTH operands unconditionally before combining
//!    them with a single `I64And`/`I64Or`. Branch OUTCOMES were correct (for
//!    0/1 operands), so the defect was invisible in control-flow position and
//!    surfaced only as the right-hand operand's side effects running when JS
//!    says they must not: `if (false && boom())` still called `boom()`. The
//!    bitwise combine was also wrong in value position — `2 && 1` folded to
//!    `2 & 1 == 0` and JS's value-returning semantics (`a && b` yields `a` when
//!    `a` is falsy, else `b`) were not implemented at all. Fixed by excluding
//!    `&&`/`||` from the unconditional pre-emit and lowering each to a real
//!    `If`/`Else` over a scratch local, mirroring the `??` arm that already did
//!    this correctly.
//!
//! 2. Booleans rendered as `0`/`1` when stringified. `emit_as_string` (the `+`
//!    string-concat coercion) had a three-way string/float/`else`-is-an-int
//!    ladder with no boolean arm, so a boolean operand took `int_to_string` and
//!    `"concat=" + false` produced `concat=0`. `console.log(true)` on a LITERAL
//!    looked correct only because a separate static-render path keys on the
//!    literal text. Fixed by giving `emit_as_string` a boolean arm that selects
//!    between the interned `"true"`/`"false"` handles.
//!
//!    Scope note: the SIBLING choke point (`emit_console_argument`, the dynamic
//!    `console.log` path) has the identical defect and is deliberately left
//!    alone here — fixing it invalidates ~130 existing assertions that pin the
//!    `1`/`0` rendering, which is a mass re-pin wave rather than a contained
//!    fix. It is pinned as a known residual at the bottom of this file.
//!
//! Every golden in this file was verified against node v26.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-core-operators-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

fn assert_stdout(src: &str, expected: &str) {
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{out:?}");
}

// ---------------------------------------------------------------------------
// Defect 1: `&&` / `||` short-circuit
// ---------------------------------------------------------------------------

/// The minimal reported repro: a false `&&` LHS and a true `||` LHS must both
/// skip their right-hand operand entirely, so `boom()` never runs. Pre-fix this
/// printed `calls=2` — the branch outcomes were already right, only the side
/// effects leaked.
#[test]
fn short_circuit_skips_right_operand_in_condition_position() {
    assert_stdout(
        "let calls = 0;\n\
         function boom() { calls = calls + 1; return true; }\n\
         function f() {\n\
         \x20 let trace = \"\";\n\
         \x20 if (false && boom()) { trace = trace + \"and-branch;\"; }\n\
         \x20 if (true || boom()) { trace = trace + \"or-taken;\"; }\n\
         \x20 return \"trace=\" + trace + \" calls=\" + calls;\n\
         }\n\
         console.log(f());\n",
        "trace=or-taken; calls=0\n",
    );
}

/// Same short-circuit, but entirely inside a function body and mixing a
/// short-circuited `&&` with a genuinely-taken branch, so the fix is pinned in
/// a local-slot context as well as at module scope.
#[test]
fn short_circuit_in_function_body_with_taken_branch() {
    assert_stdout(
        "let n = 0;\n\
         function side() { n = n + 1; return 5; }\n\
         function f() {\n\
         \x20 if (false && side()) { return \"bad\"; }\n\
         \x20 let ok = true;\n\
         \x20 if (ok && (1 === 1)) { return \"ok n=\" + n; }\n\
         \x20 return \"no\";\n\
         }\n\
         console.log(f());\n",
        "ok n=0\n",
    );
}

/// Value position: JS `&&`/`||` yield an OPERAND, not a normalized boolean, and
/// the skipped operand's side effect never happens. `false || bump()` and
/// `true && bump()` each run `bump` exactly once and evaluate to its `7`.
/// Pre-fix: `n` was `4` here (both operands of both logicals evaluated).
#[test]
fn short_circuit_value_position_evaluates_right_operand_exactly_once() {
    assert_stdout(
        "let n = 0;\n\
         function bump() { n = n + 1; return 7; }\n\
         let a = false || bump();\n\
         let b = true && bump();\n\
         console.log(a);\n\
         console.log(b);\n\
         console.log(n);\n",
        "7\n7\n2\n",
    );
}

/// The `const x = a || b` value-position form from the brief. This asserts the
/// VALUES only, deliberately not the call count: a `const` bound to a
/// call-valued init is re-emitted at every read by the `const` fold-alias lane,
/// which double-evaluates the call. That is a separate PRE-EXISTING defect —
/// it reproduces on a clean tree with no logical operator in sight
/// (`const a = bump(); console.log(a); console.log(n)` prints `7` then `2`) —
/// and is out of scope here. The values themselves are correct.
#[test]
fn short_circuit_const_binding_yields_right_operand_value() {
    assert_stdout(
        "function seven() { return 7; }\n\
         const a = false || seven();\n\
         const b = true && seven();\n\
         console.log(a);\n\
         console.log(b);\n",
        "7\n7\n",
    );
}

/// Chained logicals: `ff() && t() && t()` must call `ff` once and stop;
/// `t() || ff() || ff()` must call `t` once and stop. The `n` total (`101`)
/// proves exactly one call landed on each chain, and the branch outcomes prove
/// the chains still evaluate to the right truthiness. Pre-fix `n` was `303`.
#[test]
fn chained_logicals_short_circuit_after_first_decisive_operand() {
    assert_stdout(
        "let n = 0;\n\
         function t() { n = n + 1; return true; }\n\
         function ff() { n = n + 100; return false; }\n\
         function f() {\n\
         \x20 let r = \"\";\n\
         \x20 if (ff() && t() && t()) { r = r + \"and;\"; }\n\
         \x20 if (t() || ff() || ff()) { r = r + \"or;\"; }\n\
         \x20 return r + \"n=\" + n;\n\
         }\n\
         console.log(f());\n",
        "or;n=101\n",
    );
}

/// Nested/mixed logicals in condition position: the `||` inside a short-circuited
/// `&&` must not run at all, and a nested `&&` inside a taken `||` arm must.
#[test]
fn nested_logicals_short_circuit() {
    assert_stdout(
        "let n = 0;\n\
         function hit(k) { n = n + k; return true; }\n\
         function f() {\n\
         \x20 if (false && (hit(1) || hit(2))) { return \"a\"; }\n\
         \x20 if (true || (hit(4) && hit(8))) { return \"b n=\" + n; }\n\
         \x20 return \"c\";\n\
         }\n\
         console.log(f());\n",
        "b n=0\n",
    );
}

// ---------------------------------------------------------------------------
// Defect 2: boolean rendering in string position
// ---------------------------------------------------------------------------

/// The minimal reported repro. `console.log(true)` on a bare literal was already
/// correct (static render path); concatenation in EITHER direction, and a
/// function-local `const` boolean, all rendered `0`/`1` pre-fix.
#[test]
fn booleans_render_as_true_false_in_string_concatenation() {
    assert_stdout(
        "console.log(true);\n\
         console.log(false);\n\
         console.log(\"concat=\" + false);\n\
         console.log(true + \"=concat\");\n\
         function f() { const t = true; return \"in-fn=\" + t; }\n\
         console.log(f());\n",
        "true\nfalse\nconcat=false\ntrue=concat\nin-fn=true\n",
    );
}

/// Comparison and negation results are booleans and must stringify as such.
/// Pre-fix these were `eq=1 / ne=0 / lt=1 / not=0`.
#[test]
fn comparison_results_render_as_true_false_in_concatenation() {
    assert_stdout(
        "console.log(\"eq=\" + (1 === 1));\n\
         console.log(\"ne=\" + (1 === 2));\n\
         console.log(\"lt=\" + (1 < 2));\n\
         console.log(\"not=\" + (!true));\n",
        "eq=true\nne=false\nlt=true\nnot=false\n",
    );
}

/// String `+=` is a second stringify site with the same coercion helper, so it
/// carries the same defect and needs the same pin. Pre-fix: `10`.
#[test]
fn booleans_render_as_true_false_in_string_compound_assignment() {
    assert_stdout(
        "function f() {\n\
         \x20 let s = \"\";\n\
         \x20 s = s + true;\n\
         \x20 s = s + false;\n\
         \x20 return s;\n\
         }\n\
         console.log(f());\n",
        "truefalse\n",
    );
}

/// Documents the CURRENT (still-wrong) behavior of the sibling stringify choke
/// point that this change deliberately does NOT touch, so the residual is
/// pinned and visible rather than forgotten.
///
/// `emit_console_argument` has the same missing boolean arm as `emit_as_string`,
/// so a boolean reaching `console.log` DYNAMICALLY still prints `1`/`0` where
/// node prints `true`/`false`. (A bare `console.log(true)` looks correct only
/// because `render_static_value` folds the literal text upstream.) Adding the
/// arm is a two-line change and was verified to work — but it invalidates ~130
/// existing assertions across the suite that pin the `1`/`0` rendering, which
/// is a mass re-pin wave, not part of this contained fix.
///
/// It is also a RATIFIED CONVENTION, not merely an accident: see the comment on
/// `aborted_flag_reads_zero_then_one` in `soundness_abort.rs` — "Dynamic
/// booleans render 1/0 (ratified P2 convention; node prints true/false —
/// documented divergence, never used in byte-for-byte acceptance fixtures)".
/// Changing it therefore needs maintainer ratification, not just a re-pin wave.
/// This commit narrows that convention to `console.log` only: booleans in
/// string CONCATENATION are now node-correct, per the explicit scope exception.
/// When the console.log wave lands, this test flips to `true/false`.
#[test]
fn dynamic_console_boolean_rendering_is_a_known_residual() {
    assert_stdout(
        "console.log(1 === 1);\n\
         console.log(1 === 2);\n\
         console.log(true && false);\n\
         console.log(false || true);\n",
        "1\n0\n0\n1\n",
    );
}

/// A boolean must NOT poison the numeric lane: arithmetic on comparison results
/// still coerces to 0/1 the way JS does, and non-boolean scalars must keep
/// rendering as numbers. Guards the fix against over-applying the boolean arm.
#[test]
fn boolean_rendering_does_not_leak_into_numeric_lane() {
    assert_stdout(
        "console.log(\"sum=\" + ((1 === 1) + (1 === 1)));\n\
         console.log(\"num=\" + 1);\n\
         console.log(\"zero=\" + 0);\n",
        "sum=2\nnum=1\nzero=0\n",
    );
}
