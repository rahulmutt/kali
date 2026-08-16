use super::*;

#[test]
fn check_build_and_run_accept_global_this_deno_pid_in_js_input() {
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
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "stdout: {stdout}");
}

#[test]
fn json_check_accepts_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(globalThis.Deno.pid);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_check_accepts_bracketed_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.pid);\nconsole.log(globalThis[\"Deno\"][\"pid\"]);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_check_accepts_deno_cwd_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.cwd()); console.log(Deno[\"cwd\"]()); console.log(globalThis[\"Deno\"][\"cwd\"]());\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_check_accepts_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));\nconsole.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'));\nconsole.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_build_run_and_test_accept_deno_env_set_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"].set('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"].set('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"].env[\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"].set('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nconsole.log(Deno.env.get('KALI_ENV_SET_SMOKE'));\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg("--api")
            .arg("deno")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "{command} failed: {:?}", output);
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("deno")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "hello-environment", "stdout: {stdout}");

    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nKali.test('env mutation', () => { if (Deno.env.get('KALI_ENV_SET_SMOKE') !== 'hello-environment') { throw new Error('expected env mutation'); } });\n",
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_check_build_run_and_test_accept_deno_env_set_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"].set('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"].set('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nconsole.log(Deno.env.get('KALI_ENV_SET_SMOKE'));\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("--api")
            .arg("deno")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} failed: stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        match command {
            "check" => {
                assert_eq!(json["payload"]["filesChecked"], 1);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
            "build" => {
                let payload = json["payload"].as_object().expect("build payload object");
                assert_eq!(payload["artifactKind"], "executable");
                assert_eq!(payload["buildMode"], "fast");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
                let output_path =
                    PathBuf::from(payload["outputPath"].as_str().expect("output path"));
                assert_eq!(output_path, source_path.with_extension("wasm"));
                assert!(
                    output_path.exists(),
                    "expected build artifact at {output_path:?}"
                );
                assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
                assert!(payload["sourceHash"].as_str().is_some());
            }
            _ => unreachable!("unexpected command {command}"),
        }
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("deno")
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
    assert_eq!(json["stdout"], "hello-environment\n");
    assert_eq!(json["stderr"], "");

    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_SET_SMOKE', 'hello-environment');\nKali.test('env mutation', () => { if (Deno.env.get('KALI_ENV_SET_SMOKE') !== 'hello-environment') { throw new Error('expected env mutation'); } });\n",
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(&test_path)
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn check_build_run_and_test_accept_deno_env_delete_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno[\"env\"].set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"].set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"].env[\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"].set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno.env.delete('KALI_ENV_DELETE_SMOKE');\nDeno[\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE');\nDeno[\"env\"].delete('KALI_ENV_DELETE_SMOKE');\nglobalThis.Deno[\"env\"].delete('KALI_ENV_DELETE_SMOKE');\nglobalThis.Deno[\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE');\nglobalThis[\"Deno\"].env[\"delete\"]('KALI_ENV_DELETE_SMOKE');\nglobalThis[\"Deno\"][\"env\"].delete('KALI_ENV_DELETE_SMOKE');\nglobalThis[\"Deno\"][\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE');\nif (Deno.env.get('KALI_ENV_DELETE_SMOKE') !== void 0) { throw new Error('expected env deletion'); }\nconsole.log('deleted');\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg("--api")
            .arg("deno")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "{command} failed: {:?}", output);
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("deno")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "deleted", "stdout: {stdout}");

    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Kali.test('env delete', () => { Deno.env.set('KALI_ENV_DELETE_SMOKE', 'hello-environment'); Deno[\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment'); globalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); Deno[\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE'); globalThis[\"Deno\"][\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE'); if (Deno.env.get('KALI_ENV_DELETE_SMOKE') !== void 0) { throw new Error('expected env deletion'); } });\n",
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_check_build_run_and_test_accept_deno_env_delete_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno[\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno[\"env\"].set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis.Deno[\"env\"].set('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nglobalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment');\nDeno.env.delete('KALI_ENV_DELETE_SMOKE');\nDeno[\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE');\nDeno[\"env\"].delete('KALI_ENV_DELETE_SMOKE');\nglobalThis.Deno[\"env\"].delete('KALI_ENV_DELETE_SMOKE');\nglobalThis[\"Deno\"][\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE');\nif (Deno.env.get('KALI_ENV_DELETE_SMOKE') !== void 0) { throw new Error('expected env deletion'); }\nconsole.log('deleted');\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("--api")
            .arg("deno")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} failed: stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        match command {
            "check" => {
                assert_eq!(json["payload"]["filesChecked"], 1);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
            "build" => {
                let payload = json["payload"].as_object().expect("build payload object");
                assert_eq!(payload["artifactKind"], "executable");
                assert_eq!(payload["buildMode"], "fast");
                assert!(json["errors"].as_array().expect("errors array").is_empty());
                let output_path =
                    PathBuf::from(payload["outputPath"].as_str().expect("output path"));
                assert_eq!(output_path, source_path.with_extension("wasm"));
                assert!(
                    output_path.exists(),
                    "expected build artifact at {output_path:?}"
                );
                assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
                assert!(payload["sourceHash"].as_str().is_some());
            }
            _ => unreachable!("unexpected command {command}"),
        }
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("deno")
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
    assert_eq!(json["stdout"], "deleted\n");
    assert_eq!(json["stderr"], "");

    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Kali.test('env delete', () => { Deno.env.set('KALI_ENV_DELETE_SMOKE', 'hello-environment'); Deno[\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment'); globalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_DELETE_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); Deno[\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE'); globalThis[\"Deno\"][\"env\"][\"delete\"]('KALI_ENV_DELETE_SMOKE'); if (Deno.env.get('KALI_ENV_DELETE_SMOKE') !== void 0) { throw new Error('expected env deletion'); } });\n",
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(&test_path)
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn check_build_and_run_accept_bracketed_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"pid\"]);\n",
    )
    .expect("write source");

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
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_bracketed_global_this_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"pid\"]);\n",
    )
    .expect("write source");

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
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));\n",
    )
    .expect("write source");

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
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "hello-environment", "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_deno_env_has_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));\n",
    )
    .expect("write source");

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
        .env("KALI_ENV_HAS_SMOKE", "hello-environment")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "true", "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_deno_env_has_in_jsx_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));\n",
        )
        .expect("write source");

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
            .env("KALI_ENV_HAS_SMOKE", "hello-environment")
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "run failed: {:?}", output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "true", "stdout: {stdout}");
    }
}

