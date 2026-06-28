use super::*;

#[test]
fn package_effects_command_emits_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["package"]["registry"], "npm");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn package_effects_command_pretty_prints_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--pretty")
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("{\n"), "stdout: {stdout}");
    assert!(
        stdout.contains("\n  \"schemaVersion\": 1"),
        "stdout: {stdout}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["package"]["registry"], "npm");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_command_reports_inherited_browser_and_threaded_context_in_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browser-threaded-purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browser-threaded-purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "console.log('browser threaded package');",
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("browser-threaded-purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "browser-threaded-purepkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["package"]["registry"], "npm");
    assert_eq!(json["report"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(
        json["report"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(
        json["report"]["analysisContext"]["compatFeatures"],
        json!([])
    );
    assert_eq!(
        json["report"]["entryPoints"],
        json!(["browser-threaded-purepkg"])
    );
    assert!(!json["report"]["dynamicEffects"]
        .as_bool()
        .expect("dynamicEffects boolean"));
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn package_effects_command_supports_jsr_targets_in_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/@std/path");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "@std/path",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('jsr package');")
        .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("jsr:@std/path")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "@std/path");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["package"]["registry"], "jsr");
    assert_eq!(json["report"]["entryPoints"], json!(["jsr:@std/path"]));
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"), "effects: {kinds:?}");
}

#[test]
fn package_effects_reports_computed_deno_host_access() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"globalThis["Deno"]["env"]["set"]('KALI_CORPUS_FLAG', 'set');
"#,
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(
        json["report"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvWrite"), "effects: {kinds:?}");
}

#[test]
fn package_effects_reports_computed_bracketed_deno_env_delete_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"globalThis["Deno"]["env"]["delete"]('KALI_CORPUS_FLAG');
"#,
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(
        json["report"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvWrite"), "effects: {kinds:?}");
}

#[test]
fn package_effects_reports_computed_bracketed_deno_env_get_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"const direct = Deno["env"]["get"]('KALI_ENV_GET_SMOKE');
const bracketed = globalThis["Deno"]["env"]["get"]('KALI_ENV_GET_SMOKE');
const mixed = globalThis.Deno["env"]["get"]('KALI_ENV_GET_SMOKE');
const mixedDot = globalThis.Deno.env["get"]('KALI_ENV_GET_SMOKE');
const inherited = globalThis["Deno"].env["get"]('KALI_ENV_GET_SMOKE');
if (direct !== 'hello-environment' || bracketed !== 'hello-environment' || mixed !== 'hello-environment' || mixedDot !== 'hello-environment' || inherited !== 'hello-environment') {
  throw new Error('expected env get');
}
"#,
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_ENV_GET_SMOKE", "hello-environment")
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(
        json["report"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvRead"), "effects: {kinds:?}");
}

#[test]
fn package_effects_reports_direct_deno_network_calls_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"Deno.connect('127.0.0.1', 1);
Deno.listen('127.0.0.1', 0);
Deno.serve('127.0.0.1', 0);
"#,
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], false);
    assert_eq!(json["report"]["dynamicReasons"], json!([]));
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Connect"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Network.Listen"), "effects: {kinds:?}");
}

#[test]
fn package_effects_marks_computed_permissions_query_as_dynamic_but_effect_free() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"globalThis["Deno"]["permissions"].query({ name: "env" });
"#,
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(
        json["report"]["dynamicReasons"],
        json!(["computed-host-access"])
    );
    assert!(
        json["report"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn package_effects_treats_supported_permission_query_const_bindings_as_effect_free() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        supported_permission_query_const_binding_source(),
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["dynamicEffects"], false);
    assert_eq!(json["report"]["dynamicReasons"], json!([]));
    assert!(
        json["report"]["effects"]
            .as_array()
            .expect("effects array")
            .is_empty(),
        "unexpected effects: {json}"
    );
}

#[test]
fn package_effects_uses_inherited_browser_analysis_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["report"]["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_context_in_json_output(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');").expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "console.log('browser entry');",
    )
    .expect("write browser entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("package-effects")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let report = json
        .get("payload")
        .and_then(|value| value.get("report"))
        .or_else(|| json.get("report"))
        .expect("report object");
    assert_eq!(
        report["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["errors"], json!([]));
    assert_eq!(json["warnings"], json!([]));
}

#[test]
fn package_effects_rejects_inherited_wasm_threads_runtime_profile_in_browser_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');").expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "console.log('browser entry');",
    )
    .expect("write browser entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let report = json
        .get("payload")
        .and_then(|value| value.get("report"))
        .or_else(|| json.get("report"))
        .expect("report object");
    assert_eq!(
        report["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
}

#[test]
fn package_effects_preserves_browser_resolution_with_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(json["payload"]["package"]["name"], "browserpkg");
    assert_eq!(
        json["payload"]["report"]["entryPoints"],
        json!(["browserpkg"])
    );
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_preserves_browser_resolution_with_inherited_eval_context_and_top_level_sandbox_config_in_json_output(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  },
  "compat": {
    "features": ["eval"]
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\neval('1 + 2');\n",
    )
    .expect("write browser entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "browserpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(
        json["payload"]["report"]["entryPoints"],
        json!(["browserpkg"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_normalizes_inherited_eval_compatibility_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": [" eval "]
  }
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "eval('1 + 2');\n").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn package_effects_reports_inherited_node_analysis_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "process.env = {};\nconsole.log(process.argv.length);\nconsole.log('hello');",
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "node"
    );
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Process.EnvWrite"), "kinds: {kinds:?}");
}

#[test]
fn package_effects_reports_inherited_browser_threaded_analysis_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser",
    "runtimeProfiles": ["wasm-threads"]
  },
  "compat": {
    "features": ["eval"]
  }
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "eval('1 + 2');\n").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "browserpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "kinds: {kinds:?}");
}

