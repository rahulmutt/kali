use super::*;

#[test]
fn effects_reports_late_env_materialization_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.toObject; globalThis.Deno.env.toObject;\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvRead"), "effects: {kinds:?}");
}

#[test]
fn effects_reports_late_env_materialization_members_in_json_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno.env.toObject; globalThis.Deno.env.toObject;\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvRead"), "effects: {kinds:?}");
}

#[test]
fn effects_reports_bracketed_late_env_materialization_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "Deno[\"env\"][\"toObject\"]; globalThis[\"Deno\"][\"env\"][\"toObject\"];\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["dynamicReasons"], json!(["computed-host-access"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvRead"), "effects: {kinds:?}");
}

#[test]
fn effects_command_emits_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_emits_native_json_payload_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--quiet")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).trim().is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_emits_pretty_json_payload() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--pretty")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_emits_json_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_emits_pretty_json_envelope_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--quiet")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).trim().is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_is_deterministic_across_repeated_json_envelope_invocations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
eval("1 + 2");
"#,
    )
    .expect("write source");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--output")
            .arg("json")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_command_is_deterministic_across_repeated_default_invocations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
eval("1 + 2");
"#,
    )
    .expect("write source");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["eval"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_command_is_deterministic_across_repeated_default_invocations_under_inherited_browser_context(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
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

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_accepts_nullish_coalescing_in_default_analysis_context_in_js_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
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
        assert_eq!(json["dynamicEffects"], false);
        let kinds = json["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_accepts_nullish_coalescing_in_default_analysis_context_in_js_input_in_json_output() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--output")
            .arg("json")
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
        assert_eq!(json["command"], "effects");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["dynamicEffects"], false);
        let kinds = json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_command_reports_computed_deno_host_access() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
globalThis["Deno"]["env"]["set"]('KALI_CORPUS_FLAG', 'set');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(
        json["payload"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvWrite"), "effects: {kinds:?}");
}

#[test]
fn effects_command_reports_direct_deno_network_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
Deno.connect('127.0.0.1', 1);
Deno.listen('127.0.0.1', 0);
Deno.serve('127.0.0.1', 0);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Connect"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Listen"), "effects: {kinds:?}");
}