#[test]
fn check_build_and_run_accept_type_assertion_and_satisfies_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 'ok' as string;\nconst echoed = value satisfies unknown;\nconsole.log(echoed);\n",
    )
    .expect("write source");

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
    assert_eq!(stdout.trim(), "ok", "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_wrapped_mutable_update_targets_in_ts_input() {
    for command in ["check", "build", "run"] {
        assert_wrapped_mutable_update_targets(command, "main.ts");
    }
}

#[test]
fn check_build_and_run_accept_wrapped_mutable_compound_assignment_targets_in_ts_input() {
    for command in ["check", "build", "run"] {
        assert_wrapped_mutable_compound_assignment_targets(command, "main.ts");
    }
}

#[test]
fn check_build_and_run_accept_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno.pid);\n").expect("write source");

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
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "stdout: {stdout}");
}

#[test]
fn check_build_and_run_accept_bracketed_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno[\"pid\"]);\n").expect("write source");

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
    let pid = stdout.trim().parse::<u32>().expect("pid stdout");
    assert!(pid > 0, "stdout: {stdout}");
}

#[test]
fn check_build_run_and_test_accept_deno_filesystem_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let test_path = dir.path().join("main.test.js");
    fs::write(dir.path().join("input.txt"), "alpha").expect("write input");
    fs::write(dir.path().join("open.txt"), "beta").expect("write open input");
    fs::write(
        &source_path,
        "Deno.mkdir('./nested', false);\nDeno.rename('./input.txt', './nested/renamed.txt');\nDeno.lstat('./nested/renamed.txt');\nDeno.remove('./nested/renamed.txt');\nDeno.remove('./nested', true);\nDeno.open('./open.txt');\nDeno.create('./created.txt');\nconsole.log('done');\n",
    )
    .expect("write source");
    fs::write(
        &test_path,
        "Kali.test('filesystem', () => { Deno.mkdir('./nested', false); Deno.rename('./input.txt', './nested/renamed.txt'); Deno.lstat('./nested/renamed.txt'); Deno.remove('./nested/renamed.txt'); Deno.remove('./nested', true); Deno.open('./open.txt'); Deno.create('./created.txt'); console.log('done'); });\n",
    )
    .expect("write test source");

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
    assert_eq!(stdout.trim(), "done", "stdout: {stdout}");

    fs::write(dir.path().join("input.txt"), "alpha").expect("reset json run input");

    let json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&json_output.stdout),
        String::from_utf8_lossy(&json_output.stderr)
    );
    let json = parse_json_stdout(&json_output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("done"));
    assert_eq!(json["stderr"], "");

    fs::write(dir.path().join("input.txt"), "alpha").expect("reset test input");

    let test_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        test_output.status.success(),
        "test failed: {:?}",
        test_output
    );
    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    assert!(test_stdout.contains("done"), "stdout: {test_stdout}");

    let test_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        test_json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json_output.stdout),
        String::from_utf8_lossy(&test_json_output.stderr)
    );
    let test_json = parse_json_stdout(&test_json_output);
    assert_eq!(test_json["schemaVersion"], 1);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("test stdout")
            .contains("done"),
        "json test: {test_json}"
    );
}

