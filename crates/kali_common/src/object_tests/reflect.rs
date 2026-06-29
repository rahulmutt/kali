use super::*;

#[test]
fn test_reflect_own_keys_frozen_callable_aliases_list_all_aliases_in_order() {
    let aliases = reflect_own_keys_frozen_callable_aliases();

    for alias in [
        r#"Object.freeze(Reflect.ownKeys)"#,
        r#"Object.freeze((Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis["Reflect"].ownKeys)"#,
        r#"Object.freeze((globalThis.Reflect)["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"]).ownKeys)"#,
        r#"Object.freeze((globalThis['Reflect']).ownKeys)"#,
        r#"Object.freeze((globalThis["Reflect"])["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"].ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect.ownKeys)"#,
        r#"Object.freeze((null ?? globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((true && globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((false || globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect["ownKeys"])"#,
        r#"Object.freeze((globalThis.Reflect["ownKeys"]))"#,
        r#"Object.freeze(globalThis.Reflect['ownKeys'])"#,
        r#"Object.freeze((globalThis.Reflect['ownKeys']))"#,
        r#"Object.freeze(globalThis['Reflect']['ownKeys'])"#,
        r#"Object.freeze(globalThis['Reflect'].ownKeys)"#,
        r#"Object.freeze((globalThis['Reflect'].ownKeys))"#,
        r#"Object.freeze((globalThis['Reflect'])['ownKeys'])"#,
        r#"Object.freeze((null ?? Reflect.ownKeys))"#,
        r#"Object.freeze((true && Reflect.ownKeys))"#,
        r#"Object.freeze((false || Reflect.ownKeys))"#,
        r#"Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))"#,
        r#"Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((true && globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((false || globalThis["Reflect"]["ownKeys"]))"#,
    ] {
        assert!(aliases.contains(&alias), "missing alias: {alias}");
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Reflect.ownKeys frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
}

#[test]
fn test_reflect_own_keys_frozen_callable_source_lists_all_aliases_in_order() {
    let source = reflect_own_keys_frozen_callable_source("obj");

    for expected in [
        r#"const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(obj)"#,
        r#"const frozenParenthesizedSingleQuotedBracketRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)(obj)"#,
        r#"const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj)"#,
        r#"const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])(obj)"#,
        r#"const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"]['ownKeys']))(obj)"#,
        r#"const mixedSingleQuotedRootKeys = Object.freeze(globalThis['Reflect']["ownKeys"])(obj)"#,
        r#"const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']["ownKeys"]))(obj)"#,
        r#"const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj)"#,
        r#"const frozenSingleQuotedMixedBracketedKeys = Object.freeze(globalThis.Reflect['ownKeys'])(obj)"#,
        r#"const parenthesizedFrozenSingleQuotedMixedBracketedKeys = Object.freeze((globalThis.Reflect['ownKeys']))(obj)"#,
        r#"const nullishFrozenCallableKeys = Object.freeze((null ?? globalThis.Reflect.ownKeys))(obj)"#,
        r#"const logicalAndFrozenCallableKeys = Object.freeze((true && globalThis.Reflect.ownKeys))(obj)"#,
        r#"const logicalOrFrozenCallableKeys = Object.freeze((false || globalThis.Reflect.ownKeys))(obj)"#,
        r#"const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)(obj)"#,
        r#"const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])(obj)"#,
        r#"const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(obj)"#,
        r#"const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj)"#,
        r#"const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))(obj)"#,
        r#"const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj)"#,
        r#"const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj)"#,
        r#"const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj)"#,
        r#"const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj)"#,
        r#"const frozenSingleQuotedRootKeys = Object.freeze(globalThis['Reflect'].ownKeys)(obj)"#,
        r#"const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)(obj)"#,
        r#"const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(obj)"#,
        r#"const frozenSingleQuotedBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])(obj)"#,
        r#"const parenthesizedFrozenSingleQuotedRootKeys = Object.freeze((globalThis['Reflect'].ownKeys))(obj)"#,
        r#"const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(obj)"#,
        r#"const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj)"#,
        r#"const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj)"#,
        r#"const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj)"#,
        r#"const conditionalFrozenCallableKeys = Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))(obj)"#,
        r#"const conditionalFrozenGlobalCallableKeys = Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))(obj)"#,
        r#"const nullishFrozenBracketedKeys = Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))(obj)"#,
        r#"const logicalAndFrozenBracketedKeys = Object.freeze((true && globalThis["Reflect"]["ownKeys"]))(obj)"#,
        r#"const logicalOrFrozenBracketedKeys = Object.freeze((false || globalThis["Reflect"]["ownKeys"]))(obj)"#,
    ] {
        assert!(
            source.contains(expected),
            "missing source fragment: {expected}"
        );
    }
}
