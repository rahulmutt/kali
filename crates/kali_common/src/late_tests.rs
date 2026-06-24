use crate::*;
use super::LATE_PROCESS_CONTROL_PREFIX_SEGMENTS;

#[test]
fn test_late_object_model_aliases_and_source_are_canonical() {
    let aliases = late_object_model_aliases();
    let source = late_object_model_source();

    assert_eq!(
        aliases,
        &[
            "Proxy",
            "globalThis.Proxy",
            r#"globalThis["Proxy"]"#,
            "globalThis['Proxy']",
            "new Proxy({}, {})",
            "new globalThis.Proxy({}, {})",
            r#"new globalThis["Proxy"]({}, {})"#,
            "new globalThis['Proxy']({}, {})",
            "new WeakMap()",
            "globalThis.WeakMap",
            r#"globalThis["WeakMap"]"#,
            r#"globalThis['WeakMap']"#,
            r#"globalThis["WeakMap"]()"#,
            r#"globalThis['WeakMap']()"#,
            "Object.freeze(new WeakMap())",
            "Object.freeze((new WeakMap()))",
            "Object.freeze(globalThis.WeakMap)",
            "Object.freeze((globalThis.WeakMap))",
            r#"Object.freeze(globalThis["WeakMap"])"#,
            r#"Object.freeze((globalThis["WeakMap"]))"#,
            r#"Object.freeze(globalThis['WeakMap'])"#,
            r#"Object.freeze((globalThis['WeakMap']))"#,
            "new WeakSet()",
            "globalThis.WeakSet",
            r#"globalThis["WeakSet"]"#,
            r#"globalThis['WeakSet']"#,
            r#"globalThis["WeakSet"]()"#,
            r#"globalThis['WeakSet']()"#,
            "Object.freeze(new WeakSet())",
            "Object.freeze((new WeakSet()))",
            "Object.freeze(globalThis.WeakSet)",
            "Object.freeze((globalThis.WeakSet))",
            r#"Object.freeze(globalThis["WeakSet"])"#,
            r#"Object.freeze((globalThis["WeakSet"]))"#,
            r#"Object.freeze(globalThis['WeakSet'])"#,
            r#"Object.freeze((globalThis['WeakSet']))"#,
            "globalThis.WeakRef",
            r#"globalThis["WeakRef"]"#,
            "globalThis['WeakRef']",
            "Object.freeze(globalThis.WeakRef)",
            "Object.freeze((globalThis.WeakRef))",
            r#"Object.freeze(globalThis["WeakRef"])"#,
            r#"Object.freeze((globalThis["WeakRef"]))"#,
            "Object.freeze(globalThis['WeakRef'])",
            "Object.freeze((globalThis['WeakRef']))",
            "new FinalizationRegistry(() => {})",
            "globalThis.FinalizationRegistry",
            r#"globalThis["FinalizationRegistry"](() => {})"#,
            r#"globalThis['FinalizationRegistry'](() => {})"#,
            "Object.freeze(new FinalizationRegistry(() => {}))",
            "Object.freeze((new FinalizationRegistry(() => {})))",
            "Object.freeze(globalThis.FinalizationRegistry)",
            "Object.freeze((globalThis.FinalizationRegistry))",
            r#"Object.freeze(globalThis["FinalizationRegistry"](() => {}))"#,
            r#"Object.freeze((globalThis["FinalizationRegistry"](() => {})))"#,
            r#"Object.freeze(globalThis['FinalizationRegistry'](() => {}))"#,
            r#"Object.freeze((globalThis['FinalizationRegistry'](() => {})))"#,
            r#"Object.freeze(globalThis["FinalizationRegistry"])"#,
            r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
            r#"Object.freeze(globalThis['FinalizationRegistry'])"#,
            r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
            "Proxy.revocable({}, {})",
            "globalThis.Proxy.revocable({}, {})",
            r#"globalThis["Proxy"]["revocable"]({}, {})"#,
            r#"globalThis['Proxy']['revocable']({}, {})"#,
            r#"globalThis["Proxy"].revocable({}, {})"#,
            r#"globalThis['Proxy'].revocable({}, {})"#,
            r#"globalThis.Proxy["revocable"]({}, {})"#,
            r#"globalThis.Proxy['revocable']({}, {})"#,
            r#"globalThis['Proxy']["revocable"]({}, {})"#,
            r#"globalThis["Proxy"]['revocable']({}, {})"#,
            r#"Object.freeze(globalThis['Proxy']["revocable"])({}, {})"#,
            r#"Object.freeze((globalThis['Proxy']["revocable"]))({}, {})"#,
            r#"Object.freeze((globalThis["Proxy"])["revocable"])({}, {})"#,
            r#"Object.freeze((globalThis['Proxy'])['revocable'])({}, {})"#,
            r#"Object.freeze(globalThis["Proxy"]['revocable'])({}, {})"#,
            r#"Object.freeze((globalThis["Proxy"]['revocable']))({}, {})"#,
            "Object.freeze(Proxy.revocable)({}, {})",
            "Object.freeze((Proxy.revocable))({}, {})",
            "Object.freeze(globalThis.Proxy.revocable)({}, {})",
            "Object.freeze((globalThis.Proxy.revocable))({}, {})",
            r#"Object.freeze(globalThis["Proxy"]["revocable"])({}, {})"#,
            r#"Object.freeze((globalThis["Proxy"]["revocable"]))({}, {})"#,
            r#"Object.freeze(globalThis['Proxy']['revocable'])({}, {})"#,
            r#"Object.freeze((globalThis['Proxy']['revocable']))({}, {})"#,
            r#"Object.freeze(globalThis["Proxy"].revocable)({}, {})"#,
            r#"Object.freeze((globalThis["Proxy"].revocable))({}, {})"#,
            r#"Object.freeze(globalThis['Proxy'].revocable)({}, {})"#,
            r#"Object.freeze((globalThis['Proxy']).revocable)({}, {})"#,
            r#"Object.freeze((globalThis['Proxy'].revocable))({}, {})"#,
            r#"Object.freeze(globalThis.Proxy["revocable"])({}, {})"#,
            r#"Object.freeze((globalThis.Proxy["revocable"]))({}, {})"#,
            r#"Object.freeze(globalThis.Proxy['revocable'])({}, {})"#,
            r#"Object.freeze((globalThis.Proxy['revocable']))({}, {})"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in late-object-model inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_late_object_model_own_property_aliases_and_source_are_canonical() {
    let aliases = late_object_model_own_property_aliases();
    let source = late_object_model_own_property_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"Object.hasOwn(globalThis, "a")"#,
            r#"globalThis.Object.hasOwn(globalThis, "a")"#,
            r#"globalThis.Object["hasOwn"](globalThis, "a")"#,
            r#"globalThis["Object"].hasOwn(globalThis, "a")"#,
            r#"globalThis["Object"]["hasOwn"](globalThis, "a")"#,
            r#"Object["hasOwnProperty"].call(globalThis, "a")"#,
            r#"globalThis.Object["hasOwnProperty"].call(globalThis, "a")"#,
            r#"globalThis.Object['hasOwnProperty'].call(globalThis, "a")"#,
            r#"globalThis["Object"]["hasOwnProperty"].call(globalThis, "a")"#,
            r#"globalThis["Object"]['hasOwnProperty'].call(globalThis, "a")"#,
            r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
            r#"globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
            r#"globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
            r#"globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
            r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
            r#"globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
            r#"globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a")"#,
            r#"globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a")"#,
            r#"globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a")"#,
            r#"globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a")"#,
            r#"globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a")"#,
            r#"globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a")"#,
            r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
            r#"globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
            r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in late-object-model own-property inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_late_threaded_runtime_aliases_and_source_are_canonical() {
    assert_eq!(
        late_threaded_runtime_aliases(),
        &[
            "SharedArrayBuffer",
            "globalThis.SharedArrayBuffer",
            r#"globalThis["SharedArrayBuffer"]"#,
            "globalThis['SharedArrayBuffer']",
            "Object.freeze(globalThis.SharedArrayBuffer)",
            r#"Object.freeze(globalThis["SharedArrayBuffer"])"#,
            "Object.freeze(globalThis['SharedArrayBuffer'])",
            "Object.freeze(SharedArrayBuffer)",
            "Object.freeze((SharedArrayBuffer))",
            "Object.freeze((globalThis.SharedArrayBuffer))",
            r#"Object.freeze((globalThis["SharedArrayBuffer"]))"#,
            "Object.freeze((globalThis['SharedArrayBuffer']))",
            "Object.freeze((null ?? globalThis.SharedArrayBuffer))",
            "Object.freeze((null ?? globalThis['SharedArrayBuffer']))",
            r#"Object.freeze((true && globalThis["SharedArrayBuffer"]))"#,
            "Object.freeze((true && globalThis['SharedArrayBuffer']))",
            "Object.freeze((true && globalThis.SharedArrayBuffer))",
            r#"Object.freeze((false || globalThis["SharedArrayBuffer"]))"#,
            "Object.freeze((false || globalThis['SharedArrayBuffer']))",
            "Object.freeze((false || globalThis.SharedArrayBuffer))",
            "Atomics",
            "globalThis.Atomics",
            r#"globalThis["Atomics"]"#,
            "globalThis['Atomics']",
            "Object.freeze(globalThis.Atomics)",
            r#"Object.freeze(globalThis["Atomics"])"#,
            "Object.freeze(globalThis['Atomics'])",
            "Object.freeze(Atomics)",
            "Object.freeze((Atomics))",
            "Object.freeze((globalThis.Atomics))",
            r#"Object.freeze((globalThis["Atomics"]))"#,
            "Object.freeze((globalThis['Atomics']))",
            "Object.freeze((null ?? globalThis.Atomics))",
            "Object.freeze((null ?? globalThis['Atomics']))",
            r#"Object.freeze((true && globalThis["Atomics"]))"#,
            "Object.freeze((true && globalThis['Atomics']))",
            "Object.freeze((true && globalThis.Atomics))",
            r#"Object.freeze((false || globalThis["Atomics"]))"#,
            "Object.freeze((false || globalThis['Atomics']))",
            "Object.freeze((false || globalThis.Atomics))",
        ]
    );

    assert_eq!(
        late_threaded_runtime_source(),
        "SharedArrayBuffer; globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; Object.freeze(globalThis.SharedArrayBuffer); Object.freeze(globalThis[\"SharedArrayBuffer\"]); Object.freeze(globalThis['SharedArrayBuffer']); Object.freeze(SharedArrayBuffer); Object.freeze((SharedArrayBuffer)); Object.freeze((globalThis.SharedArrayBuffer)); Object.freeze((globalThis[\"SharedArrayBuffer\"])); Object.freeze((globalThis['SharedArrayBuffer'])); Object.freeze((null ?? globalThis.SharedArrayBuffer)); Object.freeze((null ?? globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis[\"SharedArrayBuffer\"])); Object.freeze((true && globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis.SharedArrayBuffer)); Object.freeze((false || globalThis[\"SharedArrayBuffer\"])); Object.freeze((false || globalThis['SharedArrayBuffer'])); Object.freeze((false || globalThis.SharedArrayBuffer)); Atomics; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics']; Object.freeze(globalThis.Atomics); Object.freeze(globalThis[\"Atomics\"]); Object.freeze(globalThis['Atomics']); Object.freeze(Atomics); Object.freeze((Atomics)); Object.freeze((globalThis.Atomics)); Object.freeze((globalThis[\"Atomics\"])); Object.freeze((globalThis['Atomics'])); Object.freeze((null ?? globalThis.Atomics)); Object.freeze((null ?? globalThis['Atomics'])); Object.freeze((true && globalThis[\"Atomics\"])); Object.freeze((true && globalThis['Atomics'])); Object.freeze((true && globalThis.Atomics)); Object.freeze((false || globalThis[\"Atomics\"])); Object.freeze((false || globalThis['Atomics'])); Object.freeze((false || globalThis.Atomics));"
    );
}

#[test]
fn test_late_permission_escalation_source_lists_request_and_revoke_aliases() {
    assert_eq!(
        late_permission_escalation_aliases(),
        &[
            "Deno.permissions.request()",
            "Deno.permissions.revoke()",
            r#"Deno.permissions["request"]()"#,
            r#"Deno.permissions["revoke"]()"#,
            "globalThis.Deno.permissions.request()",
            "globalThis.Deno.permissions.revoke()",
            r#"globalThis.Deno.permissions["request"]()"#,
            r#"globalThis.Deno.permissions["revoke"]()"#,
            r#"globalThis["Deno"].permissions["request"]()"#,
            r#"globalThis["Deno"].permissions["revoke"]()"#,
            r#"globalThis["Deno"].permissions.request()"#,
            r#"globalThis["Deno"].permissions.revoke()"#,
            r#"globalThis["Deno"].permissions["request"]()"#,
            r#"globalThis["Deno"]["permissions"]["request"]()"#,
            r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
            r#"globalThis["Deno"]["permissions"].request()"#,
            r#"globalThis["Deno"]["permissions"].revoke()"#,
            r#"globalThis["Deno"].permissions["request"]()"#,
            r#"globalThis["Deno"].permissions["revoke"]()"#,
            r#"globalThis.Deno["permissions"]["request"]()"#,
            r#"globalThis.Deno["permissions"]["revoke"]()"#,
        ]
    );
    assert_eq!(
        late_permission_escalation_source(),
        r#"Deno.permissions.request(); Deno.permissions.revoke(); Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions.revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis.Deno["permissions"]["request"](); globalThis.Deno["permissions"]["revoke"]();"#
    );
}

#[test]
fn test_late_env_materialization_source_lists_to_object_aliases() {
    assert_eq!(
        late_env_materialization_source(),
        r#"Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env["toObject"](); Deno["env"]["toObject"](); Deno["env"].toObject(); globalThis.Deno.env["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis.Deno["env"].toObject(); globalThis["Deno"].env.toObject(); globalThis["Deno"].env["toObject"](); globalThis["Deno"]["env"].toObject(); globalThis["Deno"]["env"]["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis["Deno"].env.toObject();"#
    );
}

#[test]
fn test_late_subprocess_source_lists_command_aliases() {
    assert_eq!(
        late_subprocess_source(),
        r#"new Deno.Command('sh').spawn(); new Deno["Command"]('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno["Command"]('sh').spawn(); new globalThis["Deno"].Command('sh').spawn(); new globalThis["Deno"]["Command"]('sh').spawn();"#
    );
}

#[test]
fn test_late_network_source_lists_connect_listen_and_serve_aliases() {
    assert_eq!(
        late_network_source(),
        r#"Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno["connect"]('127.0.0.1', 1); globalThis["Deno"].connect('127.0.0.1', 1); globalThis["Deno"]["connect"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno["listen"]('127.0.0.1', 0); globalThis["Deno"].listen('127.0.0.1', 0); globalThis["Deno"]["listen"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno["serve"]('127.0.0.1', 0); globalThis["Deno"].serve('127.0.0.1', 0); globalThis["Deno"]["serve"]('127.0.0.1', 0);"#
    );
}

#[test]
fn test_late_compat_object_has_own_source_lists_representative_aliases_in_order() {
    let source = late_compat_object_has_own_source("globalThis", r#""a""#);

    for expected in [
        r#"Object.hasOwn(globalThis, "a")"#,
        r#"globalThis.Object.hasOwn(globalThis, "a")"#,
        r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
        r#"Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
        r#"Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
        r#"Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
        r#"Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
        r#"Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
        r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
        r#"globalThis.Object['hasOwnProperty'].call(globalThis, "a")"#,
        r#"globalThis["Object"]['hasOwnProperty'].call(globalThis, "a")"#,
        r#"globalThis['Object'].hasOwnProperty.call(globalThis, "a")"#,
        r#"globalThis['Object'].prototype['hasOwnProperty']['call'](globalThis, "a")"#,
        r#"globalThis['Object'].prototype['hasOwnProperty'].call(globalThis, "a")"#,
        r#"globalThis['Object']['prototype']['hasOwnProperty']['call'](globalThis, "a")"#,
        r#"globalThis['Object']["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    ] {
        assert!(source.contains(expected), "missing alias: {expected}");
    }

    assert!(
        source.ends_with(';'),
        "source should be semicolon-terminated: {source}"
    );
}

#[test]
fn test_late_process_control_prefix_source_lists_all_prefix_aliases_in_order() {
    let prefix = late_process_control_prefix_source();
    let expected = format!(
        "{}; {}",
        format!("{};", LATE_PROCESS_CONTROL_PREFIX_SEGMENTS.join("; ")).trim_end_matches(';'),
        late_process_control_exit_source().trim_end()
    );

    assert_eq!(prefix, expected);
    assert!(
        prefix.starts_with("Deno.pid; globalThis.Deno.pid;"),
        "prefix: {prefix}"
    );
    assert!(
        prefix.contains("process.kill; globalThis.process.kill;"),
        "prefix: {prefix}"
    );
    assert!(
        prefix.contains(r#"globalThis["process"].exit"#),
        "prefix: {prefix}"
    );
    assert!(
        !prefix.contains("Object.freeze((process)).kill(0)"),
        "prefix: {prefix}"
    );
    assert!(!prefix.contains("process.kill(0)"), "prefix: {prefix}");
}

#[test]
fn test_late_process_control_exit_aliases_are_canonical() {
    assert_eq!(
        late_process_control_exit_aliases(),
        &[
            "process.exit",
            "globalThis.process.exit",
            "globalThis.process[\"exit\"]",
            r#"globalThis["process"].exit"#,
            r#"globalThis["process"]["exit"]"#,
            "process[\"exit\"]",
        ]
    );
}

#[test]
fn test_late_process_control_exit_source_lists_all_aliases_in_order() {
    let source = late_process_control_exit_source();
    let expected = concat!(
        "process.exit; ",
        "globalThis.process.exit; ",
        "globalThis.process[\"exit\"]; ",
        "globalThis[\"process\"].exit; ",
        "globalThis[\"process\"][\"exit\"]; ",
        "process[\"exit\"];"
    );

    assert_eq!(source, expected);
}

#[test]
fn test_late_process_control_source_reuses_the_shared_zero_probe_inventory_once() {
    let source = late_process_control_source();
    let prefix = late_process_control_prefix_source();
    let zero_probe_source = process_kill_zero_probe_source();

    assert!(source.starts_with(&prefix), "source: {source}");
    assert!(source.contains(&zero_probe_source), "source: {source}");
    assert_eq!(
        source.matches(&zero_probe_source).count(),
        1,
        "late process control source should embed the zero-probe inventory exactly once"
    );
    let parenthesized_receiver_aliases = process_kill_zero_probe_parenthesized_receiver_aliases();
    assert_eq!(
        parenthesized_receiver_aliases,
        &[
            r#"((process)).kill(0)"#,
            r#"((process)).kill(+0)"#,
            r#"((globalThis.process)).kill(0)"#,
            r#"((globalThis.process)).kill(+0)"#,
        ]
    );
    let parenthesized_receiver_source = process_kill_zero_probe_parenthesized_receiver_source();
    assert!(
        source.contains(parenthesized_receiver_source.trim_end()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(parenthesized_receiver_source.trim_end()).count(),
        1,
        "late process control source should embed the transparent parenthesized receiver aliases exactly once"
    );
    assert!(
        source.contains("process.kill(zeroAlias)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process).kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process).kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"]["kill"]))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])["kill"])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])["kill"])(+0)"#),
        "source: {source}"
    );
    let parenthesized_receiver_freeze_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases();
    assert_eq!(
        parenthesized_receiver_freeze_aliases,
        &[
            r#"Object.freeze((process)).kill(0)"#,
            r#"Object.freeze((process)).kill(+0)"#,
            r#"Object.freeze((globalThis.process)).kill(0)"#,
            r#"Object.freeze((globalThis.process)).kill(+0)"#,
            r#"Object.freeze((globalThis["process"])).kill(0)"#,
            r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        ]
    );
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    assert!(
        source.contains(parenthesized_receiver_freeze_source.trim_end()),
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(parenthesized_receiver_freeze_source.trim_end())
            .count(),
        1,
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
    assert!(
        source.contains(r#"((process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis.process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process["kill"]))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process["kill"]))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis["process"])).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis["process"])).kill(+0)"#),
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
        source.contains(r#"globalThis["process"].exit"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["exit"]"#),
        "source: {source}"
    );
    assert!(
        prefix.ends_with("process[\"exit\"];"),
        "prefix should preserve the process-control preamble: {prefix}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_source_reuses_the_shared_zero_probe_inventory_once(
) {
    let source = late_process_control_single_quoted_process_source();
    let zero_probe_source = late_process_control_source();
    let single_quoted_process_source =
        join_semicolon_terminated_segments(late_process_control_single_quoted_process_aliases());
    let expected = format!(
        "{} {}",
        zero_probe_source,
        single_quoted_process_source.trim_end()
    );

    assert_eq!(source, expected, "source: {source}");
    for expected in [
        r#"; process['kill']((0));"#,
        r#"; globalThis['process'].kill((0));"#,
        r#"; process['exit']((0));"#,
        r#"; globalThis['process'].exit((0));"#,
        r#"; Object.freeze((process)['exit'])(0);"#,
        r#"; Object.freeze((process)['exit'])(+0);"#,
        r#"; Object.freeze((globalThis.process)['exit'])(0);"#,
        r#"; Object.freeze((globalThis.process)['exit'])(+0);"#,
        r#"; Object.freeze((globalThis['process'])['exit'])(0);"#,
        r#"; Object.freeze((globalThis['process'])['exit'])(+0);"#,
    ] {
        assert_eq!(
            source.matches(expected).count(),
            1,
            "wrapped single-quoted alias should appear exactly once: {expected}; source: {source}"
        );
    }
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
        source.contains(r#"globalThis.process['kill'](+0)"#),
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
    assert!(source.contains(r#"process['exit'](0)"#), "source: {source}");
    assert!(
        source.contains(r#"process['exit'](+0)"#),
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
        source.contains(r#"globalThis['process'].exit((0))"#),
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
        source.contains(r#"globalThis['process']['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['exit']((0))"#),
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
fn test_late_process_control_single_quoted_process_aliases_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_process_aliases();
    let aliases_source = late_process_control_single_quoted_process_aliases_source();
    let source = late_process_control_single_quoted_process_source();
    let expected_segment = aliases.join("; ");

    assert_eq!(
        aliases,
        &[
            r#"globalThis['process'].kill(0)"#,
            r#"globalThis['process'].kill(+0)"#,
            r#"globalThis['process']['kill'](0)"#,
            r#"globalThis['process']['kill'](+0)"#,
            r#"process['kill'](0)"#,
            r#"process['kill'](+0)"#,
            r#"process['kill']((0))"#,
            r#"globalThis.process['kill'](0)"#,
            r#"globalThis.process['kill'](+0)"#,
            r#"globalThis.process['kill']((0))"#,
            r#"globalThis['process'].kill((0))"#,
            r#"globalThis['process']['kill']((0))"#,
            r#"globalThis.process['kill']((0))"#,
            r#"Object.freeze(process['kill'])(0)"#,
            r#"Object.freeze(process['kill'])(+0)"#,
            r#"Object.freeze((process['kill']))(0)"#,
            r#"Object.freeze((process['kill']))(+0)"#,
            r#"Object.freeze(globalThis.process['kill'])(0)"#,
            r#"Object.freeze(globalThis.process['kill'])(+0)"#,
            r#"Object.freeze((globalThis.process['kill']))(0)"#,
            r#"Object.freeze((globalThis.process['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process'].kill)(0)"#,
            r#"Object.freeze(globalThis['process'].kill)(+0)"#,
            r#"Object.freeze((globalThis['process']).kill)(0)"#,
            r#"Object.freeze((globalThis['process']).kill)(+0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
            r#"Object.freeze((globalThis['process'].kill))(0)"#,
            r#"Object.freeze((globalThis['process'].kill))(+0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
            r#"process['exit'](0)"#,
            r#"process['exit'](+0)"#,
            r#"process['exit']((0))"#,
            r#"Object.freeze(process['exit'])(0)"#,
            r#"Object.freeze(process['exit'])(+0)"#,
            r#"Object.freeze((process['exit']))(0)"#,
            r#"Object.freeze((process['exit']))(+0)"#,
            r#"Object.freeze((process)['exit'])(0)"#,
            r#"Object.freeze((process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
            r#"globalThis['process'].exit(0)"#,
            r#"globalThis['process'].exit(+0)"#,
            r#"globalThis['process'].exit((0))"#,
            r#"globalThis['process']['exit'](0)"#,
            r#"globalThis['process']['exit'](+0)"#,
            r#"globalThis['process']['exit']((0))"#,
            r#"globalThis.process['exit'](0)"#,
            r#"globalThis.process['exit'](+0)"#,
            r#"globalThis.process['exit']((0))"#,
            r#"Object.freeze(globalThis['process'].exit)(0)"#,
            r#"Object.freeze(globalThis['process'].exit)(+0)"#,
            r#"Object.freeze((globalThis['process'].exit))(0)"#,
            r#"Object.freeze((globalThis['process'].exit))(+0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
        ]
    );
    assert_eq!(aliases_source, format!("{};", expected_segment));
    assert_eq!(
        source.matches(&expected_segment).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_aliases_compose_kill_and_exit_helpers() {
    let aliases = late_process_control_single_quoted_process_aliases();
    let kill_aliases = late_process_control_single_quoted_kill_aliases();
    let exit_aliases = late_process_control_single_quoted_exit_aliases();
    let kill_source = late_process_control_single_quoted_kill_source();
    let exit_source = late_process_control_single_quoted_exit_source();
    let source = late_process_control_single_quoted_process_aliases_source();

    assert_eq!(kill_aliases, &aliases[..kill_aliases.len()]);
    assert_eq!(exit_aliases, &aliases[kill_aliases.len()..]);
    assert_eq!(
        source,
        format!("{} {}", kill_source.trim_end(), exit_source.trim_end())
    );
    assert_eq!(
        source.matches(kill_source.trim_end()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(exit_source.trim_end()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_late_process_control_single_quoted_kill_source_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_kill_aliases();
    let source = late_process_control_single_quoted_kill_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"globalThis['process'].kill(0)"#,
            r#"globalThis['process'].kill(+0)"#,
            r#"globalThis['process']['kill'](0)"#,
            r#"globalThis['process']['kill'](+0)"#,
            r#"process['kill'](0)"#,
            r#"process['kill'](+0)"#,
            r#"process['kill']((0))"#,
            r#"globalThis.process['kill'](0)"#,
            r#"globalThis.process['kill'](+0)"#,
            r#"globalThis.process['kill']((0))"#,
            r#"globalThis['process'].kill((0))"#,
            r#"globalThis['process']['kill']((0))"#,
            r#"globalThis.process['kill']((0))"#,
            r#"Object.freeze(process['kill'])(0)"#,
            r#"Object.freeze(process['kill'])(+0)"#,
            r#"Object.freeze((process['kill']))(0)"#,
            r#"Object.freeze((process['kill']))(+0)"#,
            r#"Object.freeze(globalThis.process['kill'])(0)"#,
            r#"Object.freeze(globalThis.process['kill'])(+0)"#,
            r#"Object.freeze((globalThis.process['kill']))(0)"#,
            r#"Object.freeze((globalThis.process['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process'].kill)(0)"#,
            r#"Object.freeze(globalThis['process'].kill)(+0)"#,
            r#"Object.freeze((globalThis['process']).kill)(0)"#,
            r#"Object.freeze((globalThis['process']).kill)(+0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
            r#"Object.freeze((globalThis['process'].kill))(0)"#,
            r#"Object.freeze((globalThis['process'].kill))(+0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
        ]
    );
    assert_eq!(source, expected, "source: {source}");
}

#[test]
fn test_late_process_control_single_quoted_exit_source_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_exit_aliases();
    let source = late_process_control_single_quoted_exit_aliases_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"process['exit'](0)"#,
            r#"process['exit'](+0)"#,
            r#"process['exit']((0))"#,
            r#"Object.freeze(process['exit'])(0)"#,
            r#"Object.freeze(process['exit'])(+0)"#,
            r#"Object.freeze((process['exit']))(0)"#,
            r#"Object.freeze((process['exit']))(+0)"#,
            r#"Object.freeze((process)['exit'])(0)"#,
            r#"Object.freeze((process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
            r#"globalThis['process'].exit(0)"#,
            r#"globalThis['process'].exit(+0)"#,
            r#"globalThis['process'].exit((0))"#,
            r#"globalThis['process']['exit'](0)"#,
            r#"globalThis['process']['exit'](+0)"#,
            r#"globalThis['process']['exit']((0))"#,
            r#"globalThis.process['exit'](0)"#,
            r#"globalThis.process['exit'](+0)"#,
            r#"globalThis.process['exit']((0))"#,
            r#"Object.freeze(globalThis['process'].exit)(0)"#,
            r#"Object.freeze(globalThis['process'].exit)(+0)"#,
            r#"Object.freeze((globalThis['process'].exit))(0)"#,
            r#"Object.freeze((globalThis['process'].exit))(+0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
        ]
    );
    assert_eq!(source, expected);
    assert_eq!(source.matches(&expected).count(), 1, "source: {source}");
    assert!(!source.contains("process.kill(0)"), "source: {source}");
}

#[test]
fn test_late_process_env_mutation_source_lists_mixed_quote_process_aliases_and_mixed_delete_aliases(
) {
    let aliases = late_process_env_mutation_aliases();
    let source = late_process_env_mutation_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(source, expected);
    for fragment in [
        r#"process['env'] = {}"#,
        r#"process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis.process['env'] = {}"#,
        r#"globalThis.process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis["process"]['env'] = {}"#,
        r#"globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"globalThis['process']["env"] = {}"#,
        r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
        r#"delete globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION']"#,
        r#"delete globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION']"#,
    ] {
        assert!(
            source.contains(fragment),
            "missing fragment: {fragment}; source: {source}"
        );
    }
}
