use super::*;

#[test]
fn json_build_accepts_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(globalThis.Deno.pid);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn json_build_accepts_bracketed_global_this_deno_pid_in_ts_input() {
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
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn json_build_accepts_bracketed_global_this_deno_pid_in_js_input() {
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
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn json_build_accepts_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn json_build_accepts_deno_env_set_and_delete_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_SET_DELETE_SMOKE'); Deno[\"env\"][\"set\"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); Deno[\"env\"][\"delete\"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis.Deno[\"env\"][\"set\"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis.Deno[\"env\"][\"delete\"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis[\"Deno\"][\"env\"][\"set\"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis[\"Deno\"][\"env\"][\"delete\"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis.Deno[\"env\"].set('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis.Deno[\"env\"].delete('KALI_ENV_SET_DELETE_SMOKE'); globalThis[\"Deno\"].env[\"set\"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis[\"Deno\"].env[\"delete\"]('KALI_ENV_SET_DELETE_SMOKE');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    assert!(
        PathBuf::from(payload["outputPath"].as_str().expect("output path")).exists(),
        "expected build artifact"
    );
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_build_accepts_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno.pid);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn json_build_accepts_bracketed_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Deno[\"pid\"]);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn build_supports_late_env_materialization_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "build should succeed: {:?}",
        output
    );
}

#[test]
fn build_supports_late_env_materialization_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "build json should succeed: {:?}",
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
fn build_rejects_late_process_control_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_late_process_control_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
        .all(|error| matches!(error["code"].as_str(), Some("E5506") | Some("E3100"))));
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
fn build_rejects_late_object_model_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_late_object_model_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 10, "unexpected errors: {errors:?}");
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
fn build_rejects_late_object_model_members_in_browser_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_rejects_late_object_model_members_in_browser_bundle_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn build_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const body = \"return \" + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    assert!(
        dir.path().join("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled_in_json() {
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
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
    assert!(payload["sizeBytes"].as_u64().expect("size bytes") > 0);
    assert!(payload["sourceHash"].as_str().is_some());
}

#[test]
fn build_emits_browser_bundle_object_property_deletion_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_property_deletion_semantics_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Re-pinned (throw-fallout Stage 2 Lane C): `delete obj.a` here is used
    // in expression position (`!== true`), not a straight-line top-level
    // statement, so it is outside the optimizer's static shape timeline
    // and now reaches codegen's default-deny fallback — the bundle BUILD
    // must fail closed with E5506, not silently succeed with a stale
    // no-op. Previously (pre-Lane-C) `delete` here was just an E8001
    // warning + no-op, so the build succeeded; that was the very hole
    // this lane closes. `in`/`instanceof` elsewhere in this fixture are
    // still evaluation-time traps, unaffected by this change.
    assert!(
        !output.status.success(),
        "bundle build must fail closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("delete"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_object_property_deletion_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_property_deletion_semantics_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Re-pinned (throw-fallout Stage 2 Lane C): see the `_in_ts_input`
    // sibling above — `delete obj.a` here is expression-position, outside
    // the static shape timeline, and now hits codegen's default-deny
    // fallback, so the bundle BUILD must fail closed with E5506.
    assert!(
        !output.status.success(),
        "bundle build must fail closed: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("delete"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_object_type_and_constructor_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_type_and_constructor_semantics_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
}

#[test]
fn build_emits_browser_bundle_object_type_and_constructor_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_type_and_constructor_semantics_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
}

#[test]
fn build_emits_browser_bundle_object_type_and_constructor_semantics_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_type_and_constructor_semantics_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], true);
}

#[test]
fn build_supports_math_pow_builtin_semantics_in_browser_bundle_context_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(Math.pow(2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
    assert!(
        stdout.contains("Built browser bundle (esm) at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main").exists(),
        "expected browser bundle artifact"
    );
}

#[test]
fn build_supports_math_pow_builtin_semantics_in_browser_bundle_context_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Math.pow(2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
    assert!(
        stdout.contains("Built browser bundle (esm) at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main").exists(),
        "expected browser bundle artifact"
    );
}

#[test]
fn build_embeds_sandbox_policy_custom_section() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "1 + 2;").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--validate-ir")
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

    assert_embeds_policy_custom_section(&dir.path().join("main.wasm"), &policy_path);
}

#[test]
fn build_with_sandbox_rejects_invalid_policy_schema() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "1 + 2;").expect("write source");
    write_invalid_policy_schema(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when policy validation fails"
    );
}

#[test]
fn build_embeds_sandbox_policy_custom_section_for_library_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "function add(a, b) { return a + b; }").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
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

    let wasm_path = dir.path().join("math.lib.wasm");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 16, None);
    assert_embeds_policy_custom_section(&wasm_path, &policy_path);
}

#[test]
fn build_embeds_sandbox_policy_custom_section_for_library_artifact_is_deterministic_across_repeated_invocations(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
    write_valid_policy(&policy_path);

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");

    let build = || {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        read_artifact_bytes(&[wasm_path.clone(), wit_path.clone(), meta_path.clone()])
    };

    let first = build();
    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 16, None);
    assert_embeds_policy_custom_section(&wasm_path, &policy_path);

    let second = build();
    assert_eq!(
        first, second,
        "library artifacts should be stable across repeated sandboxed builds"
    );
}

#[test]
fn build_emits_library_artifacts_and_metadata() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--max-specializations")
        .arg("4")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
    assert!(wit.contains("export add: func();"));

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 4, None);
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let built = fs::read(&wasm_path).expect("read wasm artifact");
    let mut seen_metadata = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:metadata" {
                seen_metadata = true;
                break;
            }
        }
    }
    assert!(
        seen_metadata,
        "custom section 'kali:metadata' was not embedded"
    );
}

#[test]
fn build_rejects_function_declaration_export_aliases_for_library_artifact_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_supports_function_declaration_export_aliases_for_library_artifact(
            extension, false,
        );
    }
}

#[test]
fn json_build_rejects_function_declaration_export_aliases_for_library_artifact_in_all_input_classes(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_supports_function_declaration_export_aliases_for_library_artifact(
            extension, true,
        );
    }
}

#[test]
fn build_emits_library_artifacts_with_validate_ir() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--validate-ir")
        .arg("--max-specializations")
        .arg("4")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 4, None);
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let built = fs::read(&wasm_path).expect("read wasm artifact");
    let mut seen_metadata = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:metadata" {
                seen_metadata = true;
                break;
            }
        }
    }
    assert!(
        seen_metadata,
        "custom section 'kali:metadata' was not embedded"
    );
}

#[test]
fn build_emits_library_artifacts_for_js_sources() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--max-specializations")
        .arg("4")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
    assert!(wit.contains("export add: func();"));

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 4, None);
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let built = fs::read(&wasm_path).expect("read wasm artifact");
    let mut seen_metadata = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:metadata" {
                seen_metadata = true;
                break;
            }
        }
    }
    assert!(
        seen_metadata,
        "custom section 'kali:metadata' was not embedded"
    );
}

#[test]
fn json_build_emits_library_artifacts_for_js_sources() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--lib")
        .arg("--max-specializations")
        .arg("4")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    assert_eq!(payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("output path")),
        source_path.with_file_name("math.lib.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("math.lib.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("math.lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"wit"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
    let exports = payload["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 4, None);
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));
}

#[test]
fn build_rejects_library_sources_without_static_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "const value = 42; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5511"), "stderr: {stderr}");
    assert!(
        stderr.contains("no statically known export surface"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("math.lib.wasm").exists());
    assert!(!dir.path().join("math.lib.meta.json").exists());
}

#[test]
fn build_rejects_library_sources_without_static_exports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "const value = 42; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5511"), "stderr: {stderr}");
    assert!(
        stderr.contains("no statically known export surface"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("math.lib.wasm").exists());
    assert!(!dir.path().join("math.lib.meta.json").exists());
}

#[test]
fn build_rejects_library_sources_without_static_exports_in_capi_and_component_inputs() {
    for selector in ["--capi", "--component"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("math.js");
        fs::write(&source_path, "const value = 42; value;").expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg(selector)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success(), "selector: {selector}");
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E5511"),
            "selector: {selector}\nstderr: {stderr}"
        );
        assert!(
            stderr.contains("no statically known export surface"),
            "selector: {selector}\nstderr: {stderr}"
        );
        assert!(!dir.path().join("math.capi.wasm").exists());
        assert!(!dir.path().join("math.capi.meta.json").exists());
        assert!(!dir.path().join("math.component.wasm").exists());
        assert!(!dir.path().join("math.component.meta.json").exists());
    }
}

#[test]
fn build_emits_conservative_unknown_signature_for_mixed_exported_function_binding_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(
        &source_path,
        "export function main(input) { return true ? 1 : input; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
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

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    let exports = payload["exports"].as_array().expect("exports array");
    assert!(
        exports.iter().any(|export| {
            export["name"] == "main" && export["signature"] == "(input) => unknown"
        }),
        "exports: {exports:?}"
    );

    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["artifactKind"], "lib");
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(
        exports.iter().any(|export| {
            export["name"] == "main" && export["signature"] == "(input) => unknown"
        }),
        "exports: {exports:?}"
    );
}

#[test]
fn build_emits_conservative_unknown_signature_for_default_export_function_declaration_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(
        &source_path,
        "export default function main(input) { return true ? 1 : input; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
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

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    let exports = payload["exports"].as_array().expect("exports array");
    assert!(
        exports.iter().any(|export| {
            export["name"] == "main" && export["signature"] == "(input) => unknown"
        }),
        "exports: {exports:?}"
    );

    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["artifactKind"], "lib");
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(
        exports.iter().any(|export| {
            export["name"] == "main" && export["signature"] == "(input) => unknown"
        }),
        "exports: {exports:?}"
    );
}

#[test]
fn build_emits_bounded_signature_for_coalesce_return_literal_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export function main() { return null ?? 'fallback'; }",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "() => string"
            }),
            "exports: {exports:?}"
        );

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "() => string"
            }),
            "exports: {exports:?}"
        );
    }
}

#[test]
fn build_emits_bounded_signature_for_default_async_function_declaration_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default async function main(input) { return 1; }",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => Promise<number>"
            }),
            "exports for {extension}: {exports:?}"
        );

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => Promise<number>"
            }),
            "exports for {extension}: {exports:?}"
        );
    }
}

#[test]
fn build_emits_bounded_signature_for_default_async_function_declaration_through_await_wrapper_in_all_input_classes(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default async function main(input) { return await 1; }",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => Promise<number>"
            }),
            "exports for {extension}: {exports:?}"
        );

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => Promise<number>"
            }),
            "exports for {extension}: {exports:?}"
        );
    }
}

#[test]
fn build_supports_default_async_arrow_export_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(&source_path, "export default async (input) => 1;").expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "default" && export["signature"] == "(input) => Promise<number>"
            }),
            "exports for {extension}: {exports:?}"
        );
    }
}

#[test]
fn build_emits_conservative_unknown_signature_for_mixed_exported_function_binding_in_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export function main(input) { return true ? 1 : input; }",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => unknown"
            }),
            "exports for {extension}: {exports:?}"
        );

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => unknown"
            }),
            "exports for {extension}: {exports:?}"
        );
    }
}

#[test]
fn build_emits_conservative_unknown_signature_for_default_export_function_declaration_in_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default function main(input) { return true ? 1 : input; }",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
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

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => unknown"
            }),
            "exports for {extension}: {exports:?}"
        );

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => unknown"
            }),
            "exports for {extension}: {exports:?}"
        );
    }
}

#[test]
fn build_supports_default_async_function_expression_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default (async function main(input) { return true ? 1 : input; });",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "default" && export["signature"] == "(input) => Promise<unknown>"
            }),
            "exports for {extension}: {exports:?}"
        );
        assert!(dir.path().join("math.lib.wasm").exists());
        assert!(dir.path().join("math.lib.wit").exists());
        assert!(dir.path().join("math.lib.meta.json").exists());
    }
}

