//! Soundness pins for `const` as a real BINDING.
//!
//! Before this suite, a `const` declarator whose initializer was not on a
//! hand-maintained promotion DENYLIST (`collect_function_locals_from_node` in
//! `crates/kali_codegen/src/lower.rs`) was not bound at all: the initializer
//! AST node was recorded in `FunctionEmitter::bindings` and RE-EMITTED at every
//! read site (`emit/control_flow.rs` identifier arm). A `const` was therefore a
//! textual substitution, not a binding, which produced two silent wrong-value
//! classes with exit 0 and no diagnostic:
//!
//! 1. STALE/WRONG VALUE — the initializer observes state at the READ site
//!    instead of the DECLARATION site. The universal swap idiom
//!    (`const tmp = a; a = b; b = tmp;`) silently lost a value: kali printed
//!    `a=2 b=2` where node prints `a=2 b=1`.
//! 2. REPEATED SIDE EFFECTS — an initializer that calls a function ran once at
//!    the declaration plus once per read (N reads => N+1 evaluations).
//!
//! Both held at module scope and in-function. The fix replaces the denylist
//! with an ALLOWLIST at the same choke point: a `const` keeps the compile-time
//! fold lane only when re-emitting its initializer is provably observationally
//! identical (literals, operators over literals, reads of names that are never
//! reassigned, function-like inits); everything else is promoted to an eager
//! local slot and bound exactly once at the declaration site.
//!
//! Every golden below is verified against node v26.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-const-binding-{}-{}-{}",
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

// The headline case: the universal swap idiom. `tmp` must capture `a`'s value
// at the declaration, not re-read `a` after it has been reassigned.
// node: `a=2 b=1`. Pre-fix kali: `a=2 b=2`, exit 0, no diagnostic.
#[test]
fn swap_idiom_in_function_captures_value_at_declaration() {
    assert_stdout(
        r#"function f() {
  let a = 1, b = 2;
  const tmp = a;
  a = b;
  b = tmp;
  return "a=" + a + " b=" + b;
}
console.log(f());
"#,
        "a=2 b=1\n",
    );
}

// The same idiom at module scope (`_start`). The module-scope path had the
// identical hole — the existing `is_pure_module_const_init` gate only guards
// reads of a module binding from INSIDE a function, not the declaration.
#[test]
fn swap_idiom_at_module_scope_captures_value_at_declaration() {
    assert_stdout(
        r#"let a = 1, b = 2;
const tmp = a;
a = b;
b = tmp;
console.log("a=" + a + " b=" + b);
"#,
        "a=2 b=1\n",
    );
}

// Side-effect count, in-function. The counter is a module-scope SCALAR (never
// an array: a growable array that escapes into a call fails closed or no-ops
// here and would measure nothing). Three reads of `c` must still leave exactly
// one increment. Pre-fix: `n == 4` (one declaration + three reads).
#[test]
fn const_initializer_side_effect_fires_exactly_once_in_function() {
    assert_stdout(
        r#"let n = 0;
function bump() { n = n + 1; return 7; }
function g() {
  const c = bump();
  return c + c + c;
}
console.log(g());
console.log(n);
"#,
        "21\n1\n",
    );
}

// Side-effect count at module scope. Pre-fix: `n == 4`.
#[test]
fn const_initializer_side_effect_fires_exactly_once_at_module_scope() {
    assert_stdout(
        r#"let n = 0;
function bump() { n = n + 1; return 7; }
const c = bump();
console.log(c + c + c);
console.log(n);
"#,
        "21\n1\n",
    );
}

// A `const` read ZERO times must still evaluate its initializer exactly once —
// node evaluates the initializer eagerly at the declaration regardless of use.
// This already held pre-fix (the declaration site emitted the init and dropped
// it); pinned so the fix does not regress it into a lazy/never evaluation.
#[test]
fn unread_const_initializer_still_evaluates_exactly_once() {
    assert_stdout(
        r#"let n = 0;
function bump() { n = n + 1; return 7; }
function g() {
  const c = bump();
  return 5;
}
console.log(g());
console.log(n);
"#,
        "5\n1\n",
    );
}

// A `const` bound to an EXPRESSION over variables that are reassigned
// afterwards, read both before and after the reassignment: both reads must see
// the value captured at the declaration. Pre-fix kali printed `1100 1100`
// (both reads re-evaluated `a + b` against the post-reassignment values).
#[test]
fn const_over_reassigned_variables_is_read_stable() {
    assert_stdout(
        r#"function h() {
  let a = 1, b = 10;
  const s = a + b;
  const before = s;
  a = 100;
  b = 1000;
  const after = s;
  return before + " " + after;
}
console.log(h());
"#,
        "11 11\n",
    );
}

// The same at module scope.
#[test]
fn const_over_reassigned_variables_is_read_stable_at_module_scope() {
    assert_stdout(
        r#"let a = 1, b = 10;
const s = a + b;
const before = s;
a = 100;
b = 1000;
const after = s;
console.log(before + " " + after);
"#,
        "11 11\n",
    );
}

// The common case: a `const` bound to a literal, at both scopes, read several
// times. Must stay correct (and stays on the cheap compile-time fold lane —
// re-emitting a literal is observationally identical).
#[test]
fn const_bound_to_literal_stays_correct() {
    assert_stdout(
        r#"const K = 7;
function f() { const L = 3; return K + L + K + L; }
console.log(f());
console.log(K);
"#,
        "20\n7\n",
    );
}

