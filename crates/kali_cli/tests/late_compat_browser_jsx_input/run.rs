use super::*;

#[test]
fn run_and_test_reject_generator_function_lowering_in_browser_api_surface_jsx_input() {
    for (command, source_name) in [("run", "main.jsx"), ("test", "smoke.test.jsx")] {
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
fn run_and_test_reject_late_network_members_in_browser_api_surface_jsx_input() {
    for (command, source_name) in [("run", "main.jsx"), ("test", "smoke.test.jsx")] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, late_network_source()).expect("write source");

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
                assert_browser_late_network_rejection_json(errors);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert_browser_late_network_rejection(&stderr);
            }
        }
    }
}

#[test]
fn run_and_test_reject_late_process_control_members_in_browser_api_surface_jsx_input() {
    for (command, source_name) in [("run", "main.jsx"), ("test", "smoke.test.jsx")] {
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
fn run_and_test_reject_generator_and_async_generator_class_expressions_in_browser_api_surface_jsx_input(
) {
    for (command, source_name) in [("run", "main.jsx"), ("test", "smoke.test.jsx")] {
        for (source, expected_message) in [
            (
                generator_class_expression_source(),
                "generator class method lowering is unavailable in the direct runtime path",
            ),
            (
                async_generator_default_export_class_expression_source(),
                "async-generator class method lowering is unavailable in the direct runtime path",
            ),
            (
                sequence_wrapped_generator_class_expression_source(),
                "generator class method lowering is unavailable in the direct runtime path for yield* delegation",
            ),
            (
                sequence_wrapped_async_generator_class_expression_source(),
                "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation",
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
fn run_and_test_reject_late_object_model_members_in_browser_api_surface_jsx_input() {
    for (command, source_name) in [("run", "main.jsx"), ("test", "smoke.test.jsx")] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, late_object_model_source()).expect("write source");

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
                assert_browser_late_object_model_rejection_json(errors);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert_browser_late_object_model_rejection(&stderr);
            }
        }
    }
}
