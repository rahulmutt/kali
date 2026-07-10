//! Soundness pins for the binary `in` / `instanceof` operators.
//!
//! Neither token was a binary operator in the parser, so `'a' in obj` parsed
//! as the bare expression `'a'` and the trailing `in obj` tokens were
//! silently dropped — the whole expression miscompiled to its LEFT operand
//! (kali printed `a` where node prints `true`). Per the reject-don't-
//! miscompile rule both operators are now rejected fail-closed with a
//! feature-unavailable diagnostic: kali's static object model cannot decide
//! runtime key presence (e.g. after `delete`), and there is no prototype
//! chain for `instanceof` to walk.
//!
//! The `for (expr in obj)` / `for (var c in obj)` statement heads are NOT
//! binary `in` and must keep working — pinned here too.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-in-operator-{}-{}-{}",
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

// node prints `true`; kali used to print `a` (left operand) with exit 0.
// Until a sound presence proof exists, the only correct behaviors are
// "true" or a compile-time reject — never the left operand.
#[test]
fn binary_in_is_rejected_not_left_operand() {
    let out = run_source("const obj = { a: 1, b: 2 };\nconst r = 'a' in obj;\nconsole.log(r);\n");
    assert!(!out.status.success(), "must not compile: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("`in` operator"), "stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains('a'),
        "left-operand miscompile resurfaced: {stdout}"
    );
}

// `in` buried in a condition — the lane the runtime_smoke self-checks used;
// previously the condition evaluated to the (truthy/falsy) left operand.
#[test]
fn binary_in_inside_condition_is_rejected() {
    let out = run_source(
        "const obj = { a: 1 };\nif (!('a' in obj)) {\n  console.log('missing');\n}\nconsole.log('done');\n",
    );
    assert!(!out.status.success(), "must not compile: {out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("E5506"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("done"));
}

// node prints `true`; kali used to evaluate the expression to `obj`.
#[test]
fn instanceof_is_rejected_fail_closed() {
    let out = run_source(
        "const obj = { a: 1 };\nif (obj instanceof Object) {\n  console.log('yes');\n}\n",
    );
    assert!(!out.status.success(), "must not compile: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("instanceof"), "stderr: {stderr}");
}

// The for-in statement head is not binary `in`; the supported for-in lane
// (declaration form + bare pre-declared form, per the fasta Spec 4a surface)
// must keep compiling and match node byte-for-byte.
#[test]
fn for_in_statement_heads_still_work() {
    let out = run_source(
        r#"function makeCumulative(table) {
  var last = null;
  for (var c in table) {
    if (last) table[c] += table[last];
    last = c;
  }
}
function firstKey(table) {
  var c;
  for (c in table) return c;
  return c;
}
var obj = { a: 0.5, b: 0.25, c: 0.25 };
makeCumulative(obj);
console.log(obj.c);
console.log(firstKey(obj));
"#,
    );
    assert!(out.status.success(), "for-in must stay green: {out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\na\n");
}
