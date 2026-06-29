use super::*;

#[test]
fn inherited_node_api_surface_is_supported_for_phase1_check_and_build_commands() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).expect("create nested dir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nimport 'node:http';\nprocess.chdir('nested');\nconsole.log('Checked 1 file(s)');\n",
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
fn inherited_node_api_surface_builds_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import 'node:path';\nimport 'node:timers';\nimport 'node:http';\nimport 'node:buffer';\nconsole.log(process.pid);\nprocess.cwd();\n",
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
fn inherited_node_api_surface_builds_library_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(
        &source_path,
        "import * as path from 'node:path';\nimport * as timers from 'node:timers';\nexport function describe() { return typeof path.basename === 'function' && typeof timers.clearInterval === 'function' ? 0 : 1; }\n",
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
        .arg("--lib")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_file_name("lib.lib.wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("lib.lib.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("lib.lib.wit")
    );
    assert!(
        payload["exports"]
            .as_array()
            .expect("exports array")
            .iter()
            .any(|entry| entry["name"] == "describe"),
        "build payload exports: {payload:?}"
    );
}

#[test]
fn inherited_node_api_surface_executes_on_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).expect("create nested dir");
    let run_file = dir.path().join("main.ts");
    let test_file = dir.path().join("main.test.ts");
    fs::write(
        &run_file,
        "import fs from 'node:fs';\nprocess.chdir('nested');\nfs.writeFileSync('marker.txt', 'ok');\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import fs from 'node:fs';\nKali.test('node', () => {\n    process.chdir('nested');\n    fs.writeFileSync('marker.txt', 'ok');\n});\n",
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
    assert_node_api_succeeds("run", run, "");

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path())
        .args(["test", test_file.to_str().unwrap()]);
    assert_node_api_succeeds("test", test, "");
}

#[test]
fn inherited_node_api_surface_executes_on_run_and_test_commands_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import 'node:path';\nimport 'node:timers';\nimport 'node:http';\nimport 'node:buffer';\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import 'node:path';\nimport 'node:timers';\nimport 'node:http';\nimport 'node:buffer';\nKali.test('node', () => {\n    console.log('node test ok');\n});\n",
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
fn inherited_node_api_surface_reports_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import 'node:path';
import 'node:timers';
import 'node:http';
import 'node:buffer';
console.log(process.argv.length);
",
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
        .arg("effects")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn inherited_node_api_surface_reports_effects_with_node_timers_promises_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import { setTimeout as delay } from 'node:timers/promises';
console.log(process.argv.length);
",
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
        .arg("effects")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn inherited_node_api_surface_rejects_node_timers_promises_helpers_in_js_input_on_check_build_run_and_test_commands(
) {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import { setTimeout as delay } from 'node:timers/promises';\ndelay(0).then(() => console.log('node timers/promises ok'));\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import { setTimeout as delay } from 'node:timers/promises';\nKali.test('node timers/promises', () => delay(0).then(() => console.log('node timers/promises ok')));\n",
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

    let expected_message =
        "node builtin 'node:timers/promises' is not available on the explicit Node API surface";

    for command in ["check", "build"] {
        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg(&run_file);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg(&run_file);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }

    let mut run = Command::new(kali_bin());
    run.current_dir(dir.path()).arg("run").arg(&run_file);
    let run_output = run.output().expect("run kali");
    assert!(
        !run_output.status.success(),
        "run should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run_output.stderr);
    assert!(
        run_stderr.contains(expected_message),
        "run stderr: {run_stderr}"
    );

    let run_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&run_file)
        .output()
        .expect("run kali");
    assert!(
        !run_json_output.status.success(),
        "json run should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_json_output.stdout),
        String::from_utf8_lossy(&run_json_output.stderr)
    );
    let run_json = parse_json_stdout(&run_json_output);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], false);
    assert_eq!(run_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        run_json["errors"].as_array().expect("errors array"),
        expected_message,
    );

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path()).arg("test").arg(&test_file);
    let test_output = test.output().expect("run kali");
    assert!(
        !test_output.status.success(),
        "test should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_output.stdout),
        String::from_utf8_lossy(&test_output.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test_output.stderr);
    assert!(
        test_stderr.contains(expected_message),
        "test stderr: {test_stderr}"
    );

    let test_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("run kali");
    assert!(
        !test_json_output.status.success(),
        "json test should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json_output.stdout),
        String::from_utf8_lossy(&test_json_output.stderr)
    );
    let test_json = parse_json_stdout(&test_json_output);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], false);
    assert_eq!(test_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        test_json["errors"].as_array().expect("errors array"),
        expected_message,
    );
}

