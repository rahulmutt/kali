//! Soundness pins for two fail-open lanes closed in soundness batch 1:
//!
//! 1. `const x = <mutating init>` double-evaluation: the `const` fold lane
//!    re-emits the bound init node at every read, which re-applies the side
//!    effect. `let b = 5; const x = ++b;` yielded `x == 7, b == 7` (node:
//!    `6, 6`); `const y = (b = b + 1)` likewise applied twice. Fixed by
//!    promoting any `const` whose init subtree contains a mutating operator
//!    to an eager local slot (`declarator_init_contains_mutation`).
//!
//! 2. `typeof` was not parsed as a unary operator at all: `typeof value`
//!    parsed as the bare identifier `typeof` (zero placeholder) and dropped
//!    `value`, so `typeof v !== 'undefined'` was always true. Now parsed as a
//!    real unary expression with a provable static lane (literals, `void`
//!    expressions, proven floats); unproven operands keep the pre-existing
//!    placeholder fallback.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-const-fold-{}-{}-{}",
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

// Golden outputs verified against node v26.
#[test]
fn const_bound_update_expression_applies_exactly_once() {
    assert_stdout(
        "let b = 5;\nconst x = ++b;\nconsole.log(x);\nconsole.log(b);\n",
        "6\n6\n",
    );
    assert_stdout(
        "let c = 5;\nconst y = c++;\nconsole.log(y);\nconsole.log(c);\n",
        "5\n6\n",
    );
    assert_stdout(
        "let d = 5;\nconst z = --d;\nconsole.log(z);\nconsole.log(d);\n",
        "4\n4\n",
    );
    assert_stdout(
        "let e = 5;\nconst w = e--;\nconsole.log(w);\nconsole.log(e);\n",
        "5\n4\n",
    );
}

#[test]
fn const_bound_assignment_expression_applies_exactly_once() {
    assert_stdout(
        "let b = 5;\nconst x = (b = b + 1);\nconsole.log(x);\nconsole.log(b);\n",
        "6\n6\n",
    );
}

// The whole unary/update ladder the runtime_smoke unary_prefix fixture pins,
// as one program — must run clean end to end.
#[test]
fn unary_prefix_fixture_semantics_hold() {
    assert_stdout(
        r#"const notTrue = !true;
if (notTrue !== false) { throw new Error('neg'); }
const bitwiseNot = ~1;
if (bitwiseNot !== -2) { throw new Error('bitnot'); }
let counter = 1;
const prefix = ++counter;
if (prefix !== 2 || counter !== 2) { throw new Error('prefix'); }
const postfix = counter--;
if (postfix !== 2 || counter !== 1) { throw new Error('postfix'); }
const value = void (1 + 2);
if (value !== void 0) { throw new Error('void'); }
if (typeof value !== 'undefined') { throw new Error('typeof'); }
console.log("ok");
"#,
        "ok\n",
    );
}

#[test]
fn typeof_provable_lane_matches_node() {
    assert_stdout(
        "console.log(typeof 42);\nconsole.log(typeof \"hi\");\nconsole.log(typeof true);\nconsole.log(typeof null);\nconsole.log(typeof undefined);\nconst v = void 0;\nconsole.log(typeof v);\n",
        "number\nstring\nboolean\nobject\nundefined\nundefined\n",
    );
}

// `typeof f()` must still evaluate f exactly once (JS evaluates the operand).
#[test]
fn typeof_direct_call_operand_still_evaluates() {
    assert_stdout(
        "let n = 0;\nfunction bump() { n = n + 1; return 1.5; }\nconsole.log(typeof bump());\nconsole.log(n);\n",
        "number\n1\n",
    );
}

// Reject pin: an array literal passed to a user function used to hand the
// callee a zero placeholder (g([1, 2]) read 0 for every element; node says
// the elements). Now rejected fail-closed.
#[test]
fn array_literal_argument_is_rejected_fail_closed() {
    let out = run_source(
        "function g(items, value) { return items[0] + items[1] + value; }\nconsole.log(g([1, 2], 1));\n",
    );
    assert!(!out.status.success(), "must not compile: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506") && stderr.contains("array literal"),
        "stderr: {stderr}"
    );
}

// Materialized arrays (new Array + element stores) must keep working when
// passed as arguments — the reject is scoped to fold-lane literals.
#[test]
fn materialized_array_argument_still_works() {
    assert_stdout(
        "function g(items, value) { return items[0] + items[1] + value; }\nconst arr = new Array(2);\narr[0] = 1;\narr[1] = 2;\nconsole.log(g(arr, 1));\n",
        "4\n",
    );
}
