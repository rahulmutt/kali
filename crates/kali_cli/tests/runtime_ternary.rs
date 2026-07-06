use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // A per-process AtomicU64 counter makes the slug unique even when two
    // sources share a length (sharing a length previously collided the dir and
    // caused macOS CI temp-slug flakes — repo convention is a counter).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-ternary-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

#[test]
fn int_ternary_selects_branch() {
    let out =
        run_source("let a = 1;\nconsole.log(a > 0 ? 10 : 20);\nconsole.log(a < 0 ? 10 : 20);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n20\n");
}

#[test]
fn float_ternary_selects_and_prints_float() {
    let out = run_source("let a = 1;\nlet x = a > 0 ? 1.5 : 2.5;\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.5\n");
}

#[test]
fn mixed_int_float_arms_promote_to_float() {
    let out = run_source("let a = 0;\nlet x = a > 0 ? 1.5 : 2;\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn string_arms_ternary_prints() {
    let out =
        run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"1\" : s + \"2\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x1\n");
}

#[test]
fn only_taken_arm_evaluates() {
    // Laziness pin: the untaken arm's side effect must not run.
    //
    // ADAPTED from the brief's module-`let` mutation form (`let n = 0; function
    // inc() { n = n + 1; ... }`). That form is rejected by a PRE-EXISTING E5506
    // gate on module-binding read/write from a function — it fails to compile
    // even when `inc` is called directly, so it is unrelated to the ternary and
    // was never a valid RED/GREEN target. The laziness property the brief means
    // to pin is preserved here with a compilable observable side effect: the
    // untaken arm calls `boom()`, whose `console.log(999)` must NOT appear. If
    // both arms evaluated, stdout would be "999\n5\n"; only the taken arm gives
    // "5\n".
    let out = run_source(
        "function boom() { console.log(999);\nreturn 1; }\nlet a = 1;\nlet x = a > 0 ? 5 : boom();\nconsole.log(x);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn nested_ternary_selects() {
    let out = run_source("let a = 2;\nconsole.log(a == 1 ? 10 : a == 2 ? 20 : 30);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "20\n");
}

#[test]
fn string_and_number_arms_are_rejected() {
    // Repr conflict (merge_nodes) or codegen guard — either way: no compile.
    let out = run_source("let a = 1;\nlet s = \"x\";\nlet v = a > 0 ? s : 5;\nconsole.log(v);\n");
    assert!(
        !out.status.success(),
        "string/number arm mix must be rejected"
    );
}

#[test]
fn string_and_float_arms_are_rejected() {
    // A float-typed result block would promote a handle to f64 — reject.
    let out = run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"!\" : 1.5);\n");
    assert!(
        !out.status.success(),
        "string/float arm mix must be rejected"
    );
}

#[test]
fn ternary_in_never_called_function_still_compiles() {
    let out = run_source("function unused(a) { return a > 0 ? 1 : 2; }\nconsole.log(7);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}
