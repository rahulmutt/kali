//! Soundness pins for the strict/loose equality and nullish-coalescing
//! conflation closed in soundness batch 1 (Fix 4).
//!
//! kali stores every value in an untyped i64 slot. `0`, `false`, `null` and
//! `undefined` all lower to the bit pattern `0`, and `true` lowers to `1`, so
//! the generic `i64.eq` lowering of `===` reported `0 === null`, `0 === false`,
//! `null === undefined` and `1 === true` as `true` (node: `false` for all
//! four). That is wrong CONTROL FLOW, not just a wrong printed value: every
//! `if (x === null)` guard fired when `x` was `0`. The same raw-bit-pattern
//! test made `??` treat `0` and `false` as nullish (`0 ?? 9` → `9`).
//!
//! The fix classifies both operands into JS type classes at the comparison
//! site (`static_equality_class`) and decides `===`/`!==`/`==`/`!=` by TYPE
//! first. Where a `null` / `undefined` / boolean operand meets an operand
//! whose class cannot be proven, the comparison fails closed with `E5506`
//! rather than emitting a wrong boolean.
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
        "kali-soundness-strict-equality-{}-{}-{}",
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

fn assert_fails_closed(src: &str, needle: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success: {out:?}"
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
// The controller-verified repro.
// ---------------------------------------------------------------------------

#[test]
fn zero_is_not_strictly_equal_to_null_or_false() {
    assert_stdout(
        r#"function f() {
  const zero = 0;
  return "zero===null:" + (zero === null) + " zero===false:" + (zero === false) + " zero===0:" + (zero === 0);
}
console.log(f());
"#,
        "zero===null:false zero===false:false zero===0:true\n",
    );
}

// ---------------------------------------------------------------------------
// Full literal truth table across the eight primitive literals kali supports.
// ---------------------------------------------------------------------------

/// The eight literal source forms, paired with the JS type class node assigns
/// them. Two literals are `===` iff their classes match AND their values match
/// (with `null === null` and `undefined === undefined` both `true`).
const LITERALS: [(&str, &str); 8] = [
    ("0", "number"),
    ("1", "number"),
    ("\"\"", "string"),
    ("\"a\"", "string"),
    ("null", "null"),
    ("undefined", "undefined"),
    ("true", "boolean"),
    ("false", "boolean"),
];

fn strictly_equal(left: usize, right: usize) -> bool {
    let (lt, lc) = LITERALS[left];
    let (rt, rc) = LITERALS[right];
    lc == rc && lt == rt
}

/// `"` is not legal inside the single-quoted label strings this test emits, so
/// labels render `"` as `Q`.
fn label(text: &str) -> String {
    text.replace('"', "Q")
}

#[test]
fn strict_equality_literal_truth_table_matches_node() {
    let mut src = String::new();
    let mut expected = String::new();
    for left in 0..LITERALS.len() {
        for right in 0..LITERALS.len() {
            let (lt, _) = LITERALS[left];
            let (rt, _) = LITERALS[right];
            let eq_label = format!("{}==={}", label(lt), label(rt));
            let ne_label = format!("{}!=={}", label(lt), label(rt));
            src.push_str(&format!(
                "console.log('{eq_label}:' + ({lt} === {rt}) + ' {ne_label}:' + ({lt} !== {rt}));\n"
            ));
            let eq = strictly_equal(left, right);
            expected.push_str(&format!("{eq_label}:{eq} {ne_label}:{}\n", !eq));
        }
    }
    assert_stdout(&src, &expected);
}

#[test]
fn strict_equality_through_const_bindings_matches_node() {
    assert_stdout(
        r#"const zero = 0;
const one = 1;
const nul = null;
const undef = undefined;
const yes = true;
const no = false;
console.log("a:" + (zero === nul) + " b:" + (zero === no) + " c:" + (one === yes));
console.log("d:" + (nul === undef) + " e:" + (nul === nul) + " f:" + (yes === no));
console.log("g:" + (zero !== nul) + " h:" + (undef !== undef) + " i:" + (no === no));
"#,
        "a:false b:false c:false\nd:false e:true f:false\ng:true h:false i:true\n",
    );
}

// ---------------------------------------------------------------------------
// Control flow, not just the printed form.
// ---------------------------------------------------------------------------