#[test]
fn package_effects_ignores_inherited_node_context_and_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "console.log(process.argv.length);\nconsole.log('hello');",
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "node"
    );
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn package_effects_ignores_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn package_effects_command_emits_pretty_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--pretty")
        .arg("purepkg")
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
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_command_emits_pretty_json_payload_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--quiet")
        .arg("--pretty")
        .arg("purepkg")
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
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_rejects_missing_dependency_state() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E6004"), "stderr: {stderr}");
    assert!(
        stderr.contains("package 'purepkg' is not materialized in the current project"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_effects_command_emits_json_envelope() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(json["payload"]["package"]["version"], "1.0.0");
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn package_effects_command_emits_json_envelope_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_command_is_deterministic_across_repeated_pretty_invocations_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("package-effects")
            .arg("--quiet")
            .arg("--pretty")
            .arg("purepkg")
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
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_command_emits_json_envelope_under_quiet_inherited_browser_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "purepkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(json["payload"]["report"]["entryPoints"], json!(["purepkg"]));
}

#[test]
fn package_effects_command_emits_json_envelope_under_quiet_inherited_browser_and_threaded_context()
{
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browser-threaded-purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
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
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browser-threaded-purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "console.log('browser threaded package');",
    )
    .expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("browser-threaded-purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["package"]["name"],
        "browser-threaded-purepkg"
    );
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(
        json["payload"]["report"]["entryPoints"],
        json!(["browser-threaded-purepkg"])
    );
}

#[test]
fn package_effects_command_is_deterministic_across_repeated_json_envelope_invocations_under_inherited_browser_context(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("package-effects")
            .arg("--output")
            .arg("json")
            .arg("browserpkg")
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
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "browserpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(
        json["payload"]["report"]["entryPoints"],
        json!(["browserpkg"])
    );
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_command_is_deterministic_across_repeated_json_envelope_invocations_under_inherited_browser_context_and_top_level_sandbox_config(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("package-effects")
            .arg("--output")
            .arg("json")
            .arg("browserpkg")
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
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "browserpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    assert_eq!(
        json["payload"]["report"]["entryPoints"],
        json!(["browserpkg"])
    );
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_tracks_eval_compatibility_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "eval('1 + 2');\nconsole.log('package eval');\n",
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "evalpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn package_effects_normalizes_inherited_compat_features_in_json_output() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().expect("tempdir");

    let package_json = serde_json::json!({
        "name": "evalpkg",
        "version": "1.0.0",
        "main": "index.js",
    });
    let package_json_bytes =
        serde_json::to_vec_pretty(&package_json).expect("serialize package json");
    let tarball_bytes = build_package_tarball(&[
        ("package/package.json", package_json_bytes.as_slice()),
        (
            "package/index.js",
            b"eval('1 + 2');\nconsole.log('package eval');\n",
        ),
    ]);
    let tarball_integrity = format!("sha512-{}", format_sha512(&tarball_bytes));
    let (tarball_base, tarball_hits, tarball_stop, tarball_handle) =
        start_binary_response_server(tarball_bytes, "application/octet-stream");
    let metadata = serde_json::json!({
        "versions": {
            "1.0.0": {
                "dist": {
                    "tarball": format!("{}/evalpkg-1.0.0.tgz", tarball_base),
                    "integrity": tarball_integrity,
                }
            }
        }
    });
    let metadata = Box::leak(metadata.to_string().into_boxed_str());
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_registry_metadata_server(metadata);
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "dependencies": {
    "evalpkg": "1.0.0"
  },
  "compat": {
    "features": [" eval "]
  }
}"#,
    )
    .expect("write manifest");

    let install_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .output()
        .expect("run kali install");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().expect("join tarball server");
    registry_handle.join().expect("join registry server");

    assert!(
        install_output.status.success(),
        "install stderr: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );
    assert!(
        tarball_hits.load(Ordering::SeqCst) > 0,
        "tarball server should be queried"
    );
    assert!(
        registry_hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        dir.path()
            .join("node_modules/evalpkg/package.json")
            .exists(),
        "evalpkg should be materialized"
    );

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--output")
        .arg("json")
        .arg("evalpkg")
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
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "evalpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn package_effects_emits_json_envelope_under_quiet_eval_context() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "eval('1 + 2');\nconsole.log('package eval');\n",
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["errors"], json!([]));
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(json["payload"]["package"]["name"], "evalpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn package_effects_rejects_inherited_eval_and_wasm_threads_runtime_profile_under_quiet_json_output()
{
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "eval('1 + 2');\nconsole.log('package eval');\n",
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  },
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let report = json
        .get("payload")
        .and_then(|value| value.get("report"))
        .or_else(|| json.get("report"))
        .expect("report object");
    assert_eq!(
        report["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
}

#[test]
fn package_effects_reports_sorted_dynamic_reasons_for_combined_eval_and_computed_host_access() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        r#"globalThis["Deno"]["env"]["set"]('KALI_CORPUS_FLAG', 'set');
eval('console.log("package eval");');
"#,
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "evalpkg");
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(
        json["report"]["dynamicReasons"],
        json!(["computed-host-access", "eval"])
    );
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Process.EnvWrite"), "effects: {kinds:?}");
    assert!(kinds.contains(&"Eval"), "effects: {kinds:?}");
}