#[test]
fn effects_command_reports_computed_bracketed_deno_env_get_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
const direct = Deno["env"]["get"]('KALI_ENV_GET_SMOKE');
const bracketed = globalThis["Deno"]["env"]["get"]('KALI_ENV_GET_SMOKE');
const mixed = globalThis.Deno["env"]["get"]('KALI_ENV_GET_SMOKE');
const mixedDot = globalThis.Deno.env["get"]('KALI_ENV_GET_SMOKE');
const inherited = globalThis["Deno"].env["get"]('KALI_ENV_GET_SMOKE');
if (direct !== 'hello-environment' || bracketed !== 'hello-environment' || mixed !== 'hello-environment' || mixedDot !== 'hello-environment' || inherited !== 'hello-environment') {
  throw new Error('expected env get');
}
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(
        json["payload"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvRead"), "effects: {kinds:?}");
}

#[test]
fn effects_command_treats_permissions_query_as_effect_free() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
Deno.permissions.query({ name: "env" });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert!(
        json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn effects_command_marks_computed_permissions_query_as_dynamic_but_effect_free() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
Deno["permissions"]["query"]({ name: "env" });
Deno.permissions["query"]({ name: "env" });
globalThis.Deno.permissions.query({ name: "env" });
globalThis.Deno.permissions["query"]({ name: "env" });
globalThis["Deno"]["permissions"].query({ name: "env" });
globalThis["Deno"]["permissions"]["query"]({ name: "env" });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(
        json["payload"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    assert!(
        json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn effects_command_treats_permissions_query_subset_as_effect_free_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
Deno.permissions.query({ name: "read" });
Deno.permissions.query({ name: "write" });
Deno.permissions.query({ name: "env" });
Deno.permissions.query({ name: "net" });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert!(
        json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn effects_command_treats_supported_permission_query_const_bindings_as_effect_free_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert!(
        json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn effects_command_marks_computed_permissions_query_subset_as_dynamic_but_effect_free_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"
Deno["permissions"]["query"]({ name: "read" });
Deno.permissions["query"]({ name: "read" });
globalThis.Deno.permissions.query({ name: "read" });
globalThis.Deno.permissions["query"]({ name: "read" });
globalThis["Deno"]["permissions"].query({ name: "read" });
globalThis["Deno"]["permissions"]["query"]({ name: "read" });
Deno["permissions"]["query"]({ name: "write" });
Deno.permissions["query"]({ name: "write" });
globalThis.Deno.permissions.query({ name: "write" });
globalThis.Deno.permissions["query"]({ name: "write" });
globalThis["Deno"]["permissions"].query({ name: "write" });
globalThis["Deno"]["permissions"]["query"]({ name: "write" });
Deno["permissions"]["query"]({ name: "env" });
Deno.permissions["query"]({ name: "env" });
globalThis.Deno.permissions.query({ name: "env" });
globalThis.Deno.permissions["query"]({ name: "env" });
globalThis["Deno"]["permissions"].query({ name: "env" });
globalThis["Deno"]["permissions"]["query"]({ name: "env" });
Deno["permissions"]["query"]({ name: "net" });
Deno.permissions["query"]({ name: "net" });
globalThis.Deno.permissions.query({ name: "net" });
globalThis.Deno.permissions["query"]({ name: "net" });
globalThis["Deno"]["permissions"].query({ name: "net" });
globalThis["Deno"]["permissions"]["query"]({ name: "net" });
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(
        json["payload"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    assert!(
        json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn effects_rejects_sandbox_flag_as_invalid_usage() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('hello');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--sandbox")
        .arg("policy.json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn effects_rejects_sandbox_flag_as_invalid_usage_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('hello');\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
        .arg("--sandbox")
        .arg("policy.json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("does not accept `--sandbox`"),
        "json: {json}"
    );
}

#[test]
fn effects_command_marks_proxy_constructor_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "new Proxy({}, {});\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["proxy-traps"]));
    assert!(json["effects"]
        .as_array()
        .expect("effects array")
        .is_empty());
}

#[test]
fn effects_command_marks_proxy_revocable_calls_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy.revocable({}, {});\nglobalThis.Proxy.revocable({}, {});\nglobalThis[\"Proxy\"][\"revocable\"]({}, {});\nglobalThis[\"Proxy\"].revocable({}, {});\nglobalThis.Proxy[\"revocable\"]({}, {});\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["proxy-traps"]));
    assert!(json["effects"]
        .as_array()
        .expect("effects array")
        .is_empty());
}

#[test]
fn effects_command_tracks_eval_compatibility_as_an_effect() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"eval("1 + 2");"#).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["eval"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_command_tracks_function_constructor_compatibility_as_an_effect() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"new Function("return 1 + 2;")();"#).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["function-constructor"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_tracks_inherited_eval_compatibility_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"eval("1 + 2");"#).expect("write source");
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
        .arg("effects")
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
    assert_eq!(json["analysisContext"]["compatFeatures"], json!(["eval"]));
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["eval"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_normalizes_explicit_compat_features_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"eval("1 + 2");"#).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--compat")
        .arg(" eval ")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["compatFeatures"], json!(["eval"]));
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["eval"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_tracks_inherited_function_constructor_compatibility_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"new Function("return 1 + 2;")();"#).expect("write source");
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
        .arg("effects")
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
    assert_eq!(json["analysisContext"]["compatFeatures"], json!(["eval"]));
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["function-constructor"]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn effects_uses_explicit_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_uses_inherited_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
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
        .arg("effects")
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
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_uses_explicit_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_uses_inherited_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
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
        .arg("effects")
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
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_accepts_nullish_coalescing_in_browser_analysis_context_in_js_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
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
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["analysisContext"]["apiSurface"], "browser");
        assert_eq!(json["dynamicEffects"], false);
        let kinds = json["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_accepts_nullish_coalescing_in_browser_analysis_context_in_js_input_in_json_output() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--output")
            .arg("json")
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
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "effects");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
        assert_eq!(json["payload"]["dynamicEffects"], false);
        let kinds = json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_accepts_nullish_coalescing_in_inherited_browser_analysis_context_in_js_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
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
            .arg("effects")
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
        assert_eq!(json["analysisContext"]["apiSurface"], "browser");
        assert_eq!(json["dynamicEffects"], false);
        let kinds = json["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_accepts_nullish_coalescing_in_inherited_browser_analysis_context_in_js_input_in_json_output(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const value = null ?? 1;\nconsole.log(value);\n",
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
            .arg("effects")
            .arg("--output")
            .arg("json")
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
        assert_eq!(json["command"], "effects");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
        assert_eq!(json["payload"]["dynamicEffects"], false);
        let kinds = json["payload"]["effects"]
            .as_array()
            .expect("effects array")
            .iter()
            .map(|entry| entry["kind"].as_str().expect("kind string"))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    }
}

#[test]
fn effects_ignores_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json",
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_command_uses_explicit_browser_analysis_context() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_command_uses_explicit_browser_analysis_context_with_top_level_sandbox_config_in_json_output(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json",
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_command_uses_inherited_browser_analysis_context_with_top_level_sandbox_config_in_json_output(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json",
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_command_is_deterministic_across_repeated_json_envelope_invocations_under_inherited_browser_context(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
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

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--output")
            .arg("json")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
}

#[test]
fn effects_command_is_deterministic_across_repeated_json_envelope_invocations_under_inherited_browser_context_and_top_level_sandbox_config(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json",
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--output")
            .arg("json")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
}

#[test]
fn effects_command_is_deterministic_across_repeated_pretty_json_envelope_invocations_under_quiet_inherited_browser_context(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
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

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
            .arg("--quiet")
            .arg("--pretty")
            .arg("--output")
            .arg("json")
            .arg(&source_path)
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );
    assert!(
        String::from_utf8_lossy(&first.stdout).contains("\n  \"schemaVersion\""),
        "stdout: {}",
        String::from_utf8_lossy(&first.stdout)
    );

    let json = parse_json_stdout(&first);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["schemaVersion"], 1);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
}

#[test]
fn effects_reports_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(process.argv.length);\nconsole.log('node');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--api")
        .arg("node")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn effects_reports_node_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(process.argv.length);\nconsole.log('node');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--api")
        .arg("node")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn effects_reports_inherited_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(process.argv.length);\nconsole.log('node');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn effects_reports_inherited_node_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(process.argv.length);\nconsole.log('node');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["entryPoints"],
        json!([source_path.display().to_string()])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn effects_accepts_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("parse raw effects json");
    assert_eq!(
        json["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_accepts_inherited_node_api_surface_with_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node",
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["analysisContext"]["apiSurface"], "node");
    assert_eq!(
        json["payload"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_accepts_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("parse raw effects json");
    assert_eq!(
        json["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_accepts_inherited_wasm_threads_runtime_profile_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");
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
        .arg("effects")
        .arg("--output")
        .arg("json")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["payload"]["dynamicEffects"], false);
    assert_eq!(json["payload"]["dynamicReasons"], json!([]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context() {
    for source_name in ["main.ts", "main.jsx", "main.tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
        fs::write(
            &source_path,
            "console.log('ok');\nfetch('https://example.com');",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("effects")
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
}

#[test]
fn json_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context() {
    for source_name in ["main.ts", "main.jsx", "main.tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
        fs::write(
            &source_path,
            "console.log('ok');\nfetch('https://example.com');",
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("effects")
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
        assert_eq!(json["command"], "effects");
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("runtime profile")
                || errors[0]["message"]
                    .as_str()
                    .expect("error message")
                    .contains("wasm-threads"),
            "json: {json}"
        );
    }
}

#[test]
fn effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_ts_jsx_and_tsx_inputs(
) {
    for source_name in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context(
            source_name,
            true,
            false,
        );
    }
}

#[test]
fn json_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_ts_jsx_and_tsx_inputs(
) {
    for source_name in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context(
            source_name,
            true,
            true,
        );
    }
}

#[test]
fn effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_ts_jsx_and_tsx_inputs(
) {
    for source_name in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context(
            source_name,
            false,
            false,
        );
    }
}

#[test]
fn json_effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_ts_jsx_and_tsx_inputs(
) {
    for source_name in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context(
            source_name,
            false,
            true,
        );
    }
}

#[test]
fn effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
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
fn json_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("runtime profile")
            || errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("wasm-threads"),
        "json: {json}"
    );
}

#[test]
fn effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_input()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");
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
        .arg("effects")
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
fn effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_analysis_context_in_ts_jsx_and_tsx_inputs(
) {
    for source_name in ["main.ts", "main.jsx", "main.tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
        fs::write(&source_path, "console.log('ok');").expect("write source");
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
            .arg("effects")
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
}

#[test]
fn json_effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_analysis_context_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log('ok');").expect("write source");
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
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5506");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("runtime profile")
            || errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("wasm-threads"),
        "json: {json}"
    );
}

#[test]
fn effects_accepts_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");
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
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("parse raw effects json");
    assert_eq!(
        json["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_accepts_inherited_whitespace_padded_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": [" wasm-threads "]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("parse raw effects json");
    assert_eq!(
        json["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn json_effects_accepts_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn json_effects_accepts_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");
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
        .arg("effects")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn json_effects_normalizes_combined_inherited_analysis_context_axes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "eval('1 + 2');\nconsole.log('ok');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": [" eval "]
  },
  "compilerOptions": {
    "runtimeProfiles": [" wasm-threads "]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
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
    assert_eq!(json["command"], "effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(
        json["payload"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["payload"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn effects_rejects_duplicate_compat_features_in_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "eval('1 + 2');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval", "eval"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5509"), "stderr: {stderr}");
}