#[test]
fn inherited_node_api_surface_rejects_node_stream_promises_helpers_in_js_input_on_check_build_run_and_test_commands(
) {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import { setTimeout as delay } from 'node:stream/promises';\ndelay(0).then(() => console.log('node stream/promises ok'));\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import { setTimeout as delay } from 'node:stream/promises';\nKali.test('node stream/promises', () => delay(0).then(() => console.log('node stream/promises ok')));\n",
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

    let expected_message =
        "node builtin 'node:stream/promises' is not available on the explicit Node API surface";

    for command in ["check", "build"] {
        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg(&run_file);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg(&run_file);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }

    let mut run = Command::new(kali_bin());
    run.current_dir(dir.path()).arg("run").arg(&run_file);
    let run_output = run.output().expect("run kali");
    assert!(
        !run_output.status.success(),
        "run should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run_output.stderr);
    assert!(
        run_stderr.contains(expected_message),
        "run stderr: {run_stderr}"
    );

    let run_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&run_file)
        .output()
        .expect("run kali");
    assert!(
        !run_json_output.status.success(),
        "json run should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_json_output.stdout),
        String::from_utf8_lossy(&run_json_output.stderr)
    );
    let run_json = parse_json_stdout(&run_json_output);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], false);
    assert_eq!(run_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        run_json["errors"].as_array().expect("errors array"),
        expected_message,
    );

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path()).arg("test").arg(&test_file);
    let test_output = test.output().expect("run kali");
    assert!(
        !test_output.status.success(),
        "test should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_output.stdout),
        String::from_utf8_lossy(&test_output.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test_output.stderr);
    assert!(
        test_stderr.contains(expected_message),
        "test stderr: {test_stderr}"
    );

    let test_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("run kali");
    assert!(
        !test_json_output.status.success(),
        "json test should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json_output.stdout),
        String::from_utf8_lossy(&test_json_output.stderr)
    );
    let test_json = parse_json_stdout(&test_json_output);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], false);
    assert_eq!(test_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        test_json["errors"].as_array().expect("errors array"),
        expected_message,
    );
}

#[test]
fn inherited_node_api_surface_rejects_node_worker_threads_module_in_js_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_file = dir.path().join(format!("main.{extension}"));
        let test_file = dir.path().join(format!("main.test.{extension}"));
        fs::write(
            &run_file,
            r#"import { Worker } from 'node:worker_threads';
console.log(typeof Worker);
"#,
        )
        .expect("write run file");
        fs::write(
            &test_file,
            r#"import { Worker } from 'node:worker_threads';
Kali.test('node worker_threads', () => console.log(typeof Worker));
"#,
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

        let expected_message =
            "node builtin 'node:worker_threads' is not available on the explicit Node API surface";

        for command in ["check", "build", "run", "test"] {
            let input_path = if command == "test" {
                &test_file
            } else {
                &run_file
            };

            let mut text_command = Command::new(kali_bin());
            text_command.current_dir(dir.path()).arg(command);
            text_command.arg(input_path);

            let text_output = text_command.output().expect("run kali");
            assert!(
                !text_output.status.success(),
                "{command} should be rejected on the inherited Node surface for node:worker_threads\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&text_output.stdout),
                String::from_utf8_lossy(&text_output.stderr)
            );
            let text_stderr = String::from_utf8_lossy(&text_output.stderr);
            assert!(
                text_stderr.contains(expected_message),
                "{command} stderr for inherited node:worker_threads: {text_stderr}"
            );

            let mut json_command = Command::new(kali_bin());
            json_command
                .current_dir(dir.path())
                .arg("--output")
                .arg("json")
                .arg(command)
                .arg(input_path);

            let json_output = json_command.output().expect("run kali");
            assert!(
                !json_output.status.success(),
                "json {command} should surface the inherited Node worker_threads rejection as machine-readable output\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&json_output.stdout),
                String::from_utf8_lossy(&json_output.stderr)
            );
            let json = parse_json_stdout(&json_output);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            assert_eq!(json["exitCode"], 1);
            assert_json_node_builtin_rejection(
                json["errors"].as_array().expect("errors array"),
                expected_message,
            );
        }
    }
}
