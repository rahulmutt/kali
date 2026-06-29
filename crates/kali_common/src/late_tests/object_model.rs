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