#[test]
fn package_effects_emits_pretty_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "eval('1 + 2');\nconsole.log('package eval');\n",
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("--pretty")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("{\n"), "stdout: {stdout}");
    assert!(stdout.contains("\n  \"package\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "evalpkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["report"]["entryPoints"], json!(["evalpkg"]));
    assert_eq!(
        json["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
}

#[test]
fn package_effects_keeps_native_json_payload_under_quiet() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "eval('1 + 2');\nconsole.log('package eval');\n",
    )
    .expect("write package entry");
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
        .arg("package-effects")
        .arg("--quiet")
        .arg("evalpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "evalpkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["report"]["entryPoints"], json!(["evalpkg"]));
    assert_eq!(
        json["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(json["report"]["dynamicReasons"], json!(["eval"]));
}

#[test]
fn package_effects_rejects_package_analysis_specific_flags() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/flagpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "flagpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let assert_rejection = |prepend_target: bool, args: &[&str]| {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        command.arg("package-effects");
        if prepend_target {
            command.arg("flagpkg").args(args);
        } else {
            command.args(args).arg("flagpkg");
        }

        let output = command.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("does not accept package-analysis-specific flags"),
            "stderr: {stderr}"
        );
    };

    for args in [
        &["--api", "browser"][..],
        &["--compat", "eval"][..],
        &["--wasm-threads"][..],
        &["--sandbox", "kali.policy.json"][..],
    ] {
        assert_rejection(false, args);
        assert_rejection(true, args);
    }
}

#[test]
fn package_effects_rejects_package_analysis_specific_flags_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/flagpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "flagpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&policy_path, "{\n  \"schemaVersion\": 1\n}\n").expect("write policy");
    let policy_path = policy_path.to_str().expect("policy path");

    for (args, expected_flag) in [
        (&["--api", "browser"][..], "--api"),
        (&["--compat", "eval"][..], "--compat"),
        (&["--wasm-threads"][..], "--wasm-threads"),
        (&["--sandbox", policy_path][..], "--sandbox"),
    ] {
        let assert_rejection = |prepend_target: bool| {
            let mut command = Command::new(kali_bin());
            command.current_dir(dir.path());
            command.arg("--output").arg("json");
            command.arg("package-effects");
            if prepend_target {
                command.arg("flagpkg").args(args);
            } else {
                command.args(args).arg("flagpkg");
            }

            let output = command.output().expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "package-effects");
            assert_eq!(json["success"], false);
            assert_eq!(json["exitCode"], 5);
            let errors = json["errors"].as_array().expect("errors array");
            assert!(!errors.is_empty(), "errors: {errors:?}");
            assert_eq!(errors[0]["code"], "E5508");
            assert_eq!(errors[0]["context"]["origin"], "cli");
            assert_eq!(errors[0]["context"]["flag"], expected_flag);
            if expected_flag == "--sandbox" {
                assert_eq!(errors[0]["context"]["requestedValue"], policy_path);
                assert_eq!(errors[0]["context"]["effectiveValue"], policy_path);
            }
            assert!(
                errors[0]["message"]
                    .as_str()
                    .expect("message string")
                    .contains("package-analysis-specific flags"),
                "json: {json}"
            );
        };

        assert_rejection(false);
        assert_rejection(true);
    }
}

#[test]
fn package_registry_analysis_commands_require_exactly_one_package_argument() {
    for (command, args, expected_message) in [
        (
            "package-effects",
            Vec::<&str>::new(),
            "requires exactly one package argument",
        ),
        (
            "package-effects",
            vec!["semver", "lodash"],
            "accepts exactly one package argument",
        ),
        (
            "package-audit",
            Vec::<&str>::new(),
            "requires exactly one package argument",
        ),
        (
            "package-audit",
            vec!["semver", "lodash"],
            "accepts exactly one package argument",
        ),
    ] {
        let output = Command::new(kali_bin())
            .arg(command)
            .args(args)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(stderr.contains(expected_message), "stderr: {stderr}");
    }
}

#[test]
fn package_registry_analysis_commands_reject_whitespace_package_argument() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg(command)
            .arg("   ")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("requires a non-empty package argument"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn package_registry_analysis_commands_reject_padded_package_argument() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg(command)
            .arg(" lodash ")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("without leading or trailing whitespace"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn json_package_registry_analysis_commands_require_exactly_one_package_argument() {
    for (command, args, expected_message) in [
        (
            "package-effects",
            Vec::<&str>::new(),
            "requires exactly one package argument",
        ),
        (
            "package-effects",
            vec!["semver", "lodash"],
            "accepts exactly one package argument",
        ),
        (
            "package-audit",
            Vec::<&str>::new(),
            "requires exactly one package argument",
        ),
        (
            "package-audit",
            vec!["semver", "lodash"],
            "accepts exactly one package argument",
        ),
    ] {
        let output = Command::new(kali_bin())
            .arg("--output")
            .arg("json")
            .arg(command)
            .args(args)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5508");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains(expected_message),
            "json: {json}"
        );
    }
}

#[test]
fn json_package_registry_analysis_commands_reject_whitespace_package_argument() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("   ")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5508");
        assert_eq!(errors[0]["context"]["origin"], "cli");
        assert_eq!(errors[0]["context"]["requestedValue"], "   ");
        assert_eq!(errors[0]["context"]["effectiveValue"], "");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("requires a non-empty package argument"),
            "json: {json}"
        );
    }
}

