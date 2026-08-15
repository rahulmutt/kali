use super::*;
use crate::model::{CountBound, CountClaim, Exit, ExitStatusWord, JsonCountClaim, Step, StepKind};
use std::collections::BTreeMap;

fn blank_step() -> Step {
    Step {
        kind: StepKind::Cli,
        args: Vec::new(),
        env: BTreeMap::new(),
        exit: None,
        stdout: None,
        stdout_contains: Vec::new(),
        stdout_absent: Vec::new(),
        stdout_count: Vec::new(),
        stderr: None,
        stderr_contains: Vec::new(),
        stderr_absent: Vec::new(),
        json: None,
        json_null: Vec::new(),
        json_count: Vec::new(),
        path: None,
        fields: None,
        entry: None,
        body: None,
        register_entry: None,
        program: None,
        verdict: None,
        timeout_ms: None,
    }
}

fn captured(success: bool, code: i32, stdout: &str, stderr: &str) -> Captured {
    Captured {
        code: Some(code),
        success,
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    }
}

#[test]
fn exit_success_passes_on_success_and_fails_on_failure() {
    let mut step = blank_step();
    step.exit = Some(Exit::Status(ExitStatusWord::Success));
    check(&step, &captured(true, 0, "", "")).expect("a successful exit must pass");
    let err = check(&step, &captured(false, 1, "", "")).expect_err("must fail");
    assert!(err.contains("exit"), "{err}");
}

#[test]
fn an_exact_exit_code_must_match() {
    let mut step = blank_step();
    step.exit = Some(Exit::Code(2));
    check(&step, &captured(false, 2, "", "")).expect("exit code 2 must match `exit = 2`");
    let err = check(&step, &captured(false, 1, "", "")).expect_err("must fail");
    assert!(err.contains("code 2"), "{err}");
    assert!(err.contains("Some(1)"), "{err}");
}

#[test]
fn exact_stdout_must_match_byte_for_byte() {
    let mut step = blank_step();
    step.stdout = Some("hahaha\n\n".to_string());
    check(&step, &captured(true, 0, "hahaha\n\n", "")).expect("exact stdout match must pass");
    assert!(check(&step, &captured(true, 0, "hahaha\n", "")).is_err());
}

#[test]
fn contains_and_absent_are_both_enforced() {
    let mut step = blank_step();
    step.stdout_contains = vec!["1\n".to_string()];
    step.stdout_absent = vec!["E5506".to_string()];
    check(&step, &captured(true, 0, "1\n0\n", "")).expect("contains+absent claims must both hold");
    assert!(check(&step, &captured(true, 0, "0\n", "")).is_err());
    assert!(check(&step, &captured(true, 0, "1\nE5506", "")).is_err());
}

fn at_least(needle: &str, n: usize) -> CountClaim {
    CountClaim {
        needle: needle.to_string(),
        bound: CountBound::AtLeast(n),
    }
}

fn exactly(needle: &str, n: usize) -> CountClaim {
    CountClaim {
        needle: needle.to_string(),
        bound: CountBound::Exact(n),
    }
}

// `stdout_count`'s dominant shape (29 of the 32 migrated sites):
// `stdout.matches("3\n").count() >= 2`. The violated half is the reason the
// key exists at all -- `stdout_contains = ["3\n"]` would pass on the
// single-occurrence output below, silently weakening `count() >= 2` to
// `count() >= 1`.
#[test]
fn stdout_count_at_least_passes_at_the_bound_and_fails_below_it() {
    let mut step = blank_step();
    step.stdout_count = vec![at_least("3\n", 2)];
    check(&step, &captured(true, 0, "3\n3\n", "")).expect("exactly the bound must pass");
    check(&step, &captured(true, 0, "3\n3\n3\n", "")).expect("above the bound must pass");
    let err = check(&step, &captured(true, 0, "3\n", "")).expect_err("one occurrence must fail");
    assert!(err.contains("at least 2"), "{err}");
    assert!(err.contains("found 1"), "{err}");
    assert!(err.contains("\"3\\n\""), "must name the needle: {err}");
    assert!(err.contains("stdout"), "must name the surface: {err}");
}

