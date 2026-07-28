use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
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
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// `s` is NEVER called. If the statement after the switch stays inside `s`,
// nothing mutates `g` and the program prints `g=0` (node's answer). Before the
// fix, `g = 99` was reparented to module scope and ran at module load, so kali
// printed `g=99`.
#[test]
fn statement_after_switch_does_not_escape_the_function() {
    let src = "var g = 0;\n\
               function s(x) {\n\
                 switch (x) {\n\
                   case 1: g = 1;\n\
                 }\n\
                 g = 99;\n\
               }\n\
               console.log(\"g=\" + g);\n";
    assert_eq!(run_js(src), "g=0\n");
}

// A whole function declared AFTER a switch-containing function used to vanish,
// because the leaked `return` terminated the module before it was reached.
#[test]
fn function_declared_after_a_switch_function_survives() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"A\";\n\
                 }\n\
                 return \"Z\";\n\
               }\n\
               function t() { return \"T\"; }\n\
               console.log(\"t=\" + t());\n";
    assert_eq!(run_js(src), "t=T\n");
}

// The callee's own output used to disappear entirely: the leaked `return 0;`
// terminated the module, so the module-scope console.log never ran.
#[test]
fn a_call_whose_callee_contains_a_switch_still_prints() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 10: r = 1;\n\
                 }\n\
                 return 0;\n\
               }\n\
               console.log(\"v=\" + s(10));\n";
    assert_eq!(run_js(src), "v=0\n");
}