#[test]
fn check_build_run_and_test_accept_deno_filesystem_apis_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        assert_deno_filesystem_apis_in_input(extension);
    }
}

#[test]
fn check_accepts_a_resolved_file() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
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
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_uses_explicit_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
}

#[test]
fn check_uses_inherited_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
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
        .arg("check")
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
fn check_uses_inherited_browser_api_surface_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser sandbox ok');").expect("write source");
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
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn json_check_uses_explicit_browser_api_surface_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser sandbox ok');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
console.log('ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_accepts_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_rejects_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("wasm-threads"), "stderr: {stderr}");
}

#[test]
fn check_rejects_browser_api_surface_with_wasm_threads_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("wasm-threads"), "stderr: {stderr}");
}

#[test]
fn check_accepts_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
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
        .arg("check")
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
fn check_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
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
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn check_rejects_threaded_runtime_globals_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
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
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn check_accepts_node_api_surface_with_human_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "import 'node:path';\nconsole.log('ok');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_accepts_node_api_surface_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "import 'node:path';\nconsole.log('ok');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("node")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_accepts_process_argv_slice_length_in_js_input_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(process.argv.slice(2).length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_rejects_permission_escalation_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Deno.permissions.request(); Deno.permissions.revoke(); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke();",
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
        stderr.contains("Deno.permissions.request") && stderr.contains("Deno.permissions.revoke"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_permission_escalation_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Deno.permissions.request(); Deno.permissions.revoke(); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke();",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 4);
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    for expected in [
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
    ] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {:?}",
            errors
        );
    }
}

#[test]
fn check_rejects_computed_permission_escalation_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
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
        stderr.contains("globalThis.Deno.permissions.request")
            && stderr.contains("globalThis.Deno.permissions.revoke"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_computed_permission_escalation_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    for expected in [
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
    ] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {:?}",
            errors
        );
    }
}

#[test]
fn check_rejects_bracketed_permission_escalation_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno["permissions"]["request"](); Deno["permissions"]["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
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
        stderr.contains("Deno.permissions.request") && stderr.contains("Deno.permissions.revoke"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_bracketed_permission_escalation_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno["permissions"]["request"](); Deno["permissions"]["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 4);
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    for expected in ["Deno.permissions.request", "Deno.permissions.revoke"] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {:?}",
            errors
        );
    }
}

#[test]
fn check_rejects_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_permission_escalation_stderr(
        &stderr,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
    );
}

#[test]
fn check_rejects_permission_escalation_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_permission_escalation_json(
        errors,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
        12,
    );
}

#[test]
fn check_rejects_computed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_computed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_permission_escalation_stderr(
        &stderr,
        &[
            "globalThis.Deno.permissions.request",
            "globalThis.Deno.permissions.revoke",
        ],
    );
}

#[test]
fn check_rejects_computed_permission_escalation_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_computed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_permission_escalation_json(
        errors,
        &[
            "globalThis.Deno.permissions.request",
            "globalThis.Deno.permissions.revoke",
        ],
        2,
    );
}