#[test]
fn null_guard_does_not_fire_for_zero() {
    assert_stdout(
        r#"const z = 0;
if (z === null) { console.log("BAD-then"); } else { console.log("GOOD-else"); }
if (z === undefined) { console.log("BAD-then2"); } else { console.log("GOOD-else2"); }
if (z === false) { console.log("BAD-then3"); } else { console.log("GOOD-else3"); }
if (z !== null) { console.log("GOOD-then4"); } else { console.log("BAD-else4"); }
"#,
        "GOOD-else\nGOOD-else2\nGOOD-else3\nGOOD-then4\n",
    );
}

#[test]
fn one_is_not_true_in_a_guard() {
    assert_stdout(
        r#"const one = 1;
if (one === true) { console.log("BAD"); } else { console.log("GOOD"); }
const yes = true;
if (yes === true) { console.log("GOOD2"); } else { console.log("BAD2"); }
"#,
        "GOOD\nGOOD2\n",
    );
}

// ---------------------------------------------------------------------------
// The unary forms that produce a provable class (`!x` is boolean, `void x` is
// undefined) must keep comparing correctly — these are pinned elsewhere in the
// suite and must not regress into the fail-closed lane.
// ---------------------------------------------------------------------------

#[test]
fn unary_derived_classes_compare_correctly() {
    assert_stdout(
        r#"const notTrue = !true;
console.log("a:" + (notTrue === false) + " b:" + (notTrue !== false) + " c:" + (notTrue === 0));
const v = void (1 + 2);
console.log("d:" + (v === undefined) + " e:" + (v === null) + " f:" + (v === 0));
"#,
        "a:true b:false c:false\nd:true e:false f:false\n",
    );
}

// ---------------------------------------------------------------------------
// Fail-closed lane: a `null` / `undefined` / boolean operand against an
// operand whose type class kali cannot prove.
// ---------------------------------------------------------------------------

#[test]
fn unprovable_operand_against_null_fails_closed() {
    assert_fails_closed(
        r#"function id(n) { return n; }
const x = id(0);
if (x === null) { console.log("then"); } else { console.log("else"); }
"#,
        "'==='",
    );
}

/// RESIDUAL, pinned honestly. An unprovable operand against a BOOLEAN literal
/// keeps the pre-existing `i64.eq` lowering rather than failing closed: kali's
/// boolean repr IS the integers 0/1, so that compare is correct for every
/// genuinely boolean-producing operand (`Object.is(a, b) !== true`,
/// `delete o.k !== true`, and 33 other pinned corpus programs), and kali cannot
/// prove "this call returns a boolean" for a user function without a
/// `Repr::Boolean` axis. The cost is this case: a function returning the NUMBER
/// `1` still compares equal to `true`. node prints `false`.
#[test]
fn unprovable_operand_against_boolean_is_a_known_residual() {
    assert_stdout(
        r#"function id(n) { return n; }
const x = id(1);
console.log("kali:" + (x === true));
"#,
        "kali:true\n",
    );
}

/// RESIDUAL, pinned honestly — records CURRENT (WRONG) behaviour, not a
/// correctness claim. This is CRITICAL-2 from the semantic-core whole-stage
/// review: an unprovable operand against a proven NUMBER (including a number
/// LITERAL like `0`) never arms the type-directed decision table at all,
/// because `EqClass::arms_the_gate` only recognizes `null`/`undefined`/
/// boolean. The pair falls straight through to the pre-existing bit-pattern
/// `i64.eq`, which is unsound because `false` also lowers to the bit pattern
/// `0`. Unlike the boolean residual above (which at least reaches the
/// decision table and is a deliberate, reasoned trade-off), this case never
/// engages the fix at all.
///
/// node prints `222` (the `else` branch: `false !== 0`); kali prints `111`
/// (the `then` branch), i.e. WRONG CONTROL FLOW, not merely a wrong printed
/// value, and exits 0 with no diagnostic. When the real fix (a
/// `Repr::Boolean` axis, out of scope for soundness-batch1-pra) lands, this
/// assertion must go RED — that is the intended signal to update this pin.
#[test]
fn unprovable_operand_against_number_literal_is_a_known_residual() {
    assert_stdout(
        r#"function f(b) { return b; }
if (f(false) === 0) { console.log(111); } else { console.log(222); }
"#,
        "111\n",
    );
}

// ---------------------------------------------------------------------------
// Object-reference-vs-`null` stays on the runtime pointer path: a live
// fixed-shape object pointer is a nonzero heap address and `null` is `0`, so
// `i64.eq` is the CORRECT test. This is the binary-trees `t.left === null`
// shape and must not regress into the fail-closed lane.
// ---------------------------------------------------------------------------