// The `count() == n` shape (3 sites, e.g.
// `browser_math_unsupported_member_calls_harness_jsx_tsx.rs:107-112`). Unlike
// `at_least`, it must fail *above* the bound too -- that is the whole point
// of the spelling being distinct.
#[test]
fn stdout_count_exact_fails_both_below_and_above_the_bound() {
    let mut step = blank_step();
    step.stdout_count = vec![exactly("x", 2)];
    check(&step, &captured(true, 0, "x x", "")).expect("exactly two must pass");
    let low = check(&step, &captured(true, 0, "x", "")).expect_err("one must fail");
    assert!(low.contains("exactly 2"), "{low}");
    assert!(low.contains("found 1"), "{low}");
    let high = check(&step, &captured(true, 0, "x x x", "")).expect_err("three must fail");
    assert!(high.contains("found 3"), "{high}");
}

// THE load-bearing semantics test. Rust's `str::matches` is non-overlapping
// and left-to-right: `"aaa".matches("aa").count()` is 1, because the scan
// resumes *after* each match rather than one byte into it. Every claim
// migrated into this key was written against that behavior, so an
// implementation that counted overlapping occurrences would report 2 here and
// silently *strengthen* those claims -- passing on output the original
// assertion rejected.
//
// Both halves are needed. The `exact = 1` half fails under an overlapping
// counter (it would see 2); the `at_least = 2` half is the one that matters
// in practice, because an overlapping counter makes it pass on output that
// contains only one real, non-overlapping occurrence.
#[test]
fn count_is_non_overlapping_exactly_as_str_matches_is() {
    assert_eq!("aaa".matches("aa").count(), 1, "premise of this test");

    let mut step = blank_step();
    step.stdout_count = vec![exactly("aa", 1)];
    check(&step, &captured(true, 0, "aaa", ""))
        .expect("`aaa` holds one non-overlapping `aa`, not two");

    step.stdout_count = vec![exactly("aa", 2)];
    assert!(
        check(&step, &captured(true, 0, "aaa", "")).is_err(),
        "counting overlapping occurrences would wrongly make this pass"
    );

    step.stdout_count = vec![at_least("aa", 2)];
    let err = check(&step, &captured(true, 0, "aaa", ""))
        .expect_err("an at-least claim must not be satisfied by an overlapping second match");
    assert!(err.contains("found 1"), "{err}");

    // The same shape in the vocabulary the migrated sites actually use: a
    // needle that is itself a repetition of the printed line.
    step.stdout_count = vec![at_least("3\n3\n", 2)];
    let err = check(&step, &captured(true, 0, "3\n3\n3\n", "")).expect_err("must fail");
    assert!(err.contains("found 1"), "{err}");
}

#[test]
fn a_count_of_zero_matches_fails_an_at_least_claim_and_is_reported_as_zero() {
    let mut step = blank_step();
    step.stdout_count = vec![at_least("3\n", 2)];
    let err = check(&step, &captured(true, 0, "nothing here\n", "")).expect_err("must fail");
    assert!(err.contains("found 0"), "{err}");
}

// A needle longer than the haystack is not a special case -- it is simply
// zero matches. Pinned so a future length-based short-circuit cannot turn it
// into a panic or a vacuous pass.
#[test]
fn a_needle_longer_than_the_haystack_counts_zero_rather_than_erroring_out() {
    let mut step = blank_step();
    step.stdout_count = vec![exactly("a very long needle indeed", 0)];
    check(&step, &captured(true, 0, "hi", "")).expect("zero occurrences is a real, passing claim");

    step.stdout_count = vec![at_least("a very long needle indeed", 1)];
    let err = check(&step, &captured(true, 0, "hi", "")).expect_err("must fail");
    assert!(err.contains("found 0"), "{err}");
}