#[test]
fn check_rejects_bracketed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_permission_escalation_stderr(
        &stderr,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
    );
}

#[test]
fn check_rejects_bracketed_permission_escalation_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_permission_escalation_json(
        errors,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
        6,
    );
}

#[test]
fn check_rejects_mixed_bracket_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_mixed_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_permission_escalation_stderr(
        &stderr,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
    );
}

#[test]
fn check_rejects_mixed_bracket_permission_escalation_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_mixed_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_permission_escalation_json(
        errors,
        &["Deno.permissions.request", "Deno.permissions.revoke"],
        4,
    );
}

#[test]
fn check_rejects_late_process_control_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis.process.cwd; globalThis[\"process\"][\"cwd\"]; process.chdir; globalThis.process.chdir; globalThis[\"process\"][\"chdir\"]; process.exit; globalThis[\"process\"][\"exit\"];",
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
        "process.exit",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_late_process_control_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis.process.cwd; globalThis[\"process\"][\"cwd\"]; process.chdir; globalThis.process.chdir; globalThis[\"process\"][\"chdir\"]; process.exit; globalThis[\"process\"][\"exit\"];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
fn check_rejects_broader_intl_support() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

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
    assert!(stderr.contains("Intl"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Intl"), "stderr: {stderr}");
    assert!(stderr.contains(r#"globalThis["Intl"]"#), "stderr: {stderr}");
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
fn check_rejects_broader_intl_support_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(errors[0]["code"], "E5506");
    assert!(errors[0]["message"]
        .as_str()
        .expect("error message")
        .contains("Intl"));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("globalThis.Intl")));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]"#)));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]["NumberFormat"]"#)));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]["DateTimeFormat"]"#)));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("globalThis.Intl.PluralRules")));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]["PluralRules"]"#)));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("globalThis.Intl.Collator")));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("globalThis.Intl.Locale")));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]["Collator"]"#)));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains(r#"globalThis["Intl"]["Locale"]"#)));
}

#[test]
fn check_supports_late_env_materialization_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "check should succeed: {:?}",
        output
    );
}

#[test]
fn check_supports_late_env_materialization_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "check json should succeed: {:?}",
        output
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], true);
    assert!(
        json["errors"].as_array().expect("errors array").is_empty(),
        "unexpected errors: {json:?}"
    );
}

#[test]
fn check_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
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
    assert!(stderr.contains("Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("WeakMap"), "stderr: {stderr}");
    assert!(stderr.contains("WeakSet"), "stderr: {stderr}");
    assert!(stderr.contains("FinalizationRegistry"), "stderr: {stderr}");
}

#[test]
fn check_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 8);
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
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn check_rejects_late_object_model_globals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
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
    assert!(stderr.contains("Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("WeakMap"), "stderr: {stderr}");
    assert!(stderr.contains("WeakSet"), "stderr: {stderr}");
    assert!(stderr.contains("FinalizationRegistry"), "stderr: {stderr}");
}

#[test]
fn check_rejects_late_object_model_globals_in_browser_analysis_context() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
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
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_late_object_model_globals_in_browser_analysis_context_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
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
fn check_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("check")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 70 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_discovers_fixture_tree_from_cwd_but_stops_at_nested_child_projects() {
    let dir = tempdir().expect("tempdir");
    let root_source = dir.path().join("main.ts");
    fs::write(&root_source, "const ok = 1;\nok;\n").expect("write root source");

    let child_project = dir.path().join("child");
    fs::create_dir(&child_project).expect("create child project directory");
    fs::write(child_project.join("kali.json"), "{}\n").expect("write child manifest");
    fs::write(child_project.join("bad.ts"), "missing;\n").expect("write child source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_reports_unresolved_identifiers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "missing;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
}

#[test]
fn check_reports_unresolved_identifiers_inside_default_export_function_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(stderr.contains("missing"), "stderr: {stderr}");
}

#[test]
fn json_check_reports_unresolved_identifiers_inside_default_export_function_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert_eq!(json["payload"]["filesChecked"], 1);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "json: {json}");
    assert!(errors.iter().any(|error| error["code"] == "E3100"));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("missing")));
}

