use super::*;

#[test]
fn json_run_accepts_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(globalThis.Deno.pid);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_deno_args_length_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno.args.length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "0");
}

#[test]
fn run_supports_deno_chdir_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "Deno.chdir('nested'); console.log(1);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
}

#[test]
fn json_run_accepts_bracketed_global_this_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"pid\"]);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_accepts_bracketed_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"pid\"]);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(globalThis.Deno.pid);\n").expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "{command} failed: {:?}", output);
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().parse::<u32>().is_ok(), "stdout: {stdout}");
}

#[test]
fn json_run_accepts_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(
        json["stdout"].as_str().expect("stdout").trim(),
        "hello-environment"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_accepts_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno.pid);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_accepts_bracketed_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno[\"pid\"]);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_rejects_bracketed_deno_pid_in_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_harness_bracketed_deno_pid_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "run unexpectedly succeeded: {:?}",
        output
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Deno.pid"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Deno.pid"), "stderr: {stderr}");
}

#[test]
fn json_run_rejects_bracketed_deno_pid_in_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_harness_bracketed_deno_pid_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "run unexpectedly succeeded: {:?}",
        output
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 1);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert!(
        errors.iter().any(|error| error["code"] == "E5506"),
        "errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("Deno.pid")),
        "errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("globalThis.Deno.pid")),
        "errors: {errors:?}"
    );
}

#[test]
fn json_run_supports_web_baseline_structured_clone_and_event_primitives_in_ts_and_js_input() {
    let dir = tempdir().expect("tempdir");

    for ext in ["ts", "js"] {
        let source_path = dir.path().join(format!("web-baseline-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(false),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Flipped pin: `instanceof` is rejected fail-closed (E5506): kali has no prototype
        // chain; the token was previously dropped so the expression miscompiled
        // to its left operand.
        assert!(
            !output.status.success(),
            "must be rejected fail-closed: {output:?}"
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["success"], false);
        assert_eq!(json["errors"][0]["code"], "E5506");
    }
}

#[test]
fn json_run_supports_web_baseline_structured_clone_and_event_primitives_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    let dir = tempdir().expect("tempdir");

    for ext in ["ts", "js"] {
        let source_path = dir.path().join(format!("browser-web-baseline-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(false),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Flipped pin: `instanceof` is rejected fail-closed (E5506): kali has no prototype
        // chain; the token was previously dropped so the expression miscompiled
        // to its left operand.
        assert!(
            !output.status.success(),
            "must be rejected fail-closed: {output:?}"
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["success"], false);
        assert_eq!(json["errors"][0]["code"], "E5506");
    }
}

#[test]
fn json_run_supports_web_baseline_structured_clone_and_event_primitives_when_browser_harness_is_configured_with_inherited_browser_api_surface_in_ts_and_js_input(
) {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    for ext in ["ts", "js"] {
        let source_path = dir
            .path()
            .join(format!("browser-web-baseline-inherited-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(false),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Flipped pin: `instanceof` is rejected fail-closed (E5506): kali has no prototype
        // chain; the token was previously dropped so the expression miscompiled
        // to its left operand.
        assert!(
            !output.status.success(),
            "must be rejected fail-closed: {output:?}"
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["success"], false);
        assert_eq!(json["errors"][0]["code"], "E5506");
    }
}

#[test]
fn run_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn run_rejects_threaded_runtime_globals_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn run_rejects_broader_intl_support() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Intl"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Intl"), "stderr: {stderr}");
    assert!(stderr.contains(r#"globalThis["Intl"]"#), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.Intl.NumberFormat"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Intl"]["NumberFormat"]"#),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Intl"]["DateTimeFormat"]"#),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("globalThis.Intl.Collator"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("globalThis.Intl.Locale"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Intl"]["Collator"]"#),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Intl"]["Locale"]"#),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_broader_intl_support_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| message.contains("Intl")));
    assert!(messages
        .iter()
        .any(|message| message.contains("globalThis.Intl")));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Intl"]"#)));
    assert!(messages
        .iter()
        .any(|message| message.contains("globalThis.Intl.NumberFormat")));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Intl"]["NumberFormat"]"#)));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Intl"]["DateTimeFormat"]"#)));
    assert!(messages
        .iter()
        .any(|message| message.contains("globalThis.Intl.Collator")));
    assert!(messages
        .iter()
        .any(|message| message.contains("globalThis.Intl.Locale")));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Intl"]["Collator"]"#)));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Intl"]["Locale"]"#)));
}

#[test]
fn run_rejects_late_process_control_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.kill",
        r#"globalThis.process.kill"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"]["kill"]"#,
        "process.exit",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn run_rejects_late_process_control_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 12, "unexpected errors: {errors:?}");
    assert!(errors
        .iter()
        .all(|error| { matches!(error["code"].as_str(), Some("E5506") | Some("E3100")) }));
    assert!(errors.iter().any(|error| error["code"] == "E5506"));
    assert!(errors.iter().any(|error| error["code"] == "E3100"));
    for expected in [
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.kill",
        r#"globalThis.process.kill"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"]["kill"]"#,
        "process.exit",
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
fn run_rejects_late_object_model_revocable_calls() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); globalThis[\"Proxy\"].revocable({}, {}); globalThis.Proxy[\"revocable\"]({}, {});",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Proxy.revocable"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.Proxy.revocable"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Proxy"]["revocable"]"#),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_frozen_late_object_model_revocable_calls() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis[\"Proxy\"][\"revocable\"])({}, {}); Object.freeze((globalThis[\"Proxy\"][\"revocable\"]))({}, {}); Object.freeze(globalThis[\"Proxy\"].revocable)({}, {}); Object.freeze(globalThis.Proxy[\"revocable\"])({}, {}); Object.freeze(globalThis?.Proxy.revocable)({}, {}); Object.freeze((globalThis?.Proxy.revocable))({}, {});",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Proxy.revocable"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.Proxy.revocable"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Proxy"]["revocable"]"#),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_late_object_model_revocable_calls_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); globalThis[\"Proxy\"].revocable({}, {}); globalThis.Proxy[\"revocable\"]({}, {}); Object.freeze(globalThis?.Proxy.revocable)({}, {}); Object.freeze((globalThis?.Proxy.revocable))({}, {});",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 5);
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("Proxy.revocable")));
    assert!(messages
        .iter()
        .any(|message| message.contains("globalThis.Proxy.revocable")));
    assert!(messages
        .iter()
        .any(|message| message.contains(r#"globalThis["Proxy"]["revocable"]"#)));
}

#[test]
fn run_rejects_late_object_model_revocable_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); globalThis[\"Proxy\"].revocable({}, {}); globalThis.Proxy[\"revocable\"]({}, {});",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Proxy.revocable"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.Proxy.revocable"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"globalThis["Proxy"]["revocable"]"#),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn run_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 30, "unexpected errors: {errors:?}");
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn run_executes_the_hello_fixture() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_the_explicit_deno_api_surface() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("deno")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_the_browser_api_surface_when_a_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser run"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_the_browser_api_surface_when_a_harness_command_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('browser run');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser run"),
        "json: {json}"
    );
}

#[test]
fn run_supports_unary_prefix_semantics_when_browser_harness_is_configured() {
    assert_browser_requested_unary_prefix_semantics("run", "main.ts", false);
}

#[test]
fn run_supports_unary_prefix_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_unary_prefix_semantics("run", "main.js", false);
}

#[test]
fn json_run_supports_unary_prefix_semantics_when_browser_harness_is_configured() {
    assert_browser_requested_unary_prefix_semantics("run", "main.ts", true);
}

#[test]
fn json_run_supports_unary_prefix_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_unary_prefix_semantics("run", "main.js", true);
}

#[test]
fn run_supports_unary_prefix_semantics_when_browser_api_surface_is_inherited_in_ts_and_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_unary_prefix_semantics_with_inherited_browser_api_surface(
        "run", false,
    );
}

#[test]
fn json_run_supports_unary_prefix_semantics_when_browser_api_surface_is_inherited_in_ts_and_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_unary_prefix_semantics_with_inherited_browser_api_surface("run", true);
}

#[test]
fn run_supports_async_await_sequencing_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
}

#[test]
fn run_supports_async_await_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
}

#[test]
fn json_run_supports_async_await_sequencing_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
  console.log('async ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("async ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_async_await_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
  console.log('async ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("async ok"),
        "json: {json}"
    );
}

