use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_process_control_source() -> &'static str {
    "Deno.pid; globalThis.Deno.pid; globalThis[\"Deno\"][\"pid\"]; globalThis[\"Deno\"].cwd; globalThis[\"Deno\"].chdir; globalThis[\"Deno\"].exit; Deno[\"pid\"]; globalThis.Deno[\"pid\"]; globalThis.Deno.cwd; globalThis[\"Deno\"][\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno[\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno.chdir; globalThis.Deno.chdir; globalThis[\"Deno\"][\"chdir\"]; globalThis.Deno[\"chdir\"]; Deno[\"chdir\"]; globalThis.Deno[\"chdir\"]; globalThis.Deno.exit; globalThis[\"Deno\"][\"exit\"]; globalThis.Deno[\"exit\"]; Deno[\"exit\"]; globalThis.Deno[\"exit\"]; process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis[\"process\"].pid; process[\"pid\"]; globalThis.process[\"pid\"]; globalThis.process.cwd; globalThis[\"process\"].cwd; process.chdir; globalThis.process.chdir; process[\"cwd\"]; globalThis.process[\"cwd\"]; process[\"chdir\"]; globalThis.process[\"chdir\"]; process.exit; globalThis[\"process\"].chdir; globalThis[\"process\"].exit; globalThis[\"process\"][\"cwd\"]; globalThis[\"process\"][\"chdir\"]; globalThis[\"process\"][\"exit\"]; process[\"exit\"]; globalThis.process[\"exit\"];"
}

fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis[\"Deno\"].env.toObject();"
}

fn late_process_env_mutation_source() -> &'static str {
    "process.env = {}; process.env.KALI_BROWSER_ENV_MUTATION = {}; globalThis.process.env = {}; globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}; process[\"env\"] = {}; process[\"env\"].KALI_BROWSER_ENV_MUTATION = {}; process[\"env\"][\"KALI_BROWSER_ENV_MUTATION\"] = {}; globalThis.process[\"env\"] = {}; globalThis.process[\"env\"].KALI_BROWSER_ENV_MUTATION = {}; globalThis.process[\"env\"][\"KALI_BROWSER_ENV_MUTATION\"] = {}; globalThis[\"process\"].env = {}; globalThis[\"process\"].env.KALI_BROWSER_ENV_MUTATION = {}; globalThis[\"process\"][\"env\"] = {}; globalThis[\"process\"][\"env\"].KALI_BROWSER_ENV_MUTATION = {}; globalThis[\"process\"][\"env\"][\"KALI_BROWSER_ENV_MUTATION\"] = {}; delete process[\"env\"][\"KALI_BROWSER_ENV_MUTATION\"]; delete globalThis.process[\"env\"][\"KALI_BROWSER_ENV_MUTATION\"]; delete globalThis[\"process\"].env[\"KALI_BROWSER_ENV_MUTATION\"]; delete globalThis[\"process\"][\"env\"][\"KALI_BROWSER_ENV_MUTATION\"];"
}

fn late_env_mutation_source() -> &'static str {
    r#"Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno.env.delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"]["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"]["env"]["delete"]('KALI_ENV_DELETE_SMOKE');"#
}

fn late_permission_escalation_source() -> &'static str {
    r#"Deno.permissions.request(); Deno.permissions.revoke(); Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions.revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"]();"#
}

fn late_subprocess_source() -> &'static str {
    "new Deno.Command('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno[\"Command\"]('sh').spawn(); new globalThis[\"Deno\"].Command('sh').spawn(); new globalThis[\"Deno\"][\"Command\"]('sh').spawn();"
}

fn late_network_source() -> &'static str {
    "Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno[\"connect\"]('127.0.0.1', 1); globalThis[\"Deno\"].connect('127.0.0.1', 1); globalThis[\"Deno\"][\"connect\"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno[\"listen\"]('127.0.0.1', 0); globalThis[\"Deno\"].listen('127.0.0.1', 0); globalThis[\"Deno\"][\"listen\"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno[\"serve\"]('127.0.0.1', 0); globalThis[\"Deno\"].serve('127.0.0.1', 0); globalThis[\"Deno\"][\"serve\"]('127.0.0.1', 0);"
}

