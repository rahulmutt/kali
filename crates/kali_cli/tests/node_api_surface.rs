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

fn assert_node_api_succeeds(name: &str, mut command: Command, expected_stdout: &str) {
    let output = command.output().expect("run kali");
    assert!(
        output.status.success(),
        "{name} stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(expected_stdout), "{name} stdout: {stdout}");
}

#[test]
fn explicit_node_api_surface_is_rejected_for_phase1_check_and_build_commands() {
    let dir = tempdir().expect("tempdir");
    let check_file = dir.path().join("main.ts");
    fs::write(&check_file, "import 'node:path';\nconst answer = 1;\n").expect("write check file");

    for (name, args) in [
        (
            "check",
            vec!["check", "--api", "node", check_file.to_str().unwrap()],
        ),
        (
            "build",
            vec!["build", "--api", "node", check_file.to_str().unwrap()],
        ),
        (
            "effects",
            vec!["effects", "--api", "node", check_file.to_str().unwrap()],
        ),
    ] {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        for arg in args {
            command.arg(arg);
        }
        assert_node_api_rejected(name, command);
    }
}

#[test]
fn inherited_node_api_surface_is_rejected_for_phase1_check_and_build_commands() {
    let dir = tempdir().expect("tempdir");
    let check_file = dir.path().join("main.ts");
    fs::write(&check_file, "import 'node:path';\nconst answer = 1;\n").expect("write check file");
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

    for (name, args) in [
        ("check", vec!["check", check_file.to_str().unwrap()]),
        ("build", vec!["build", check_file.to_str().unwrap()]),
        ("effects", vec!["effects", check_file.to_str().unwrap()]),
    ] {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        for arg in args {
            command.arg(arg);
        }
        assert_node_api_rejected(name, command);
    }
}

#[test]
fn explicit_node_api_surface_executes_on_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(
        &run_file,
        "import 'node:path';\nconsole.log('node run ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
    )
    .expect("write test file");

    let mut run = Command::new(kali_bin());
    run.current_dir(dir.path())
        .args(["run", "--api", "node", run_file.to_str().unwrap()]);
    assert_node_api_succeeds("run", run, "node run ok\n");

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path())
        .args(["test", "--api", "node", test_file.to_str().unwrap()]);
    assert_node_api_succeeds("test", test, "node test ok\n");
}

#[test]
fn inherited_node_api_surface_executes_on_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(
        &run_file,
        "import 'node:path';\nconsole.log('node run ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
    )
    .expect("write test file");
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

    let mut run = Command::new(kali_bin());
    run.current_dir(dir.path())
        .args(["run", run_file.to_str().unwrap()]);
    assert_node_api_succeeds("run", run, "node run ok\n");

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path())
        .args(["test", test_file.to_str().unwrap()]);
    assert_node_api_succeeds("test", test, "node test ok\n");
}