#[test]
fn build_supports_default_function_expression_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default (function main(input) { return true ? 1 : input; });",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
        )
        .expect("parse metadata json");
        assert_eq!(metadata["artifactKind"], "lib");
        let exports = metadata["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "default" && export["signature"] == "(input) => unknown"
            }),
            "exports for {extension}: {exports:?}"
        );
        assert!(dir.path().join("math.lib.wasm").exists());
        assert!(dir.path().join("math.lib.wit").exists());
        assert!(dir.path().join("math.lib.meta.json").exists());
    }
}

#[test]
fn json_build_supports_default_function_expression_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default (function main(input) { return true ? 1 : input; });",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .arg("--lib")
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
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "default" && export["signature"] == "(input) => unknown"
            }),
            "expected default export in {exports:?}"
        );
        assert!(dir.path().join("math.lib.wasm").exists());
        assert!(dir.path().join("math.lib.wit").exists());
        assert!(dir.path().join("math.lib.meta.json").exists());
    }
}

#[test]
fn json_build_supports_default_async_function_expression_in_all_input_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("math.{extension}"));
        fs::write(
            &source_path,
            "export default (async function main(input) { return true ? 1 : input; });",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .arg("--lib")
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
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        let payload = json["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "default" && export["signature"] == "(input) => Promise<unknown>"
            }),
            "expected default export in {exports:?}"
        );
        assert!(dir.path().join("math.lib.wasm").exists());
        assert!(dir.path().join("math.lib.wit").exists());
        assert!(dir.path().join("math.lib.meta.json").exists());
    }
}

#[test]
fn json_build_rejects_library_sources_without_static_exports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "const value = 42; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--lib")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().any(|error| error["code"] == "E5511"),
        "expected E5511 in {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("no statically known export surface")),
        "expected export-surface message in {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error["context"]["origin"] == "source"),
        "expected source-origin context in {errors:?}"
    );
    assert!(!dir.path().join("math.lib.wasm").exists());
    assert!(!dir.path().join("math.lib.meta.json").exists());
}

#[test]
fn json_build_rejects_library_sources_without_static_exports_in_capi_and_component_inputs() {
    for selector in ["--capi", "--component"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("math.js");
        fs::write(&source_path, "const value = 42; value;").expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .arg(selector)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success(), "selector: {selector}");
        assert_eq!(output.status.code(), Some(1));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            errors.iter().any(|error| error["code"] == "E5511"),
            "selector: {selector}\nerrors: {errors:?}"
        );
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains("no statically known export surface")),
            "selector: {selector}\nerrors: {errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|error| error["context"]["origin"] == "source"),
            "selector: {selector}\nerrors: {errors:?}"
        );
        assert!(!dir.path().join("math.capi.wasm").exists());
        assert!(!dir.path().join("math.capi.meta.json").exists());
        assert!(!dir.path().join("math.component.wasm").exists());
        assert!(!dir.path().join("math.component.meta.json").exists());
    }
}

#[test]
fn build_accepts_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    assert!(
        dir.path().join("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_accepts_inherited_wasm_threads_runtime_profile() {
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
        .arg("build")
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
        dir.path().join("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_accepts_wasm_threads_runtime_profile_in_js_input_for_library_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
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

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "lib");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert!(metadata.get("profileDataHash").is_none());
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
    assert!(wit.contains("export add: func();"));
}

#[test]
fn build_accepts_wasm_threads_runtime_profile_in_js_input_for_capi_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
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

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["kind"], "cabi-metadata");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
}

#[test]
fn build_accepts_wasm_threads_runtime_profile_in_js_input_for_component_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
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

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["artifactKind"], "component");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
}

#[test]
fn build_accepts_inherited_wasm_threads_runtime_profile_in_js_input_for_library_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--lib")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "lib");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert!(metadata.get("profileDataHash").is_none());
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
    assert!(wit.contains("export add: func();"));
}

#[test]
fn build_accepts_inherited_wasm_threads_runtime_profile_in_js_input_for_capi_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--capi")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["kind"], "cabi-metadata");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
}

#[test]
fn build_accepts_inherited_wasm_threads_runtime_profile_in_js_input_for_component_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--component")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["artifactKind"], "component");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
}

#[test]
fn build_emits_component_json_artifacts_with_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--component")
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

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["outputPath"]
                .as_str()
                .expect("component output path")
        ),
        source_path.with_file_name("lib.component.wasm")
    );
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        source_path.with_file_name("lib.binding-package.json")
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let component_bytes = fs::read(&component_path).expect("read component bytes");
    wasmparser::Validator::new()
        .validate_all(&component_bytes)
        .expect("generated component should validate");
}

#[test]
fn build_emits_component_json_artifacts_with_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--wasm-threads")
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

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["outputPath"]
                .as_str()
                .expect("component output path")
        ),
        source_path.with_file_name("lib.component.wasm")
    );
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        source_path.with_file_name("lib.binding-package.json")
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let component_bytes = fs::read(&component_path).expect("read component bytes");
    wasmparser::Validator::new()
        .validate_all(&component_bytes)
        .expect("generated component should validate");
}

#[test]
fn build_emits_component_artifacts_with_wasm_threads_runtime_profile_in_js_input() {
    for inherited_runtime_profile in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("lib.js");
        fs::write(&source_path, "export function add(a, b) { return a + b; }")
            .expect("write source");
        if inherited_runtime_profile {
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

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--component");
        if !inherited_runtime_profile {
            command.arg("--wasm-threads");
        }
        let output = command.arg(&source_path).output().expect("run kali");

        assert!(
            output.status.success(),
            "inherited_runtime_profile={inherited_runtime_profile}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Built component artifact at "),
            "stdout: {stdout}"
        );

        let component_path = source_path.with_file_name("lib.component.wasm");
        let wit_path = source_path.with_file_name("lib.wit");
        let meta_path = source_path.with_file_name("lib.component.meta.json");
        let binding_package_path = source_path.with_file_name("lib.binding-package.json");
        assert!(
            component_path.exists(),
            "missing {}",
            component_path.display()
        );
        assert!(wit_path.exists(), "missing {}", wit_path.display());
        assert!(meta_path.exists(), "missing {}", meta_path.display());
        assert!(
            binding_package_path.exists(),
            "missing {}",
            binding_package_path.display()
        );

        let metadata: Value =
            serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
                .expect("parse metadata json");
        assert_eq!(
            metadata["runtimeProfiles"],
            serde_json::json!(["wasm-threads"])
        );
        assert_eq!(metadata["artifactKind"], "component");
        assert_eq!(metadata["hostContract"], "kali-hosted");
        assert_eq!(metadata["runtimeBackend"], "wasmtime");

        let binding_package: Value = serde_json::from_str(
            &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
        )
        .expect("parse binding package manifest json");
        assert_eq!(
            binding_package["runtimeProfiles"],
            serde_json::json!(["wasm-threads"])
        );
        assert_eq!(binding_package["kind"], "binding-package");
        assert_eq!(binding_package["hostContract"], "kali-hosted");
        assert_eq!(binding_package["runtimeBackend"], "wasmtime");

        let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
        assert!(wit.contains("package kali:embed;"));

        let component_bytes = fs::read(&component_path).expect("read component bytes");
        wasmparser::Validator::new()
            .validate_all(&component_bytes)
            .expect("generated component should validate");
    }
}

