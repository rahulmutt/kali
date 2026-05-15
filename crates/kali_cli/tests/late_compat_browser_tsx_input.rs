use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_browser_tsx_compatibility_source() -> &'static str {
    r#"Intl; globalThis.Intl.NumberFormat; globalThis["Intl"].NumberFormat; globalThis.Intl["NumberFormat"]; globalThis["Intl"]["NumberFormat"]; globalThis["Intl"].DateTimeFormat; globalThis.Intl["DateTimeFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis.Intl.RelativeTimeFormat; globalThis["Intl"].RelativeTimeFormat; globalThis.Intl["RelativeTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis.Intl.Collator; globalThis["Intl"].Collator; globalThis.Intl["Collator"]; globalThis["Intl"]["Collator"]; globalThis.Intl.DisplayNames; globalThis["Intl"].DisplayNames; globalThis.Intl["DisplayNames"]; globalThis["Intl"]["DisplayNames"]; globalThis.Intl.Segmenter; globalThis["Intl"].Segmenter; globalThis.Intl["Segmenter"]; globalThis["Intl"]["Segmenter"]; globalThis.Intl.Locale; globalThis["Intl"].Locale; globalThis.Intl["Locale"]; globalThis["Intl"]["Locale"]; globalThis.Intl.PluralRules; globalThis["Intl"].PluralRules; globalThis.Intl["PluralRules"]; globalThis["Intl"]["PluralRules"]; Deno.permissions.request(); Deno.permissions.revoke(); Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); Deno.env.toObject(); Deno["env"]["toObject"](); globalThis["Deno"]["env"]["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis["Deno"]["env"].toObject(); Deno.env.set('KALI_ENV_SET_SMOKE', 'hello'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"]["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"]["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); process.pid; globalThis.process.pid; globalThis["process"].pid; globalThis["process"]["pid"]; process["pid"]; globalThis.process["pid"]; process.cwd; globalThis.process.cwd; globalThis["process"].cwd; globalThis["process"]["cwd"]; process["cwd"]; globalThis.process["cwd"]; process.chdir; globalThis.process.chdir; globalThis["process"].chdir; globalThis["process"]["chdir"]; process["chdir"]; globalThis.process["chdir"]; process.kill; globalThis.process.kill; globalThis["process"].kill; globalThis["process"]["kill"]; process["kill"]; globalThis.process["kill"]; const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); process.kill(0); process.kill(+0); process.kill((0)); ((process)).kill(0); ((globalThis.process)).kill(0); globalThis.process.kill(0); globalThis["process"]["kill"](+0); globalThis["process"]["kill"]((0)); globalThis["process"].kill(0); globalThis["process"]["kill"](0); globalThis.process["kill"](0); Object.freeze(globalThis.process["kill"])(0); Object.freeze(globalThis["process"]["kill"])(0); ((process.kill))(0); ((process["kill"]))(0); ((process["kill"]))(0); ((globalThis.process.kill))(0); ((globalThis["process"].kill))(0); ((globalThis["process"]["kill"]))(0); process.exit; globalThis.process.exit; globalThis["process"].exit; globalThis["process"]["exit"]; process["exit"]; globalThis.process["exit"]; Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); Object.hasOwn({}, 'a'); Object.prototype.hasOwnProperty.call({}, 'a'); globalThis.Object.prototype["hasOwnProperty"].call({}, 'a'); globalThis.Object["hasOwn"]({}, 'a'); globalThis["Object"].prototype["hasOwnProperty"].call({}, 'a'); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; new WeakRef(); globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"]; globalThis.SharedArrayBuffer; globalThis.Atomics; Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno["connect"]('127.0.0.1', 1); globalThis["Deno"].connect('127.0.0.1', 1); globalThis["Deno"]["connect"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno["listen"]('127.0.0.1', 0); globalThis["Deno"].listen('127.0.0.1', 0); globalThis["Deno"]["listen"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno["serve"]('127.0.0.1', 0); globalThis["Deno"].serve('127.0.0.1', 0); globalThis["Deno"]["serve"]('127.0.0.1', 0);"#
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