#[cfg(unix)]
#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.js");
    fs::write(&source_path, "console.log('browser unreadable summary');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("browser unreadable summary\n");'"#,
        )
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("browser unreadable summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[cfg(unix)]
#[test]
fn run_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.ts");
    fs::write(&source_path, "console.log('browser unreadable summary');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("browser unreadable summary\n");'"#,
        )
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("browser unreadable summary"),
        "stdout: {stdout}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.js");
    fs::write(
        &source_path,
        "console.log('browser invalid tests array items');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"stdout\"],\"tests\":[1],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid tests array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":8"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.ts");
    fs::write(
        &source_path,
        "console.log('browser invalid tests array items');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"stdout\"],\"tests\":[1],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid tests array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":8"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.js");
    fs::write(&source_path, "console.log('browser unparseable');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unparseable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser unparseable"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_invalid_labels_when_browser_harness_is_configured_in_js_input_legacy(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels.js");
    fs::write(&source_path, "console.log('browser invalid labels');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"hostContract\":\"browser-requested\""),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_invalid_labels_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels.ts");
    fs::write(&source_path, "console.log('browser invalid labels');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"hostContract\":\"browser-requested\""),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_invalid_labels_and_is_missing_tests_failed_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-missing-tests-failed.ts");
    fs::write(&source_path, "console.log('browser invalid labels');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":9"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_invalid_labels_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels.js");
    fs::write(&source_path, "console.log('browser invalid labels');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"hostContract\":\"browser-requested\""),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_invalid_labels_and_is_missing_tests_failed_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-missing-tests-failed.js");
    fs::write(&source_path, "console.log('browser invalid labels');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":9"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_uses_stdout_metadata_when_browser_summary_file_has_unexpected_top_level_keys_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unexpected-top-level-keys.js");
    fs::write(
        &source_path,
        "console.log('browser unexpected top-level keys');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser unexpected top-level keys\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"unexpected\":true}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unexpected top-level keys\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser unexpected top-level keys"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_array_items_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-array-items.js");
    fs::write(
        &source_path,
        "console.log('browser invalid array items');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid array items\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"args\":[\"stdout\"]"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_non_integer_tests_failed_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("non-integer-summary.ts");
    fs::write(&source_path, "console.log('browser invalid summary');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":1.5,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":7"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_negative_tests_failed_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("negative-tests-failed.ts");
    fs::write(&source_path, "console.log('browser invalid summary');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":-1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":7"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_args_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-args.ts");
    fs::write(&source_path, "console.log('browser invalid args');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid args\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"args\":[\"stdout\"]"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_args_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-args.js");
    fs::write(&source_path, "console.log('browser invalid args');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid args\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"args\":[\"stdout\"]"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_and_invalid_args_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-and-args.ts");
    fs::write(
        &source_path,
        "console.log('browser invalid labels and args');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"args\":[\"stdout\"]"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_and_invalid_args_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-and-args.js");
    fs::write(
        &source_path,
        "console.log('browser invalid labels and args');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"args\":[\"stdout\"]"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_object_enumeration_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
obj["a"] = 3;
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 3 ||
  values[1] !== 2
) {
  throw 'unexpected overwrite ordering';
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw 'unexpected delete-reinsert ordering';
}
const stringKeys = Object.keys('abc');
const stringEntries = Object.entries('ab');
const stringValues = Object.values('ab');
if (
  stringKeys.length !== 3 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  stringKeys[2] !== '2' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
obj["a"] = 3;
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 3 ||
  values[1] !== 2
) {
  throw 'unexpected overwrite ordering';
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw 'unexpected delete-reinsert ordering';
}
const stringKeys = Object.keys('abc');
const stringEntries = Object.entries('ab');
const stringValues = Object.values('ab');
if (
  stringKeys.length !== 3 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  stringKeys[2] !== '2' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
obj["a"] = 3;
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 3 ||
  values[1] !== 2
) {
  throw 'unexpected overwrite ordering';
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw 'unexpected delete-reinsert ordering';
}
const stringKeys = Object.keys('abc');
const stringEntries = Object.entries('ab');
const stringValues = Object.values('ab');
if (
  stringKeys.length !== 3 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  stringKeys[2] !== '2' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_string_primitive_enumeration_semantics_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const stringKeys = Object.keys('abc');
const stringEntries = Object.entries('ab');
const stringValues = Object.values('ab');
if (
  stringKeys.length !== 3 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  stringKeys[2] !== '2' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
console.log(stringKeys.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
obj["a"] = 3;
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 3 ||
  values[1] !== 2
) {
  throw 'unexpected overwrite ordering';
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw 'unexpected delete-reinsert ordering';
}
const stringKeys = Object.keys('abc');
const stringEntries = Object.entries('ab');
const stringValues = Object.values('ab');
if (
  stringKeys.length !== 3 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  stringKeys[2] !== '2' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_enumeration_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_enumeration_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_array_from_iteration_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_array_from_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\n1\n2\n1\n2\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_json_run_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_input_when_a_browser_harness_command_is_configured(extension);
    }
}

#[test]
fn run_supports_boolean_logic_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\n3\n4\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\n3\n4\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n3\n4\n"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\n3\n4\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_boolean_logic_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\n3\n4\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_strict_equality_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("2\n"), "json: {json}");
}

#[test]
fn run_supports_strict_equality_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("2\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_strict_equality_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("2\n"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_strict_equality_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("2\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_strict_equality_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("2\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_math_suite_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (1 === 1) {
  console.log(1);
} else {
  console.log(0);
}
if ('a' === 'a') {
  console.log(2);
} else {
  console.log(0);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("2\n"), "json: {json}");
}

#[test]
fn run_supports_math_suite_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\nconsole.log(Math.clz32(1));\nconsole.log(Math.clz32());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert!(stdout.contains("31\n"), "json: {json}");
    assert!(stdout.contains("32\n"), "json: {json}");
}

#[test]
fn json_run_supports_math_suite_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\nconsole.log(Math.clz32(1));\nconsole.log(Math.clz32());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert!(stdout.contains("31\n"), "json: {json}");
    assert!(stdout.contains("32\n"), "json: {json}");
}

#[test]
fn run_supports_math_max_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains(
            "3
"
        ),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_min_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.min(3, 2, 1));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_abs_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.abs(3 - 6));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_sign_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.sign(3 - 6));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-1\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_sign_fractional_literal_semantics_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sign(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_hypot_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_supports_math_hypot_semantics("run", "main.js", "5\n", true);
}

#[test]
fn run_supports_math_hypot_semantics_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_supports_math_hypot_semantics("run", "main.jsx", "5\n", true);
}

#[test]
fn run_supports_math_hypot_semantics_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_supports_math_hypot_semantics("run", "main.tsx", "5\n", true);
}

#[test]
fn run_supports_math_imul_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-2\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_imul_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-2\n"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_zero_budget_pair_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.ts");
    fs::write(&source_path, "console.log(0);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_suite_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\nconsole.log(Math.clz32(1));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert!(stdout.contains("31\n"), "json: {json}");
}

#[test]
fn run_supports_math_clz32_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_clz32_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_try_finally_sequencing_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"try {
  0;
} finally {
  console.log(2);
}
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_math_ceil_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_ceil_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_ceil_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_ceil_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_try_finally_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"function main() {
  try {
    for (const value of [1, 2]) {
      console.log(value);
      break;
    }
  } finally {
    console.log(3);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_try_catch_exception_handling_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"let caught = false;
try {
  throw 'boom';
} catch {
  caught = true;
}
if (!caught) {
  throw new Error('catch did not run');
}
console.log(2);
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_try_catch_exception_handling_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"let caught = false;
try {
  throw 'boom';
} catch {
  caught = true;
}
if (!caught) {
  throw new Error('catch did not run');
}
console.log(2);
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_try_catch_and_finally_sequencing_when_browser_harness_is_configured_with_inherited_browser_api_surface(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.ts");
    fs::write(&source_path, browser_runtime_try_catch_and_finally_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_try_catch_and_finally_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.js");
    fs::write(&source_path, browser_runtime_try_catch_and_finally_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_queue_microtask_ordering_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  let microtaskRan = false;
  queueMicrotask(() => {
    microtaskRan = true;
  });
  if (microtaskRan) {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve();
  if (!microtaskRan) {
    throw new Error('microtask did not run before the next turn');
  }
  console.log('queueMicrotask ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_queue_microtask_ordering_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  let microtaskRan = false;
  queueMicrotask(() => {
    microtaskRan = true;
  });
  if (microtaskRan) {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve();
  if (!microtaskRan) {
    throw new Error('microtask did not run before the next turn');
  }
  console.log('queueMicrotask ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_negative_numeric_console_output_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(-3);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let chunk_dir = dir.path().join("lazy");
    fs::create_dir(&chunk_dir).expect("create lazy dir");
    fs::write(
        chunk_dir.join("index.js"),
        "export function lazyValue() { return 0n; }",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const chunk = await import("./lazy");
  if (typeof chunk.lazyValue !== 'function') {
    throw new Error('missing lazyValue export');
  }
  const value = await chunk.lazyValue();
  if (value !== 0n) {
    throw new Error(`unexpected chunk result ${value}`);
  }
  console.log(String(value));
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
}

#[test]
fn run_supports_dynamic_import_file_specifier_targets_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        dir.path().join("lazy.js"),
        "export function lazyValue() { return 0n; }",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  if (typeof chunk.lazyValue !== 'function') {
    throw new Error('missing lazyValue export');
  }
  const value = await chunk.lazyValue();
  if (value !== 0n) {
    throw new Error(`unexpected chunk result ${value}`);
  }
  console.log(String(value));
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn run_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let chunk_dir = dir.path().join("lazy");
    fs::create_dir(&chunk_dir).expect("create lazy dir");
    fs::write(
        chunk_dir.join("index.ts"),
        "export function lazyValue() { return 0n; }",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const chunk = await import("./lazy");
  if (typeof chunk.lazyValue !== 'function') {
    throw new Error('missing lazyValue export');
  }
  const value = await chunk.lazyValue();
  if (value !== 0n) {
    throw new Error(`unexpected chunk result ${value}`);
  }
  console.log(String(value));
}
main();
Kali.test('browser runtime smoke', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
}

#[test]
fn run_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_ts_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\n",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
}

#[test]
fn json_run_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\n",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
}

#[test]
fn run_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\n",
    )
    .expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
}

#[test]
fn json_run_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\n",
    )
    .expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
}

#[test]
fn run_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
}

#[test]
fn run_supports_console_assert_routing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.assert(false, 'assert failed');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["stdout"], "");
    assert!(
        json["stderr"]
            .as_str()
            .expect("stderr")
            .contains("assert failed"),
        "json: {json}"
    );
}

#[test]
fn run_supports_console_level_routing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.info('info');\nconsole.debug('debug');\nconsole.error('err');\nconsole.warn('warn');\nconsole.log(-1);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("info")
            && json["stdout"].as_str().expect("stdout").contains("debug")
            && json["stdout"].as_str().expect("stdout").contains("-1"),
        "json: {json}"
    );
    assert!(
        json["stderr"].as_str().expect("stderr").contains("err")
            && json["stderr"].as_str().expect("stderr").contains("warn"),
        "json: {json}"
    );
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_browser_like_executables() {
    run_browser_entrypoint_smoke("chromium");
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_source_paths_with_spaces() {
    browser_entrypoint_smoke(
        "run",
        "browser entry.ts",
        "console.log('browser run');",
        "browser run",
        "chromium",
    );
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_google_chrome_stable_executables() {
    for browser_name in ["google-chrome-stable", "google chrome stable"] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_chrome_aliases() {
    for browser_name in [
        "chrome",
        "chrome-beta",
        "chrome-canary",
        "chrome-dev",
        "chrome-unstable",
        "google-chrome",
        "google-chrome-beta",
        "google-chrome-canary",
        "google-chrome-dev",
        "google-chrome-unstable",
        "chrome-headless-shell",
        "google-chrome-headless-shell",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_remaining_browser_aliases() {
    for browser_name in [
        "chromium-browser",
        "chromium-for-testing",
        "chromium for testing",
        "chromium-dev",
        "chromium-headless-shell",
        "chrome for testing",
        "google chrome",
        "google chrome beta",
        "google chrome canary",
        "google chrome dev",
        "google chrome for testing",
        "google chrome unstable",
        "brave",
        "brave-browser",
        "brave browser",
        "edge",
        "msedge",
        "opera",
        "privacy-browser",
        "privacy browser",
        "vivaldi",
        "vivaldi snapshot",
        "vivaldi-snapshot",
        "microsoft-edge",
        "microsoft edge",
        "mullvad browser",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_mullvad_browser_executables() {
    run_browser_entrypoint_smoke("mullvad-browser");
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_command_wrapped_executables() {
    run_browser_entrypoint_smoke("chromium.command");
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_edge_beta_executables() {
    run_browser_entrypoint_smoke("edge-beta");
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_msedge_canary_executables() {
    run_browser_entrypoint_smoke("msedge-canary");
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_msedge_stable_executables() {
    for browser_name in [
        "msedge-stable",
        "edge-stable",
        "microsoft-edge-stable",
        "microsoft edge stable",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_edge_aliases() {
    for browser_name in [
        "msedge-beta",
        "msedge-dev",
        "msedge-insider",
        "edge-canary",
        "edge-dev",
        "edge-insider",
        "microsoft-edge-beta",
        "microsoft-edge-canary",
        "microsoft-edge-dev",
        "microsoft-edge-insider",
        "microsoft edge beta",
        "microsoft edge canary",
        "microsoft edge dev",
        "microsoft edge insider",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_brave_browser_stable_executables() {
    for browser_name in ["brave-browser-stable", "brave browser stable"] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_brave_aliases() {
    for browser_name in [
        "brave-browser-beta",
        "brave-browser-dev",
        "brave-browser-nightly",
        "brave browser beta",
        "brave browser dev",
        "brave browser nightly",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_stable_browser_aliases() {
    for browser_name in ["firefox-esr", "opera-stable", "vivaldi-stable"] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_opera_aliases() {
    for browser_name in [
        "opera-beta",
        "opera-developer",
        "opera-unstable",
        "opera beta",
        "opera developer",
        "opera unstable",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_firefox_aliases() {
    for browser_name in [
        "firefox",
        "firefox-beta",
        "firefox-nightly",
        "firefox-developer-edition",
        "firefox developer edition",
        "firefox beta",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_chrome_for_testing_aliases() {
    for browser_name in [
        "chrome-for-testing",
        "chromium-for-testing",
        "google-chrome-for-testing",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn run_uses_browser_entrypoint_for_additional_privacy_browser_aliases() {
    for browser_name in [
        "librewolf",
        "waterfox",
        "zen-browser",
        "zen browser",
        "thorium-browser",
        "thorium browser",
    ] {
        run_browser_entrypoint_smoke(browser_name);
    }
}

#[test]
fn run_uses_browser_package_resolution_when_a_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["stdout"], "0\n", "json: {json}");
}

#[test]
fn run_uses_browser_package_resolution_when_a_harness_command_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["stdout"], "0\n", "json: {json}");
}

#[test]
fn run_uses_browser_package_resolution_when_the_browser_api_surface_is_inherited() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\n",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["stdout"], "0\n", "json: {json}");
}

#[test]
fn run_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('browser zero budgets');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser zero budgets"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser zero budgets');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser zero budgets"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, "console.log('browser zero budgets');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser zero budgets"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, "console.log('browser zero budgets');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("browser zero budgets"),
        "json: {json}"
    );
}

#[test]
fn run_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_without_json_output(
) {
    let dir = tempdir().expect("tempdir");
    for (filename, source) in [
        ("main.js", "console.log('browser zero budgets');\n"),
        ("main.ts", "console.log('browser zero budgets');\n"),
        ("main.jsx", "console.log('browser zero budgets');\n"),
        ("main.tsx", "console.log('browser zero budgets');\n"),
    ] {
        let source_path = dir.path().join(filename);
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .arg("run")
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
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("browser zero budgets"), "stdout: {stdout}");
    }
}

#[test]
fn run_accepts_positive_thread_budget_override_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    for (filename, source) in [
        ("main.js", "console.log('browser threaded budgets');\n"),
        ("main.ts", "console.log('browser threaded budgets');\n"),
        ("main.jsx", "console.log('browser threaded budgets');\n"),
        ("main.tsx", "console.log('browser threaded budgets');\n"),
    ] {
        let source_path = dir.path().join(filename);
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg("--wasm-threads")
            .arg("--max-threads")
            .arg("1")
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
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("browser threaded budgets"),
            "json: {json}"
        );
    }
}

#[test]
fn run_uses_browser_exports_condition_package_resolution_when_a_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserexports");
    write_browser_runtime_exports_package_fixture(&package_dir, "browserexports");

    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import describe from 'browserexports';\nconsole.log(describe());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n");
}

#[test]
fn run_accepts_zero_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-threads")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
    assert!(
        stderr.contains("compilerOptions.runtimeProfiles"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_accepts_threaded_profile_with_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--wasm-threads")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_rejects_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("resources.maxSpawnedProcesses"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_run_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_rejects_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("resources.maxSpawnedProcesses"),
        "json: {json}"
    );
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--max-spawned-processes");
    assert_eq!(errors[0]["context"]["requestedValue"], "1");
    assert_eq!(errors[0]["context"]["effectiveValue"], "1");
}

#[test]
fn run_and_test_accept_the_specialization_cap_override() {
    let dir = tempdir().expect("tempdir");
    let run_source = dir.path().join("main.ts");
    let test_source = dir.path().join("smoke.test.ts");

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
    fs::write(&run_source, "console.log('specialization-cap');").expect("write run source");
    fs::write(
        &test_source,
        "Kali.test('addition', () => {\n    1 + 2;\n});\n",
    )
    .expect("write test source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--max-specializations")
        .arg("4")
        .arg(&run_source)
        .output()
        .expect("run kali");

    assert!(
        run.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("specialization-cap"),
        "stdout: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--max-specializations")
        .arg("4")
        .arg(&test_source)
        .output()
        .expect("run kali");

    assert!(
        test.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
}

#[test]
fn run_rejects_declaration_only_fixture_entrypoints() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/decl.d.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5507"), "stderr: {stderr}");
}

#[test]
fn run_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn run_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn run_and_test_reject_malformed_browser_harness_command_overrides_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser');").expect("write source");

    for command in ["run", "test"] {
        for json_output in [false, true] {
            let mut cli = Command::new(kali_bin());
            cli.current_dir(dir.path()).env(
                kali_runtime::BROWSER_HARNESS_COMMAND_ENV,
                r#"browser-wrapper "unterminated"#,
            );
            if json_output {
                cli.arg("--output").arg("json");
            }
            cli.arg(command)
                .arg("--api")
                .arg("browser")
                .arg(&source_path);
            let output = cli.output().expect("run kali");

            assert!(
                !output.status.success(),
                "{command} should reject malformed browser harness command overrides"
            );
            assert_eq!(output.status.code(), Some(1));
            if json_output {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(!errors.is_empty(), "errors: {errors:?}");
                assert_eq!(errors[0]["code"], "E5506");
                assert_browser_runtime_rejection_message(
                    errors[0]["message"]
                        .as_str()
                        .expect("browser rejection message"),
                );
                assert_browser_runtime_rejection_notes(
                    errors[0]["notes"]
                        .as_array()
                        .expect("browser rejection notes"),
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(stderr.contains("E5506"), "stderr: {stderr}");
                assert_browser_runtime_rejection_text(&stderr);
            }
        }
    }
}

#[test]
fn run_accepts_browser_api_surface_when_a_browser_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("browser run"), "stdout: {stdout}");
}

#[test]
fn run_accepts_inherited_browser_api_surface_when_a_browser_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("browser run"), "stdout: {stdout}");
}

#[test]
fn run_accepts_browser_api_surface_with_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_inherited_browser_api_surface_with_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_browser_api_surface_with_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_inherited_browser_api_surface_with_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_browser_api_surface_with_integer_like_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_inherited_browser_api_surface_with_integer_like_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_browser_api_surface_with_integer_like_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_inherited_browser_api_surface_with_integer_like_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--api");
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "config");
    assert_eq!(
        errors[0]["context"]["configPath"],
        "compilerOptions.apiSurface"
    );
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn run_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
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
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
}

#[test]
fn run_rejects_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
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
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
}

#[test]
fn run_rejects_browser_api_surface_with_missing_sandbox_policy_before_policy_loading_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let missing_policy_path = dir.path().join("missing.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&missing_policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
    assert!(
        !stderr.contains("failed to load sandbox policy"),
        "stderr should reject browser runtime before policy loading: {stderr}"
    );
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_with_sandbox_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
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
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
}

#[test]
fn json_run_rejects_browser_api_surface_with_guest_args_in_phase_one() {
    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .arg("--")
        .arg("guest-flag")
        .arg("guest-value")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_with_guest_args_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .arg("--")
        .arg("guest-flag")
        .arg("guest-value")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
}

#[test]
fn run_rejects_inherited_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn run_rejects_inherited_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
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
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "config");
    assert_eq!(
        errors[0]["context"]["configPath"],
        "compilerOptions.apiSurface"
    );
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn run_accepts_wasm_threads_runtime_profile() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--wasm-threads")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("threaded run"), "stdout: {stdout}");
}

#[test]
fn run_accepts_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("threaded run"), "stdout: {stdout}");
}

#[test]
fn run_accepts_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("threaded run"), "stdout: {stdout}");
}

#[test]
fn run_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_run_supports_object_enumeration_integer_like_key_ordering_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw 'unexpected numeric-key ordering';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_enumeration_integer_like_key_ordering_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw 'unexpected numeric-key ordering';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_abs_and_sign_slices_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "console.log(globalThis[\"Math\"][\"abs\"](3 - 6));\nconsole.log(globalThis[\"Math\"][\"sign\"](3 - 6));\n",
            "3",
        ),
        (
            "run",
            "main.js",
            "console.log(globalThis[\"Math\"][\"abs\"](3 - 6));\nconsole.log(globalThis[\"Math\"][\"sign\"](3 - 6));\n",
            "3",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed globalThis math abs/sign', () => { console.log(globalThis[\"Math\"][\"abs\"](3 - 6)); console.log(globalThis[\"Math\"][\"sign\"](3 - 6)); });\n",
            "3\n-1\nok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed globalThis math abs/sign', () => { console.log(globalThis[\"Math\"][\"abs\"](3 - 6)); console.log(globalThis[\"Math\"][\"sign\"](3 - 6)); });\n",
            "3\n-1\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("3"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("-1"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if command == "run" {
                    assert!(stdout.contains("3\n"), "stdout: {stdout}");
                    assert!(stdout.contains("-1\n"), "stdout: {stdout}");
                } else {
                    assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
                }
            }
        }
    }
}