#[test]
fn build_emits_library_json_artifacts_with_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--wasm-threads")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    assert_eq!(payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("output path")),
        source_path.with_file_name("math.lib.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("math.lib.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("math.lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"wit"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["artifactKind"], "lib");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
}

#[test]
fn build_emits_library_json_artifacts_with_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--lib")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "lib");
    assert_eq!(payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("output path")),
        source_path.with_file_name("math.lib.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("math.lib.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("math.lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"wit"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["artifactKind"], "lib");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
}

#[test]
fn build_emits_capi_json_artifacts_with_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg("--wasm-threads")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "capi");
    assert_eq!(payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("capi output path")),
        source_path.with_file_name("lib.capi.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["headerPath"].as_str().expect("c header path")),
        source_path.with_file_name("lib.h")
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("lib.capi.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["kind"] == "cabi-metadata"));

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("lib.capi.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("lib.binding-package.json"))
            .expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
}

#[test]
fn build_emits_capi_json_artifacts_with_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
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
        .arg("build")
        .arg("--capi")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "capi");
    assert_eq!(payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("capi output path")),
        source_path.with_file_name("lib.capi.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["headerPath"].as_str().expect("c header path")),
        source_path.with_file_name("lib.h")
    );
    assert_eq!(
        PathBuf::from(payload["metadataPath"].as_str().expect("metadata path")),
        source_path.with_file_name("lib.capi.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["kind"] == "cabi-metadata"));

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("lib.capi.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("lib.binding-package.json"))
            .expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(
        binding_package["runtimeProfiles"],
        serde_json::json!(["wasm-threads"])
    );
}

#[test]
fn build_rejects_inherited_duplicate_runtime_profiles() {
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
        .arg("build")
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
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_rejects_inherited_empty_or_whitespace_runtime_profiles() {
    for runtime_profiles in [r#"[""]"#, r#"["   "]"#] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
        fs::write(
            dir.path().join("kali.json"),
            format!(
                r#"{{
  "schemaVersion": 1,
  "compilerOptions": {{
    "runtimeProfiles": {runtime_profiles}
  }}
}}"#
            ),
        )
        .expect("write manifest");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5509"), "stderr: {stderr}");
        assert!(
            stderr.contains("runtimeProfile") && stderr.contains("empty or whitespace-only"),
            "stderr: {stderr}"
        );
        assert!(!dir.path().join("main.wasm").exists());
    }
}

#[test]
fn build_rejects_inherited_unknown_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["fiber-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("unsupported runtimeProfile"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_emits_browser_bundle_artifacts() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.js");
    let source_map_path = bundle_dir.join("app.js.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("instantiateStreaming"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        js.contains("sourceMappingURL=app.js.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.js");
    assert_eq!(source_map["sources"][0], "app.ts");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_emits_browser_bundle_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.js");
    let source_map_path = bundle_dir.join("app.js.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("instantiateStreaming"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        js.contains("sourceMappingURL=app.js.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.js");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_emits_inherited_browser_bundle_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("build")
        .arg("--bundle")
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
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.js");
    let source_map_path = bundle_dir.join("app.js.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("instantiateStreaming"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        js.contains("sourceMappingURL=app.js.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.js");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_embeds_sandbox_policy_custom_section_for_browser_bundle_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
    assert_embeds_policy_custom_section(&wasm_path, &policy_path);
}

#[test]
fn build_embeds_sandbox_policy_custom_section_for_browser_bundle_artifact_with_validate_ir() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg("--validate-ir")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
    assert_embeds_policy_custom_section(&wasm_path, &policy_path);
    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_trees_shakes_unused_browser_bundle_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; } function unused() { return 1; }",
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
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        !js.contains("export async function unused"),
        "bundle js: {js}"
    );

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    let exports = metadata["exports"].as_array().expect("exports array");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0]["name"], "greet");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_emits_browser_bundle_crypto_web_apis() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: digestSmoke\nasync function digestSmoke(left, right) {\n  const bytes = new TextEncoder().encode(`browser crypto ${String(left + right)}`);\n  if (bytes.byteLength !== 16) {\n    throw new Error(`unexpected encoded length ${bytes.byteLength}`);\n  }\n  const randomBytes = new globalThis[\"Uint8Array\"](8);\n  const filledBytes = crypto.getRandomValues(randomBytes);\n  if (filledBytes !== randomBytes) {\n    throw new Error('crypto.getRandomValues should return the provided buffer');\n  }\n  if (filledBytes.length !== 8 || filledBytes.byteLength !== 8) {\n    throw new Error(`unexpected random buffer length ${filledBytes.length}/${filledBytes.byteLength}`);\n  }\n  const digest = await crypto.subtle.digest('SHA-512', bytes);\n  const uuid = crypto.randomUUID();\n  if (digest.byteLength !== 64) {\n    throw new Error(`unexpected digest length ${digest.byteLength}`);\n  }\n  if (typeof uuid !== 'string' || uuid.length === 0) {\n    throw new Error(`unexpected uuid ${uuid}`);\n  }\n  return left - left;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "digestSmoke");
}

#[test]
fn build_emits_browser_bundle_crypto_web_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: digestSmoke\nasync function digestSmoke(left, right) {\n  const bytes = new TextEncoder().encode(`browser crypto ${String(left + right)}`);\n  if (bytes.byteLength !== 16) {\n    throw new Error(`unexpected encoded length ${bytes.byteLength}`);\n  }\n  const randomBytes = new globalThis[\"Uint8Array\"](8);\n  const filledBytes = crypto.getRandomValues(randomBytes);\n  if (filledBytes !== randomBytes) {\n    throw new Error('crypto.getRandomValues should return the provided buffer');\n  }\n  if (filledBytes.length !== 8 || filledBytes.byteLength !== 8) {\n    throw new Error(`unexpected random buffer length ${filledBytes.length}/${filledBytes.byteLength}`);\n  }\n  const digest = await crypto.subtle.digest('SHA-512', bytes);\n  const uuid = crypto.randomUUID();\n  if (digest.byteLength !== 64) {\n    throw new Error(`unexpected digest length ${digest.byteLength}`);\n  }\n  if (typeof uuid !== 'string' || uuid.length === 0) {\n    throw new Error(`unexpected uuid ${uuid}`);\n  }\n  return left - left;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "digestSmoke");
}

#[test]
fn json_build_emits_browser_bundle_crypto_web_apis_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: digestSmoke\nasync function digestSmoke(left, right) {\n  const bytes = new TextEncoder().encode(`browser crypto ${String(left + right)}`);\n  if (bytes.byteLength !== 16) {\n    throw new Error(`unexpected encoded length ${bytes.byteLength}`);\n  }\n  const randomBytes = new globalThis[\"Uint8Array\"](8);\n  const filledBytes = crypto.getRandomValues(randomBytes);\n  if (filledBytes !== randomBytes) {\n    throw new Error('crypto.getRandomValues should return the provided buffer');\n  }\n  if (filledBytes.length !== 8 || filledBytes.byteLength !== 8) {\n    throw new Error(`unexpected random buffer length ${filledBytes.length}/${filledBytes.byteLength}`);\n  }\n  const digest = await crypto.subtle.digest('SHA-512', bytes);\n  const uuid = crypto.randomUUID();\n  if (digest.byteLength !== 64) {\n    throw new Error(`unexpected digest length ${digest.byteLength}`);\n  }\n  if (typeof uuid !== 'string' || uuid.length === 0) {\n    throw new Error(`unexpected uuid ${uuid}`);\n  }\n  return left - left;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "digestSmoke");
}

#[test]
fn json_build_emits_browser_bundle_crypto_web_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: digestSmoke\nasync function digestSmoke(left, right) {\n  const bytes = new TextEncoder().encode(`browser crypto ${String(left + right)}`);\n  if (bytes.byteLength !== 16) {\n    throw new Error(`unexpected encoded length ${bytes.byteLength}`);\n  }\n  const randomBytes = new globalThis[\"Uint8Array\"](8);\n  const filledBytes = crypto.getRandomValues(randomBytes);\n  if (filledBytes !== randomBytes) {\n    throw new Error('crypto.getRandomValues should return the provided buffer');\n  }\n  if (filledBytes.length !== 8 || filledBytes.byteLength !== 8) {\n    throw new Error(`unexpected random buffer length ${filledBytes.length}/${filledBytes.byteLength}`);\n  }\n  const digest = await crypto.subtle.digest('SHA-512', bytes);\n  const uuid = crypto.randomUUID();\n  if (digest.byteLength !== 64) {\n    throw new Error(`unexpected digest length ${digest.byteLength}`);\n  }\n  if (typeof uuid !== 'string' || uuid.length === 0) {\n    throw new Error(`unexpected uuid ${uuid}`);\n  }\n  return left - left;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "digestSmoke");
}

#[test]
fn build_emits_browser_bundle_web_baseline_primitives() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, browser_bundle_web_baseline_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
}

#[test]
fn build_emits_browser_bundle_web_baseline_primitives_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, browser_bundle_web_baseline_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
}

#[test]
fn json_build_emits_browser_bundle_web_baseline_primitives() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, browser_bundle_web_baseline_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], true);
}

#[test]
fn json_build_emits_browser_bundle_web_baseline_primitives_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, browser_bundle_web_baseline_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Flipped pin (evaluation-trap layering): in/instanceof are runtime
    // traps, not compile rejects, so the bundle BUILD must succeed —
    // analysis and builds of code containing them stay usable (the browser
    // package corpus pins this). Executing the smoke entrypoint traps
    // fail-closed; that behavior is pinned by soundness_in_operator.rs and
    // the run/test variants of this family.
    assert!(
        output.status.success(),
        "bundle build must succeed: {output:?}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["success"], true);
}

