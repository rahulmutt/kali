//! Soundness pins for the call-through-a-first-class-function-value family
//! closed in soundness batch 1 (Fix 5).
//!
//! kali's closure lane resolves a callee by NAME: `emit_call` looks
//! `callee_node.text` up in the emitter's compiled-function map. When the
//! callee is a *value* rather than a statically-resolvable name — a returned
//! closure, an alias (`var g = fn`), a callback parameter, a reassigned
//! function binding, a method held in an object field, or an array-callback
//! argument — that lookup misses and the call used to fall through the
//! terminal "undefined call target" fallback in `emit_call`, which pushed a
//! WARNING and an `i64.const 0` placeholder. The program exited 0, the callee
//! never ran, and the call site evaluated to `0`:
//!
//! ```js
//! function make() { let n = 0; return function () { n = n + 1; return n; }; }
//! const c = make();
//! console.log("returned=" + c());   // node: returned=1   kali: returned=0
//! ```
//!
//! That escalates past a wrong printed value into wrong control flow
//! (`if (g())` took the else branch) and silently skipped side effects.
//!
//! kali has no first-class function representation to lower these onto: there
//! is no `Repr` variant for a closure value, and codegen emits no WASM table,
//! element section or `call_indirect` anywhere. Real support is an
//! architectural change (uniform calling convention across a per-function repr
//! table, escaping environment records, interaction with monomorphization and
//! arena reclamation), so the honest target here is REJECT-DON'T-MISCOMPILE:
//! every call whose callee cannot be proven to be a compiled function fails
//! closed with `E5506`.
//!
//! The decision is made at ONE choke point — the terminal fallback of
//! `emit_call`, which every unresolved callee shape already converged on —
//! with an allowlist of the callee shapes that keep the pre-existing
//! non-invoking lowering. Shapes are not enumerated for denial, so a new
//! call-through-a-value spelling is denied by construction rather than needing
//! its own arm (the `forEach`/`filter`/`map` inconsistency below is what the
//! old per-method denylist produced).
//!
//! Every expected value in this file was captured from node v26.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-first-class-calls-{}-{}-{}",
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

/// A call-through-a-value must fail closed with `E5506` — never exit 0 having
/// silently evaluated the call to `0` without running the callee.
fn assert_fails_closed(src: &str, needle: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success with stdout {stdout:?}: {out:?}"
    );
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    assert!(
        stderr.contains(needle),
        "expected {needle:?} in stderr, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// The controller-verified repro: a returned closure.
// ---------------------------------------------------------------------------

#[test]
fn calling_a_returned_closure_fails_closed() {
    // node: returned=1. Pre-fix kali: returned=0 at exit 0, callee never ran.
    assert_fails_closed(
        r#"function make() {
  let n = 0;
  return function () { n = n + 1; return n; };
}
const c = make();
console.log("returned=" + c());
"#,
        "first-class function value",
    );
}

// ---------------------------------------------------------------------------
// The rest of the family. Each was `0` at exit 0 before the fix.
// ---------------------------------------------------------------------------

