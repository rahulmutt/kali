use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-multiarr-{}-{}-{}",
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

// `Array(1,2,3)` is exactly `[1,2,3]` in JS (n>=2 args). Today types
// registers the binding as an array but codegen can't allocate it → 0.
#[test]
fn multiarg_array_call_is_array_literal() {
    let out = run_source("const a = Array(1, 2, 3);\nconsole.log(a.length);\nconsole.log(a[1]);\n");
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.trim().split('\n').collect::<Vec<_>>(),
        vec!["3", "2"]
    );
}

#[test]
fn multiarg_new_array_call_is_array_literal() {
    let out = run_source("const a = new Array(4, 5);\nconsole.log(a.length + a[0]);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "6");
}

// Single-arg `new Array(n)` is a LENGTH, not an element — must stay on the
// existing allocation lane, NOT desugar.
#[test]
fn single_arg_array_still_allocates_by_length() {
    let out = run_source("const a = new Array(5);\nconsole.log(a.length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}

// `new [1,2,3]` is not constructible (node: TypeError). The New-arm unwrap
// must be scoped to the `Array(...)` desugar only — a bracket-literal callee
// rejects fail-closed, never silently drops the `new`.
#[test]
fn new_on_a_bracket_array_literal_rejects_fail_closed() {
    let out = run_source("const a = new [1, 2, 3];\nconsole.log(a.length);\n");
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not constructible"),
        "{out:?}"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "");
}

// Postfix chaining directly off the desugared literal stays green:
// `Array(7,8,9).length` is `[7,8,9].length`.
#[test]
fn multiarg_array_call_supports_postfix_member_chain() {
    let out = run_source("console.log(Array(7, 8, 9).length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

// Whole-branch review finding I1: the `Array(a, b, …)` desugar is scope-blind.
// A user who SHADOWS `Array` (`const Array = (a,b,c)=>a+b+c`) must NOT get the
// silent array-literal miscompile — node runs the user binding and prints 6,
// this branch used to print 3. Reject-don't-miscompile: fail closed with a
// clean E5506 diagnostic (a shadow of the built-in `Array` is unavailable),
// NEVER a silent wrong output. The critical invariant is that stdout is not the
// wrong value `3`.
#[test]
fn shadowed_multiarg_array_call_rejects_not_miscompiles() {
    let out = run_source("const Array = (a, b, c) => a + b + c;\nconsole.log(Array(1, 2, 3));\n");
    // MUST NOT silently print the miscompiled array-literal result `3`.
    assert_ne!(String::from_utf8_lossy(&out.stdout).trim(), "3", "{out:?}");
    // Reject cleanly (option 2, fail-closed) — no wrong output.
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("shadowing the built-in `Array`"),
        "{out:?}"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "");
}

// Function-local shadow: `function f(){ const Array=(a,b)=>a*b; return Array(3,4); }`
// → node 12, this branch used to print 0. Same reject-don't-miscompile guard.
#[test]
fn shadowed_fn_local_array_call_rejects_not_miscompiles() {
    let out = run_source(
        "function f() {\n  const Array = (a, b) => a * b;\n  return Array(3, 4);\n}\nconsole.log(f());\n",
    );
    assert_ne!(String::from_utf8_lossy(&out.stdout).trim(), "0", "{out:?}");
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("shadowing the built-in `Array`"),
        "{out:?}"
    );
}

// The single-arg `Array(n)` allocation lane is ALSO scope-blind (pre-existing,
// separate from Task 6's multiarg desugar). A shadow used to miscompile to a
// raw heap pointer; the binding-site reject closes it too, fail-closed.
#[test]
fn shadowed_single_arg_array_call_rejects_not_miscompiles() {
    let out = run_source("const Array = (n) => n * 10;\nconst a = Array(5);\nconsole.log(a);\n");
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("shadowing the built-in `Array`"),
        "{out:?}"
    );
}