/// Stage D event lane, browser glue end-to-end: the bundle's JS import list
/// registers and synchronously dispatches through kaliEventListeners.
/// node v26.5.0 (same source, plain node): "before=0\nafter=1\n".
#[test]
fn browser_bundle_event_lane_executes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: eventLaneSmoke\nfunction eventLaneSmoke(left, right) {\n  const t = new EventTarget();\n  let n = 0;\n  t.addEventListener(\"tick\", function () { n += 1; });\n  console.log(\"before=\" + n);\n  t.dispatchEvent(new CustomEvent(\"tick\"));\n  console.log(\"after=\" + n);\n  return left - left;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    // Mirrors `assert_browser_bundle_executes_with_result`'s helper calls
    // (harness script + command construction), but asserts the FULL
    // captured stdout byte-for-byte instead of a `contains` check on a
    // return value, since this fixture's load-bearing assertion is the
    // ordering of the two console.log calls around the synchronous dispatch.
    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        "const mod = await import(bundleJs.href);\nawait mod.eventLaneSmoke(1n, 2n);\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "before=0\nafter=1\n");
}

#[test]
fn build_emits_browser_bundle_async_await_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: awaitSmoke
async function awaitSmoke(left, right) {
  const order = [];
  order.push('before');
  const value = await Promise.resolve(left + right);
  order.push('after');
  if (value !== 3n || order.join(',') !== 'before,after') {
    throw new Error('unexpected await sequencing');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "awaitSmoke");
}

#[test]
fn build_emits_browser_bundle_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: awaitSmoke
async function awaitSmoke(left, right) {
  const order = [];
  order.push('before');
  const value = await Promise.resolve(left + right);
  order.push('after');
  if (value !== 3n || order.join(',') !== 'before,after') {
    throw new Error('unexpected await sequencing');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "awaitSmoke");
}

#[test]
fn json_build_emits_browser_bundle_async_await_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: awaitSmoke
async function awaitSmoke(left, right) {
  const order = [];
  order.push('before');
  const value = await Promise.resolve(left + right);
  order.push('after');
  if (value !== 3n || order.join(',') !== 'before,after') {
    throw new Error('unexpected await sequencing');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "awaitSmoke");
}

#[test]
fn json_build_emits_browser_bundle_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: awaitSmoke
async function awaitSmoke(left, right) {
  const order = [];
  order.push('before');
  const value = await Promise.resolve(left + right);
  order.push('after');
  if (value !== 3n || order.join(',') !== 'before,after') {
    throw new Error('unexpected await sequencing');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "awaitSmoke");
}

#[test]
fn build_emits_browser_bundle_promise_all_sequencing() {
    assert_browser_bundle_promise_all_sequencing("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_promise_all_sequencing_in_js_input() {
    assert_browser_bundle_promise_all_sequencing("app.js", false);
}

#[test]
fn json_build_emits_browser_bundle_promise_all_sequencing() {
    assert_browser_bundle_promise_all_sequencing("app.ts", true);
}

#[test]
fn json_build_emits_browser_bundle_promise_all_sequencing_in_js_input() {
    assert_browser_bundle_promise_all_sequencing("app.js", true);
}

#[test]
fn build_emits_browser_bundle_queue_microtask_ordering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: queueMicrotaskSmoke
async function queueMicrotaskSmoke(left, right) {
  const order = [];
  queueMicrotask(() => {
    order.push('microtask');
  });
  order.push('before');
  if (order.join(',') !== 'before') {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve(left + right);
  if (order.join(',') !== 'before,microtask') {
    throw new Error(`unexpected queueMicrotask ordering ${order.join(',')}`);
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "queueMicrotaskSmoke");
}

#[test]
fn build_emits_browser_bundle_queue_microtask_ordering_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: queueMicrotaskSmoke\nasync function queueMicrotaskSmoke(left, right) {\n  let order = [];\n  queueMicrotask(() => {\n    order.push('microtask');\n  });\n  order.push('before');\n  if (order.join(',') !== 'before') {\n    throw new Error('microtask ran too early');\n  }\n  await Promise.resolve(left + right);\n  if (order.join(',') !== 'before,microtask') {\n    throw new Error(`unexpected queueMicrotask ordering ${order.join(',')}`);\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "queueMicrotaskSmoke");
}

#[test]
fn build_emits_browser_bundle_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: queueMicrotaskSmoke
async function queueMicrotaskSmoke(left, right) {
  const order = [];
  queueMicrotask(() => {
    order.push('microtask');
  });
  order.push('before');
  if (order.join(',') !== 'before') {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve(left + right);
  if (order.join(',') !== 'before,microtask') {
    throw new Error(`unexpected queueMicrotask ordering ${order.join(',')}`);
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "queueMicrotaskSmoke");
}

#[test]
fn json_build_emits_browser_bundle_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: queueMicrotaskSmoke
async function queueMicrotaskSmoke(left, right) {
  const order = [];
  queueMicrotask(() => {
    order.push('microtask');
  });
  order.push('before');
  if (order.join(',') !== 'before') {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve(left + right);
  if (order.join(',') !== 'before,microtask') {
    throw new Error(`unexpected queueMicrotask ordering ${order.join(',')}`);
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "queueMicrotaskSmoke");
}

#[test]
fn build_emits_browser_bundle_performance_now_monotonic_ordering_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn build_emits_browser_bundle_performance_now_monotonic_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke
async function performanceNowSmoke(left, right) {
  const first = performance.now();
  await Promise.resolve(left + right);
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn build_uses_inherited_browser_api_surface_for_performance_now_monotonic_ordering_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
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
        .arg("build")
        .arg("--bundle")
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
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn json_build_emits_browser_bundle_performance_now_monotonic_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    assert_browser_bundle_executes(&dir.path().join("app"), "performanceNowSmoke");
}

#[test]
fn build_uses_inherited_browser_api_surface_for_performance_now_monotonic_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
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
        .arg("build")
        .arg("--bundle")
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
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn json_build_uses_inherited_browser_api_surface_for_performance_now_monotonic_ordering_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn json_build_uses_inherited_browser_api_surface_for_performance_now_monotonic_ordering_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: performanceNowSmoke\nasync function performanceNowSmoke(left, right) {\n  const first = performance.now();\n  await Promise.resolve(left + right);\n  const second = performance.now();\n  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {\n    throw new Error('performance.now moved backwards');\n  }\n  return 0n;\n}\n",
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "performanceNowSmoke");
}

#[test]
fn build_emits_browser_bundle_boolean_logic_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: logicSmoke\nfunction logicSmoke() {\n  let observed = 0;\n  const andResult = true && (++observed, true);\n  const orResult = false || (++observed, true);\n  const skippedAnd = false && (++observed, true);\n  const skippedOr = true || (++observed, true);\n  if (observed !== 2 || andResult !== true || orResult !== true || skippedAnd !== false || skippedOr !== true) {\n    throw new Error('unexpected boolean logic');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "logicSmoke");
}

#[test]
fn build_emits_browser_bundle_boolean_logic_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: logicSmoke\nfunction logicSmoke() {\n  let observed = 0;\n  const andResult = true && (++observed, true);\n  const orResult = false || (++observed, true);\n  const skippedAnd = false && (++observed, true);\n  const skippedOr = true || (++observed, true);\n  if (observed !== 2 || andResult !== true || orResult !== true || skippedAnd !== false || skippedOr !== true) {\n    throw new Error('unexpected boolean logic');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "logicSmoke");
}

#[test]
fn json_build_emits_browser_bundle_boolean_logic_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: logicSmoke\nfunction logicSmoke() {\n  let observed = 0;\n  const andResult = true && (++observed, true);\n  const orResult = false || (++observed, true);\n  const skippedAnd = false && (++observed, true);\n  const skippedOr = true || (++observed, true);\n  if (observed !== 2 || andResult !== true || orResult !== true || skippedAnd !== false || skippedOr !== true) {\n    throw new Error('unexpected boolean logic');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "logicSmoke");
}

#[test]
fn json_build_emits_browser_bundle_boolean_logic_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: logicSmoke\nfunction logicSmoke() {\n  let observed = 0;\n  const andResult = true && (++observed, true);\n  const orResult = false || (++observed, true);\n  const skippedAnd = false && (++observed, true);\n  const skippedOr = true || (++observed, true);\n  if (observed !== 2 || andResult !== true || orResult !== true || skippedAnd !== false || skippedOr !== true) {\n    throw new Error('unexpected boolean logic');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "logicSmoke");
}

#[test]
fn build_emits_browser_bundle_unary_prefix_semantics() {
    assert_browser_bundle_unary_prefix_semantics("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_unary_prefix_semantics_in_js_input() {
    assert_browser_bundle_unary_prefix_semantics("app.js", false);
}

#[test]
fn json_build_emits_browser_bundle_unary_prefix_semantics() {
    assert_browser_bundle_unary_prefix_semantics("app.ts", true);
}

#[test]
fn json_build_emits_browser_bundle_unary_prefix_semantics_in_js_input() {
    assert_browser_bundle_unary_prefix_semantics("app.js", true);
}

#[test]
fn build_emits_browser_bundle_bigint_addition_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: bigintSmoke\nfunction bigintSmoke() {\n  const result = 1n + 2n;\n  if (result !== 3n) {\n    throw new Error('unexpected bigint');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "bigintSmoke");
}

#[test]
fn build_emits_browser_bundle_bigint_addition_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: bigintSmoke\nfunction bigintSmoke() {\n  const result = 1n + 2n;\n  if (result !== 3n) {\n    throw new Error('unexpected bigint');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "bigintSmoke");
}

#[test]
fn build_emits_browser_bundle_bigint_multiplication_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: bigintSmoke\nfunction bigintSmoke() {\n  const result = 1n * 2n;\n  if (result !== 2n) {\n    throw new Error('unexpected bigint');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "bigintSmoke");
}

#[test]
fn build_emits_browser_bundle_bigint_multiplication_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: bigintSmoke\nfunction bigintSmoke() {\n  const result = 1n * 2n;\n  if (result !== 2n) {\n    throw new Error('unexpected bigint');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "bigintSmoke");
}

#[test]
fn build_emits_browser_bundle_bigint_remainder_semantics() {
    assert_browser_bundle_supports_bigint_binary_semantics("ts", "3n % 2n", "1n");
}

#[test]
fn build_emits_browser_bundle_bigint_remainder_semantics_in_js_input() {
    assert_browser_bundle_supports_bigint_binary_semantics("js", "3n % 2n", "1n");
}

#[test]
fn build_emits_browser_bundle_bigint_exponentiation_semantics() {
    assert_browser_bundle_supports_bigint_binary_semantics("ts", "2n ** 3n", "8n");
}

#[test]
fn build_emits_browser_bundle_bigint_exponentiation_semantics_in_js_input() {
    assert_browser_bundle_supports_bigint_binary_semantics("js", "2n ** 3n", "8n");
}

#[test]
fn build_emits_browser_bundle_math_sign_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: signSmoke\nfunction signSmoke() {\n  const result = Math.sign(-3);\n  if (result !== -1) {\n    throw new Error('unexpected sign');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "signSmoke");
}

#[test]
fn build_emits_browser_bundle_math_sign_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: signSmoke\nfunction signSmoke() {\n  const result = Math.sign(-3);\n  if (result !== -1) {\n    throw new Error('unexpected sign');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "signSmoke");
}

#[test]
fn build_emits_browser_bundle_math_hypot_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: hypotSmoke\nfunction hypotSmoke() {\n  const result = Math.hypot(3, 4);\n  if (result !== 5) {\n    throw new Error('unexpected hypot');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "hypotSmoke");
}

#[test]
fn build_emits_browser_bundle_math_imul_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: imulSmoke\nfunction imulSmoke() {\n  const result = Math.imul(2147483647, 2);\n  if (result !== -2) {\n    throw new Error('unexpected imul');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "imulSmoke");
}

#[test]
fn build_emits_browser_bundle_math_imul_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: imulSmoke\nfunction imulSmoke() {\n  const result = Math.imul(2147483647, 2);\n  if (result !== -2) {\n    throw new Error('unexpected imul');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "imulSmoke");
}

#[test]
fn build_emits_browser_bundle_math_abs_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: absSmoke\nfunction absSmoke() {\n  const result = Math.abs(3 - 6);\n  if (result !== 3) {\n    throw new Error('unexpected abs');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "absSmoke");
}

#[test]
fn build_emits_browser_bundle_math_abs_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: absSmoke\nfunction absSmoke() {\n  const result = Math.abs(3 - 6);\n  if (result !== 3) {\n    throw new Error('unexpected abs');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "absSmoke");
}

#[test]
fn build_emits_browser_bundle_math_ceil_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: ceilSmoke\nfunction ceilSmoke() {\n  const result = Math.ceil(3 - 6);\n  if (result !== -3) {\n    throw new Error('unexpected ceil');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "ceilSmoke");
}

#[test]
fn build_emits_browser_bundle_math_ceil_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: ceilSmoke\nfunction ceilSmoke() {\n  const result = Math.ceil(3 - 6);\n  if (result !== -3) {\n    throw new Error('unexpected ceil');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "ceilSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_ceil_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: ceilSmoke\nfunction ceilSmoke() {\n  const result = Math.ceil(3 - 6);\n  if (result !== -3) {\n    throw new Error('unexpected ceil');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "ceilSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_ceil_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: ceilSmoke\nfunction ceilSmoke() {\n  const result = Math.ceil(3 - 6);\n  if (result !== -3) {\n    throw new Error('unexpected ceil');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "ceilSmoke");
}

#[test]
fn build_emits_browser_bundle_math_trunc_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: truncSmoke\nfunction truncSmoke() {\n  const result = Math.trunc(3 - 6);\n  if (result !== -3) {\n    throw new Error('unexpected trunc');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "truncSmoke");
}

#[test]
fn build_emits_browser_bundle_math_max_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const result = Math.max(1, 2, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn build_emits_browser_bundle_math_max_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const result = Math.max(1, 2, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn build_emits_browser_bundle_math_max_semantics_with_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const value = 2; const alias = value;\n  const result = Math.max(1, alias, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn build_emits_browser_bundle_math_min_semantics_with_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const value = 3; const alias = value;\n  const result = Math.min(alias, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn build_emits_browser_bundle_math_max_semantics_with_const_alias_chain_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const value = 2; const alias = value;\n  const result = Math.max(1, alias, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn build_emits_browser_bundle_math_min_semantics_with_const_alias_chain_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const value = 3; const alias = value;\n  const result = Math.min(alias, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_max_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const result = Math.max(1, 2, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_min_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const result = Math.min(3, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_abs_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: absSmoke\nfunction absSmoke() {\n  const result = Math.abs(3 - 6);\n  if (result !== 3) {\n    throw new Error('unexpected abs');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "absSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_sign_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: signSmoke\nfunction signSmoke() {\n  const result = Math.sign(3 - 6);\n  if (result !== -1) {\n    throw new Error('unexpected sign');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "signSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_max_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const result = Math.max(1, 2, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_min_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const result = Math.min(3, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_abs_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: absSmoke\nfunction absSmoke() {\n  const result = Math.abs(3 - 6);\n  if (result !== 3) {\n    throw new Error('unexpected abs');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "absSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_sign_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: signSmoke\nfunction signSmoke() {\n  const result = Math.sign(3 - 6);\n  if (result !== -1) {\n    throw new Error('unexpected sign');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "signSmoke");
}

#[test]
fn build_emits_browser_bundle_math_clz32_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: clz32Smoke\nfunction clz32Smoke() {\n  const result = Math.clz32(1);\n  if (result !== 31) {\n    throw new Error('unexpected clz32');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "clz32Smoke");
}

#[test]
fn build_emits_browser_bundle_math_clz32_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: clz32Smoke
function clz32Smoke() {
  const result = Math.clz32(1);
  if (result !== 31) {
    throw new Error('unexpected clz32');
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "clz32Smoke");
}

#[test]
fn json_build_emits_browser_bundle_math_trunc_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: truncSmoke
function truncSmoke() {
  const result = Math.trunc(3 - 6);
  if (result !== -3) {
    throw new Error('unexpected trunc');
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "truncSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_trunc_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: truncSmoke
function truncSmoke() {
  const result = Math.trunc(3 - 6);
  if (result !== -3) {
    throw new Error('unexpected trunc');
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "truncSmoke");
}

#[test]
fn json_build_emits_browser_bundle_math_clz32_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: clz32Smoke
function clz32Smoke() {
  const result = Math.clz32(1);
  if (result !== 31) {
    throw new Error('unexpected clz32');
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "clz32Smoke");
}

#[test]
fn json_build_emits_browser_bundle_math_clz32_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        r#"// kali-tree-shake: clz32Smoke
function clz32Smoke() {
  const result = Math.clz32(1);
  if (result !== 31) {
    throw new Error('unexpected clz32');
  }
  return 0n;
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "clz32Smoke");
}

#[test]
fn build_emits_browser_bundle_math_min_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const result = Math.min(3, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn build_emits_browser_bundle_math_min_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: minSmoke\nfunction minSmoke() {\n  const result = Math.min(3, 2, 1);\n  if (result !== 1) {\n    throw new Error('unexpected min');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "minSmoke");
}

#[test]
fn build_emits_browser_bundle_try_finally_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: finallySmoke
function finallySmoke() {
  let value = 0;
  try {
    value += 1;
  } finally {
    value += 2;
  }
  if (value !== 3) {
    throw new Error('unexpected finally');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_try_finally_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: finallySmoke
function finallySmoke() {
  let value = 0;
  try {
    value += 1;
  } finally {
    value += 2;
  }
  if (value !== 3) {
    throw new Error('unexpected finally');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_try_catch_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: catchSmoke
function catchSmoke() {
  let value = 0;
  try {
    value += 1;
    throw new Error('expected');
  } catch {
    value += 2;
  }
  if (value !== 3) {
    throw new Error('unexpected catch');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_try_catch_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: catchSmoke
function catchSmoke() {
  let value = 0;
  try {
    value += 1;
    throw new Error('expected');
  } catch {
    value += 2;
  }
  if (value !== 3) {
    throw new Error('unexpected catch');
  }
  return 0n;
}
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_try_finally_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: finallySmoke\nfunction finallySmoke() {\n  let value = 0;\n  try {\n    value += 1;\n  } finally {\n    value += 2;\n  }\n  if (value !== 3) {\n    throw new Error('unexpected finally');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_try_catch_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: catchSmoke\nfunction catchSmoke() {\n  let value = 0;\n  try {\n    value += 1;\n    throw new Error('expected');\n  } catch {\n    value += 2;\n  }\n  if (value !== 3) {\n    throw new Error('unexpected catch');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_string_enumeration_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_string_enumeration_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_string_enumeration_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_string_enumeration_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_string_primitive_enumeration_semantics() {
    assert_browser_bundle_string_primitive_enumeration("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_string_primitive_enumeration_semantics_in_js_input() {
    assert_browser_bundle_string_primitive_enumeration("app.js", false);
}

#[test]
fn json_build_emits_browser_bundle_string_primitive_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_string_primitive_enumeration("app.ts", true);
}

#[test]
fn json_build_emits_browser_bundle_string_primitive_enumeration_semantics_in_js_input() {
    assert_browser_bundle_string_primitive_enumeration("app.js", true);
}

#[test]
fn build_emits_browser_bundle_integer_like_key_ordering_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_integer_like_object_enumeration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_integer_like_key_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_integer_like_object_enumeration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_integer_like_key_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_integer_like_object_enumeration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_integer_like_key_ordering_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_integer_like_object_enumeration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_overwrite_ordering_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_overwrite_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_overwrite_ordering_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn json_build_emits_browser_bundle_overwrite_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_enumeration_overwrite_ordering_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
fn build_emits_browser_bundle_strict_equality_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: equalitySmoke\nasync function equalitySmoke(left, right) {\n  const same = left === left;\n  const different = left !== right;\n  if (!same || !different) {\n    throw new Error('unexpected equality');\n  }\n  return left - left + right - right;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "equalitySmoke");
}

#[test]
fn build_emits_browser_bundle_strict_equality_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: equalitySmoke\nasync function equalitySmoke(left, right) {\n  const same = left === left;\n  const different = left !== right;\n  if (!same || !different) {\n    throw new Error('unexpected equality');\n  }\n  return left - left + right - right;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "equalitySmoke");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.info('info');\n  console.debug('debug');\n  console.error('err');\n  console.warn('warn');\n  console.log(-1);\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["success"], true);
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

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
    assert!(stdout.contains("-1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("warn"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_js_input_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.info('info');\n  console.debug('debug');\n  console.error('err');\n  console.warn('warn');\n  console.log(-1);\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["success"], true);
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

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
    assert!(stdout.contains("-1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("warn"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_assert_routing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.assert(false, 'assert failed');\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-assert-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("assert failed"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.info('info');\n  console.debug('debug');\n  console.error('err');\n  console.warn('warn');\n  console.log(-1);\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

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
    assert!(stdout.contains("-1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("warn"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_ts_input_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.info('info');\n  console.debug('debug');\n  console.error('err');\n  console.warn('warn');\n  console.log(-1);\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["success"], true);
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

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
    assert!(stdout.contains("-1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("warn"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_assert_routing_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.assert(false, 'assert failed');\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-assert-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("assert failed"), "stderr: {stderr}");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_jsx_input() {
    assert_browser_bundle_console_level_routing_in_extension("jsx");
}

#[test]
fn build_emits_browser_bundle_console_level_routing_in_tsx_input() {
    assert_browser_bundle_console_level_routing_in_extension("tsx");
}

#[test]
fn build_emits_browser_bundle_console_assert_routing_in_jsx_input() {
    assert_browser_bundle_console_assert_routing_in_extension("jsx");
}

#[test]
fn build_emits_browser_bundle_console_assert_routing_in_tsx_input() {
    assert_browser_bundle_console_assert_routing_in_extension("tsx");
}

#[test]
fn build_emits_browser_bundle_chunks_for_literal_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import(\"./lazy.ts\");\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    let chunk_dirs: Vec<_> = fs::read_dir(&chunk_root)
        .expect("read chunk root")
        .map(|entry| entry.expect("chunk entry").path())
        .collect();
    assert!(
        !chunk_dirs.is_empty(),
        "expected at least one emitted chunk"
    );
    for chunk_dir in chunk_dirs {
        assert!(
            chunk_dir.is_dir(),
            "chunk entry should be a directory: {}",
            chunk_dir.display()
        );
    }
}

#[test]
fn build_emits_browser_bundle_chunks_for_literal_dynamic_imports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const lazy = import(\"./lazy.js\");\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    let chunk_dirs: Vec<_> = fs::read_dir(&chunk_root)
        .expect("read chunk root")
        .map(|entry| entry.expect("chunk entry").path())
        .collect();
    assert!(
        !chunk_dirs.is_empty(),
        "expected at least one emitted chunk"
    );
    for chunk_dir in chunk_dirs {
        assert!(
            chunk_dir.is_dir(),
            "chunk entry should be a directory: {}",
            chunk_dir.display()
        );
    }
}

#[test]
fn build_supports_math_log2_and_log10_const_alias_chains_in_ts_input() {
    assert_build_supports_math_log2_and_log10_const_alias_chains("main.ts");
}

#[test]
fn build_supports_math_log2_and_log10_const_alias_chains_in_js_input() {
    assert_build_supports_math_log2_and_log10_const_alias_chains("main.js");
}

#[test]
fn build_supports_math_hypot_on_perfect_square_integer_literal_sums_in_ts_input() {
    assert_build_supports_math_hypot_on_perfect_square_integer_literal_sums("main.ts");
}

#[test]
fn build_supports_math_hypot_on_perfect_square_integer_literal_sums_in_js_input() {
    assert_build_supports_math_hypot_on_perfect_square_integer_literal_sums("main.js");
}

#[test]
fn build_supports_math_hypot_on_perfect_square_integer_literal_sums_in_jsx_input() {
    assert_build_supports_math_hypot_on_perfect_square_integer_literal_sums("main.jsx");
}

#[test]
fn build_supports_math_hypot_on_perfect_square_integer_literal_sums_in_tsx_input() {
    assert_build_supports_math_hypot_on_perfect_square_integer_literal_sums("main.tsx");
}

#[test]
fn build_supports_math_sqrt_member_calls_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
}

#[test]
fn json_build_supports_math_sqrt_member_calls_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn build_supports_math_sqrt_member_calls_in_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg("build")
                .arg("--bundle")
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
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], "build");
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
        }
    }
}

#[test]
fn build_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
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
        .arg("build")
        .arg("--bundle")
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
}

#[test]
fn json_build_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
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
        .arg("build")
        .arg("--bundle")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn build_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("app.{extension}"));
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

        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg("build")
                .arg("--bundle")
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
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], "build");
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
        }
    }
}

#[test]
fn build_rejects_generator_function_lowering_in_browser_api_surface_js_input() {
    assert_generator_function_lowering_rejection_in_browser_context(
        "build",
        true,
        "js",
        "function* main() { yield 1; }\nmain();",
    );
}

#[test]
fn build_rejects_generator_and_async_generator_function_lowering_in_browser_api_surface_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for source_contents in [
            "function* main() { yield 1; }\nmain();",
            "async function* main() { yield 1; }\nmain();",
        ] {
            assert_generator_function_lowering_rejection_in_browser_context(
                "build",
                true,
                extension,
                source_contents,
            );
        }
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("build", false, "js", source);
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_jsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("build", false, "jsx", source);
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_tsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("build", false, "tsx", source);
    }
}

#[test]
fn json_build_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("build", true, "js", source);
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_js_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "build", false, true, "js", source,
        );
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_jsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "build", false, true, "jsx", source,
        );
    }
}

#[test]
fn build_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_tsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "build", false, true, "tsx", source,
        );
    }
}

#[test]
fn json_build_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_jsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "build", true, true, "jsx", source,
        );
    }
}

#[test]
fn json_build_rejects_class_generator_and_async_generator_method_lowering_in_browser_analysis_context_in_tsx_input(
) {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection_in_browser_context(
            "build", true, true, "tsx", source,
        );
    }
}

#[test]
fn build_emits_browser_bundle_chunks_for_simple_string_concat_dynamic_imports() {
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    let chunk_dirs: Vec<_> = fs::read_dir(&chunk_root)
        .expect("read chunk root")
        .map(|entry| entry.expect("chunk entry").path())
        .collect();
    assert!(
        !chunk_dirs.is_empty(),
        "expected at least one emitted chunk"
    );
    for chunk_dir in chunk_dirs {
        assert!(
            chunk_dir.is_dir(),
            "chunk entry should be a directory: {}",
            chunk_dir.display()
        );
    }
}

#[test]
fn build_emits_browser_bundle_chunks_for_const_bound_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const root = \"./\"; const name = \"lazy.ts\"; const lazy = import((root + name));\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );
}

#[test]
fn build_emits_browser_bundle_chunks_for_const_bound_dynamic_imports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const root = \"./\"; const name = \"lazy.js\"; const lazy = import((root + name));\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );
}

#[test]
fn build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.ts");
}

#[test]
fn build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.js");
}

#[test]
fn build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let chunk_path = dir.path().join("lazy.jsx");
    fs::write(
        &source_path,
        "const name = \"lazy.jsx\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.jsx");
}

#[test]
fn build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let chunk_path = dir.path().join("lazy.tsx");
    fs::write(
        &source_path,
        "const name = \"lazy.tsx\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.tsx");
}

#[test]
fn json_build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let chunk_path = dir.path().join("lazy.js");
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.js");
}

#[test]
fn json_build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let chunk_path = dir.path().join("lazy.jsx");
    fs::write(
        &source_path,
        "const name = \"lazy.jsx\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.jsx");
}

#[test]
fn json_build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let chunk_path = dir.path().join("lazy.tsx");
    fs::write(
        &source_path,
        "const name = \"lazy.tsx\"; const lazy = import(`./${name}`);\nfunction greet(name) { return name; }",
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
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.tsx");
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.ts");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "cjs");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["file"], "app.cjs");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "cjs");
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "cjs");
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_js_input_human_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
        stdout.contains("Built browser bundle (cjs) at "),
        "stdout: {stdout}"
    );

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
}

