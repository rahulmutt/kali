use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
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
    assert!(stderr.contains("process.kill"), "stderr: {stderr}");
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
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("process.kill")),
        "missing process.kill in {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("undefined identifier 'process'")),
        "missing process identifier gate in {errors:?}"
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

fn late_object_model_source() -> &'static str {
    r#"Intl; Object.freeze(globalThis.Intl.NumberFormat); Object.freeze((null ?? globalThis.Intl.NumberFormat)); globalThis.Intl; globalThis["Intl"]; globalThis['Intl']; globalThis.Intl.NumberFormat; globalThis["Intl"].NumberFormat; globalThis.Intl["NumberFormat"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"].DateTimeFormat; globalThis['Intl']['DateTimeFormat']; globalThis.Intl["DateTimeFormat"]; globalThis["Intl"]["DateTimeFormat"]; globalThis.Intl.RelativeTimeFormat; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"].RelativeTimeFormat; globalThis.Intl["RelativeTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis.Intl.Collator; globalThis['Intl']['Collator']; globalThis["Intl"].Collator; globalThis.Intl["Collator"]; globalThis["Intl"]["Collator"]; globalThis.Intl.DisplayNames; globalThis['Intl']['DisplayNames']; globalThis["Intl"].DisplayNames; globalThis.Intl["DisplayNames"]; globalThis["Intl"]["DisplayNames"]; globalThis.Intl.Segmenter; globalThis['Intl']['Segmenter']; globalThis["Intl"].Segmenter; globalThis.Intl["Segmenter"]; globalThis["Intl"]["Segmenter"]; globalThis.Intl.Locale; globalThis['Intl']['Locale']; globalThis["Intl"].Locale; globalThis.Intl["Locale"]; globalThis["Intl"]["Locale"]; globalThis.Intl.PluralRules; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis["Intl"].PluralRules; globalThis.Intl["PluralRules"]; Proxy; globalThis.Proxy; globalThis["Proxy"]; globalThis['Proxy']; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); new globalThis['Proxy']({}, {}); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis['Proxy']['revocable']({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis['Proxy'].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); globalThis.Proxy['revocable']({}, {}); globalThis['Proxy']["revocable"]({}, {}); globalThis['Proxy']["revocable"]({}, {}); Object.freeze(globalThis['Proxy']["revocable"])({}, {}); Object.freeze((globalThis['Proxy']["revocable"]))({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze((globalThis["Proxy"])["revocable"])({}, {}); Object.freeze(globalThis['Proxy']['revocable'])({}, {}); Object.freeze((globalThis['Proxy']['revocable']))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze((globalThis["Proxy"].revocable))({}, {}); Object.freeze(globalThis['Proxy'].revocable)({}, {}); Object.freeze((globalThis['Proxy'].revocable))({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {}); Object.freeze((globalThis.Proxy["revocable"]))({}, {}); Object.freeze(globalThis.Proxy['revocable'])({}, {}); Object.freeze((globalThis.Proxy['revocable']))({}, {}); Object.freeze(globalThis?.Proxy.revocable)({}, {}); Object.freeze((globalThis?.Proxy.revocable))({}, {}); Object.hasOwn({}, 'a'); globalThis.Object.hasOwn({}, 'a'); globalThis.Object["hasOwn"]({}, 'a'); globalThis["Object"]["hasOwn"]({}, 'a'); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; new WeakRef(); globalThis.WeakRef; globalThis["WeakRef"]; Object.freeze(globalThis["WeakRef"]); Object.freeze((globalThis["WeakRef"])); new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"]; Object.freeze(globalThis["FinalizationRegistry"]); Object.freeze((globalThis["FinalizationRegistry"])); globalThis.SharedArrayBuffer; globalThis["SharedArrayBuffer"]; globalThis['SharedArrayBuffer']; Object.freeze((true && globalThis.SharedArrayBuffer)); Object.freeze((false || globalThis.SharedArrayBuffer)); globalThis.Atomics; globalThis["Atomics"]; globalThis['Atomics']; Object.freeze((true && globalThis.Atomics)); Object.freeze((false || globalThis.Atomics));"#
}

fn assert_browser_late_object_model_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis['Intl']",
        "globalThis.Intl.NumberFormat",
        "globalThis['Intl']['NumberFormat']",
        "globalThis['Intl']['DateTimeFormat']",
        "globalThis['Intl']['RelativeTimeFormat']",
        "globalThis['Intl']['Collator']",
        "globalThis['Intl']['DisplayNames']",
        "globalThis['Intl']['Segmenter']",
        "globalThis['Intl']['Locale']",
        "globalThis['Intl']['PluralRules']",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "globalThis.Intl.PluralRules",
        r#"globalThis["Intl"]["PluralRules"]"#,
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
        "SharedArrayBuffer",
        "Atomics",
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
        "globalThis['Intl']",
        "globalThis.Intl.NumberFormat",
        "globalThis['Intl']['NumberFormat']",
        "globalThis['Intl']['DateTimeFormat']",
        "globalThis['Intl']['RelativeTimeFormat']",
        "globalThis['Intl']['Collator']",
        "globalThis['Intl']['DisplayNames']",
        "globalThis['Intl']['Segmenter']",
        "globalThis['Intl']['Locale']",
        "globalThis['Intl']['PluralRules']",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Segmenter",
        "globalThis.Intl.Locale",
        "globalThis.Intl.PluralRules",
        r#"globalThis["Intl"]["PluralRules"]"#,
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
        "SharedArrayBuffer",
        "Atomics",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

fn nullish_coalescing_source() -> &'static str {
    "const value = null ?? 1;\nconsole.log(value);\n"
}

#[path = "late_compat_browser_jsx_input/run.rs"]
mod run;

#[path = "late_compat_browser_jsx_input/build.rs"]
mod build;

#[path = "late_compat_browser_jsx_input/check.rs"]
mod check;

#[path = "late_compat_browser_jsx_input/misc.rs"]
mod misc;
