use super::*;

#[test]
fn check_rejects_frozen_intl_number_format_late_compatibility_member_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        "Object.freeze(globalThis.Intl.NumberFormat);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("Intl.NumberFormat") || stderr.contains("Intl"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_await_wrapped_proxy_revocable_late_compatibility_member_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        "(async function main() { Object.freeze(await globalThis.Proxy.revocable)({}, {}); })();\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("Proxy.revocable") || stderr.contains("globalThis.Proxy.revocable"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_and_build_reject_generator_and_async_generator_class_expressions_in_browser_api_surface_jsx_input(
) {
    for (command, is_build) in [("check", false), ("build", true)] {
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
                let source_path = dir.path().join("main.jsx");
                fs::write(&source_path, source).expect("write source");

                let mut cli = Command::new(kali_bin());
                cli.current_dir(dir.path());
                if output_json {
                    cli.arg("--output").arg("json");
                }
                if is_build {
                    cli.arg("build").arg("--bundle");
                } else {
                    cli.arg("check");
                }
                let output = cli
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
fn check_rejects_late_process_control_members_in_browser_api_surface_jsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.jsx");
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
fn check_supports_nullish_coalescing_in_browser_api_surface_jsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.jsx");
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
