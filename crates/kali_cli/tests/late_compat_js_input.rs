use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_js_compatibility_source() -> String {
    format!(
        "{}{} {}",
        "Intl; globalThis.Intl; globalThis[\"Intl\"]; globalThis.Intl.NumberFormat; globalThis[\"Intl\"].NumberFormat; globalThis.Intl[\"NumberFormat\"]; globalThis.Intl.DateTimeFormat; globalThis[\"Intl\"].DateTimeFormat; globalThis.Intl[\"DateTimeFormat\"]; globalThis.Intl.PluralRules; globalThis.Intl.RelativeTimeFormat; globalThis.Intl.Collator; globalThis.Intl.DisplayNames; globalThis.Intl.Segmenter; globalThis.Intl.Locale; globalThis[\"Intl\"][\"NumberFormat\"]; globalThis[\"Intl\"][\"DateTimeFormat\"]; globalThis[\"Intl\"][\"PluralRules\"]; globalThis[\"Intl\"][\"RelativeTimeFormat\"]; globalThis[\"Intl\"][\"Collator\"]; globalThis[\"Intl\"][\"DisplayNames\"]; globalThis[\"Intl\"][\"Segmenter\"]; globalThis[\"Intl\"][\"Locale\"]; Intl.NumberFormat; Intl.DateTimeFormat; Intl.PluralRules; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale; Deno.permissions[\"request\"](); Deno.permissions[\"revoke\"](); globalThis.Deno.permissions[\"request\"](); globalThis.Deno.permissions[\"revoke\"](); globalThis[\"Deno\"].permissions[\"request\"](); globalThis[\"Deno\"].permissions[\"revoke\"](); globalThis[\"Deno\"].permissions.request(); globalThis[\"Deno\"].permissions.revoke(); globalThis.Deno[\"permissions\"][\"request\"](); globalThis.Deno[\"permissions\"][\"revoke\"](); globalThis[\"Deno\"][\"permissions\"][\"request\"](); globalThis[\"Deno\"][\"permissions\"][\"revoke\"](); globalThis[\"Deno\"][\"permissions\"].request(); globalThis[\"Deno\"][\"permissions\"].revoke(); Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis.Deno[\"env\"][\"toObject\"](); ",
        kali_common::late_process_control_source(),
        "Proxy; globalThis.Proxy; globalThis[\"Proxy\"]; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis[\"Proxy\"]({}, {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); globalThis[\"Proxy\"].revocable({}, {}); globalThis.Proxy[\"revocable\"]({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze(globalThis[\"Proxy\"][\"revocable\"])({}, {}); Object.freeze(globalThis[\"Proxy\"].revocable)({}, {}); Object.freeze(globalThis.Proxy[\"revocable\"])({}, {}); Object.hasOwn(globalThis, \"a\"); globalThis.Object.hasOwn(globalThis, \"a\"); globalThis.Object[\"hasOwn\"](globalThis, \"a\"); globalThis[\"Object\"].hasOwn(globalThis, \"a\"); globalThis[\"Object\"][\"hasOwn\"](globalThis, \"a\"); Object[\"hasOwnProperty\"].call(globalThis, \"a\"); Object[\"hasOwnProperty\"][\"call\"](globalThis, \"a\"); globalThis.Object[\"hasOwnProperty\"].call(globalThis, \"a\"); globalThis[\"Object\"][\"hasOwnProperty\"].call(globalThis, \"a\"); globalThis[\"Object\"][\"hasOwnProperty\"][\"call\"](globalThis, \"a\"); Object.prototype.hasOwnProperty.call(globalThis, \"a\"); globalThis.Object.prototype.hasOwnProperty.call(globalThis, \"a\"); globalThis.Object.prototype.hasOwnProperty[\"call\"](globalThis, \"a\"); globalThis.Object[\"prototype\"].hasOwnProperty.call(globalThis, \"a\"); globalThis.Object[\"prototype\"][\"hasOwnProperty\"][\"call\"](globalThis, \"a\"); globalThis.Object.prototype[\"hasOwnProperty\"].call(globalThis, \"a\"); globalThis[\"Object\"].prototype.hasOwnProperty.call(globalThis, \"a\"); globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"](globalThis, \"a\"); globalThis[\"Object\"].prototype[\"hasOwnProperty\"].call(globalThis, \"a\"); globalThis[\"Object\"][\"prototype\"].hasOwnProperty.call(globalThis, \"a\"); globalThis[\"Object\"][\"prototype\"][\"hasOwnProperty\"][\"call\"](globalThis, \"a\"); new WeakMap(); globalThis.WeakMap; globalThis[\"WeakMap\"]; globalThis[\"WeakMap\"](); new WeakSet(); globalThis.WeakSet; globalThis[\"WeakSet\"]; globalThis[\"WeakSet\"](); globalThis.WeakRef; globalThis[\"WeakRef\"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis[\"FinalizationRegistry\"]; globalThis[\"FinalizationRegistry\"](() => {}); globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis.Atomics; globalThis[\"Atomics\"];",
    )
}