#[test]
fn build_emits_inherited_browser_bundle_cjs_artifacts_in_js_input_human_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
        stdout.contains("Built browser bundle (cjs) at "),
        "stdout: {stdout}"
    );

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
}

#[test]
fn build_emits_browser_bundle_cjs_math_max_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: maxSmoke\nfunction maxSmoke() {\n  const result = Math.max(1, 2, 3);\n  if (result !== 3) {\n    throw new Error('unexpected max');\n  }\n  return 0n;\n}\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("module.exports.maxSmoke = exported.maxSmoke"),
        "bundle js: {js}"
    );
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    fs::copy(bundle_dir.join("app.cjs"), bundle_dir.join("app.js")).expect("copy cjs bundle");
    assert_browser_bundle_executes(&bundle_dir, "maxSmoke");
}

#[test]
fn build_emits_inherited_browser_bundle_cjs_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("console_info"), "bundle js: {js}");
    assert!(js.contains("console_debug"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.js");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
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
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["file"], "app.cjs");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "cjs");
}

#[test]
fn json_build_emits_browser_bundle_cjs_artifacts_with_sandbox_in_js_input() {
    for inherited_browser_api_surface in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("app.js");
        fs::write(
            &source_path,
            "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
        )
        .expect("write source");
        if inherited_browser_api_surface {
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
        }

        let policy_path = dir.path().join("kali.policy.json");
        write_valid_policy(&policy_path);

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build");
        if !inherited_browser_api_surface {
            command.arg("--api").arg("browser");
        }
        let output = command
            .arg("--bundle")
            .arg("--format")
            .arg("cjs")
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "inherited_browser_api_surface={inherited_browser_api_surface}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let bundle_dir = dir.path().join("app");
        let js_path = bundle_dir.join("app.cjs");
        let source_map_path = bundle_dir.join("app.cjs.map");
        let meta_path = bundle_dir.join("app.meta.json");
        assert!(js_path.exists(), "missing {}", js_path.display());
        assert!(
            source_map_path.exists(),
            "missing {}",
            source_map_path.display()
        );
        assert!(meta_path.exists(), "missing {}", meta_path.display());

        let source_map: Value =
            serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
                .expect("parse source map json");
        assert_eq!(source_map["file"], "app.cjs");

        let metadata: Value =
            serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
                .expect("parse metadata json");
        assert_eq!(metadata["apiSurface"], "browser");
        assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);

        let envelope = parse_json_stdout(&output);
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "cjs");
        let artifacts = payload["artifacts"].as_array().expect("artifacts array");
        let kinds: Vec<_> = artifacts
            .iter()
            .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
            .collect();
        assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
        assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
        assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
        assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
    }
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_with_sandbox_in_js_input() {
    for inherited_browser_api_surface in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("app.js");
        fs::write(
            &source_path,
            "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
        )
        .expect("write source");
        if inherited_browser_api_surface {
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
        }

        let policy_path = dir.path().join("kali.policy.json");
        write_valid_policy(&policy_path);

        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path()).arg("build");
        if !inherited_browser_api_surface {
            command.arg("--api").arg("browser");
        }
        let output = command
            .arg("--bundle")
            .arg("--format")
            .arg("cjs")
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "inherited_browser_api_surface={inherited_browser_api_surface}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let bundle_dir = dir.path().join("app");
        let js_path = bundle_dir.join("app.cjs");
        let source_map_path = bundle_dir.join("app.cjs.map");
        let meta_path = bundle_dir.join("app.meta.json");
        assert!(js_path.exists(), "missing {}", js_path.display());
        assert!(
            source_map_path.exists(),
            "missing {}",
            source_map_path.display()
        );
        assert!(meta_path.exists(), "missing {}", meta_path.display());

        let js = fs::read_to_string(&js_path).expect("read bundle js");
        assert!(js.contains("pathToFileURL"), "bundle js: {js}");
        assert!(js.contains("console_info"), "bundle js: {js}");
        assert!(js.contains("console_debug"), "bundle js: {js}");
        assert!(js.contains("module.exports = exported"), "bundle js: {js}");
        assert!(
            js.contains("sourceMappingURL=app.cjs.map"),
            "bundle js: {js}"
        );

        let source_map: Value =
            serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
                .expect("parse source map json");
        assert_eq!(source_map["version"], 3);
        assert_eq!(source_map["file"], "app.cjs");
        assert_eq!(source_map["sources"][0], "app.js");

        let metadata: Value =
            serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
                .expect("parse metadata json");
        assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
        assert_eq!(metadata["apiSurface"], "browser");
    }
}

