use super::*;

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
