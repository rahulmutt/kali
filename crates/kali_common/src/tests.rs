use super::*;

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
fn test_broader_intl_aliases_and_source_are_canonical() {
    let aliases = broader_intl_aliases();
    let source = broader_intl_source();

    assert_eq!(
        aliases,
        &[
            "Intl",
            "globalThis.Intl",
            r#"globalThis["Intl"]"#,
            "globalThis['Intl']",
            "globalThis.Intl.NumberFormat",
            r#"globalThis["Intl"].NumberFormat"#,
            r#"globalThis.Intl["NumberFormat"]"#,
            r#"globalThis['Intl'].NumberFormat"#,
            r#"globalThis['Intl']["NumberFormat"]"#,
            "globalThis.Intl.DateTimeFormat",
            r#"globalThis["Intl"].DateTimeFormat"#,
            r#"globalThis.Intl["DateTimeFormat"]"#,
            r#"globalThis['Intl'].DateTimeFormat"#,
            r#"globalThis['Intl']["DateTimeFormat"]"#,
            r#"globalThis["Intl"]["DateTimeFormat"]"#,
            "globalThis.Intl.PluralRules",
            r#"globalThis["Intl"].PluralRules"#,
            r#"globalThis.Intl["PluralRules"]"#,
            r#"globalThis['Intl'].PluralRules"#,
            r#"globalThis['Intl']["PluralRules"]"#,
            "globalThis.Intl.RelativeTimeFormat",
            r#"globalThis["Intl"].RelativeTimeFormat"#,
            r#"globalThis.Intl["RelativeTimeFormat"]"#,
            r#"globalThis['Intl'].RelativeTimeFormat"#,
            r#"globalThis['Intl']["RelativeTimeFormat"]"#,
            "globalThis.Intl.Collator",
            r#"globalThis["Intl"].Collator"#,
            r#"globalThis.Intl["Collator"]"#,
            r#"globalThis['Intl'].Collator"#,
            r#"globalThis['Intl']["Collator"]"#,
            "globalThis.Intl.DisplayNames",
            r#"globalThis["Intl"].DisplayNames"#,
            r#"globalThis.Intl["DisplayNames"]"#,
            r#"globalThis['Intl'].DisplayNames"#,
            r#"globalThis['Intl']["DisplayNames"]"#,
            "globalThis.Intl.Segmenter",
            r#"globalThis["Intl"].Segmenter"#,
            r#"globalThis.Intl["Segmenter"]"#,
            r#"globalThis['Intl'].Segmenter"#,
            r#"globalThis['Intl']["Segmenter"]"#,
            "globalThis.Intl.Locale",
            r#"globalThis["Intl"].Locale"#,
            r#"globalThis.Intl["Locale"]"#,
            r#"globalThis['Intl'].Locale"#,
            r#"globalThis['Intl']["Locale"]"#,
            "globalThis['Intl']['Segmenter']",
            "globalThis['Intl']['NumberFormat']",
            "globalThis['Intl']['DateTimeFormat']",
            "globalThis['Intl']['PluralRules']",
            "globalThis['Intl']['RelativeTimeFormat']",
            "globalThis['Intl']['Collator']",
            "globalThis['Intl']['DisplayNames']",
            "globalThis['Intl']['Locale']",
            r#"globalThis["Intl"]["NumberFormat"]"#,
            r#"globalThis["Intl"]["PluralRules"]"#,
            r#"globalThis["Intl"]["RelativeTimeFormat"]"#,
            r#"globalThis["Intl"]["Collator"]"#,
            r#"globalThis["Intl"]["DisplayNames"]"#,
            r#"globalThis["Intl"]["Segmenter"]"#,
            r#"globalThis["Intl"]["Locale"]"#,
            "Intl.NumberFormat",
            "Intl.DateTimeFormat",
            "Intl.PluralRules",
            "Intl.RelativeTimeFormat",
            "Intl.Collator",
            "Intl.DisplayNames",
            "Intl.Locale",
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in broader Intl inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_array_from_aliases_list_all_supported_aliases_in_order() {
    let aliases = array_from_aliases();
    let source = array_from_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            "Array.from",
            "globalThis.Array.from",
            r#"globalThis["Array"].from"#,
            r#"globalThis["Array"]["from"]"#,
            r#"globalThis["Array"]['from']"#,
            r#"globalThis['Array'].from"#,
            r#"globalThis['Array']['from']"#,
            r#"globalThis['Array']["from"]"#,
            r#"Array["from"]"#,
            r#"Array['from']"#,
            r#"globalThis.Array["from"]"#,
            r#"globalThis.Array['from']"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Array.from inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_array_from_frozen_callable_aliases_contains_representative_supported_aliases_and_source_is_canonical(
) {
    let aliases = array_from_frozen_callable_aliases();
    let source = array_from_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for alias in [
        r#"Object.freeze(Array.from)"#,
        r#"Object.freeze((Array.from))"#,
        r#"Object.freeze(globalThis.Array.from)"#,
        r#"Object.freeze((globalThis.Array.from))"#,
        r#"Object.freeze(globalThis["Array"].from)"#,
        r#"Object.freeze((globalThis["Array"].from))"#,
        r#"Object.freeze(globalThis["Array"]["from"])"#,
        r#"Object.freeze(globalThis["Array"]['from'])"#,
        r#"Object.freeze(globalThis['Array']["from"])"#,
        r#"Object.freeze(globalThis['Array']['from'])"#,
        r#"Object.freeze((globalThis["Array"])["from"])"#,
        r#"Object.freeze((globalThis["Array"])['from'])"#,
        r#"Object.freeze((globalThis['Array'])["from"])"#,
        r#"Object.freeze((globalThis['Array'])['from'])"#,
        r#"Object.freeze(globalThis.Array["from"])"#,
        r#"Object.freeze(globalThis.Array['from'])"#,
        r#"Object.freeze((null ?? globalThis.Array["from"]))"#,
        r#"Object.freeze((true && globalThis.Array["from"]))"#,
        r#"Object.freeze((false || globalThis.Array["from"]))"#,
        r#"Object.freeze((Array.from, Array.from))"#,
        r#"Object.freeze((globalThis.Array.from, globalThis.Array.from))"#,
        r#"Object.freeze((globalThis["Array"].from, globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis['Array'].from, globalThis['Array'].from))"#,
    ] {
        assert!(aliases.contains(&alias), "missing alias: {alias}");
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Array.from frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_array_from_alias_inventory_source_reuses_the_shared_helper_sources_once() {
    let source = array_from_alias_inventory_source();
    assert_eq!(
        source,
        format!(
            "{} {}",
            array_from_source().trim_end(),
            array_from_frozen_callable_source().trim_end()
        )
    );
    assert_eq!(
        source.matches(&array_from_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(&array_from_frozen_callable_source()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_array_from_loop_lines_renders_all_aliases_in_order() {
    let source = array_from_loop_lines(
        "Array.from; globalThis.Array.from",
        "for (const value of ",
        "  ",
    );
    assert_eq!(
        source,
        "  for (const value of Array.from(values)) {\n    console.log(value);\n  }\n  for (const value of globalThis.Array.from(values)) {\n    console.log(value);\n  }"
    );
}

#[test]
fn test_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        template_literal_string_iteration_body_source(),
        "for (const ch of `hello`) { console.log(ch); }"
    );
}

#[test]
fn test_browser_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        browser_template_literal_string_iteration_body_source(),
        concat!(
            "const prefix = \"he\";\n",
            "const suffix = \"llo\";\n",
            "const syncChars = [];\n",
            "for (const item of `${prefix}${suffix}`) {\n",
            "  syncChars.push(item);\n",
            "}\n",
            "const asyncChars = [];\n",
            "for await (const item of `${prefix}${suffix}`) {\n",
            "  asyncChars.push(item);\n",
            "}\n",
            "if (syncChars.join(\"\") !== \"hello\" || asyncChars.join(\"\") !== \"hello\") {\n",
            "  throw new Error('unexpected template literal iteration semantics');\n",
            "}"
        )
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
fn test_number_predicates_source_helpers_are_canonical() {
    assert_eq!(
        number_predicates_preamble_source("1"),
        r#"const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);"#,
    );
    assert_eq!(
        number_predicates_preamble_source("1 as const"),
        r#"const alias = 1 as const; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);"#,
    );
    assert_eq!(
        number_predicates_console_log_body_source(),
        concat!(
            r#"console.log(Number.isFinite(alias)); "#,
            r#"console.log(integer(alias)); "#,
            r#"console.log(Number.isSafeInteger(alias)); "#,
            r#"console.log(integer(1.5)); "#,
            r#"console.log(Number.isFinite("hello")); "#,
            r#"console.log(Number.isSafeInteger(1.5)); "#,
            r#"console.log(globalThis["Number"]["isNaN"](NaN)); "#,
            r#"console.log(globalThis.Number.isNaN(1)); "#,
            r#"console.log(globalThis["Number"].isNaN(1)); "#,
            r#"console.log(globalThis["Number"]["isFinite"](alias)); "#,
            r#"console.log(globalThis["Number"]["isInteger"](alias)); "#,
            r#"console.log(globalThis["Number"]["isSafeInteger"](alias)); "#,
            r#"console.log(globalThis.Number["isNaN"](1)); "#,
            r#"console.log(globalThis["Number"].isFinite(alias)); "#,
            r#"console.log(globalThis.Number["isInteger"](alias)); "#,
            r#"console.log(globalThis["Number"].isSafeInteger(alias)); "#,
            r#"console.log(Number["isFinite"](alias)); "#,
            r#"console.log(Number["isInteger"](alias)); "#,
            r#"console.log(Number["isSafeInteger"](alias)); "#,
            r#"console.log(Number["isNaN"](1)); "#,
            r#"console.log(frozenFinite(alias)); "#,
            r#"console.log(frozenNaN(NaN)); "#,
            r#"console.log(frozenNaN(1)); "#,
            r#"console.log(frozenInteger(alias)); "#,
            r#"console.log(frozenSafeInteger(alias)); "#,
            r#"console.log(frozenBracketedFinite(alias)); "#,
            r#"console.log(frozenBracketedNaN(NaN)); "#,
            r#"console.log(frozenBracketedNaN(1)); "#,
            r#"console.log(frozenBracketedInteger(alias)); "#,
            r#"console.log(frozenBracketedSafeInteger(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedFinite(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedNaN(NaN)); "#,
            r#"console.log(frozenParenthesizedBracketedNaN(1)); "#,
            r#"console.log(frozenParenthesizedBracketedInteger(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedSafeInteger(alias)); "#,
            r#"console.log(frozenParenthesizedPropertyFinite(alias)); "#,
            r#"console.log(frozenParenthesizedPropertyNaN(NaN)); "#,
            r#"console.log(frozenParenthesizedPropertyNaN(1)); "#,
            r#"console.log(frozenParenthesizedPropertyInteger(alias)); "#,
            r#"console.log(frozenParenthesizedPropertySafeInteger(alias)); "#,
            r#"console.log(finite(alias)); "#,
            r#"console.log(integer(alias)); "#,
            r#"console.log(safeInteger(alias));"#
        )
    );
    assert_eq!(
        number_predicates_runtime_source(),
        format!(
            "{} {}",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert_eq!(
        number_predicates_test_source(),
        format!(
            "Kali.test('number predicates', () => {{ {} {} }});",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert!(number_predicates_browser_bundle_source("1").starts_with(
        r#"// kali-tree-shake: browserNumberPredicates
async function browserNumberPredicates() {
  const alias = 1;"#
    ));
    assert!(
        number_predicates_browser_bundle_source("1 as const").contains("const alias = 1 as const;")
    );
    assert!(number_predicates_browser_bundle_source("1")
        .contains("Number.isSafeInteger(await alias) !== true"));
    assert!(number_predicates_browser_bundle_source("1").contains("Object.freeze(Number.isFinite)"));
    assert!(number_predicates_browser_bundle_source("1").contains("Object.freeze(Number.isNaN)"));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze(Number["isFinite"])"#));
    assert!(
        number_predicates_browser_bundle_source("1").contains(r#"Object.freeze(Number["isNaN"])"#)
    );
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"])["isFinite"])"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"])["isNaN"])"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isFinite)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isNaN)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isInteger)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isSafeInteger)"#));
    assert!(number_predicates_browser_bundle_source("1").ends_with("}\n"));
}

#[test]
fn test_math_abs_sign_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_abs_sign_frozen_callable_aliases();
    let source = math_abs_sign_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["abs"])"#,
        r#"Object.freeze((globalThis.Math["abs"]))"#,
        r#"Object.freeze(globalThis.Math['abs'])"#,
        r#"Object.freeze((globalThis.Math['abs']))"#,
        r#"Object.freeze(globalThis.Math.abs)"#,
        r#"Object.freeze((globalThis.Math.abs))"#,
        r#"Object.freeze(globalThis["Math"]["abs"])"#,
        r#"Object.freeze((globalThis["Math"]["abs"]))"#,
        r#"Object.freeze(globalThis["Math"]['abs'])"#,
        r#"Object.freeze((globalThis["Math"]['abs']))"#,
        r#"Object.freeze(globalThis["Math"].abs)"#,
        r#"Object.freeze((globalThis["Math"].abs))"#,
        r#"Object.freeze(globalThis['Math']['abs'])"#,
        r#"Object.freeze((globalThis['Math']['abs']))"#,
        r#"Object.freeze(globalThis['Math'].abs)"#,
        r#"Object.freeze((globalThis['Math'].abs))"#,
        r#"Object.freeze(Math.abs)"#,
        r#"Object.freeze((Math.abs))"#,
        r#"Object.freeze(Math["abs"])"#,
        r#"Object.freeze((Math["abs"]))"#,
        r#"Object.freeze(Math['abs'])"#,
        r#"Object.freeze((Math['abs']))"#,
        r#"Object.freeze(globalThis.Math["sign"])"#,
        r#"Object.freeze((globalThis.Math["sign"]))"#,
        r#"Object.freeze(globalThis.Math['sign'])"#,
        r#"Object.freeze((globalThis.Math['sign']))"#,
        r#"Object.freeze(globalThis.Math.sign)"#,
        r#"Object.freeze((globalThis.Math.sign))"#,
        r#"Object.freeze(globalThis["Math"]["sign"])"#,
        r#"Object.freeze((globalThis["Math"]["sign"]))"#,
        r#"Object.freeze(globalThis["Math"]['sign'])"#,
        r#"Object.freeze((globalThis["Math"]['sign']))"#,
        r#"Object.freeze(globalThis["Math"].sign)"#,
        r#"Object.freeze((globalThis["Math"].sign))"#,
        r#"Object.freeze(globalThis['Math']['sign'])"#,
        r#"Object.freeze((globalThis['Math']['sign']))"#,
        r#"Object.freeze(globalThis['Math'].sign)"#,
        r#"Object.freeze((globalThis['Math'].sign))"#,
        r#"Object.freeze(Math.sign)"#,
        r#"Object.freeze((Math.sign))"#,
        r#"Object.freeze(Math["sign"])"#,
        r#"Object.freeze((Math["sign"]))"#,
        r#"Object.freeze(Math['sign'])"#,
        r#"Object.freeze((Math['sign']))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.abs / Math.sign frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_abs_sign_frozen_callable_invocation_and_entry_sources_are_canonical() {
    let aliases = math_abs_sign_frozen_callable_aliases();

    assert_eq!(
        math_abs_sign_frozen_callable_invocation_source(),
        aliases
            .iter()
            .map(|alias| format!("console.log({alias}(alias));"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        math_abs_sign_frozen_callable_entries_source(),
        aliases
            .iter()
            .map(|alias| format!("{alias}(alias)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn test_math_floor_trunc_ceil_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_floor_trunc_ceil_frozen_callable_aliases();
    let source = math_floor_trunc_ceil_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math['floor'])"#,
        r#"Object.freeze((globalThis.Math['floor']))"#,
        r#"Object.freeze(globalThis.Math.floor)"#,
        r#"Object.freeze((globalThis.Math.floor))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze((globalThis["Math"]))["floor"]"#,
        r#"Object.freeze((globalThis["Math"]))['floor']"#,
        r#"Object.freeze((globalThis.Math))["floor"]"#,
        r#"Object.freeze((globalThis.Math))['floor']"#,
        r#"Object.freeze((globalThis['Math']))["floor"]"#,
        r#"Object.freeze((globalThis['Math']))['floor']"#,
        r#"Object.freeze(globalThis["Math"]['floor'])"#,
        r#"Object.freeze((globalThis["Math"]['floor']))"#,
        r#"Object.freeze(globalThis["Math"].floor)"#,
        r#"Object.freeze((globalThis["Math"])["floor"])"#,
        r#"Object.freeze((globalThis['Math'])['floor'])"#,
        r#"Object.freeze(globalThis['Math'].floor)"#,
        r#"Object.freeze((globalThis['Math']).floor)"#,
        r#"Object.freeze((globalThis["Math"]).floor)"#,
        r#"Object.freeze((globalThis["Math"].floor))"#,
        r#"Object.freeze(Math["floor"])"#,
        r#"Object.freeze((Math["floor"]))"#,
        r#"Object.freeze(Math['floor'])"#,
        r#"Object.freeze((Math['floor']))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math['trunc'])"#,
        r#"Object.freeze((globalThis.Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math.trunc)"#,
        r#"Object.freeze((globalThis.Math.trunc))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze((globalThis["Math"]))["trunc"]"#,
        r#"Object.freeze((globalThis["Math"]))['trunc']"#,
        r#"Object.freeze((globalThis.Math))["trunc"]"#,
        r#"Object.freeze((globalThis.Math))['trunc']"#,
        r#"Object.freeze((globalThis['Math']))["trunc"]"#,
        r#"Object.freeze((globalThis['Math']))['trunc']"#,
        r#"Object.freeze(globalThis["Math"]['trunc'])"#,
        r#"Object.freeze((globalThis["Math"]['trunc']))"#,
        r#"Object.freeze(globalThis["Math"].trunc)"#,
        r#"Object.freeze((globalThis["Math"])["trunc"])"#,
        r#"Object.freeze((globalThis['Math'])['trunc'])"#,
        r#"Object.freeze(globalThis['Math'].trunc)"#,
        r#"Object.freeze((globalThis['Math']).trunc)"#,
        r#"Object.freeze((globalThis["Math"]).trunc)"#,
        r#"Object.freeze((globalThis["Math"].trunc))"#,
        r#"Object.freeze(Math["trunc"])"#,
        r#"Object.freeze((Math["trunc"]))"#,
        r#"Object.freeze(Math['trunc'])"#,
        r#"Object.freeze((Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis.Math['ceil'])"#,
        r#"Object.freeze((globalThis.Math['ceil']))"#,
        r#"Object.freeze(globalThis.Math.ceil)"#,
        r#"Object.freeze((globalThis.Math.ceil))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
        r#"Object.freeze((globalThis["Math"]))["ceil"]"#,
        r#"Object.freeze((globalThis["Math"]))['ceil']"#,
        r#"Object.freeze((globalThis.Math))["ceil"]"#,
        r#"Object.freeze((globalThis.Math))['ceil']"#,
        r#"Object.freeze((globalThis['Math']))["ceil"]"#,
        r#"Object.freeze((globalThis['Math']))['ceil']"#,
        r#"Object.freeze(globalThis["Math"]['ceil'])"#,
        r#"Object.freeze((globalThis["Math"]['ceil']))"#,
        r#"Object.freeze(globalThis["Math"].ceil)"#,
        r#"Object.freeze((globalThis["Math"])["ceil"])"#,
        r#"Object.freeze((globalThis['Math'])['ceil'])"#,
        r#"Object.freeze(globalThis['Math'].ceil)"#,
        r#"Object.freeze((globalThis['Math']).ceil)"#,
        r#"Object.freeze((globalThis["Math"]).ceil)"#,
        r#"Object.freeze((globalThis["Math"].ceil))"#,
        r#"Object.freeze(Math["ceil"])"#,
        r#"Object.freeze((Math["ceil"]))"#,
        r#"Object.freeze(Math['ceil'])"#,
        r#"Object.freeze((Math['ceil']))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.floor / Math.trunc / Math.ceil frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_floor_trunc_ceil_frozen_callable_invocation_and_entry_sources_are_canonical() {
    let aliases = math_floor_trunc_ceil_frozen_callable_aliases();

    assert_eq!(
        math_floor_trunc_ceil_frozen_callable_invocation_source(),
        aliases
            .iter()
            .map(|alias| format!("console.log({alias}(alias));"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        math_floor_trunc_ceil_frozen_callable_entries_source(),
        aliases
            .iter()
            .map(|alias| format!("{alias}(alias)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn test_math_round_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_round_frozen_callable_aliases();
    let source = math_round_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["round"])"#,
        r#"Object.freeze((globalThis.Math["round"]))"#,
        r#"Object.freeze(globalThis.Math['round'])"#,
        r#"Object.freeze((globalThis.Math['round']))"#,
        r#"Object.freeze(globalThis.Math.round)"#,
        r#"Object.freeze((globalThis.Math.round))"#,
        r#"Object.freeze(globalThis?.Math.round)"#,
        r#"Object.freeze((globalThis?.Math.round))"#,
        r#"Object.freeze(globalThis["Math"]["round"])"#,
        r#"Object.freeze((globalThis["Math"]["round"]))"#,
        r#"Object.freeze(globalThis["Math"]['round'])"#,
        r#"Object.freeze((globalThis["Math"]['round']))"#,
        r#"Object.freeze(globalThis["Math"].round)"#,
        r#"Object.freeze((globalThis["Math"]).round)"#,
        r#"Object.freeze((globalThis["Math"].round))"#,
        r#"Object.freeze((globalThis["Math"])["round"])"#,
        r#"Object.freeze((globalThis['Math'])['round'])"#,
        r#"Object.freeze((globalThis['Math'])["round"])"#,
        r#"Object.freeze(globalThis['Math']['round'])"#,
        r#"Object.freeze((globalThis['Math']['round']))"#,
        r#"Object.freeze(globalThis['Math'].round)"#,
        r#"Object.freeze((globalThis['Math']).round)"#,
        r#"Object.freeze((globalThis['Math'].round))"#,
        r#"Object.freeze(Math.round)"#,
        r#"Object.freeze((Math.round))"#,
        r#"Object.freeze(Math["round"])"#,
        r#"Object.freeze((Math["round"]))"#,
        r#"Object.freeze(Math['round'])"#,
        r#"Object.freeze((Math['round']))"#,
        r#"Object.freeze((null ?? Math.round))"#,
        r#"Object.freeze((true && globalThis.Math.round))"#,
        r#"Object.freeze((false || globalThis["Math"]["round"]))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.round frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_source_lists_all_aliases_in_order() {
    let aliases = math_pow_aliases();
    let source = math_pow_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        "Math.pow",
        r#"Math['pow']"#,
        r#"Math["pow"]"#,
        "globalThis.Math.pow",
        r#"globalThis.Math['pow']"#,
        r#"globalThis.Math["pow"]"#,
        r#"globalThis['Math'].pow"#,
        r#"globalThis['Math']['pow']"#,
        r#"globalThis['Math']["pow"]"#,
        r#"globalThis["Math"].pow"#,
        r#"globalThis["Math"]["pow"]"#,
        r#"globalThis["Math"]['pow']"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_alias_inventory_source_reuses_the_shared_helper_sources_once() {
    let source = math_pow_alias_inventory_source();
    assert_eq!(
        source,
        format!(
            "{} {}",
            math_pow_source().trim_end(),
            math_pow_frozen_callable_source().trim_end()
        )
    );
    assert_eq!(
        source.matches(&math_pow_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(&math_pow_frozen_callable_source()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_math_pow_browser_alias_inventory_aliases_list_all_aliases_in_order() {
    let aliases = math_pow_browser_alias_inventory_aliases();
    let source = math_pow_browser_alias_inventory_source();

    assert_eq!(
        aliases,
        &[
            "Math.pow",
            r#"Math['pow']"#,
            r#"Math["pow"]"#,
            "globalThis.Math.pow",
            r#"globalThis.Math['pow']"#,
            r#"globalThis.Math["pow"]"#,
            r#"globalThis['Math'].pow"#,
            r#"globalThis['Math']['pow']"#,
            r#"globalThis['Math']["pow"]"#,
            r#"globalThis["Math"].pow"#,
            r#"globalThis["Math"]["pow"]"#,
            r#"globalThis["Math"]['pow']"#,
            r#"Object.freeze(globalThis.Math['pow'])"#,
            r#"Object.freeze(globalThis.Math["pow"])"#,
            r#"Object.freeze(globalThis['Math']['pow'])"#,
            r#"Object.freeze(globalThis['Math']["pow"])"#,
            r#"Object.freeze(globalThis["Math"]["pow"])"#,
            r#"Object.freeze(globalThis["Math"]['pow'])"#,
            r#"Object.freeze(globalThis.Math.pow)"#,
            r#"Object.freeze(globalThis['Math'].pow)"#,
            r#"Object.freeze(globalThis["Math"].pow)"#,
            r#"Object.freeze(Math.pow)"#,
            r#"Object.freeze(Math['pow'])"#,
            r#"Object.freeze(Math["pow"])"#,
            r#"Object.freeze((globalThis.Math['pow']))"#,
            r#"Object.freeze((globalThis.Math["pow"]))"#,
            r#"Object.freeze((globalThis['Math']['pow']))"#,
            r#"Object.freeze((globalThis['Math']["pow"]))"#,
            r#"Object.freeze((globalThis["Math"]["pow"]))"#,
            r#"Object.freeze((globalThis["Math"]['pow']))"#,
            r#"Object.freeze((globalThis.Math.pow))"#,
            r#"Object.freeze((globalThis['Math'].pow))"#,
            r#"Object.freeze((globalThis["Math"].pow))"#,
            r#"Object.freeze((Math.pow))"#,
            r#"Object.freeze((Math['pow']))"#,
            r#"Object.freeze((Math["pow"]))"#,
            r#"Object.freeze((null ?? Math.pow))"#,
            r#"Object.freeze((true && Math.pow))"#,
            r#"Object.freeze((false || Math.pow))"#,
            r#"Object.freeze((null ?? globalThis.Math.pow))"#,
            r#"Object.freeze((true && globalThis.Math.pow))"#,
            r#"Object.freeze((false || globalThis.Math.pow))"#,
            r#"Object.freeze((null ?? globalThis["Math"]["pow"]))"#,
            r#"Object.freeze((true && globalThis["Math"]["pow"]))"#,
            r#"Object.freeze((false || globalThis["Math"]["pow"]))"#,
            r#"Object.freeze((null ?? globalThis['Math']['pow']))"#,
            r#"Object.freeze((true && globalThis['Math']['pow']))"#,
            r#"Object.freeze((false || globalThis['Math']['pow']))"#,
            r#"Object.freeze((globalThis.Math))["pow"]"#,
            r#"Object.freeze((globalThis.Math))['pow']"#,
            r#"Object.freeze((globalThis.Math).pow)"#,
            r#"Object.freeze((globalThis.Math)['pow'])"#,
            r#"Object.freeze((globalThis["Math"]))["pow"]"#,
            r#"Object.freeze((globalThis['Math']))['pow']"#,
            r#"Object.freeze((globalThis['Math'])["pow"])"#,
            r#"Object.freeze((globalThis['Math'])['pow'])"#,
            r#"Object.freeze((globalThis["Math"]).pow)"#,
            r#"Object.freeze((globalThis['Math']).pow)"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow browser inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_math_pow_browser_alias_inventory_source_is_canonical() {
    let source = math_pow_browser_alias_inventory_source();
    let aliases = math_pow_browser_alias_inventory_aliases();
    assert_eq!(source, format!("{};", aliases.join("; ")));
    assert_eq!(
        source.matches(&math_pow_alias_inventory_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(&math_pow_bracketed_frozen_callable_source())
            .count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_math_pow_browser_alias_inventory_source_reuses_the_canonical_math_pow_alias_inventory() {
    let source = math_pow_browser_alias_inventory_source();
    let canonical = math_pow_alias_inventory_source();
    let bracketed = math_pow_bracketed_frozen_callable_source();

    assert!(source.starts_with(&canonical), "source: {source}");
    assert_eq!(source.matches(&canonical).count(), 1, "source: {source}");
    assert!(source.ends_with(&bracketed), "source: {source}");
    assert_eq!(source.matches(&bracketed).count(), 1, "source: {source}");
}

#[test]
fn test_math_cbrt_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_cbrt_frozen_callable_aliases();
    let source = math_cbrt_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze(globalThis.Math["cbrt"])"#,
            r#"Object.freeze((globalThis.Math["cbrt"]))"#,
            r#"Object.freeze(globalThis.Math['cbrt'])"#,
            r#"Object.freeze((globalThis.Math['cbrt']))"#,
            r#"Object.freeze(globalThis.Math.cbrt)"#,
            r#"Object.freeze((globalThis.Math.cbrt))"#,
            r#"Object.freeze((globalThis.Math)["cbrt"])"#,
            r#"Object.freeze((globalThis.Math)['cbrt'])"#,
            r#"Object.freeze(globalThis["Math"]["cbrt"])"#,
            r#"Object.freeze((globalThis["Math"]["cbrt"]))"#,
            r#"Object.freeze(globalThis["Math"]['cbrt'])"#,
            r#"Object.freeze((globalThis["Math"]['cbrt']))"#,
            r#"Object.freeze((globalThis["Math"]))["cbrt"]"#,
            r#"Object.freeze((globalThis["Math"]))['cbrt']"#,
            r#"Object.freeze((globalThis.Math))["cbrt"]"#,
            r#"Object.freeze((globalThis.Math))['cbrt']"#,
            r#"Object.freeze((globalThis["Math"]).cbrt)"#,
            r#"Object.freeze((globalThis["Math"])["cbrt"])"#,
            r#"Object.freeze(globalThis["Math"].cbrt)"#,
            r#"Object.freeze((globalThis["Math"].cbrt))"#,
            r#"Object.freeze((globalThis['Math'])["cbrt"])"#,
            r#"Object.freeze((globalThis['Math'])['cbrt'])"#,
            r#"Object.freeze((globalThis['Math']))["cbrt"]"#,
            r#"Object.freeze((globalThis['Math']))['cbrt']"#,
            r#"Object.freeze(globalThis['Math'].cbrt)"#,
            r#"Object.freeze((globalThis['Math'].cbrt))"#,
            r#"Object.freeze(Math.cbrt)"#,
            r#"Object.freeze((Math.cbrt))"#,
            r#"Object.freeze(Math["cbrt"])"#,
            r#"Object.freeze((Math["cbrt"]))"#,
            r#"Object.freeze(Math['cbrt'])"#,
            r#"Object.freeze((Math['cbrt']))"#,
        ]
    );

    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_math_hypot_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_hypot_frozen_callable_aliases();
    let source = math_hypot_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze(globalThis.Math["hypot"])"#,
            r#"Object.freeze((globalThis.Math["hypot"]))"#,
            r#"Object.freeze(globalThis.Math['hypot'])"#,
            r#"Object.freeze((globalThis.Math['hypot']))"#,
            r#"Object.freeze(globalThis.Math.hypot)"#,
            r#"Object.freeze((globalThis.Math.hypot))"#,
            r#"Object.freeze(globalThis["Math"]["hypot"])"#,
            r#"Object.freeze((globalThis["Math"]["hypot"]))"#,
            r#"Object.freeze(globalThis["Math"]['hypot'])"#,
            r#"Object.freeze((globalThis["Math"]['hypot']))"#,
            r#"Object.freeze((globalThis["Math"]).hypot)"#,
            r#"Object.freeze((globalThis["Math"])["hypot"])"#,
            r#"Object.freeze((globalThis["Math"])['hypot'])"#,
            r#"Object.freeze(globalThis["Math"].hypot)"#,
            r#"Object.freeze((globalThis["Math"].hypot))"#,
            r#"Object.freeze(globalThis['Math']['hypot'])"#,
            r#"Object.freeze((globalThis['Math']['hypot']))"#,
            r#"Object.freeze((globalThis['Math']).hypot)"#,
            r#"Object.freeze((globalThis['Math'])["hypot"])"#,
            r#"Object.freeze((globalThis['Math'])['hypot'])"#,
            r#"Object.freeze((globalThis["Math"]))["hypot"]"#,
            r#"Object.freeze((globalThis['Math']))["hypot"]"#,
            r#"Object.freeze((globalThis.Math))["hypot"]"#,
            r#"Object.freeze((globalThis.Math))['hypot']"#,
            r#"Object.freeze(globalThis['Math'].hypot)"#,
            r#"Object.freeze((globalThis['Math'].hypot))"#,
            r#"Object.freeze(Math.hypot)"#,
            r#"Object.freeze((Math.hypot))"#,
            r#"Object.freeze(Math["hypot"])"#,
            r#"Object.freeze((Math["hypot"]))"#,
            r#"Object.freeze(Math['hypot'])"#,
            r#"Object.freeze((Math['hypot']))"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.hypot frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_math_exp2_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_exp2_frozen_callable_aliases();
    let source = math_exp2_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze(globalThis.Math["exp2"])"#,
            r#"Object.freeze((globalThis.Math["exp2"]))"#,
            r#"Object.freeze(globalThis.Math['exp2'])"#,
            r#"Object.freeze((globalThis.Math['exp2']))"#,
            r#"Object.freeze(globalThis.Math.exp2)"#,
            r#"Object.freeze((globalThis.Math.exp2))"#,
            r#"Object.freeze(globalThis?.Math.exp2)"#,
            r#"Object.freeze((globalThis?.Math.exp2))"#,
            r#"Object.freeze(globalThis["Math"]["exp2"])"#,
            r#"Object.freeze((globalThis["Math"]["exp2"]))"#,
            r#"Object.freeze(globalThis["Math"]['exp2'])"#,
            r#"Object.freeze((globalThis["Math"]['exp2']))"#,
            r#"Object.freeze(globalThis["Math"].exp2)"#,
            r#"Object.freeze((globalThis["Math"]).exp2)"#,
            r#"Object.freeze((globalThis["Math"].exp2))"#,
            r#"Object.freeze((globalThis["Math"])["exp2"])"#,
            r#"Object.freeze((globalThis['Math'])['exp2'])"#,
            r#"Object.freeze((globalThis['Math'])["exp2"])"#,
            r#"Object.freeze(globalThis['Math']['exp2'])"#,
            r#"Object.freeze((globalThis['Math']['exp2']))"#,
            r#"Object.freeze(globalThis['Math'].exp2)"#,
            r#"Object.freeze((globalThis['Math']).exp2)"#,
            r#"Object.freeze((globalThis['Math'].exp2))"#,
            r#"Object.freeze(Math.exp2)"#,
            r#"Object.freeze((Math.exp2))"#,
            r#"Object.freeze(Math["exp2"])"#,
            r#"Object.freeze((Math["exp2"]))"#,
            r#"Object.freeze(Math['exp2'])"#,
            r#"Object.freeze((Math['exp2']))"#,
            r#"Object.freeze((null ?? globalThis.Math["exp2"]))"#,
            r#"Object.freeze((true && globalThis.Math["exp2"]))"#,
            r#"Object.freeze((false || globalThis.Math["exp2"]))"#,
            r#"Object.freeze((null ?? globalThis["Math"].exp2))"#,
            r#"Object.freeze((true && globalThis["Math"].exp2))"#,
            r#"Object.freeze((false || globalThis["Math"].exp2))"#,
            r#"Object.freeze((null ?? Math.exp2))"#,
            r#"Object.freeze((true && globalThis.Math.exp2))"#,
            r#"Object.freeze((false || globalThis.Math.exp2))"#,
            r#"Object.freeze((null ?? globalThis["Math"]["exp2"]))"#,
            r#"Object.freeze((true && globalThis["Math"]["exp2"]))"#,
            r#"Object.freeze((false || globalThis["Math"]["exp2"]))"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.exp2 frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_math_pow_browser_alias_inventory_invocation_lines_are_canonical() {
    let source = math_pow_browser_alias_inventory_invocation_lines("");
    let expected = math_pow_invocation_lines_for_aliases(
        math_pow_browser_alias_inventory_aliases().as_slice(),
        "2",
        "alias",
        "",
    );

    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_browser_alias_inventory_invocation_source_is_canonical() {
    let source = math_pow_browser_alias_inventory_invocation_source();
    let expected = format!(
        "const exponent = 3; const alias = exponent;\n{}\n",
        math_pow_browser_alias_inventory_invocation_lines("")
    );

    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_bracketed_global_this_alias_chain_source_is_canonical() {
    assert_eq!(
        math_pow_bracketed_global_this_alias_chain_source(),
        concat!(
            "// kali-tree-shake: bracketedGlobalThisMathPowAliasChain\n",
            "function bracketedGlobalThisMathPowAliasChain() {\n",
            "  const exponent = 3;\n",
            "  const alias = exponent;\n",
            "  console.log(globalThis[\"Math\"].pow(2, alias));\n",
            "  return globalThis[\"Math\"].pow(2, alias);\n",
            "}\n",
        )
    );
}

#[test]
fn test_promise_all_settled_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_all_settled_browser_body_source();

    assert!(
        body.contains("const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"]).allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedSettled = await Object.freeze(globalThis[\"Promise\"][\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis[\"Promise\"][\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis[\"Promise\"].allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"].allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise[\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketedRootFrozenSettled = await Object.freeze(Promise[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise[\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("parenthesizedFrozenBracketedSettled.length !== 2"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.allSettled semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_promise_race_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_race_browser_body_source();

    assert!(
        body.contains(
            "const mixed = await Promise[\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains(
            "const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketed = await globalThis[\"Promise\"].race([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedDotted = await globalThis.Promise[\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketedBracketed = await globalThis[\"Promise\"][\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)[\"race\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketedBracketed = await Object.freeze((globalThis[\"Promise\"][\"race\"]))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketed = await Object.freeze(globalThis[\"Promise\"].race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedBracketed = await Object.freeze(globalThis[\"Promise\"][\"race\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.race semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_promise_any_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_any_browser_body_source();

    assert!(
        body.contains(
            "const direct = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains(
            "const mixed = await Promise[\"any\"]([Promise.reject('boom'), Promise.resolve(1)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixed = await Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketed = await globalThis[\"Promise\"].any([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketed = await Object.freeze((globalThis[\"Promise\"])[\"any\"])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketed = await globalThis['Promise']['any']([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)[\"any\"])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketed = await globalThis['Promise'].any([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenBracketed = await Object.freeze(globalThis["Promise"].any)([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenMixedBracketed = await Object.freeze(globalThis["Promise"]["any"])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketRoot = await Object.freeze(globalThis['Promise']['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenSingleMixedBracketed = await Object.freeze(globalThis["Promise"]['any'])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenMixedBracketed = await Object.freeze((globalThis["Promise"]["any"]))([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleBracketRoot = await Object.freeze((globalThis['Promise']['any']))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenSingleMixedBracketed = await Object.freeze((globalThis["Promise"]['any']))([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenReceiverWrappedDotted = await Object.freeze((globalThis["Promise"]).any)([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleReceiverWrappedDotted = await Object.freeze((globalThis['Promise']).any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRoot = await Object.freeze((null ?? Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrRoot = await Object.freeze((false || Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenDottedBracketed = await Object.freeze((globalThis.Promise)["any"])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.any semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_promise_all_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_all_browser_body_source();

    assert!(
        body.contains(
            "const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedRoot = await Object.freeze(Promise[\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise[\"all\"]))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedRoot = await Object.freeze(globalThis.Promise[\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedBracketedRoot = await Object.freeze(globalThis[\"Promise\"][\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error(\"unexpected Promise.all results\");"),
        "body: {body}"
    );
}

#[test]
fn test_math_pow_frozen_callable_source_lists_all_aliases_in_order() {
    let direct_aliases = math_pow_frozen_callable_direct_aliases();
    let parenthesized_aliases = math_pow_frozen_callable_parenthesized_aliases();
    let nullish_logical_aliases = math_pow_frozen_callable_nullish_logical_aliases();
    let aliases = math_pow_frozen_callable_aliases();
    let source = math_pow_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math['pow'])"#,
        r#"Object.freeze(globalThis.Math["pow"])"#,
        r#"Object.freeze(globalThis['Math']['pow'])"#,
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
        r#"Object.freeze(globalThis.Math.pow)"#,
        r#"Object.freeze(globalThis['Math'].pow)"#,
        r#"Object.freeze(globalThis["Math"].pow)"#,
        r#"Object.freeze(Math.pow)"#,
        r#"Object.freeze(Math['pow'])"#,
        r#"Object.freeze(Math["pow"])"#,
    ] {
        assert!(
            direct_aliases.contains(&expected_alias),
            "missing direct alias: {expected_alias}"
        );
    }

    for expected_alias in [
        r#"Object.freeze((globalThis.Math['pow']))"#,
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze((globalThis['Math']['pow']))"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze((globalThis['Math'].pow))"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
        r#"Object.freeze((Math.pow))"#,
        r#"Object.freeze((Math['pow']))"#,
        r#"Object.freeze((Math["pow"]))"#,
    ] {
        assert!(
            parenthesized_aliases.contains(&expected_alias),
            "missing parenthesized alias: {expected_alias}"
        );
    }

    for expected_alias in [
        r#"Object.freeze((null ?? Math.pow))"#,
        r#"Object.freeze((true && Math.pow))"#,
        r#"Object.freeze((false || Math.pow))"#,
        r#"Object.freeze((null ?? globalThis.Math.pow))"#,
        r#"Object.freeze((true && globalThis.Math.pow))"#,
        r#"Object.freeze((false || globalThis.Math.pow))"#,
        r#"Object.freeze((null ?? globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((true && globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((false || globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((null ?? globalThis['Math']['pow']))"#,
        r#"Object.freeze((true && globalThis['Math']['pow']))"#,
        r#"Object.freeze((false || globalThis['Math']['pow']))"#,
    ] {
        assert!(
            nullish_logical_aliases.contains(&expected_alias),
            "missing nullish/logical alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(
        aliases.len(),
        direct_aliases.len() + parenthesized_aliases.len() + nullish_logical_aliases.len()
    );
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_bracketed_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_pow_bracketed_frozen_callable_aliases();
    let source = math_pow_bracketed_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze((globalThis.Math))["pow"]"#,
            r#"Object.freeze((globalThis.Math))['pow']"#,
            r#"Object.freeze((globalThis.Math).pow)"#,
            r#"Object.freeze((globalThis.Math)['pow'])"#,
            r#"Object.freeze((globalThis["Math"]))["pow"]"#,
            r#"Object.freeze((globalThis['Math']))['pow']"#,
            r#"Object.freeze((globalThis['Math'])["pow"])"#,
            r#"Object.freeze((globalThis['Math'])['pow'])"#,
            r#"Object.freeze((globalThis["Math"]).pow)"#,
            r#"Object.freeze((globalThis['Math']).pow)"#,
        ]
    );
    assert_eq!(source, "Object.freeze((globalThis.Math))[\"pow\"]; Object.freeze((globalThis.Math))['pow']; Object.freeze((globalThis.Math).pow); Object.freeze((globalThis.Math)['pow']); Object.freeze((globalThis[\"Math\"]))[\"pow\"]; Object.freeze((globalThis['Math']))['pow']; Object.freeze((globalThis['Math'])[\"pow\"]); Object.freeze((globalThis['Math'])['pow']); Object.freeze((globalThis[\"Math\"]).pow); Object.freeze((globalThis['Math']).pow);");
}

#[test]
fn test_math_pow_bracketed_frozen_callable_invocation_lines_are_canonical() {
    let source = math_pow_bracketed_frozen_callable_invocation_lines("  ");
    let expected = math_pow_invocation_lines_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        "  ",
    );

    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_bracketed_frozen_callable_invocation_entries_are_canonical() {
    let source = math_pow_bracketed_frozen_callable_invocation_entries("    ");
    let expected = math_pow_invocation_entries_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        "    ",
    );

    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_invocation_lines_are_canonical() {
    let source = math_pow_invocation_lines(&math_pow_source(), "  ");
    let direct = math_pow_invocation_lines_for_aliases(math_pow_aliases(), "2", "alias", "  ");
    let direct_entries =
        math_pow_invocation_entries_for_aliases(math_pow_aliases(), "2", "alias", "    ");
    let expected = concat!(
        "  console.log(Math.pow(2, alias));\n",
        "  console.log(Math['pow'](2, alias));\n",
        "  console.log(Math[\"pow\"](2, alias));\n",
        "  console.log(globalThis.Math.pow(2, alias));\n",
        "  console.log(globalThis.Math['pow'](2, alias));\n",
        "  console.log(globalThis.Math[\"pow\"](2, alias));\n",
        "  console.log(globalThis['Math'].pow(2, alias));\n",
        "  console.log(globalThis['Math']['pow'](2, alias));\n",
        "  console.log(globalThis['Math'][\"pow\"](2, alias));\n",
        "  console.log(globalThis[\"Math\"].pow(2, alias));\n",
        "  console.log(globalThis[\"Math\"][\"pow\"](2, alias));\n",
        "  console.log(globalThis[\"Math\"]['pow'](2, alias));"
    );
    let expected_entries = concat!(
        "    Math.pow(2, alias),\n",
        "    Math['pow'](2, alias),\n",
        "    Math[\"pow\"](2, alias),\n",
        "    globalThis.Math.pow(2, alias),\n",
        "    globalThis.Math['pow'](2, alias),\n",
        "    globalThis.Math[\"pow\"](2, alias),\n",
        "    globalThis['Math'].pow(2, alias),\n",
        "    globalThis['Math']['pow'](2, alias),\n",
        "    globalThis['Math'][\"pow\"](2, alias),\n",
        "    globalThis[\"Math\"].pow(2, alias),\n",
        "    globalThis[\"Math\"][\"pow\"](2, alias),\n",
        "    globalThis[\"Math\"]['pow'](2, alias),"
    );

    assert_eq!(source, expected);
    assert_eq!(direct, expected);
    assert_eq!(direct_entries, expected_entries);
}

#[test]
fn test_set_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = set_constructor_aliases();
    let frozen_aliases = set_constructor_frozen_callable_aliases();
    let source = set_constructor_source();
    let iteration_source = set_constructor_iteration_source();
    let frozen_source = set_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Set",
            "globalThis.Set",
            r#"globalThis["Set"]"#,
            r#"globalThis['Set']"#
        ]
    );
    assert_eq!(
        source,
        "Set; globalThis.Set; globalThis[\"Set\"]; globalThis['Set'];"
    );
    assert_eq!(
        frozen_aliases,
        &[
            r#"Object.freeze(Set)"#,
            r#"Object.freeze((Set))"#,
            r#"Object.freeze((null ?? Set))"#,
            r#"Object.freeze((true && Set))"#,
            r#"Object.freeze((false || Set))"#,
            r#"Object.freeze(globalThis.Set)"#,
            r#"Object.freeze((globalThis.Set))"#,
            r#"Object.freeze((null ?? globalThis.Set))"#,
            r#"Object.freeze((true && globalThis.Set))"#,
            r#"Object.freeze((false || globalThis.Set))"#,
            r#"Object.freeze(globalThis["Set"])"#,
            r#"Object.freeze((globalThis["Set"]))"#,
            r#"Object.freeze((null ?? globalThis["Set"]))"#,
            r#"Object.freeze((true && globalThis["Set"]))"#,
            r#"Object.freeze((false || globalThis["Set"]))"#,
            r#"Object.freeze(globalThis['Set'])"#,
            r#"Object.freeze((globalThis['Set']))"#,
            r#"Object.freeze((null ?? globalThis['Set']))"#,
            r#"Object.freeze((true && globalThis['Set']))"#,
            r#"Object.freeze((false || globalThis['Set']))"#,
        ]
    );
    assert_eq!(
        iteration_source,
        concat!(
            "for (const value of new Set([1, 2, 1])) { console.log(value); } ",
            "for (const value of new Set(Object.freeze([1, 2, 1]))) { console.log(value); } ",
            "for (const value of new globalThis.Set([1, 2, 1])) { console.log(value); } ",
            "for (const value of new globalThis[\"Set\"]([1, 2, 1])) { console.log(value); } ",
            "for (const value of new globalThis['Set']([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new globalThis['Set'](Object.freeze([1, 2, 1]))) { console.log(value); } ",
            "for (const value of new (Object.freeze((Set)))([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (Object.freeze((globalThis.Set)))([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (Object.freeze((globalThis[\"Set\"])))([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (Object.freeze((globalThis['Set'])))([1, 2, 1])) { console.log(value); } ",
            "for (const value of Object.freeze(new Set([1, 2, 1]))) { console.log(value); } ",
            "for (const value of Object.freeze((new Set([1, 2, 1])))) { console.log(value); } ",
            "for (const value of Object.freeze((null ?? new Set([1, 2, 1])))) { console.log(value); } ",
            "for (const value of Object.freeze((true && new Set([1, 2, 1])))) { console.log(value); } ",
            "for (const value of Object.freeze((false || new Set([1, 2, 1])))) { console.log(value); } ",
            "for (const value of Object.freeze(new globalThis[\"Set\"]([1, 2, 1]))) { console.log(value); } ",
            "for (const value of Object.freeze((new globalThis[\"Set\"]([1, 2, 1])))) { console.log(value); } ",
            "for (const value of new (null ?? Set)([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (true && Set)([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (false || Set)([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (null ?? globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (true && globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (false || globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (null ?? globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (true && globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (false || globalThis['Set'])([1, 2, 1])) { console.log(value); }"
        )
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Set); Object.freeze((Set)); Object.freeze((null ?? Set)); ",
            "Object.freeze((true && Set)); Object.freeze((false || Set)); Object.freeze(globalThis.Set); ",
            "Object.freeze((globalThis.Set)); Object.freeze((null ?? globalThis.Set)); ",
            "Object.freeze((true && globalThis.Set)); Object.freeze((false || globalThis.Set)); ",
            "Object.freeze(globalThis[\"Set\"]); Object.freeze((globalThis[\"Set\"])); ",
            "Object.freeze((null ?? globalThis[\"Set\"])); Object.freeze((true && globalThis[\"Set\"])); ",
            "Object.freeze((false || globalThis[\"Set\"])); Object.freeze(globalThis['Set']); ",
            "Object.freeze((globalThis['Set'])); Object.freeze((null ?? globalThis['Set'])); ",
            "Object.freeze((true && globalThis['Set'])); Object.freeze((false || globalThis['Set']));"
        )
    );
}

#[test]
fn test_map_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = map_constructor_aliases();
    let frozen_aliases = map_constructor_frozen_callable_aliases();
    let source = map_constructor_source();
    let iteration_source = map_constructor_iteration_source();
    let frozen_source = map_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Map",
            "globalThis.Map",
            r#"globalThis["Map"]"#,
            r#"globalThis['Map']"#
        ]
    );
    assert_eq!(
        source,
        "Map; globalThis.Map; globalThis[\"Map\"]; globalThis['Map'];"
    );
    assert_eq!(
        frozen_aliases,
        &[
            r#"Object.freeze(Map)"#,
            r#"Object.freeze((Map))"#,
            r#"Object.freeze((null ?? Map))"#,
            r#"Object.freeze((true && Map))"#,
            r#"Object.freeze((false || Map))"#,
            r#"Object.freeze(globalThis.Map)"#,
            r#"Object.freeze((globalThis.Map))"#,
            r#"Object.freeze((null ?? globalThis.Map))"#,
            r#"Object.freeze((true && globalThis.Map))"#,
            r#"Object.freeze((false || globalThis.Map))"#,
            r#"Object.freeze(globalThis["Map"])"#,
            r#"Object.freeze((globalThis["Map"]))"#,
            r#"Object.freeze((null ?? globalThis["Map"]))"#,
            r#"Object.freeze((true && globalThis["Map"]))"#,
            r#"Object.freeze((false || globalThis["Map"]))"#,
            r#"Object.freeze(globalThis['Map'])"#,
            r#"Object.freeze((globalThis['Map']))"#,
            r#"Object.freeze((null ?? globalThis['Map']))"#,
            r#"Object.freeze((true && globalThis['Map']))"#,
            r#"Object.freeze((false || globalThis['Map']))"#,
        ]
    );
    assert_eq!(
        iteration_source,
        concat!(
            "for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis.Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis['Map']([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis['Map'](Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((globalThis.Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((globalThis[\"Map\"])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((globalThis['Map'])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze((null ?? new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze((true && new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze((false || new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze(new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of Object.freeze((new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (null ?? Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (true && Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (false || Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (null ?? globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (true && globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (false || globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (null ?? globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (true && globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (false || globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }"
        )
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Map); Object.freeze((Map)); Object.freeze((null ?? Map)); ",
            "Object.freeze((true && Map)); Object.freeze((false || Map)); Object.freeze(globalThis.Map); ",
            "Object.freeze((globalThis.Map)); Object.freeze((null ?? globalThis.Map)); ",
            "Object.freeze((true && globalThis.Map)); Object.freeze((false || globalThis.Map)); ",
            "Object.freeze(globalThis[\"Map\"]); Object.freeze((globalThis[\"Map\"])); ",
            "Object.freeze((null ?? globalThis[\"Map\"])); Object.freeze((true && globalThis[\"Map\"])); ",
            "Object.freeze((false || globalThis[\"Map\"])); Object.freeze(globalThis['Map']); ",
            "Object.freeze((globalThis['Map'])); Object.freeze((null ?? globalThis['Map'])); ",
            "Object.freeze((true && globalThis['Map'])); Object.freeze((false || globalThis['Map']));"
        )
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
