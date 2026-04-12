use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn kali_bin() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(std::path::PathBuf::from)
        .expect("kali binary path")
}

#[test]
fn run_executes_a_simple_source_file() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_rejects_declaration_only_entrypoints() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("decl.d.ts");
    fs::write(&source_path, "declare const value: string;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5007"), "stderr: {stderr}");
}

#[test]
fn test_reports_success_for_explicit_file_sets() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_runs_guest_registered_tests() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("suite.test.ts");
    fs::write(
        &source_path,
        r#"
        Kali.test("addition", () => {
            1 + 2;
        });
        "#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_filters_selected_files_before_execution() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(&keep, "1 + 2;").expect("write keep source");
    fs::write(&skip, "3 + 4;").expect("write skip source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_rejects_coverage_flag_until_report_contract_exists() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--coverage")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}
