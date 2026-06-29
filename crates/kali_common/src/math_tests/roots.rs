use super::*;

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