fn late_object_model_source() -> &'static str {
    "Intl; globalThis.Intl; globalThis[\"Intl\"]; globalThis.Intl.NumberFormat; globalThis.Intl.DateTimeFormat; globalThis.Intl.PluralRules; globalThis.Intl.RelativeTimeFormat; globalThis.Intl.Collator; globalThis.Intl.DisplayNames; globalThis.Intl.Segmenter; globalThis.Intl.Locale; globalThis[\"Intl\"][\"Segmenter\"]; globalThis[\"Intl\"][\"NumberFormat\"]; globalThis[\"Intl\"][\"DateTimeFormat\"]; globalThis[\"Intl\"][\"PluralRules\"]; globalThis[\"Intl\"][\"RelativeTimeFormat\"]; globalThis[\"Intl\"][\"Collator\"]; globalThis[\"Intl\"][\"DisplayNames\"]; globalThis[\"Intl\"][\"Locale\"]; Proxy; globalThis.Proxy; globalThis[\"Proxy\"]; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis[\"Proxy\"]({}, {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); globalThis[\"Proxy\"].revocable({}, {}); Object.hasOwn({}, \"a\"); globalThis.Object.hasOwn({}, \"a\"); globalThis[\"Object\"][\"hasOwn\"]({}, \"a\"); Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis.Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis.Object.prototype.hasOwnProperty[\"call\"]({}, \"a\"); globalThis.Object[\"prototype\"].hasOwnProperty.call({}, \"a\"); globalThis.Object[\"prototype\"][\"hasOwnProperty\"][\"call\"]({}, \"a\"); globalThis.Object.prototype[\"hasOwnProperty\"].call({}, \"a\"); globalThis[\"Object\"].prototype.hasOwnProperty.call({}, \"a\"); globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]({}, \"a\"); globalThis[\"Object\"].prototype[\"hasOwnProperty\"].call({}, \"a\"); globalThis[\"Object\"][\"prototype\"].hasOwnProperty.call({}, \"a\"); globalThis[\"Object\"][\"prototype\"][\"hasOwnProperty\"][\"call\"]({}, \"a\"); new WeakMap(); globalThis.WeakMap; globalThis[\"WeakMap\"](); new WeakSet(); globalThis.WeakSet; globalThis[\"WeakSet\"](); globalThis.WeakRef; globalThis[\"WeakRef\"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis[\"FinalizationRegistry\"](() => {});"
}

fn write_browser_api_surface_manifest(dir: &tempfile::TempDir) {
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

fn late_threaded_runtime_source() -> &'static str {
    "globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis.Atomics; globalThis[\"Atomics\"];"
}

fn non_literal_dynamic_import_source() -> &'static str {
    "let specifier; import(specifier);"
}

fn non_literal_dynamic_import_test_source() -> &'static str {
    "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n"
}

fn generator_function_source() -> &'static str {
    "function* main() { yield 1; }\nmain();"
}

fn async_generator_function_source() -> &'static str {
    "async function* main() { yield 1; }\nmain();"
}

