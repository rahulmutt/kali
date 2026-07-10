use super::*;

#[test]
fn test_supports_deno_chdir_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("smoke.test.js");
    let nested = serde_json::to_string(&nested_dir.to_string_lossy()).expect("encode nested path");
    fs::write(
        &source_path,
        format!(
            r#"Kali.test('chdir aliases', () => {{
  const nested = {nested};
  Deno.chdir(nested);
  Deno["chdir"](nested);
  globalThis.Deno.chdir(nested);
  globalThis.Deno["chdir"](nested);
  globalThis["Deno"].chdir(nested);
  globalThis["Deno"]["chdir"](nested);
  const direct = Deno.cwd();
  const bracketed = Deno["cwd"]();
  const mixed = globalThis.Deno["cwd"]();
  const inherited = globalThis["Deno"]["cwd"]();
  if (!(direct === nested && bracketed === nested && mixed === nested && inherited === nested)) {{
    throw new Error('expected cwd aliases to agree after chdir');
  }}
}});
"#
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_supports_deno_chdir_aliases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("smoke.test.js");
    let nested = serde_json::to_string(&nested_dir.to_string_lossy()).expect("encode nested path");
    fs::write(
        &source_path,
        format!(
            r#"Kali.test('chdir aliases', () => {{
  const nested = {nested};
  Deno.chdir(nested);
  Deno["chdir"](nested);
  globalThis.Deno.chdir(nested);
  globalThis.Deno["chdir"](nested);
  globalThis["Deno"].chdir(nested);
  globalThis["Deno"]["chdir"](nested);
  const direct = Deno.cwd();
  const bracketed = Deno["cwd"]();
  const mixed = globalThis.Deno["cwd"]();
  const inherited = globalThis["Deno"]["cwd"]();
  if (!(direct === nested && bracketed === nested && mixed === nested && inherited === nested)) {{
    throw new Error('expected cwd aliases to agree after chdir');
  }}
}});
"#
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_bracketed_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('env get', () => { const direct = Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const bracketed = globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const mixed = globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const inherited = globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'); if (direct !== 'hello-environment' || bracketed !== 'hello-environment' || mixed !== 'hello-environment' || inherited !== 'hello-environment') { throw new Error('expected env get'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_supports_bracketed_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('env get', () => { const direct = Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const bracketed = globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const mixed = globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'); const inherited = globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'); if (direct !== 'hello-environment' || bracketed !== 'hello-environment' || mixed !== 'hello-environment' || inherited !== 'hello-environment') { throw new Error('expected env get'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_deno_env_has_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('env has', () => { if (!(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'))) { throw new Error('expected env presence'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_HAS_SMOKE", "hello-environment")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_deno_env_has_in_jsx_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            "Kali.test('env has', () => { if (!(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'))) { throw new Error('expected env presence'); } });\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_ENV_HAS_SMOKE", "hello-environment")
            .arg("test")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "test failed: {:?}", output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn json_test_accepts_deno_env_has_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('env has', () => { if (!(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'))) { throw new Error('expected env presence'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_HAS_SMOKE", "hello-environment")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));\nKali.test('env baseline', () => {});\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let env_line = lines.next().expect("env line");
    assert_eq!(env_line, "hello-environment", "stdout: {stdout}");
    assert_eq!(lines.next(), Some("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_accepts_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('env baseline', () => { if (Deno.env.get('KALI_ENV_GET_SMOKE') !== 'hello-environment') { throw new Error('expected env'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(globalThis.Deno.pid);\nKali.test('pid baseline', () => {});\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let pid_line = lines.next().expect("pid line");
    assert!(pid_line.parse::<u32>().is_ok(), "stdout: {stdout}");
    assert_eq!(lines.next(), Some("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_bracketed_direct_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!Deno[\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_accepts_bracketed_direct_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!Deno[\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_accepts_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!Deno.pid) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_accepts_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!Deno.pid) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_accepts_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!globalThis.Deno.pid) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_accepts_bracketed_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!globalThis[\"Deno\"][\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_accepts_bracketed_global_this_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!globalThis[\"Deno\"][\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_bracketed_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!globalThis[\"Deno\"][\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test failed: {:?}", output);
}

#[test]
fn json_test_accepts_bracketed_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('pid', () => { if (!globalThis[\"Deno\"][\"pid\"]) { throw new Error('expected pid'); } });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_supports_web_baseline_structured_clone_and_event_primitives_in_ts_and_js_input() {
    let dir = tempdir().expect("tempdir");

    for ext in ["ts", "js"] {
        let source_path = dir.path().join(format!("web-baseline-test-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(true),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("test")
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
fn json_test_supports_web_baseline_structured_clone_and_event_primitives_when_browser_harness_is_configured_in_ts_and_js_input(
) {
    let dir = tempdir().expect("tempdir");

    for ext in ["ts", "js"] {
        let source_path = dir
            .path()
            .join(format!("browser-web-baseline-test-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(true),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
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
fn json_test_supports_web_baseline_structured_clone_and_event_primitives_when_browser_harness_is_configured_with_inherited_browser_api_surface_in_ts_and_js_input(
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
            .join(format!("browser-web-baseline-test-inherited-{ext}.{ext}"));
        fs::write(
            &source_path,
            structured_clone_and_event_primitives_source(true),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
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
fn test_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_threaded_runtime_globals_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_supports_late_env_materialization_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "test should succeed: {:?}", output);
}

#[test]
fn test_supports_late_env_materialization_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "test json should succeed: {:?}",
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
fn test_rejects_broader_intl_support() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_broader_intl_support_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, broader_intl_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_rejects_late_process_control_members() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_late_process_control_members_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_rejects_late_object_model_globals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_supports_unary_prefix_semantics_when_browser_harness_is_configured() {
    assert_browser_requested_unary_prefix_semantics("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_unary_prefix_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_unary_prefix_semantics("test", "smoke.test.js", false);
}

#[test]
fn json_test_supports_unary_prefix_semantics_when_browser_harness_is_configured() {
    assert_browser_requested_unary_prefix_semantics("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_unary_prefix_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_unary_prefix_semantics("test", "smoke.test.js", true);
}

#[test]
fn test_supports_unary_prefix_semantics_when_browser_api_surface_is_inherited_in_ts_and_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_unary_prefix_semantics_with_inherited_browser_api_surface(
        "test", false,
    );
}

#[test]
fn json_test_supports_unary_prefix_semantics_when_browser_api_surface_is_inherited_in_ts_and_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_unary_prefix_semantics_with_inherited_browser_api_surface(
        "test", true,
    );
}

#[test]
fn test_supports_async_await_sequencing_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
}

#[test]
fn test_supports_async_await_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
}

#[test]
fn json_test_supports_async_await_sequencing_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("async ok\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_async_await_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("async ok\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_merges_missing_tests_failed_from_browser_summary_stdout_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("merge-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser merge', () => { console.log('browser merge'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"],
        serde_json::json!({
            "total": 1,
            "passed": 0,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": json["payload"]["runtimeMs"],
            "hostContract": "browser-requested",
            "runtimeBackend": "browser-harness",
            "threadTopology": {
                "totalInstances": 0,
                "terminatedInstances": 0,
                "liveInstances": [],
            },
        })
    );
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":1"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_uses_padded_browser_summary_labels_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("padded-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser padded labels', () => { console.log('browser padded labels'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser padded labels\"],\"testsFailed\":4,\"hostContract\":\" browser-requested \",\"runtimeBackend\":\" browser-harness \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser padded labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 4);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unparseable_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unparseable-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser unparseable', () => { console.log('browser unparseable'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unparseable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_whitespace_only_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("whitespace-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser whitespace summary', () => { console.log('browser whitespace summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_empty_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("empty-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser empty summary', () => { console.log('browser empty summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, ""); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser empty summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[cfg(unix)]
#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser unreadable summary', () => { console.log('browser unreadable summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[cfg(unix)]
#[test]
fn test_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser unreadable summary', () => { console.log('browser unreadable summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("browser unreadable summary\n");'"#,
        )
        .arg("test")
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser unreadable summary', () => { console.log('browser unreadable summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[cfg(unix)]
#[test]
fn test_falls_back_to_stdout_when_browser_summary_file_is_unreadable_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unreadable-summary.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser unreadable summary', () => { console.log('browser unreadable summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("browser unreadable summary\n");'"#,
        )
        .arg("test")
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_whitespace_only_when_browser_harness_is_configured_in_source_inputs(
) {
    for source_name in [
        "whitespace-summary.test.js",
        "whitespace-summary.test.ts",
        "whitespace-summary.test.tsx",
    ] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
        fs::write(
            &source_path,
            "Kali.test('browser whitespace summary', () => { console.log('browser whitespace summary'); });\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .env(
                "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace summary\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
            )
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "source: {source_name}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("\"testsFailed\":0"),
            "source: {source_name}\njson: {json}"
        );
        assert_eq!(json["stderr"], "");
    }
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_failed_type_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid summary', () => { console.log('browser invalid summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 7);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_non_integer_tests_failed_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("non-integer-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid summary', () => { console.log('browser invalid summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":1.5,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 7);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_non_integer_tests_failed_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("non-integer-summary.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid summary', () => { console.log('browser invalid summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":1.5,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid summary\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 7);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid labels', () => { console.log('browser invalid labels'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 4);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_and_invalid_args_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-and-args.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid labels and args', () => { console.log('browser invalid labels and args'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_array_items_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-array-items.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid tests array items', () => { console.log('browser invalid tests array items'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid tests array items\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid tests array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_args_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-args.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid args', () => { console.log('browser invalid args'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid args\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_array_items_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-array-items.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid array items', () => { console.log('browser invalid array items'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid tests array items', () => { console.log('browser invalid tests array items'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_tests_array_items_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-tests-array-items.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid tests array items', () => { console.log('browser invalid tests array items'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_args_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-args.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser invalid args', () => { console.log('browser invalid args'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid args\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid args\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid labels', () => { console.log('browser invalid labels'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 4);
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
fn json_test_falls_back_to_stdout_when_browser_summary_file_has_invalid_labels_and_invalid_args_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("invalid-labels-and-args.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser invalid labels and args', () => { console.log('browser invalid labels and args'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 8);
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
fn json_test_uses_stdout_metadata_when_browser_summary_file_has_unexpected_top_level_keys_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("unexpected-top-level-keys.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser unexpected top-level keys', () => { console.log('browser unexpected top-level keys'); });\n",
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_falls_back_to_stdout_when_browser_summary_file_is_incomplete_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("incomplete-summary.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser incomplete summary', () => { console.log('browser incomplete summary'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser incomplete summary\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser incomplete summary\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#,
        )
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 1);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("\"testsFailed\":1"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_array_from_iteration_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
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
fn json_test_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_json_test_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_input_when_a_browser_harness_command_is_configured(extension);
    }
}

#[test]
fn test_supports_try_catch_and_finally_sequencing_when_browser_harness_is_configured_with_inherited_browser_api_surface(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, browser_runtime_try_catch_and_finally_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_try_catch_and_finally_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, browser_runtime_try_catch_and_finally_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_ceil_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
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
fn test_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
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
        .arg("test")
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
    assert!(stdout.contains("3\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_suite_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_browser_like_executables() {
    test_browser_entrypoint_smoke("chromium");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_source_paths_with_spaces() {
    browser_entrypoint_smoke(
        "test",
        "browser entry.test.ts",
        "console.log('browser test');\nKali.test('browser test', () => { 1 + 1; });",
        "browser test",
        "chromium",
    );
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_google_chrome_stable_executables() {
    for browser_name in ["google-chrome-stable", "google chrome stable"] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_chrome_aliases() {
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
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_remaining_browser_aliases() {
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
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_mullvad_browser_executables() {
    test_browser_entrypoint_smoke("mullvad-browser");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_command_wrapped_executables() {
    test_browser_entrypoint_smoke("chromium.command");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_edge_beta_executables() {
    test_browser_entrypoint_smoke("edge-beta");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_msedge_canary_executables() {
    test_browser_entrypoint_smoke("msedge-canary");
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_msedge_stable_executables() {
    for browser_name in [
        "msedge-stable",
        "edge-stable",
        "microsoft-edge-stable",
        "microsoft edge stable",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_edge_aliases() {
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
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_brave_browser_stable_executables() {
    for browser_name in ["brave-browser-stable", "brave browser stable"] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_brave_aliases() {
    for browser_name in [
        "brave-browser-beta",
        "brave-browser-dev",
        "brave-browser-nightly",
        "brave browser beta",
        "brave browser dev",
        "brave browser nightly",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_stable_browser_aliases() {
    for browser_name in ["firefox-esr", "opera-stable", "vivaldi-stable"] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_opera_aliases() {
    for browser_name in [
        "opera-beta",
        "opera-developer",
        "opera-unstable",
        "opera beta",
        "opera developer",
        "opera unstable",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_firefox_aliases() {
    for browser_name in [
        "firefox",
        "firefox-beta",
        "firefox-nightly",
        "firefox-developer-edition",
        "firefox developer edition",
        "firefox beta",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_chrome_for_testing_aliases() {
    for browser_name in [
        "chrome-for-testing",
        "chromium-for-testing",
        "google-chrome-for-testing",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[cfg(unix)]
#[test]
fn test_uses_browser_entrypoint_for_additional_privacy_browser_aliases() {
    for browser_name in [
        "librewolf",
        "waterfox",
        "zen-browser",
        "zen browser",
        "thorium-browser",
        "thorium browser",
    ] {
        test_browser_entrypoint_smoke(browser_name);
    }
}

#[test]
fn test_reports_success_for_explicit_file_sets() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_the_explicit_deno_api_surface() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_the_browser_api_surface_when_a_harness_command_is_configured() {
    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_the_browser_api_surface_when_a_harness_command_is_configured_in_js_input() {
    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.js"))
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
fn test_supports_boolean_logic_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert!(stdout.contains("1\n2\n3\n4\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert!(stdout.contains("1\n2\n3\n4\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert!(stdout.contains("1\n2\n3\n4\nok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_boolean_logic_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
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
fn json_test_supports_boolean_logic_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
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
fn test_supports_strict_equality_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_strict_equality_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1\n"),
        "json: {json}"
    );
    assert!(
        json["stdout"].as_str().expect("stdout").contains("2\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_strict_equality_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_strict_equality_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1\n"),
        "json: {json}"
    );
    assert!(
        json["stdout"].as_str().expect("stdout").contains("2\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_object_enumeration_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_object_string_primitive_enumeration_semantics_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
Kali.test('string primitive enumeration', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_object_enumeration_semantics_when_browser_harness_is_configured_in_ts_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn json_test_supports_object_enumeration_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_supports_object_enumeration_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_supports_object_enumeration_integer_like_key_ordering_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_object_enumeration_integer_like_key_ordering_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_supports_math_suite_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\nconsole.log(Math.clz32(1));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("3\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
    assert!(stdout.contains("31\n"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_suite_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\nconsole.log(Math.imul(2147483647, 2));\nconsole.log(Math.clz32(1));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("3\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
    assert!(stdout.contains("31\n"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_suite_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert!(stdout.contains("31\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_suite_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-2\n"), "json: {json}");
    assert!(stdout.contains("31\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_trunc_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_trunc_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_imul_semantics_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_imul_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
}

#[test]
fn test_accepts_zero_budget_pair_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('zero budget pair', () => { console.log(0); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("0\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_clz32_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("31\n"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_clz32_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("31\n"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_clz32_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_clz32_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_ceil_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("-3\n"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_ceil_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_ceil_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("-3\n"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_ceil_semantics_when_browser_harness_is_configured_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("-3\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_ceil_semantics_when_browser_harness_is_configured_with_inherited_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn test_supports_try_finally_sequencing_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_supports_try_finally_sequencing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"async function main() {
  try {
    for await (const value of [1, 2]) {
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
        .arg("test")
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
fn test_supports_try_catch_exception_handling_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_supports_try_catch_exception_handling_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn test_supports_queue_microtask_ordering_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn json_test_supports_queue_microtask_ordering_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn json_test_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn json_test_supports_queue_microtask_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_supports_queue_microtask_ordering_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn json_test_supports_queue_microtask_ordering_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, browser_runtime_queue_microtask_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
Kali.test('browser runtime smoke', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_dynamic_import_file_specifier_targets_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
Kali.test('browser runtime smoke', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
}

#[test]
fn test_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
}

#[test]
fn test_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn json_test_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"].get("hostContract").and_then(Value::as_str),
        Some("browser-requested")
    );
    assert_eq!(
        json["payload"]
            .get("runtimeBackend")
            .and_then(Value::as_str),
        Some("browser-harness")
    );
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn test_supports_performance_now_monotonic_ordering_when_browser_harness_is_configured_in_ts_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn test_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn test_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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
fn json_test_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_performance_now_monotonic_ordering_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("performance.now ok"),
        "json: {json}"
    );
}

#[test]
fn test_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_trunc_builtin_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");
    write_browser_api_surface_manifest(dir.path());

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_clz32_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_console_assert_routing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.assert(false, 'assert failed');\nKali.test('browser console assert', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(json) => {
            assert_eq!(json["command"], "test");
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
        Err(_) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("assert failed"),
                "stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn test_supports_console_level_routing_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.info('info');\nconsole.debug('debug');\nconsole.error('err');\nconsole.warn('warn');\nconsole.log(-1);\nKali.test('browser console routing', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
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

#[test]
fn test_reports_function_coverage_in_json_output_when_browser_api_surface_is_configured() {
    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--coverage")
        .arg("--output")
        .arg("json")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_reports_function_coverage_in_json_output_when_browser_api_surface_is_inherited() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let fixture = fs::read_to_string(fixture_path("tests/smoke.test.ts")).expect("read fixture");
    fs::write(&source_path, fixture).expect("write source");
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
        .arg("test")
        .arg("--coverage")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_uses_browser_package_resolution_when_a_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.test.ts");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\nKali.test('browser package', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn test_uses_browser_package_resolution_when_a_harness_command_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\nKali.test('browser package', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_test_uses_browser_package_resolution_when_a_harness_command_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\nKali.test('browser package', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert_eq!(json["stdout"], "0\n", "json: {json}");
}

#[test]
fn test_uses_browser_package_resolution_when_the_browser_api_surface_is_inherited() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    write_browser_runtime_package_fixture(&package_dir, "browserpkg");

    let source_path = dir.path().join("main.test.ts");
    fs::write(
        &source_path,
        "import describe from 'browserpkg';\nconsole.log(describe());\nKali.test('browser package', () => { 1 + 1; });\n",
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
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_max_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_min_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.min(3, 2, 1));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_abs_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.abs(3 - 6));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_sign_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.sign(3 - 6));
",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("-1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_hypot_semantics_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_supports_math_hypot_semantics("test", "smoke.test.js", "5", true);
}

#[test]
fn test_supports_math_hypot_semantics_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_supports_math_hypot_semantics("test", "smoke.test.jsx", "5", true);
}

#[test]
fn test_supports_math_hypot_semantics_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_supports_math_hypot_semantics("test", "smoke.test.tsx", "5", true);
}

#[test]
fn test_uses_browser_exports_condition_package_resolution_when_a_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserexports");
    write_browser_runtime_exports_package_fixture(&package_dir, "browserexports");

    let source_path = dir.path().join("main.test.ts");
    fs::write(
        &source_path,
        "import describe from 'browserexports';\nconsole.log(describe());\nKali.test('browser exports package', () => { 1 + 1; });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn test_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_threaded_profile_with_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--wasm-threads")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn json_test_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_rejects_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn json_test_rejects_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
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
fn test_accepts_wasm_threads_runtime_profile() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--wasm-threads")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn test_accepts_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
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
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_wasm_threads_runtime_profile_in_js_input() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--wasm-threads")
        .arg(fixture_path("tests/smoke.test.js"))
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
fn test_accepts_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "1 + 2;").expect("write source");
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
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
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
        .arg("test")
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
fn test_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("test")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 2"), "stdout: {stdout}");
}

#[test]
fn test_filters_selected_files_before_execution() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(&keep, "1 + 2;").expect("write keep source");
    fs::write(&skip, "3 + 4;").expect("write skip source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
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
fn test_reports_function_coverage_for_explicit_file_sets() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1 (coverage:"), "stdout: {stdout}");
}

#[test]
fn test_reports_function_coverage_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_coverage_reaches_100_percent_when_every_user_function_is_exercised() {
    // A named function declaration referenced by name as the `Kali.test`
    // callback compiles to its own real wasm function (unlike an inline
    // anonymous arrow, which the front end currently lowers to a bare
    // `"unknown"` placeholder value and never registers as a callback at
    // all — a separate, pre-existing quirk, not something this test
    // exercises). Here exactly two source-level functions are emitted,
    // `_start` and `cb`, and executing the suite runs both. The synthetic,
    // uninstrumented `__alloc` helper (added for the reclaiming allocator)
    // must NOT be counted in `functionsTotal`, or 100% coverage becomes
    // structurally unreachable even though every user function ran.
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("cb.test.ts");
    fs::write(
        &source_path,
        r#"function cb() {
    1 + 2;
}
Kali.test("addition", cb);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
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
    let summary = &json["payload"]["coverage"]["summary"];
    assert_eq!(summary["functionsTotal"], json!(2), "summary: {summary}");
    assert_eq!(summary["functionsCovered"], json!(2), "summary: {summary}");
    assert_eq!(summary["functionsMissed"], json!(0), "summary: {summary}");
    assert_eq!(
        summary["coveragePercent"],
        json!(100.0),
        "summary: {summary}"
    );
}

#[test]
fn test_reports_function_coverage_in_pretty_json_output() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_reports_function_coverage_in_json_output_under_quiet() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--quiet")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_reports_function_coverage_with_normalized_relative_paths() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 1, "coverage files: {files:?}");
    let file = files[0]["file"].as_str().expect("coverage file path");
    assert!(
        file.ends_with("tests/fixtures/tests/smoke.test.ts"),
        "file path should be project-root relative: {file}"
    );
    assert!(
        !Path::new(file).is_absolute(),
        "file path should not be absolute: {file}"
    );
}

#[test]
fn test_reports_function_coverage_in_deterministic_file_order() {
    let dir = tempdir().expect("tempdir");
    let first_path = dir.path().join("z.test.ts");
    let second_path = dir.path().join("a.test.ts");
    fs::write(
        &first_path,
        r#"Kali.test("z", () => {
    1 + 1;
});
"#,
    )
    .expect("write first test file");
    fs::write(
        &second_path,
        r#"Kali.test("a", () => {
    2 + 2;
});
"#,
    )
    .expect("write second test file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(&first_path)
        .arg(&second_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 2, "coverage files: {files:?}");

    let first_file = files[0]["file"].as_str().expect("first coverage file path");
    let second_file = files[1]["file"]
        .as_str()
        .expect("second coverage file path");

    assert!(
        first_file.ends_with("a.test.ts"),
        "coverage rows should be sorted deterministically: {first_file}"
    );
    assert!(
        second_file.ends_with("z.test.ts"),
        "coverage rows should be sorted deterministically: {second_file}"
    );
    assert!(
        !Path::new(first_file).is_absolute() && !Path::new(second_file).is_absolute(),
        "coverage file paths should be relative: {files:?}"
    );
}

#[test]
fn test_reports_function_coverage_is_deterministic_across_repeated_runs() {
    let dir = tempdir().expect("tempdir");
    let first_path = dir.path().join("z.test.ts");
    let second_path = dir.path().join("a.test.ts");
    fs::write(
        &first_path,
        r#"Kali.test("z", () => {
    1 + 1;
});
"#,
    )
    .expect("write first test file");
    fs::write(
        &second_path,
        r#"Kali.test("a", () => {
    2 + 2;
});
"#,
    )
    .expect("write second test file");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("test")
            .arg("--output")
            .arg("json")
            .arg("--coverage")
            .arg(&first_path)
            .arg(&second_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "first stdout: {}\nfirst stderr: {}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "second stdout: {}\nsecond stderr: {}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json["payload"]["coverage"], second_json["payload"]["coverage"],
        "coverage output should stay identical across repeated runs"
    );
}

#[test]
fn test_reports_function_coverage_respects_filter_selection() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(
        &keep,
        r#"Kali.test("math", () => {
    1 + 1;
});
"#,
    )
    .expect("write keep test file");
    fs::write(
        &skip,
        r#"Kali.test("strings", () => {
    2 + 2;
});
"#,
    )
    .expect("write skip test file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 1, "coverage files: {files:?}");
    let file = files[0]["file"].as_str().expect("coverage file path");
    assert!(
        file.ends_with("math.test.ts"),
        "coverage should only report filtered files: {file}"
    );
    assert!(
        !Path::new(file).is_absolute(),
        "coverage file path should be relative: {file}"
    );
}

#[test]
fn test_reports_function_coverage_for_empty_filter_matches() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(
        &keep,
        r#"Kali.test("math", () => {
    1 + 1;
});
"#,
    )
    .expect("write keep test file");
    fs::write(
        &skip,
        r#"Kali.test("strings", () => {
    2 + 2;
});
"#,
    )
    .expect("write skip test file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg("--filter")
        .arg("nomatch")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["total"], 0);
    assert_eq!(json["payload"]["passed"], 0);
    assert_eq!(json["payload"]["failed"], 0);
    let coverage = &json["payload"]["coverage"];
    assert_eq!(coverage["mode"], "function");
    assert_eq!(coverage["files"], json!([]));
    assert_eq!(coverage["summary"]["functionsTotal"], 0);
    assert_eq!(coverage["summary"]["functionsCovered"], 0);
    assert_eq!(coverage["summary"]["functionsMissed"], 0);
    assert_eq!(coverage["summary"]["coveragePercent"], json!(100.0));
}

#[test]
fn test_accepts_node_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('node smoke', () => { console.log('node test'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert_eq!(stdout, "node test\nok 1\n", "stdout: {stdout}");
}

#[test]
fn test_supports_browser_web_crypto_get_random_values_when_browser_api_surface_is_inherited_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_requested_web_crypto_get_random_values_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.ts",
    );
}

#[test]
fn test_supports_browser_web_crypto_get_random_values_when_browser_api_surface_is_inherited_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_requested_web_crypto_get_random_values_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.js",
    );
}

#[test]
fn test_supports_arithmetic_precedence_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(1 + 2 * 3);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("7\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_array_literal_length_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log([1, 2, 3].length);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_promise_all_sequencing() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, promise_all_sequencing_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_promise_all_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, promise_all_sequencing_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.js", false, false);
}

#[test]
fn test_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.js", false, true);
}

#[test]
fn json_test_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.js", true, false);
}

#[test]
fn json_test_supports_browser_requested_promise_all_sequencing_in_js_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.js", true, true);
}

#[test]
fn test_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.ts", false, false);
}

#[test]
fn test_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.ts", false, true);
}

#[test]
fn json_test_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.ts", true, false);
}

#[test]
fn json_test_supports_browser_requested_promise_all_sequencing_in_ts_input_when_browser_api_surface_is_inherited_when_browser_harness_is_configured(
) {
    assert_browser_requested_promise_all_sequencing("test", "smoke.test.ts", true, true);
}

#[test]
fn test_supports_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_async_await_sequencing_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
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
fn json_test_supports_async_await_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
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
fn test_rejects_async_class_method_sequencing_in_ts_js_jsx_and_tsx_input() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            r#"Kali.test('async class method', () => {
  async function main() {
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

  return main();
});
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("test")
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
fn json_test_rejects_async_class_method_sequencing_in_ts_js_jsx_and_tsx_input() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            r#"Kali.test('async class method', () => {
  async function main() {
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

  return main();
});
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "test");
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
fn test_supports_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_queue_microtask_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
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
fn test_supports_literal_string_dynamic_import_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
Kali.test('literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_literal_string_dynamic_import_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
Kali.test('literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("main loaded"),
        "json: {json}"
    );
}

#[test]
fn test_supports_literal_string_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
Kali.test('literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_template_literal_dynamic_import_targets_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        dir.path().join("lazy.ts"),
        "console.log('lazy loaded'); export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const name = "lazy.ts";
  await import(`./${name}`);
  console.log("main loaded");
}
main();
Kali.test('template literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_literal_string_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
Kali.test('literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("main loaded"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_template_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        dir.path().join("lazy.js"),
        "console.log('lazy loaded'); export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  const name = "lazy.js";
  await import(`./${name}`);
  console.log("main loaded");
}
main();
Kali.test('template literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("main loaded"),
        "json: {json}"
    );
}

#[test]
fn test_supports_object_freeze_wrapped_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        dir.path().join("lazy.js"),
        "console.log('lazy loaded'); export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        r#"async function main() {
  await import(Object.freeze("./lazy.js"));
  console.log("main loaded");
}
main();
Kali.test('object.freeze literal dynamic import', () => {});
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_optional_chaining_semantics_in_js_input() {
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
        dir.path().join("smoke.test.js"),
        r#"import { minVersion } from 'semver';
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(dir.path().join("smoke.test.js"))
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
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1.2.3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_try_catch_exception_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn test_supports_try_finally_sequencing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"async function main() {
  try {
    for await (const value of [1, 2]) {
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
        .arg("test")
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
fn test_supports_crypto_get_random_values_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(stdout.trim(), "ok\nok 1", "stdout: {stdout}");
}

#[test]
fn test_supports_browser_web_crypto_subtle_digest_and_random_uuid_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn test_supports_browser_web_crypto_subtle_digest_and_random_uuid_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn test_supports_browser_web_crypto_get_random_values_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn test_supports_browser_web_crypto_get_random_values_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

#[test]
fn test_supports_optional_chaining_semantics_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        dir.path().join("smoke.test.js"),
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("1.2.3"),
        "json: {json}"
    );
}

#[test]
fn test_supports_relational_comparison_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "if (1 < 2) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_strict_inequality_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "if (1 !== 0) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_strict_inequality_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "if (1 !== 0) { console.log(3); }\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_object_string_primitive_enumeration_semantics_in_js_input() {
    assert_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.js",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn test_supports_object_string_primitive_enumeration_semantics_in_ts_input() {
    assert_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.ts",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn test_supports_object_string_primitive_enumeration_semantics_in_jsx_input() {
    assert_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.jsx",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn test_supports_object_string_primitive_enumeration_semantics_in_tsx_input() {
    assert_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.tsx",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn json_test_supports_object_string_primitive_enumeration_semantics_in_js_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.js",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn json_test_supports_object_string_primitive_enumeration_semantics_in_ts_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.ts",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn json_test_supports_object_string_primitive_enumeration_semantics_in_jsx_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.jsx",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn json_test_supports_object_string_primitive_enumeration_semantics_in_tsx_input() {
    assert_json_object_string_primitive_enumeration_semantics(
        "test",
        "smoke.test.tsx",
        object_string_primitive_enumeration_semantics_test_source(),
    );
}

#[test]
fn json_test_supports_object_enumeration_semantics_in_js_input() {
    assert_json_object_enumeration_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_object_from_entries_enumeration_semantics_in_js_input() {
    assert_json_object_from_entries_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_frozen_object_enumeration_spread_semantics_in_js_input() {
    assert_json_frozen_object_enumeration_spread_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_frozen_object_enumeration_spread_semantics_in_ts_input() {
    assert_json_frozen_object_enumeration_spread_semantics("test", "smoke.test.ts");
}

#[test]
fn test_supports_object_from_entries_enumeration_semantics_with_satisfies_wrapper_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_object_from_entries_satisfies_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("test")
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
}

#[test]
fn test_supports_object_from_entries_has_own_semantics_in_jsx_input_when_browser_harness_is_configured(
) {
    assert_browser_runtime_object_from_entries_has_own_semantics_in_input(
        "test",
        "smoke.test.jsx",
        browser_runtime_object_from_entries_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_from_entries_has_own_semantics_in_tsx_input_when_browser_harness_is_configured(
) {
    assert_browser_runtime_object_from_entries_has_own_semantics_in_input(
        "test",
        "smoke.test.tsx",
        browser_runtime_object_from_entries_has_own_test_source(),
        false,
    );
}

#[test]
fn json_test_supports_frozen_object_enumeration_spread_semantics_in_jsx_input_when_browser_harness_is_configured(
) {
    assert_json_browser_runtime_frozen_object_enumeration_spread_semantics_in_input(
        "test",
        "smoke.test.jsx",
        browser_runtime_frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn json_test_supports_frozen_object_enumeration_spread_semantics_in_tsx_input_when_browser_harness_is_configured(
) {
    assert_json_browser_runtime_frozen_object_enumeration_spread_semantics_in_input(
        "test",
        "smoke.test.tsx",
        browser_runtime_frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn test_supports_object_property_deletion_semantics() {
    assert_object_property_deletion_semantics("test", "smoke.test.ts");
}

#[test]
fn test_supports_object_property_deletion_semantics_in_js_input() {
    assert_object_property_deletion_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_object_property_deletion_semantics() {
    assert_json_object_property_deletion_semantics("test", "smoke.test.ts");
}

#[test]
fn json_test_supports_object_property_deletion_semantics_in_js_input() {
    assert_json_object_property_deletion_semantics("test", "smoke.test.js");
}

#[test]
fn test_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_requested_object_property_deletion_semantics("test", "smoke.test.ts");
}

#[test]
fn test_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_requested_object_property_deletion_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_json_browser_requested_object_property_deletion_semantics("test", "smoke.test.ts");
}

#[test]
fn json_test_supports_browser_requested_object_property_deletion_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_json_browser_requested_object_property_deletion_semantics("test", "smoke.test.js");
}

#[test]
fn test_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.ts",
    );
}

#[test]
fn test_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.js",
    );
}

#[test]
fn json_test_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    assert_json_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.ts",
    );
}

#[test]
fn json_test_supports_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited_in_js_input_when_a_browser_harness_command_is_configured(
) {
    assert_json_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
        "test",
        "smoke.test.js",
    );
}

#[test]
fn test_supports_object_type_and_constructor_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(true),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_supports_object_type_and_constructor_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(true),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_supports_object_type_and_constructor_semantics() {
    assert_json_object_type_and_constructor_semantics("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_object_type_and_constructor_semantics_in_js_input() {
    assert_json_object_type_and_constructor_semantics("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_browser_requested_object_type_and_constructor_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    assert_json_browser_requested_object_type_and_constructor_semantics(
        "test",
        "smoke.test.ts",
        true,
    );
}

#[test]
fn json_test_supports_browser_requested_object_type_and_constructor_semantics_when_browser_harness_is_configured_in_js_input(
) {
    assert_json_browser_requested_object_type_and_constructor_semantics(
        "test",
        "smoke.test.js",
        true,
    );
}

#[test]
fn test_supports_unary_prefix_semantics() {
    assert_unary_prefix_semantics("test", "smoke.test.ts");
}

#[test]
fn test_supports_unary_prefix_semantics_in_js_input() {
    assert_unary_prefix_semantics("test", "smoke.test.js");
}

#[test]
fn json_test_supports_unary_prefix_semantics() {
    assert_json_unary_prefix_semantics("test", "smoke.test.ts");
}

#[test]
fn json_test_supports_unary_prefix_semantics_in_js_input() {
    assert_json_unary_prefix_semantics("test", "smoke.test.js");
}

#[test]
fn test_supports_unary_prefix_semantics_with_browser_harness_in_ts_input() {
    assert_browser_unary_prefix_semantics("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_unary_prefix_semantics_with_browser_harness_in_ts_input() {
    assert_json_browser_unary_prefix_semantics("test", "smoke.test.ts", true);
}

#[test]
fn test_supports_wrapped_mutable_update_targets_with_browser_harness_in_ts_input() {
    assert_browser_wrapped_mutable_update_targets("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_wrapped_mutable_update_targets_with_browser_harness_in_ts_input() {
    assert_json_browser_wrapped_mutable_update_targets("test", "smoke.test.ts", true);
}

#[test]
fn test_supports_wrapped_mutable_compound_assignment_targets_with_browser_harness_in_ts_input() {
    assert_browser_wrapped_mutable_compound_assignment_targets("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_wrapped_mutable_compound_assignment_targets_with_browser_harness_in_ts_input()
{
    assert_json_browser_wrapped_mutable_compound_assignment_targets("test", "smoke.test.ts", true);
}

#[test]
fn test_supports_bigint_addition_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(1n + 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_bigint_multiplication_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(1n * 2n);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_bigint_remainder_semantics_in_js_input() {
    assert_test_supports_bigint_binary_semantics("js", "3n % 2n", "1");
}

#[test]
fn test_supports_bigint_exponentiation_semantics_in_js_input() {
    assert_test_supports_bigint_binary_semantics("js", "2n ** 3n", "8");
}

#[test]
fn test_supports_math_max_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.max(1, 2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_max_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.max(1, 2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.pow(2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("8\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.pow(2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("8\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_exponent_one_identity_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const exponent = 1; const alias = exponent; console.log(Math.pow(2, alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_base_one_identity_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const exponent = 7; const alias = exponent; console.log(Math.pow(1, alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_negative_integer_exponents_for_unit_bases_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias)); console.log(Math.pow(-1, alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1\n-1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_negative_base_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.pow(-2, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-8\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_zero_exponent_for_non_integer_base_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.pow(1.6, 0));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_pow_zero_base_positive_integer_exponents_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.pow(0, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("0\nok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_pow_zero_base_positive_integer_exponents_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.pow(0, 3));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert_eq!(stdout, "0\n", "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_min_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.min(3, 2, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_min_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.min(3, 2, 1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_abs_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.abs(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_abs_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.abs(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_sign_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.sign(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-1\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_sign_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.sign(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-1\nok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_max_min_abs_sign_suite_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_math_max_min_abs_sign_suite_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.max(1, 2, 3));\nconsole.log(Math.min(3, 2, 1));\nconsole.log(Math.abs(3 - 6));\nconsole.log(Math.sign(3 - 6));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    let stdout = json["stdout"].as_str().expect("stdout");
    assert!(stdout.contains("3\n"), "json: {json}");
    assert!(stdout.contains("1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert!(stdout.contains("-1\n"), "json: {json}");
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_imul_builtin_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_imul_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_imul_builtin_omitted_operands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.imul());\nconsole.log(Math.imul(7));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("0\n0\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_clz32_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.clz32(1.6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("31\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_trunc_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-3\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_ceil_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("-3\nok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_ceil_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.ceil(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_math_clz32_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.clz32(1));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("31\n"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_math_imul_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.imul(2147483647, 2));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-2\n"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_math_imul_builtin_omitted_operands_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.imul());\nconsole.log(Math.imul(7));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n0"),
        "json: {json}"
    );
}

#[test]
fn json_test_supports_math_trunc_builtin_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.trunc(3 - 6));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stderr"], "");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("-3\n"),
        "json: {json}"
    );
}

#[test]
fn test_supports_boolean_logic_semantics() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
    assert!(stdout.contains("1\n2\n3\n4\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_boolean_logic_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert!(stdout.contains("1\n2\n3\n4\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_nested_math_call_composition_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('nested math calls', () => { console.log(Math.max(Math.min(1, 2), Math.abs(3 - 6))); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("3\nok 1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_nested_math_call_composition_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('nested math calls', () => { console.log(Math.max(Math.min(1, 2), Math.abs(3 - 6))); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("3\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_object_keys_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert!(stdout.contains("2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_object_entries_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert!(stdout.contains("2\nok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_object_values_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn test_supports_object_enumeration_semantics_with_overwrite_ordering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_supports_object_enumeration_semantics_with_overwrite_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_supports_object_enumeration_integer_like_key_ordering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_supports_object_enumeration_integer_like_key_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_object_enumeration_semantics_with_overwrite_ordering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_supports_object_enumeration_semantics_with_overwrite_ordering_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, object_enumeration_overwrite_ordering_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_set_and_map_constructor_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, set_and_map_iteration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_supports_set_and_map_constructor_iteration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, set_and_map_iteration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_console_level_routing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.info('info');\nconsole.debug('debug');\nconsole.error('err');\nconsole.warn('warn');\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("info"), "stdout: {stdout}");
    assert!(stdout.contains("debug"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("[warn] warn"), "stderr: {stderr}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_rejects_non_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_non_literal_dynamic_import_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_non_literal_dynamic_import_targets_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn test_supports_nullish_assignment_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        // fasta Spec 7 Task 3: scalar `??=` rejects fail-closed; the surviving
        // `??=` lowering is a for-in-key ALIAS binding (`-1` null sentinel).
        r#"Kali.test('browser nullish assignment', () => { var table = { a: 1, b: 2 }; var last = null; for (var c in table) { last = c; } last ??= null; if (last) { return 1; } return 0; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_nullish_assignment_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        // fasta Spec 7 Task 3: scalar `??=` rejects fail-closed; the surviving
        // `??=` lowering is a for-in-key ALIAS binding (`-1` null sentinel).
        r#"Kali.test('browser nullish assignment', () => { var table = { a: 1, b: 2 }; var last = null; for (var c in table) { last = c; } last ??= null; if (last) { return 1; } return 0; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
}

#[test]
fn test_supports_nullish_assignment_in_browser_api_surface_with_harness_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        // fasta Spec 7 Task 3: scalar `??=` rejects fail-closed; the surviving
        // `??=` lowering is a for-in-key ALIAS binding (`-1` null sentinel).
        r#"Kali.test('browser nullish assignment', () => { var table = { a: 1, b: 2 }; var last = null; for (var c in table) { last = c; } last ??= null; if (last) { return 1; } return 0; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_nullish_assignment_in_browser_api_surface_with_harness_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        // fasta Spec 7 Task 3: scalar `??=` rejects fail-closed; the surviving
        // `??=` lowering is a for-in-key ALIAS binding (`-1` null sentinel).
        r#"Kali.test('browser nullish assignment', () => { var table = { a: 1, b: 2 }; var last = null; for (var c in table) { last = c; } last ??= null; if (last) { return 1; } return 0; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
}

#[test]
fn test_supports_object_is_numeric_literals_in_browser_api_surface_with_harness_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        "Kali.test('browser object.is', () => { console.log(Object.is(-0, 0)); console.log(globalThis[\"Object\"][\"is\"](1, 1)); console.log(globalThis.Object[\"is\"](1, 1)); console.log(globalThis[\"Object\"].is(1, 1)); console.log(globalThis.Object.is(1, 1)); console.log(Object.is(1n, 1n)); console.log(Object.is(-1n, -1n)); });\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0\n1\n1\n1\n1\n1\n1"), "stdout: {stdout}");
}

#[test]
fn json_test_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_js_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("test", "js", true);
}

#[test]
fn json_test_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_ts_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("test", "ts", true);
}

#[test]
fn json_test_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("test", "jsx", true);
}

#[test]
fn json_test_supports_object_is_same_reference_alias_chain_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_object_is_same_reference_alias_chain_in_browser_harness("test", "tsx", true);
}

#[test]
fn test_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_test_supports_promise_all_settled_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn test_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn json_test_supports_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn test_supports_nullish_coalescing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('nullish', () => { const value = null ?? 1; return value; });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_nullish_coalescing_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('nullish', () => { const value = null ?? 1; return value; });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
}

#[test]
fn json_test_supports_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_math_inverse_hyperbolic_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("0").count() >= 3, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_inverse_hyperbolic_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn test_supports_math_hyperbolic_zero_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.sinh(0));\nconsole.log(Math.cosh(0));\nconsole.log(Math.tanh(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
    assert!(stdout.matches("1").count() >= 1, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_hyperbolic_zero_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.sinh(0));\nconsole.log(Math.cosh(0));\nconsole.log(Math.tanh(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn test_supports_math_inverse_trig_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.asin(0));\nconsole.log(Math.acos(1));\nconsole.log(Math.atan(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("0").count() >= 3, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_inverse_trig_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "console.log(Math.asin(0));\nconsole.log(Math.acos(1));\nconsole.log(Math.atan(0));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn test_supports_math_expm1_and_log1p_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_expm1_and_log1p_exact_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn test_supports_math_exp2_exact_identity_literals_through_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.exp2(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("1").count() >= 2, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_exp2_exact_identity_literals_through_const_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.exp2(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn test_supports_math_expm1_and_log1p_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.matches("0").count() >= 2, "stdout: {stdout}");
}

#[test]
fn json_test_supports_math_expm1_and_log1p_on_const_numeric_alias_chain_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["payload"]["passed"], 1);
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
fn json_test_supports_math_hypot_on_perfect_square_integer_literal_sums_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "console.log(Math.hypot(3, 4));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("5"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

#[test]
fn test_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('non-integer math', () => { const value = 1.6; const alias = value; console.log(Math.ceil(alias)); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_non_integer_numeric_literals_in_math_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('non-integer math', () => { const value = 1.6; const alias = value; console.log(Math.ceil(alias)); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
}

#[test]
fn test_supports_math_sqrt_member_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_sqrt_member_calls_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("json stdout")
            .contains("1.2649110640673518"),
        "json: {json}"
    );
}

#[test]
fn test_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("json stdout")
            .contains("1.2649110640673518"),
        "json: {json}"
    );
}

#[test]
fn test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_js_input_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("json stdout")
            .contains("1.2649110640673518"),
        "json: {json}"
    );
}

#[test]
fn test_supports_nullish_coalescing_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('browser nullish', () => { const value = null ?? 1; return value; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_nullish_coalescing_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('browser nullish', () => { const value = null ?? 1; return value; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
}

#[test]
fn test_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('browser nullish fallbacks', () => { const voidFallback = void 0 ?? 1; const undefinedFallback = undefined ?? 2; return voidFallback + undefinedFallback; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_supports_nullish_coalescing_with_void_and_undefined_fallbacks_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        r#"Kali.test('browser nullish fallbacks', () => { const voidFallback = void 0 ?? 1; const undefinedFallback = undefined ?? 2; return voidFallback + undefinedFallback; });
"#,
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn test_supports_for_of_array_iteration_in_browser_api_surface_with_harness_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser for-of', () => { for (const value of [1, 2]) { console.log(value); } });\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn test_supports_for_of_array_iteration_in_browser_api_surface_with_harness_ts_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser for-of', () => { for (const value of [1, 2]) { console.log(value); } });\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn test_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn test_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn test_supports_for_of_array_iteration_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser for-of', () => { for (const value of [1, 2]) { console.log(value); } });\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_of_array_iteration(&stdout);
}

#[test]
fn test_supports_for_of_array_iteration_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser for-of', () => { for (const value of [1, 2]) { console.log(value); } });\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert_browser_for_of_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn test_supports_set_and_map_constructor_iteration_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, set_and_map_iteration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_supports_set_and_map_constructor_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, set_and_map_iteration_test_source()).expect("write source");

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
fn test_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
    assert_browser_for_await_array_iteration_json(json["success"].as_bool().unwrap());
}

#[test]
fn test_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_browser_for_await_array_iteration(&stdout);
}

#[test]
fn test_supports_for_await_array_iteration_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "for await (const value of [1, 2]) { console.log(value); }\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn test_supports_for_await_array_iteration_in_browser_api_surface_with_harness_ts_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "for await (const value of [1, 2]) { console.log(value); }\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn test_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input_in_json(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            "for await (const value of await [1, 2]) { console.log(value); }\n",
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

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(0));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            errors.is_empty(),
            "errors array should be empty: {errors:?}"
        );
    }
}

#[test]
fn test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    assert_test_for_await_object_enumeration(&String::from_utf8_lossy(&output.stdout));
}

#[test]
fn json_test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
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
fn json_test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
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
fn json_test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
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
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.jsx");
    fs::write(
        &source_path,
        browser_spread_of_object_enumeration_in_for_await_array_iteration_source(),
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
fn json_test_supports_spread_of_object_enumeration_in_for_await_array_iteration_in_browser_api_surface_with_harness_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.jsx");
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
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.is_empty(),
        "errors array should be empty: {errors:?}"
    );
}

#[test]
fn test_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", false, "js", source);
    }
}

#[test]
fn test_rejects_class_generator_and_async_generator_method_lowering_in_ts_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", false, "ts", source);
    }
}

#[test]
fn test_rejects_class_generator_and_async_generator_method_lowering_in_jsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", false, "jsx", source);
    }
}

#[test]
fn test_rejects_class_generator_and_async_generator_method_lowering_in_tsx_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", false, "tsx", source);
    }
}

#[test]
fn json_test_rejects_class_generator_and_async_generator_method_lowering_in_js_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", true, "js", source);
    }
}

#[test]
fn json_test_rejects_class_generator_and_async_generator_method_lowering_in_ts_input() {
    for source in [
        "class Example { *main() { yield 1; } }\nnew Example();",
        "class Example { async *main() { yield 1; } }\nnew Example();",
    ] {
        assert_class_generator_method_lowering_rejection("test", true, "ts", source);
    }
}

#[test]
fn test_rejects_async_generator_lowering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_generator_delegating_yield_lowering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_generator_function_lowering() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn test_rejects_generator_function_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_generator_function_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_rejects_generator_function_lowering_in_ts_input() {
    assert_json_generator_function_lowering_rejection("test", "ts");
}

#[test]
fn test_rejects_generator_function_lowering_in_jsx_input() {
    assert_generator_function_lowering_rejection("test", "jsx");
}

#[test]
fn test_rejects_generator_function_lowering_in_ts_input() {
    assert_generator_function_lowering_rejection("test", "ts");
}

#[test]
fn test_rejects_generator_function_lowering_in_tsx_input() {
    assert_generator_function_lowering_rejection("test", "tsx");
}

#[test]
fn test_rejects_async_generator_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_async_generator_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_rejects_generator_delegating_yield_lowering_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_rejects_threaded_runtime_globals_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics'];",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_accepts_zero_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-threads")
        .arg("0")
        .arg(fixture_path("tests/smoke.test.ts"))
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
fn json_test_accepts_zero_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--max-threads")
        .arg("0")
        .arg(fixture_path("tests/smoke.test.ts"))
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
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
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
fn json_test_emits_a_command_envelope() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_supports_integer_like_key_ordering_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
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
        .arg("test")
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
fn json_test_supports_integer_like_key_ordering_semantics_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
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
        .arg("test")
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
fn test_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_dir = dir.path().join("tests");
    fs::create_dir_all(&source_dir).expect("create test dir");
    let test_path = source_dir.join("node.test.ts");
    fs::write(
        &test_path,
        r#"import 'node:path';
Kali.test('node', () => {
    console.log('node test ok');
});
"#,
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("node")
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
    assert!(stdout.contains("node test ok"), "stdout: {stdout}");
}

#[test]
fn test_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn test_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
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
fn test_accepts_browser_api_surface_when_a_browser_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser runtime smoke', () => { console.log('browser test'); });",
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("browser test"), "stdout: {stdout}");
}

#[test]
fn test_accepts_inherited_browser_api_surface_when_a_browser_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser runtime smoke', () => { console.log('browser test'); });",
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("browser test"), "stdout: {stdout}");
}

#[test]
fn test_accepts_supported_array_callback_slices_when_a_browser_harness_command_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        r#"Kali.test('browser runtime smoke', () => {
  for (const item of [1, 2, 3].map((value) => value)) {
    console.log("map:" + item);
  }
  for (const item of [1, 2].filter((value) => value)) {
    console.log("filter:" + item);
  }
  if ([0, 1].some((value) => value)) {
    console.log("some:true");
  } else {
    console.log("some:false");
  }
  if ([1, 0].every((value) => value)) {
    console.log("every:true");
  } else {
    console.log("every:false");
  }
  for (const item of [1, 2].flatMap((value) => [value])) {
    console.log("flatMap:" + item);
  }
});"#,
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_jsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.jsx");
    fs::write(
        &source_path,
        "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_in_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
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
fn test_accepts_zero_thread_and_spawn_budget_overrides_when_browser_harness_is_configured_without_json_output(
) {
    let dir = tempdir().expect("tempdir");
    for (filename, source) in [
        (
            "smoke.test.js",
            "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
        ),
        (
            "smoke.test.ts",
            "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
        ),
        (
            "smoke.test.jsx",
            "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
        ),
        (
            "smoke.test.tsx",
            "Kali.test('browser zero budgets', () => { console.log('browser zero budgets'); });\n",
        ),
    ] {
        let source_path = dir.path().join(filename);
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path())
            .arg("test")
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
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn test_accepts_positive_thread_budget_override_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    let dir = tempdir().expect("tempdir");
    for (filename, source) in [
        (
            "smoke.test.js",
            "Kali.test('browser threaded budgets', () => { console.log('browser threaded budgets'); });\n",
        ),
        (
            "smoke.test.ts",
            "Kali.test('browser threaded budgets', () => { console.log('browser threaded budgets'); });\n",
        ),
        (
            "smoke.test.jsx",
            "Kali.test('browser threaded budgets', () => { console.log('browser threaded budgets'); });\n",
        ),
        (
            "smoke.test.tsx",
            "Kali.test('browser threaded budgets', () => { console.log('browser threaded budgets'); });\n",
        ),
    ] {
        let source_path = dir.path().join(filename);
        fs::write(&source_path, source).expect("write source");

        let output = Command::new(kali_bin())
            .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("test")
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
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
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
fn test_accepts_browser_api_surface_with_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
fn test_accepts_inherited_browser_api_surface_with_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn test_accepts_browser_api_surface_with_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
fn test_accepts_inherited_browser_api_surface_with_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn test_accepts_browser_api_surface_with_integer_like_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
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
fn test_accepts_inherited_browser_api_surface_with_integer_like_object_enumeration_in_js_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn test_accepts_browser_api_surface_with_integer_like_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
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
fn test_accepts_inherited_browser_api_surface_with_integer_like_object_enumeration_in_ts_input_when_a_browser_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
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
fn json_test_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
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
fn test_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_browser_api_surface_with_missing_sandbox_policy_before_policy_loading_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let missing_policy_path = dir.path().join("missing.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&missing_policy_path)
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
fn json_test_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
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
fn test_rejects_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_rejects_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
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
fn json_test_rejects_inherited_browser_api_surface_with_sandbox_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
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
fn test_rejects_inherited_browser_api_surface_with_sandbox_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
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
fn json_test_rejects_inherited_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
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
fn test_rejects_inherited_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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
        .arg("test")
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
fn test_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_supported_permission_query_descriptor_const_bindings_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
fn json_test_with_sandbox_rejects_phase_three_deno_host_effects_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, phase_three_deno_host_effects_source()).expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
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
fn test_with_sandbox_allows_a_benign_test_program() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "Kali.test('sandbox ok', () => { 1 + 1; });").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "Kali.test(\"addition\", () => { 1 + 2; });").expect("write source");
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
        .arg("test")
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
fn json_test_accepts_positive_thread_budget_policy_when_threaded_profile_is_active() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    write_threaded_policy(&policy_path);

    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        let source_path = dir.path().join(filename);
        fs::write(
            &source_path,
            "Kali.test('thread policy', () => { console.log('thread policy'); });\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("test")
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
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("thread policy"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
    }
}

#[test]
fn json_test_supports_integer_like_object_enumeration_semantics_when_browser_harness_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn json_test_supports_integer_like_object_enumeration_semantics_when_browser_harness_is_configured_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        browser_runtime_integer_like_object_enumeration_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
fn test_supports_number_predicates_in_ts_and_js_input() {
    for extension in ["ts", "js"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            r#"Kali.test('number predicates', () => {
  const alias = 1;
  if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) {
    throw new Error('expected positive integer predicates');
  }
  if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) {
    throw new Error('expected negative primitive predicate cases');
  }
  if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) {
    throw new Error('expected bracketed Number predicate aliases');
  }
});
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("test")
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
    }
}

#[test]
fn json_test_supports_number_predicates_in_ts_and_js_input() {
    for extension in ["ts", "js"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            r#"Kali.test('number predicates', () => {
  const alias = 1;
  if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) {
    throw new Error('expected positive integer predicates');
  }
  if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) {
    throw new Error('expected negative primitive predicate cases');
  }
  if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) {
    throw new Error('expected bracketed Number predicate aliases');
  }
});
"#,
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("test")
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
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
    }
}
