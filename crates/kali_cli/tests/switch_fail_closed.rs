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

// Until a shape is explicitly admitted, every switch must fail closed with an
// honest E5506 naming the limit — never silently select the wrong clause.
#[test]
fn switch_is_fail_closed_not_silently_wrong() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 10: return \"A\";\n\
             case 20: return \"B\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(20));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("switch"),
        "the diagnostic must name switch as the limit, got: {out}"
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
           console.log(\"after=\" + i);\n\
         }\n\
         console.log(\"done\");\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("switch"),
        "the diagnostic must name switch as the limit, got: {out}"
    );
}
