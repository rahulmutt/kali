use crate::*;

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
