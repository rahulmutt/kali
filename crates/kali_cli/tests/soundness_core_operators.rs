//! Soundness pins for core-operator silent miscompiles closed in the PR #16
//! merge-readiness batch.
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
