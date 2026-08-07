use super::*;
use crate::model::{Exit, ExitStatusWord, Step, StepKind};
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
        stderr_contains: Vec::new(),
        stderr_absent: Vec::new(),
        json: None,
        path: None,
        fields: None,
        entry: None,
        body: None,
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
    assert!(check(&step, &captured(true, 0, "", "")).is_ok());
    let err = check(&step, &captured(false, 1, "", "")).expect_err("must fail");
    assert!(err.contains("exit"), "{err}");
}

#[test]
fn an_exact_exit_code_must_match() {
    let mut step = blank_step();
    step.exit = Some(Exit::Code(2));
    assert!(check(&step, &captured(false, 2, "", "")).is_ok());
    assert!(check(&step, &captured(false, 1, "", "")).is_err());
}

#[test]
fn exact_stdout_must_match_byte_for_byte() {
    let mut step = blank_step();
    step.stdout = Some("hahaha\n\n".to_string());
    assert!(check(&step, &captured(true, 0, "hahaha\n\n", "")).is_ok());
    assert!(check(&step, &captured(true, 0, "hahaha\n", "")).is_err());
}

#[test]
fn contains_and_absent_are_both_enforced() {
    let mut step = blank_step();
    step.stdout_contains = vec!["1\n".to_string()];
    step.stdout_absent = vec!["E5506".to_string()];
    assert!(check(&step, &captured(true, 0, "1\n0\n", "")).is_ok());
    assert!(check(&step, &captured(true, 0, "0\n", "")).is_err());
    assert!(check(&step, &captured(true, 0, "1\nE5506", "")).is_err());
}

#[test]
fn stderr_claims_are_checked_against_stderr() {
    let mut step = blank_step();
    step.stderr_contains = vec!["E5506".to_string()];
    step.stderr_absent = vec!["is used as both a string and a number".to_string()];
    assert!(check(&step, &captured(false, 1, "", "E5506 denied")).is_ok());
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
    assert!(check(&step, &captured(true, 0, good, "")).is_ok());
    let bad = r#"{"schemaVersion":2,"success":true,"payload":{"artifactKind":"bundle"}}"#;
    let err = check(&step, &captured(true, 0, bad, "")).expect_err("must fail");
    assert!(err.contains("schemaVersion"), "{err}");
}

#[test]
fn a_missing_json_path_fails_and_names_the_path() {
    let mut step = blank_step();
    step.json = Some(toml::from_str(r#"payload.bundleFormat = "esm""#).expect("toml"));
    let err = check(&step, &captured(true, 0, r#"{"payload":{}}"#, "")).expect_err("must fail");
    assert!(err.contains("payload.bundleFormat"), "{err}");
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
#[test]
fn failure_text_never_uses_the_four_space_name_indent() {
    let mut step = blank_step();
    step.stdout = Some("expected".to_string());
    let err = check(
        &step,
        &captured(true, 0, "actual output here", "and stderr"),
    )
    .expect_err("must fail");
    for line in err.lines() {
        assert!(
            !(line.starts_with("    ")
                && line
                    .chars()
                    .nth(4)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')),
            "line would be misparsed by test-gate.sh: {line:?}"
        );
    }
}