#[test]
fn object_field_against_null_still_compares_at_runtime() {
    assert_stdout(
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
console.log("leaf:" + itemCheck(bottomUpTree(0)));
console.log("depth2:" + itemCheck(bottomUpTree(2)));
"#,
        "leaf:1\ndepth2:7\n",
    );
}

// ---------------------------------------------------------------------------
// Loose equality.
// ---------------------------------------------------------------------------

#[test]
fn loose_equality_literals_match_node() {
    assert_stdout(
        r#"console.log("l1:" + (0 == null));
console.log("l2:" + (0 == false));
console.log("l3:" + (null == undefined));
console.log("l4:" + (1 == true));
console.log("l5:" + (0 != null));
console.log("l6:" + (null != undefined));
console.log("l7:" + (null == 0));
console.log("l8:" + (undefined == false));
"#,
        "l1:false\nl2:true\nl3:true\nl4:true\nl5:true\nl6:false\nl7:false\nl8:false\n",
    );
}

// ---------------------------------------------------------------------------
// `??` nullish tests: only `null` / `undefined` are nullish. `0`, `""` and
// `false` are NOT.
// ---------------------------------------------------------------------------

#[test]
fn nullish_coalescing_does_not_treat_falsy_as_nullish() {
    assert_stdout(
        r#"console.log("n1:" + (0 ?? 9));
console.log("n2:[" + ("" ?? "x") + "]");
console.log("n3:" + (false ?? true));
console.log("n4:" + (null ?? 7));
console.log("n5:" + (undefined ?? 8));
"#,
        "n1:0\nn2:[]\nn3:false\nn4:7\nn5:8\n",
    );
}

#[test]
fn nullish_coalescing_through_const_bindings() {
    assert_stdout(
        r#"const zero = 0;
const no = false;
const nul = null;
console.log("a:" + (zero ?? 9) + " b:" + (no ?? 9) + " c:" + (nul ?? 9));
"#,
        "a:0 b:false c:9\n",
    );
}

// ---------------------------------------------------------------------------
// RESIDUAL, pinned honestly — these four record CURRENT (WRONG) behaviour,
// not a correctness claim.
//
// THE MECHANISM (the actual proof condition, not a list of shapes): `??`
// decides its branch at compile time, correctly, if and only if
// `static_equality_class` (`crates/kali_codegen/src/emit/equality.rs:228`)
// returns `Some(class)` for the left operand. That happens ONLY when the
// left operand is (a) a literal (or one of the unary forms
// `static_equality_class` folds directly to a class: `void`, `!`, `typeof`,
// `delete`, a numeric `-`/`~`), or (b) an identifier whose ENTIRE initializer
// chain resolves, at compile time, all the way down to such a literal via
// `resolve_literal_aggregate`/`self.bindings` (the `const`-alias chain).
// Every other operand — anything actually read back from a runtime storage
// slot with `LocalGet`, regardless of which keyword bound it — returns `None`
// and falls through to the pre-existing `i64.eqz` bit-pattern test, which
// conflates a runtime `0`/`false` with nullish (`??` degrades to `||`).
//
// The four shapes pinned below (`let`, `var`, a function parameter, a call's
// return value) are ILLUSTRATIONS of that rule, not the boundary itself —
// see the next section for further illustrations (a `const` whose
// initializer chain does NOT bottom out at a literal) that are equally
// uncovered for the identical reason. When the real fix (the
// `Repr::Boolean`/null axis) lands, every assertion in both sections must go
// RED — that is the intended signal to update these pins.
// ---------------------------------------------------------------------------

#[test]
fn nullish_coalescing_over_let_binding_is_a_known_residual() {
    assert_stdout(
        r#"let a = 0;
console.log(a ?? 9);
"#,
        "9\n",
    );
}

#[test]
fn nullish_coalescing_over_var_binding_is_a_known_residual() {
    assert_stdout(
        r#"var v = 0;
console.log(v ?? 9);
"#,
        "9\n",
    );
}

#[test]
fn nullish_coalescing_over_parameter_is_a_known_residual() {
    assert_stdout(
        r#"function opt(n) { return n ?? 10; }
console.log(opt(0));
"#,
        "10\n",
    );
}

#[test]
fn nullish_coalescing_over_call_return_is_a_known_residual() {
    assert_stdout(
        r#"function zero() { return 0; }
console.log(zero() ?? 9);
"#,
        "9\n",
    );
}