#[test]
fn check_accepts_compat_eval_flag() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_accepts_inherited_compat_eval_feature_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const body = \"return \" + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const body = \"return \" + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_rejects_eval_without_compat_eval() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "if (eval(\"1 + 2\") !== 3) { throw new Error('bad eval result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("compatibility feature 'eval'"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_with_sandbox_rejects_invalid_policy_schema() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('policy check');").expect("write source");
    write_invalid_policy_schema(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5510"), "stderr: {stderr}");
    assert!(stderr.contains("unknown field"), "stderr: {stderr}");
}

#[test]
fn check_rejects_non_literal_dynamic_import_targets() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let specifier; import(specifier);").expect("write source");

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
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_non_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "let specifier; import(specifier);").expect("write source");

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
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_supports_nullish_coalescing_in_js_input() {
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_nullish_coalescing_in_js_input() {
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_nullish_assignment_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_nullish_assignment_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_rejects_compound_assignment_on_non_local_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, compound_assignment_non_local_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_compound_assignment_rejection_text(
        &stderr,
        "compound assignment lowering is unavailable unless the target is a mutable local binding",
    );
}

#[test]
fn json_check_rejects_compound_assignment_on_non_local_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, compound_assignment_non_local_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_compound_assignment_rejection_json(
        errors,
        "compound assignment lowering is unavailable unless the target is a mutable local binding",
    );
}

#[test]
fn check_supports_nullish_assignment_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_nullish_assignment_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, nullish_assignment_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_check_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_check_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_supports_promise_all_settled_source_variants_in_js_input() {
    for source in promise_all_settled_source_variants() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("check")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success());
        assert_eq!(output.status.code(), Some(0));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("E5506"), "stderr: {stderr}");
    }
}

#[test]
fn check_supports_math_sqrt_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_rejects_additional_unsupported_math_member_calls_in_js_input() {
    for (source, expected_method) in [
        ("console.log(Math.exp(1));\n", "Math.exp"),
        ("console.log(Math.log(2));\n", "Math.log"),
    ] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).expect("write source");

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
        assert!(stderr.contains(expected_method), "stderr: {stderr}");
        assert!(stderr.contains("later compatibility"), "stderr: {stderr}");
    }
}

#[test]
fn check_rejects_negative_math_pow_exponents_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.pow(2, -1));\n").expect("write source");

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
        stderr.contains("Math.pow is unavailable for negative numeric literals"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("later compatibility"), "stderr: {stderr}");
}

#[test]
fn check_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_math_sqrt_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["filesChecked"], 1);
}

#[test]
fn json_check_rejects_additional_unsupported_math_member_calls_in_js_input() {
    for (source, expected_method) in [
        ("console.log(Math.exp(1));\n", "Math.exp"),
        ("console.log(Math.log(2));\n", "Math.log"),
    ] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("check")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
        assert!(errors.iter().any(|error| {
            error["message"]
                .as_str()
                .expect("error message")
                .contains(expected_method)
        }));
    }
}

#[test]
fn json_check_rejects_negative_math_pow_exponents_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.pow(2, -1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(errors.iter().any(|error| {
        error["message"]
            .as_str()
            .expect("error message")
            .contains("Math.pow is unavailable for negative numeric literals")
    }));
    let source_file = source_path.to_string_lossy();
    assert_eq!(errors[0]["file"], json!(source_file.as_ref()));
    assert_eq!(errors[0]["span"]["file"], json!(source_file.as_ref()));
}

