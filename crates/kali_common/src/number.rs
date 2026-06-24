use crate::*;

/// Canonical source text for the supported Number predicate slice.
pub fn number_predicates_preamble_source(alias_literal: &str) -> String {
    format!(
        "const alias = {alias_literal}; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number[\"isFinite\"]); const frozenBracketedNaN = Object.freeze(Number[\"isNaN\"]); const frozenBracketedInteger = Object.freeze(Number[\"isInteger\"]); const frozenBracketedSafeInteger = Object.freeze(Number[\"isSafeInteger\"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis[\"Number\"])[\"isFinite\"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis[\"Number\"])[\"isNaN\"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis[\"Number\"])[\"isInteger\"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis[\"Number\"])[\"isSafeInteger\"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis[\"Number\"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis[\"Number\"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis[\"Number\"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis[\"Number\"]).isSafeInteger);"
    )
}

/// Canonical console-log body for the supported Number predicate slice.
pub fn number_predicates_console_log_body_source() -> String {
    join_semicolon_terminated_segments(&[
        r#"console.log(Number.isFinite(alias))"#,
        r#"console.log(integer(alias))"#,
        r#"console.log(Number.isSafeInteger(alias))"#,
        r#"console.log(integer(1.5))"#,
        r#"console.log(Number.isFinite("hello"))"#,
        r#"console.log(Number.isSafeInteger(1.5))"#,
        r#"console.log(globalThis["Number"]["isNaN"](NaN))"#,
        r#"console.log(globalThis.Number.isNaN(1))"#,
        r#"console.log(globalThis["Number"].isNaN(1))"#,
        r#"console.log(globalThis["Number"]["isFinite"](alias))"#,
        r#"console.log(globalThis["Number"]["isInteger"](alias))"#,
        r#"console.log(globalThis["Number"]["isSafeInteger"](alias))"#,
        r#"console.log(globalThis.Number["isNaN"](1))"#,
        r#"console.log(globalThis["Number"].isFinite(alias))"#,
        r#"console.log(globalThis.Number["isInteger"](alias))"#,
        r#"console.log(globalThis["Number"].isSafeInteger(alias))"#,
        r#"console.log(Number["isFinite"](alias))"#,
        r#"console.log(Number["isInteger"](alias))"#,
        r#"console.log(Number["isSafeInteger"](alias))"#,
        r#"console.log(Number["isNaN"](1))"#,
        r#"console.log(frozenFinite(alias))"#,
        r#"console.log(frozenNaN(NaN))"#,
        r#"console.log(frozenNaN(1))"#,
        r#"console.log(frozenInteger(alias))"#,
        r#"console.log(frozenSafeInteger(alias))"#,
        r#"console.log(frozenBracketedFinite(alias))"#,
        r#"console.log(frozenBracketedNaN(NaN))"#,
        r#"console.log(frozenBracketedNaN(1))"#,
        r#"console.log(frozenBracketedInteger(alias))"#,
        r#"console.log(frozenBracketedSafeInteger(alias))"#,
        r#"console.log(frozenParenthesizedBracketedFinite(alias))"#,
        r#"console.log(frozenParenthesizedBracketedNaN(NaN))"#,
        r#"console.log(frozenParenthesizedBracketedNaN(1))"#,
        r#"console.log(frozenParenthesizedBracketedInteger(alias))"#,
        r#"console.log(frozenParenthesizedBracketedSafeInteger(alias))"#,
        r#"console.log(frozenParenthesizedPropertyFinite(alias))"#,
        r#"console.log(frozenParenthesizedPropertyNaN(NaN))"#,
        r#"console.log(frozenParenthesizedPropertyNaN(1))"#,
        r#"console.log(frozenParenthesizedPropertyInteger(alias))"#,
        r#"console.log(frozenParenthesizedPropertySafeInteger(alias))"#,
        r#"console.log(finite(alias))"#,
        r#"console.log(integer(alias))"#,
        r#"console.log(safeInteger(alias))"#,
    ])
}

/// Canonical runtime source text for the supported Number predicate slice.
pub fn number_predicates_runtime_source() -> String {
    format!(
        "{} {}",
        number_predicates_preamble_source("1"),
        number_predicates_console_log_body_source()
    )
}

