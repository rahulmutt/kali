use super::*;

#[test]
fn late_env_materialization_source_includes_bracketed_spellings() {
    let source = late_env_materialization_source();
    for expected in [
        r#"Deno.env["toObject"]"#,
        r#"Deno["env"]["toObject"]"#,
        r#"Deno["env"].toObject"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis.Deno["env"].toObject"#,
        r#"globalThis.Deno.env["toObject"]"#,
        r#"globalThis["Deno"].env.toObject"#,
        r#"globalThis["Deno"].env["toObject"]"#,
        r#"globalThis["Deno"]["env"].toObject"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
        r#"globalThis["Deno"]["env"].toObject"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_process_control_source_includes_bracketed_spellings() {
    let source = late_process_control_source();
    for expected in [
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        r#"globalThis["process"].pid"#,
        r#"globalThis["process"]["pid"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["kill"]"#,
        r#"process['kill']"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis['process'].kill"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["exit"]"#,
        r#"process['exit']"#,
        r#"globalThis.process["exit"]"#,
        r#"globalThis['process'].exit"#,
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_process_control_source_includes_single_quoted_spellings() {
    let source = late_process_control_source();
    for expected in [
        r#"process['kill']"#,
        r#"globalThis['process'].kill"#,
        r#"globalThis['process']['kill']"#,
        r#"process['exit']"#,
        r#"globalThis['process'].exit"#,
        r#"globalThis['process']['exit']"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_process_control_source_includes_zero_probe_spellings() {
    let source = late_process_control_source();
    for expected in [
        "process.kill(0)",
        "process.kill(+0)",
        r#"process["kill"](+0)"#,
        r#"process['kill'](0)"#,
        r#"process['kill'](+0)"#,
        "process.kill((0))",
        "((process)).kill(0)",
        "((process)).kill(+0)",
        "((globalThis.process)).kill(0)",
        "((globalThis.process)).kill(+0)",
        "globalThis.process.kill(0)",
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis['process'].kill(0)"#,
        r#"globalThis['process'].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis.process["kill"](0)"#,
        r#"globalThis["process"].kill(0)"#,
        "((process.kill))(0)",
        r#"((process["kill"]))(0)"#,
        r#"((process['kill']))(0)"#,
        "((globalThis.process.kill))(0)",
        "((globalThis.process.kill))(+0)",
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(+0)"#,
        r#"((globalThis['process'].kill))(0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process).kill)(0)"#,
        r#"Object.freeze((globalThis.process).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0)"#,
        r#"Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"Object.freeze((globalThis.process))["kill"](0)"#,
        r#"Object.freeze((globalThis.process))["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](+0)"#,
        r#"Object.freeze((process))["kill"](0)"#,
        r#"Object.freeze((process))["kill"](+0)"#,
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((process).kill)(0)"#,
        r#"Object.freeze((process).kill)(+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze(process.kill)(+0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
    assert!(
        source.contains(kali_common::process_kill_zero_probe_alias_inventory_source().as_str()),
        "source: {source}"
    );
}

#[test]
fn late_process_env_mutation_source_includes_bracketed_spellings() {
    let source = late_process_env_mutation_source();
    for expected in [
        r#"process["env"]"#,
        r#"process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"process.env.KALI_BROWSER_ENV_MUTATION"#,
        r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis.process["env"]"#,
        r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env"#,
        r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"globalThis["process"]["env"]"#,
        r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_process_env_mutation_source_is_rejected_on_the_default_standalone_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_env_mutation_source()).expect("write source");

    for command in ["check", "build", "run", "test"] {
        for json_output in [false, true] {
            let mut command_line = Command::new(kali_bin());
            command_line.current_dir(dir.path());
            if json_output {
                command_line.arg("--output").arg("json");
            }
            command_line.arg(command).arg(&source_path);

            let output = command_line.output().expect("run kali");
            assert!(
                !output.status.success(),
                "{command} should reject late process env mutation (json={json_output})\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.status.code(), Some(1));

            if json_output {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["success"], false);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.iter().any(|error| error["code"] == "E5506"));
                assert!(errors.iter().any(|error| {
                    error["message"]
                        .as_str()
                        .expect("error message")
                        .contains("process.env")
                }));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(stderr.contains("E5506"), "stderr: {stderr}");
                assert!(
                    stderr.contains("process.env") || stderr.contains(r#"process["env"]"#),
                    "stderr: {stderr}"
                );
            }
        }
    }
}

#[test]
fn late_object_model_source_includes_bracketed_spellings() {
    let source = late_object_model_source();
    for expected in [
        r#"new Proxy({}, {})"#,
        r#"new globalThis.Proxy({}, {})"#,
        r#"new globalThis["Proxy"]({}, {})"#,
        r#"new globalThis['Proxy']({}, {})"#,
        r#"globalThis["Proxy"]"#,
        r#"globalThis['Proxy']"#,
        r#"globalThis["WeakMap"]"#,
        r#"globalThis['WeakMap']"#,
        r#"Object.freeze(globalThis["WeakMap"])"#,
        r#"Object.freeze(globalThis['WeakMap'])"#,
        r#"globalThis["WeakSet"]"#,
        r#"globalThis['WeakSet']"#,
        r#"Object.freeze(globalThis["WeakSet"])"#,
        r#"Object.freeze(globalThis['WeakSet'])"#,
        r#"globalThis["WeakRef"]"#,
        r#"globalThis['WeakRef']"#,
        r#"Object.freeze((globalThis["WeakRef"]))"#,
        r#"Object.freeze((globalThis['WeakRef']))"#,
        r#"globalThis["FinalizationRegistry"]"#,
        r#"globalThis['FinalizationRegistry']"#,
        r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
        r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
        r#"globalThis["Proxy"]["revocable"]"#,
        r#"globalThis['Proxy']['revocable']"#,
        r#"globalThis["Proxy"].revocable"#,
        r#"globalThis['Proxy'].revocable"#,
        r#"globalThis.Proxy["revocable"]"#,
        r#"globalThis['Proxy']["revocable"]"#,
        r#"Object.freeze(globalThis['Proxy']["revocable"])"#,
        r#"Object.freeze((globalThis["Proxy"])["revocable"])"#,
        r#"Object.freeze((globalThis['Proxy'])['revocable'])"#,
        r#"globalThis.Proxy['revocable']"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_object_model_own_property_source_includes_bracketed_spellings() {
    let source = late_object_model_own_property_source();
    for expected in [
        r#"globalThis.Object["hasOwn"]"#,
        r#"globalThis["Object"].hasOwn"#,
        r#"globalThis.Object["prototype"].hasOwnProperty.call"#,
        r#"globalThis.Object.prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty["call"]"#,
        r#"globalThis["Object"].prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty.call"#,
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty.call"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty["call"]"#,
        r#"globalThis.Object["prototype"].hasOwnProperty["call"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn permission_escalation_source_includes_bracketed_spellings() {
    let source = permission_escalation_source();
    for expected in [
        r#"Deno.permissions["request"]"#,
        r#"Deno.permissions["revoke"]"#,
        r#"globalThis.Deno.permissions["request"]"#,
        r#"globalThis.Deno.permissions["revoke"]"#,
        r#"globalThis["Deno"].permissions.request"#,
        r#"globalThis["Deno"].permissions.revoke"#,
        r#"globalThis["Deno"].permissions["request"]"#,
        r#"globalThis["Deno"].permissions["revoke"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn permission_escalation_bracketed_source_includes_inherited_bracketed_spellings() {
    let source = permission_escalation_bracketed_source();
    for expected in [
        r#"globalThis.Deno["permissions"]["request"]"#,
        r#"globalThis.Deno["permissions"]["revoke"]"#,
        r#"globalThis["Deno"]["permissions"]["request"]"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn permission_escalation_mixed_bracketed_source_includes_mixed_spellings() {
    let source = permission_escalation_mixed_bracketed_source();
    for expected in [
        r#"globalThis["Deno"].permissions.request"#,
        r#"globalThis["Deno"].permissions.revoke"#,
        r#"globalThis["Deno"].permissions["request"]"#,
        r#"globalThis["Deno"].permissions["revoke"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn broader_intl_source_includes_bracketed_spellings() {
    let source = broader_intl_source();
    let intl_source = kali_common::broader_intl_source();

    assert!(source.contains(intl_source.as_str()), "source: {source}");
}

#[test]
fn threaded_runtime_source_includes_bracketed_spellings() {
    let source = threaded_runtime_source();
    for expected in [
        r#"globalThis["SharedArrayBuffer"]"#,
        r#"globalThis["Atomics"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn standalone_surface_supports_bracketed_deno_chdir_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno[\"chdir\"]('nested'); globalThis.Deno.chdir('nested'); globalThis.Deno[\"chdir\"]('nested'); globalThis[\"Deno\"].chdir('nested'); globalThis[\"Deno\"][\"chdir\"]('nested');\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn standalone_surface_supports_deno_exit_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "globalThis.Deno.exit(7); globalThis[\"Deno\"].exit(7); Deno.exit(7); Deno[\"exit\"](7); globalThis.Deno[\"exit\"](7); globalThis[\"Deno\"][\"exit\"](7);\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert_eq!(
        output.status.code(),
        Some(7),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn browser_bundle_harness_command_override_supports_quoted_arguments() {
    let parts = split_command_spec(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    )
    .expect("split valid browser harness command");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );
}

#[test]
fn browser_bundle_harness_command_override_preserves_empty_quoted_arguments() {
    let parts = split_command_spec(r#"browser-wrapper "" --flag '' trailing"#)
        .expect("split browser harness command with empty quoted arguments");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "".to_string(),
            "--flag".to_string(),
            "".to_string(),
            "trailing".to_string(),
        ]
    );
}

#[test]
fn browser_bundle_harness_command_override_rejects_empty_executable_token() {
    assert_eq!(split_command_spec("   "), None);
    assert_eq!(split_command_spec(r#"" --flag"#), None);
}

#[test]
fn browser_bundle_harness_command_override_rejects_unterminated_quotes() {
    assert_eq!(split_command_spec(r#"browser-wrapper "unterminated"#), None);
}

#[test]
fn browser_bundle_harness_command_override_rejects_malformed_environment_values() {
    assert!(
        std::panic::catch_unwind(|| { browser_bundle_harness_command_parts_for(Some("")) })
            .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| { browser_bundle_harness_command_parts_for(Some("   ")) })
            .is_err()
    );
    assert!(std::panic::catch_unwind(|| {
        browser_bundle_harness_command_parts_for(Some(r#"" --flag"#))
    })
    .is_err());
    assert!(std::panic::catch_unwind(|| {
        browser_bundle_harness_command_parts_for(Some(r#"browser-wrapper "unterminated"#))
    })
    .is_err());
}

#[test]
fn doctor_emits_json_envelope_for_browser_harness_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env(
            kali_runtime::BROWSER_HARNESS_COMMAND_ENV,
            "definitely-missing-browser-harness --flag",
        )
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
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);

    let browser_harness = &json["payload"]["browserHarness"];
    assert_eq!(
        browser_harness["envVar"],
        kali_runtime::BROWSER_HARNESS_COMMAND_ENV
    );
    assert_eq!(browser_harness["source"], "env");
    assert_eq!(
        browser_harness["override"],
        "definitely-missing-browser-harness --flag"
    );
    assert_eq!(
        browser_harness["command"],
        json!(["definitely-missing-browser-harness", "--flag"])
    );
    assert_eq!(
        browser_harness["executable"],
        "definitely-missing-browser-harness"
    );
    assert_eq!(browser_harness["args"], json!(["--flag"]));
    assert_eq!(browser_harness["executableAvailable"], false);

    let browser_runtime_contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(browser_runtime_contract["hostLabel"], "browser-requested");
    assert_eq!(
        browser_runtime_contract["hostDescription"],
        "real browser host"
    );
    assert_eq!(
        browser_runtime_contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        browser_runtime_contract["supportedCommands"],
        json!(["run", "test"])
    );
    assert_eq!(
        browser_runtime_contract["diagnosticHint"],
        json!(kali_runtime::BrowserRuntimeContract::diagnostic_hint())
    );
    assert_eq!(browser_runtime_contract["diagnosticNotes"], json!([
        "supported browser runtime commands: run, test",
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
        "browser runtime host description: real browser host"
    ]));
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn doctor_emits_pretty_json_envelope_for_browser_harness_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("--pretty")
        .arg("doctor")
        .env(
            kali_runtime::BROWSER_HARNESS_COMMAND_ENV,
            "definitely-missing-browser-harness --flag",
        )
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("{\n"), "stdout: {stdout}");
    assert!(
        stdout.contains("\n    \"browserRuntimeContract\""),
        "stdout: {stdout}"
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);

    let browser_runtime_contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(
        browser_runtime_contract["diagnosticHint"],
        json!(kali_runtime::BrowserRuntimeContract::diagnostic_hint())
    );
    assert_eq!(browser_runtime_contract["hostLabel"], "browser-requested");
    assert_eq!(
        browser_runtime_contract["hostDescription"],
        "real browser host"
    );
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn doctor_emits_human_output_for_browser_harness_override() {
    let output = Command::new(kali_bin())
        .arg("doctor")
        .env(
            kali_runtime::BROWSER_HARNESS_COMMAND_ENV,
            "definitely-missing-browser-harness --flag",
        )
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Browser harness:"), "stdout: {stdout}");
    assert!(
        stdout.contains("env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("source: env"), "stdout: {stdout}");
    assert!(
        stdout.contains("override: definitely-missing-browser-harness --flag"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("command: definitely-missing-browser-harness --flag"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("executable available: false"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("Browser runtime contract:"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("host label: browser-requested"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("host description: real browser host"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("supported commands: run, test"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("diagnostic hint: Use the Phase-1 browser-targeted command set"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("browser runtime host description: real browser host"),
        "stdout: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn threaded_runtime_globals_accept_on_default_standalone_surface() {
    for inherited in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            "SharedArrayBuffer; globalThis['SharedArrayBuffer']; Atomics; globalThis['Atomics']; console.log('threaded globals ok');\n",
        )
        .expect("write source");
        fs::write(
            &test_path,
            "Kali.test('threaded globals', () => { SharedArrayBuffer; globalThis['SharedArrayBuffer']; Atomics; globalThis['Atomics']; console.log('threaded globals ok'); });\n",
        )
        .expect("write test source");

        if inherited {
            fs::write(
                dir.path().join("kali.json"),
                r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
            )
            .expect("write manifest");
        }

        for command in ["check", "build", "run", "test"] {
            let input_path = if command == "test" {
                &test_path
            } else {
                &source_path
            };

            let mut cli_command = Command::new(kali_bin());
            cli_command.current_dir(dir.path()).arg(command);
            if !inherited {
                cli_command.arg("--wasm-threads");
            }
            cli_command.arg(input_path);

            let output = cli_command.output().expect("run kali");
            assert!(
                output.status.success(),
                "{command} should accept threaded globals on the default standalone surface (inherited={inherited})\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            match command {
                "check" => assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}"),
                "build" => {
                    assert!(
                        stdout.contains("Built executable artifact at"),
                        "stdout: {stdout}"
                    );
                    assert!(
                        source_path.with_file_name("main.wasm").exists(),
                        "expected build artifact"
                    );
                }
                "run" | "test" => {
                    assert!(stdout.contains("threaded globals ok"), "stdout: {stdout}")
                }
                _ => unreachable!("unexpected command"),
            }
        }
    }
}

#[test]
fn smoke_supports_late_object_model_own_property_helpers_in_js_input() {
    for (command, source_name) in [
        ("check", "main.js"),
        ("build", "main.js"),
        ("run", "main.js"),
        ("test", "smoke.test.js"),
    ] {
        for json_mode in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, late_object_model_own_property_source()).expect("write source");

            let mut cli = Command::new(kali_bin());
            cli.current_dir(dir.path());
            if json_mode {
                cli.arg("--output").arg("json");
            }
            cli.arg(command).arg(&source_path);
            let output = cli.output().expect("run kali");

            assert!(output.status.success(), "{command} unexpectedly failed");
            assert_eq!(output.status.code(), Some(0));

            if json_mode {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
        }
    }
}

#[test]
fn fmt_check_reports_drift_without_rewriting() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function add(a,b){return a+b;}").expect("write source");

    let output = Command::new(kali_bin())
        .arg("fmt")
        .arg("--check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would format 1 file(s)"),
        "stdout: {stdout}"
    );
    let contents = fs::read_to_string(&source_path).expect("read source");
    assert_eq!(contents, "function add(a,b){return a+b;}");
}

#[test]
fn fmt_rejects_sandbox_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("fmt")
        .arg("--sandbox")
        .arg("kali.policy.json")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn fmt_rejects_api_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("fmt")
        .arg("--api")
        .arg("browser")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn lint_fix_applies_structured_safe_fixes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "var x = 1; debugger; if (x == 1) { }").expect("write source");

    let output = Command::new(kali_bin())
        .arg("lint")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let contents = fs::read_to_string(&source_path).expect("read source");
    assert!(contents.contains("let x = 1;"));
    assert!(contents.contains("==="));
    assert!(!contents.contains("debugger"));
}

#[test]
fn lint_rejects_sandbox_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("lint")
        .arg("--sandbox")
        .arg("kali.policy.json")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn lint_rejects_api_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("lint")
        .arg("--api")
        .arg("browser")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn init_scaffolds_application_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("main.ts").exists());

    let manifest = fs::read_to_string(dir.path().join("kali.json")).expect("manifest");
    assert!(
        manifest.contains("\"schemaVersion\": 1"),
        "manifest: {manifest}"
    );
    let source = fs::read_to_string(dir.path().join("main.ts")).expect("source");
    assert!(source.contains("Hello, world!"), "source: {source}");
}

#[test]
fn init_scaffolds_nested_child_project() {
    let parent = tempdir().expect("tempdir");
    fs::write(parent.path().join("kali.json"), "{}\n").expect("parent manifest");

    let child = parent.path().join("nested");
    fs::create_dir(&child).expect("child dir");

    let output = Command::new(kali_bin())
        .current_dir(&child)
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(child.join("kali.json").exists());
    assert!(child.join("main.ts").exists());
    assert!(parent.path().join("kali.json").exists());

    let manifest = fs::read_to_string(child.join("kali.json")).expect("manifest");
    assert!(
        manifest.contains("\"schemaVersion\": 1"),
        "manifest: {manifest}"
    );
    let source = fs::read_to_string(child.join("main.ts")).expect("source");
    assert!(source.contains("Hello, world!"), "source: {source}");
}

#[test]
fn init_scaffolds_library_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .arg("--lib")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("lib.ts").exists());

    let source = fs::read_to_string(dir.path().join("lib.ts")).expect("source");
    assert!(source.contains("export function add"), "source: {source}");
}

#[test]
fn browser_bundle_js_exposes_runtime_dynamic_import_loader() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import(\"./\" + \"lazy.ts\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(js.contains("loadDynamicImport"), "bundle js: {js}");
    assert!(js.contains("loadWithImports"), "bundle js: {js}");
    assert!(js.contains("lazy.ts"), "bundle js: {js}");

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.ts");
}

#[test]
fn browser_bundle_js_exposes_runtime_dynamic_import_loader_for_directory_index_targets() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_dir = dir.path().join("lazy");
    fs::create_dir(&chunk_dir).expect("create chunk dir");
    fs::write(
        chunk_dir.join("index.ts"),
        "export function lazyValue() { return 7; }",
    )
    .expect("write chunk source");
    fs::write(
        &source_path,
        "const lazy = import(\"./lazy\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(js.contains("loadDynamicImport"), "bundle js: {js}");
    assert!(js.contains("loadWithImports"), "bundle js: {js}");
    assert!(js.contains("lazy"), "bundle js: {js}");

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy");
}

#[test]
fn browser_bundle_normalizes_runtime_dynamic_import_specifiers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import((\"./\" + \"lazy.ts\"));\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./sub/../lazy.ts");
}

#[test]
fn browser_bundle_normalizes_runtime_dynamic_import_specifiers_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const lazy = import((\"./\" + \"lazy.js\"));\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./sub/../lazy.js");
}

#[test]
fn browser_bundle_js_exposes_runtime_dynamic_import_loader_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const lazy = import(\"./\" + \"lazy.js\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(js.contains("loadDynamicImport"), "bundle js: {js}");
    assert!(js.contains("loadWithImports"), "bundle js: {js}");
    assert!(js.contains("lazy.js"), "bundle js: {js}");

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.js");
}

#[test]
fn browser_bundle_js_exposes_runtime_dynamic_import_loader_for_directory_index_targets_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_dir = dir.path().join("lazy");
    fs::create_dir(&chunk_dir).expect("create chunk dir");
    fs::write(
        chunk_dir.join("index.js"),
        "export function lazyValue() { return 7; }",
    )
    .expect("write chunk source");
    fs::write(
        &source_path,
        "const lazy = import(\"./lazy\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(js.contains("loadDynamicImport"), "bundle js: {js}");
    assert!(js.contains("loadWithImports"), "bundle js: {js}");
    assert!(js.contains("lazy"), "bundle js: {js}");

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy");
}

#[test]
fn release_build_constant_folds_literal_expressions() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function main() { return 1 + 2 + 3; } main();",
    )
    .expect("write source");

    let fast_dir = dir.path().join("fast");
    let fast_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--fast")
        .arg("--out-dir")
        .arg(&fast_dir)
        .arg(&source_path)
        .output()
        .expect("run kali fast build");
    assert!(
        fast_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&fast_output.stdout),
        String::from_utf8_lossy(&fast_output.stderr)
    );

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let fast_wasm = fs::read(fast_dir.join("math.wasm")).expect("read fast wasm");
    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let fast_adds = count_i64_adds(&fast_wasm);
    let release_adds = count_i64_adds(&release_wasm);

    assert!(
        release_adds < fast_adds,
        "expected release build to reduce add instructions (fast={fast_adds}, release={release_adds})"
    );
}

#[test]
fn release_hot_paths_stay_unboxed_without_tag_checks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function hot(a, b) { return a + b; } hot(1, 2);",
    )
    .expect("write source");

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let release_adds = count_i64_adds(&release_wasm);
    let release_tag_ops = count_tag_boxing_ops(&release_wasm);

    assert!(
        release_adds > 0,
        "expected a numeric hot path in the optimized wasm"
    );
    assert_eq!(
        release_tag_ops, 0,
        "expected the specialized hot path to avoid tag-check / untag boxing ops"
    );
}

#[test]
fn optimization_benchmark_suite_tracks_compile_time_size_and_speed() {
    for (fixture_stem, benchmark_name) in [
        ("math-benchmark-v1", "folded-arithmetic"),
        ("math-benchmark-v1-js", "folded-arithmetic-js"),
        ("math-trunc-benchmark-v1", "math-trunc-builtin"),
        ("math-imul-benchmark-v1", "math-imul-builtin"),
        ("math-imul-benchmark-v1-js", "math-imul-builtin-js"),
        ("math-clz32-benchmark-v1", "math-clz32-builtin"),
        ("math-clz32-benchmark-v1-js", "math-clz32-builtin-js"),
        ("math-ceil-benchmark-v1", "math-ceil-builtin"),
        ("math-abs-sign-benchmark-v1", "math-abs-sign-builtin"),
        ("math-abs-sign-benchmark-v1-js", "math-abs-sign-builtin-js"),
        ("math-max-min-benchmark-v1", "math-max-min-builtin"),
        ("math-max-min-benchmark-v1-js", "math-max-min-builtin-js"),
        ("math-floor-benchmark-v1", "math-floor-builtin"),
        ("math-floor-benchmark-v1-js", "math-floor-builtin-js"),
        ("math-round-benchmark-v1", "math-round-builtin"),
        ("math-round-benchmark-v1-js", "math-round-builtin-js"),
        ("math-pow-benchmark-v1", "math-pow-builtin"),
        ("math-pow-benchmark-v1-js", "math-pow-builtin-js"),
        (
            "division-by-one-benchmark-v1",
            "division-by-one-elimination",
        ),
        (
            "multiplication-by-one-benchmark-v1",
            "multiplication-by-one-elimination",
        ),
        (
            "dead-branch-elimination-benchmark-v1",
            "dead-branch-elimination",
        ),
        (
            "dead-inlined-function-pruning-benchmark-v1",
            "dead-inlined-function-pruning",
        ),
        ("call-inlining-benchmark-v1", "division-and-identity"),
        (
            "closure-inlining-benchmark-v1",
            "closure-inlining-and-folding",
        ),
        (
            "nested-call-inlining-chain-benchmark-v1",
            "nested-call-inlining-chain",
        ),
        (
            "object-enumeration-benchmark-v1",
            "object-enumeration-folding",
        ),
        (
            "object-string-enumeration-benchmark-v1",
            "object-string-enumeration-folding",
        ),
        ("reflect-own-keys-benchmark-v1", "reflect-own-keys-folding"),
        (
            "reflect-own-keys-const-bound-literal-benchmark-v1",
            "reflect-own-keys-const-bound-literal",
        ),
        (
            "reflect-own-keys-alias-chain-benchmark-v1",
            "reflect-own-keys-alias-chain",
        ),
        (
            "integer-like-object-enumeration-benchmark-v1",
            "integer-like-object-enumeration-folding",
        ),
        (
            "object-enumeration-alias-chain-benchmark-v1",
            "object-enumeration-alias-chain",
        ),
        (
            "object-enumeration-alias-chain-benchmark-v1-js",
            "object-enumeration-alias-chain-js",
        ),
        (
            "object-enumeration-const-bound-literal-benchmark-v1",
            "object-enumeration-const-bound-literal",
        ),
        (
            "object-enumeration-delete-reinsert-benchmark-v1",
            "object-enumeration-delete-reinsert",
        ),
        (
            "object-literal-property-order-canonicalization-benchmark-v1",
            "object-literal-property-order-canonicalization",
        ),
        (
            "object-literal-property-order-canonicalization-benchmark-v1-js",
            "object-literal-property-order-canonicalization-js",
        ),
        (
            "identity-chain-benchmark-v1",
            "identity-chain-and-simplification",
        ),
        (
            "nested-wrapper-pruning-benchmark-v1",
            "nested-wrapper-pruning",
        ),
        (
            "algebraic-simplification-benchmark-v1",
            "algebraic-simplification",
        ),
        (
            "duplicate-pure-expression-elimination-benchmark-v1",
            "duplicate-pure-expression-elimination",
        ),
        (
            "nullish-specialization-repeat-benchmark-v1",
            "nullish-specialization-repeat",
        ),
        ("specialization-reuse-benchmark-v1", "specialization-reuse"),
        (
            "bigint-literal-arguments-benchmark-v1",
            "bigint-literal-arguments",
        ),
        (
            "bigint-addition-chain-benchmark-v1",
            "bigint-addition-chain",
        ),
        (
            "bigint-multiplication-chain-benchmark-v1",
            "bigint-multiplication-chain",
        ),
        (
            "numeric-literal-arguments-benchmark-v1",
            "numeric-literal-arguments",
        ),
        (
            "boolean-literal-arguments-benchmark-v1",
            "boolean-literal-arguments",
        ),
        (
            "branch-specialization-repeat-benchmark-v1",
            "branch-specialization-repeat",
        ),
        (
            "const-array-element-access-benchmark-v1",
            "const-array-element-access",
        ),
        (
            "const-object-property-access-benchmark-v1",
            "const-object-property-access",
        ),
        ("math-variant-benchmark-v1", "folded-arithmetic-variant"),
        (
            "math-variant-benchmark-v1-js",
            "folded-arithmetic-variant-js",
        ),
        ("string-concatenation-benchmark-v1", "string-concatenation"),
        (
            "array-literal-arguments-benchmark-v1",
            "array-literal-arguments",
        ),
        (
            "template-literal-concatenation-benchmark-v1",
            "template-literal-concatenation",
        ),
        (
            "template-literal-concatenation-benchmark-v1-js",
            "template-literal-concatenation-js",
        ),
        (
            "layout-specialization-benchmark-v1",
            "layout-specialization",
        ),
        ("call-inlining-chain-benchmark-v1", "call-inlining-chain"),
        ("nullish-benchmark-v1", "nullish-specialization"),
        ("spectral-norm-benchmark-v1", "spectral-norm"),
        ("nbody-benchmark-v1", "nbody"),
    ] {
        assert_optimization_benchmark_fixture(fixture_stem, benchmark_name);
    }
}

#[test]
fn release_advanced_strengthens_algebraic_simplification() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function addZero(x) { return x + 0; } addZero(1);",
    )
    .expect("write source");

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let advanced_dir = dir.path().join("advanced");
    let advanced_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release-advanced")
        .arg("--out-dir")
        .arg(&advanced_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release-advanced build");
    assert!(
        advanced_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&advanced_output.stdout),
        String::from_utf8_lossy(&advanced_output.stderr)
    );

    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let advanced_wasm = fs::read(advanced_dir.join("math.wasm")).expect("read advanced wasm");
    let release_adds = count_i64_adds(&release_wasm);
    let advanced_adds = count_i64_adds(&advanced_wasm);

    assert!(
        advanced_adds < release_adds,
        "expected release-advanced build to reduce add instructions further (release={release_adds}, advanced={advanced_adds})"
    );
}

#[test]
fn node_cross_module_inference_stays_within_the_phase_3_budget() {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        &math_path,
        r#"export function double(value) {
    return value + value;
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { double } from './math.ts';

export function quadruple(value) {
    return double(double(value));
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { quadruple } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { quadruple } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { quadruple } from './public.ts';

console.log(quadruple(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn node_cross_module_inference_stays_within_the_phase_3_budget_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.js");
    let helper_path = dir.path().join("helper.js");
    let bridge_path = dir.path().join("bridge.js");
    let public_path = dir.path().join("public.js");
    let source_path = dir.path().join("main.js");

    fs::write(
        &math_path,
        r#"export function double(value) {
    return value + value;
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { double } from './math.js';

export function quadruple(value) {
    return double(double(value));
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { quadruple } from './helper.js';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { quadruple } from './bridge.js';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { quadruple } from './public.js';

console.log(quadruple(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn node_cross_module_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget()
{
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &math_path,
        r#"export function makePair(value) {
    return { left: value, right: value + value };
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { makePair } from './math.ts';

export function projectLeft(value) {
    return makePair(value).left;
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectLeft } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectLeft } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectLeft } from './public.ts';

console.log(projectLeft(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn cross_module_higher_order_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget(
) {
    let dir = tempdir().expect("tempdir");
    let factory_path = dir.path().join("factory.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &factory_path,
        r#"export function makeProjector(value) {
    return function project() {
        return value + value;
    };
}
"#,
    )
    .expect("write factory module");
    fs::write(
        &helper_path,
        r#"import { makeProjector } from './factory.ts';

export function projectValue(value) {
    const project = makeProjector(value);
    return project();
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectValue } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectValue } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectValue } from './public.ts';

console.log(projectValue(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn node_cross_module_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.js");
    let helper_path = dir.path().join("helper.js");
    let bridge_path = dir.path().join("bridge.js");
    let public_path = dir.path().join("public.js");
    let source_path = dir.path().join("main.js");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &math_path,
        r#"export function makePair(value) {
    return { left: value, right: value + value };
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { makePair } from './math.js';

export function projectLeft(value) {
    return makePair(value).left;
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectLeft } from './helper.js';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectLeft } from './bridge.js';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectLeft } from './public.js';

console.log(projectLeft(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn default_standalone_cross_module_inference_stays_within_the_phase_3_budget_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.js");
    let helper_path = dir.path().join("helper.js");
    let bridge_path = dir.path().join("bridge.js");
    let public_path = dir.path().join("public.js");
    let source_path = dir.path().join("main.js");

    fs::write(
        &math_path,
        r#"export function double(value) {
    return value + value;
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { double } from './math.js';

export function quadruple(value) {
    return double(double(value));
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { quadruple } from './helper.js';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { quadruple } from './bridge.js';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { quadruple } from './public.js';

console.log(quadruple(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn default_standalone_cross_module_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.js");
    let helper_path = dir.path().join("helper.js");
    let bridge_path = dir.path().join("bridge.js");
    let public_path = dir.path().join("public.js");
    let source_path = dir.path().join("main.js");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &math_path,
        r#"export function makePair(value) {
    return { left: value, right: value + value };
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { makePair } from './math.js';

export function projectLeft(value) {
    return makePair(value).left;
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectLeft } from './helper.js';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectLeft } from './bridge.js';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectLeft } from './public.js';

console.log(projectLeft(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn json_init_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let root = dir.path().to_string_lossy().into_owned();
    let manifest_path = dir.path().join("kali.json").to_string_lossy().into_owned();
    let source_path = dir.path().join("main.ts").to_string_lossy().into_owned();
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["root"], root);
    assert_eq!(json["payload"]["manifestPath"], manifest_path);
    assert_eq!(json["payload"]["sourcePath"], source_path);
    assert_eq!(json["payload"]["library"], false);
    assert_eq!(json["exitCode"], 0);
}

#[test]
fn json_fmt_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function add(a,b){return a+b;}").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("fmt")
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
    assert_eq!(json["command"], "fmt");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(
        json["payload"],
        json!({"filesChecked": 1, "filesFormatted": 1})
    );
}

#[test]
fn json_lint_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const x = 1; x;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("lint")
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
    assert_eq!(json["command"], "lint");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(
        json["payload"],
        json!({"filesLinted": 1, "errorCount": 0, "warningCount": 0, "fixedCount": 0})
    );
}

#[test]
fn pretty_without_json_exits_with_usage_code() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--pretty")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
}

#[test]
fn verbose_pretty_without_json_includes_error_docs_link() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--verbose")
        .arg("--pretty")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("https://kali-lang.org/errors/E5508"),
        "stderr: {stderr}"
    );
}

#[test]
fn init_rejects_non_empty_directory_with_usage_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("notes.txt"), "keep me").expect("write file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
}

#[test]
fn init_rejects_sandbox_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .arg("--sandbox")
        .arg("kali.policy.json")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn init_rejects_api_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .arg("--api")
        .arg("browser")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn regression_package_bin_entrypoints_requiring_package_json_still_fail_on_default_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "1.0.0",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/semver.js"),
        "#!/usr/bin/env node\nconst pkg = require('../package.json');\nconsole.log(pkg.version);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("npm package bin 'semver'"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("CommonJS require()"), "stderr: {stderr}");
}

#[test]
fn global_pretty_without_json_output_reports_canonical_cli_usage_error() {
    let output = Command::new(kali_bin())
        .arg("--pretty")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("`--pretty` is only meaningful when JSON output is active"),
        "stderr: {stderr}"
    );
}