#[test]
fn browser_late_threaded_runtime_source_includes_bracketed_forms() {
    let source = late_threaded_runtime_source();
    assert!(
        source.contains(r#"globalThis["SharedArrayBuffer"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Atomics"]"#),
        "source: {source}"
    );
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "Deno.pid",
        "globalThis.Deno.pid",
        r#"globalThis["Deno"].cwd"#,
        r#"globalThis["Deno"].chdir"#,
        r#"globalThis["Deno"].exit"#,
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        r#"globalThis["process"].pid"#,
        r#"globalThis["process"]["pid"]"#,
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        "globalThis.process.cwd",
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        "process.chdir",
        "globalThis.process.chdir",
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        "process.exit",
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
    ] {
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
    for expected in [
        "Deno.pid",
        "globalThis.Deno.pid",
        r#"globalThis["Deno"].cwd"#,
        r#"globalThis["Deno"].chdir"#,
        r#"globalThis["Deno"].exit"#,
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        r#"globalThis["process"].pid"#,
        r#"globalThis["process"]["pid"]"#,
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        "globalThis.process.cwd",
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        "process.chdir",
        "globalThis.process.chdir",
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        "process.exit",
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
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

fn assert_browser_late_env_materialization_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        r#"Deno["env"].toObject"#,
        r#"globalThis.Deno["env"].toObject"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis["Deno"].env.toObject"#,
        r#"globalThis["Deno"].env["toObject"]"#,
        r#"globalThis["Deno"]["env"].toObject"#,
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "environment snapshot materialization API",
        "object-aggregate lowering",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_env_materialization_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    for expected in [
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        r#"Deno["env"].toObject"#,
        r#"globalThis.Deno["env"].toObject"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis["Deno"].env.toObject"#,
        r#"globalThis["Deno"].env["toObject"]"#,
        r#"globalThis["Deno"]["env"].toObject"#,
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "environment snapshot materialization API",
        "object-aggregate lowering",
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

fn assert_browser_late_env_mutation_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Deno.env.set",
        "Deno.env.delete",
        "globalThis.Deno.env.set",
        "globalThis.Deno.env.delete",
        r#"Deno["env"]["set"]"#,
        r#"Deno["env"]["delete"]"#,
        r#"globalThis["Deno"]["env"]["set"]"#,
        r#"globalThis["Deno"]["env"]["delete"]"#,
        "environment mutation API",
        "browser API surface",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_env_mutation_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    for expected in [
        "Deno.env.set",
        "Deno.env.delete",
        "globalThis.Deno.env.set",
        "globalThis.Deno.env.delete",
        r#"Deno["env"]["set"]"#,
        r#"Deno["env"]["delete"]"#,
        r#"globalThis["Deno"]["env"]["set"]"#,
        r#"globalThis["Deno"]["env"]["delete"]"#,
        "environment mutation API",
        "browser API surface",
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

fn assert_browser_late_permission_escalation_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.matches("Deno.permissions.request").count() >= 2,
        "missing repeated request coverage in stderr: {stderr}"
    );
    assert!(
        stderr.matches("Deno.permissions.revoke").count() >= 2,
        "missing repeated revoke coverage in stderr: {stderr}"
    );
    for expected in [
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
        r#"globalThis["Deno"]["permissions"]["request"]"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]"#,
        "permission escalation API",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_permission_escalation_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains("Deno.permissions.request"))
            .count()
            >= 2,
        "missing repeated request coverage in {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains("Deno.permissions.revoke"))
            .count()
            >= 2,
        "missing repeated revoke coverage in {errors:?}"
    );
    for expected in [
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
        r#"globalThis["Deno"]["permissions"]["request"]"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]"#,
        "permission escalation API",
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

fn assert_browser_non_literal_dynamic_import_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

fn assert_browser_non_literal_dynamic_import_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("non-literal dynamic import()")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("statically known import specifier")),
        "missing non-literal dynamic import in {errors:?}"
    );
}

fn assert_browser_late_subprocess_rejection(stderr: &str) {
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(stderr.contains("Process.Spawn"), "stderr: {stderr}");
    assert_eq!(
        stderr.matches("Process.Spawn").count(),
        5,
        "stderr: {stderr}"
    );
}

fn assert_browser_late_subprocess_rejection_json(errors: &[Value]) {
    assert_eq!(errors.len(), 5, "errors array: {errors:?}");
    assert!(
        errors.iter().all(|error| error["code"] == "E9007"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().all(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("Process.Spawn")),
        "missing Process.Spawn in {errors:?}"
    );
}

fn assert_browser_late_network_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "socket/listener networking API",
        "Deno.connect",
        "globalThis.Deno.connect",
        "Deno.listen",
        "globalThis.Deno.listen",
        "Deno.serve",
        "globalThis.Deno.serve",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_network_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    for expected in [
        "socket/listener networking API",
        "Deno.connect",
        "globalThis.Deno.connect",
        "Deno.listen",
        "globalThis.Deno.listen",
        "Deno.serve",
        "globalThis.Deno.serve",
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

fn assert_browser_late_object_model_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.PluralRules",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
        r#"globalThis["Intl"]["DisplayNames"]"#,
        r#"globalThis["Intl"]["Segmenter"]"#,
        r#"globalThis["Intl"]["Locale"]"#,
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
        "globalThis.Intl.DateTimeFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.PluralRules",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
        r#"globalThis["Intl"]["DisplayNames"]"#,
        r#"globalThis["Intl"]["Segmenter"]"#,
        r#"globalThis["Intl"]["Locale"]"#,
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
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

fn assert_browser_late_threaded_runtime_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "SharedArrayBuffer",
        "globalThis.SharedArrayBuffer",
        "Atomics",
        "globalThis.Atomics",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_threaded_runtime_rejection_json(errors: &[Value]) {
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
        "SharedArrayBuffer",
        "globalThis.SharedArrayBuffer",
        "Atomics",
        "globalThis.Atomics",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

#[test]
fn browser_late_object_model_source_includes_bracketed_intl_forms() {
    let source = late_object_model_source();
    assert!(source.contains(r#"globalThis["Intl"]"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis["Intl"]["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["DateTimeFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["RelativeTimeFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["PluralRules"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["Collator"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["DisplayNames"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["Segmenter"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["Locale"]"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_object_model_source_includes_bracketed_proxy_and_finalization_forms() {
    let source = late_object_model_source();
    for expected in [
        r#"new Proxy({}, {})"#,
        r#"new globalThis.Proxy({}, {})"#,
        r#"new globalThis["Proxy"]({}, {})"#,
        r#"globalThis["Proxy"]"#,
        r#"Proxy.revocable"#,
        r#"globalThis.Proxy.revocable"#,
        r#"globalThis["Proxy"]["revocable"]"#,
        r#"globalThis["Proxy"].revocable"#,
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis.Object["prototype"].hasOwnProperty.call"#,
        r#"globalThis.Object.prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty["call"]"#,
        r#"globalThis["Object"].prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty.call"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_object_model_source_includes_mixed_bracketed_proxy_revocable_form() {
    let source = late_object_model_source();
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_process_control_source_includes_bracketed_forms() {
    let source = late_process_control_source();
    for expected in [
        r#"Deno.pid"#,
        r#"globalThis.Deno.pid"#,
        r#"globalThis["Deno"]["pid"]"#,
        r#"globalThis["Deno"].cwd"#,
        r#"globalThis["Deno"].chdir"#,
        r#"globalThis["Deno"].exit"#,
        r#"Deno["pid"]"#,
        r#"globalThis.Deno["pid"]"#,
        r#"globalThis.Deno.cwd"#,
        r#"globalThis.Deno.chdir"#,
        r#"globalThis.Deno.exit"#,
        r#"globalThis["Deno"]["cwd"]"#,
        r#"Deno["cwd"]"#,
        r#"globalThis.Deno["cwd"]"#,
        r#"globalThis["Deno"]["chdir"]"#,
        r#"Deno["chdir"]"#,
        r#"globalThis.Deno["chdir"]"#,
        r#"globalThis["Deno"]["exit"]"#,
        r#"Deno["exit"]"#,
        r#"globalThis.Deno["exit"]"#,
        r#"globalThis["process"].pid"#,
        r#"globalThis["process"]["pid"]"#,
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_subprocess_source_includes_bracketed_forms() {
    let source = late_subprocess_source();
    for expected in [
        r#"new Deno.Command('sh').spawn()"#,
        r#"new globalThis.Deno.Command('sh').spawn()"#,
        r#"new globalThis.Deno["Command"]('sh').spawn()"#,
        r#"new globalThis["Deno"].Command('sh').spawn()"#,
        r#"new globalThis["Deno"]["Command"]('sh').spawn()"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_network_source_includes_bracketed_forms() {
    let source = late_network_source();
    for expected in [
        r#"Deno.connect('127.0.0.1', 1)"#,
        r#"globalThis.Deno.connect('127.0.0.1', 1)"#,
        r#"globalThis.Deno["connect"]('127.0.0.1', 1)"#,
        r#"globalThis["Deno"].connect('127.0.0.1', 1)"#,
        r#"globalThis["Deno"]["connect"]('127.0.0.1', 1)"#,
        r#"Deno.listen('127.0.0.1', 0)"#,
        r#"globalThis.Deno.listen('127.0.0.1', 0)"#,
        r#"globalThis.Deno["listen"]('127.0.0.1', 0)"#,
        r#"globalThis["Deno"].listen('127.0.0.1', 0)"#,
        r#"globalThis["Deno"]["listen"]('127.0.0.1', 0)"#,
        r#"Deno.serve('127.0.0.1', 0)"#,
        r#"globalThis.Deno.serve('127.0.0.1', 0)"#,
        r#"globalThis.Deno["serve"]('127.0.0.1', 0)"#,
        r#"globalThis["Deno"].serve('127.0.0.1', 0)"#,
        r#"globalThis["Deno"]["serve"]('127.0.0.1', 0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_env_materialization_source_includes_bracketed_forms() {
    let source = late_env_materialization_source();
    for expected in [
        r#"Deno.env["toObject"]"#,
        r#"Deno["env"]["toObject"]"#,
        r#"Deno["env"].toObject"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis.Deno["env"].toObject"#,
        r#"globalThis["Deno"].env.toObject"#,
        r#"globalThis["Deno"].env["toObject"]"#,
        r#"globalThis["Deno"]["env"].toObject"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
        r#"globalThis["Deno"].env.toObject"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_process_env_mutation_source_includes_bracketed_forms() {
    let source = late_process_env_mutation_source();
    for expected in [
        r#"process.env"#,
        r#"process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"process.env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis.process.env"#,
        r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env"#,
        r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"]["env"]"#,
        r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis.process["env"]"#,
        r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_process_env_mutation_source_is_rejected_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_env_mutation_source()).expect("write source");
    fs::write(&test_path, late_process_env_mutation_source()).expect("write test source");

    for command in ["check", "build", "run", "test"] {
        for json_output in [false, true] {
            let mut command_line = Command::new(kali_bin());
            command_line.current_dir(dir.path());
            if json_output {
                command_line.arg("--output").arg("json");
            }
            if command == "run" || command == "test" {
                command_line.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
            }
            command_line.arg(command);
            if command == "build" {
                command_line.arg("--bundle");
            }
            command_line.arg("--api").arg("browser");
            command_line.arg(if command == "test" {
                &test_path
            } else {
                &source_path
            });

            let output = command_line.output().expect("run kali");
            assert!(
                !output.status.success(),
                "{command} should reject late browser process env mutation (json={json_output})\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.status.code(), Some(1));

            if json_output {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["success"], false);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(
                    errors.iter().any(|error| matches!(
                        error["code"].as_str(),
                        Some("E3100") | Some("E5506")
                    )),
                    "expected E3100 or E5506 in {errors:?}"
                );
                assert!(
                    errors.iter().any(|error| {
                        error["message"]
                            .as_str()
                            .expect("error message")
                            .contains("process")
                    }),
                    "missing process reference in {errors:?}"
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    stderr.contains("E3100") || stderr.contains("E5506"),
                    "stderr: {stderr}"
                );
                assert!(stderr.contains("process"), "stderr: {stderr}");
            }
        }
    }
}

#[test]
fn browser_late_permission_escalation_source_includes_bracketed_forms() {
    let source = late_permission_escalation_source();
    for expected in [
        r#"Deno.permissions.request()"#,
        r#"Deno.permissions.revoke()"#,
        r#"Deno.permissions["request"]()"#,
        r#"Deno.permissions["revoke"]()"#,
        r#"globalThis.Deno.permissions.request()"#,
        r#"globalThis.Deno.permissions.revoke()"#,
        r#"globalThis.Deno.permissions["request"]()"#,
        r#"globalThis.Deno.permissions["revoke"]()"#,
        r#"globalThis["Deno"].permissions.request()"#,
        r#"globalThis["Deno"].permissions.revoke()"#,
        r#"globalThis["Deno"].permissions["request"]()"#,
        r#"globalThis["Deno"].permissions["revoke"]()"#,
        r#"globalThis["Deno"]["permissions"].request()"#,
        r#"globalThis["Deno"]["permissions"].revoke()"#,
        r#"globalThis["Deno"]["permissions"]["request"]()"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
        r#"globalThis["Deno"]["permissions"].request()"#,
        r#"globalThis["Deno"]["permissions"].revoke()"#,
        r#"globalThis["Deno"].permissions["request"]()"#,
        r#"globalThis["Deno"].permissions["revoke"]()"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_env_mutation_source_includes_bracketed_forms() {
    let source = late_env_mutation_source();
    for expected in [
        r#"Deno["env"].set"#,
        r#"globalThis.Deno["env"].set"#,
        r#"globalThis.Deno["env"]["set"]"#,
        r#"globalThis["Deno"].env["set"]"#,
        r#"globalThis["Deno"]["env"].set"#,
        r#"globalThis["Deno"]["env"]["set"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_globalthis_deno_env_and_permission_source_includes_bracketed_forms() {
    let source = format!(
        "{} {} globalThis[\"Deno\"][\"permissions\"][\"request\"](); globalThis[\"Deno\"][\"permissions\"][\"revoke\"](); globalThis[\"Deno\"][\"permissions\"].request(); globalThis[\"Deno\"][\"permissions\"].revoke(); globalThis[\"Deno\"].env[\"toObject\"]; globalThis[\"Deno\"].env.toObject;",
        late_env_materialization_source(),
        late_permission_escalation_source()
    );
    for expected in [
        r#"globalThis.Deno.permissions["request"]()"#,
        r#"globalThis.Deno.permissions["revoke"]()"#,
        r#"globalThis["Deno"].permissions.request()"#,
        r#"globalThis["Deno"].permissions.revoke()"#,
        r#"globalThis["Deno"]["permissions"]["request"]()"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
        r#"globalThis["Deno"].permissions["request"]()"#,
        r#"globalThis["Deno"].permissions["revoke"]()"#,
        r#"globalThis["Deno"].env["toObject"]"#,
        r#"globalThis["Deno"].env.toObject"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn check_rejects_late_permission_escalation_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_permission_escalation_source()).expect("write source");

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
    assert_browser_late_permission_escalation_rejection(&stderr);
}

#[test]
fn build_rejects_late_permission_escalation_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_permission_escalation_source()).expect("write source");

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
    assert_browser_late_permission_escalation_rejection(&stderr);
}

#[test]
fn check_rejects_late_permission_escalation_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_permission_escalation_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_permission_escalation_rejection_json(errors);
}

#[test]
fn check_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

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
    assert_browser_non_literal_dynamic_import_rejection(&stderr);
}

#[test]
fn build_rejects_non_literal_dynamic_import_targets_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

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
    assert_browser_non_literal_dynamic_import_rejection(&stderr);
}

#[test]
fn check_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn build_rejects_non_literal_dynamic_import_targets_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn run_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_non_literal_dynamic_import_rejection(&stderr);
}

#[test]
fn run_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, non_literal_dynamic_import_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn test_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, non_literal_dynamic_import_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_non_literal_dynamic_import_rejection(&stderr);
}

#[test]
fn test_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, non_literal_dynamic_import_test_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_non_literal_dynamic_import_rejection_json(errors);
}

#[test]
fn run_and_test_reject_generator_function_lowering_in_browser_api_surface_js_input() {
    for (command, source_name) in [("run", "main.js"), ("test", "smoke.test.js")] {
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
fn check_rejects_late_subprocess_members_in_browser_api_surface_js_input_with_sandbox_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, late_subprocess_source()).expect("write source");
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
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_subprocess_rejection_json(errors);
}

#[test]
fn check_rejects_late_network_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_network_source()).expect("write source");

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
    assert_browser_late_network_rejection(&stderr);
}

#[test]
fn check_rejects_late_network_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_network_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_network_rejection_json(errors);
}

#[test]
fn build_rejects_late_permission_escalation_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_permission_escalation_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_permission_escalation_rejection_json(errors);
}

#[test]
fn build_rejects_late_subprocess_members_in_browser_bundle_js_input_with_sandbox_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, late_subprocess_source()).expect("write source");
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
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_subprocess_rejection_json(errors);
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn check_rejects_late_subprocess_members_in_browser_api_surface_js_input_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, late_subprocess_source()).expect("write source");
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
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
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
    assert_browser_late_subprocess_rejection(&stderr);
}

#[test]
fn build_rejects_late_network_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_network_source()).expect("write source");

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
    assert_browser_late_network_rejection(&stderr);
}

