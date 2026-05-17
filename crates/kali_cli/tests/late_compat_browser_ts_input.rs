use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn generator_function_source() -> &'static str {
    "function* main() { yield* []; }\nmain();"
}

fn async_generator_function_source() -> &'static str {
    "async function* main() { yield* []; }\nmain();"
}

fn generator_class_expression_source() -> &'static str {
    "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n"
}

fn async_generator_default_export_class_expression_source() -> &'static str {
    "export default (class NamedExample { async *main() { yield 1; } });\n"
}

fn late_process_control_source() -> String {
    let process_kill_zero_probe_source = kali_common::process_kill_zero_probe_source();
    format!(
        "process.kill; globalThis.process.kill; globalThis[\"process\"].kill; globalThis[\"process\"][\"kill\"]; process[\"kill\"]; globalThis.process[\"kill\"]; const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); process.kill(0); process.kill(+0); process[\"kill\"](+0); process.kill((0)); ((process)).kill(0); ((globalThis.process)).kill(0); globalThis.process.kill(0); globalThis.process[\"kill\"](+0); globalThis[\"process\"][\"kill\"](+0); globalThis[\"process\"][\"kill\"]((0)); globalThis[\"process\"].kill(0); globalThis[\"process\"].kill(+0); globalThis[\"process\"][\"kill\"](0); globalThis.process[\"kill\"](0); Object.freeze(globalThis.process.kill)(0); Object.freeze(globalThis.process.kill)(+0); Object.freeze((globalThis.process[\"kill\"]))(0); Object.freeze((globalThis.process[\"kill\"]))(+0); Object.freeze((globalThis[\"process\"][\"kill\"]))(0); Object.freeze((globalThis[\"process\"][\"kill\"]))(+0); Object.freeze(globalThis.process[\"kill\"])(0); Object.freeze(globalThis.process[\"kill\"])(+0); Object.freeze((process.kill))(0); Object.freeze((process.kill))(+0); Object.freeze(globalThis[\"process\"].kill)(0); Object.freeze(globalThis[\"process\"].kill)(+0); Object.freeze(globalThis[\"process\"][\"kill\"])(0); Object.freeze(globalThis[\"process\"][\"kill\"])(+0); Object.freeze(process)[\"kill\"](0); Object.freeze(globalThis.process)[\"kill\"](0); Object.freeze(globalThis.process)[\"kill\"](+0); Object.freeze(globalThis[\"process\"])[\"kill\"](0); Object.freeze(globalThis[\"process\"])[\"kill\"](+0); Object.freeze(process.kill)(0); Object.freeze(process.kill)(+0); ((process.kill))(0); ((process[\"kill\"]))(0); ((process[\"kill\"]))(+0); ((globalThis.process.kill))(0); ((globalThis.process[\"kill\"]))(0); ((globalThis[\"process\"].kill))(0); ((globalThis[\"process\"][\"kill\"]))(0); ((globalThis[\"process\"][\"kill\"]))(+0); {}; process.exit; globalThis[\"process\"].chdir; globalThis[\"process\"].exit; globalThis[\"process\"][\"cwd\"]; globalThis[\"process\"][\"chdir\"]; globalThis[\"process\"][\"exit\"]; process[\"exit\"]; globalThis.process[\"exit\"];",
        process_kill_zero_probe_source.trim_end_matches(';'),
    )
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "process.kill",
        "globalThis.process.kill",
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_process_control_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E3100") | Some("E5506"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E3100"),
        "expected at least one E3100 error: {errors:?}"
    );
    for expected in [
        "process.kill",
        "globalThis.process.kill",
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        "undefined identifier 'process'",
    ] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {errors:?}"
        );
    }
}

