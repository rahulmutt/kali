use crate::*;

const BROADER_INTL_SEGMENTS: &[&str] = &[
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
];

/// Canonical broader `Intl` aliases used by the browser and runtime smoke.
pub fn broader_intl_aliases() -> &'static [&'static str] {
    BROADER_INTL_SEGMENTS
}

/// Canonical broader `Intl` source text used by the browser and runtime smoke.
pub fn broader_intl_source() -> String {
    join_semicolon_terminated_segments(broader_intl_aliases())
}

#[cfg(test)]
#[path = "intl_tests.rs"]
mod intl_tests;