#[test]
fn calling_a_function_alias_fails_closed() {
    // node: alias=7.
    assert_fails_closed(
        r#"function fn() { return 7; }
var g = fn;
console.log("alias=" + g());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_callback_parameter_fails_closed() {
    // node: param=7.
    assert_fails_closed(
        r#"function fn() { return 7; }
function apply(cb) { return cb(); }
console.log("param=" + apply(fn));
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_reassigned_function_binding_fails_closed() {
    // node: reassigned=2.
    assert_fails_closed(
        r#"function a() { return 1; }
function b() { return 2; }
var h = a;
h = b;
console.log("reassigned=" + h());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_function_held_in_an_object_field_fails_closed() {
    // node: field=42. Pre-fix kali: field=0 at exit 0, while the sibling
    // class-method `this` shape already failed closed — the same inconsistency
    // the single choke point removes.
    assert_fails_closed(
        r#"const o = { v: 3, f: function () { return 42; } };
console.log("field=" + o.f());
"#,
        "first-class function value",
    );
}

// ---------------------------------------------------------------------------
// A silently-skipped call is worse than a wrong value: it takes the wrong
// branch. Probe the side effect with a module-scope SCALAR counter (a
// growable array escaping into a call is itself a no-op here and would
// measure nothing).
// ---------------------------------------------------------------------------

#[test]
fn a_call_through_a_value_never_silently_skips_its_side_effect() {
    // node: hits=1 taken. Pre-fix kali: `g()` was `0`, so the `else` branch
    // ran and `bump` never executed — wrong control flow AND a dropped effect.
    assert_fails_closed(
        r#"let hits = 0;
function bump() { hits = hits + 1; return 1; }
var g = bump;
if (g()) {
  console.log("hits=" + hits + " taken");
} else {
  console.log("hits=" + hits + " not-taken");
}
"#,
        "first-class function value",
    );
}

#[test]
fn calling_an_array_element_directly_fails_closed() {
    // node: 9. The sibling SPELLING of the re-pinned `let g = arr[0]; g()`
    // tripwire in soundness_closures.rs — it reached the choke point through
    // the computed-element path, where the property-name rule cannot see it,
    // and printed `0` at exit 0 until the aggregate rule closed it. Pinned so
    // the two spellings stay consistent.
    assert_fails_closed(
        r#"function outer() {
  let c = 9;
  let arr = [function () { return c; }];
  console.log(arr[0]());
}
outer();
"#,
        "E5506",
    );
}

#[test]
fn calling_a_const_bound_arrow_by_name_fails_closed() {
    // node: lay=5. Pre-fix kali: lay=0 at exit 0 — the whole
    // `nested-wrapper-pruning` benchmark fixture had been compiling to a
    // program whose layers never ran. See the report's follow-up: this shape
    // IS statically resolvable and is the best candidate for real support.
    assert_fails_closed(
        r#"function bench() {
  const layer0 = (x) => x + 0;
  const layer1 = (x) => layer0(x);
  return layer1(5);
}
console.log("lay=" + bench());
"#,
        "first-class function value",
    );
}

// ---------------------------------------------------------------------------
// Sibling SPELLINGS. Each of these reached the choke point by a different
// route and was silently `0` until the shared "does this expression denote a
// program-defined function" predicate closed them together. They are pinned so
// the family cannot regrow holes one spelling at a time.
// ---------------------------------------------------------------------------

#[test]
fn calling_a_function_named_by_an_object_property_fails_closed() {
    // node: a=7. The property holds an IDENTIFIER naming a function, not an
    // inline function expression — the shape that slipped the first rule.
    assert_fails_closed(
        r#"function f() { return 7; }
const o = { g: f };
console.log("a=" + o.g());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_function_named_by_an_array_element_fails_closed() {
    // node: b=7.
    assert_fails_closed(
        r#"function f() { return 7; }
const arr = [f];
console.log("b=" + arr[0]());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_ternary_selected_function_fails_closed() {
    // node: c=7.
    assert_fails_closed(
        r#"function f() { return 7; }
console.log("c=" + (true ? f : f)());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_call_result_fails_closed() {
    // node: e=3. A callee that is itself a call.
    assert_fails_closed(
        r#"function make() { return function () { return 3; }; }
console.log("e=" + make()());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_function_returned_from_a_callback_taking_helper_fails_closed() {
    // node: j=7.
    assert_fails_closed(
        r#"function f() { return 7; }
function g(cb) { return cb; }
console.log("j=" + g(f)());
"#,
        "first-class function value",
    );
}

#[test]
fn calling_a_nested_object_property_function_fails_closed() {
    // node: k=7.
    assert_fails_closed(
        r#"function f() { return 7; }
const o = { n: { p: f } };
console.log("k=" + o.n.p());
"#,
        "first-class function value",
    );
}

#[test]
fn a_parenthesized_direct_call_still_runs() {
    // node: d=7. `(f)()` is still a statically-resolvable direct call and must
    // NOT be swept up by the family deny.
    assert_stdout(
        r#"function f() { return 7; }
console.log("d=" + (f)());
"#,
        "d=7\n",
    );
}

// ---------------------------------------------------------------------------
// Array-callback consistency. Pre-fix, `map` failed closed with E5506 while
// `forEach` (absent from the gated method list entirely) and a predicate
// `filter` (admitted by the kali_types allowlist but never lowered by codegen)
// silently no-opped at exit 0. All three must now agree.
// ---------------------------------------------------------------------------

#[test]
fn for_each_with_a_callback_fails_closed() {
    // node: forEach=6.
    assert_fails_closed(
        r#"let hits = 0;
const arr = [1, 2, 3];
arr.forEach(function (x) { hits = hits + x; });
console.log("forEach=" + hits);
"#,
        "E5506",
    );
}

#[test]
fn filter_with_a_predicate_callback_fails_closed() {
    // node: filter=2.
    assert_fails_closed(
        r#"const arr = [1, 2, 3];
const r = arr.filter(x => x > 1);
console.log("filter=" + r.length);
"#,
        "E5506",
    );
}

#[test]
fn map_with_a_callback_still_fails_closed() {
    // The one member of the family that already failed closed; pinned so the
    // three stay consistent by construction.
    assert_fails_closed(
        r#"const arr = [1, 2, 3];
const r = arr.map(x => x * 2);
console.log("map=" + r.length);
"#,
        "E5506",
    );
}

// ---------------------------------------------------------------------------
// Regression guard: the shipped closure lane. A callee that IS statically
// resolvable by name keeps running, captures included.
// ---------------------------------------------------------------------------

#[test]
fn direct_sibling_capture_still_runs() {
    // node: captured=2. This is the repo's shipped env-pointer closure lane
    // and must not regress: the callee is resolved by NAME, so it is on the
    // allowlist at the choke point.
    assert_stdout(
        r#"function direct() {
  let n = 0;
  function inc() { n = n + 1; }
  inc();
  inc();
  return "captured=" + n;
}
console.log(direct());
"#,
        "captured=2\n",
    );
}

#[test]
fn a_directly_named_call_still_runs() {
    assert_stdout(
        r#"function add(a, b) { return a + b; }
console.log("sum=" + add(2, 3));
"#,
        "sum=5\n",
    );
}
