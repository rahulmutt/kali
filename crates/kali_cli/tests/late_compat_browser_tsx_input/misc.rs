use super::*;

#[test]
fn browser_late_process_control_source_includes_zero_probe_invocation_forms() {
    let source = late_process_control_source();
    for expected in [
        r#"const zero = 0"#,
        r#"const zeroAlias = zero"#,
        r#"process.kill(zeroAlias)"#,
        "process.kill(0)",
        "process.kill(+0)",
        "process.kill((0))",
        "((process)).kill(0)",
        "((process)).kill(+0)",
        "((globalThis.process)).kill(0)",
        "((globalThis.process)).kill(+0)",
        "globalThis.process.kill(0)",
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"process["kill"]((0))"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
        r#"Object.freeze((globalThis.process))["kill"](0)"#,
        r#"Object.freeze((globalThis.process))["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](+0)"#,
        r#"Object.freeze((process))["kill"](0)"#,
        r#"Object.freeze((process))["kill"](+0)"#,
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process).kill)(0)"#,
        r#"Object.freeze((globalThis.process).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze((process).kill)(0)"#,
        r#"Object.freeze((process).kill)(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0); Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0); Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        "((process.kill))(0)",
        r#"((process["kill"]))(0)"#,
        r#"((process["kill"]))(+0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis.process.kill))(+0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(+0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
    let zero_probe_source = kali_common::process_kill_zero_probe_alias_inventory_source();
    let parenthesized_receiver_freeze_source =
        kali_common::process_kill_zero_probe_parenthesized_receiver_freeze_source();
    assert!(
        source.contains(zero_probe_source.as_str()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(zero_probe_source.as_str()).count(),
        1,
        "browser TSX late-compat source should embed the shared zero-probe inventory exactly once"
    );
    assert!(
        source.contains(parenthesized_receiver_freeze_source.as_str()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(parenthesized_receiver_freeze_source.as_str()).count(),
        1,
        "browser TSX late-compat source should embed the shared parenthesized receiver-freeze source exactly once"
    );
    for expected in
        kali_common::process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases()
    {
        assert!(source.contains(expected), "source: {source}");
        assert_eq!(
            source.matches(expected).count(),
            1,
            "browser TSX late-compat source should embed each shared parenthesized receiver-freeze bracket alias exactly once: {expected}"
        );
    }
}

#[test]
fn browser_late_process_control_source_includes_single_quoted_process_root_forms() {
    let source = late_browser_tsx_compatibility_source();
    for expected in [
        r#"globalThis['process'].kill(0)"#,
        r#"globalThis['process']['kill'](0)"#,
        r#"process['kill'](0)"#,
        r#"process['exit'](0)"#,
        r#"globalThis['process'].exit(0)"#,
        r#"Object.freeze((globalThis['process'].kill))(0)"#,
        r#"Object.freeze((globalThis['process'].kill))(+0)"#,
        r#"Object.freeze((globalThis['process']['kill']))(0)"#,
        r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_tsx_compatibility_source_includes_bracketed_forms() {
    let source = late_browser_tsx_compatibility_source();
    assert!(
        source.contains(r#"globalThis.Object.prototype["hasOwnProperty"].call"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Object["hasOwn"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Object"].prototype["hasOwnProperty"].call"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"].NumberFormat"#),
        "source: {source}"
    );
    assert!(source.contains(r#"globalThis['Intl']"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis['Intl']['NumberFormat']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['DateTimeFormat']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['RelativeTimeFormat']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['Collator']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['DisplayNames']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['Segmenter']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['Locale']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Intl']['PluralRules']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.RelativeTimeFormat"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Collator"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.DisplayNames"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Segmenter"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.Locale"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl.PluralRules"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Intl["PluralRules"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["SharedArrayBuffer"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['SharedArrayBuffer']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Atomics"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Atomics']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy.revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Proxy"]["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy['revocable']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((Proxy.revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Proxy.revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['Proxy'].revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['Proxy'].revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["Proxy"].revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["Proxy"].revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["Proxy"]["revocable"]))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Proxy["revocable"]))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Deno["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"]["toObject"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].toObject"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["permissions"]["request"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["permissions"]["revoke"]"#),
        "source: {source}"
    );
    assert!(source.contains(r#"Deno["env"].set"#), "source: {source}");
    assert!(source.contains(r#"Deno["env"].delete"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis.Deno["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Deno["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"].env["set"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"].env["delete"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].set"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"].delete"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["set"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["env"]["delete"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["pid"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["cwd"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["chdir"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["kill"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["exit"]"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.connect('127.0.0.1', 1)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["connect"]('127.0.0.1', 1)"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.listen('127.0.0.1', 0)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["listen"]('127.0.0.1', 0)"#),
        "source: {source}"
    );
    assert!(
        source.contains("Deno.serve('127.0.0.1', 0)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Deno"]["serve"]('127.0.0.1', 0)"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_tsx_compatibility_source_includes_mixed_bracketed_proxy_revocable_form() {
    let source = late_browser_tsx_compatibility_source();
    assert!(
        source.contains(r#"globalThis['Proxy']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"new globalThis['Proxy']({}, {})"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Proxy"].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Proxy'].revocable"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.Proxy['revocable']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Proxy.revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['Proxy'].revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['Proxy'].revocable))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Proxy']['revocable']"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['Proxy']['revocable'])"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['Proxy']['revocable']))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis["Proxy"].revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["Proxy"]["revocable"]))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["Proxy"])["revocable"])"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_tsx_compatibility_source_includes_frozen_intl_number_format_form() {
    let source = late_browser_tsx_compatibility_source();
    assert!(
        source.contains(r#"Object.freeze(globalThis.Intl.NumberFormat)"#),
        "source: {source}"
    );
}
