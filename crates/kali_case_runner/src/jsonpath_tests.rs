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

// A numeric segment indexes into a JSON array -- this is the whole point of
// the feature: `errors.0.code` is how a case pins "the first diagnostic has
// this code" without asserting the rest of the (null-bearing, otherwise
// unmatchable) diagnostic object.
#[test]
fn lookup_indexes_into_an_array_with_a_numeric_segment() {
    let actual: serde_json::Value = serde_json::json!({"errors": [{"code": "E5506"}]});
    assert_eq!(
        lookup(&actual, "errors.0.code").and_then(|v| v.as_str()),
        Some("E5506")
    );
}

#[test]
fn lookup_rejects_an_out_of_range_array_index() {
    let actual: serde_json::Value = serde_json::json!({"errors": [{"code": "E5506"}]});
    assert!(lookup(&actual, "errors.5.code").is_none());
}

// Not a silent skip: a non-numeric segment against an array is a hard
// `None`, exactly like an absent key would be -- `errors.code` (forgetting
// the index) must fail loudly, not vacuously match nothing and pass.
#[test]
fn lookup_rejects_a_non_numeric_segment_into_an_array() {
    let actual: serde_json::Value = serde_json::json!({"errors": [1, 2, 3]});
    assert!(lookup(&actual, "errors.code").is_none());
}

// `usize` cannot represent a negative number, so "-1" fails to parse as an
// index exactly like any other non-numeric segment would. There is no
// negative-from-end indexing in this format.
#[test]
fn lookup_rejects_a_negative_looking_array_segment() {
    let actual: serde_json::Value = serde_json::json!({"errors": [1, 2, 3]});
    assert!(lookup(&actual, "errors.-1").is_none());
}

// A segment too large to fit a `usize` fails to parse, the same as any other
// invalid index -- not a panic, not a silent skip.
#[test]
fn lookup_rejects_an_index_segment_that_overflows_usize() {
    let actual: serde_json::Value = serde_json::json!({"errors": [1, 2, 3]});
    assert!(lookup(&actual, "errors.99999999999999999999999999999999999999").is_none());
}

// The numeric-key ambiguity, resolved and pinned: dispatch is by the actual
// JSON type of the node being indexed, not by whether the segment looks
// numeric. Against an *object*, a numeric-looking segment is an ordinary key
// lookup (an object with a literal key "0" behaves like any other object) --
// it is only ever treated as an index when the node is a JSON *array*. This
// is the less surprising rule: whether `.0` means "index" or "key" is a
// property of what's being addressed, not of how the path was spelled.
#[test]
fn a_numeric_looking_key_on_an_object_is_a_plain_key_not_an_index() {
    let actual: serde_json::Value = serde_json::json!({"payload": {"0": "x"}});
    assert_eq!(
        lookup(&actual, "payload.0").and_then(|v| v.as_str()),
        Some("x")
    );
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