#[test]
fn run_supports_math_trunc_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_trunc_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_ceil_semantics_when_browser_harness_is_configured_with_inherited_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn run_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
"#,
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
}

#[test]
fn run_evaluates_dynamic_eval_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const source = \"1\" + \" + 2\"; if (eval(source) !== 3) { throw new Error('bad eval result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_evaluates_dynamic_function_constructor_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const body = \"return \" + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_arithmetic_precedence() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1 + 2 * 3);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('7'), "stdout: {stdout}");
}

#[test]
fn run_supports_array_literal_length() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log([1, 2, 3].length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_arithmetic_precedence_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(1 + 2 * 3);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "7", "stdout: {stdout}");
}

#[test]
fn run_supports_array_literal_length_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log([1, 2, 3].length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_process_argv_slice_length_in_js_input_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(process.argv.slice(2).length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .arg("--")
        .arg("alpha")
        .arg("beta")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "2", "stdout: {stdout}");
}

#[test]
fn run_supports_array_literal_indexing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log([1, 2, 3][1]);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "2", "stdout: {stdout}");
}

#[test]
fn run_supports_object_literal_property_access_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(({ a: 1, b: 2 }).b);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "2", "stdout: {stdout}");
}

#[test]
fn run_supports_nested_math_call_composition_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(Math.min(1, 2), Math.abs(3 - 6)));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn json_run_supports_nested_math_call_composition_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(Math.min(1, 2), Math.abs(3 - 6)));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_crypto_get_random_values_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log('ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "ok", "stdout: {stdout}");
}

#[test]
fn run_supports_literal_string_dynamic_import_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        dir.path().join("lazy.ts"),
        "console.log('lazy loaded'); export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  await import("./lazy.ts");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main loaded"), "stdout: {stdout}");
}

#[test]
fn run_supports_literal_string_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        dir.path().join("lazy.js"),
        "console.log('lazy loaded'); export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  await import("./lazy.js");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main loaded"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_literal_string_dynamic_import_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const value = 7;").expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const chunk = await import("./lazy.ts");
  console.log(chunk.value);
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("main loaded"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_literal_string_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const value = 7;").expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  console.log(chunk.value);
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("main loaded"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_literal_string_dynamic_import_targets_in_jsx_input() {
    assert_literal_string_dynamic_import_runtime_support("jsx", false);
}

#[test]
fn json_run_supports_literal_string_dynamic_import_targets_in_jsx_input() {
    assert_literal_string_dynamic_import_runtime_support("jsx", true);
}

#[test]
fn run_supports_literal_string_dynamic_import_targets_in_tsx_input() {
    assert_literal_string_dynamic_import_runtime_support("tsx", false);
}

#[test]
fn json_run_supports_literal_string_dynamic_import_targets_in_tsx_input() {
    assert_literal_string_dynamic_import_runtime_support("tsx", true);
}

#[test]
fn run_supports_browser_web_crypto_subtle_digest_and_random_uuid_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const bytes = new TextEncoder().encode('browser crypto');
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  const uuid = crypto.randomUUID();
  if (digest.byteLength !== 32) {
    throw new Error(`unexpected digest length ${digest.byteLength}`);
  }
  if (typeof uuid !== 'string' || uuid.length === 0) {
    throw new Error(`unexpected uuid ${uuid}`);
  }
  console.log('ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_browser_web_crypto_subtle_digest_and_random_uuid_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const bytes = new TextEncoder().encode('browser crypto');
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  const uuid = crypto.randomUUID();
  if (digest.byteLength !== 32) {
    throw new Error(`unexpected digest length ${digest.byteLength}`);
  }
  if (typeof uuid !== 'string' || uuid.length === 0) {
    throw new Error(`unexpected uuid ${uuid}`);
  }
  console.log('ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_browser_web_crypto_get_random_values_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log('ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_browser_web_crypto_get_random_values_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log('ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn run_supports_browser_web_crypto_get_random_values_when_browser_api_surface_is_inherited_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_requested_web_crypto_get_random_values_when_browser_api_surface_is_inherited(
        "run", "main.ts",
    );
}

#[test]
fn run_supports_browser_web_crypto_get_random_values_when_browser_api_surface_is_inherited_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_requested_web_crypto_get_random_values_when_browser_api_surface_is_inherited(
        "run", "main.js",
    );
}

#[test]
fn run_supports_function_call_return_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } if (add(2, 5) !== 7) { throw new Error('bad function result'); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_function_call_return_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } if (add(2, 5) !== 7) { throw new Error('bad function result'); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_async_await_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn json_run_supports_async_await_sequencing_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
  console.log('asyncAwait ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("asyncAwait ok"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  const result = await Promise.resolve(7);
  if (result !== 7) {
    throw new Error(`unexpected async result ${result}`);
  }
  console.log('asyncAwait ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("asyncAwait ok"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_promise_all_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, promise_all_sequencing_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_promise_all_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, promise_all_sequencing_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.js", false, false);
}

#[test]
fn run_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.js", false, true);
}

#[test]
fn json_run_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.js", true, false);
}

#[test]
fn json_run_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.js", true, true);
}

#[test]
fn run_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.ts", false, false);
}

#[test]
fn run_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.ts", false, true);
}

#[test]
fn json_run_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.ts", true, false);
}

#[test]
fn json_run_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("run", "main.ts", true, true);
}

#[test]
fn run_supports_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  let microtaskRan = false;
  queueMicrotask(() => {
    microtaskRan = true;
  });
  if (microtaskRan) {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve();
  if (!microtaskRan) {
    throw new Error('microtask did not run before the next turn');
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn json_run_supports_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"async function main() {
  let microtaskRan = false;
  queueMicrotask(() => {
    microtaskRan = true;
  });
  if (microtaskRan) {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve();
  if (!microtaskRan) {
    throw new Error('microtask did not run before the next turn');
  }
  console.log('queueMicrotask ok');
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("queueMicrotask ok"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_rejects_async_class_method_sequencing_in_ts_js_jsx_and_tsx_input() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            r#"async function main() {
  class Example {
    async main() {
      await Promise.resolve();
      return 1;
    }
  }

  const value = await new Example().main();
  if (value !== 1) {
    throw new Error(`unexpected async class method result ${value}`);
  }
}
main();
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("async class method lowering is unavailable"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn json_run_rejects_async_class_method_sequencing_in_ts_js_jsx_and_tsx_input() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            r#"async function main() {
  class Example {
    async main() {
      await Promise.resolve();
      return 1;
    }
  }

  const value = await new Example().main();
  if (value !== 1) {
    throw new Error(`unexpected async class method result ${value}`);
  }
}
main();
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        assert!(errors.iter().any(|error| error["message"]
            .as_str()
            .unwrap()
            .contains("async class method lowering is unavailable")));
    }
}

#[test]
fn run_supports_optional_chaining_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("index.js"),
        r#"export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("main.js"),
        r#"import { minVersion } from 'semver';
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(dir.path().join("main.js"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2.3\n", "stdout: {stdout}");
}

#[test]
fn json_run_supports_optional_chaining_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("index.js"),
        r#"export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("main.js"),
        r#"import { minVersion } from 'semver';
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(dir.path().join("main.js"))
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1.2.3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_optional_chaining_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("index.js"),
        r#"export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("main.js"),
        r#"import { minVersion } from 'semver';
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1.2.3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_relational_comparison_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "if (1 < 2) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_relational_comparison_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "if (1 < 2) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_strict_inequality_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "if (1 !== 0) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_strict_inequality_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "if (1 !== 0) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_try_catch_exception_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"try {
  throw 'boom';
} catch {}
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_try_finally_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"try {
  0;
} finally {
  console.log(2);
}
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_try_catch_exception_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"try {
  throw 'boom';
} catch {}
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_try_finally_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"function main() {
  try {
    for (const value of [1, 2]) {
      console.log(value);
      break;
    }
  } finally {
    console.log(3);
  }
}
main();
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_bigint_addition_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1n + 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_bigint_addition_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(1n + 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3", "stdout: {stdout}");
}