#[test]
fn json_package_registry_analysis_commands_reject_padded_package_argument() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg(" lodash ")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5508");
        assert_eq!(errors[0]["context"]["origin"], "cli");
        assert_eq!(errors[0]["context"]["requestedValue"], " lodash ");
        assert_eq!(errors[0]["context"]["effectiveValue"], "lodash");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("without leading or trailing whitespace"),
            "json: {json}"
        );
    }
}

#[test]
fn json_package_registry_analysis_commands_reject_extra_package_arguments() {
    for (command, json_flags) in [
        ("package-effects", vec!["--output", "json"]),
        ("package-effects", vec!["--pretty", "--output", "json"]),
        ("package-audit", vec!["--output", "json"]),
        ("package-audit", vec!["--pretty", "--output", "json"]),
    ] {
        let output = Command::new(kali_bin())
            .args(json_flags)
            .arg(command)
            .args(["semver", "lodash"])
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5508");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("accepts exactly one package argument"),
            "json: {json}"
        );
    }
}

#[test]
fn package_registry_commands_reject_explicit_package_versions() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg(command)
            .arg("semver@1.2.3")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("does not accept explicit package versions yet"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn package_registry_commands_reject_malformed_jsr_targets() {
    for (target, expected_message) in [
        ("jsr:", "requires a package name after `jsr:`"),
        ("jsr: foo", "without whitespace"),
        (
            "jsr:https://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:./local/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:../local/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:/absolute/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:npm:lodash",
            "accepts only registry package identifiers",
        ),
    ] {
        for command in ["package-effects", "package-audit"] {
            let output = Command::new(kali_bin())
                .arg(command)
                .arg(target)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("E5508"), "stderr: {stderr}");
            assert!(stderr.contains(expected_message), "stderr: {stderr}");
        }
    }
}

#[test]
fn json_package_registry_commands_reject_malformed_jsr_targets() {
    for (target, expected_message) in [
        ("jsr:", "requires a package name after `jsr:`"),
        ("jsr: foo", "without whitespace"),
        (
            "jsr:https://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:./local/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:../local/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:/absolute/pkg",
            "accepts only registry package identifiers",
        ),
        (
            "jsr:npm:lodash",
            "accepts only registry package identifiers",
        ),
    ] {
        for command in ["package-effects", "package-audit"] {
            let output = Command::new(kali_bin())
                .arg("--output")
                .arg("json")
                .arg(command)
                .arg(target)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            assert_eq!(json["exitCode"], 5);
            let errors = json["errors"].as_array().expect("errors array");
            assert!(!errors.is_empty(), "errors: {errors:?}");
            assert_eq!(errors[0]["code"], "E5508");
            assert!(
                errors[0]["message"]
                    .as_str()
                    .expect("message string")
                    .contains(expected_message),
                "json: {json}"
            );
        }
    }
}

#[test]
fn package_registry_commands_reject_non_registry_targets() {
    let cases = [
        (
            "https://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        (
            "http://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        ("./local/pkg", "accepts only registry package identifiers"),
        ("../local/pkg", "accepts only registry package identifiers"),
        ("/absolute/pkg", "accepts only registry package identifiers"),
        (
            "file:///tmp/pkg",
            "bare npm package names or `jsr:` identifiers",
        ),
        (
            "git+https://example.com/pkg.git",
            "bare npm package names or `jsr:` identifiers",
        ),
        ("npm:lodash", "bare npm package names or `jsr:` identifiers"),
    ];

    for command in ["package-effects", "package-audit"] {
        for (target, expected_message) in cases {
            let output = Command::new(kali_bin())
                .arg(command)
                .arg(target)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("E5508"), "stderr: {stderr}");
            assert!(stderr.contains(expected_message), "stderr: {stderr}");
            assert!(stderr.contains(target), "stderr: {stderr}");
        }
    }
}

#[test]
fn package_registry_commands_reject_explicit_package_versions_in_json_output() {
    for command in ["package-effects", "package-audit"] {
        let output = Command::new(kali_bin())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("semver@1.2.3")
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5508");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("explicit package versions yet"),
            "json: {json}"
        );
    }
}

#[test]
fn package_audit_rejects_pretty_without_output_json() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--pretty")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("--pretty") && stderr.contains("JSON output is active"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_registry_commands_reject_non_registry_targets_in_json_output() {
    let cases = [
        (
            "https://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        (
            "http://example.com/pkg.tgz",
            "accepts only registry package identifiers",
        ),
        ("./local/pkg", "accepts only registry package identifiers"),
        ("../local/pkg", "accepts only registry package identifiers"),
        ("/absolute/pkg", "accepts only registry package identifiers"),
        (
            "file:///tmp/pkg",
            "bare npm package names or `jsr:` identifiers",
        ),
        (
            "git+https://example.com/pkg.git",
            "bare npm package names or `jsr:` identifiers",
        ),
        ("npm:lodash", "bare npm package names or `jsr:` identifiers"),
    ];

    for command in ["package-effects", "package-audit"] {
        for (target, expected_message) in cases {
            let output = Command::new(kali_bin())
                .arg("--output")
                .arg("json")
                .arg(command)
                .arg(target)
                .output()
                .expect("run kali");

            assert!(!output.status.success());
            assert_eq!(output.status.code(), Some(5));
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            assert_eq!(json["exitCode"], 5);
            let errors = json["errors"].as_array().expect("errors array");
            assert!(!errors.is_empty(), "errors: {errors:?}");
            assert_eq!(errors[0]["code"], "E5508");
            assert!(
                errors[0]["message"]
                    .as_str()
                    .expect("message string")
                    .contains(expected_message),
                "json: {json}"
            );
            assert!(
                errors[0]["message"]
                    .as_str()
                    .expect("message string")
                    .contains(target),
                "json: {json}"
            );
        }
    }
}

