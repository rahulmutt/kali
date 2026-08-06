use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn generator_default_export_class_expression_source() -> &'static str {
    "export default (class NamedExample { *main() { yield* []; } });\n"
}

fn async_generator_default_export_class_expression_source() -> &'static str {
    "export default (class NamedExample { async *main() { yield* []; } });\n"
}

fn generator_default_export_sequence_wrapped_class_expression_source() -> &'static str {
    "export default (0, class NamedExample { *main() { yield* []; } });\n"
}

fn async_generator_default_export_sequence_wrapped_class_expression_source() -> &'static str {
    "export default (0, class NamedExample { async *main() { yield* []; } });\n"
}

fn generator_default_export_class_expression_no_delegate_source() -> &'static str {
    "export default (class NamedExample { *main() { yield 1; } });\n"
}

fn async_generator_default_export_class_expression_no_delegate_source() -> &'static str {
    "export default (class NamedExample { async *main() { yield 1; } });\n"
}

fn generator_default_export_sequence_wrapped_class_expression_no_delegate_source() -> &'static str {
    "export default (0, class NamedExample { *main() { yield 1; } });\n"
}

fn async_generator_default_export_sequence_wrapped_class_expression_no_delegate_source(
) -> &'static str {
    "export default (0, class NamedExample { async *main() { yield 1; } });\n"
}

fn assert_browser_harness_generator_rejection(
    command: &str,
    bundle: bool,
    extension: &str,
    source: &str,
    json_output: bool,
) {
    let expected_messages: &[&str] =
        if source.contains("(0, async function*") && matches!(command, "check" | "build") {
            &["generator and async-generator function lowering"]
        } else if source.contains("yield*") {
            &["yield* delegation"]
        } else if source.contains("async function*") {
            &["async-generator function lowering"]
        } else {
            &["generator function lowering"]
        };

    assert_browser_harness_generator_rejection_with_expected_messages(
        command,
        bundle,
        extension,
        source,
        json_output,
        expected_messages,
    );
}

fn assert_browser_harness_generator_rejection_with_expected_messages(
    command: &str,
    bundle: bool,
    extension: &str,
    source: &str,
    json_output: bool,
    expected_messages: &[&str],
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);

    let output = cli.output().expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("error message"))
            .collect::<Vec<_>>();
        assert!(
            expected_messages
                .iter()
                .all(|expected| messages.iter().any(|message| message.contains(expected))),
            "messages: {messages:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            expected_messages
                .iter()
                .all(|expected| stderr.contains(expected)),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn check_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}
