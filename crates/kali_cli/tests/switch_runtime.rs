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

const S: &str = "function s(x) {\n\
                   switch (x) {\n\
                     case 10: return \"A\";\n\
                     case 20: return \"B\";\n\
                     default: return \"D\";\n\
                   }\n\
                 }\n";

// Anti-spot-check: s(10) is EXCLUDED on purpose. The pre-fix lowering returned
// "A" for every truthy discriminant, so s(10) agreed with node by coincidence
// and proves nothing. Every assertion below uses a discriminant the broken
// lowering got wrong, and the answers must vary with the input.
#[test]
fn numeric_switch_selects_the_matching_clause() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(20));")), "v=B\n");
}
#[test]
fn numeric_switch_falls_to_default_on_no_match() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(40));")), "v=D\n");
}
#[test]
fn numeric_switch_handles_a_zero_discriminant() {
    // The pre-fix lowering truthiness-tested the discriminant, so 0 took the
    // else branch and returned clause 1's "B". This is the cell that proves
    // the discriminant is compared, not tested for truth.
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(0));")), "v=D\n");
}
#[test]
fn numeric_switch_reaches_the_third_clause() {
    // Clauses beyond the second were never emitted at all (the Tier-1 half).
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   case 3: return \"three\";\n\
                   case 4: return \"four\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(3));")), "v=three\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(4));")), "v=four\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=other\n");
}
#[test]
fn numeric_switch_evaluates_the_discriminant_exactly_once() {
    // If the chain re-emitted the discriminant per clause test, `hits` would
    // count once per comparison instead of once per call.
    let src = "var hits = 0;\n\
               function d(x) { hits = hits + 1; return x; }\n\
               function s(x) {\n\
                 switch (d(x)) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n\
               s(2);\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "hits=1\n");
}
// Moved from `switch_fail_closed.rs`'s `switch_is_fail_closed_not_silently_wrong`:
// this exact shape (numeric discriminant, `case 10`/`case 20`/`default`, all
// `return`) is now ADMITTED and correct, so it belongs here among the
// correctness tests rather than in a file named "fail_closed".
#[test]
fn numeric_switch_is_correct_for_the_shape_that_was_previously_denied() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 10: return \"A\";\n\
                   case 20: return \"B\";\n\
                   default: return \"D\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(20));")), "v=B\n");
}
#[test]
fn numeric_switch_selects_correctly_with_a_negative_case_test() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case -1: return \"neg\";\n\
                   case 1: return \"pos\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(-1));")), "v=neg\n");
}