#[test]
fn build_rejects_late_network_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_network_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_network_rejection_json(errors);
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn build_rejects_late_subprocess_members_in_browser_bundle_js_input_with_sandbox() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, late_subprocess_source()).expect("write source");
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
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_subprocess_rejection(&stderr);
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn check_rejects_late_env_materialization_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn check_rejects_late_env_materialization_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn build_rejects_late_env_materialization_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn build_rejects_late_env_materialization_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn check_rejects_late_env_mutation_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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
    assert_browser_late_env_mutation_rejection(&stderr);
}

#[test]
fn check_rejects_late_env_mutation_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_mutation_rejection_json(errors);
}

#[test]
fn build_rejects_late_env_mutation_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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
    assert_browser_late_env_mutation_rejection(&stderr);
}

#[test]
fn build_rejects_late_env_mutation_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_mutation_rejection_json(errors);
}

#[test]
fn run_rejects_late_env_mutation_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_mutation_rejection(&stderr);
}

#[test]
fn run_rejects_late_env_mutation_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_mutation_rejection_json(errors);
}

#[test]
fn test_rejects_late_env_mutation_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_mutation_rejection(&stderr);
}

#[test]
fn test_rejects_late_env_mutation_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_mutation_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_mutation_rejection_json(errors);
}

#[test]
fn run_rejects_late_env_materialization_members_in_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn run_rejects_late_env_materialization_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn test_rejects_late_env_materialization_members_in_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn test_rejects_late_env_materialization_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn run_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn run_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn test_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn test_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn check_rejects_late_object_model_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn build_rejects_late_object_model_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn check_rejects_late_object_model_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
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
fn build_rejects_late_object_model_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
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
fn run_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn test_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn run_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_threaded_runtime_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_threaded_runtime_rejection(&stderr);
}