#[test]
fn browser_late_process_control_source_includes_zero_probe_invocation_forms() {
    let source = late_process_control_source();
    for expected in [
        r#"const zero = 0"#,
        r#"const zeroAlias = zero"#,
        r#"process.kill(zeroAlias)"#,
        "process.kill(0)",
        "process.kill(+0)",
        "process.kill((0))",
        "((process)).kill(0)",
        "((globalThis.process)).kill(0)",
        "globalThis.process.kill(0)",
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0); Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0); Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        "((process.kill))(0)",
        r#"((process["kill"]))(0)"#,
        r#"((process["kill"]))(+0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn run_and_test_reject_generator_function_lowering_in_browser_api_surface_ts_input() {
    for (command, source_name) in [("run", "main.ts"), ("test", "smoke.test.ts")] {
        for source in [
            generator_function_source(),
            async_generator_function_source(),
        ] {
            for output_json in [false, true] {
                let dir = tempdir().expect("tempdir");
                let source_path = dir.path().join(source_name);
                fs::write(&source_path, source).expect("write source");

                let mut cli = Command::new(kali_bin());
                cli.current_dir(dir.path())
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
                if output_json {
                    cli.arg("--output").arg("json");
                }
                let output = cli
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

                assert!(!output.status.success());
                assert_eq!(output.status.code(), Some(1));
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], false);
                    let errors = json["errors"].as_array().expect("errors array");
                    assert!(errors.iter().any(|error| error["code"] == "E5506"));
                    let messages = errors
                        .iter()
                        .map(|error| error["message"].as_str().expect("message"))
                        .collect::<Vec<_>>();
                    assert!(
                        messages
                            .iter()
                            .any(|message| message.contains("generator function lowering")
                                || message.contains("yield expressions")),
                        "messages: {messages:?}"
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    assert!(stderr.contains("E5506"), "stderr: {stderr}");
                    assert!(
                        stderr.contains("generator function lowering")
                            || stderr.contains("yield expressions"),
                        "stderr: {stderr}"
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_reject_late_process_control_members_in_browser_api_surface_ts_input() {
    for (command, source_name) in [("run", "main.ts"), ("test", "smoke.test.ts")] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, late_process_control_source()).expect("write source");

            let mut cli = Command::new(kali_bin());
            cli.current_dir(dir.path())
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
            if output_json {
                cli.arg("--output").arg("json");
            }
            let output = cli
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

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(1));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                let errors = json["errors"].as_array().expect("errors array");
                assert_browser_late_process_control_rejection_json(errors);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert_browser_late_process_control_rejection(&stderr);
            }
        }
    }
}

#[test]
fn run_and_test_reject_generator_and_async_generator_class_expressions_in_browser_api_surface_ts_input(
) {
    for (command, source_name) in [("run", "main.ts"), ("test", "smoke.test.ts")] {
        for (source, expected_message) in [
            (
                generator_class_expression_source(),
                "generator class method lowering is unavailable in the direct runtime path",
            ),
            (
                async_generator_default_export_class_expression_source(),
                "async-generator class method lowering is unavailable in the direct runtime path",
            ),
        ] {
            for output_json in [false, true] {
                let dir = tempdir().expect("tempdir");
                let source_path = dir.path().join(source_name);
                fs::write(&source_path, source).expect("write source");

                let mut cli = Command::new(kali_bin());
                cli.current_dir(dir.path())
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
                if output_json {
                    cli.arg("--output").arg("json");
                }
                let output = cli
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

                assert!(!output.status.success());
                assert_eq!(output.status.code(), Some(1));
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], false);
                    let errors = json["errors"].as_array().expect("errors array");
                    assert!(errors.iter().any(|error| error["code"] == "E5506"));
                    let messages = errors
                        .iter()
                        .map(|error| error["message"].as_str().expect("message"))
                        .collect::<Vec<_>>();
                    assert!(
                        messages.iter().any(|message| {
                            message.contains(expected_message)
                                || message.contains("yield expressions")
                        }),
                        "messages: {messages:?}"
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    assert!(stderr.contains("E5506"), "stderr: {stderr}");
                    assert!(
                        stderr.contains(expected_message) || stderr.contains("yield expressions"),
                        "stderr: {stderr}"
                    );
                }
            }
        }
    }
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_ts_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, late_process_control_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("check")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "check");
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_process_control_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_process_control_rejection(&stderr);
        }
    }
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_ts_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, late_process_control_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_process_control_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_process_control_rejection(&stderr);
        }
    }
}

fn nullish_coalescing_source() -> &'static str {
    "const value = null ?? 1;\nconsole.log(value);\n"
}

#[test]
fn check_supports_nullish_coalescing_in_browser_api_surface_ts_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, nullish_coalescing_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("check")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success());
        assert_eq!(output.status.code(), Some(0));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "check");
            assert_eq!(json["success"], true);
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        }
    }
}

#[test]
fn build_supports_nullish_coalescing_in_browser_bundle_ts_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, nullish_coalescing_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success());
        assert_eq!(output.status.code(), Some(0));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], true);
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        }
    }
}