#[test]
fn run_supports_bigint_multiplication_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1n * 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "2", "stdout: {stdout}");
}

#[test]
fn run_supports_bigint_multiplication_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(1n * 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "2", "stdout: {stdout}");
}

#[test]
fn run_supports_bigint_subtraction_semantics() {
    assert_run_supports_bigint_binary_semantics("ts", "3n - 2n", "1");
}

#[test]
fn run_supports_bigint_subtraction_semantics_in_js_input() {
    assert_run_supports_bigint_binary_semantics("js", "3n - 2n", "1");
}

#[test]
fn run_supports_bigint_division_semantics() {
    assert_run_supports_bigint_binary_semantics("ts", "3n / 2n", "1");
}

#[test]
fn run_supports_bigint_division_semantics_in_js_input() {
    assert_run_supports_bigint_binary_semantics("js", "3n / 2n", "1");
}

#[test]
fn run_supports_negative_bigint_literal_division_semantics() {
    // node: -7n / 2n === -3n — BigInt `/` truncates toward zero, so a
    // unary-minus-wrapped BigInt literal dividend must still take the
    // truncating i64 division lane, not the float lane (which would print
    // -3.5).
    assert_run_supports_bigint_binary_semantics("ts", "-7n / 2n", "-3");
}

#[test]
fn run_supports_bigint_remainder_semantics() {
    assert_run_supports_bigint_binary_semantics("ts", "3n % 2n", "1");
}

#[test]
fn run_supports_bigint_remainder_semantics_in_js_input() {
    assert_run_supports_bigint_binary_semantics("js", "3n % 2n", "1");
}

#[test]
fn run_supports_bigint_exponentiation_semantics() {
    assert_run_supports_bigint_binary_semantics("ts", "2n ** 3n", "8");
}

#[test]
fn run_supports_bigint_exponentiation_semantics_in_js_input() {
    assert_run_supports_bigint_binary_semantics("js", "2n ** 3n", "8");
}

#[test]
fn run_supports_object_keys_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const keys = Object.keys(obj);
if (keys.length !== 2 || keys[0] !== 'a' || keys[1] !== 'b') {
  throw 'unexpected keys';
}
console.log(keys.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

#[test]
fn run_supports_object_entries_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const entries = Object.entries(obj);
if (
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 1 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2
) {
  throw 'unexpected entries';
}
console.log(entries.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

#[test]
fn run_supports_object_values_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
  throw 'unexpected values';
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_object_keys_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const keys = Object.keys(obj);
if (keys.length !== 2 || keys[0] !== 'a' || keys[1] !== 'b') {
  throw 'unexpected keys';
}
console.log(keys.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

#[test]
fn run_supports_object_entries_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const entries = Object.entries(obj);
if (
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 1 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2
) {
  throw 'unexpected entries';
}
console.log(entries.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

#[test]
fn run_supports_object_values_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "a": 1, "b": 2 };
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
  throw 'unexpected values';
}
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_object_string_primitive_enumeration_semantics_in_js_input() {
    assert_object_string_primitive_enumeration_semantics(
        "run",
        "main.js",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn run_supports_object_string_primitive_enumeration_semantics_in_ts_input() {
    assert_object_string_primitive_enumeration_semantics(
        "run",
        "main.ts",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn run_supports_object_string_primitive_enumeration_semantics_in_jsx_input() {
    assert_object_string_primitive_enumeration_semantics(
        "run",
        "main.jsx",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn run_supports_object_string_primitive_enumeration_semantics_in_tsx_input() {
    assert_object_string_primitive_enumeration_semantics(
        "run",
        "main.tsx",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn json_run_supports_object_string_primitive_enumeration_semantics_in_js_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "run",
        "main.js",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn json_run_supports_object_string_primitive_enumeration_semantics_in_ts_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "run",
        "main.ts",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn json_run_supports_object_string_primitive_enumeration_semantics_in_jsx_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "run",
        "main.jsx",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn json_run_supports_object_string_primitive_enumeration_semantics_in_tsx_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "run",
        "main.tsx",
        object_string_primitive_enumeration_semantics_source(),
    );
}

#[test]
fn json_run_supports_object_enumeration_semantics_in_js_input() {
    assert_json_object_enumeration_semantics("run", "main.js");
}

#[test]
fn json_run_supports_object_from_entries_enumeration_semantics_in_js_input() {
    assert_json_object_from_entries_semantics("run", "main.js");
}

#[test]
fn json_run_supports_frozen_object_enumeration_spread_semantics_in_js_input() {
    assert_json_frozen_object_enumeration_spread_semantics("run", "main.js");
}

#[test]
fn json_run_supports_frozen_object_enumeration_spread_semantics_in_ts_input() {
    assert_json_frozen_object_enumeration_spread_semantics("run", "main.ts");
}

#[test]
fn run_supports_object_from_entries_enumeration_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_from_entries_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2\n2\n2\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_object_from_entries_enumeration_semantics_with_satisfies_wrapper_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        browser_runtime_object_from_entries_satisfies_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2\n2\n2\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_object_from_entries_has_own_semantics_in_jsx_input_when_browser_harness_is_configured(
) {
    assert_browser_runtime_object_from_entries_has_own_semantics_in_input(
        "run",
        "main.jsx",
        browser_runtime_object_from_entries_has_own_source(),
        true,
    );
}

#[test]
fn run_supports_object_from_entries_has_own_semantics_in_tsx_input_when_browser_harness_is_configured(
) {
    assert_browser_runtime_object_from_entries_has_own_semantics_in_input(
        "run",
        "main.tsx",
        browser_runtime_object_from_entries_has_own_source(),
        true,
    );
}

#[test]
fn json_run_supports_frozen_object_enumeration_spread_semantics_in_jsx_input_when_browser_harness_is_configured(
) {
    assert_json_browser_runtime_frozen_object_enumeration_spread_semantics_in_input(
        "run",
        "main.jsx",
        browser_runtime_frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn json_run_supports_frozen_object_enumeration_spread_semantics_in_tsx_input_when_browser_harness_is_configured(
) {
    assert_json_browser_runtime_frozen_object_enumeration_spread_semantics_in_input(
        "run",
        "main.tsx",
        browser_runtime_frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn run_supports_object_property_deletion_semantics() {
    assert_object_property_deletion_semantics("run", "smoke.ts");
}

#[test]
fn run_supports_object_property_deletion_semantics_in_js_input() {
    assert_object_property_deletion_semantics("run", "smoke.js");
}

#[test]
fn json_run_supports_object_property_deletion_semantics() {
    assert_json_object_property_deletion_semantics("run", "smoke.ts");
}

#[test]
fn json_run_supports_object_property_deletion_semantics_in_js_input() {
    assert_json_object_property_deletion_semantics("run", "smoke.js");
}

#[test]
fn run_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_requested_object_property_deletion_semantics("run", "main.ts");
}

#[test]
fn run_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_requested_object_property_deletion_semantics("run", "main.js");
}

#[test]
fn json_run_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_json_browser_requested_object_property_deletion_semantics("run", "main.ts");
}

#[test]
fn json_run_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_json_browser_requested_object_property_deletion_semantics("run", "main.js");
}

#[test]
fn run_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "run",
        "main.ts",
    );
}

#[test]
fn run_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "run",
        "main.js",
    );
}

#[test]
fn json_run_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    assert_json_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "run",
        "main.ts",
    );
}

#[test]
fn json_run_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_json_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "run",
        "main.js",
    );
}

#[test]
fn run_supports_object_type_and_constructor_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.ts");
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(false),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: `instanceof` is rejected fail-closed (E5506): kali has no prototype
    // chain; the token was previously dropped so the expression miscompiled
    // to its left operand.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_object_type_and_constructor_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.js");
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(false),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: `instanceof` is rejected fail-closed (E5506): kali has no prototype
    // chain; the token was previously dropped so the expression miscompiled
    // to its left operand.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_object_type_and_constructor_semantics() {
    assert_json_object_type_and_constructor_semantics("run", "smoke.ts", false);
}

#[test]
fn json_run_supports_object_type_and_constructor_semantics_in_js_input() {
    assert_json_object_type_and_constructor_semantics("run", "smoke.js", false);
}

#[test]
fn json_run_supports_browser_requested_object_type_and_constructor_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_json_browser_requested_object_type_and_constructor_semantics("run", "main.ts", false);
}

#[test]
fn json_run_supports_browser_requested_object_type_and_constructor_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_json_browser_requested_object_type_and_constructor_semantics("run", "main.js", false);
}

#[test]
fn run_supports_unary_prefix_semantics() {
    assert_unary_prefix_semantics("run", "smoke.ts");
}

#[test]
fn run_supports_unary_prefix_semantics_in_js_input() {
    assert_unary_prefix_semantics("run", "smoke.js");
}

#[test]
fn json_run_supports_unary_prefix_semantics() {
    assert_json_unary_prefix_semantics("run", "smoke.ts");
}

#[test]
fn json_run_supports_unary_prefix_semantics_in_js_input() {
    assert_json_unary_prefix_semantics("run", "smoke.js");
}

#[test]
fn run_supports_unary_prefix_semantics_with_browser_harness_in_js_input() {
    assert_browser_unary_prefix_semantics("run", "main.js", false);
}

#[test]
fn json_run_supports_unary_prefix_semantics_with_browser_harness_in_js_input() {
    assert_json_browser_unary_prefix_semantics("run", "main.js", false);
}

#[test]
fn run_supports_wrapped_mutable_update_targets_with_browser_harness_in_ts_input() {
    assert_browser_wrapped_mutable_update_targets("run", "main.ts", false);
}

#[test]
fn json_run_supports_wrapped_mutable_update_targets_with_browser_harness_in_ts_input() {
    assert_json_browser_wrapped_mutable_update_targets("run", "main.ts", false);
}

#[test]
fn run_supports_wrapped_mutable_compound_assignment_targets_with_browser_harness_in_ts_input() {
    assert_browser_wrapped_mutable_compound_assignment_targets("run", "main.ts", false);
}

#[test]
fn json_run_supports_wrapped_mutable_compound_assignment_targets_with_browser_harness_in_ts_input()
{
    assert_json_browser_wrapped_mutable_compound_assignment_targets("run", "main.ts", false);
}

#[test]
fn run_supports_object_enumeration_semantics_with_overwrite_ordering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_object_enumeration_integer_like_key_ordering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw 'unexpected numeric-key ordering';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_object_enumeration_semantics_with_overwrite_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_object_enumeration_semantics_with_overwrite_ordering_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_console_level_routing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.info('info');\nconsole.debug('debug');\nconsole.error('err');\nconsole.warn('warn');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("info"), "stdout: {stdout}");
    assert!(stdout.contains("debug"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("[warn] warn"), "stderr: {stderr}");
}

#[test]
fn run_supports_array_iteration_semantics_for_now() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"for (const value of [1, 2, 3]) {
  console.log(value);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
    assert!(stdout.contains("2"), "stdout: {stdout}");
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn run_supports_array_iteration_semantics_for_now_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of [1, 2, 3]) {
  console.log(value);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
    assert!(stdout.contains("2"), "stdout: {stdout}");
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn run_supports_static_array_search_helpers_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"console.log([0, 1, 2].includes(1));
console.log([0, 1, 2, 1].indexOf(1, 2));
console.log([0, 1, 2, 1].lastIndexOf(1, 2));
console.log([0, 1, 2, 1].lastIndexOf(1));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, ["1", "3", "1", "3"], "stdout: {stdout}");
}

#[test]
fn run_supports_array_iteration_semantics_with_const_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = 1; const alias = value; for (const item of [alias]) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_array_iteration_semantics_with_const_string_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = "hello"; const alias = value; for (const item of [alias]) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"), "stdout: {stdout}");
}

#[test]
fn run_supports_array_iteration_semantics_with_string_concatenation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const prefix = "he"; const suffix = "llo"; for (const item of prefix + suffix) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, ["h", "e", "l", "l", "o"], "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_semantics_for_now() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"for await (const value of [1, 2, 3]) {
  console.log(value);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}