#[test]
fn package_effects_rejects_missing_or_multiple_package_arguments() {
    let cases: [&[&str]; 2] = [&[], &["lodash", "react"]];

    for args in cases {
        let output = Command::new(kali_bin())
            .arg("package-effects")
            .args(args)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("exactly one package argument"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn package_effects_uses_browser_package_resolution_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");
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
        .arg("package-effects")
        .arg("browserpkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["report"]["analysisContext"]["apiSurface"], "browser");
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_emits_pretty_json_envelope_under_browser_resolution() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");
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
        .arg("package-effects")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("browserpkg")
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
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["apiSurface"],
        "browser"
    );
    let kinds = json["payload"]["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_emits_pretty_json_payload_without_output_json_under_browser_resolution() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/browserpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "browserpkg",
  "version": "1.0.0",
  "main": "main.js",
  "browser": "browser.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("main.js"), "console.log('main entry');\n")
        .expect("write main entry");
    fs::write(
        package_dir.join("browser.js"),
        "fetch('https://example.com/data');\n",
    )
    .expect("write browser entry");
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
        .arg("package-effects")
        .arg("--pretty")
        .arg("browserpkg")
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
    assert_eq!(json["package"]["name"], "browserpkg");
    assert_eq!(json["report"]["analysisContext"]["apiSurface"], "browser");
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Network.Fetch"), "effects: {kinds:?}");
    assert!(
        !kinds.contains(&"Console.Write"),
        "browser resolution should analyze the browser entrypoint, not the main entrypoint"
    );
}

#[test]
fn package_effects_command_is_deterministic_across_repeated_invocations() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "console.log('hello');\nfetch('https://example.com/data');\neval('1 + 2');\n",
    )
    .expect("write package entry");
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

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("package-effects")
            .arg("evalpkg")
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
    assert_eq!(json["package"]["name"], "evalpkg");
    assert_eq!(
        json["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["report"]["dynamicEffects"], true);
    assert_eq!(json["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["report"]["effects"]
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
fn package_effects_command_is_deterministic_across_repeated_json_envelope_invocations() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/evalpkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "evalpkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(
        package_dir.join("index.js"),
        "console.log('hello');\nfetch('https://example.com/data');\neval('1 + 2');\n",
    )
    .expect("write package entry");
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

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("package-effects")
            .arg("--output")
            .arg("json")
            .arg("evalpkg")
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
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["package"]["name"], "evalpkg");
    assert_eq!(
        json["payload"]["report"]["analysisContext"]["compatFeatures"],
        json!(["eval"])
    );
    assert_eq!(json["payload"]["report"]["dynamicEffects"], true);
    assert_eq!(json["payload"]["report"]["dynamicReasons"], json!(["eval"]));
    let kinds = json["payload"]["report"]["effects"]
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
fn package_effects_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
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
        .arg("package-effects")
        .arg("purepkg")
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
fn package_effects_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
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
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let report = json
        .get("payload")
        .and_then(|value| value.get("report"))
        .or_else(|| json.get("report"))
        .expect("report object");
    assert_eq!(
        report["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
}

#[test]
fn json_package_effects_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
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
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let report = json
        .get("payload")
        .and_then(|value| value.get("report"))
        .or_else(|| json.get("report"))
        .expect("report object");
    assert_eq!(
        report["analysisContext"]["runtimeProfiles"],
        json!(["wasm-threads"])
    );
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["errors"], json!([]));
    assert_eq!(json["warnings"], json!([]));
}

#[test]
fn package_audit_command_emits_envelope() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_preview_flag_is_rejected_before_registry_lookup() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--preview")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("--preview"), "stderr: {stderr}");
    assert!(stderr.contains("package-audit"), "stderr: {stderr}");
}

#[test]
fn package_audit_pretty_still_wins_over_preview_without_json() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("--pretty"), "stderr: {stderr}");
    assert!(stderr.contains("JSON output is active"), "stderr: {stderr}");
    assert!(!stderr.contains("--preview"), "stderr: {stderr}");
}

#[test]
fn package_audit_pretty_without_json_is_rejected_before_registry_lookup() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--pretty"), "stderr: {stderr}");
    assert!(stderr.contains("JSON output is active"), "stderr: {stderr}");
}

