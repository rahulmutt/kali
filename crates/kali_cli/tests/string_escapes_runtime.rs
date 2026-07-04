use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn string_escapes_decode_to_real_bytes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("esc.ts");
    std::fs::write(
        &src,
        "console.log(\"a\\tb\");\nconsole.log(\"c\\nd\");\nconsole.log(\"e\\\\f\");\n",
    )
    .expect("write");
    let out = Command::new(kali_bin())
        .arg("run")
        .arg(&src)
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Real TAB (0x09), then a\tb newline; real newline inside c/d; single backslash e\f.
    assert_eq!(out.stdout, b"a\tb\nc\nd\ne\\f\n");
}