// `stderr` mirrors `stdout`'s exact-equality check (the field it is added
// symmetric with) -- pinned the same way `exact_stdout_must_match_byte_for_byte`
// pins `stdout`. Added because `stderr_contains`/`stderr_absent` are both
// substring claims: neither can express "stderr is exactly empty," which a
// stray unrelated diagnostic on stderr would satisfy every `stderr_absent`
// needle for while still not being what the source actually asserted.
#[test]
fn exact_stderr_must_match_byte_for_byte() {
    let mut step = blank_step();
    step.stderr = Some("".to_string());
    check(&step, &captured(true, 0, "ok\n", "")).expect("exact empty stderr must pass");
    let err = check(&step, &captured(true, 0, "ok\n", "warning: spurious\n"))
        .expect_err("non-empty stderr must fail an exact-empty claim");
    assert!(err.contains("stderr mismatch"), "{err}");
    assert!(err.contains("spurious"), "{err}");
}

#[test]
fn stderr_claims_are_checked_against_stderr() {
    let mut step = blank_step();
    step.stderr_contains = vec!["E5506".to_string()];
    step.stderr_absent = vec!["is used as both a string and a number".to_string()];
    check(&step, &captured(false, 1, "", "E5506 denied")).expect("stderr claims must both hold");
    let err = check(
        &step,
        &captured(false, 1, "", "E5506 is used as both a string and a number"),
    )
    .expect_err("must fail on a present absence claim");
    assert!(err.contains("is used as both"), "{err}");
}

#[test]
fn json_fields_are_checked_by_dotted_path() {
    let mut step = blank_step();
    step.json = Some(
        toml::from_str(
            r#"
schemaVersion = 1
success = true
[payload]
artifactKind = "bundle"
"#,
        )
        .expect("toml"),
    );
    let good = r#"{"schemaVersion":1,"success":true,"payload":{"artifactKind":"bundle"}}"#;
    check(&step, &captured(true, 0, good, "")).expect("matching json fields must pass");
    let bad = r#"{"schemaVersion":2,"success":true,"payload":{"artifactKind":"bundle"}}"#;
    let err = check(&step, &captured(true, 0, bad, "")).expect_err("must fail");
    assert!(err.contains("schemaVersion"), "{err}");
}