// ---------------------------------------------------------------------------
// RESIDUAL, pinned honestly — FAMILY (a) of the same mechanism above, one
// level indirect: a `const` binding whose initializer chain does NOT bottom
// out at a compile-time literal. `resolve_literal_aggregate` will follow a
// `const`'s binding, but if what it finds at the end of the chain is a call,
// a folded runtime expression, another (non-`const`) binding, or an object
// field read, `static_equality_class` still returns `None` there — the
// `const` keyword itself proves nothing; only a chain that terminates in a
// literal does. This falsifies a claim from an earlier round of this
// register entry ("`??` is closed for a literal or a `const`-bound operand"),
// which conflated "bound via `const`" with "provably a literal". Four
// illustrations, all re-verified on a freshly built binary, all WRONG (kali
// `9`, node `0`):
// ---------------------------------------------------------------------------

#[test]
fn nullish_coalescing_over_const_bound_call_result_is_a_known_residual() {
    assert_stdout(
        r#"function zero() { return 0; }
const c1 = zero();
console.log(c1 ?? 9);
"#,
        "9\n",
    );
}

#[test]
fn nullish_coalescing_over_const_bound_folded_expression_is_a_known_residual() {
    assert_stdout(
        r#"const c2 = 1 - 1;
console.log(c2 ?? 9);
"#,
        "9\n",
    );
}

#[test]
fn nullish_coalescing_over_const_bound_let_alias_is_a_known_residual() {
    assert_stdout(
        r#"let d = 0;
const c3 = d;
console.log(c3 ?? 9);
"#,
        "9\n",
    );
}

#[test]
fn nullish_coalescing_over_const_bound_member_read_is_a_known_residual() {
    assert_stdout(
        r#"const o = { a: 0 };
console.log(o.a ?? 9);
"#,
        "9\n",
    );
}

// ---------------------------------------------------------------------------
// RESIDUAL, pinned honestly — FAMILY (b): independent of the mechanism above,
// a `??` whose selected result is a BOOLEAN loses its boolean-ness — for
// every binding kind, including a bare literal operand, and even when `??`'s
// branch selection is itself correct (a bare `false`/`true`/`null` operand
// IS provable, so this fires on top of a correct decision, not because of
// the residual above).
//
// Mechanism: when `??`'s left operand is provably `Boolean`-classed (never
// nullish) or provably `Null`/`Undefined`-classed (always nullish), the
// selected operand's `EmittedValue` correctly carries `shape:
// ValueShape::Boolean` (via `selected_nullish_operand`,
// `crates/kali_codegen/src/emit/equality.rs:433-436`). But the SINGLE-
// ARGUMENT `console.log`/`.error`/`.warn`/`.info` sink
// (`emit_console_argument`, `crates/kali_codegen/src/emit/call.rs:23-41`) —
// which is what a `??` expression falls to whenever the WHOLE call isn't
// statically renderable — never inspects `shape` except for `Float`; it
// hands the raw i64 to the host, which does `value.to_string()` for anything
// that isn't a string handle. A bare `console.log(false)` prints correctly
// only because the ENTIRE call is folded to the literal string "false" by a
// SEPARATE, independent constant-folder (`render_console_call` /
// `render_static_value`, `crates/kali_codegen/src/intrinsics/host.rs:345-`),
// which has no case for a `??`/binary node and so never folds a `??`
// expression — a fresh instance of this repo's "hand-mirrored oracle" class
// of bug (two independent notions of "is this a boolean", one used by `??`'s
// own branch decision, one used by console's static-fold decision, and they
// disagree). The multi-argument console lane (`emit_console_argument_as_string`
// / `emit_as_string`) DOES honor `shape: Boolean` and is NOT affected — see
// the passing (non-residual) coverage this section deliberately omits.
//
// Do not route these pins through string concatenation (`"n:" + (false ?? 9)`)
// — that takes `emit_as_string`'s correct path (per the existing
// `nullish_coalescing_does_not_treat_falsy_as_nullish` test above) and would
// mask this residual entirely, which is exactly what happened to a green
// suite in an earlier round.
// ---------------------------------------------------------------------------

#[test]
fn nullish_coalescing_boolean_literal_result_loses_shape_is_a_known_residual() {
    assert_stdout(
        r#"console.log(false ?? 9);
"#,
        "0\n",
    );
}

#[test]
fn nullish_coalescing_right_operand_boolean_loses_shape_is_a_known_residual() {
    assert_stdout(
        r#"console.log(null ?? false);
"#,
        "0\n",
    );
}
