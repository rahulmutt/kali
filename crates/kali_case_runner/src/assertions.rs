//! Evaluate a step's eight assertion keys against captured process output.
//!
//! Failure messages are indented with two spaces, never four. `scripts/test-gate.sh`
//! parses `^    [A-Za-z_]` as a failed-test name, and a four-space-indented
//! detail line would be misread as a test that does not exist.

use crate::jsonpath::{flatten_expected, lookup, values_equal};
use crate::model::{Exit, ExitStatusWord, Step};

pub struct Captured {
    pub code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Captured {
    fn context(&self) -> String {
        format!(
            "  --- stdout ---\n{}\n  --- stderr ---\n{}",
            indent(&self.stdout),
            indent(&self.stderr)
        )
    }
}

fn indent(text: &str) -> String {
    if text.is_empty() {
        return "  (empty)".to_string();
    }
    text.lines()
        .map(|line| format!("  | {line}"))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn check(step: &Step, captured: &Captured) -> Result<(), String> {
    let fail = |claim: String| -> String { format!("{claim}\n{}", captured.context()) };

    match step.exit {
        Some(Exit::Status(ExitStatusWord::Success)) if !captured.success => {
            return Err(fail(format!(
                "expected exit success, got code {:?}",
                captured.code
            )));
        }
        Some(Exit::Status(ExitStatusWord::Failure)) if captured.success => {
            return Err(fail("expected exit failure, but it succeeded".to_string()));
        }
        Some(Exit::Code(expected)) if captured.code != Some(expected) => {
            return Err(fail(format!(
                "expected exit code {expected}, got {:?}",
                captured.code
            )));
        }
        _ => {}
    }

    if let Some(expected) = &step.stdout {
        if &captured.stdout != expected {
            return Err(fail(format!(
                "stdout mismatch\n  expected: {expected:?}\n  actual:   {:?}",
                captured.stdout
            )));
        }
    }

    for needle in &step.stdout_contains {
        if !captured.stdout.contains(needle.as_str()) {
            return Err(fail(format!("stdout missing {needle:?}")));
        }
    }
    for needle in &step.stdout_absent {
        if captured.stdout.contains(needle.as_str()) {
            return Err(fail(format!("stdout must not contain {needle:?}")));
        }
    }
    for needle in &step.stderr_contains {
        if !captured.stderr.contains(needle.as_str()) {
            return Err(fail(format!("stderr missing {needle:?}")));
        }
    }
    for needle in &step.stderr_absent {
        if captured.stderr.contains(needle.as_str()) {
            return Err(fail(format!("stderr must not contain {needle:?}")));
        }
    }

    if let Some(expected) = &step.json {
        let actual: serde_json::Value = serde_json::from_str(&captured.stdout)
            .map_err(|error| fail(format!("stdout is not valid json: {error}")))?;
        check_json(expected, &actual).map_err(fail)?;
    }

    Ok(())
}

/// Shared by the `json` key and by `file_json`'s `fields` key.
///
/// A path absent from `actual` is a hard failure, not something to skip: if
/// "not found" were treated as "nothing to assert," a case could go green
/// having verified nothing (the exact degradation this format exists to
/// close). A leaf that still contains a literal `${` is an unsubstituted
/// placeholder that escaped matrix/constant substitution; comparing it
/// literally would silently never match real output, so it hard-fails here
/// too rather than being compared as a string.
pub fn check_json(expected: &toml::Value, actual: &serde_json::Value) -> Result<(), String> {
    for (path, leaf) in flatten_expected(expected) {
        if let Some(text) = leaf.as_str() {
            if text.contains("${") {
                return Err(format!(
                    "json path {path} still contains an unsubstituted placeholder: {text:?}"
                ));
            }
        }
        match lookup(actual, &path) {
            None => return Err(format!("json path {path} is absent")),
            Some(found) if !values_equal(&leaf, found) => {
                return Err(format!(
                    "json path {path} mismatch\n  expected: {leaf}\n  actual:   {found}"
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "assertions_tests.rs"]
mod assertions_tests;
