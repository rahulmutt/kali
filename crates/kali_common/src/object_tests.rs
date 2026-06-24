use crate::*;

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

#[test]
fn test_object_has_own_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_frozen_callable_aliases();
    let source = object_has_own_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(Object.hasOwn)"#,
        r#"Object.freeze((Object.hasOwn))"#,
        r#"Object.freeze(Object["hasOwn"])"#,
        r#"Object.freeze((Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis.Object.hasOwn)"#,
        r#"Object.freeze((globalThis.Object.hasOwn))"#,
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze(globalThis.Object['hasOwn'])"#,
        r#"Object.freeze((globalThis.Object)["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object).hasOwn)"#,
        r#"Object.freeze((globalThis.Object)['hasOwn'])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis?.Object.hasOwn)"#,
        r#"Object.freeze((globalThis?.Object.hasOwn))"#,
        r#"Object.freeze((globalThis?.Object).hasOwn)"#,
        r#"Object.freeze((globalThis?.Object)["hasOwn"])"#,
        r#"Object.freeze(globalThis?.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis?.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwn)"#,
        r#"Object.freeze((globalThis["Object"].hasOwn))"#,
        r#"Object.freeze((globalThis["Object"]).hasOwn)"#,
        r#"Object.freeze((globalThis["Object"])["hasOwn"])"#,
        r#"Object.freeze(globalThis["Object"]["hasOwn"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwn"]))"#,
        r#"Object.freeze((globalThis["Object"]))["hasOwn"]"#,
        r#"Object.freeze((globalThis["Object"]))['hasOwn']"#,
        r#"Object.freeze((globalThis['Object']))["hasOwn"]"#,
        r#"Object.freeze((globalThis['Object']))['hasOwn']"#,
        r#"Object.freeze(globalThis['Object'].hasOwn)"#,
        r#"Object.freeze((globalThis['Object'].hasOwn))"#,
        r#"Object.freeze((globalThis['Object']).hasOwn)"#,
        r#"Object.freeze((globalThis['Object'])['hasOwn'])"#,
        r#"Object.freeze(globalThis['Object']['hasOwn'])"#,
        r#"Object.freeze((globalThis['Object']['hasOwn']))"#,
        r#"Object.freeze((null ?? Object.hasOwn))"#,
        r#"Object.freeze((true && Object.hasOwn))"#,
        r#"Object.freeze((false || Object.hasOwn))"#,
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
            "duplicate alias in Object.hasOwn frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_object_enumeration_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_enumeration_frozen_callable_aliases();
    let source = object_enumeration_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze((globalThis["Object"]).keys)"#,
        r#"Object.freeze((globalThis["Object"])["keys"])"#,
        r#"Object.freeze((globalThis["Object"]).values)"#,
        r#"Object.freeze((globalThis["Object"])["values"])"#,
        r#"Object.freeze((globalThis["Object"]).entries)"#,
        r#"Object.freeze((globalThis["Object"])["entries"])"#,
        r#"Object.freeze((globalThis["Object"]["keys"]))"#,
        r#"Object.freeze((globalThis["Object"]))["keys"]"#,
        r#"Object.freeze((globalThis['Object']))['keys']"#,
        r#"Object.freeze((globalThis["Object"]["values"]))"#,
        r#"Object.freeze((globalThis["Object"]))["values"]"#,
        r#"Object.freeze((globalThis['Object']))['values']"#,
        r#"Object.freeze((globalThis["Object"]["entries"]))"#,
        r#"Object.freeze((globalThis["Object"]))["entries"]"#,
        r#"Object.freeze((globalThis['Object']))['entries']"#,
        r#"Object.freeze((globalThis['Object']).keys)"#,
        r#"Object.freeze((globalThis['Object'])['keys'])"#,
        r#"Object.freeze((globalThis['Object'])["keys"])"#,
        r#"Object.freeze((globalThis['Object']).values)"#,
        r#"Object.freeze((globalThis['Object'])['values'])"#,
        r#"Object.freeze((globalThis['Object'])["values"])"#,
        r#"Object.freeze((globalThis['Object']).entries)"#,
        r#"Object.freeze((globalThis['Object'])['entries'])"#,
        r#"Object.freeze((globalThis['Object'])["entries"])"#,
        r#"Object.freeze(globalThis["Object"]["keys"])"#,
        r#"Object.freeze(globalThis["Object"]['keys'])"#,
        r#"Object.freeze(globalThis["Object"]["values"])"#,
        r#"Object.freeze(globalThis["Object"]["entries"])"#,
        r#"Object.freeze(globalThis.Object['keys'])"#,
        r#"Object.freeze(globalThis.Object['values'])"#,
        r#"Object.freeze(globalThis.Object['entries'])"#,
        r#"Object.freeze(globalThis.Object["keys"])"#,
        r#"Object.freeze(globalThis.Object["values"])"#,
        r#"Object.freeze(globalThis.Object["entries"])"#,
        r#"Object.freeze((globalThis.Object["keys"]))"#,
        r#"Object.freeze((globalThis.Object)['keys'])"#,
        r#"Object.freeze((globalThis.Object['keys']))"#,
        r#"Object.freeze((globalThis.Object["values"]))"#,
        r#"Object.freeze((globalThis.Object)['values'])"#,
        r#"Object.freeze((globalThis.Object['values']))"#,
        r#"Object.freeze((globalThis.Object["entries"]))"#,
        r#"Object.freeze((globalThis.Object)['entries'])"#,
        r#"Object.freeze((globalThis.Object['entries']))"#,
        r#"Object.freeze(globalThis['Object']["keys"])"#,
        r#"Object.freeze(globalThis['Object']["values"])"#,
        r#"Object.freeze(globalThis['Object']["entries"])"#,
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
            "duplicate alias in Object.keys/values/entries frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_object_has_own_frozen_callable_condition_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_frozen_callable_aliases();
    let condition_source = object_has_own_frozen_callable_condition_source("wrapped", r#""a""#);
    let expected = aliases
        .iter()
        .map(|alias| format!("!{alias}(wrapped, \"a\")"))
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(condition_source, expected);
}

#[test]
fn test_object_has_own_combined_frozen_callable_condition_source_reuses_both_helpers_once() {
    let source = object_has_own_combined_frozen_callable_condition_source("wrapped", r#""a""#);
    assert_eq!(
        source,
        format!(
            "{} || {}",
            object_has_own_frozen_callable_condition_source("wrapped", r#""a""#),
            object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
        )
    );
    assert_eq!(
        source
            .matches(&object_has_own_frozen_callable_condition_source(
                "wrapped", r#""a""#
            ))
            .count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(
                &object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
            )
            .count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_object_has_own_property_call_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_property_call_frozen_callable_aliases();
    let source = object_has_own_property_call_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis['Object']).prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty']['call'])"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty']['call']))"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty'].call))"#,
        r#"Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((Object.prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze((null ?? Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty["call"]))"#,
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
            "duplicate alias in Object.prototype.hasOwnProperty.call frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_object_has_own_property_call_frozen_callable_condition_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_property_call_frozen_callable_aliases();
    let condition_source =
        object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#);
    let expected = aliases
        .iter()
        .map(|alias| format!("!{alias}(wrapped, \"a\")"))
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(condition_source, expected);
}

#[test]
fn test_object_has_own_property_call_binding_source_is_canonical() {
    let binding_source = object_has_own_property_call_binding_source("hasOwnPropertyCall");

    assert_eq!(
        binding_source,
        "const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;"
    );
}