#[test]
fn run_supports_for_await_array_iteration_semantics_for_now_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for await (const value of [1, 2, 3]) {
  console.log(value);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_supports_for_await_array_iteration_with_await_wrapped_literal_array_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for await (const value of await [1, 2, 3]) {
  console.log(value);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, ["1", "2", "3"], "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_semantics_with_const_string_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = "hello"; const alias = value; for await (const item of [alias]) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_supports_for_await_array_iteration_semantics_with_string_concatenation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const prefix = "he"; const suffix = "llo"; for await (const item of prefix + suffix) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, ["h", "e", "l", "l", "o"], "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_semantics_with_string_concatenation_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const prefix = "he"; const suffix = "llo"; for await (const item of prefix + suffix) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, ["h", "e", "l", "l", "o"], "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_semantics_with_const_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = 1; const alias = value; for await (const item of [alias]) {
  console.log(item);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_set_and_map_constructor_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, set_and_map_iteration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_set_and_map_constructor_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, set_and_map_iteration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_math_max_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.max(1, 2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_max_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.max(1, 2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_global_this_math_max_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(globalThis.Math.max(1, 2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_global_this_math_builtin_slices_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis.Math.min(3, 2, 1));\nconsole.log(globalThis.Math.abs(-4));\nconsole.log(globalThis.Math.sign(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("4\n"), "stdout: {stdout}");
    assert!(stdout.contains("0\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_global_this_math_atan2_zero_slice_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.atan2(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

#[test]
fn run_supports_bracketed_global_this_math_atan2_zero_slice_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one)); console.log(globalThis.Math[\"atan2\"](zero, one)); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n0\n0\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_frozen_math_atan2_zero_slice_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; const frozenDotRoot = Object.freeze(globalThis.Math.atan2); const frozenBracketedRoot = Object.freeze(globalThis[\"Math\"][\"atan2\"]); const frozenSingleQuotedRoot = Object.freeze(globalThis['Math']['atan2']); const frozenDirect = Object.freeze(Math.atan2); console.log(frozenDotRoot(zero, one)); console.log(frozenBracketedRoot(zero, one)); console.log(frozenSingleQuotedRoot(zero, one)); console.log(frozenDirect(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n0\n0\n0\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_min_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.min(3, 2, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_min_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.min(3, 2, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_ceil_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_ceil_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("-3"), "json: {json}");
}

#[test]
fn run_supports_math_round_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.round(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_round_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.round(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("-3"), "json: {json}");
}

#[test]
fn run_supports_math_round_builtin_semantics_through_const_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.round(alias)); console.log(Math.round(Object.freeze(alias))); console.log(Object.freeze(globalThis.Math.round)(alias)); console.log(Object.freeze(globalThis.Math[\"round\"])(alias)); console.log(Object.freeze((globalThis.Math[\"round\"]))(alias)); console.log(Object.freeze(globalThis.Math['round'])(alias)); console.log(Object.freeze(globalThis[\"Math\"]['round'])(alias)); console.log(Object.freeze(globalThis['Math'].round)(alias)); console.log(Object.freeze(Math.round)(alias)); console.log(Object.freeze((Math.round))(alias)); console.log(Object.freeze(globalThis['Math']['round'])(alias)); console.log(Object.freeze((globalThis.Math)[\"round\"])(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|line| *line == "2").count(),
        12,
        "stdout: {stdout}"
    );
}

#[test]
fn json_run_supports_math_round_builtin_semantics_through_const_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.round(alias)); console.log(Math.round(Object.freeze(alias))); console.log(Object.freeze(globalThis.Math.round)(alias)); console.log(Object.freeze(globalThis.Math[\"round\"])(alias)); console.log(Object.freeze((globalThis.Math[\"round\"]))(alias)); console.log(Object.freeze(globalThis.Math['round'])(alias)); console.log(Object.freeze(globalThis[\"Math\"]['round'])(alias)); console.log(Object.freeze(globalThis['Math'].round)(alias)); console.log(Object.freeze(Math.round)(alias)); console.log(Object.freeze((Math.round))(alias)); console.log(Object.freeze(globalThis['Math']['round'])(alias)); console.log(Object.freeze((globalThis.Math)[\"round\"])(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert_eq!(
        stdout.lines().filter(|line| *line == "2").count(),
        12,
        "json: {json}"
    );
}

#[test]
fn run_supports_math_abs_sign_frozen_callable_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        format!(
            "const value = -3; const alias = value; console.log(Math.abs(alias)); console.log(Math.sign(alias)); {}\n",
            math_abs_sign_frozen_callable_invocation_lines("")
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.lines().any(|line| line == "3"), "stdout: {stdout}");
    assert!(stdout.lines().any(|line| line == "-1"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_abs_sign_frozen_callable_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        format!(
            "const value = -3; const alias = value; console.log(Math.abs(alias)); console.log(Math.sign(alias)); {}\n",
            math_abs_sign_frozen_callable_invocation_lines("")
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
}

#[test]
fn run_and_test_supports_math_cbrt_frozen_callable_aliases_in_js_ts_jsx_and_tsx_input() {
    let expected_line_count = math_cbrt_frozen_callable_aliases().len();
    for (command, source_name, _extension) in [
        ("run", "main.js", "js"),
        ("run", "main.ts", "ts"),
        ("run", "main.jsx", "jsx"),
        ("run", "main.tsx", "tsx"),
        ("test", "smoke.test.js", "js"),
        ("test", "smoke.test.ts", "ts"),
        ("test", "smoke.test.jsx", "jsx"),
        ("test", "smoke.test.tsx", "tsx"),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            let body = math_cbrt_frozen_callable_aliases()
                .iter()
                .map(|alias| format!("console.log({alias}(alias));"))
                .collect::<Vec<_>>()
                .join("\n");
            let source = if command == "test" {
                format!("Kali.test('cbrt frozen callable aliases', () => {{\nconst value = 27; const alias = value;\n{body}\n}});\n")
            } else {
                format!("const value = 27; const alias = value;\n{body}\n")
            };
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
                let json = parse_json_stdout(&output);
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
                let stdout = json["stdout"].as_str().expect("stdout");
                assert_eq!(
                    stdout.lines().filter(|line| *line == "3").count(),
                    expected_line_count,
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert_eq!(
                    stdout.lines().filter(|line| *line == "3").count(),
                    expected_line_count,
                    "stdout: {stdout}"
                );
            }
        }
    }
}

#[test]
fn run_supports_math_floor_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.floor(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_floor_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.floor(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("-3"), "json: {json}");
}

#[test]
fn run_supports_math_floor_numeric_literal_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"console.log(Math.floor(1.6));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_floor_numeric_literal_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"console.log(Math.floor(1.6));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1"), "json: {json}");
}

#[test]
fn run_supports_math_floor_trunc_and_ceil_direct_object_freeze_callable_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const floor = Object.freeze(Math.floor); const trunc = Object.freeze(Math.trunc); const ceil = Object.freeze(Math.ceil); console.log(floor(value)); console.log(trunc(value)); console.log(ceil(value));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("2\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_floor_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        math_floor_trunc_ceil_const_numeric_alias_chain_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_floor_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        math_floor_trunc_ceil_const_numeric_alias_chain_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("1"), "json: {json}");
}

#[test]
fn run_supports_math_cbrt_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"console.log(Math.cbrt(27));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_cbrt_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"console.log(Math.cbrt(27));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3"), "json: {json}");
}

#[test]
fn run_supports_math_abs_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.abs(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_abs_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.abs(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_sign_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.sign(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-1"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_sign_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sign(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-1"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_max_min_abs_sign_suite_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
}

#[test]
fn json_run_supports_math_max_min_abs_sign_suite_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
}

#[test]
fn run_supports_math_imul_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-2"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_imul_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-2"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_imul_builtin_omitted_operands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.imul());\nconsole.log(Math.imul(7));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n0"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_clz32_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.clz32(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("31"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_trunc_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_clz32_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_imul_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-2"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_imul_builtin_omitted_operands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.imul());\nconsole.log(Math.imul(7));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n0"),
        "json: {json}"
    );
}

#[test]
fn json_run_supports_math_trunc_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn run_supports_boolean_logic_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n3\n4"), "stdout: {stdout}");
}

#[test]
fn run_supports_boolean_logic_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"if (true && true) {
  console.log(1);
} else {
  console.log(0);
}
if (true || false) {
  console.log(2);
} else {
  console.log(0);
}
if (false && true) {
  console.log(0);
} else {
  console.log(3);
}
if (false || false) {
  console.log(0);
} else {
  console.log(4);
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n3\n4"), "stdout: {stdout}");
}

#[test]
fn run_supports_console_level_routing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.info('info');\nconsole.debug('debug');\nconsole.error('err');\nconsole.warn('warn');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("info"), "stdout: {stdout}");
    assert!(stdout.contains("debug"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("[warn] warn"), "stderr: {stderr}");
}

#[test]
fn run_supports_console_assert_routing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.assert(false, 'assert failed');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("assert failed"), "stderr: {stderr}");
}

#[test]
fn run_supports_console_assert_routing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.assert(false, 'assert failed');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("assert failed"), "stderr: {stderr}");
}

#[test]
fn run_rejects_non_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "let specifier; import(specifier);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_non_literal_dynamic_import_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_non_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "let specifier; import(specifier);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn run_supports_nullish_assignment_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_run_supports_nullish_assignment_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_rejects_compound_assignment_on_immutable_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, compound_assignment_immutable_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_compound_assignment_rejection_text(
        &stderr,
        "compound assignment lowering is unavailable for binding 'value'",
    );
}

#[test]
fn json_run_rejects_compound_assignment_on_immutable_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, compound_assignment_immutable_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_compound_assignment_rejection_json(
        errors,
        "compound assignment lowering is unavailable for binding 'value'",
    );
}

#[test]
fn run_supports_nullish_assignment_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_run_supports_nullish_assignment_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_nullish_assignment_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_run_supports_nullish_assignment_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_object_is_numeric_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Object.is(-0, 0) ? 'base-true' : 'base-false');\nconsole.log(globalThis[\"Object\"][\"is\"](1, 1) ? 'bracket-true' : 'bracket-false');\nconsole.log(globalThis.Object[\"is\"](1, 1) ? 'object-dot-bracket-true' : 'object-dot-bracket-false');\nconsole.log(globalThis[\"Object\"].is(1, 1) ? 'bracket-dot-true' : 'bracket-dot-false');\nconsole.log(globalThis.Object.is(1, 1) ? 'dot-dot-true' : 'dot-dot-false');\nconsole.log(Object.is(1n, 1n) ? 'bigint-true' : 'bigint-false');\nconsole.log(Object.is(-1n, -1n) ? 'neg-bigint-true' : 'neg-bigint-false');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The fixture wraps each `Object.is(...)` in a string-armed ternary. This
    // golden previously read "0\n1\n1\n1\n1\n1\n1" — the output of the parser's
    // SILENT `?:`-tail drop (`console.log(Object.is(...))` rendering the bool as
    // 0/1). Now that `?:` parses (Task 7) and branch-selects (Task 8), the
    // ternaries actually evaluate: `Object.is(-0, 0)` is false -> the alternate
    // string, the rest true -> the consequent strings. This is the correct JS
    // result; the old numeric golden encoded the now-fixed drop bug.
    assert_eq!(
        stdout.trim(),
        "base-false\nbracket-true\nobject-dot-bracket-true\nbracket-dot-true\ndot-dot-true\nbigint-true\nneg-bigint-true",
        "stdout: {stdout}"
    );
}

#[test]
fn json_run_supports_object_is_numeric_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Object.is(-0, 0) ? 'base-true' : 'base-false');\nconsole.log(globalThis[\"Object\"][\"is\"](1, 1) ? 'bracket-true' : 'bracket-false');\nconsole.log(globalThis.Object[\"is\"](1, 1) ? 'object-dot-bracket-true' : 'object-dot-bracket-false');\nconsole.log(globalThis[\"Object\"].is(1, 1) ? 'bracket-dot-true' : 'bracket-dot-false');\nconsole.log(globalThis.Object.is(1, 1) ? 'dot-dot-true' : 'dot-dot-false');\nconsole.log(Object.is(1n, 1n) ? 'bigint-true' : 'bigint-false');\nconsole.log(Object.is(-1n, -1n) ? 'neg-bigint-true' : 'neg-bigint-false');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    // Correct branch-selected output now that `?:` parses (Task 7) and evaluates
    // (Task 8); the old "0\n1\n1\n1\n1\n1\n1\n" golden was the parser's silent
    // `?:`-tail-drop result (see the sibling non-JSON test for the rationale).
    assert_eq!(
        json["stdout"],
        "base-false\nbracket-true\nobject-dot-bracket-true\nbracket-dot-true\ndot-dot-true\nbigint-true\nneg-bigint-true\n"
    );
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_object_is_infinity_and_nan_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Object.is(Infinity, Infinity));\nconsole.log(Object.is(NaN, NaN));\nconsole.log(Object.is(-Infinity, -Infinity));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1\n1\n1", "stdout: {stdout}");
}

#[test]
fn json_run_supports_object_is_infinity_and_nan_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Object.is(Infinity, Infinity));\nconsole.log(Object.is(NaN, NaN));\nconsole.log(Object.is(-Infinity, -Infinity));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n1\n1\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_object_is_same_static_reference_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const object = { a: 1 }; const alias = object; console.log(Object.is(alias, object)); console.log(Object.is(object, object));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1\n1", "stdout: {stdout}");
}

#[test]
fn json_run_supports_object_is_same_static_reference_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const object = { a: 1 }; const alias = object; console.log(Object.is(alias, object)); console.log(Object.is(object, object));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n1\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_object_is_unary_plus_wrapped_numeric_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Object.is(+1, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1", "stdout: {stdout}");
}

