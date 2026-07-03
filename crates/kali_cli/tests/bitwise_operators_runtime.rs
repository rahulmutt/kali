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

// Builds a runtime byte = 255 via loop-carried shift+or (mandelbrot's packing),
// defeating constant folding, then exercises each operator against it.
const PACK: &str = "let byte = 0;\nfor (let i = 0; i < 8; i = i + 1) { byte = (byte << 1) | 1; }\n";

#[test]
fn shift_left_and_or_pack_bits() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + byte);")), "255\n");
}
#[test]
fn bitwise_and() {
    assert_eq!(
        run_js(&format!("{PACK}console.log(\"\" + (byte & 15));")),
        "15\n"
    );
}
#[test]
fn bitwise_or() {
    assert_eq!(
        run_js(&format!("{PACK}console.log(\"\" + (byte | 256));")),
        "511\n"
    );
}
#[test]
fn bitwise_xor() {
    assert_eq!(
        run_js(&format!("{PACK}console.log(\"\" + (byte ^ 255));")),
        "0\n"
    );
}
#[test]
fn shift_right_arithmetic() {
    assert_eq!(
        run_js(&format!("{PACK}console.log(\"\" + (byte >> 4));")),
        "15\n"
    );
}
#[test]
fn shift_right_arithmetic_negative() {
    // neg = -255 (runtime), -255 >> 1 = -128 (sign-preserving)
    let src = format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >> 1));");
    assert_eq!(run_js(&src), "-128\n");
}
#[test]
fn unsigned_shift_zero_extends() {
    // -255 >>> 0 = 4294967041 (uint32)
    let src = format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >>> 0));");
    assert_eq!(run_js(&src), "4294967041\n");
}
#[test]
fn bitwise_on_float_operand_is_rejected() {
    // x is f64 (seeded by 1.5), loop-derived so not folded; `x & 1` must reject.
    let src = "let x = 0.0;\nfor (let i = 0; i < 3; i = i + 1) { x = x + 1.5; }\nconsole.log(\"\" + (x & 1));";
    let out = run_js_expect_failure(src);
    assert!(
        out.contains("5506") || out.to_lowercase().contains("bitwise"),
        "expected E5506 bitwise-on-float diagnostic, got: {out}"
    );
}