fn late_process_control_source() -> &'static str {
    "Deno.pid; globalThis.Deno.pid; globalThis[\"Deno\"][\"pid\"]; globalThis[\"Deno\"].cwd; globalThis[\"Deno\"].chdir; globalThis[\"Deno\"].exit; Deno[\"pid\"]; globalThis.Deno[\"pid\"]; globalThis.Deno.cwd; globalThis[\"Deno\"][\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno[\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno.chdir; globalThis.Deno.chdir; globalThis[\"Deno\"][\"chdir\"]; globalThis.Deno[\"chdir\"]; Deno[\"chdir\"]; globalThis.Deno[\"chdir\"]; globalThis.Deno.exit; globalThis[\"Deno\"][\"exit\"]; globalThis.Deno[\"exit\"]; Deno[\"exit\"]; globalThis.Deno[\"exit\"]; process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis[\"process\"].pid; process[\"pid\"]; globalThis.process[\"pid\"]; process.cwd; globalThis.process.cwd; globalThis[\"process\"].cwd; globalThis[\"process\"][\"cwd\"]; process[\"cwd\"]; globalThis.process[\"cwd\"]; process.chdir; globalThis.process.chdir; globalThis[\"process\"].chdir; globalThis[\"process\"][\"chdir\"]; process[\"chdir\"]; globalThis.process[\"chdir\"]; process.kill; globalThis.process.kill; globalThis[\"process\"].kill; globalThis[\"process\"][\"kill\"]; process[\"kill\"]; globalThis.process[\"kill\"]; const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); process.kill(0); process.kill(+0); process.kill((0)); ((process)).kill(0); ((globalThis.process)).kill(0); globalThis.process.kill(0); globalThis[\"process\"][\"kill\"](+0); globalThis[\"process\"][\"kill\"]((0)); globalThis[\"process\"].kill(0); globalThis[\"process\"][\"kill\"](0); globalThis.process[\"kill\"](0); Object.freeze(globalThis.process[\"kill\"])(0); ((process.kill))(0); ((process[\"kill\"]))(0); ((globalThis.process.kill))(0); ((globalThis[\"process\"].kill))(0); ((globalThis[\"process\"][\"kill\"]))(0); process.exit; globalThis.process.exit; globalThis[\"process\"].exit; globalThis[\"process\"][\"exit\"]; process[\"exit\"]; globalThis.process[\"exit\"];"
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in ["process.kill", "undefined identifier 'process'"] {
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
    for expected in ["process.kill", "undefined identifier 'process'"] {
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
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        "((process.kill))(0)",
        r#"((process["kill"]))(0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

fn assert_browser_late_tsx_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("process.kill"), "stderr: {stderr}");
}

#[test]
fn browser_late_tsx_compatibility_source_includes_bracketed_forms() {
    let source = late_browser_tsx_compatibility_source();
    assert!(
        source.contains(r#"globalThis.Object.prototype["hasOwnProperty"].call"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Object["hasOwn"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Object"].prototype["hasOwnProperty"].call"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"].NumberFormat"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.RelativeTimeFormat"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Collator"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.DisplayNames"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Segmenter"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Locale"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.PluralRules"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl["PluralRules"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy.revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Proxy"]["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Deno["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].toObject"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["permissions"]["request"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["permissions"]["revoke"]"#),
        "source: {source}"
    );
    assert!(source.contains(r#"Deno["env"].set"#), "source: {source}");
    assert!(source.contains(r#"Deno["env"].delete"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis.Deno["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"].env["set"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"].env["delete"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["set"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["delete"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["pid"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["cwd"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["chdir"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["kill"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["exit"]"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.connect('127.0.0.1', 1)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["connect"]('127.0.0.1', 1)"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.listen('127.0.0.1', 0)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["listen"]('127.0.0.1', 0)"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.serve('127.0.0.1', 0)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["serve"]('127.0.0.1', 0)"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_tsx_compatibility_source_includes_mixed_bracketed_proxy_revocable_form() {
    let source = late_browser_tsx_compatibility_source();
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
}

#[test]
fn run_and_test_reject_generator_function_lowering_in_browser_api_surface_tsx_input() {
    for (command, source_name) in [("run", "main.tsx"), ("test", "smoke.test.tsx")] {
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
fn run_and_test_reject_generator_and_async_generator_class_expressions_in_browser_api_surface_tsx_input(
) {
    for (command, source_name) in [("run", "main.tsx"), ("test", "smoke.test.tsx")] {
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
fn check_rejects_late_process_control_members_in_browser_api_surface_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
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
fn build_rejects_late_process_control_members_in_browser_bundle_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
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

fn assert_browser_late_tsx_compatibility_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().any(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("undefined identifier 'process'")),
        "missing process identifier gate in {errors:?}"
    );
}

#[test]
fn run_rejects_late_browser_compatibility_forms_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, late_browser_tsx_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_tsx_compatibility_rejection(&stderr);
}

#[test]
fn run_rejects_late_browser_compatibility_forms_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, late_browser_tsx_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_tsx_compatibility_rejection_json(errors);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_tsx_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_tsx_compatibility_rejection_json(errors);
}

fn nullish_coalescing_source() -> &'static str {
    "const value = null ?? 1;\nconsole.log(value);\n"
}

#[test]
fn check_supports_nullish_coalescing_in_browser_api_surface_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
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
fn build_supports_nullish_coalescing_in_browser_bundle_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
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