// `json_null` exists because TOML has no null literal (see `Step::json_null`'s
// doc comment) -- these pin its three outcomes: matching null passes, a
// present-but-non-null value fails, and (like every other path lookup in
// this module) an absent path fails rather than passing vacuously.
#[test]
fn json_null_passes_when_the_path_resolves_to_null() {
    let mut step = blank_step();
    step.json_null = vec!["stdout".to_string(), "stderr".to_string()];
    check(
        &step,
        &captured(true, 0, r#"{"stdout":null,"stderr":null}"#, ""),
    )
    .expect("both null paths must pass");
}

#[test]
fn json_null_fails_when_the_path_resolves_to_a_non_null_value() {
    let mut step = blank_step();
    step.json_null = vec!["stdout".to_string()];
    let err = check(&step, &captured(true, 0, r#"{"stdout":""}"#, "")).expect_err("must fail");
    assert!(err.contains("stdout"), "{err}");
    assert!(err.contains("null"), "{err}");
}

#[test]
fn json_null_fails_when_the_path_is_absent_rather_than_passing_vacuously() {
    let mut step = blank_step();
    step.json_null = vec!["payload.missing".to_string()];
    let err = check(&step, &captured(true, 0, r#"{"payload":{}}"#, "")).expect_err("must fail");
    assert!(err.contains("payload.missing"), "{err}");
}

// `json` and `json_null` must be checked together against the same parsed
// document without one silently short-circuiting the other -- this is the
// case that motivated restructuring `check`'s single `if let Some(expected)
// = &step.json` guard into one that also parses when only `json_null` is
// set.
#[test]
fn json_and_json_null_are_both_enforced_on_the_same_step() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"schemaVersion = 1"#).expect("toml"));
    step.json_null = vec!["stderr".to_string()];
    check(
        &step,
        &captured(true, 0, r#"{"schemaVersion":1,"stderr":null}"#, ""),
    )
    .expect("both claims must pass together");
    let err = check(
        &step,
        &captured(true, 0, r#"{"schemaVersion":1,"stderr":""}"#, ""),
    )
    .expect_err("json_null must still be enforced even though json passed");
    assert!(err.contains("stderr"), "{err}");
}

fn json_at_least(path: &str, needle: &str, n: usize) -> JsonCountClaim {
    JsonCountClaim {
        path: path.to_string(),
        needle: needle.to_string(),
        bound: CountBound::AtLeast(n),
    }
}

// The second surface. `browser_math_log2_log10.rs` asserts the *same* count
// on both branches of its `--output json` split -- `:177-179` against
// `json["stdout"].as_str()`, `:186` against the process's raw stdout -- so a
// count key that handled only raw stdout would leave half of every such
// helper hand-written.
#[test]
fn json_count_takes_the_count_against_a_json_string_leaf() {
    let mut step = blank_step();
    step.json_count = vec![json_at_least("stdout", "3\n", 2)];
    check(&step, &captured(true, 0, r#"{"stdout":"3\n3\n"}"#, ""))
        .expect("two occurrences inside the json leaf must pass");
    let err = check(&step, &captured(true, 0, r#"{"stdout":"3\n"}"#, ""))
        .expect_err("one occurrence must fail");
    assert!(
        err.contains("json path stdout"),
        "must name the surface: {err}"
    );
    assert!(err.contains("at least 2"), "{err}");
    assert!(err.contains("found 1"), "{err}");
}

#[test]
fn json_count_exact_matches_the_migrated_assert_eq_shape() {
    let mut step = blank_step();
    step.json_count = vec![JsonCountClaim {
        path: "payload.stdout".to_string(),
        needle: "1.2649110640673518".to_string(),
        bound: CountBound::Exact(2),
    }];
    let two = r#"{"payload":{"stdout":"1.2649110640673518\n1.2649110640673518\n"}}"#;
    check(&step, &captured(true, 0, two, "")).expect("exactly two must pass");
    let three =
        r#"{"payload":{"stdout":"1.2649110640673518\n1.2649110640673518\n1.2649110640673518\n"}}"#;
    let err = check(&step, &captured(true, 0, three, "")).expect_err("three must fail");
    assert!(err.contains("payload.stdout"), "{err}");
    assert!(err.contains("found 3"), "{err}");
}

// Same §5.10 rule the `json` and `json_null` keys follow: a path that does
// not resolve is a hard failure, never a pass. "Nothing was found" must never
// silently become "nothing to assert."
#[test]
fn json_count_fails_when_the_path_is_absent_rather_than_passing_vacuously() {
    let mut step = blank_step();
    step.json_count = vec![json_at_least("payload.missing", "3\n", 2)];
    let err = check(&step, &captured(true, 0, r#"{"payload":{}}"#, "")).expect_err("must fail");
    assert!(err.contains("payload.missing"), "{err}");
    assert!(err.contains("absent"), "{err}");
}

// The subtler half of the same rule: the path resolves, but to something with
// no text to count in. `json["stdout"]` is legitimately `null` in this CLI's
// envelope (that is why `json_null` exists), so this is a reachable shape --
// and treating it as zero matches would turn an `exact = 0` claim into a
// vacuous pass.
#[test]
fn json_count_fails_when_the_leaf_is_not_a_string() {
    let mut step = blank_step();
    step.json_count = vec![json_at_least("stdout", "3\n", 2)];
    let err = check(&step, &captured(true, 0, r#"{"stdout":null}"#, ""))
        .expect_err("a null leaf must hard-fail, not count zero");
    assert!(err.contains("json path stdout"), "{err}");
    assert!(err.contains("not a json string"), "{err}");

    step.json_count = vec![JsonCountClaim {
        path: "stdout".to_string(),
        needle: "3".to_string(),
        bound: CountBound::Exact(0),
    }];
    let err = check(&step, &captured(true, 0, r#"{"stdout":[]}"#, ""))
        .expect_err("an array leaf must hard-fail rather than vacuously satisfying `exact = 0`");
    assert!(err.contains("not a json string"), "{err}");
}

// All three json-surface keys read the same parsed document; none may
// short-circuit another. This is the guard that `check`'s parse condition
// grew a `|| !step.json_count.is_empty()` arm rather than staying keyed on
// `json`/`json_null` alone.
#[test]
fn json_count_is_enforced_even_when_it_is_the_only_json_key_on_the_step() {
    let mut step = blank_step();
    step.json_count = vec![json_at_least("stdout", "3\n", 2)];
    let err = check(&step, &captured(true, 0, "not json at all", ""))
        .expect_err("stdout that is not json must fail, not skip the claim");
    assert!(err.to_lowercase().contains("json"), "{err}");

    step.json = Some(toml::from_str("schemaVersion = 1").expect("toml"));
    step.json_null = vec!["stderr".to_string()];
    let good = r#"{"schemaVersion":1,"stderr":null,"stdout":"3\n3\n"}"#;
    check(&step, &captured(true, 0, good, "")).expect("all three claims must pass together");
    let bad = r#"{"schemaVersion":1,"stderr":null,"stdout":"3\n"}"#;
    let err = check(&step, &captured(true, 0, bad, ""))
        .expect_err("json_count must still be enforced even though json and json_null passed");
    assert!(err.contains("found 1"), "{err}");
}

#[test]
fn a_missing_json_path_fails_and_names_the_path() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"payload.bundleFormat = "esm""#).expect("toml"));
    let err = check(&step, &captured(true, 0, r#"{"payload":{}}"#, "")).expect_err("must fail");
    assert!(err.contains("payload.bundleFormat"), "{err}");
}

// An empty-table expectation ("this key is an empty object") must be
// enforced, not silently skipped because there is nothing under it to
// recurse into. Mirrors the design spec's §5.6 `fields` spelling
// (`json = { schemaVersion = 1, payload = { diagnostics = {} } }`): actual
// output with no `diagnostics` key at all (and nine unrelated errors) must
// fail, not pass having verified nothing.
#[test]
fn a_nested_empty_table_expectation_is_enforced() {
    let mut step = blank_step();
    step.json = Some(
        toml::from_str("schemaVersion = 1\n[payload]\n[payload.diagnostics]\n").expect("toml"),
    );
    let good = r#"{"schemaVersion":1,"payload":{"diagnostics":{}}}"#;
    check(&step, &captured(true, 0, good, ""))
        .expect("a matching empty-table expectation must pass");

    let missing_key = r#"{"schemaVersion":1,"payload":{"errorCount":9}}"#;
    let err = check(&step, &captured(true, 0, missing_key, "")).expect_err("must fail");
    assert!(err.contains("payload.diagnostics"), "{err}");

    let non_empty = r#"{"schemaVersion":1,"payload":{"diagnostics":{"x":1}}}"#;
    assert!(check(&step, &captured(true, 0, non_empty, "")).is_err());
}

// The same rule, on `check_json` directly -- this is the function Task 11's
// `run_file_json` calls for a `file_json` step's `fields` key, so the
// vacuous-pass bug must be closed there too, not just on the `json` key's
// path through `check`.
#[test]
fn a_top_level_empty_table_expectation_is_enforced_via_check_json() {
    let expected: toml::Value = toml::from_str("").expect("toml");
    check_json(&expected, &serde_json::json!({})).expect("a matching empty document must pass");
    let err = check_json(&expected, &serde_json::json!({"other": 1})).expect_err("must fail");
    assert!(err.contains("top-level"), "{err}");
}

// `check` receives the whole `Step`, including `kind` -- a `file_json` step
// sets no `cli` field by construction (`finalize_step` forbids it), so
// without this guard `check` would return `Ok(())` having verified nothing.
// This makes the seam un-bypassable from this side: if a future dispatch
// mistake ever routes a `file_json` step into `check` instead of
// `run_file_json`, it fails loudly instead of silently.
#[test]
fn check_rejects_a_file_json_step_rather_than_passing_it_vacuously() {
    let mut step = blank_step();
    step.kind = StepKind::FileJson;
    step.path = Some("out.json".to_string());
    step.fields = Some(toml::from_str("schemaVersion = 1").expect("toml"));
    let err = check(&step, &captured(true, 0, "", "")).expect_err("must fail");
    assert!(err.contains("file_json"), "{err}");
    assert!(err.contains("run_file_json"), "{err}");
}

// A scalar top-level expectation (`json = "hi"`) flattens to a leaf at the
// empty path. Before the fix this produced "json path  is absent" (empty
// path, double space, and wrong besides -- the path is never actually
// absent, since an empty path addresses the whole document). It must instead
// report a proper mismatch, labelled clearly.
#[test]
fn a_scalar_top_level_json_claim_reports_a_labelled_mismatch() {
    let mut step = blank_step();
    // `toml::Value: FromStr` parses a single inline *value* expression (not
    // a document), which is exactly what's wanted here: the bare TOML value
    // `"hi"`, not a document containing a key named `hi`.
    step.json = Some(r#""hi""#.parse::<toml::Value>().expect("inline toml value"));
    let err = check(&step, &captured(true, 0, r#"{"a":1}"#, "")).expect_err("must fail");
    assert!(err.contains("top-level"), "{err}");
    assert!(!err.contains("is absent"), "{err}");
    assert!(!err.contains("path  "), "{err}");
}

// A numeric path segment indexes into a JSON array -- this is what lets a
// case pin "the first diagnostic has this code" (`errors.0.code`) without
// asserting the rest of the diagnostic object, which is unmatchable in this
// format (every diagnostic carries a hard-coded `"fix": null`, and TOML has
// no null literal to match it with).
#[test]
fn an_indexed_array_path_resolves_and_is_checked() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"errors."0".code = "E5506""#).expect("toml"));
    check(
        &step,
        &captured(true, 0, r#"{"errors":[{"code":"E5506"}]}"#, ""),
    )
    .expect("errors.0.code must resolve and match");
    let err = check(
        &step,
        &captured(true, 0, r#"{"errors":[{"code":"E1234"}]}"#, ""),
    )
    .expect_err("must fail on the wrong code");
    assert!(err.contains("errors.0.code"), "{err}");
}

// An out-of-range index is a hard failure that names the actual problem
// (which index, and the array's real length), not a plain "is absent" that
// leaves a case author guessing whether it's a typo or an index past the end.
#[test]
fn an_out_of_range_array_index_hard_fails_and_names_the_range() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"errors."5".code = "E5506""#).expect("toml"));
    let err = check(
        &step,
        &captured(true, 0, r#"{"errors":[{"code":"E5506"}]}"#, ""),
    )
    .expect_err("must fail");
    assert!(err.contains("errors.5.code"), "{err}");
    assert!(err.contains("out of range"), "{err}");
    assert!(err.contains("length 1"), "{err}");
}

// A non-numeric segment against an array (forgetting the index) must still
// fail loudly -- not silently match nothing and pass vacuously -- and the
// message should point at the actual problem: this segment is not a valid
// index, so the array needs one.
#[test]
fn a_non_numeric_segment_into_an_array_hard_fails_and_names_the_problem() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"errors.code = "E5506""#).expect("toml"));
    let err = check(
        &step,
        &captured(true, 0, r#"{"errors":[{"code":"E5506"}]}"#, ""),
    )
    .expect_err("must fail");
    assert!(err.contains("errors.code"), "{err}");
    assert!(err.contains("not a valid array index"), "{err}");
}

// `usize` cannot represent a negative number, so a negative-looking segment
// is rejected exactly like any other invalid index -- there is no
// negative-from-end indexing in this format.
#[test]
fn a_negative_looking_array_segment_hard_fails() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"errors."-1".code = "E5506""#).expect("toml"));
    let err = check(
        &step,
        &captured(true, 0, r#"{"errors":[{"code":"E5506"}]}"#, ""),
    )
    .expect_err("must fail");
    assert!(err.contains("not a valid array index"), "{err}");
}

// The numeric-key ambiguity, pinned end to end: a numeric-looking segment
// against a JSON *object* is a plain key lookup, not an index -- indexing
// only ever applies to JSON *arrays*. An object with a literal key "0"
// behaves exactly like any other object.
#[test]
fn a_numeric_segment_against_an_object_is_a_plain_key_not_an_index() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"payload.0 = "x""#).expect("toml"));
    check(&step, &captured(true, 0, r#"{"payload":{"0":"x"}}"#, ""))
        .expect("a numeric segment against an object must be a plain key lookup");
}

// Defense in depth: an unresolved `${...}` placeholder should be caught by
// Task 9's substitution pass, but if one ever escapes it into a `json`
// expectation, comparing it literally would silently accept output that
// happens to contain the same placeholder text instead of the real value.
// The actual value here is deliberately the placeholder itself -- a plain
// `values_equal` comparison would call this a match -- to prove the guard
// (not an ordinary mismatch) is what makes this fail.
#[test]
fn an_unsubstituted_placeholder_in_a_json_claim_hard_fails() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"payload.bundleFormat = "${format}""#).expect("toml"));
    let err = check(
        &step,
        &captured(true, 0, r#"{"payload":{"bundleFormat":"${format}"}}"#, ""),
    )
    .expect_err("must fail even though the literal text matches");
    assert!(err.contains("payload.bundleFormat"), "{err}");
    assert!(err.contains("${format}"), "{err}");
}

#[test]
fn unparseable_stdout_under_a_json_claim_fails_rather_than_passing_vacuously() {
    let mut step = blank_step();
    step.json = Some(toml::from_str("schemaVersion = 1").expect("toml"));
    let err = check(&step, &captured(true, 0, "not json", "")).expect_err("must fail");
    assert!(err.to_lowercase().contains("json"), "{err}");
}

// Failure text must not emit lines matching `^    [A-Za-z_]`, which
// scripts/test-gate.sh parses as failed-test names.
fn assert_no_four_space_name_indent(err: &str) {
    for line in err.lines() {
        assert!(
            !(line.starts_with("    ")
                && line
                    .chars()
                    .nth(4)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')),
            "line would be misparsed by test-gate.sh: {line:?}\nfull message:\n{err}"
        );
    }
}

#[test]
fn failure_text_never_uses_the_four_space_name_indent() {
    let mut step = blank_step();
    step.stdout = Some("expected".to_string());
    let err = check(
        &step,
        &captured(true, 0, "actual output here", "and stderr"),
    )
    .expect_err("must fail");
    assert_no_four_space_name_indent(&err);
}

// A count failure renders the needle with `{:?}`, so a needle containing
// newlines (`"3\n"` -- the single most common one in the migrated corpus)
// stays on one line rather than breaking the message across lines the gate
// would then reparse. The `json_count` non-string case additionally
// `Display`s a `serde_json::Value`.
#[test]
fn a_count_failure_never_uses_the_four_space_name_indent() {
    let mut step = blank_step();
    step.stdout_count = vec![at_least("3\n    indented\n", 2)];
    let err = check(&step, &captured(true, 0, "3\n", "some stderr")).expect_err("must fail");
    assert_no_four_space_name_indent(&err);

    let mut step = blank_step();
    step.json_count = vec![json_at_least("stdout", "3\n", 2)];
    let err = check(&step, &captured(true, 0, r#"{"stdout":{"a":1}}"#, "")).expect_err("must fail");
    assert_no_four_space_name_indent(&err);
}

// The `json path ... mismatch` pair `Display`s a foreign type (`toml::Value`
// / `serde_json::Value`) rather than text this module controls -- a
// multi-line rendering there (e.g. a pretty-printed array) would introduce
// exactly the four-space lines this rule forbids. Both crates render inline
// today, but this pin is what would catch a regression under a toml or
// serde_json upgrade, which the stdout-mismatch case above cannot.
#[test]
fn a_json_mismatch_failure_also_never_uses_the_four_space_name_indent() {
    let mut step = blank_step();
    step.json = Some(toml::from_str("codes = [1, 2, 3]").expect("toml"));
    let err = check(
        &step,
        &captured(true, 0, r#"{"codes":[1,2,4]}"#, "some stderr"),
    )
    .expect_err("must fail");
    assert!(err.contains("codes"), "{err}");
    assert_no_four_space_name_indent(&err);
}