#[test]
fn test_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_threaded_runtime_rejection_json(errors);
}

fn assert_browser_late_threaded_runtime_rejection_for_command(
    command: &str,
    command_args: &[&str],
    with_browser_harness: bool,
    with_explicit_browser_api_surface: bool,
    with_browser_api_surface_manifest: bool,
    source_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");
    if with_browser_api_surface_manifest {
        write_browser_api_surface_manifest(&dir);
    }

    for json_output in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if with_browser_harness {
            output.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        }
        if json_output {
            output.arg("--output").arg("json");
        }
        output.arg(command);
        for arg in command_args {
            output.arg(arg);
        }
        if with_explicit_browser_api_surface {
            output.arg("--api").arg("browser");
        }
        output.arg(&source_path);

        let output = output.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_threaded_runtime_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_threaded_runtime_rejection(&stderr);
        }
    }
}

#[test]
fn check_rejects_threaded_runtime_globals_in_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        true,
        false,
        "main.js",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_browser_bundle_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        true,
        false,
        "main.js",
    );
}

#[test]
fn check_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        false,
        true,
        "main.js",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        false,
        true,
        "main.js",
    );
}

#[test]
fn check_rejects_threaded_runtime_globals_in_browser_api_surface_ts_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        true,
        false,
        "main.ts",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_browser_bundle_ts_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        true,
        false,
        "main.ts",
    );
}

