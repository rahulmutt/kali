use crate::*;

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
