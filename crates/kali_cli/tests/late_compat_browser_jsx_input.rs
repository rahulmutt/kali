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
    "function* main() { yield 1; }\nmain();"
}

fn async_generator_function_source() -> &'static str {
    "async function* main() { yield 1; }\nmain();"
}

fn late_process_control_source() -> &'static str {
    "Deno.pid; globalThis.Deno.pid; globalThis[\"Deno\"][\"pid\"]; globalThis[\"Deno\"].cwd; globalThis[\"Deno\"].chdir; globalThis[\"Deno\"].exit; Deno[\"pid\"]; globalThis.Deno[\"pid\"]; globalThis.Deno.cwd; globalThis[\"Deno\"][\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno[\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno.chdir; globalThis.Deno.chdir; globalThis[\"Deno\"][\"chdir\"]; globalThis.Deno[\"chdir\"]; Deno[\"chdir\"]; globalThis.Deno[\"chdir\"]; globalThis.Deno.exit; globalThis[\"Deno\"][\"exit\"]; globalThis.Deno[\"exit\"]; Deno[\"exit\"]; globalThis.Deno[\"exit\"]; process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis[\"process\"].pid; process[\"pid\"]; globalThis.process[\"pid\"]; globalThis.process.cwd; globalThis[\"process\"].cwd; process.chdir; globalThis.process.chdir; process[\"cwd\"]; globalThis.process[\"cwd\"]; process[\"chdir\"]; globalThis.process[\"chdir\"]; process.kill; globalThis.process.kill; globalThis[\"process\"].kill; globalThis[\"process\"][\"kill\"]; process[\"kill\"]; globalThis.process[\"kill\"]; process.exit; globalThis[\"process\"].chdir; globalThis[\"process\"].exit; globalThis[\"process\"][\"cwd\"]; globalThis[\"process\"][\"chdir\"]; globalThis[\"process\"][\"exit\"]; process[\"exit\"]; globalThis.process[\"exit\"];"
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("process.kill"), "stderr: {stderr}");
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
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("process.kill")),
        "missing process.kill in {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("undefined identifier 'process'")),
        "missing process identifier gate in {errors:?}"
    );
}

fn late_object_model_source() -> &'static str {
    r#"Intl; globalThis.Intl; globalThis["Intl"]; globalThis.Intl.NumberFormat; globalThis["Intl"].NumberFormat; globalThis.Intl["NumberFormat"]; globalThis["Intl"].DateTimeFormat; globalThis.Intl["DateTimeFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis.Intl.RelativeTimeFormat; globalThis["Intl"].RelativeTimeFormat; globalThis.Intl["RelativeTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis.Intl.Collator; globalThis["Intl"].Collator; globalThis.Intl["Collator"]; globalThis["Intl"]["Collator"]; globalThis.Intl.DisplayNames; globalThis["Intl"].DisplayNames; globalThis.Intl["DisplayNames"]; globalThis["Intl"]["DisplayNames"]; globalThis.Intl.Segmenter; globalThis["Intl"].Segmenter; globalThis.Intl["Segmenter"]; globalThis["Intl"]["Segmenter"]; globalThis.Intl.Locale; globalThis["Intl"].Locale; globalThis.Intl["Locale"]; globalThis["Intl"]["Locale"]; globalThis.Intl.PluralRules; globalThis["Intl"]["PluralRules"]; globalThis["Intl"].PluralRules; globalThis.Intl["PluralRules"]; Proxy; globalThis.Proxy; globalThis["Proxy"]; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; new WeakRef(); globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"]; globalThis.SharedArrayBuffer; globalThis.Atomics;"#
}

fn assert_browser_late_object_model_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "globalThis.Intl.PluralRules",
        r#"globalThis["Intl"]["PluralRules"]"#,
        "Proxy",
        "globalThis.Proxy",
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
        "WeakMap",
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        "SharedArrayBuffer",
        "Atomics",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_object_model_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "globalThis.Intl.PluralRules",
        r#"globalThis["Intl"]["PluralRules"]"#,
        "Proxy",
        "globalThis.Proxy",
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
        "WeakMap",
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        "SharedArrayBuffer",
        "Atomics",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

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