#[test]
fn check_and_build_reject_optional_chain_wrapped_math_pow_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for (command, extension, source) in [
        ("check", "js", "console.log(Math?.pow(2, 3));\n"),
        ("check", "ts", "console.log(Math?.pow(2, 3));\n"),
        ("check", "jsx", "console.log(Math?.pow(2, 3));\n"),
        ("check", "tsx", "console.log(Math?.pow(2, 3));\n"),
        ("build", "js", "console.log(Math?.pow(2, 3));\n"),
        ("build", "ts", "console.log(Math?.pow(2, 3));\n"),
        ("build", "jsx", "console.log(Math?.pow(2, 3));\n"),
        ("build", "tsx", "console.log(Math?.pow(2, 3));\n"),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(format!("main.{extension}"));
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            let output = output
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
fn check_and_build_reject_global_this_optional_chain_wrapped_math_pow_in_browser_api_surface_in_js_input(
) {
    for (command, source) in [
        ("check", "console.log(globalThis.Math?.pow(2, 3));\n"),
        ("check", "console.log(globalThis?.Math.pow(2, 3));\n"),
        ("check", "console.log(globalThis?.Math[\"pow\"](2, 3));\n"),
        ("check", "console.log(globalThis?.[\"Math\"].pow(2, 3));\n"),
        (
            "check",
            "console.log(globalThis?.[\"Math\"][\"pow\"](2, 3));\n",
        ),
        (
            "check",
            "console.log(globalThis?.[\"Math\"]['pow'](2, 3));\n",
        ),
        ("check", "console.log(globalThis?.['Math'].pow(2, 3));\n"),
        (
            "check",
            "console.log(globalThis?.['Math'][\"pow\"](2, 3));\n",
        ),
        ("check", "console.log(globalThis?.['Math']['pow'](2, 3));\n"),
        ("build", "console.log(globalThis.Math?.pow(2, 3));\n"),
        ("build", "console.log(globalThis?.Math.pow(2, 3));\n"),
        ("build", "console.log(globalThis?.Math[\"pow\"](2, 3));\n"),
        ("build", "console.log(globalThis?.[\"Math\"].pow(2, 3));\n"),
        (
            "build",
            "console.log(globalThis?.[\"Math\"][\"pow\"](2, 3));\n",
        ),
        (
            "build",
            "console.log(globalThis?.[\"Math\"]['pow'](2, 3));\n",
        ),
        ("build", "console.log(globalThis?.['Math'].pow(2, 3));\n"),
        (
            "build",
            "console.log(globalThis?.['Math'][\"pow\"](2, 3));\n",
        ),
        ("build", "console.log(globalThis?.['Math']['pow'](2, 3));\n"),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("main.js");
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            let output = output
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
fn json_check_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_non_integer_numeric_literals_in_math_trunc_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.trunc(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_non_integer_numeric_literals_in_math_trunc_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.trunc(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_math_floor_numeric_literal_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.floor(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
}

#[test]
fn json_check_supports_math_floor_numeric_literal_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.floor(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = 1.6; const alias = value; console.log(Math.floor(alias));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
}

#[test]
fn json_check_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const value = 1.6; const alias = value; console.log(Math.floor(alias));
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_math_sqrt_member_calls_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn json_check_supports_math_sqrt_member_calls_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["filesChecked"], 1);
}

#[test]
fn check_supports_math_sqrt_member_calls_in_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

        for command in ["check", "build"] {
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output.current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command);
                if command == "build" {
                    output.arg("--bundle");
                }
                output.arg("--api").arg("browser").arg(&source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`).
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
                }
            }
        }
    }
}

#[test]
fn check_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_js_input() {
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
        .current_dir(dir.path())
        .arg("check")
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
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn json_check_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_js_input() {
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
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["filesChecked"], 1);
}

#[test]
fn check_supports_math_sin_and_cos_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.sin(zero)); console.log(Math.cos(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_math_sin_and_cos_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.sin(zero)); console.log(Math.cos(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_math_tan_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.tan(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn check_rejects_math_tan_non_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.tan(1));\n").expect("write source");

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
    assert!(stderr.contains("Math.tan"), "stderr: {stderr}");
}

#[test]
fn check_rejects_math_sin_and_cos_non_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.sin(1)); console.log(Math.cos(1));\n",
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
    assert!(stderr.contains("Math.sin"), "stderr: {stderr}");
    assert!(stderr.contains("Math.cos"), "stderr: {stderr}");
}

#[test]
fn json_check_rejects_math_sin_and_cos_non_zero_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Math.sin(1)); console.log(Math.cos(1));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("Math.sin")));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("Math.cos")));
}

#[test]
fn check_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn json_check_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
}

#[test]
fn check_supports_math_hypot_on_perfect_square_integer_literal_sums_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.hypot(3, 4));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn check_supports_promise_all_settled_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    for command in ["check", "build"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            output.arg("--api").arg("browser").arg(&source_path);
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
fn check_supports_promise_all_settled_in_inherited_browser_api_surface_in_js_input() {
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

    for command in ["check", "build"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            output.arg(&source_path);
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
fn check_supports_bracketed_promise_all_settled_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Promise\"][\"allSettled\"]([1, 2]));\n",
    )
    .expect("write source");

    for command in ["check", "build"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            output.arg("--api").arg("browser").arg(&source_path);
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
fn check_supports_bracketed_promise_all_settled_in_inherited_browser_api_surface_in_js_input() {
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

    for command in ["check", "build"] {
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command);
            if command == "build" {
                output.arg("--bundle");
            }
            output.arg(&source_path);
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
fn check_supports_frozen_object_enumeration_spread_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            browser_runtime_frozen_object_enumeration_spread_source(),
        )
        .expect("write source");

        for command in ["check", "build"] {
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output.current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command);
                if command == "build" {
                    output.arg("--bundle");
                }
                output.arg("--api").arg("browser").arg(&source_path);
                let output = output.output().expect("run kali");

                // Throw-fallout Stage 2 selection-callee drain: `build`'s
                // former E5506 backstop hits on this source came from its
                // three short-circuit frozen-callable-selection lines
                // (`null ??`/`true &&`/`false ||` over
                // `globalThis["Object"]["entries"]`), which the enumeration
                // fold now resolves through the shared static callable
                // oracle. The whole source now compiles and executes with
                // node-identical output (see the sibling `run`/`test`
                // execution variants of this exact source, drained green) —
                // so `build` succeeds like `check` (node-verified un-flip).
                assert!(
                    output.status.success(),
                    "stderr: {}",
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
}

#[test]
fn check_supports_frozen_object_enumeration_spread_in_inherited_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            browser_runtime_frozen_object_enumeration_spread_source(),
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

        for command in ["check", "build"] {
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output.current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command);
                if command == "build" {
                    output.arg("--bundle");
                }
                output.arg(&source_path);
                let output = output.output().expect("run kali");

                // Throw-fallout Stage 2 selection-callee drain: see the
                // sibling (explicit `--api browser`) test above — the former
                // `build` E5506 backstop hits were the short-circuit
                // frozen-callable-selection lines, which now fold via the
                // shared static callable oracle; `build` succeeds like
                // `check` (node-verified un-flip).
                assert!(
                    output.status.success(),
                    "stderr: {}",
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
}

#[test]
fn check_rejects_generator_function_lowering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

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
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_generator_function_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

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
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_generator_function_lowering_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
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
fn check_rejects_generator_function_lowering_in_browser_api_surface_js_input() {
    assert_generator_function_lowering_rejection_in_browser_context(
        "check",
        false,
        "js",
        "function* main() { yield 1; }\nmain();",
    );
}

#[test]
fn check_rejects_generator_and_async_generator_function_lowering_in_browser_api_surface_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for source_contents in [
            "function* main() { yield 1; }\nmain();",
            "async function* main() { yield 1; }\nmain();",
        ] {
            assert_generator_function_lowering_rejection_in_browser_context(
                "check",
                false,
                extension,
                source_contents,
            );
        }
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
        "class Example { *main() { yield* []; } }\nnew Example();",
        "class Example { async *main() { yield* []; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("check", false, "js", source);
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_jsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("check", false, "jsx", source);
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_tsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("check", false, "tsx", source);
    }
}

#[test]
fn json_check_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("check", true, "js", source);
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_js_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "check", false, false, "js", source,
        );
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_jsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "check", false, false, "jsx", source,
        );
    }
}

#[test]
fn check_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_tsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "check", false, false, "tsx", source,
        );
    }
}

#[test]
fn json_check_emits_diagnostic_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "missing;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_rejects_fix_flag_as_later_compatibility() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("lint --fix"), "stderr: {stderr}");
}

#[test]
fn json_check_rejects_fix_flag_as_later_compatibility() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5506");
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("error message")
        .contains("lint --fix"));
}

#[test]
fn json_check_accepts_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--wasm-threads")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_check_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
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
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(errors[0]["message"]
        .as_str()
        .expect("error message")
        .contains("duplicate runtimeProfile"));
}

#[test]
fn json_check_rejects_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_check_rejects_browser_api_surface_with_wasm_threads_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn json_check_rejects_inherited_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser",
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert!(errors[0]["message"]
        .as_str()
        .expect("error message")
        .contains("runtime profile"));
}

#[test]
fn json_check_rejects_threaded_runtime_globals() {
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 2, "errors: {errors:?}");
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
fn json_check_rejects_threaded_runtime_globals_js_input() {
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
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 2, "errors: {errors:?}");
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
fn check_accepts_permissions_query_subset_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
Deno.permissions.query({ "name": "read" });
Deno.permissions["query"]({ "name": "read" });
Deno.permissions.query({ "name": "write" });
Deno.permissions["query"]({ "name": "write" });
Deno.permissions.query({ "name": "env" });
Deno.permissions["query"]({ "name": "env" });
Deno.permissions.query({ "name": "net" });
Deno.permissions["query"]({ "name": "net" });
globalThis["Deno"]["permissions"].query({ "name": "read" });
globalThis["Deno"]["permissions"]["query"]({ "name": "read" });
globalThis["Deno"]["permissions"].query({ "name": "write" });
globalThis["Deno"]["permissions"]["query"]({ "name": "write" });
globalThis["Deno"]["permissions"].query({ "name": "env" });
globalThis["Deno"]["permissions"]["query"]({ "name": "env" });
globalThis["Deno"]["permissions"].query({ "name": "net" });
globalThis["Deno"]["permissions"]["query"]({ "name": "net" });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_with_sandbox_rejects_inferred_effects() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
}

#[test]
fn check_with_sandbox_rejects_inferred_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
}

#[test]
fn json_check_with_sandbox_rejects_inferred_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E9007") | Some("E5506"))),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn check_build_run_and_test_accept_zero_budget_policy_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('sandbox ok');\n").expect("write source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Kali.test('sandbox ok', () => { if (1 + 1 !== 2) { throw new Error('expected ok'); } });\n",
    )
    .expect("write test source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} stdout: {}\n{command} stderr: {}",
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
                    source_path.with_extension("wasm").exists(),
                    "expected build artifact"
                );
            }
            _ => unreachable!(),
        }
    }

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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "sandbox ok\n", "stdout: {stdout}");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_check_build_run_and_test_accept_zero_budget_policy_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('sandbox ok');\n").expect("write source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        "Kali.test('sandbox ok', () => { if (1 + 1 !== 2) { throw new Error('expected ok'); } });\n",
    )
    .expect("write test source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "{command} stdout: {}\n{command} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        match command {
            "check" => {
                assert_eq!(json["payload"]["filesChecked"], 1);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
            "build" => {
                let payload = json["payload"].as_object().expect("build payload object");
                assert_eq!(payload["artifactKind"], "executable");
                assert_eq!(payload["buildMode"], "fast");
                let output_path =
                    PathBuf::from(payload["outputPath"].as_str().expect("output path"));
                assert_eq!(output_path, source_path.with_extension("wasm"));
                assert!(
                    output_path.exists(),
                    "expected build artifact at {output_path:?}"
                );
                assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
                assert!(payload["sourceHash"].as_str().is_some());
            }
            _ => unreachable!(),
        }
    }

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
    assert_eq!(json["stdout"], "sandbox ok\n");
    assert_eq!(json["stderr"], "");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&test_path)
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn check_with_sandbox_rejects_deno_command_spawn_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(stderr.contains("Process.Spawn"), "stderr: {stderr}");
}

#[test]
fn json_check_with_sandbox_rejects_deno_command_spawn_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
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
            .contains("Process.Spawn")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn check_with_sandbox_rejects_deno_command_spawn_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(stderr.contains("Process.Spawn"), "stderr: {stderr}");
}

#[test]
fn json_check_with_sandbox_rejects_deno_command_spawn_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
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
            .contains("Process.Spawn")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn check_with_sandbox_rejects_phase_three_deno_host_effects() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
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
fn check_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
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
fn json_check_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
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
fn check_with_sandbox_rejects_positive_thread_budget_policy() {
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
        .arg("check")
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
fn json_check_with_sandbox_rejects_positive_thread_budget_policy() {
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
        .arg("check")
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
fn check_with_sandbox_accepts_zero_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn json_check_with_sandbox_accepts_zero_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_check_accepts_positive_thread_budget_policy_when_threaded_profile_is_active() {
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
            .arg("check")
            .arg("--wasm-threads")
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
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["filesChecked"], 1);
        assert_eq!(json["payload"]["errorCount"], 0);
        assert_eq!(json["payload"]["warningCount"], 0);
    }
}
