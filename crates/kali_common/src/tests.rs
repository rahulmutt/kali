use super::*;

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