#[test]
fn build_artifacts_are_deterministic_across_repeated_invocations() {
    let dir = tempdir().expect("tempdir");

    let executable_source = dir.path().join("main.ts");
    fs::write(&executable_source, "console.log(1);").expect("write executable source");
    let executable_output = dir.path().join("main.wasm");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&executable_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let executable_first = read_artifact_bytes(std::slice::from_ref(&executable_output));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&executable_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(std::slice::from_ref(&executable_output), &executable_first);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--validate-ir")
        .arg(&executable_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(std::slice::from_ref(&executable_output), &executable_first);

    let library_source = dir.path().join("lib.ts");
    fs::write(
        &library_source,
        "export function add(a, b) { return a + b; }",
    )
    .expect("write library source");
    let library_wasm = dir.path().join("lib.lib.wasm");
    let library_meta = dir.path().join("lib.lib.meta.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&library_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let library_first = read_artifact_bytes(&[library_wasm.clone(), library_meta.clone()]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&library_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[library_wasm.clone(), library_meta.clone()],
        &library_first,
    );

    let browser_source = dir.path().join("app.ts");
    fs::write(&browser_source, "function greet(name) { return name; }")
        .expect("write browser source");
    let browser_root = dir.path().join("app");
    let browser_wasm = browser_root.join("app.wasm");
    let browser_js = browser_root.join("app.js");
    let browser_meta = browser_root.join("app.meta.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&browser_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let browser_first = read_artifact_bytes(&[
        browser_wasm.clone(),
        browser_js.clone(),
        browser_meta.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&browser_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            browser_wasm.clone(),
            browser_js.clone(),
            browser_meta.clone(),
        ],
        &browser_first,
    );

    let browser_cjs_source = dir.path().join("app-cjs.ts");
    fs::write(&browser_cjs_source, "function greet(name) { return name; }")
        .expect("write browser cjs source");
    let browser_cjs_root = dir.path().join("app-cjs");
    let browser_cjs_wasm = browser_cjs_root.join("app-cjs.wasm");
    let browser_cjs_js = browser_cjs_root.join("app-cjs.cjs");
    let browser_cjs_meta = browser_cjs_root.join("app-cjs.meta.json");
    let browser_cjs_map = browser_cjs_root.join("app-cjs.cjs.map");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg(&browser_cjs_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let browser_cjs_first = read_artifact_bytes(&[
        browser_cjs_wasm.clone(),
        browser_cjs_js.clone(),
        browser_cjs_meta.clone(),
        browser_cjs_map.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg(&browser_cjs_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            browser_cjs_wasm.clone(),
            browser_cjs_js.clone(),
            browser_cjs_meta.clone(),
            browser_cjs_map.clone(),
        ],
        &browser_cjs_first,
    );

    let browser_cjs_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&browser_cjs_source)
        .output()
        .expect("run kali");
    assert!(
        browser_cjs_json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&browser_cjs_json_output.stdout),
        String::from_utf8_lossy(&browser_cjs_json_output.stderr)
    );
    let browser_cjs_json_first = read_artifact_bytes(&[
        browser_cjs_wasm.clone(),
        browser_cjs_js.clone(),
        browser_cjs_meta.clone(),
        browser_cjs_map.clone(),
    ]);

    let browser_cjs_json_repeat = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&browser_cjs_source)
        .output()
        .expect("run kali");
    assert!(
        browser_cjs_json_repeat.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&browser_cjs_json_repeat.stdout),
        String::from_utf8_lossy(&browser_cjs_json_repeat.stderr)
    );
    assert_eq!(
        browser_cjs_json_output.stdout, browser_cjs_json_repeat.stdout,
        "browser bundle cjs JSON output should be stable across repeated invocations"
    );
    assert_artifact_bytes_stable(
        &[
            browser_cjs_wasm.clone(),
            browser_cjs_js.clone(),
            browser_cjs_meta.clone(),
            browser_cjs_map.clone(),
        ],
        &browser_cjs_json_first,
    );

    let browser_cjs_inherited_dir = dir.path().join("browser-cjs-inherited");
    fs::create_dir_all(&browser_cjs_inherited_dir).expect("create browser cjs inherited dir");
    let browser_cjs_inherited_source = browser_cjs_inherited_dir.join("app-cjs.ts");
    fs::write(
        &browser_cjs_inherited_source,
        "function greet(name) { return name; }",
    )
    .expect("write inherited browser cjs source");
    fs::write(
        browser_cjs_inherited_dir.join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write inherited browser cjs manifest");
    let browser_cjs_inherited_root = browser_cjs_inherited_dir.join("app-cjs");
    let browser_cjs_inherited_wasm = browser_cjs_inherited_root.join("app-cjs.wasm");
    let browser_cjs_inherited_js = browser_cjs_inherited_root.join("app-cjs.cjs");
    let browser_cjs_inherited_meta = browser_cjs_inherited_root.join("app-cjs.meta.json");
    let browser_cjs_inherited_map = browser_cjs_inherited_root.join("app-cjs.cjs.map");

    let output = Command::new(kali_bin())
        .current_dir(&browser_cjs_inherited_dir)
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg(&browser_cjs_inherited_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let browser_cjs_inherited_first = read_artifact_bytes(&[
        browser_cjs_inherited_wasm.clone(),
        browser_cjs_inherited_js.clone(),
        browser_cjs_inherited_meta.clone(),
        browser_cjs_inherited_map.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(&browser_cjs_inherited_dir)
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg(&browser_cjs_inherited_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            browser_cjs_inherited_wasm.clone(),
            browser_cjs_inherited_js.clone(),
            browser_cjs_inherited_meta.clone(),
            browser_cjs_inherited_map.clone(),
        ],
        &browser_cjs_inherited_first,
    );

    let browser_cjs_inherited_json_output = Command::new(kali_bin())
        .current_dir(&browser_cjs_inherited_dir)
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&browser_cjs_inherited_source)
        .output()
        .expect("run kali");
    assert!(
        browser_cjs_inherited_json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&browser_cjs_inherited_json_output.stdout),
        String::from_utf8_lossy(&browser_cjs_inherited_json_output.stderr)
    );
    let browser_cjs_inherited_json_first = read_artifact_bytes(&[
        browser_cjs_inherited_wasm.clone(),
        browser_cjs_inherited_js.clone(),
        browser_cjs_inherited_meta.clone(),
        browser_cjs_inherited_map.clone(),
    ]);

    let browser_cjs_inherited_json_repeat = Command::new(kali_bin())
        .current_dir(&browser_cjs_inherited_dir)
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&browser_cjs_inherited_source)
        .output()
        .expect("run kali");
    assert!(
        browser_cjs_inherited_json_repeat.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&browser_cjs_inherited_json_repeat.stdout),
        String::from_utf8_lossy(&browser_cjs_inherited_json_repeat.stderr)
    );
    assert_eq!(
        browser_cjs_inherited_json_output.stdout, browser_cjs_inherited_json_repeat.stdout,
        "browser bundle cjs JSON output should be stable across repeated inherited-browser invocations"
    );
    assert_artifact_bytes_stable(
        &[
            browser_cjs_inherited_wasm.clone(),
            browser_cjs_inherited_js.clone(),
            browser_cjs_inherited_meta.clone(),
            browser_cjs_inherited_map.clone(),
        ],
        &browser_cjs_inherited_json_first,
    );

    let capi_source = dir.path().join("lib.ts");
    fs::write(&capi_source, "export function add(a, b) { return a + b; }")
        .expect("write capi source");
    let capi_wasm = dir.path().join("lib.capi.wasm");
    let capi_meta = dir.path().join("lib.capi.meta.json");
    let capi_wit = dir.path().join("lib.wit");
    let capi_header = dir.path().join("lib.h");
    let capi_binding = dir.path().join("lib.binding-package.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg(&capi_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let capi_first = read_artifact_bytes(&[
        capi_wasm.clone(),
        capi_meta.clone(),
        capi_wit.clone(),
        capi_header.clone(),
        capi_binding.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg(&capi_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            capi_wasm.clone(),
            capi_meta.clone(),
            capi_wit.clone(),
            capi_header.clone(),
            capi_binding.clone(),
        ],
        &capi_first,
    );

    let component_source = dir.path().join("component.ts");
    fs::write(
        &component_source,
        "export function add(a, b) { return a + b; }",
    )
    .expect("write component source");
    let component_wasm = dir.path().join("component.component.wasm");
    let component_meta = dir.path().join("component.component.meta.json");
    let component_wit = dir.path().join("component.wit");
    let component_binding = dir.path().join("component.binding-package.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg(&component_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let component_first = read_artifact_bytes(&[
        component_wasm.clone(),
        component_meta.clone(),
        component_wit.clone(),
        component_binding.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg(&component_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            component_wasm.clone(),
            component_meta.clone(),
            component_wit.clone(),
            component_binding.clone(),
        ],
        &component_first,
    );
}

