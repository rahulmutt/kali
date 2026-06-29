use super::*;

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
