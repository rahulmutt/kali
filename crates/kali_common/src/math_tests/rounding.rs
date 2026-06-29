use super::*;

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