#[test]
fn package_audit_ignores_inherited_analysis_context() {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_ignores_inherited_browser_context() {
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_ignores_inherited_node_context() {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_ignores_top_level_sandbox_config() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_ignores_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_node_context_and_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "sandbox": "./missing.policy.json",
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_eval_context_in_json_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "compat": {
      "features": ["eval"]
    }
  }
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_browser_context_in_json_output() {
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_browser_and_thread_context_in_json_output() {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_browser_context_in_json_output_under_quiet() {
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_browser_context_and_top_level_sandbox_config_in_json_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  },
  "sandbox": "./missing.policy.json"
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_node_context_in_json_output() {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_ignores_inherited_compat_and_thread_context_in_json_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  },
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_rejects_pretty_without_json_output() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--pretty")
        .arg("lodash")
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

#[test]
fn package_audit_rejects_missing_or_multiple_package_arguments() {
    let cases: [&[&str]; 2] = [&[], &["lodash", "react"]];

    for args in cases {
        let output = Command::new(kali_bin())
            .arg("package-audit")
            .args(args)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("exactly one package argument"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn package_audit_rejects_missing_package_argument_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("message string")
            .contains("exactly one package argument"),
        "errors: {errors:?}"
    );
}

#[test]
fn package_audit_rejects_preview_compatibility_shim() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_before_package_analysis_flag_validation_in_text_output(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--api")
        .arg("browser")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "stderr: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_before_package_analysis_flag_validation_with_sandbox_in_text_output(
) {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("ignored.policy.json");
    fs::write(&policy_path, "{\n  \"schemaVersion\": 1\n}\n").expect("write policy");
    let policy_path = policy_path.to_str().expect("policy path");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--sandbox")
        .arg(policy_path)
        .arg("--api")
        .arg("browser")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("--api"), "stderr: {stderr}");
    assert!(!stderr.contains("--sandbox"), "stderr: {stderr}");
    assert!(
        output.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_in_json_output() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let error = json["errors"]
        .as_array()
        .expect("errors array")
        .first()
        .expect("error entry");
    assert_eq!(error["code"], "E5508");
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], "--preview");
    assert_eq!(error["context"]["requestedValue"], "true");
    assert_eq!(error["context"]["effectiveValue"], "true");
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_before_package_analysis_flag_validation_in_json_output(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("--api")
        .arg("browser")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let error = json["errors"]
        .as_array()
        .expect("errors array")
        .first()
        .expect("error entry");
    assert_eq!(error["code"], "E5508");
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], "--preview");
    assert_eq!(error["context"]["requestedValue"], "true");
    assert_eq!(error["context"]["effectiveValue"], "true");
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_before_package_analysis_flag_validation_with_sandbox_in_json_output(
) {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&policy_path, "{\n  \"schemaVersion\": 1\n}\n").expect("write policy");
    let policy_path = policy_path.to_str().expect("policy path");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--sandbox")
        .arg(policy_path)
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let error = json["errors"]
        .as_array()
        .expect("errors array")
        .first()
        .expect("error entry");
    assert_eq!(error["code"], "E5508");
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], "--preview");
    assert_eq!(error["context"]["requestedValue"], "true");
    assert_eq!(error["context"]["effectiveValue"], "true");
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_in_pretty_json_output() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let error = json["errors"]
        .as_array()
        .expect("errors array")
        .first()
        .expect("error entry");
    assert_eq!(error["code"], "E5508");
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], "--preview");
    assert_eq!(error["context"]["requestedValue"], "true");
    assert_eq!(error["context"]["effectiveValue"], "true");
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_before_malformed_target_validation_in_json_output(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("--preview")
        .arg("npm:lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let error = json["errors"]
        .as_array()
        .expect("errors array")
        .first()
        .expect("error entry");
    assert_eq!(error["code"], "E5508");
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], "--preview");
    assert_eq!(error["context"]["requestedValue"], "true");
    assert_eq!(error["context"]["effectiveValue"], "true");
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_preview_compatibility_shim_without_package_argument() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("--preview")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("message string")
            .contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "errors: {errors:?}"
    );
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
}

#[test]
fn package_audit_rejects_package_analysis_specific_flags() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&policy_path, "{\n  \"schemaVersion\": 1\n}\n").expect("write policy");
    let policy_path = policy_path.to_str().expect("policy path");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let assert_rejection = |prepend_target: bool, args: &[&str]| {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        command.env("KALI_REGISTRY", &registry_url);
        command.arg("package-audit");
        if prepend_target {
            command.arg("lodash").args(args);
        } else {
            command.args(args).arg("lodash");
        }

        let output = command.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5508"), "stderr: {stderr}");
        assert!(
            stderr.contains("does not accept package-analysis-specific flags"),
            "stderr: {stderr}"
        );
    };

    for args in [
        &["--api", "browser"][..],
        &["--compat", "eval"][..],
        &["--wasm-threads"][..],
        &["--sandbox", policy_path][..],
    ] {
        assert_rejection(false, args);
        assert_rejection(true, args);
    }

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
}

