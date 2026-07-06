use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    let dir =
        std::env::temp_dir().join(format!("kali-strflow-{}-{}", std::process::id(), src.len()));
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
fn string_variable_concat_prints() {
    let out = run_source("let x = \"GG\";\nx = x + \"CC\";\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "GGCC\n");
}

#[test]
fn string_param_return_roundtrip_prints() {
    let out = run_source("function f(s) { return s + \"!\"; }\nconsole.log(f(\"hi\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hi!\n");
}

#[test]
fn string_accumulation_loop_prints() {
    let out = run_source(
        "let a = \"\";\nfor (let i = 0; i < 3; i = i + 1) { a = a + \"y\"; }\nconsole.log(a);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "yyy\n");
}

#[test]
fn string_then_number_binding_is_rejected() {
    // A binding used as both string and number must fail to compile (fail-closed),
    // not silently miscompile.
    let out = run_source("let x = \"a\";\nx = 5;\nconsole.log(x);\n");
    assert!(
        !out.status.success(),
        "mixed string/number binding must be rejected"
    );
}
