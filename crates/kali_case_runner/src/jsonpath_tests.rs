use super::*;

// `toml::Value: FromStr` (`.parse()`) parses a single inline TOML *value*
// expression, not a document -- it rejects top-level `key = value` pairs
// with "unexpected content, expected nothing". `toml::from_str` is the
// document parser and is what every case-file line in these tests needs.
fn toml_of(text: &str) -> toml::Value {
    toml::from_str(text).expect("toml")
}

#[test]
fn a_nested_table_flattens_to_dotted_leaf_paths() {
    let table = toml_of(
        r#"
schemaVersion = 1
[payload]
artifactKind = "bundle"
bundleFormat = "esm"
"#,
    );
    let mut pairs = flatten_expected(&table);
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let paths: Vec<&str> = pairs.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "payload.artifactKind",
            "payload.bundleFormat",
            "schemaVersion"
        ]
    );
}

// An empty array is a leaf, not a table to recurse into. `json.errors = []` is a
// real assertion used 245 times in the suite being migrated.
#[test]
fn an_empty_array_is_a_leaf() {
    let table = toml_of("errors = []");
    let pairs = flatten_expected(&table);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "errors");
    assert!(pairs[0].1.as_array().expect("array").is_empty());
}

// An empty table is a leaf too, for the same reason as an empty array:
// `payload.diagnostics = {}` asserts "this is an empty object". Recursing
// into it instead would contribute zero paths, leaving `check_json` nothing
// to iterate and the assertion passing vacuously. §5.6's worked example
// spells `fields` this way.
#[test]
fn a_nested_empty_table_is_a_leaf() {
    let table = toml_of("[payload]\n[payload.diagnostics]\n");
    let pairs = flatten_expected(&table);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "payload.diagnostics");
    assert_eq!(pairs[0].1.as_table().expect("table").len(), 0);
}

// The top-level table itself can be empty (`json = {}`), which must flatten
// to a single leaf at the empty path, not zero leaves.
#[test]
fn a_top_level_empty_table_is_a_single_leaf_at_the_empty_path() {
    let table = toml_of("");
    let pairs = flatten_expected(&table);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "");
    assert_eq!(pairs[0].1.as_table().expect("table").len(), 0);
}

#[test]
fn lookup_walks_a_dotted_path() {
    let actual: serde_json::Value = serde_json::json!({"payload": {"artifactKind": "bundle"}});
    assert_eq!(
        lookup(&actual, "payload.artifactKind").and_then(|v| v.as_str()),
        Some("bundle")
    );
    assert!(lookup(&actual, "payload.missing").is_none());
    assert!(lookup(&actual, "absent.deeper").is_none());
}

// An empty path addresses the whole document -- this is what
// `flatten_expected` emits for a top-level `json = {}`. `"".split('.')`
// would otherwise look for a JSON key literally named `""`, which is never
// what an empty-table expectation means.
#[test]
fn lookup_with_an_empty_path_returns_the_whole_document() {
    let actual: serde_json::Value = serde_json::json!({"a": 1});
    assert_eq!(lookup(&actual, ""), Some(&actual));
}

#[test]
fn values_equal_matches_across_toml_and_json_types() {
    assert!(values_equal(
        &toml::Value::Integer(1),
        &serde_json::json!(1)
    ));
    assert!(values_equal(
        &toml::Value::Boolean(true),
        &serde_json::json!(true)
    ));
    assert!(values_equal(
        &toml::Value::String("bundle".into()),
        &serde_json::json!("bundle")
    ));
    assert!(values_equal(
        &toml::Value::Array(vec![]),
        &serde_json::json!([])
    ));
    assert!(!values_equal(
        &toml::Value::Integer(1),
        &serde_json::json!("1")
    ));
    assert!(!values_equal(
        &toml::Value::Integer(1),
        &serde_json::json!(2)
    ));
}

// TOML has no null; JSON does. An expected empty string must not match null.
#[test]
fn a_json_null_matches_nothing_expressible_in_toml() {
    assert!(!values_equal(
        &toml::Value::String(String::new()),
        &serde_json::Value::Null
    ));
}
