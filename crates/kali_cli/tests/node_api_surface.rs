use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_kali not set")
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

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

#[test]
fn explicit_node_api_surface_is_supported_for_phase1_check_and_build_commands() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nconsole.log('Checked 1 file(s)');\n",
    )
    .expect("write source");

    let mut check = Command::new(kali_bin());
    check
        .current_dir(dir.path())
        .args(["check", "--api", "node", source_path.to_str().unwrap()]);
    assert_node_api_succeeds("check", check, "Checked 1 file(s)");

    let mut build = Command::new(kali_bin());
    build
        .current_dir(dir.path())
        .args(["build", "--api", "node", source_path.to_str().unwrap()]);
    let output = build.output().expect("run kali");
    assert!(
        output.status.success(),
        "build stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Built executable artifact at"),
        "build stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn inherited_node_api_surface_is_supported_for_phase1_check_and_build_commands() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nconsole.log('Checked 1 file(s)');\n",
    )
    .expect("write source");
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

    let mut check = Command::new(kali_bin());
    check
        .current_dir(dir.path())
        .arg("check")
        .arg(source_path.to_str().unwrap());
    assert_node_api_succeeds("check", check, "Checked 1 file(s)");

    let mut build = Command::new(kali_bin());
    build
        .current_dir(dir.path())
        .arg("build")
        .arg(source_path.to_str().unwrap());
    let output = build.output().expect("run kali");
    assert!(
        output.status.success(),
        "build stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Built executable artifact at"),
        "build stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn explicit_node_api_surface_builds_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn inherited_node_api_surface_builds_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn explicit_node_api_surface_executes_on_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(
        &run_file,
        "import 'node:path';\nimport 'node:timers';\nconsole.log('node run ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
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
        "import 'node:path';\nimport 'node:timers';\nconsole.log('node run ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
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

#[test]
fn inherited_node_api_surface_executes_on_run_and_test_commands_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import 'node:path';\nimport 'node:timers';\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
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

    let run_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&run_file)
        .arg("--")
        .arg("alpha")
        .arg("beta")
        .output()
        .expect("run kali");

    assert!(
        run_output.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_json = parse_json_stdout(&run_output);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["exitCode"], 0);
    assert_eq!(run_json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(run_json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(run_json["stdout"].as_str().expect("stdout").trim(), "2");

    let test_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("run kali");

    assert!(
        test_output.status.success(),
        "test stderr: {}",
        String::from_utf8_lossy(&test_output.stderr)
    );
    let test_json = parse_json_stdout(&test_output);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(test_json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(
        test_json["stdout"].as_str().expect("stdout"),
        "node test ok\n"
    );
}

#[test]
fn explicit_node_api_surface_is_supported_for_phase1_check_and_build_commands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nconsole.log('Checked 1 file(s)');\n",
    )
    .expect("write source");

    let mut check = Command::new(kali_bin());
    check
        .current_dir(dir.path())
        .args(["check", "--api", "node", source_path.to_str().unwrap()]);
    assert_node_api_succeeds("check", check, "Checked 1 file(s)");

    let mut build = Command::new(kali_bin());
    build
        .current_dir(dir.path())
        .args(["build", "--api", "node", source_path.to_str().unwrap()]);
    let output = build.output().expect("run kali");
    assert!(
        output.status.success(),
        "build stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Built executable artifact at"),
        "build stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn explicit_node_api_surface_executes_on_run_and_test_commands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import 'node:path';\nimport 'node:timers';\nconsole.log('node run ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
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
fn explicit_node_api_surface_executes_on_run_and_test_commands_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import 'node:path';\nimport 'node:timers';\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
    )
    .expect("write test file");

    let run_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(&run_file)
        .arg("--")
        .arg("alpha")
        .arg("beta")
        .output()
        .expect("run kali");

    assert!(
        run_output.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_json = parse_json_stdout(&run_output);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["exitCode"], 0);
    assert_eq!(run_json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(run_json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(run_json["stdout"].as_str().expect("stdout").trim(), "2");

    let test_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("node")
        .arg(&test_file)
        .output()
        .expect("run kali");

    assert!(
        test_output.status.success(),
        "test stderr: {}",
        String::from_utf8_lossy(&test_output.stderr)
    );
    let test_json = parse_json_stdout(&test_output);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(test_json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(
        test_json["stdout"].as_str().expect("stdout"),
        "node test ok\n"
    );
}
