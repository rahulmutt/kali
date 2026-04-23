use std::{fs, path::PathBuf, process::Command};
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_kali not set")
}

fn assert_node_api_rejected(name: &str, mut command: Command) {
    let output = command.output().expect("run kali");
    assert!(!output.status.success(), "{name} unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(5), "{name} exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "{name} stderr: {stderr}");
    assert!(
        stderr.contains("API surface 'node' is unavailable in this phase"),
        "{name} stderr: {stderr}"
    );
}

#[test]
fn explicit_node_api_surface_is_rejected_for_phase1_commands() {
    let dir = tempdir().expect("tempdir");
    let check_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(&check_file, "const answer = 1;").expect("write check file");
    fs::write(&test_file, "test('ok', () => {});").expect("write test file");

    let cases = [
        (
            "check",
            vec!["check", "--api", "node", check_file.to_str().unwrap()],
        ),
        (
            "build",
            vec!["build", "--api", "node", check_file.to_str().unwrap()],
        ),
        (
            "run",
            vec!["run", "--api", "node", check_file.to_str().unwrap()],
        ),
        (
            "test",
            vec!["test", "--api", "node", test_file.to_str().unwrap()],
        ),
        (
            "effects",
            vec!["effects", "--api", "node", check_file.to_str().unwrap()],
        ),
    ];

    for (name, args) in cases {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        for arg in args {
            command.arg(arg);
        }
        assert_node_api_rejected(name, command);
    }
}

#[test]
fn inherited_node_api_surface_is_rejected_for_phase1_commands() {
    let dir = tempdir().expect("tempdir");
    let check_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(&check_file, "const answer = 1;").expect("write check file");
    fs::write(&test_file, "test('ok', () => {});").expect("write test file");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let cases = [
        ("check", vec!["check", check_file.to_str().unwrap()]),
        ("build", vec!["build", check_file.to_str().unwrap()]),
        ("run", vec!["run", check_file.to_str().unwrap()]),
        ("test", vec!["test", test_file.to_str().unwrap()]),
        ("effects", vec!["effects", check_file.to_str().unwrap()]),
    ];

    for (name, args) in cases {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        for arg in args {
            command.arg(arg);
        }
        assert_node_api_rejected(name, command);
    }
}
