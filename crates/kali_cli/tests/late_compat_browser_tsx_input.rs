use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_browser_tsx_compatibility_source() -> String {
    format!(
        "{} {}",
        r#"Intl; Object.freeze(globalThis.Intl.NumberFormat); Object.freeze((null ?? globalThis.Intl.NumberFormat)); globalThis.Intl.NumberFormat; globalThis["Intl"].NumberFormat; globalThis['Intl']; globalThis.Intl["NumberFormat"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; globalThis["Intl"].DateTimeFormat; globalThis['Intl']['DateTimeFormat']; globalThis.Intl["DateTimeFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis.Intl.RelativeTimeFormat; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"].RelativeTimeFormat; globalThis.Intl["RelativeTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis.Intl.Collator; globalThis['Intl']['Collator']; globalThis["Intl"].Collator; globalThis.Intl["Collator"]; globalThis["Intl"]["Collator"]; globalThis.Intl.DisplayNames; globalThis['Intl']['DisplayNames']; globalThis["Intl"].DisplayNames; globalThis.Intl["DisplayNames"]; globalThis["Intl"]["DisplayNames"]; globalThis.Intl.Segmenter; globalThis['Intl']['Segmenter']; globalThis["Intl"].Segmenter; globalThis.Intl["Segmenter"]; globalThis["Intl"]["Segmenter"]; globalThis.Intl.Locale; globalThis['Intl']['Locale']; globalThis["Intl"].Locale; globalThis.Intl["Locale"]; globalThis["Intl"]["Locale"]; globalThis.Intl.PluralRules; globalThis['Intl']['PluralRules']; globalThis["Intl"].PluralRules; globalThis.Intl["PluralRules"]; globalThis["Intl"]["PluralRules"]; Deno.permissions.request(); Deno.permissions.revoke(); Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); Deno.env.toObject(); Deno["env"]["toObject"](); globalThis["Deno"]["env"]["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis["Deno"]["env"].toObject(); Deno.env.set('KALI_ENV_SET_SMOKE', 'hello'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"]["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"]["env"]["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"]["env"]["delete"]('KALI_ENV_DELETE_SMOKE'); globalThis.Deno["env"].set('KALI_ENV_SET_SMOKE', 'hello'); globalThis.Deno["env"].delete('KALI_ENV_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_SMOKE', 'hello'); globalThis["Deno"].env["delete"]('KALI_ENV_DELETE_SMOKE'); delete process.env["KALI_BROWSER_ENV_MUTATION"]; delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]; process.pid; globalThis.process.pid; globalThis["process"].pid; globalThis["process"]["pid"]; process["pid"]; globalThis.process["pid"]; process.cwd; globalThis.process.cwd; globalThis["process"].cwd; globalThis["process"]["cwd"]; process["cwd"]; globalThis.process["cwd"]; process.chdir; globalThis.process.chdir; globalThis["process"].chdir; globalThis["process"]["chdir"]; process["chdir"]; globalThis.process["chdir"]; process.kill; globalThis.process.kill; globalThis["process"].kill; globalThis["process"]["kill"]; process["kill"]; globalThis.process["kill"]; const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); process.kill(0); process.kill(+0); process["kill"](+0); process.kill((0)); ((process)).kill(0); ((globalThis.process)).kill(0); globalThis.process.kill(0); globalThis.process["kill"](+0); globalThis["process"]["kill"](+0); globalThis["process"]["kill"]((0)); globalThis["process"].kill(0); globalThis["process"].kill(+0); globalThis["process"]["kill"](0); globalThis.process["kill"](0); Object.freeze(globalThis.process["kill"])(0); Object.freeze(globalThis.process["kill"])(+0); Object.freeze(process)["kill"](0); Object.freeze((process)["kill"])(0); Object.freeze((process)["kill"])(+0); Object.freeze((process["kill"]))(0); Object.freeze((process["kill"]))(+0); Object.freeze((process).kill)(0); Object.freeze((process).kill)(+0); Object.freeze(globalThis.process.kill)(0); Object.freeze(globalThis.process.kill)(+0); Object.freeze((globalThis.process["kill"]))(0); Object.freeze((globalThis.process["kill"]))(+0); Object.freeze((process.kill))(0); Object.freeze((process.kill))(+0); Object.freeze((globalThis["process"]["kill"]))(0); Object.freeze((globalThis["process"]["kill"]))(+0); Object.freeze(globalThis["process"].kill)(0); Object.freeze(globalThis["process"].kill)(+0); Object.freeze(globalThis.process)["kill"](0); Object.freeze(globalThis.process)["kill"](+0); Object.freeze(globalThis["process"])["kill"](0); Object.freeze(globalThis["process"])["kill"](+0); Object.freeze((globalThis["process"].kill))(0); Object.freeze((globalThis["process"].kill))(+0); Object.freeze(globalThis["process"]["kill"])(0); Object.freeze(globalThis["process"]["kill"])(+0); Object.freeze(process.kill)(0); Object.freeze(process.kill)(+0); ((process.kill))(0); ((process["kill"]))(0); ((process["kill"]))(+0); ((globalThis.process.kill))(0); ((globalThis["process"].kill))(0); ((globalThis["process"]["kill"]))(0); process.exit; globalThis.process.exit; globalThis["process"].exit; globalThis["process"]["exit"]; process["exit"]; globalThis.process["exit"]; globalThis['Proxy']; new globalThis['Proxy']({}, {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis['Proxy']['revocable']({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis['Proxy'].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); globalThis.Proxy['revocable']({}, {}); globalThis['Proxy']["revocable"]({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze((globalThis["Proxy"])["revocable"])({}, {}); Object.freeze(globalThis['Proxy']['revocable'])({}, {}); Object.freeze((globalThis['Proxy']['revocable']))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze((globalThis["Proxy"].revocable))({}, {}); Object.freeze(globalThis['Proxy'].revocable)({}, {}); Object.freeze((globalThis['Proxy'].revocable))({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {}); Object.freeze((globalThis.Proxy["revocable"]))({}, {}); Object.freeze(globalThis.Proxy['revocable'])({}, {}); Object.freeze((globalThis.Proxy['revocable']))({}, {}); Object.hasOwn({}, 'a'); Object.prototype.hasOwnProperty.call({}, 'a'); globalThis.Object.prototype["hasOwnProperty"].call({}, 'a'); globalThis.Object["hasOwn"]({}, 'a'); globalThis["Object"].prototype["hasOwnProperty"].call({}, 'a'); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; new WeakRef(); globalThis.WeakRef; globalThis["WeakRef"]; globalThis['WeakRef']; Object.freeze(globalThis["WeakRef"]); Object.freeze((globalThis["WeakRef"])); Object.freeze(globalThis['WeakRef']); Object.freeze((globalThis['WeakRef'])); new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"]; globalThis['FinalizationRegistry']; Object.freeze(globalThis["FinalizationRegistry"]); Object.freeze((globalThis["FinalizationRegistry"])); Object.freeze(globalThis['FinalizationRegistry']); Object.freeze((globalThis['FinalizationRegistry'])); globalThis.SharedArrayBuffer; globalThis["SharedArrayBuffer"]; globalThis['SharedArrayBuffer']; Object.freeze((true && globalThis.SharedArrayBuffer)); Object.freeze((false || globalThis.SharedArrayBuffer)); globalThis.Atomics; globalThis["Atomics"]; globalThis['Atomics']; Object.freeze((true && globalThis.Atomics)); Object.freeze((false || globalThis.Atomics)); Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno["connect"]('127.0.0.1', 1); globalThis["Deno"].connect('127.0.0.1', 1); globalThis["Deno"]["connect"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno["listen"]('127.0.0.1', 0); globalThis["Deno"].listen('127.0.0.1', 0); globalThis["Deno"]["listen"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno["serve"]('127.0.0.1', 0); globalThis["Deno"].serve('127.0.0.1', 0); globalThis["Deno"]["serve"]('127.0.0.1', 0);"#,
        kali_common::late_process_control_single_quoted_process_aliases_source()
    )
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

fn late_process_control_source() -> String {
    format!(
        "{} {} {}",
        kali_common::late_process_control_source(),
        kali_common::late_process_control_single_quoted_kill_source().trim_end(),
        kali_common::late_process_control_single_quoted_exit_source().trim_end()
    )
}

fn late_network_source() -> &'static str {
    kali_common::late_network_source()
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in ["process.kill", "undefined identifier 'process'"] {
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
    for expected in ["process.kill", "undefined identifier 'process'"] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {errors:?}"
        );
    }
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

fn assert_browser_late_tsx_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        [
            "undefined identifier 'process'",
            "environment mutation API 'process.env'"
        ]
        .iter()
        .any(|expected| stderr.contains(expected)),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("process.kill") || stderr.contains("process.env"),
        "stderr: {stderr}"
    );
}

fn assert_browser_late_tsx_compatibility_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().any(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            let message = error["message"].as_str().expect("error message");
            message.contains("undefined identifier 'process'")
                || message.contains("environment mutation API 'process.env'")
        }),
        "missing process gate in {errors:?}"
    );
}

fn nullish_coalescing_source() -> &'static str {
    "const value = null ?? 1;\nconsole.log(value);\n"
}

#[path = "late_compat_browser_tsx_input/run.rs"]
mod run;

#[path = "late_compat_browser_tsx_input/build.rs"]
mod build;

#[path = "late_compat_browser_tsx_input/check.rs"]
mod check;

#[path = "late_compat_browser_tsx_input/test.rs"]
mod test;

#[path = "late_compat_browser_tsx_input/misc.rs"]
mod misc;