#[test]
fn json_run_supports_object_is_unary_plus_wrapped_numeric_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Object.is(+1, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_object_is_numeric_literals_in_browser_api_surface_with_harness_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        "console.log(Object.is(-0, 0));\nconsole.log(globalThis[\"Object\"][\"is\"](1, 1));\nconsole.log(globalThis.Object[\"is\"](1, 1));\nconsole.log(globalThis[\"Object\"].is(1, 1));\nconsole.log(globalThis.Object.is(1, 1));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n1\n1\n1\n1"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_object_is_numeric_literals_in_browser_api_surface_with_harness_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        "console.log(Object.is(-0, 0));\nconsole.log(globalThis[\"Object\"][\"is\"](1, 1));\nconsole.log(globalThis.Object[\"is\"](1, 1));\nconsole.log(globalThis[\"Object\"].is(1, 1));\nconsole.log(globalThis.Object.is(1, 1));\nconsole.log(Object.is(1n, 1n));\nconsole.log(Object.is(-1n, -1n));\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "0\n1\n1\n1\n1\n1\n1\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_run_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_js_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("run", "js", true);
}

#[test]
fn json_run_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_ts_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("run", "ts", true);
}

#[test]
fn json_run_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("run", "jsx", true);
}

#[test]
fn json_run_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("run", "tsx", true);
}

#[test]
fn run_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_nullish_coalescing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = null ?? 1;
console.log(value);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn json_run_supports_nullish_coalescing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = null ?? 1;
console.log(value);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const voidFallback = void 0 ?? 1;
const undefinedFallback = undefined ?? 2;
console.log(voidFallback);
console.log(undefinedFallback);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const voidFallback = void 0 ?? 1;
const undefinedFallback = undefined ?? 2;
console.log(voidFallback);
console.log(undefinedFallback);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n2\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_and_test_reject_optional_chain_wrapped_math_pow_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for (command, source_name, source) in [
        ("run", "main.js", "console.log(Math?.pow(2, 3));\n"),
        ("run", "main.ts", "console.log(Math?.pow(2, 3));\n"),
        ("run", "main.jsx", "console.log(Math?.pow(2, 3));\n"),
        ("run", "main.tsx", "console.log(Math?.pow(2, 3));\n"),
        (
            "run",
            "main.js",
            "console.log(globalThis?.Math[\"pow\"](2, 3));\n",
        ),
        (
            "run",
            "main.ts",
            "console.log(globalThis?.[\"Math\"].pow(2, 3));\n",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('optional-chain pow', () => { console.log(Math?.pow(2, 3)); });\n",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('optional-chain pow', () => { console.log(Math?.pow(2, 3)); });\n",
        ),
        (
            "test",
            "smoke.test.jsx",
            "Kali.test('optional-chain pow', () => { console.log(Math?.pow(2, 3)); });\n",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('optional-chain pow', () => { console.log(Math?.pow(2, 3)); });\n",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('optional-chain pow', () => { console.log(globalThis?.Math[\"pow\"](2, 3)); });\n",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('optional-chain pow', () => { console.log(globalThis?.[\"Math\"].pow(2, 3)); });\n",
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
                assert_optional_chain_math_pow_rejection_json(errors);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert_optional_chain_math_pow_rejection_text(&stderr);
            }
        }
    }
}

#[test]
fn run_supports_math_sqrt_on_perfect_square_integer_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(4));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_log2_on_positive_power_of_two_integer_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.log2(8)); console.log(Object.freeze(Math.log2)(8));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_log10_on_positive_power_of_ten_integer_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.log10(1000)); console.log(Object.freeze(Math.log10)(1000));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_sin_and_cos_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.sin(zero)); console.log(Math.cos(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn run_and_test_supports_math_sin_cos_tan_zero_literals_with_transparent_aliases_in_js_input() {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const zero = 0; console.log(globalThis[\"Math\"].sin(zero)); console.log(Object.freeze(globalThis.Math).cos(zero)); console.log(Object.freeze(globalThis[\"Math\"][\"tan\"])(zero));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('math sin/cos/tan zero identities with aliases', () => { const zero = 0; console.log(globalThis[\"Math\"].sin(zero)); console.log(Object.freeze(globalThis.Math).cos(zero)); console.log(Object.freeze(globalThis[\"Math\"][\"tan\"])(zero)); });\n",
            "ok 1",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut command_builder = Command::new(kali_bin());
            command_builder.current_dir(dir.path());
            if output_json {
                command_builder.arg("--output").arg("json");
            }
            let output = command_builder
                .arg(command)
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
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                if command == "run" {
                    assert_eq!(json["exitCode"], 0);
                    assert_eq!(json["payload"]["exitCode"], 0);
                } else {
                    assert_eq!(json["payload"]["total"], 1);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["failed"], 0);
                }
                let stdout = json["stdout"].as_str().expect("stdout string");
                assert!(stdout.contains("1"), "json: {json}");
                assert!(stdout.contains("0"), "json: {json}");
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("1"), "stdout: {stdout}");
                assert!(stdout.contains("0"), "stdout: {stdout}");
                if command == "test" {
                    assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
                }
            }
        }
    }
}

#[test]
fn run_supports_math_exp_and_log_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.exp(zero)); console.log(Math.log(one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn run_supports_object_freeze_wrapped_math_and_number_roots_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn run_and_test_supports_global_this_math_exp_and_log_exact_identity_literals_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one));\n",
            "1",
        ),
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('globalThis math exp/log', () => { const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one)); });\n",
            "ok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('globalThis math exp/log', () => { const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one)); });\n",
            "ok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_exp_and_log_exact_identity_literals_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].exp(zero)); console.log(globalThis[\"Math\"].log(one));\n",
            "1",
        ),
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].exp(zero)); console.log(globalThis[\"Math\"].log(one));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed globalThis math exp/log', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].exp(zero)); console.log(globalThis[\"Math\"].log(one)); });\n",
            "1\n0\nok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed globalThis math exp/log', () => { const zero = 0; const one = 1; console.log(globalThis[\"Math\"].exp(zero)); console.log(globalThis[\"Math\"].log(one)); });\n",
            "1\n0\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_object_freeze_wrapped_math_exp_and_log_exact_identity_literals_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(Object.freeze(Math.exp)(zero)); console.log(Object.freeze(globalThis[\"Math\"][\"log\"])(one));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('frozen math exp/log', () => { const zero = 0; const one = 1; console.log(Object.freeze(Math.exp)(zero)); console.log(Object.freeze(globalThis[\"Math\"][\"log\"])(one)); });\n",
            "1\n0\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_object_freeze_wrapped_math_and_number_roots_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one));\n",
            "1",
        ),
        (
            "run",
            "main.js",
            "const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('frozen math roots', () => { const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one)); });\n",
            "1\n0\n1\n1\nok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('frozen math roots', () => { const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one)); });\n",
            "1\n0\n1\n1\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("0"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const value = 1.6; console.log(globalThis[\"Math\"][\"floor\"](value)); console.log(globalThis[\"Math\"][\"trunc\"](value)); console.log(globalThis[\"Math\"][\"ceil\"](value));\n",
            "1",
        ),
        (
            "run",
            "main.js",
            "const value = 1.6; console.log(globalThis[\"Math\"][\"floor\"](value)); console.log(globalThis[\"Math\"][\"trunc\"](value)); console.log(globalThis[\"Math\"][\"ceil\"](value));\n",
            "1",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed globalThis math floor/trunc/ceil', () => { const value = 1.6; console.log(globalThis[\"Math\"][\"floor\"](value)); console.log(globalThis[\"Math\"][\"trunc\"](value)); console.log(globalThis[\"Math\"][\"ceil\"](value)); });\n",
            "1\n1\n2\nok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed globalThis math floor/trunc/ceil', () => { const value = 1.6; console.log(globalThis[\"Math\"][\"floor\"](value)); console.log(globalThis[\"Math\"][\"trunc\"](value)); console.log(globalThis[\"Math\"][\"ceil\"](value)); });\n",
            "1\n1\n2\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1"),
                    "json: {json}"
                );
                assert!(
                    json["stdout"].as_str().expect("stdout").contains("2"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn json_run_supports_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_and_test_supports_math_pow_positive_integer_exponent_alias_chain_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias));\n",
            "8",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('pow alias', () => { const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias)); });\n",
            "8\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("8"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_pow_negative_one_base_positive_integer_exponent_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const exponent = 3; const alias = exponent; console.log(Math.pow(-1, alias));\n",
            "-1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('pow negative one base', () => { const exponent = 3; const alias = exponent; console.log(Math.pow(-1, alias)); });\n",
            "-1\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("-1"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_pow_negative_integer_exponents_for_unit_bases_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias)); console.log(Math.pow(-1, alias));\n",
            "1\n-1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('pow negative unit bases', () => { const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias)); console.log(Math.pow(-1, alias)); });\n",
            "1\n-1\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("1\n-1"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_pow_negative_integer_base_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const exponent = 3; console.log(Math.pow(-2, exponent));\n",
            "-8",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('pow negative base', () => { const exponent = 3; console.log(Math.pow(-2, exponent)); });\n",
            "-8\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("-8"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_atan2_zero_slice_when_browser_harness_is_configured_in_ts_input() {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('atan2 zero slice', () => { const zero = 0; const one = 1; console.log(Math.atan2(zero, one)); });\n",
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
                let json = parse_json_stdout(&output);
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
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_atan2_zero_slice_when_browser_harness_is_configured_in_tsx_input() {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.tsx",
            "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('atan2 zero slice', () => { const zero = 0; const one = 1; console.log(Math.atan2(zero, one)); });\n",
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
                let json = parse_json_stdout(&output);
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
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "const bump = () => { console.log(\"bump\"); return 2; }; console.log(Math.atan2(0, 1, bump()));\n",
            "0",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('atan2 trailing argument evaluation', () => { const bump = () => { console.log(\"bump\"); return 2; }; console.log(Math.atan2(0, 1, bump())); });\n",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("bump"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("bump"), "stdout: {stdout}");
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_pow_positive_integer_exponent_alias_chain_when_browser_harness_is_configured_in_ts_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias));\n",
            "8",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('pow alias', () => { const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias)); });\n",
            "8\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("8"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_global_this_math_pow_positive_integer_exponent_alias_chain_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.ts",
            "const exponent = 3; const alias = exponent; console.log(globalThis.Math.pow(2, alias));\n",
            "8",
        ),
        (
            "run",
            "main.js",
            "const exponent = 3; const alias = exponent; console.log(globalThis.Math.pow(2, alias));\n",
            "8",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('pow alias', () => { const exponent = 3; const alias = exponent; console.log(globalThis.Math.pow(2, alias)); });\n",
            "8\nok 1",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('pow alias', () => { const exponent = 3; const alias = exponent; console.log(globalThis.Math.pow(2, alias)); });\n",
            "8\nok 1",
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
                let json = parse_json_stdout(&output);
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
                    json["stdout"].as_str().expect("stdout").contains("8"),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_pow_frozen_callable_alias_inventory_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    let expected_line_count = math_pow_browser_alias_inventory_aliases().len();
    for (command, source_name, _extension) in [
        ("run", "main.js", "js"),
        ("run", "main.ts", "ts"),
        ("run", "main.jsx", "jsx"),
        ("run", "main.tsx", "tsx"),
        ("test", "smoke.test.js", "js"),
        ("test", "smoke.test.ts", "ts"),
        ("test", "smoke.test.jsx", "jsx"),
        ("test", "smoke.test.tsx", "tsx"),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            let body = math_pow_browser_alias_inventory_invocation_source();
            let source = if command == "test" {
                format!("Kali.test('pow browser alias inventory', () => {{\n{body}}});\n")
            } else {
                body
            };
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
                let json = parse_json_stdout(&output);
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
                let stdout = json["stdout"].as_str().expect("stdout");
                assert_eq!(
                    stdout.lines().filter(|line| *line == "8").count(),
                    expected_line_count,
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert_eq!(
                    stdout.lines().filter(|line| *line == "8").count(),
                    expected_line_count,
                    "stdout: {stdout}"
                );
            }
        }
    }
}

#[test]
fn run_supports_math_atan2_trailing_argument_evaluation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const bump = () => { console.log(\"bump\"); return 2; }; console.log(Math.atan2(0, 1, bump()));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bump"), "stdout: {stdout}");
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_inverse_hyperbolic_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0").count() >= 3, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_inverse_hyperbolic_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("0")
            .count()
            >= 3
    );
}

#[test]
fn run_supports_math_hyperbolic_zero_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.sinh(0));\nconsole.log(Math.cosh(0));\nconsole.log(Math.tanh(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
    assert!(stdout.matches("1").count() >= 1, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_hyperbolic_zero_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.sinh(0));\nconsole.log(Math.cosh(0));\nconsole.log(Math.tanh(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("0")
            .count()
            >= 2
    );
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("1")
            .count()
            >= 1
    );
}

#[test]
fn run_supports_math_inverse_trig_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.asin(0));\nconsole.log(Math.acos(1));\nconsole.log(Math.atan(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0").count() >= 3, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_inverse_trig_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.asin(0));\nconsole.log(Math.acos(1));\nconsole.log(Math.atan(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("0")
            .count()
            >= 3
    );
}

