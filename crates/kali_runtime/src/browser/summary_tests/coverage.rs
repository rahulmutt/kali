use super::*;

// --- coverage_hits summary-parser tests (throw-fallout Stage 3 Task 2 review fix) ---
//
// These exercise `parse_coverage_hits_field` (summary.rs:411-422, called from
// the `"coverageHits"` arm of `parse_browser_runtime_summary_value`,
// summary.rs:440-451) directly against real `serde_json::Value` input — no
// mocks, no subprocess. They close the gap the reviewer flagged: the two
// pinned bucket-H tests only assert on a static wasm-section scan
// (`functionsTotal`), so they never exercise whether `coverageHits` actually
// parses into `BrowserRuntimeSummary.coverage_hits`. A regression here would
// have gone undetected by every other test in this module, since none of them
// set `coverageHits` in their fixture JSON.

/// A base summary JSON object missing only the `coverageHits` field, used so
/// each test only needs to vary that one field.
fn base_summary_object() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
        "args": ["alpha"],
        "tests": ["coverage test"],
        "testsFailed": 0,
        "hostContract": "browser-requested",
        "runtimeBackend": "browser-harness",
    })
    .as_object()
    .expect("base summary object literal")
    .clone()
}

#[test]
fn parse_browser_runtime_summary_value_populates_coverage_hits_from_a_valid_array() {
    let mut object = base_summary_object();
    object.insert(
        "coverageHits".to_string(),
        serde_json::json!([0, 3, 7, 4294967295u32]),
    );
    let value = serde_json::Value::Object(object);

    let summary =
        parse_browser_runtime_summary_value(&value).expect("valid coverageHits must parse");

    assert_eq!(
        summary.coverage_hits,
        Some(vec![0, 3, 7, 4294967295]),
        "coverageHits array must populate BrowserRuntimeSummary.coverage_hits verbatim"
    );
}

#[test]
fn parse_browser_runtime_summary_value_treats_a_missing_coverage_hits_field_as_none() {
    let object = base_summary_object();
    let value = serde_json::Value::Object(object);

    let summary = parse_browser_runtime_summary_value(&value)
        .expect("summary without coverageHits key must still parse");

    assert_eq!(summary.coverage_hits, None);
}

#[test]
fn parse_browser_runtime_summary_value_accepts_an_empty_coverage_hits_array() {
    let mut object = base_summary_object();
    object.insert("coverageHits".to_string(), serde_json::json!([]));
    let value = serde_json::Value::Object(object);

    let summary =
        parse_browser_runtime_summary_value(&value).expect("empty coverageHits array must parse");

    assert_eq!(summary.coverage_hits, Some(vec![]));
}

#[test]
fn parse_browser_runtime_summary_value_fails_closed_when_coverage_hits_is_not_an_array() {
    let mut object = base_summary_object();
    object.insert(
        "coverageHits".to_string(),
        serde_json::json!("not-an-array"),
    );
    let value = serde_json::Value::Object(object);

    assert!(
        parse_browser_runtime_summary_value(&value).is_none(),
        "a non-array coverageHits must reject the whole summary, matching the strict-parse \
         discipline every other field in this module uses (e.g. testsFailed with a non-numeric \
         type)"
    );
}

#[test]
fn parse_browser_runtime_summary_value_fails_closed_when_coverage_hits_has_a_negative_element() {
    let mut object = base_summary_object();
    object.insert("coverageHits".to_string(), serde_json::json!([1, -1]));
    let value = serde_json::Value::Object(object);

    assert!(
        parse_browser_runtime_summary_value(&value).is_none(),
        "a negative coverageHits element must reject the whole summary"
    );
}

#[test]
fn parse_browser_runtime_summary_value_fails_closed_when_coverage_hits_has_a_non_integer_element() {
    let mut object = base_summary_object();
    object.insert("coverageHits".to_string(), serde_json::json!([1, 2.5]));
    let value = serde_json::Value::Object(object);

    assert!(
        parse_browser_runtime_summary_value(&value).is_none(),
        "a non-integer coverageHits element must reject the whole summary"
    );
}

#[test]
fn parse_browser_runtime_summary_value_fails_closed_when_coverage_hits_element_exceeds_u32_max() {
    let mut object = base_summary_object();
    object.insert(
        "coverageHits".to_string(),
        serde_json::json!([1, 4294967296u64]),
    );
    let value = serde_json::Value::Object(object);

    assert!(
        parse_browser_runtime_summary_value(&value).is_none(),
        "a coverageHits element beyond u32::MAX must reject the whole summary"
    );
}

#[test]
fn parse_browser_runtime_summary_value_fails_closed_when_coverage_hits_has_a_string_element() {
    let mut object = base_summary_object();
    object.insert("coverageHits".to_string(), serde_json::json!([1, "2"]));
    let value = serde_json::Value::Object(object);

    assert!(
        parse_browser_runtime_summary_value(&value).is_none(),
        "a string coverageHits element must reject the whole summary"
    );
}

// --- parse_coverage_hits_field direct tests ---

#[test]
fn parse_coverage_hits_field_parses_a_valid_array() {
    let value = serde_json::json!([0, 1, 2]);
    assert_eq!(parse_coverage_hits_field(Some(&value)), Some(vec![0, 1, 2]));
}

#[test]
fn parse_coverage_hits_field_returns_none_for_a_missing_value() {
    assert_eq!(parse_coverage_hits_field(None), None);
}

#[test]
fn parse_coverage_hits_field_returns_none_for_a_non_array_value() {
    let value = serde_json::json!(42);
    assert_eq!(parse_coverage_hits_field(Some(&value)), None);
}

#[test]
fn parse_coverage_hits_field_returns_none_for_an_out_of_range_element() {
    let value = serde_json::json!([u64::MAX]);
    assert_eq!(parse_coverage_hits_field(Some(&value)), None);
}

// --- end-to-end merge test through the stdout/file merge path ---

#[test]
fn browser_runtime_summary_merges_missing_coverage_hits_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness","coverageHits":[5,9]}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(
        summary.coverage_hits,
        Some(vec![5, 9]),
        "coverageHits missing from the summary file must fall back to stdout's coverageHits, \
         mirroring the existing thread_topology merge behavior"
    );
}

#[test]
fn browser_runtime_summary_keeps_coverage_hits_from_summary_file_over_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness","coverageHits":[1,2,3]}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness","coverageHits":[9]}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.coverage_hits, Some(vec![1, 2, 3]));
}
