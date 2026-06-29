use super::*;

#[test]
fn run_and_test_supports_bracketed_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.jsx",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.jsx",
            "Kali.test('bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.tsx",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg(command)
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["payload"]["hostContract"], "browser-requested");
                assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_atan2_as_const_wrapper_when_browser_harness_is_configured_in_ts_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed atan2 as const wrapper', () => { const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one)); });\n",
            "0\nok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg(command)
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["payload"]["hostContract"], "browser-requested");
                assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_atan2_satisfies_wrapper_when_browser_harness_is_configured_in_ts_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed atan2 satisfies wrapper', () => { const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one)); });\n",
            "0\nok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg(command)
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["payload"]["hostContract"], "browser-requested");
                assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_atan2_bracketed_method_when_browser_harness_is_configured_in_js_and_ts_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed atan2 bracketed method', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed atan2 bracketed method', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one)); });\n",
            "0\nok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg(command)
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}
stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["payload"]["hostContract"], "browser-requested");
                assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_single_quoted_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('single quoted bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('single quoted bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.jsx",
            "const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.jsx",
            "Kali.test('single quoted bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
        (
            "run",
            "main.tsx",
            "const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('single quoted bracketed atan2 zero slice', () => { const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one)); });\n",
            "0\nok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg(command)
                .arg("--api")
                .arg("browser")
                .arg("--max-threads")
                .arg("0")
                .arg("--max-spawned-processes")
                .arg("0")
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["payload"]["hostContract"], "browser-requested");
                assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}