#[test]
fn run_supports_math_expm1_and_log1p_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_expm1_and_log1p_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("0")
            .count()
            >= 2
    );
}

#[test]
fn run_supports_math_exp2_exact_identity_literals_through_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.exp2(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("1").count() >= 1, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_exp2_exact_identity_literals_through_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.exp2(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("1")
            .count()
            >= 1
    );
}

#[test]
fn run_supports_math_expm1_and_log1p_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_expm1_and_log1p_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .matches("0")
            .count()
            >= 2
    );
}

#[test]
fn run_supports_math_log2_and_log10_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const log2Value = 8; const log2Alias = log2Value; console.log(Math.log2(log2Alias));\nconst log10Value = 1000; const log10Alias = log10Value; console.log(Math.log10(log10Alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_log2_and_log10_on_const_numeric_alias_chain_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const log2Value = 8; const log2Alias = log2Value; console.log(Math.log2(log2Alias));\nconst log10Value = 1000; const log10Alias = log10Value; console.log(Math.log10(log10Alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_log2_and_log10_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const log2Value = 8; const log2Alias = log2Value; console.log(Math.log2(log2Alias));\nconst log10Value = 1000; const log10Alias = log10Value; console.log(Math.log10(log10Alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_math_sqrt_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 4; const alias = value; console.log(Math.sqrt(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_sqrt_on_frozen_callable_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 4; console.log(Object.freeze(globalThis.Math.sqrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"sqrt\"])(value));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_sqrt_on_frozen_callable_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 4; console.log(Object.freeze(globalThis.Math.sqrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"sqrt\"])(value));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("2"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn run_supports_math_cbrt_on_negative_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = -27; const alias = value; console.log(Math.cbrt(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-3"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_hypot_zero_arguments_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.hypot());\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_hypot_zero_arguments_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.hypot());\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_math_sqrt_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2649110640673518\n", "stdout: {stdout}");
}

#[test]
fn run_and_test_rejects_additional_unsupported_math_member_calls_in_js_input() {
    for (expected_method, run_source, test_source) in [
        (
            "Math.exp",
            "console.log(Math.exp(1));\n",
            "Kali.test('unsupported math', () => { console.log(Math.exp(1)); });\n",
        ),
        (
            "Math.log",
            "console.log(Math.log(2));\n",
            "Kali.test('unsupported math', () => { console.log(Math.log(2)); });\n",
        ),
    ] {
        for (command, source_name, source) in [
            ("run", "main.js", run_source),
            ("test", "smoke.test.js", test_source),
        ] {
            for output_json in [false, true] {
                let dir = tempdir().expect("tempdir");
                let source_path = dir.path().join(source_name);
                fs::write(&source_path, source).expect("write source");

                let mut output = Command::new(kali_bin());
                output.current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                let output = output
                    .arg(command)
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
                    assert_unsupported_math_member_calls_rejection_json_for_method(
                        errors,
                        expected_method,
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    assert_unsupported_math_member_calls_rejection_text_for_method(
                        &stderr,
                        expected_method,
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_rejects_additional_unsupported_math_member_calls_in_browser_api_surface_with_harness_js_input(
) {
    for (expected_method, run_source, test_source) in [
        (
            "Math.exp",
            "console.log(Math.exp(1));\n",
            "Kali.test('unsupported math', () => { console.log(Math.exp(1)); });\n",
        ),
        (
            "Math.log",
            "console.log(Math.log(2));\n",
            "Kali.test('unsupported math', () => { console.log(Math.log(2)); });\n",
        ),
    ] {
        for (command, source_name, source) in [
            ("run", "main.js", run_source),
            ("test", "smoke.test.js", test_source),
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
                    assert_unsupported_math_member_calls_rejection_json_for_method(
                        errors,
                        expected_method,
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    assert_unsupported_math_member_calls_rejection_text_for_method(
                        &stderr,
                        expected_method,
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_rejects_negative_math_pow_exponents_in_browser_api_surface_with_harness_js_input() {
    for (command, source_name, source) in [
        ("run", "main.js", "console.log(Math.pow(2, -1));\n"),
        ("run", "main.jsx", "console.log(Math.pow(2, -1));\n"),
        ("run", "main.tsx", "console.log(Math.pow(2, -1));\n"),
        (
            "test",
            "smoke.test.js",
            "Kali.test('negative pow', () => { console.log(Math.pow(2, -1)); });\n",
        ),
        (
            "test",
            "smoke.test.jsx",
            "Kali.test('negative pow', () => { console.log(Math.pow(2, -1)); });\n",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('negative pow', () => { console.log(Math.pow(2, -1)); });\n",
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
                assert_unsupported_math_member_calls_rejection_json_for_method(errors, "Math.pow");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert_unsupported_math_member_calls_rejection_text_for_method(&stderr, "Math.pow");
            }
        }
    }
}

#[test]
fn run_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_run_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2649110640673518\n", "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

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

    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["stdout"], "1.2649110640673518\n");
}

#[test]
fn run_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2649110640673518\n", "stdout: {stdout}");
}

#[test]
fn json_run_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["stdout"], "1.2649110640673518\n");
}

#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_source_path = dir.path().join(format!("main.{extension}"));
        let test_source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &run_source_path,
            "console.log(globalThis[\"Math\"][\"sqrt\"](1.6));\n",
        )
        .expect("write run source");
        fs::write(
            &test_source_path,
            "Kali.test('supported math', () => { console.log(globalThis[\"Math\"][\"sqrt\"](1.6)); });\n",
        )
        .expect("write test source");

        for command in ["run", "test"] {
            let source_path = if command == "run" {
                &run_source_path
            } else {
                &test_source_path
            };
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                    .current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command).arg(source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`,
                // verified for the bracket/globalThis access forms too).
                assert!(
                    output.status.success(),
                    "stdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], true);
                    assert!(json["errors"].as_array().expect("errors array").is_empty());
                    assert!(
                        json["stdout"]
                            .as_str()
                            .expect("json stdout")
                            .contains("1.2649110640673518"),
                        "json: {json}"
                    );
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                    if command == "test" {
                        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                    }
                }
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_source_path = dir.path().join(format!("main.{extension}"));
        let test_source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &run_source_path,
            "console.log(globalThis[\"Math\"][\"sqrt\"](1.6));\n",
        )
        .expect("write run source");
        fs::write(
            &test_source_path,
            "Kali.test('supported math', () => { console.log(globalThis[\"Math\"][\"sqrt\"](1.6)); });\n",
        )
        .expect("write test source");
        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
        )
        .expect("write manifest");

        for command in ["run", "test"] {
            let source_path = if command == "run" {
                &run_source_path
            } else {
                &test_source_path
            };
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                    .current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command).arg(source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`,
                // verified for the bracket/globalThis access forms too).
                assert!(
                    output.status.success(),
                    "stdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], true);
                    assert!(json["errors"].as_array().expect("errors array").is_empty());
                    assert!(
                        json["stdout"]
                            .as_str()
                            .expect("json stdout")
                            .contains("1.2649110640673518"),
                        "json: {json}"
                    );
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                    if command == "test" {
                        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                    }
                }
            }
        }
    }
}

#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let run_source_path = dir.path().join("main.ts");
    let test_source_path = dir.path().join("smoke.test.ts");
    fs::write(&run_source_path, "console.log(Math.sqrt(1.6));\n").expect("write run source");
    fs::write(
        &test_source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
    )
    .expect("write test source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    for command in ["run", "test"] {
        let source_path = if command == "run" {
            &run_source_path
        } else {
            &test_source_path
        };
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg(source_path);
            let output = output.output().expect("run kali");

            // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
            // 1.2649110640673518 (bit-for-bit match with `kali run`).
            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
                assert!(
                    json["stdout"]
                        .as_str()
                        .expect("json stdout")
                        .contains("1.2649110640673518"),
                    "json: {json}"
                );
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                if command == "test" {
                    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                }
            }
        }
    }
}

#[test]
fn run_supports_nullish_coalescing_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = null ?? 1;
console.log(value);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn json_run_supports_nullish_coalescing_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = null ?? 1;
console.log(value);
"#,
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const voidFallback = void 0 ?? 1;
const undefinedFallback = undefined ?? 2;
console.log(voidFallback);
console.log(undefinedFallback);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2"), "stdout: {stdout}");
}

#[test]
fn json_run_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const voidFallback = void 0 ?? 1;
const undefinedFallback = undefined ?? 2;
console.log(voidFallback);
console.log(undefinedFallback);
"#,
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "1\n2\n");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_promise_all_settled_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    for command in ["run", "test"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg("--api").arg("browser");
            if command == "test" {
                let test_source = dir.path().join("smoke.test.js");
                fs::write(
                    &test_source,
                    "Kali.test('browser promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
                )
                .expect("write test source");
                output.arg(&test_source);
            } else {
                output.arg(&source_path);
            }
            let output = output.output().expect("run kali");

            assert!(output.status.success());
            assert_eq!(output.status.code(), Some(0));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("E5506"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn run_supports_promise_all_settled_in_inherited_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    for command in ["run", "test"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg("--api").arg("browser");
            if command == "test" {
                let test_source = dir.path().join("smoke.test.js");
                fs::write(
                    &test_source,
                    "Kali.test('browser promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
                )
                .expect("write test source");
                output.arg(&test_source);
            } else {
                output.arg(&source_path);
            }
            let output = output.output().expect("run kali");

            assert!(output.status.success());
            assert_eq!(output.status.code(), Some(0));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("E5506"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn run_supports_bracketed_promise_all_settled_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Promise\"][\"allSettled\"]([1, 2]));\n",
    )
    .expect("write source");

    for command in ["run", "test"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg("--api").arg("browser");
            if command == "test" {
                let test_source = dir.path().join("smoke.test.js");
                fs::write(
                    &test_source,
                    "Kali.test('browser promise allSettled', () => { return globalThis[\"Promise\"][\"allSettled\"]([1, 2]); });\n",
                )
                .expect("write test source");
                output.arg(&test_source);
            } else {
                output.arg(&source_path);
            }
            let output = output.output().expect("run kali");

            assert!(output.status.success());
            assert_eq!(output.status.code(), Some(0));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("E5506"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn run_supports_frozen_promise_all_settled_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Object.freeze(globalThis[\"Promise\"][\"allSettled\"])([1, 2]));\nconsole.log(Object.freeze((globalThis[\"Promise\"][\"allSettled\"]))([1, 2]));\nconsole.log(Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([1, 2]));\nconsole.log(Object.freeze((globalThis['Promise'])['allSettled'])([1, 2]));\nconsole.log(Object.freeze((globalThis[\"Promise\"]).allSettled)([1, 2]));\nconsole.log(Object.freeze((globalThis['Promise']).allSettled)([1, 2]));\n",
    )
    .expect("write source");

    for command in ["run", "test"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg("--api").arg("browser");
            if command == "test" {
                let test_source = dir.path().join("smoke.test.js");
                fs::write(
                    &test_source,
                    "Kali.test('browser promise allSettled', () => { return Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([1, 2]); });\n",
                )
                .expect("write test source");
                output.arg(&test_source);
            } else {
                output.arg(&source_path);
            }
            let output = output.output().expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.status.code(), Some(0));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("E5506"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn run_rejects_bracketed_promise_all_settled_in_inherited_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Promise\"][\"allSettled\"]([1, 2]));\n",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    for command in ["run", "test"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg("--api").arg("browser");
            if command == "test" {
                let test_source = dir.path().join("smoke.test.js");
                fs::write(
                    &test_source,
                    "Kali.test('browser promise allSettled', () => { return globalThis[\"Promise\"][\"allSettled\"]([1, 2]); });\n",
                )
                .expect("write test source");
                output.arg(&test_source);
            } else {
                output.arg(&source_path);
            }
            let output = output.output().expect("run kali");

            assert!(output.status.success());
            assert_eq!(output.status.code(), Some(0));
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("E5506"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn run_supports_for_of_array_iteration_in_browser_api_surface_with_harness_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const value of [1, 2, 3]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn run_supports_for_of_array_iteration_in_browser_api_surface_with_harness_ts_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const value of [1, 2, 3]) { console.log(value); }\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn run_supports_for_of_array_iteration_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn run_supports_for_of_array_iteration_with_const_boolean_alias_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = true; const alias = value; for (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("true"), "stdout: {stdout}");
}

#[test]
fn run_supports_for_of_array_iteration_with_const_alias_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn run_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn run_supports_for_of_array_iteration_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn run_supports_set_and_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, set_and_map_iteration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_run_supports_set_and_map_constructor_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, set_and_map_iteration_run_source()).expect("write source");

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

    // Flipped pin: try/catch/finally is rejected fail-closed (E5506): kali has no
    // exception machinery; the old lowering was an if-shaped miscompile that
    // only looked correct while `throw` was a silent no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_for_await_array_iteration_with_const_alias_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_with_const_boolean_alias_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = true; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("true"), "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
    assert_browser_for_await_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn run_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_await_array_iteration(&stdout);
}

#[test]
fn run_supports_for_await_array_iteration_with_const_string_alias_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"), "stdout: {stdout}");
}

#[test]
fn run_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "for await (const value of await [1, 2]) { console.log(value); }\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path())
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(0));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1"), "stdout: {stdout}");
        assert!(stdout.contains("2"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_for_of_template_literal_string_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const ch of `hello`) { console.log(ch); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_for_of_template_literal_iteration(&String::from_utf8_lossy(&output.stdout));
}

#[test]
fn run_supports_for_of_template_literal_string_iteration_in_js_input_with_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const ch of `hello`) { console.log(ch); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
}

#[test]
fn run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    assert_for_await_object_enumeration(&String::from_utf8_lossy(&output.stdout));
}

#[test]
fn json_run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_await_object_enumeration(&stdout);
}

#[test]
fn json_run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_await_object_enumeration(&stdout);
}

#[test]
fn json_run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_await_object_enumeration(&stdout);
}

#[test]
fn json_run_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
    )
    .expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn json_run_rejects_generator_function_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
}