#[test]
fn build_with_profile_data_is_deterministic_across_repeated_invocations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "export function hot(a, b) { return a + b; }").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":"hot","weight":8}]}"#,
    )
    .expect("write profile");
    let out_dir = dir.path().join("out");
    let meta_path = out_dir.join("math.lib.meta.json");

    let build = |json_output: bool| {
        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--lib")
            .arg("--release")
            .arg("--profile")
            .arg(&profile_path)
            .arg("--max-specializations")
            .arg("24")
            .arg("--out-dir")
            .arg(&out_dir)
            .arg(&source_path);
        if json_output {
            command.arg("--output").arg("json");
        }

        let output = command.output().expect("run kali build with profile");
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        (
            output,
            fs::read(out_dir.join("math.lib.wasm")).expect("read profiled wasm"),
            fs::read(&meta_path).expect("read profiled metadata"),
        )
    };

    let (text_first, first, first_meta) = build(false);
    let (text_second, second, second_meta) = build(false);
    assert_eq!(
        text_first.stdout, text_second.stdout,
        "PGO build output should be deterministic across repeated text-mode invocations"
    );
    assert_eq!(
        first, second,
        "PGO builds should be deterministic across repeated invocations"
    );
    assert_eq!(
        first_meta, second_meta,
        "PGO metadata should be deterministic across repeated invocations"
    );

    let (json_first, json_first_wasm, json_first_meta) = build(true);
    let (json_second, json_second_wasm, json_second_meta) = build(true);
    assert_eq!(
        json_first.stdout, json_second.stdout,
        "PGO build JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(
        json_first_wasm, json_second_wasm,
        "PGO builds should be deterministic across repeated JSON invocations"
    );
    assert_eq!(
        json_first_meta, json_second_meta,
        "PGO metadata should be deterministic across repeated JSON invocations"
    );

    let envelope = parse_json_stdout(&json_first);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    assert!(envelope["payload"].is_object(), "envelope: {envelope:?}");

    let profile_data = ProfileData::new(vec![ProfileSample::new(
        ProfileSampleKind::Function,
        "hot",
        8,
    )]);
    let expected_profile_data_hash = {
        let normalized = profile_data.clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{:x}", Sha256::digest(profile_json))
    };

    let metadata: Value = serde_json::from_slice(&json_first_meta).expect("parse metadata");
    assert_artifact_metadata_provenance(
        &metadata,
        "lib",
        24,
        Some(expected_profile_data_hash.as_str()),
    );
    assert_eq!(metadata["profileDataHash"], expected_profile_data_hash);
    assert_eq!(
        envelope["payload"]["profileDataHash"],
        expected_profile_data_hash
    );
}

#[test]
fn build_rejects_unsupported_pgo_profile_data_version() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with unsupported profile version");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("unsupported PGO profile data version"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_build_rejects_unsupported_pgo_profile_data_version() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with unsupported profile version");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("unsupported PGO profile data version"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_rejects_pgo_profile_data_with_unknown_fields() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[],"unexpected":true}"#,
    )
    .expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with malformed profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(stderr.contains("unknown field"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_pgo_profile_data_with_unknown_fields() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[],"unexpected":true}"#,
    )
    .expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with malformed profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("unknown field"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_rejects_malformed_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"[1,2,3]"#).expect("write malformed profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with malformed profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("failed to parse PGO profile data"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_build_rejects_malformed_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"[1,2,3]"#).expect("write malformed profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with malformed profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("failed to parse PGO profile data"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_rejects_empty_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, "").expect("write empty profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with empty profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("failed to parse PGO profile data")
            || stderr.contains("EOF while parsing a value"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_build_rejects_empty_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, "").expect("write empty profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with empty profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("failed to parse PGO profile data")
            || errors[0]["message"]
                .as_str()
                .expect("json build rejection message")
                .contains("EOF while parsing a value"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_rejects_whitespace_only_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, " \n\t").expect("write whitespace-only profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with whitespace-only profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
    assert!(
        stderr.contains("failed to parse PGO profile data")
            || stderr.contains("EOF while parsing a value"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_build_rejects_whitespace_only_pgo_profile_data() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, " \n\t").expect("write whitespace-only profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with whitespace-only profile");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5509");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("failed to parse PGO profile data")
            || errors[0]["message"]
                .as_str()
                .expect("json build rejection message")
                .contains("EOF while parsing a value"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_uses_inherited_browser_api_surface_for_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn build_uses_inherited_browser_api_surface_for_bundle_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn build_uses_inherited_browser_api_surface_for_bundle_with_validate_ir_and_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--validate-ir")
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

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");
    assert_embeds_policy_custom_section(&wasm_path, &policy_path);
    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
}

#[test]
fn build_uses_explicit_browser_api_surface_for_bundle_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--bundle")
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

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn build_uses_inherited_browser_api_surface_for_bundle_with_sandbox_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn build_uses_explicit_browser_api_surface_for_bundle_with_sandbox_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--bundle")
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

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn json_build_emits_browser_bundle_artifacts_for_inherited_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn json_build_emits_browser_bundle_artifacts_for_explicit_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--bundle")
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

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");

    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
}

#[test]
fn json_build_emits_browser_bundle_artifacts_with_profile_data_hash() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":"greet","weight":8}]}"#,
    )
    .expect("write profile");
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
    let out_dir = dir.path().join("out");
    let meta_path = out_dir.join("app").join("app.meta.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--release")
        .arg("--profile")
        .arg(&profile_path)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
    assert_eq!(payload["hostContract"], "kali-hosted");
    assert_eq!(payload["runtimeBackend"], "wasmtime");

    let profile_data = ProfileData::new(vec![ProfileSample::new(
        ProfileSampleKind::Function,
        "greet",
        8,
    )]);
    let expected_profile_data_hash = {
        let normalized = profile_data.clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{:x}", Sha256::digest(profile_json))
    };

    assert_eq!(payload["profileDataHash"], expected_profile_data_hash);

    let metadata: Value =
        serde_json::from_slice(&fs::read(&meta_path).expect("read bundle metadata"))
            .expect("parse bundle metadata");
    assert_eq!(metadata["artifactKind"], "bundle");
    assert_eq!(metadata["profileDataHash"], expected_profile_data_hash);
    assert_eq!(metadata["buildMode"], "release");
}

#[test]
fn build_rejects_bundle_without_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_explicit_browser_api_surface_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_explicit_browser_api_surface_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_explicit_browser_api_surface_with_sandbox_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_explicit_browser_api_surface_with_sandbox_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_explicit_node_bundle_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}
#[test]
fn json_build_rejects_explicit_node_bundle_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_explicit_node_bundle_api_surface_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}
#[test]
fn json_build_rejects_explicit_node_bundle_api_surface_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_explicit_browser_library_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_explicit_node_bundle_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_explicit_node_bundle_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_explicit_browser_capi_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_explicit_browser_component_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_explicit_browser_capi_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--capi")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn json_build_rejects_explicit_browser_component_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--component")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn json_build_rejects_explicit_browser_library_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--lib")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("browser API surface"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_browser_library_oriented_api_surfaces_with_sandbox_in_js_input() {
    for args in [["--lib"], ["--capi"]] {
        for inherited_browser_api_surface in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("lib.js");
            fs::write(&source_path, "export function greet(name) { return name; }")
                .expect("write source");
            if inherited_browser_api_surface {
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
            }
            let policy_path = dir.path().join("kali.policy.json");
            write_valid_policy(&policy_path);

            let mut command = Command::new(kali_bin());
            command.current_dir(dir.path()).arg("build").args(args);
            if !inherited_browser_api_surface {
                command.arg("--api").arg("browser");
            }
            let output = command
                .arg("--sandbox")
                .arg(&policy_path)
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("E5508"), "stderr: {stderr}");
            assert!(
                stderr.contains("browser API surface"),
                "args: {args:?}\ninherited_browser_api_surface={inherited_browser_api_surface}\nstderr: {stderr}"
            );
        }
    }
}

#[test]
fn json_build_rejects_browser_library_oriented_api_surfaces_with_sandbox_in_js_input() {
    for args in [["--lib"], ["--capi"]] {
        for inherited_browser_api_surface in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("lib.js");
            fs::write(&source_path, "export function greet(name) { return name; }")
                .expect("write source");
            if inherited_browser_api_surface {
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
            }
            let policy_path = dir.path().join("kali.policy.json");
            write_valid_policy(&policy_path);

            let mut command = Command::new(kali_bin());
            command
                .current_dir(dir.path())
                .arg("--output")
                .arg("json")
                .arg("build")
                .args(args);
            if !inherited_browser_api_surface {
                command.arg("--api").arg("browser");
            }
            let output = command
                .arg("--sandbox")
                .arg(&policy_path)
                .arg(&source_path)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert!(!json["success"].as_bool().expect("success boolean"));
            assert_eq!(json["errors"][0]["code"], "E5508");
            assert!(
                json["errors"][0]["message"]
                    .as_str()
                    .expect("error message")
                    .contains("browser API surface"),
                "args: {args:?}\ninherited_browser_api_surface={inherited_browser_api_surface}\njson: {json}"
            );
        }
    }
}

#[test]
fn build_rejects_browser_component_api_surface_with_sandbox_in_js_input() {
    for inherited_browser_api_surface in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("lib.js");
        fs::write(&source_path, "export function greet(name) { return name; }")
            .expect("write source");
        if inherited_browser_api_surface {
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
        }
        let policy_path = dir.path().join("kali.policy.json");
        write_valid_policy(&policy_path);

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--component");
        if !inherited_browser_api_surface {
            command.arg("--api").arg("browser");
        }
        let output = command
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("browser API surface"),
            "inherited_browser_api_surface={inherited_browser_api_surface}\nstderr: {stderr}"
        );
    }
}

#[test]
fn json_build_rejects_browser_component_api_surface_with_sandbox_in_js_input() {
    for inherited_browser_api_surface in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("lib.js");
        fs::write(&source_path, "export function greet(name) { return name; }")
            .expect("write source");
        if inherited_browser_api_surface {
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
        }
        let policy_path = dir.path().join("kali.policy.json");
        write_valid_policy(&policy_path);

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .arg("--component");
        if !inherited_browser_api_surface {
            command.arg("--api").arg("browser");
        }
        let output = command
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert!(
            json["errors"][0]["message"]
                .as_str()
                .expect("error message")
                .contains("browser API surface"),
            "inherited_browser_api_surface={inherited_browser_api_surface}\njson: {json}"
        );
    }
}

#[test]
fn build_rejects_explicit_browser_library_oriented_api_surfaces_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");

    for args in [["--lib"], ["--capi"], ["--component"]] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .args(args)
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "args: {args:?}\nstderr: {stderr}");
        assert!(
            stderr.contains("browser API surface"),
            "args: {args:?}\nstderr: {stderr}"
        );
    }
}

#[test]
fn json_build_rejects_explicit_browser_library_oriented_api_surfaces_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");

    for args in [["--lib"], ["--capi"], ["--component"]] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .args(args)
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert!(
            json["errors"][0]["message"]
                .as_str()
                .expect("error message")
                .contains("browser API surface"),
            "args: {args:?}\njson: {json}"
        );
    }
}

#[test]
fn build_rejects_inherited_browser_library_oriented_api_surfaces() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");
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

    for args in [["--lib"], ["--capi"], ["--component"]] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .args(args)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "args: {args:?}\nstderr: {stderr}");
        assert!(
            stderr.contains("browser API surface"),
            "args: {args:?}\nstderr: {stderr}"
        );
    }
}

#[test]
fn json_build_rejects_inherited_browser_library_oriented_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.js");
    fs::write(&source_path, "export function greet(name) { return name; }").expect("write source");
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

    for args in [["--lib"], ["--capi"], ["--component"]] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .args(args)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert!(
            json["errors"][0]["message"]
                .as_str()
                .expect("error message")
                .contains("browser API surface"),
            "args: {args:?}\njson: {json}"
        );
    }
}

#[test]
fn json_build_rejects_inherited_browser_bundle_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
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
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5506");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("runtime profile"),
        "json: {json}"
    );
}

#[test]
fn json_build_rejects_inherited_browser_api_surface_with_wasm_threads_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
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
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
fn json_build_rejects_inherited_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
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
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
fn build_rejects_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
    assert!(
        stderr.contains("runtime profile") || stderr.contains("wasm-threads"),
        "stderr: {stderr}"
    );
}