fn late_js_compatibility_source_with_mixed_process_forms() -> String {
    format!(
        "{} globalThis[\"process\"].pid; globalThis[\"process\"].cwd; globalThis[\"process\"].chdir; globalThis[\"process\"].exit;",
        late_js_compatibility_source()
    )
}
fn late_js_compatibility_source_without_object_has_own() -> String {
    late_js_compatibility_source().replace(
        &format!(
            "{} ",
            kali_common::late_compat_object_has_own_source("globalThis", r#""a""#)
        ),
        "",
    )
}

fn late_process_env_mutation_source() -> String {
    kali_common::late_process_env_mutation_source()
}

fn assert_late_js_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.matches("Deno.permissions.request").count() >= 2,
        "stderr: {stderr}"
    );
    assert!(
        stderr.matches("Deno.permissions.revoke").count() >= 2,
        "stderr: {stderr}"
    );
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
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
        "Intl.RelativeTimeFormat",
        "Intl.PluralRules",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
        "globalThis[\"Intl\"][\"DisplayNames\"]",
        "globalThis[\"Intl\"][\"Segmenter\"]",
        "globalThis[\"Intl\"][\"Locale\"]",
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        r#"globalThis["Deno"]["permissions"]["request"]"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]"#,
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
        "Proxy",
        "globalThis.Proxy",
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
        "SharedArrayBuffer",
        "Atomics",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_late_js_compatibility_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E5506") | Some("E3100"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E3100"),
        "expected at least one E3100 error: {errors:?}"
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
        "missing bracketed request coverage in {errors:?}"
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
        "missing bracketed revoke coverage in {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(r#"globalThis["Deno"]["permissions"]["request"]"#))
            .count()
            >= 2,
        "missing mixed-bracket request coverage in {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(r#"globalThis["Deno"]["permissions"]["revoke"]"#))
            .count()
            >= 2,
        "missing mixed-bracket revoke coverage in {errors:?}"
    );
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
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
        "Intl.RelativeTimeFormat",
        "Intl.PluralRules",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
        "globalThis[\"Intl\"][\"DisplayNames\"]",
        "globalThis[\"Intl\"][\"Segmenter\"]",
        "globalThis[\"Intl\"][\"Locale\"]",
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
        "Proxy",
        "globalThis.Proxy",
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
        "SharedArrayBuffer",
        "Atomics",
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
fn late_js_compatibility_source_includes_bracketed_intl_forms() {
    let source = late_js_compatibility_source_with_mixed_process_forms();
    assert!(source.contains(r#"globalThis["Intl"]"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis["Intl"]["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"].DateTimeFormat"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl["DateTimeFormat"]"#),
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
    assert!(source.contains(r#"globalThis["Intl"]["DisplayNames"]"#),);
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
fn late_js_compatibility_source_includes_bracketed_process_object_and_env_forms() {
    let source = late_js_compatibility_source();
    for expected in [
        r#"globalThis["process"]["pid"]"#,
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        r#"globalThis["process"].pid"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process.kill(0)"#,
        r#"process.kill(+0)"#,
        r#"process.kill((0))"#,
        r#"process["kill"](0)"#,
        r#"process["kill"](+0)"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"((process)).kill(0)"#,
        r#"((globalThis.process)).kill(0)"#,
        r#"globalThis.process.kill(0)"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"((process.kill))(0)"#,
        r#"((process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
        r#"new Proxy({}, {})"#,
        r#"new globalThis.Proxy({}, {})"#,
        r#"new globalThis["Proxy"]({}, {})"#,
        r#"globalThis["Proxy"]"#,
        r#"Proxy.revocable"#,
        r#"globalThis.Proxy.revocable"#,
        r#"globalThis["Proxy"]["revocable"]"#,
        r#"globalThis["Proxy"].revocable"#,
        r#"globalThis.Proxy["revocable"]"#,
        "globalThis.Object.hasOwn",
        r#"globalThis.Object["hasOwn"]"#,
        r#"globalThis["Object"].hasOwn"#,
        "globalThis.Object.prototype.hasOwnProperty.call",
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis.Object["prototype"].hasOwnProperty.call"#,
        r#"globalThis.Object.prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty["call"]"#,
        r#"globalThis["Object"].prototype["hasOwnProperty"].call"#,
        r#"globalThis.Object["prototype"].hasOwnProperty.call"#,
        r#"globalThis.Object.prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty["call"]"#,
        r#"globalThis["Object"].prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty.call"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty.call"#,
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_without_object_has_own_omits_shared_helper_block() {
    let source = late_js_compatibility_source_without_object_has_own();
    let helper_block = kali_common::late_compat_object_has_own_source("globalThis", r#""a""#);

    assert!(!source.contains(helper_block.as_str()), "source: {source}");
}

#[test]
fn late_js_compatibility_source_includes_frozen_process_zero_probe_alias() {
    let source = late_js_compatibility_source();
    assert!(
        source.contains(r#"Object.freeze(globalThis.process["kill"])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process.kill))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process.kill))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["process"].kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process)["kill"](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process)["kill"](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process)["kill"](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)["kill"])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)["kill"])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["process"])["kill"](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["process"])["kill"](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["process"]["kill"])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["process"]["kill"])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])).kill(+0)"#),
        "source: {source}"
    );
}

#[test]
fn late_js_compatibility_source_includes_mixed_bracketed_proxy_revocable_form() {
    let source = late_js_compatibility_source_with_mixed_process_forms();
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["Proxy"].revocable)"#),
        "source: {source}"
    );
}

#[test]
fn late_js_compatibility_source_includes_bracketed_process_env_mutation_forms() {
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
fn late_js_compatibility_source_includes_bracketed_permission_escalation_forms() {
    let source = late_js_compatibility_source_with_mixed_process_forms();
    for expected in [
        r#"Deno.permissions["request"]()"#,
        r#"Deno.permissions["revoke"]()"#,
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
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_includes_bracketed_globalthis_deno_env_and_permission_forms() {
    let source = format!(
        "{} globalThis[\"Deno\"].permissions[\"request\"](); globalThis[\"Deno\"].permissions[\"revoke\"](); globalThis.Deno.permissions[\"request\"](); globalThis.Deno.permissions[\"revoke\"](); globalThis[\"Deno\"][\"permissions\"].request(); globalThis[\"Deno\"][\"permissions\"].revoke(); globalThis[\"Deno\"].env[\"toObject\"]; globalThis[\"Deno\"].env.toObject;",
        late_js_compatibility_source_with_mixed_process_forms()
    );
    for expected in [
        r#"globalThis.Deno.permissions["request"]()"#,
        r#"globalThis.Deno.permissions["revoke"]()"#,
        r#"globalThis["Deno"].permissions.request()"#,
        r#"globalThis["Deno"].permissions.revoke()"#,
        r#"globalThis["Deno"]["permissions"]["request"]()"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
        r#"globalThis["Deno"]["permissions"].request()"#,
        r#"globalThis["Deno"]["permissions"].revoke()"#,
        r#"globalThis["Deno"]["permissions"].request()"#,
        r#"globalThis["Deno"]["permissions"].revoke()"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_includes_bracketed_threaded_runtime_forms() {
    let source = late_js_compatibility_source_with_mixed_process_forms();
    for expected in [
        r#"globalThis["SharedArrayBuffer"]"#,
        r#"globalThis["Atomics"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn check_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn check_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn run_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn build_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn build_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn run_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn test_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        late_js_compatibility_source_without_object_has_own(),
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
    assert_late_js_compatibility_rejection_json(errors);
}