#[test]
fn run_and_test_reject_generator_function_lowering_when_browser_harness_is_configured_in_js_input()
{
    for command in ["run", "test"] {
        for json_output in [false, true] {
            assert_generator_function_lowering_rejection_when_browser_harness_is_configured(
                command,
                "js",
                json_output,
                "function* main() { yield 1; }\nmain();",
            );
        }
    }
}

#[test]
fn run_and_test_reject_generator_and_async_generator_function_lowering_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                for source_contents in [
                    "function* main() { yield 1; }\nmain();",
                    "async function* main() { yield 1; }\nmain();",
                ] {
                    assert_generator_function_lowering_rejection_when_browser_harness_is_configured(
                        command,
                        extension,
                        json_output,
                        source_contents,
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_reject_array_callback_iteration_lowering_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                for source in array_callback_iteration_sources() {
                    assert_runtime_entrypoint_rejection(
                        command,
                        json_output,
                        extension,
                        source,
                        "literal array",
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_reject_array_callback_iteration_lowering_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for command in ["run", "test"] {
        for json_output in [false, true] {
            for extension in ["js", "ts", "jsx", "tsx"] {
                for source in array_callback_iteration_sources() {
                    assert_runtime_entrypoint_rejection_when_browser_harness_is_configured(
                        command,
                        json_output,
                        extension,
                        source,
                        &["array callback-produced iterables", "literal array"],
                    );
                }
            }
        }
    }
}

#[test]
fn run_and_test_reject_generator_function_expression_lowering_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                for (source, expected_message) in [
                    (
                        "const generatorExpr = function* generatorExpr() { yield 1; };\ngeneratorExpr;\n",
                        "generator function lowering is unavailable",
                    ),
                    (
                        "const asyncGeneratorExpr = async function* asyncGeneratorExpr() { yield 1; };\nasyncGeneratorExpr;\n",
                        "async-generator function lowering is unavailable",
                    ),
                ] {
                    assert_runtime_entrypoint_rejection(
                        command,
                        json_output,
                        extension,
                        source,
                        expected_message,
                    );
                }
            }
        }
    }
}

#[test]
fn run_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
        "class Example { *main() { yield* []; } }\nnew Example();",
        "class Example { async *main() { yield* []; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", false, "js", source);
    }
}

#[test]
fn run_rejects_class_generator_and_async_generator_method_lowering_in_ts_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", false, "ts", source);
    }
}

#[test]
fn run_rejects_class_generator_and_async_generator_method_lowering_in_jsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", false, "jsx", source);
    }
}

#[test]
fn run_rejects_class_generator_and_async_generator_method_lowering_in_tsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", false, "tsx", source);
    }
}

#[test]
fn json_run_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", true, "js", source);
    }
}

#[test]
fn json_run_rejects_class_generator_and_async_generator_method_lowering_in_ts_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("run", true, "ts", source);
    }
}

#[test]
fn run_and_test_reject_class_generator_and_async_generator_method_lowering_when_browser_harness_is_configured_in_js_input(
) {
    for command in ["run", "test"] {
        for json_output in [false, true] {
            for source in [
                "class Example { *main() { yield 1; } }\nnew Example();",
                "class Example { async *main() { yield 1; } }\nnew Example();",
            ] {
                assert_class_generator_method_lowering_rejection_when_browser_harness_is_configured(
                    command,
                    json_output,
                    "js",
                    source,
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_async_class_expressions_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "const Example = class NamedExample { async main() { return 1; } };\nnew Example().main();\n",
                    "async class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_generator_class_expressions_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n",
                    "generator class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_async_default_export_class_expressions_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "export default (class NamedExample { async main() { return 1; } });\n",
                    "async class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_async_generator_default_export_class_expressions_in_js_ts_jsx_and_tsx_input()
{
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "export default (class NamedExample { async *main() { yield 1; } });\n",
                    "async-generator class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_async_generator_class_expressions_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "const Example = class NamedExample { async *main() { yield 1; } };\nnew Example();\n",
                    "async-generator class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_generator_default_export_class_expressions_in_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    extension,
                    "export default (class NamedExample { *main() { yield 1; } });\n",
                    "generator class method lowering is unavailable in the direct runtime path",
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_wrapped_generator_and_async_generator_class_expressions_in_ts_input() {
    for (source, expected_message) in [
        (
            "const Example = (class NamedExample { *main() { yield 1; } }) as new () => any;\nnew Example();\n",
            "generator class method lowering is unavailable in the direct runtime path",
        ),
        (
            "const Example = (class NamedExample { async *main() { yield 1; } }) as new () => any;\nnew Example();\n",
            "async-generator class method lowering is unavailable in the direct runtime path",
        ),
        (
            "const Example = (class NamedExample { *main() { yield 1; } }) satisfies new () => any;\nnew Example();\n",
            "generator class method lowering is unavailable in the direct runtime path",
        ),
        (
            "const Example = (class NamedExample { async *main() { yield 1; } }) satisfies new () => any;\nnew Example();\n",
            "async-generator class method lowering is unavailable in the direct runtime path",
        ),
    ] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                assert_runtime_entrypoint_rejection(
                    command,
                    json_output,
                    "ts",
                    source,
                    expected_message,
                );
            }
        }
    }
}

#[test]
fn run_and_test_reject_sequence_wrapped_generator_and_async_generator_class_expressions_in_js_ts_jsx_and_tsx_input(
) {
    for (source, expected_message) in [
        (
            "const Example = (0, class NamedExample { *main() { yield* []; } });\nnew Example();\n",
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation",
        ),
        (
            "const Example = (0, class NamedExample { async *main() { yield* []; } });\nnew Example();\n",
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation",
        ),
    ] {
        for command in ["run", "test"] {
            for json_output in [false, true] {
                for extension in ["js", "ts", "jsx", "tsx"] {
                    assert_runtime_entrypoint_rejection(
                        command,
                        json_output,
                        extension,
                        source,
                        expected_message,
                    );
                }
            }
        }
    }
}

#[test]
fn run_rejects_async_generator_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("async-generator function lowering")
            || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_run_rejects_async_generator_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert!(errors.iter().any(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages.iter().any(
            |message| message.contains("async-generator function lowering")
                || message.contains("yield expressions")
        ),
        "messages: {messages:?}"
    );
}

#[test]
fn run_rejects_generator_delegating_yield_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_generator_function_lowering_in_jsx_input() {
    assert_generator_function_lowering_rejection("run", "jsx");
}

#[test]
fn run_rejects_generator_function_lowering_in_ts_input() {
    assert_generator_function_lowering_rejection("run", "ts");
}

#[test]
fn run_rejects_generator_function_lowering_in_tsx_input() {
    assert_generator_function_lowering_rejection("run", "tsx");
}

#[test]
fn run_rejects_generator_delegating_yield_lowering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_run_rejects_generator_function_lowering_in_ts_input() {
    assert_json_generator_function_lowering_rejection("run", "ts");
}

#[test]
fn json_run_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert!(errors.iter().any(|entry| entry["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|entry| entry["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.SharedArrayBuffer")),
        "messages: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.Atomics")),
        "messages: {messages:?}"
    );
}

#[test]
fn json_run_rejects_threaded_runtime_globals_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert!(errors.iter().any(|entry| entry["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|entry| entry["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.SharedArrayBuffer")),
        "messages: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.Atomics")),
        "messages: {messages:?}"
    );
}

#[test]
fn json_run_accepts_zero_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--max-threads")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
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
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5506");
    assert_eq!(
        json["errors"][0]["message"],
        "selected resource budget(s) [\"resources.maxThreads\"] are unavailable in this phase"
    );
    assert_eq!(json["errors"][0]["context"]["origin"], "cli");
    assert_eq!(json["errors"][0]["context"]["flag"], "--max-threads");
    assert_eq!(json["errors"][0]["context"]["requestedValue"], "1");
    assert_eq!(json["errors"][0]["context"]["effectiveValue"], "1");
}

#[test]
fn json_run_emits_a_command_envelope() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_run_supports_integer_like_key_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw 'unexpected numeric-key ordering';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_integer_like_key_ordering_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw 'unexpected numeric-key ordering';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
console.log('node run ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node run ok"), "stdout: {stdout}");
}

#[test]
fn run_surfaces_console_stdout_for_numeric_logs() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

#[test]
fn run_executes_package_bin_entrypoints_with_shebangs_after_stripping_the_shebang_line() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/hello-bin");
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "hello-bin",
  "version": "1.0.0",
  "bin": "bin/hello.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/hello.js"),
        "#!/usr/bin/env node\nconsole.log(1);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/hello.js"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

#[test]
fn run_executes_semver_style_package_bin_help_path_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package_fixture(&package_dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout, "Usage: semver [options] <version> [<version> [...]]\n",
        "stdout: {stdout}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_executes_semver_style_package_bin_argument_passthrough_on_node_api_surface() {
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
        "#!/usr/bin/env node\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(package_dir.join("bin/semver.js"))
        .arg("--")
        .arg("1.2.3")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

#[test]
fn run_executes_semver_style_package_bin_package_json_require_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_package_json_probe_fixture(&package_dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(package_dir.join("bin/semver.js"))
        .arg("--")
        .arg("1.2.3")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.0.0\n3\n");
}

#[test]
fn run_rejects_semver_style_package_bin_on_the_default_standalone_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_package_json_probe_fixture(&package_dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "expected the default standalone surface to reject a Node-only package bin\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("Node.js CLI features")
            && stderr.contains("unavailable on the 'deno' API surface"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_executes_semver_package_consumer_calls_on_the_default_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("index.js"),
        r#"export function valid(v) { return v; }
export function satisfies(version, range) { return version === '1.2.3' && range === '^1.0.0'; }
export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("main.ts"),
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(dir.path().join("main.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2.3\n1\n1.2.3\n", "stdout: {stdout}");
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn run_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_runtime_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("permission query const bindings ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn run_accepts_supported_permission_query_descriptor_const_bindings_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_runtime_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("permission query const bindings ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn run_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(
        stderr.contains("Process.EnvWrite")
            || stderr.contains("Process.Spawn")
            || stderr.contains("Network.Connect")
            || stderr.contains("Network.Listen"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_run_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E9007") | Some("E5506"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("Process.EnvWrite")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("Process.Spawn")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("Network.Connect")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("Network.Listen")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn run_with_sandbox_allows_a_benign_program() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('sandbox ok');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "sandbox ok\n");
}

#[test]
fn run_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
}

#[test]
fn json_run_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_accepts_positive_thread_budget_policy_when_threaded_profile_is_active() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_threaded_policy(&policy_path);

    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        let source_path = dir.path().join(filename);
        fs::write(&source_path, "console.log('thread policy');").expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--wasm-threads")
            .arg("--max-threads")
            .arg("1")
            .arg("--sandbox")
            .arg(&policy_path)
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
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
        assert_eq!(json["stdout"], "thread policy\n");
        assert_eq!(json["stderr"], "");
    }
}

#[test]
fn json_run_supports_integer_like_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_run_supports_integer_like_object_enumeration_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, browser_runtime_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin: the fixture's array-literal-argument self-check preamble is rejected
    // fail-closed (E5506): such arguments used to pass a zero placeholder, so
    // callee element reads silently yielded 0. The checks behind the preamble
    // never actually ran while `throw` was a no-op.
    assert!(
        !output.status.success(),
        "must be rejected fail-closed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn run_supports_number_predicates_in_ts_and_js_input() {
    for extension in ["ts", "js"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            r#"const alias = 1;
if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) {
  throw new Error('expected positive integer predicates');
}
if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) {
  throw new Error('expected negative primitive predicate cases');
}
if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) {
  throw new Error('expected bracketed Number predicate aliases');
}
console.log('number predicates ok');
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("number predicates ok"),
            "stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn json_run_supports_number_predicates_in_ts_and_js_input() {
    for extension in ["ts", "js"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            r#"const alias = 1;
if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) {
  throw new Error('expected positive integer predicates');
}
if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) {
  throw new Error('expected negative primitive predicate cases');
}
if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) {
  throw new Error('expected bracketed Number predicate aliases');
}
console.log('number predicates ok');
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("run")
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
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert!(json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("number predicates ok"));
    }
}
