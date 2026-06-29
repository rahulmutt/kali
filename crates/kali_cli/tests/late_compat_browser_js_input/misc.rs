use super::*;

#[test]
fn browser_late_object_model_source_is_composed_from_shared_helpers() {
    let source = late_object_model_source();
    let intl_source = kali_common::broader_intl_source();
    let object_model_source = kali_common::late_object_model_source();
    let has_own_source = kali_common::late_compat_object_has_own_source("{}", r#""a""#);
    let has_own_source: &str = has_own_source.as_ref();

    assert!(source.starts_with(intl_source.as_str()), "source: {source}");
    assert!(source.contains(object_model_source), "source: {source}");
    assert!(source.contains(has_own_source), "source: {source}");
    assert_eq!(
        source.matches(object_model_source).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(has_own_source).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn browser_late_threaded_runtime_source_includes_bracketed_forms() {
    let source = late_threaded_runtime_source();
    assert!(source.contains("SharedArrayBuffer"), "source: {source}");
    assert!(
        source.contains("Object.freeze(SharedArrayBuffer)"),
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
    assert!(source.contains("Atomics"), "source: {source}");
    assert!(
        source.contains("Object.freeze(Atomics)"),
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
        source.contains(r#"Object.freeze((true && globalThis.SharedArrayBuffer))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((false || globalThis.SharedArrayBuffer))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((true && globalThis.Atomics))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((false || globalThis.Atomics))"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_process_control_source_includes_single_quoted_process_root_forms() {
    let source = kali_common::late_process_control_single_quoted_process_source();
    let single_quoted_process_source =
        kali_common::late_process_control_single_quoted_process_aliases_source();
    assert!(
        source.contains(single_quoted_process_source.as_str()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(single_quoted_process_source.as_str()).count(),
        1,
        "browser JS late-compat source should embed the shared single-quoted process inventory exactly once"
    );
    assert!(
        source.contains(r#"globalThis['process'].kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill'](+0)"#),
        "source: {source}"
    );
    assert!(source.contains(r#"process['kill'](0)"#), "source: {source}");
    assert!(
        source.contains(r#"process['kill'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"process['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].kill((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']).kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']).kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'])['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'])['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(source.contains(r#"process['exit'](0)"#), "source: {source}");
    assert!(
        source.contains(r#"process['exit'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"process['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['exit'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['exit'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['exit'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['exit']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['exit']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].exit)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].exit)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].exit))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].exit))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['exit'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['exit'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['exit']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['exit']))(+0)"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_object_model_source_includes_bracketed_intl_forms() {
    let source = late_object_model_source();
    let intl_source = kali_common::broader_intl_source();

    assert!(source.contains(intl_source.as_str()), "source: {source}");
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
        r#"globalThis.Proxy["revocable"]"#,
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis.Object["hasOwn"]"#,
        r#"globalThis.Object["prototype"].hasOwnProperty.call"#,
        r#"globalThis.Object.prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"].prototype.hasOwnProperty["call"]"#,
        r#"globalThis["Object"].prototype["hasOwnProperty"].call"#,
        r#"globalThis["Object"]["prototype"].hasOwnProperty.call"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        r#"globalThis['WeakMap']"#,
        r#"Object.freeze(globalThis["WeakMap"])"#,
        r#"Object.freeze(globalThis['WeakMap'])"#,
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        r#"globalThis['WeakSet']"#,
        r#"Object.freeze(globalThis["WeakSet"])"#,
        r#"Object.freeze(globalThis['WeakSet'])"#,
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        r#"Object.freeze((globalThis["WeakRef"]))"#,
        r#"Object.freeze((globalThis['WeakRef']))"#,
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
        r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
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
    assert!(
        source.contains(r#"globalThis.Proxy["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['Proxy']["revocable"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['Proxy']["revocable"])"#),
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
    assert!(
        source.contains(r#"Object.freeze(globalThis["Proxy"]["revocable"])"#),
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
        source.contains(r#"Object.freeze(globalThis.Proxy["revocable"])"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Proxy["revocable"]))"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_process_control_source_includes_bracketed_forms() {
    let source = late_process_control_source();
    let prefix = kali_common::late_process_control_prefix_source();
    assert!(source.starts_with(prefix.as_str()), "source: {source}");
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
        r#"globalThis["process"]["pid"]"#,
        r#"globalThis["process"].cwd"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"globalThis["process"].chdir"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"const zero = 0"#,
        r#"const zeroAlias = zero"#,
        r#"process.kill(zeroAlias)"#,
        r#"process.kill(0)"#,
        r#"process.kill(+0)"#,
        r#"process.kill((0))"#,
        r#"((process)).kill(0)"#,
        r#"((process)).kill(+0)"#,
        r#"((globalThis.process)).kill(0)"#,
        r#"((globalThis.process)).kill(+0)"#,
        r#"globalThis.process.kill(0)"#,
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
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
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
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
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
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze((process).kill)(0)"#,
        r#"Object.freeze((process).kill)(+0)"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0); Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0); Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"((process.kill))(0)"#,
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
    assert!(source.contains("process.kill(0)"), "source: {source}");
    assert!(source.contains("process.kill(+0)"), "source: {source}");
    assert!(source.contains("process.kill((0))"), "source: {source}");
    assert!(
        source.contains("globalThis.process.kill(0)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"].kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["kill"](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process["kill"](0)"#),
        "source: {source}"
    );
    assert!(source.contains("((process.kill))(0)"), "source: {source}");
    assert!(
        source.contains(r#"((process["kill"]))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process["kill"]))(+0)"#),
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
        "browser JS late-compat source should embed the shared zero-probe inventory exactly once"
    );
    assert!(
        source.contains(parenthesized_receiver_freeze_source.as_str()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(parenthesized_receiver_freeze_source.as_str()).count(),
        1,
        "browser JS late-compat source should embed the shared parenthesized receiver-freeze source exactly once"
    );
    for expected in
        kali_common::process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases()
    {
        assert!(source.contains(expected), "source: {source}");
        assert_eq!(
            source.matches(expected).count(),
            1,
            "browser JS late-compat source should embed each shared parenthesized receiver-freeze bracket alias exactly once: {expected}"
        );
    }
}

#[test]
fn browser_late_subprocess_source_includes_bracketed_forms() {
    let source = late_subprocess_source();
    for expected in [
        r#"new Deno.Command('sh').spawn()"#,
        r#"new Deno["Command"]('sh').spawn()"#,
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
        r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis.process.env"#,
        r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env"#,
        r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
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
        r#"globalThis.Deno["permissions"]["request"]()"#,
        r#"globalThis.Deno["permissions"]["revoke"]()"#,
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
        r#"globalThis.Deno["permissions"]["request"]()"#,
        r#"globalThis.Deno["permissions"]["revoke"]()"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_env_mutation_source_includes_bracketed_forms() {
    let source = late_env_mutation_source();
    for expected in [
        r#"Deno["env"].set"#,
        r#"Deno["env"].delete"#,
        r#"globalThis.Deno["env"].set"#,
        r#"globalThis.Deno["env"].delete"#,
        r#"globalThis.Deno["env"]["set"]"#,
        r#"globalThis.Deno["env"]["delete"]"#,
        r#"globalThis["Deno"].env["set"]"#,
        r#"globalThis["Deno"].env["delete"]"#,
        r#"globalThis["Deno"]["env"].set"#,
        r#"globalThis["Deno"]["env"].delete"#,
        r#"globalThis["Deno"]["env"]["set"]"#,
        r#"globalThis["Deno"]["env"]["delete"]"#,
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
