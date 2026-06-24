use crate::*;

#[test]
fn test_number_predicates_source_helpers_are_canonical() {
    assert_eq!(
        number_predicates_preamble_source("1"),
        r#"const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);"#,
    );
    assert_eq!(
        number_predicates_preamble_source("1 as const"),
        r#"const alias = 1 as const; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);"#,
    );
    assert_eq!(
        number_predicates_console_log_body_source(),
        concat!(
            r#"console.log(Number.isFinite(alias)); "#,
            r#"console.log(integer(alias)); "#,
            r#"console.log(Number.isSafeInteger(alias)); "#,
            r#"console.log(integer(1.5)); "#,
            r#"console.log(Number.isFinite("hello")); "#,
            r#"console.log(Number.isSafeInteger(1.5)); "#,
            r#"console.log(globalThis["Number"]["isNaN"](NaN)); "#,
            r#"console.log(globalThis.Number.isNaN(1)); "#,
            r#"console.log(globalThis["Number"].isNaN(1)); "#,
            r#"console.log(globalThis["Number"]["isFinite"](alias)); "#,
            r#"console.log(globalThis["Number"]["isInteger"](alias)); "#,
            r#"console.log(globalThis["Number"]["isSafeInteger"](alias)); "#,
            r#"console.log(globalThis.Number["isNaN"](1)); "#,
            r#"console.log(globalThis["Number"].isFinite(alias)); "#,
            r#"console.log(globalThis.Number["isInteger"](alias)); "#,
            r#"console.log(globalThis["Number"].isSafeInteger(alias)); "#,
            r#"console.log(Number["isFinite"](alias)); "#,
            r#"console.log(Number["isInteger"](alias)); "#,
            r#"console.log(Number["isSafeInteger"](alias)); "#,
            r#"console.log(Number["isNaN"](1)); "#,
            r#"console.log(frozenFinite(alias)); "#,
            r#"console.log(frozenNaN(NaN)); "#,
            r#"console.log(frozenNaN(1)); "#,
            r#"console.log(frozenInteger(alias)); "#,
            r#"console.log(frozenSafeInteger(alias)); "#,
            r#"console.log(frozenBracketedFinite(alias)); "#,
            r#"console.log(frozenBracketedNaN(NaN)); "#,
            r#"console.log(frozenBracketedNaN(1)); "#,
            r#"console.log(frozenBracketedInteger(alias)); "#,
            r#"console.log(frozenBracketedSafeInteger(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedFinite(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedNaN(NaN)); "#,
            r#"console.log(frozenParenthesizedBracketedNaN(1)); "#,
            r#"console.log(frozenParenthesizedBracketedInteger(alias)); "#,
            r#"console.log(frozenParenthesizedBracketedSafeInteger(alias)); "#,
            r#"console.log(frozenParenthesizedPropertyFinite(alias)); "#,
            r#"console.log(frozenParenthesizedPropertyNaN(NaN)); "#,
            r#"console.log(frozenParenthesizedPropertyNaN(1)); "#,
            r#"console.log(frozenParenthesizedPropertyInteger(alias)); "#,
            r#"console.log(frozenParenthesizedPropertySafeInteger(alias)); "#,
            r#"console.log(finite(alias)); "#,
            r#"console.log(integer(alias)); "#,
            r#"console.log(safeInteger(alias));"#
        )
    );
    assert_eq!(
        number_predicates_runtime_source(),
        format!(
            "{} {}",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert_eq!(
        number_predicates_test_source(),
        format!(
            "Kali.test('number predicates', () => {{ {} {} }});",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert!(number_predicates_browser_bundle_source("1").starts_with(
        r#"// kali-tree-shake: browserNumberPredicates
async function browserNumberPredicates() {
  const alias = 1;"#
    ));
    assert!(
        number_predicates_browser_bundle_source("1 as const").contains("const alias = 1 as const;")
    );
    assert!(number_predicates_browser_bundle_source("1")
        .contains("Number.isSafeInteger(await alias) !== true"));
    assert!(number_predicates_browser_bundle_source("1").contains("Object.freeze(Number.isFinite)"));
    assert!(number_predicates_browser_bundle_source("1").contains("Object.freeze(Number.isNaN)"));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze(Number["isFinite"])"#));
    assert!(
        number_predicates_browser_bundle_source("1").contains(r#"Object.freeze(Number["isNaN"])"#)
    );
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"])["isFinite"])"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"])["isNaN"])"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isFinite)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isNaN)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isInteger)"#));
    assert!(number_predicates_browser_bundle_source("1")
        .contains(r#"Object.freeze((globalThis["Number"]).isSafeInteger)"#));
    assert!(number_predicates_browser_bundle_source("1").ends_with("}\n"));
}
