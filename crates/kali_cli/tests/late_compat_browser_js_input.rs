use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_process_control_source() -> String {
    kali_common::late_process_control_source()
}

fn late_env_materialization_source() -> &'static str {
    kali_common::late_env_materialization_source()
}

fn late_process_env_mutation_source() -> String {
    kali_common::late_process_env_mutation_source()
}

fn late_env_mutation_source() -> &'static str {
    r#"Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno.env.set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno.env.delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis.Deno["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"].set('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"]["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello-environment'); globalThis["Deno"]["env"]["delete"]('KALI_ENV_DELETE_SMOKE');"#
}

fn late_permission_escalation_source() -> String {
    kali_common::late_permission_escalation_source()
}

fn late_subprocess_source() -> &'static str {
    kali_common::late_subprocess_source()
}

fn late_network_source() -> &'static str {
    kali_common::late_network_source()
}

fn late_object_model_source() -> String {
    format!(
        "{} {} {}",
        kali_common::broader_intl_source(),
        kali_common::late_object_model_source(),
        kali_common::late_compat_object_has_own_source("{}", r#""a""#)
    )
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
    kali_common::late_threaded_runtime_source()
}

fn non_literal_dynamic_import_source() -> &'static str {
    "let specifier; import(specifier);"
}

fn non_literal_dynamic_import_test_source() -> &'static str {
    "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n"
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

fn sequence_wrapped_generator_class_expression_source() -> &'static str {
    "const Example = (0, class NamedExample { *main() { yield* []; } });\nnew Example();\n"
}

fn sequence_wrapped_async_generator_class_expression_source() -> &'static str {
    "const Example = (0, class NamedExample { async *main() { yield* []; } });\nnew Example();\n"
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
        r#"globalThis["process"]["pid"]"#,
        "globalThis.process.cwd",
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"]["cwd"]"#,
        "process.chdir",
        "globalThis.process.chdir",
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"]["chdir"]"#,
        "process.kill",
        "globalThis.process.kill",
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"]["kill"]"#,
        "process.exit",
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
        r#"globalThis["process"]["exit"]"#,
        "undefined identifier 'process'",
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
        r#"globalThis["process"]["pid"]"#,
        "globalThis.process.cwd",
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"]["cwd"]"#,
        "process.chdir",
        "globalThis.process.chdir",
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"]["chdir"]"#,
        "process.kill",
        "globalThis.process.kill",
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"]["kill"]"#,
        "process.exit",
        r#"globalThis["process"].exit"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
        r#"globalThis["process"]["exit"]"#,
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
        r#"globalThis["Deno"]["env"]["toObject"]"#,
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
        r#"globalThis["Deno"]["env"]["toObject"]"#,
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
        6,
        "stderr: {stderr}"
    );
}

fn assert_browser_late_subprocess_rejection_json(errors: &[Value]) {
    assert_eq!(errors.len(), 6, "errors array: {errors:?}");
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
        r#"globalThis["Intl"]["NumberFormat"]"#,
        r#"globalThis['Intl']['NumberFormat']"#,
        "globalThis.Intl.DateTimeFormat",
        r#"globalThis['Intl']['DateTimeFormat']"#,
        "globalThis.Intl.RelativeTimeFormat",
        r#"globalThis['Intl']['RelativeTimeFormat']"#,
        "globalThis.Intl.PluralRules",
        r#"globalThis['Intl']['PluralRules']"#,
        "globalThis.Intl.Collator",
        r#"globalThis['Intl']['Collator']"#,
        "globalThis.Intl.DisplayNames",
        r#"globalThis['Intl']['DisplayNames']"#,
        "globalThis.Intl.Segmenter",
        r#"globalThis['Intl']['Segmenter']"#,
        "globalThis.Intl.Locale",
        r#"globalThis['Intl']['Locale']"#,
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
        r#"globalThis["Intl"]["NumberFormat"]"#,
        r#"globalThis['Intl']['NumberFormat']"#,
        "globalThis.Intl.DateTimeFormat",
        r#"globalThis['Intl']['DateTimeFormat']"#,
        "globalThis.Intl.RelativeTimeFormat",
        r#"globalThis['Intl']['RelativeTimeFormat']"#,
        "globalThis.Intl.PluralRules",
        r#"globalThis['Intl']['PluralRules']"#,
        "globalThis.Intl.Collator",
        r#"globalThis['Intl']['Collator']"#,
        "globalThis.Intl.DisplayNames",
        r#"globalThis['Intl']['DisplayNames']"#,
        "globalThis.Intl.Segmenter",
        r#"globalThis['Intl']['Segmenter']"#,
        "globalThis.Intl.Locale",
        r#"globalThis['Intl']['Locale']"#,
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

// fasta Spec 7 Task 3: scalar `??=` rejects fail-closed (null and 0 are
// indistinguishable for a scalar), so the `??=` pipeline coverage rides the one
// surviving lowering — a for-in-key ALIAS binding (`-1` null sentinel).
fn nullish_assignment_source() -> &'static str {
    "var table = { a: 1, b: 2 };\nvar last = null;\nfor (var c in table) {\n  last = c;\n}\nlast ??= null;\nif (last) { console.log(\"set\"); }\n"
}

fn late_eval_compatibility_source() -> &'static str {
    "eval('1 + 2'); new Function('return 3')();"
}

fn late_eval_compatibility_alias_source() -> &'static str {
    "globalThis.Function('return 3')(); globalThis[\"Function\"]('return 4')(); globalThis['Function']('return 5')(); new globalThis.Function('return 6')(); new globalThis[\"Function\"]('return 7')(); new globalThis['Function']('return 8')();"
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
    source: &str,
    source_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, source).expect("write source");
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

#[path = "late_compat_browser_js_input/run.rs"]
mod run;

#[path = "late_compat_browser_js_input/build.rs"]
mod build;

#[path = "late_compat_browser_js_input/check.rs"]
mod check;

#[path = "late_compat_browser_js_input/test.rs"]
mod test;

#[path = "late_compat_browser_js_input/misc.rs"]
mod misc;
