//! Evaluate a step's twelve assertion keys against captured process output.
//!
//! Failure messages are indented with two spaces, never four. `scripts/test-gate.sh`
//! parses `^    [A-Za-z_]` as a failed-test name, and a four-space-indented
//! detail line would be misread as a test that does not exist.

use crate::jsonpath::{describe_absence, flatten_expected, lookup, values_equal};
use crate::model::{CountBound, Exit, ExitStatusWord, Step, StepKind};

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

/// `file_json` steps never run a process -- their `fields` claim is
/// evaluated by Task 11's `run_file_json` calling `check_json` directly
/// against the parsed file, not by this function. `finalize_step` already
/// forbids a `file_json` step from setting any `cli`-only field, so calling
/// `check` on one would otherwise see every field `None`/empty and silently
/// return `Ok(())` having verified nothing. Rejecting the kind here makes
/// that seam un-bypassable from this side: if a future dispatch mistake ever
/// routes a `file_json` step into `check`, it fails loudly instead of
/// passing vacuously.
pub fn check(step: &Step, captured: &Captured) -> Result<(), String> {
    if step.kind == StepKind::FileJson {
        return Err(
            "a `file_json` step's `fields` must be evaluated by `run_file_json`, not `check` \
             -- `file_json` steps do not run a process"
                .to_string(),
        );
    }

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
    for claim in &step.stdout_count {
        check_count("stdout", &captured.stdout, &claim.needle, claim.bound).map_err(fail)?;
    }

    if let Some(expected) = &step.stderr {
        if &captured.stderr != expected {
            return Err(fail(format!(
                "stderr mismatch\n  expected: {expected:?}\n  actual:   {:?}",
                captured.stderr
            )));
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

    if step.json.is_some() || !step.json_null.is_empty() || !step.json_count.is_empty() {
        let actual: serde_json::Value = serde_json::from_str(&captured.stdout)
            .map_err(|error| fail(format!("stdout is not valid json: {error}")))?;
        if let Some(expected) = &step.json {
            check_json(expected, &actual).map_err(fail)?;
        }
        for path in &step.json_null {
            check_json_null(path, &actual).map_err(fail)?;
        }
        for claim in &step.json_count {
            check_json_count(claim.bound, &claim.path, &claim.needle, &actual).map_err(fail)?;
        }
    }

    Ok(())
}

/// The `stdout_count` / `json_count` evaluator.
///
/// **Counting is non-overlapping, left-to-right**, because that is what
/// `str::matches` does and every claim being migrated into this key was
/// written as `haystack.matches(needle).count()` against exactly those
/// semantics. `"aaa".matches("aa").count()` is **1**, not 2: the scan resumes
/// after the end of each match, never inside it. This function therefore
/// *delegates* to `str::matches` rather than reimplementing the scan -- a
/// hand-rolled counter that advanced by one byte per hit instead of by
/// `needle.len()` would count overlapping occurrences, and would silently
/// *strengthen* every migrated claim (reporting more matches than the source
/// assertion ever saw, so an at-least claim passes on output the original
/// rejected). `count_is_non_overlapping_exactly_as_str_matches_is` in
/// `assertions_tests.rs` pins this against that specific mistake.
///
/// The needle is never empty -- `model::count_needle` rejects an empty one at
/// parse time, so the `len + 1` result `str::matches("")` would produce here
/// is unreachable rather than merely undocumented.
///
/// `surface` names where the count was taken (`"stdout"`, or `json path
/// payload.stdout`) so the failure message is diagnosable without reading the
/// case file: §5.9 requires the needle, the expectation, and the actual count
/// all be named.
fn check_count(
    surface: &str,
    haystack: &str,
    needle: &str,
    bound: CountBound,
) -> Result<(), String> {
    let actual = haystack.matches(needle).count();
    if bound.holds(actual) {
        return Ok(());
    }
    Err(format!(
        "{surface}: expected {} non-overlapping occurrences of {needle:?}, found {actual}",
        bound.describe()
    ))
}

/// `check_count` against a JSON string leaf of the captured stdout.
///
/// Both non-resolving ways this can go wrong are hard failures, never a
/// silent pass, for the same reason `check_json_null` treats an absent path
/// as one (§5.10): an unresolvable path is "nothing was checked," not
/// "nothing to check." A leaf that resolves to a non-string is the subtler of
/// the two -- `json["stdout"]` being `null` (a real shape in this CLI's
/// envelope, which is why `json_null` exists) has no text to count needles
/// in, and quietly treating it as zero matches would turn every `at_least`
/// claim into a failure for the wrong reason and every `exact = 0` claim into
/// a vacuous pass.
fn check_json_count(
    bound: CountBound,
    path: &str,
    needle: &str,
    actual: &serde_json::Value,
) -> Result<(), String> {
    let label = describe_path(path);
    match lookup(actual, path) {
        None => Err(missing_path_message(&label, actual, path)),
        Some(serde_json::Value::String(text)) => check_count(&label, text, needle, bound),
        Some(found) => Err(format!(
            "{label} is not a json string, so {needle:?} cannot be counted in it\n  actual:   \
             {found}"
        )),
    }
}

/// A `json_null` path claim: `Step::json_null`'s doc comment explains why
/// this is a separate key from `json` rather than a `toml::Value` leaf --
/// TOML has no null literal, so `values_equal` (jsonpath.rs) can never match
/// one no matter how `json` is spelled.
///
/// A path absent from `actual` is a hard failure here for the same reason
/// `check_json` treats it as one (see that function's doc comment): "not
/// found" is not "null," and treating it as a pass would let a case go
/// green having verified nothing.
fn check_json_null(path: &str, actual: &serde_json::Value) -> Result<(), String> {
    let label = describe_path(path);
    match lookup(actual, path) {
        None => Err(missing_path_message(&label, actual, path)),
        Some(serde_json::Value::Null) => Ok(()),
        Some(found) => Err(format!("{label} expected null\n  actual:   {found}")),
    }
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
        let label = describe_path(&path);
        if let Some(text) = leaf.as_str() {
            if text.contains("${") {
                return Err(format!(
                    "{label} still contains an unsubstituted placeholder: {text:?}"
                ));
            }
        }
        match lookup(actual, &path) {
            None => return Err(missing_path_message(&label, actual, &path)),
            Some(found) if !values_equal(&leaf, found) => {
                return Err(format!(
                    "{label} mismatch\n  expected: {leaf}\n  actual:   {found}"
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// A top-level `json`/`fields` expectation that is itself a scalar (`json =
/// "hi"`) flattens to a leaf at the empty path -- `lookup` treats that as
/// "the whole document," so it is never actually absent, but its failure
/// messages still need a label. "json path  is absent" (empty path, double
/// space) tells a case author nothing; "top-level json value" does.
fn describe_path(path: &str) -> String {
    if path.is_empty() {
        "top-level json value".to_string()
    } else {
        format!("json path {path}")
    }
}

/// `path` is never empty here -- `lookup` only returns `None` for a
/// non-empty path (an empty path always addresses the whole document, per
/// its doc comment). `describe_absence` walks the path a second time to say
/// exactly which segment broke and why -- absent key, non-array-index
/// segment against an array, or an index past the end.
fn missing_path_message(label: &str, actual: &serde_json::Value, path: &str) -> String {
    format!("{label} {}", describe_absence(actual, path))
}

#[cfg(test)]
#[path = "assertions_tests.rs"]
mod assertions_tests;
