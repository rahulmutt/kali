use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// A named function expression keeps its own name; two anonymous arrows in the
/// same module get DISTINCT names. If the pre-pass reused one name, the second
/// body would overwrite the first and this prints the wrong value.
#[test]
fn anonymous_functions_get_distinct_stable_names() {
    let out = run_kali(
        r#"const a = () => 1;
const b = () => 2;
console.log(a() + b());
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