#[test]
fn package_audit_rejects_wasm_threads_flag() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--wasm-threads")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept package-analysis-specific flags"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_rejects_package_analysis_specific_flags_in_json_output() {
    let dir = tempdir().expect("tempdir");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&policy_path, "{\n  \"schemaVersion\": 1\n}\n").expect("write policy");
    let policy_path = policy_path.to_str().expect("policy path");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let assert_rejection = |prepend_target: bool, args: &[&str], expected_flag: &str| {
        let mut command = Command::new(kali_bin());
        command.current_dir(dir.path());
        command.env("KALI_REGISTRY", &registry_url);
        command.arg("--output").arg("json");
        command.arg("package-audit");
        if prepend_target {
            command.arg("lodash").args(args);
        } else {
            command.args(args).arg("lodash");
        }

        let output = command.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "package-audit");
        assert!(!json["success"].as_bool().expect("success boolean"));
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert_eq!(json["errors"][0]["context"]["origin"], "cli");
        assert_eq!(json["errors"][0]["context"]["flag"], expected_flag);
        if expected_flag == "--sandbox" {
            assert_eq!(json["errors"][0]["context"]["requestedValue"], policy_path);
            assert_eq!(json["errors"][0]["context"]["effectiveValue"], policy_path);
        }
        assert!(
            json["errors"][0]["message"]
                .as_str()
                .expect("error message")
                .contains("package-analysis-specific flags"),
            "json: {json}"
        );
    };

    for (args, expected_flag) in [
        (&["--api", "browser"][..], "--api"),
        (&["--compat", "eval"][..], "--compat"),
        (&["--wasm-threads"][..], "--wasm-threads"),
        (&["--sandbox", policy_path][..], "--sandbox"),
    ] {
        assert_rejection(false, args, expected_flag);
        assert_rejection(true, args, expected_flag);
    }

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
}

#[test]
fn package_audit_rejects_wasm_threads_runtime_profile_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--wasm-threads")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("package-analysis-specific flags"),
        "json: {json}"
    );
}

#[test]
fn package_audit_rejects_browser_api_surface_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--api")
        .arg("browser")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("package-analysis-specific flags"),
        "json: {json}"
    );
}

#[test]
fn package_audit_rejects_compat_feature_surface_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--compat")
        .arg("eval")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("package-analysis-specific flags"),
        "json: {json}"
    );
}

#[test]
fn package_audit_rejects_compat_feature_surface() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--compat")
        .arg("eval")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept package-analysis-specific flags"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_rejects_wasm_threads_runtime_profile() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--wasm-threads")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept package-analysis-specific flags"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_rejects_browser_api_surface() {
    let output = Command::new(kali_bin())
        .arg("package-audit")
        .arg("--api")
        .arg("browser")
        .arg("lodash")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept package-analysis-specific flags"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_rejects_node_api_surface() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--api")
        .arg("node")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept package-analysis-specific flags"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_command_emits_json_envelope() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_emits_pretty_json_envelope() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_emits_pretty_json_envelope_under_quiet() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_emits_pretty_json_envelope_under_inherited_browser_context() {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_suppresses_human_output_under_quiet() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn package_audit_suppresses_human_output_under_quiet_with_inherited_compat_and_thread_context() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  },
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn package_audit_command_emits_json_envelope_under_quiet_with_inherited_compat_and_thread_context()
{
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  },
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_emits_json_envelope_under_quiet() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_reports_findings() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(Some("echo ok"), false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("0 error(s), 1 warning(s)"));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
    assert_eq!(
        json["warnings"].as_array().expect("warnings array").len(),
        1
    );
    assert_eq!(json["warnings"][0]["code"], "W6006");
}

#[test]
fn package_audit_command_reports_lifecycle_scripts_in_order() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_lifecycle_scripts());

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
    let message = warnings[0]["message"].as_str().expect("warning message");
    assert!(
        message.contains("declares lifecycle scripts in preinstall, install, postinstall"),
        "warning message should keep lifecycle phases in deterministic order: {message}"
    );
}