#[test]
fn check_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_ts_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        false,
        true,
        "main.ts",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_ts_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        false,
        true,
        "main.ts",
    );
}

#[test]
fn run_rejects_threaded_runtime_globals_in_browser_api_surface_ts_input_with_browser_harness() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "run",
        &[],
        true,
        true,
        false,
        "main.ts",
    );
}

#[test]
fn test_rejects_threaded_runtime_globals_in_browser_api_surface_ts_input_with_browser_harness() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "test",
        &[],
        true,
        true,
        false,
        "smoke.test.ts",
    );
}

#[test]
fn run_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_ts_input_with_browser_harness(
) {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "run",
        &[],
        true,
        false,
        true,
        "main.ts",
    );
}

#[test]
fn test_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_ts_input_with_browser_harness(
) {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "test",
        &[],
        true,
        false,
        true,
        "smoke.test.ts",
    );
}

#[test]
fn check_rejects_fully_bracketed_promise_all_settled_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis['Promise']['allSettled']([1, 2]));\n",
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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn check_supports_promise_all_settled_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn build_supports_promise_all_settled_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn check_supports_promise_all_settled_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn build_supports_promise_all_settled_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

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

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn run_supports_promise_all_settled_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_supports_promise_all_settled_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

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
}

#[test]
fn test_supports_promise_all_settled_in_browser_api_surface_with_harness_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn test_supports_promise_all_settled_in_browser_api_surface_with_harness_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        "Kali.test('browser promise allSettled', () => { return Promise.allSettled([1, 2]); });\n",
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
fn check_supports_nullish_coalescing_in_browser_api_surface_js_input() {
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
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn build_supports_nullish_coalescing_in_browser_bundle_js_input() {
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
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_check_supports_nullish_coalescing_in_browser_api_surface_js_input() {
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
        .arg("--api")
        .arg("browser")
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
fn json_build_supports_nullish_coalescing_in_browser_bundle_js_input() {
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
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
}

fn late_eval_compatibility_source() -> &'static str {
    "eval('1 + 2'); new Function('return 3')();"
}

fn assert_browser_late_eval_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("compatibility feature 'eval'"),
        "stderr: {stderr}"
    );
}