/// Canonical browser-bundle source text for the supported Number predicate slice.
pub fn number_predicates_browser_bundle_source(alias_literal: &str) -> String {
    format!(
        concat!(
            "// kali-tree-shake: browserNumberPredicates\n",
            "async function browserNumberPredicates() {{\n",
            "  const alias = {};\n",
            "  const finite = Number.isFinite;\n",
            "  const integer = Number.isInteger;\n",
            "  const safeInteger = Number.isSafeInteger;\n",
            "  const frozenFinite = Object.freeze(Number.isFinite);\n",
            "  const frozenNaN = Object.freeze(Number.isNaN);\n",
            "  const frozenInteger = Object.freeze(Number.isInteger);\n",
            "  const frozenSafeInteger = Object.freeze(Number.isSafeInteger);\n",
            "  const frozenBracketedFinite = Object.freeze(Number[\"isFinite\"]);\n",
            "  const frozenBracketedNaN = Object.freeze(Number[\"isNaN\"]);\n",
            "  const frozenBracketedInteger = Object.freeze(Number[\"isInteger\"]);\n",
            "  const frozenBracketedSafeInteger = Object.freeze(Number[\"isSafeInteger\"]);\n",
            "  const frozenParenthesizedBracketedFinite = Object.freeze((globalThis[\"Number\"])[\"isFinite\"]);\n",
            "  const frozenParenthesizedBracketedNaN = Object.freeze((globalThis[\"Number\"])[\"isNaN\"]);\n",
            "  const frozenParenthesizedBracketedInteger = Object.freeze((globalThis[\"Number\"])[\"isInteger\"]);\n",
            "  const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis[\"Number\"])[\"isSafeInteger\"]);\n",
            "  const frozenParenthesizedPropertyFinite = Object.freeze((globalThis[\"Number\"]).isFinite);\n",
            "  const frozenParenthesizedPropertyNaN = Object.freeze((globalThis[\"Number\"]).isNaN);\n",
            "  const frozenParenthesizedPropertyInteger = Object.freeze((globalThis[\"Number\"]).isInteger);\n",
            "  const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis[\"Number\"]).isSafeInteger);\n",
            "  if (\n",
            "    Number.isFinite(alias) !== true ||\n",
            "    Number.isSafeInteger(await alias) !== true ||\n",
            "    integer(alias) !== true ||\n",
            "    Number.isSafeInteger(alias) !== true ||\n",
            "    integer(1.5) !== false ||\n",
            "    Number.isFinite(\"hello\") !== false ||\n",
            "    Number.isSafeInteger(1.5) !== false ||\n",
            "    globalThis[\"Number\"][\"isNaN\"](NaN) !== true ||\n",
            "    globalThis.Number.isNaN(1) !== false ||\n",
            "    globalThis[\"Number\"].isNaN(1) !== false ||\n",
            "    globalThis[\"Number\"][\"isFinite\"](alias) !== true ||\n",
            "    globalThis[\"Number\"][\"isInteger\"](alias) !== true ||\n",
            "    globalThis[\"Number\"][\"isSafeInteger\"](alias) !== true ||\n",
            "    globalThis.Number[\"isNaN\"](1) !== false ||\n",
            "    globalThis[\"Number\"].isFinite(alias) !== true ||\n",
            "    globalThis.Number[\"isInteger\"](alias) !== true ||\n",
            "    globalThis[\"Number\"].isSafeInteger(alias) !== true ||\n",
            "    Number[\"isFinite\"](alias) !== true ||\n",
            "    Number[\"isInteger\"](alias) !== true ||\n",
            "    Number[\"isSafeInteger\"](alias) !== true ||\n",
            "    Number[\"isNaN\"](1) !== false ||\n",
            "    frozenFinite(alias) !== true ||\n",
            "    frozenNaN(NaN) !== true ||\n",
            "    frozenNaN(1) !== false ||\n",
            "    frozenInteger(alias) !== true ||\n",
            "    frozenSafeInteger(alias) !== true ||\n",
            "    frozenBracketedFinite(alias) !== true ||\n",
            "    frozenBracketedNaN(NaN) !== true ||\n",
            "    frozenBracketedNaN(1) !== false ||\n",
            "    frozenBracketedInteger(alias) !== true ||\n",
            "    frozenBracketedSafeInteger(alias) !== true ||\n",
            "    frozenParenthesizedBracketedFinite(alias) !== true ||\n",
            "    frozenParenthesizedBracketedNaN(NaN) !== true ||\n",
            "    frozenParenthesizedBracketedNaN(1) !== false ||\n",
            "    frozenParenthesizedBracketedInteger(alias) !== true ||\n",
            "    frozenParenthesizedBracketedSafeInteger(alias) !== true ||\n",
            "    frozenParenthesizedPropertyFinite(alias) !== true ||\n",
            "    frozenParenthesizedPropertyNaN(NaN) !== true ||\n",
            "    frozenParenthesizedPropertyNaN(1) !== false ||\n",
            "    frozenParenthesizedPropertyInteger(alias) !== true ||\n",
            "    frozenParenthesizedPropertySafeInteger(alias) !== true ||\n",
            "    safeInteger(alias) !== true ||\n",
            "    finite(alias) !== true\n",
            "  ) {{\n",
            "    throw new Error('unexpected browser Number predicate result');\n",
            "  }}\n",
            "  console.log('browser number predicates ok');\n",
            "}}\n"
        ),
        alias_literal
    )
}

/// Canonical `Kali.test` source text for the supported Number predicate slice.
pub fn number_predicates_test_source() -> String {
    format!(
        "Kali.test('number predicates', () => {{ {} {} }});",
        number_predicates_preamble_source("1"),
        number_predicates_console_log_body_source()
    )
}

#[cfg(test)]
#[path = "number_tests.rs"]
mod number_tests;