#[test]
fn package_audit_command_reports_error_findings() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, true));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("1 error(s), 0 warning(s)"));
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E6005");
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_sorts_multiple_findings_deterministically() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));

    let errors = json["errors"].as_array().expect("errors array");
    let messages: Vec<_> = errors
        .iter()
        .map(|error| {
            error["message"]
                .as_str()
                .expect("error message")
                .to_string()
        })
        .collect();
    let mut sorted_messages = messages.clone();
    sorted_messages.sort();
    assert_eq!(
        messages, sorted_messages,
        "errors should be emitted in deterministic order"
    );
    assert_eq!(errors.len(), 4);
    assert_eq!(
        json["warnings"].as_array().expect("warnings array").len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_across_repeated_invocations() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("--output")
            .arg("json")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic"
    );

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic"
    );
    assert_eq!(first_json["schemaVersion"], 1);
    assert_eq!(first_json["command"], "package-audit");
    assert_eq!(first_json["success"], false);
    assert_eq!(first_json["payload"], serde_json::Value::Null);
    assert!(first_json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));
    assert_eq!(
        first_json["errors"].as_array().expect("errors array").len(),
        4
    );
    assert_eq!(
        first_json["warnings"]
            .as_array()
            .expect("warnings array")
            .len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_across_repeated_pretty_json_envelope_invocations_under_quiet(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("--quiet")
            .arg("--pretty")
            .arg("--output")
            .arg("json")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(first_json["schemaVersion"], 1);
    assert_eq!(first_json["command"], "package-audit");
    assert_eq!(first_json["success"], false);
    assert_eq!(first_json["payload"], serde_json::Value::Null);
    assert!(first_json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));
    assert_eq!(
        first_json["errors"].as_array().expect("errors array").len(),
        4
    );
    assert_eq!(
        first_json["warnings"]
            .as_array()
            .expect("warnings array")
            .len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_across_repeated_json_envelope_invocations_under_inherited_browser_context(
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("--output")
            .arg("json")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(first_json["schemaVersion"], 1);
    assert_eq!(first_json["command"], "package-audit");
    assert_eq!(first_json["success"], false);
    assert_eq!(first_json["payload"], serde_json::Value::Null);
    assert!(first_json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));
    assert_eq!(
        first_json["errors"].as_array().expect("errors array").len(),
        4
    );
    assert_eq!(
        first_json["warnings"]
            .as_array()
            .expect("warnings array")
            .len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_across_repeated_pretty_json_envelope_invocations_under_quiet_inherited_browser_context(
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("--quiet")
            .arg("--pretty")
            .arg("--output")
            .arg("json")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(first_json["schemaVersion"], 1);
    assert_eq!(first_json["command"], "package-audit");
    assert_eq!(first_json["success"], false);
    assert_eq!(first_json["payload"], serde_json::Value::Null);
    assert!(first_json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));
    assert_eq!(
        first_json["errors"].as_array().expect("errors array").len(),
        4
    );
    assert_eq!(
        first_json["warnings"]
            .as_array()
            .expect("warnings array")
            .len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_across_repeated_pretty_json_envelope_invocations_under_quiet_inherited_browser_context_and_top_level_sandbox_config(
) {
    let dir = tempdir().expect("tempdir");
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

    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("--quiet")
            .arg("--pretty")
            .arg("--output")
            .arg("json")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated invocations"
    );

    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(stdout.contains("\n  \"schemaVersion\""), "stdout: {stdout}");

    let first_json = parse_json_stdout(&first);
    let second_json = parse_json_stdout(&second);
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(first_json["schemaVersion"], 1);
    assert_eq!(first_json["command"], "package-audit");
    assert_eq!(first_json["success"], false);
    assert_eq!(first_json["payload"], serde_json::Value::Null);
    assert!(first_json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("4 error(s), 1 warning(s)"));
    assert_eq!(
        first_json["errors"].as_array().expect("errors array").len(),
        4
    );
    assert_eq!(
        first_json["warnings"]
            .as_array()
            .expect("warnings array")
            .len(),
        1
    );
}

#[test]
fn package_audit_command_is_deterministic_in_human_output() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let run = || {
        Command::new(kali_bin())
            .env("KALI_REGISTRY", &registry_url)
            .arg("package-audit")
            .arg("lodash")
            .output()
            .expect("run kali")
    };

    let first = run();
    let second = run();

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) >= 2,
        "registry server should be queried for each invocation"
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic"
    );

    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(
        stdout.contains("4 error(s), 1 warning(s)"),
        "stdout: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&first.stderr);
    let gypfile = stderr
        .find("declares gypfile=true")
        .expect("gypfile finding should be reported");
    let bin = stderr
        .find("native addon bin entrypoint")
        .expect("bin finding should be reported");
    let entrypoint = stderr
        .find("native addon entrypoint")
        .expect("entrypoint finding should be reported");
    let exports = stderr
        .find("native addon exports target")
        .expect("exports finding should be reported");
    let lifecycle = stderr
        .find("declares lifecycle scripts in postinstall")
        .expect("lifecycle warning should be reported");

    assert!(
        gypfile < bin && bin < entrypoint && entrypoint < exports && exports < lifecycle,
        "human-output findings should keep the deterministic severity/code/message order\nstderr: {stderr}"
    );
}

#[test]
fn package_audit_command_reports_findings_in_human_output() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_multiple_findings());

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        !output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("4 error(s), 1 warning(s)"),
        "stdout: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let gypfile = stderr
        .find("declares gypfile=true")
        .expect("gypfile finding should be reported");
    let bin = stderr
        .find("native addon bin entrypoint")
        .expect("bin finding should be reported");
    let entrypoint = stderr
        .find("native addon entrypoint")
        .expect("entrypoint finding should be reported");
    let exports = stderr
        .find("native addon exports target")
        .expect("exports finding should be reported");
    let lifecycle = stderr
        .find("declares lifecycle scripts in postinstall")
        .expect("lifecycle warning should be reported");

    assert!(
        gypfile < bin && bin < entrypoint && entrypoint < exports && exports < lifecycle,
        "human-output findings should keep the deterministic severity/code/message order\nstderr: {stderr}"
    );
}

#[test]
fn package_audit_command_selects_latest_stable_version_over_prerelease() {
    let (registry_url, hits, stop, handle) = start_registry_metadata_server(
        package_audit_metadata_body_with_stable_and_prerelease_versions(),
    );

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("lodash@1.0.0"));
    assert!(!json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("2.0.0-beta.1"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_prefers_the_highest_stable_version_over_newer_prereleases() {
    let (registry_url, hits, stop, handle) = start_registry_metadata_server(
        package_audit_metadata_body_with_multiple_stable_and_prerelease_versions(),
    );

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    let stdout = json["stdout"].as_str().expect("stdout string");
    assert!(stdout.contains("lodash@1.2.0"), "stdout: {stdout}");
    assert!(!stdout.contains("lodash@1.0.0"), "stdout: {stdout}");
    assert!(!stdout.contains("2.0.0-beta.1"), "stdout: {stdout}");
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_rejects_prerelease_only_versions() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body_with_prerelease_only_versions());

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(!output.status.success(), "stdout: {:?}", output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 1);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .any(|entry| entry["code"] == "E6001"));
    assert!(json["stdout"].is_null());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
    assert!(json["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .any(|entry| entry["message"]
            .as_str()
            .expect("error message")
            .contains("has no stable published version")));
}
