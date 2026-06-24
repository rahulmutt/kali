//! Common utilities shared across all Kali crates.
//!
//! This crate provides:
//! - String interning for identifiers and literals
//! - Source file registry with compact FileId
//! - Span type for source positions
//! - SourceMap for human-readable diagnostics

pub mod interner;
pub mod source_map;
pub mod span;
pub mod template;
mod helpers;

pub use interner::{InternedString, Interner};
pub use span::Span;
pub(crate) use helpers::*;
mod registry;
pub use registry::*;
mod messages;
pub use messages::*;
mod process_kill;
pub use process_kill::*;
mod object;
pub use object::*;
mod number;
pub use number::*;
mod math;
pub use math::*;
mod promise;
pub use promise::*;
mod array;
pub use array::*;
mod template_literal;
pub use template_literal::*;
mod collections;
pub use collections::*;
mod late;
pub use late::*;

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
#[path = "tests.rs"]
mod tests;