// A `const` whose initializer is a pure expression over names that are NEVER
// reassigned stays on the fold lane and must remain correct.
#[test]
fn const_over_never_reassigned_names_stays_correct() {
    assert_stdout(
        r#"function f(x) {
  const a = 2;
  const b = a * 3;
  const c = b + a;
  return c + b + a;
}
console.log(f(0));
"#,
        "16\n",
    );
}

// A chain of `const`s where the ROOT of the chain reads a variable that is
// later reassigned: every link must be captured at its own declaration.
// Pre-fix, the whole chain re-expanded to a read of `a` at every use.
#[test]
fn const_chain_rooted_at_a_reassigned_variable_is_stable() {
    assert_stdout(
        r#"function f() {
  let a = 1;
  const p = a;
  const q = p + 1;
  a = 50;
  return p + " " + q;
}
console.log(f());
"#,
        "1 2\n",
    );
}

// A `const` bound inside a loop body must re-evaluate its initializer once per
// ITERATION and be read-stable within the iteration.
#[test]
fn const_in_loop_body_evaluates_once_per_iteration() {
    assert_stdout(
        r#"let n = 0;
function bump(i) { n = n + 1; return i * 2; }
function g() {
  let total = 0;
  for (let i = 0; i < 3; i = i + 1) {
    const v = bump(i);
    total = total + v + v;
  }
  return total;
}
console.log(g());
console.log(n);
"#,
        "12\n3\n",
    );
}

// The reassigned-name analysis is PROGRAM-wide, not per-function: a `const`
// whose initializer reads a name that some OTHER function reassigns must still
// be captured at its declaration. Here `f` itself contains no assignment to
// `a`, so a per-function analysis would call `const p = a` stable and keep it
// on the re-emitting fold lane — after `g()` runs, the read returns 99.
// node: `1`. Pre-fix kali: `99`.
#[test]
fn const_reading_a_name_reassigned_by_another_function_is_stable() {
    assert_stdout(
        r#"let a = 1;
function g() { a = 99; return 0; }
function f() {
  const p = a;
  g();
  return p;
}
console.log(f());
"#,
        "1\n",
    );
}

// A member read is a SNAPSHOT of whatever the property holds at the
// declaration. Host state reached through a member can be mutated by a METHOD
// CALL with no assignment to that property appearing anywhere in the program —
// `c.abort()` mutates `s.aborted`. An earlier form of the stability allowlist
// admitted any member read whose property was never an *assignment* target,
// which folded this read and re-evaluated it after the mutation: ONE `const`,
// TWO values, exit 0 and no diagnostic.
//
// node: `0 0`. Pre-narrowing kali: `0 1`. (Predates the allowlist — the
// unconditional fold lane had the same hole — so this is a hole the allowlist
// must CLOSE, not one it opened.) Writing `let before = s.aborted` was already
// correct, which isolates it to the fold lane.
#[test]
fn const_member_read_of_mutable_host_state_is_not_folded() {
    assert_stdout(
        r#"const c = new AbortController();
const s = c.signal;
const before = s.aborted;
console.log(before ? 1 : 0);
c.abort();
console.log(before ? 1 : 0);
"#,
        "0\n0\n",
    );
}

// The `Object.freeze(x)` identity arm is stable iff `x` is, so it must INHERIT
// the member-read narrowing rather than route around it. Pre-narrowing this
// printed `0 1` exactly like the bare member read above, confirming the arm
// propagates whatever the inner expression's stability rule allows.
#[test]
fn const_object_freeze_of_mutable_host_state_is_not_folded() {
    assert_stdout(
        r#"const c = new AbortController();
const s = c.signal;
const before = Object.freeze(s.aborted);
console.log(before ? 1 : 0);
c.abort();
console.log(before ? 1 : 0);
"#,
        "0\n0\n",
    );
}

// The guard for the member-read arm's whole justification: an alias off an
// INTRINSIC namespace must still resolve to its intrinsic. Narrowing the arm
// must not cost this — if `same` stops folding, the alias analyses can no
// longer see that it denotes `Object.is` and the call fails to lower.
#[test]
fn intrinsic_namespace_alias_still_resolves() {
    assert_stdout(
        r#"const same = Object.is;
const r1 = same(1, 1);
const r2 = Object.is(2, 2);
console.log(r1 ? 1 : 0);
console.log(r2 ? 1 : 0);
"#,
        "1\n1\n",
    );
}

// A function-valued `const` must keep working: a function-like initializer
// stays on the fold lane by design (promoting it produces no value and calls
// resolve through a phantom zero local — see
// `soundness_const_fold_side_effects::const_bound_arrow_with_mutating_body_is_not_promoted`).
#[test]
fn function_valued_const_still_calls() {
    assert_stdout(
        r#"const mk = (n) => { let r = n; r = r + 1; return r; };
console.log(mk(5));
console.log(mk(10));
"#,
        "6\n11\n",
    );
}

// A `const` bound to a string expression over a reassigned variable: the
// string lane must capture at declaration too (the value class is a handle,
// but the binding rule is the same).
#[test]
fn const_string_binding_is_read_stable() {
    assert_stdout(
        r#"function f() {
  let a = "x";
  const s = "v=" + a;
  a = "y";
  return s + "/" + s;
}
console.log(f());
"#,
        "v=x/v=x\n",
    );
}