#[test]
fn build_rejects_browser_api_surface_with_wasm_threads_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
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
    assert!(
        stderr.contains("runtime profile") || stderr.contains("wasm-threads"),
        "stderr: {stderr}"
    );
}

#[test]
fn build_rejects_inherited_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
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
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("runtime profile") || stderr.contains("wasm-threads"),
        "stderr: {stderr}"
    );
}

#[test]
fn build_rejects_bundle_format_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--format")
        .arg("cjs")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("--format"), "stderr: {stderr}");
}

#[test]
fn build_accepts_explicit_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    assert!(
        stdout.contains("Built executable artifact at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_accepts_process_argv_slice_length_in_js_input_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(process.argv.slice(2).length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    assert!(
        stdout.contains("Built executable artifact at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_rejects_multiple_source_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let extra_path = dir.path().join("extra.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    fs::write(&extra_path, "console.log(2);").expect("write extra source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .arg(&extra_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("only one primary source file"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_emits_capi_artifacts_and_header_compiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], 2);
    assert_eq!(metadata["minHostAbiVersion"], 2);
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["moduleName"],
        source_path.display().to_string()
    );
    assert_eq!(binding_package["artifacts"]["library"], "lib.capi.wasm");
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.capi.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.h");

    let header_check = dir.path().join("header-check.c");
    fs::write(
        &header_check,
        "#include \"lib.h\"\nint main(void) { return 0; }\n",
    )
    .expect("write header check");
    let compile = Command::new("cc")
        .current_dir(dir.path())
        .arg("-fsyntax-only")
        .arg(&header_check)
        .output()
        .expect("run cc");
    assert!(
        compile.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
}

#[test]
fn build_emits_capi_json_artifacts_for_binding_package_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg("--validate-ir")
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

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "capi");
    assert_eq!(
        PathBuf::from(payload["outputPath"].as_str().expect("capi output path")),
        source_path.with_file_name("lib.capi.wasm")
    );
    assert_eq!(
        PathBuf::from(payload["headerPath"].as_str().expect("c header path")),
        source_path.with_file_name("lib.h")
    );
    assert_eq!(
        PathBuf::from(
            payload["metadataPath"]
                .as_str()
                .expect("cabi metadata path")
        ),
        source_path.with_file_name("lib.capi.meta.json")
    );
    assert_eq!(
        PathBuf::from(payload["witPath"].as_str().expect("wit path")),
        source_path.with_file_name("lib.wit")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["kind"] == "cabi-metadata"));

    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read cabi metadata"))
            .expect("parse cabi metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );
    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["moduleName"],
        source_path.display().to_string()
    );
    assert_eq!(binding_package["artifacts"]["library"], "lib.capi.wasm");
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.capi.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.h");
}

#[test]
fn build_emits_capi_json_artifacts_with_specialization_override() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg("--max-specializations")
        .arg("8")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read cabi metadata"))
            .expect("parse cabi metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 8);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["maxSpecializations"], 8);
}

#[test]
fn build_emits_component_artifacts_and_valid_component_bytes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["artifacts"]["library"],
        "lib.component.wasm"
    );
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.component.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.wit");

    let component_bytes = fs::read(&component_path).expect("read component bytes");
    wasmparser::Validator::new()
        .validate_all(&component_bytes)
        .expect("generated component should validate");
}

#[test]
fn build_emits_component_json_artifacts_with_validate_ir() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--validate-ir")
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

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["outputPath"]
                .as_str()
                .expect("component output path")
        ),
        source_path.with_file_name("lib.component.wasm")
    );
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        source_path.with_file_name("lib.binding-package.json")
    );

    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read component metadata"))
            .expect("parse component metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["artifacts"]["library"],
        "lib.component.wasm"
    );
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.component.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.wit");

    let component_path = source_path.with_file_name("lib.component.wasm");
    let component_bytes = fs::read(&component_path).expect("read component bytes");
    wasmparser::Validator::new()
        .validate_all(&component_bytes)
        .expect("generated component should validate");
}

#[test]
fn build_emits_component_json_artifacts_for_binding_package_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
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

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        source_path.with_file_name("lib.binding-package.json")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["kind"] == "binding-package"));
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["role"] == "binding-package-manifest"));
}

#[test]
fn build_emits_component_json_artifacts_with_specialization_override() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--max-specializations")
        .arg("8")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read component metadata"))
            .expect("parse component metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 8);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["maxSpecializations"], 8);
}

#[test]
fn build_emits_component_artifacts_into_an_explicit_output_directory() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    let out_dir = dir.path().join("dist");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--output")
        .arg("json")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["outputPath"]
                .as_str()
                .expect("component output path")
        ),
        out_dir.join("lib.component.wasm")
    );
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        out_dir.join("lib.binding-package.json")
    );

    for artifact in payload["artifacts"].as_array().expect("artifacts array") {
        let artifact_path = PathBuf::from(artifact["path"].as_str().expect("artifact path string"));
        assert!(
            artifact_path.starts_with(&out_dir),
            "artifact path should stay in the requested output directory: {artifact_path:?}"
        );
    }

    assert!(out_dir.join("lib.component.wasm").exists());
    assert!(out_dir.join("lib.wit").exists());
    assert!(out_dir.join("lib.component.meta.json").exists());
    assert!(out_dir.join("lib.binding-package.json").exists());
}

#[test]
fn build_emits_capi_artifacts_on_explicit_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
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

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], 2);
    assert_eq!(metadata["minHostAbiVersion"], 2);
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["moduleName"],
        source_path.display().to_string()
    );
    assert_eq!(binding_package["artifacts"]["library"], "lib.capi.wasm");
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.capi.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.h");
}

#[test]
fn build_emits_capi_artifacts_on_inherited_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"compilerOptions":{"apiSurface":"node"}}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], 2);
    assert_eq!(metadata["minHostAbiVersion"], 2);
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["moduleName"],
        source_path.display().to_string()
    );
    assert_eq!(binding_package["artifacts"]["library"], "lib.capi.wasm");
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.capi.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.h");
}

#[test]
fn build_emits_component_artifacts_on_explicit_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
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

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read component metadata"))
            .expect("parse component metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["artifacts"]["library"],
        "lib.component.wasm"
    );
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.component.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.wit");
}

#[test]
fn build_emits_component_artifacts_on_inherited_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"compilerOptions":{"apiSurface":"node"}}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read component metadata"))
            .expect("parse component metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], "component");
    assert_eq!(metadata["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(metadata["maxSpecializations"], 16);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(binding_package["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(binding_package["hostContract"], "kali-hosted");
    assert_eq!(binding_package["runtimeBackend"], "wasmtime");
    assert_eq!(binding_package["maxSpecializations"], 16);
    assert_eq!(
        binding_package["artifacts"]["library"],
        "lib.component.wasm"
    );
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.component.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.wit");
}

#[test]
fn build_rejects_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
fn build_rejects_computed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_computed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_computed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_computed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
fn build_rejects_bracketed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_bracketed_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
fn build_rejects_mixed_bracket_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_mixed_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
fn json_build_rejects_mixed_bracket_permission_escalation_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, permission_escalation_mixed_bracketed_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
fn build_accepts_permissions_query_subset_in_js_input() {
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
        .arg("build")
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
        stdout.contains("Built executable artifact at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn build_with_sandbox_rejects_inferred_effects_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn json_build_with_sandbox_rejects_inferred_effects_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
fn json_build_with_sandbox_accepts_zero_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    let payload = json["payload"].as_object().expect("build payload object");
    assert_eq!(payload["artifactKind"], "executable");
    assert_eq!(payload["buildMode"], "fast");
    let output_path = PathBuf::from(payload["outputPath"].as_str().expect("output path"));
    assert_eq!(output_path, source_path.with_extension("wasm"));
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
}

#[test]
fn build_with_sandbox_accepts_zero_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
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
    let output_path = source_path.with_extension("wasm");
    assert!(
        output_path.exists(),
        "expected build artifact at {output_path:?}"
    );
}

#[test]
fn build_with_sandbox_accepts_zero_budget_policy_in_js_input_for_library_and_embedding_artifacts() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export function add(a, b) { return a + b; }\n",
    )
    .expect("write source");

    let cases: [(&[&str], &[&str]); 3] = [
        (
            &["--lib"],
            &["main.lib.wasm", "main.lib.wit", "main.lib.meta.json"],
        ),
        (
            &["--capi"],
            &[
                "main.capi.wasm",
                "main.wit",
                "main.h",
                "main.capi.meta.json",
                "main.binding-package.json",
            ],
        ),
        (
            &["--component"],
            &[
                "main.component.wasm",
                "main.wit",
                "main.component.meta.json",
                "main.binding-package.json",
            ],
        ),
    ];

    for (args, expected_files) in cases {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .args(args)
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "args: {args:?}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        for expected_file in expected_files {
            let expected_path = source_path.with_file_name(expected_file);
            assert!(
                expected_path.exists(),
                "missing {}",
                expected_path.display()
            );
        }
    }
}

#[test]
fn json_build_with_sandbox_accepts_zero_budget_policy_in_js_input_for_library_and_embedding_artifacts(
) {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export function add(a, b) { return a + b; }\n",
    )
    .expect("write source");

    let cases: [(&[&str], &str); 3] = [
        (&["--lib"], "lib"),
        (&["--capi"], "capi"),
        (&["--component"], "component"),
    ];

    for (args, artifact_kind) in cases {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .args(args)
            .arg("--sandbox")
            .arg(&policy_path)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "args: {args:?}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        let payload = json["payload"].as_object().expect("build payload object");
        assert_eq!(payload["artifactKind"], artifact_kind);
        assert_eq!(payload["buildMode"], "fast");

        match args[0] {
            "--lib" => {
                assert_eq!(
                    PathBuf::from(payload["outputPath"].as_str().expect("output path")),
                    source_path.with_file_name("main.lib.wasm")
                );
            }
            "--capi" => {
                assert_eq!(
                    PathBuf::from(payload["outputPath"].as_str().expect("output path")),
                    source_path.with_file_name("main.capi.wasm")
                );
            }
            "--component" => {
                assert_eq!(
                    PathBuf::from(payload["outputPath"].as_str().expect("output path")),
                    source_path.with_file_name("main.component.wasm")
                );
            }
            _ => unreachable!(),
        }

        for expected_file in match args[0] {
            "--lib" => &["main.lib.wasm", "main.lib.wit", "main.lib.meta.json"][..],
            "--capi" => &[
                "main.capi.wasm",
                "main.wit",
                "main.h",
                "main.capi.meta.json",
                "main.binding-package.json",
            ][..],
            "--component" => &[
                "main.component.wasm",
                "main.wit",
                "main.component.meta.json",
                "main.binding-package.json",
            ][..],
            _ => unreachable!(),
        } {
            let expected_path = source_path.with_file_name(expected_file);
            assert!(
                expected_path.exists(),
                "missing {}",
                expected_path.display()
            );
        }
    }
}

#[test]
fn build_with_sandbox_rejects_deno_command_spawn_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(stderr.contains("Process.Spawn"), "stderr: {stderr}");
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn json_build_with_sandbox_rejects_deno_command_spawn_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn build_with_sandbox_rejects_deno_command_spawn_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(stderr.contains("Process.Spawn"), "stderr: {stderr}");
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn json_build_with_sandbox_rejects_deno_command_spawn_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, deno_command_spawn_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn build_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(
        stderr.contains("Process.EnvWrite")
            || stderr.contains("Process.Spawn")
            || stderr.contains("Network.Connect")
            || stderr.contains("Network.Listen"),
        "stderr: {stderr}"
    );
    assert!(
        !dir.path().join("main.wasm").exists(),
        "build should not emit an artifact when sandbox validation fails"
    );
}

#[test]
fn json_build_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
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
