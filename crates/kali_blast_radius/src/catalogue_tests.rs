use super::*;

use crate::parse_register;

const SAMPLE: &str = r#"{
  "entries": [
    { "id": "R-13", "kind": "countable",
      "matcher": "computedMemberNonLiteralKey",
      "description": "computed member access whose key expression is not a literal" },
    { "id": "R-16", "kind": "uncountable",
      "reason": "a representation condition (handle leaks in concat position), not a syntactic shape" }
  ]
}"#;

#[test]
fn parses_both_predicate_kinds() {
    let entries = parse_catalogue(SAMPLE).expect("sample parses");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, "R-13");
    match &entries[0].predicate {
        Predicate::Countable { matcher, .. } => assert_eq!(matcher, "computedMemberNonLiteralKey"),
        Predicate::Uncountable { .. } => panic!("R-13 is countable"),
    }
    match &entries[1].predicate {
        Predicate::Uncountable { reason } => assert!(!reason.is_empty()),
        Predicate::Countable { .. } => panic!("R-16 is uncountable"),
    }
}

#[test]
fn an_uncountable_entry_with_an_empty_reason_is_rejected() {
    let json = r#"{"entries":[{"id":"R-16","kind":"uncountable","reason":"  "}]}"#;
    let error = parse_catalogue(json).expect_err("an unexplained uncountable must be rejected");
    assert!(error.contains("R-16"), "error names the entry: {error}");
}

/// A record that claims both kinds at once has not decided which it is, and
/// whichever half the parser silently ignored would be an unreviewed judgment.
#[test]
fn a_countable_entry_that_also_sets_a_reason_is_rejected() {
    let json = r#"{"entries":[{"id":"R-13","kind":"countable","matcher":"m",
                   "description":"d","reason":"and also uncountable"}]}"#;
    let error = parse_catalogue(json).expect_err("a record cannot be both kinds");
    assert!(error.contains("R-13"), "error names the entry: {error}");
    assert!(
        error.contains("reason"),
        "error names the offending field: {error}"
    );
}

#[test]
fn an_uncountable_entry_that_also_sets_a_matcher_is_rejected() {
    let json = r#"{"entries":[{"id":"R-21","kind":"uncountable","reason":"r","matcher":"m"}]}"#;
    let error = parse_catalogue(json).expect_err("an uncountable cannot name a matcher");
    assert!(error.contains("R-21"), "error names the entry: {error}");
    assert!(
        error.contains("matcher"),
        "error names the offending field: {error}"
    );
}

/// A typo'd key must fail the load rather than produce a record that measures
/// nothing -- `matchers` here would leave the entry with no matcher at all.
#[test]
fn an_unknown_field_is_rejected_rather_than_ignored() {
    let json = r#"{"entries":[{"id":"R-13","kind":"countable","matchers":"m","description":"d"}]}"#;
    let error = parse_catalogue(json).expect_err("an unknown field must fail the load");
    assert!(
        error.contains("matchers"),
        "error names the unknown field: {error}"
    );
}

#[test]
fn completeness_rejects_a_register_entry_with_no_catalogue_record() {
    let register = vec![
        RegisterEntry {
            id: "R-13".into(),
            tier: 2,
            title: "t".into(),
        },
        RegisterEntry {
            id: "R-99".into(),
            tier: 2,
            title: "t".into(),
        },
    ];
    let catalogue = parse_catalogue(SAMPLE).expect("sample parses");
    let error = check_completeness(&register, &catalogue)
        .expect_err("a missing record must fail, not default to uncountable");
    assert!(error.contains("R-99"), "error names the gap: {error}");
}

#[test]
fn completeness_rejects_a_catalogue_record_for_an_unknown_entry() {
    let register = vec![RegisterEntry {
        id: "R-13".into(),
        tier: 2,
        title: "t".into(),
    }];
    let catalogue = parse_catalogue(SAMPLE).expect("sample parses");
    let error = check_completeness(&register, &catalogue)
        .expect_err("a stale record must fail so the catalogue cannot drift");
    assert!(
        error.contains("R-16"),
        "error names the stale record: {error}"
    );
}

#[test]
fn the_shipped_catalogue_covers_the_real_register_exactly() {
    let register_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/superpowers/followups/kali-silent-miscompile-register.md"
    ))
    .expect("the register is readable");
    let catalogue_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius/predicates.json"
    ))
    .expect("the catalogue is readable");
    let register = parse_register(&register_text).expect("register parses");
    let catalogue = parse_catalogue(&catalogue_text).expect("catalogue parses");
    check_completeness(&register, &catalogue).expect("catalogue covers the register exactly");
    assert_eq!(
        catalogue.len(),
        42,
        "expected exactly 42 catalogue records (§2's tier-ranked total from the register's \
         numbering note, R-50 excluded); this count must be updated deliberately when §2 gains \
         an entry, and a 0 here would be a ran-nothing-green, got {}",
        catalogue.len()
    );
}