fn assert_browser_late_eval_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("compatibility feature 'eval'")),
        "missing eval compatibility gate in {errors:?}"
    );
}

fn assert_browser_late_eval_rejection_for_command(
    command: &str,
    command_args: &[&str],
    with_browser_harness: bool,
    with_browser_api_surface_manifest: bool,
    source_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, late_eval_compatibility_source()).expect("write source");
    if with_browser_api_surface_manifest {
        write_browser_api_surface_manifest(&dir);
    }

    for json_output in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if with_browser_harness {
            output.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        }
        if json_output {
            output.arg("--output").arg("json");
        }
        output.arg(command);
        for arg in command_args {
            output.arg(arg);
        }
        output.arg("--api").arg("browser");
        output.arg(&source_path);

        let output = output.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_eval_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_eval_rejection(&stderr);
        }
    }
}

#[test]
fn check_rejects_eval_and_function_constructor_in_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("check", &[], false, false, "main.js");
}

#[test]
fn build_rejects_eval_and_function_constructor_in_browser_bundle_js_input() {
    assert_browser_late_eval_rejection_for_command("build", &["--bundle"], false, false, "main.js");
}

#[test]
fn run_rejects_eval_and_function_constructor_in_browser_api_surface_js_input_with_browser_harness()
{
    assert_browser_late_eval_rejection_for_command("run", &[], true, false, "main.js");
}

#[test]
fn test_rejects_eval_and_function_constructor_in_browser_api_surface_js_input_with_browser_harness()
{
    assert_browser_late_eval_rejection_for_command("test", &[], true, false, "smoke.test.js");
}

#[test]
fn check_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("check", &[], false, true, "main.js");
}

#[test]
fn build_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("build", &["--bundle"], false, true, "main.js");
}

#[test]
fn run_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    assert_browser_late_eval_rejection_for_command("run", &[], true, true, "main.js");
}

#[test]
fn test_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    assert_browser_late_eval_rejection_for_command("test", &[], true, true, "smoke.test.js");
}
