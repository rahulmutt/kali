//! Serde model for a `.toml` case file.
//!
//! Every struct that can, is `deny_unknown_fields`. That is load-bearing: a
//! typo'd assertion key must fail the run, not silently assert nothing.
//!
//! ## Why `Case` is not a single `#[derive(Deserialize)]` struct
//!
//! The obvious design -- `Case { name, rationale, ignore, step,
//! #[serde(flatten)] inline: Option<Step> }` -- does not work. `#[serde(flatten)]`
//! is incompatible with `#[serde(deny_unknown_fields)]` on the *containing*
//! struct (a compile error), which is well known. What is *not* well known,
//! and is not just a consequence of that compile-time restriction, is this:
//! `#[serde(deny_unknown_fields)]` on the *flattened-into* type (`Step`
//! here) is silently ignored too. A key that matches neither the outer
//! struct's named fields nor any field of `Step` is dropped without error,
//! regardless of whether the flattened field is `Step` or `Option<Step>`.
//! Verified directly against both `toml` and `serde_json`:
//!
//! ```text
//! #[derive(Deserialize)] struct Case { name: String, #[serde(flatten)] inline: Step }
//! #[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Step { args: Vec<String>, .. }
//! toml::from_str::<Case>(r#"name="c"\nargs=["run"]\nstdout_contain=["oops"]"#)
//!   // => Ok(Case { inline: Step { args: ["run"], .. } }), no error, typo dropped
//! ```
//!
//! This is the long-standing upstream limitation tracked as
//! <https://github.com/serde-rs/serde/issues/1600>: `#[serde(flatten)]`'s
//! `Content`-buffering deserialization path does not honor
//! `deny_unknown_fields` on the flattened type. It has nothing to do with
//! `Option` wrapping.
//!
//! The fix here routes around the derive machinery instead of relying on
//! it: `RawCase` flattens the residual (non-`name`/`rationale`/`ignore`/
//! `step`) keys into a raw `toml::Table`, and `parse_case_file` converts
//! that table into `RawStep` with
//! `toml::Value::Table(rest).try_into::<RawStep>()` -- a plain `Deserialize`
//! call, outside the flatten `Content` buffer, which *does* honor
//! `RawStep`'s `deny_unknown_fields`. `finalize_step` then turns a `RawStep`
//! into the public `Step`, resolving its default `kind` and rejecting
//! fields that don't apply to that kind (see its doc comment). Both the
//! manual conversion and `finalize_step` are hand-written, unlike the rest
//! of this module; their correctness (every `Step` field actually carried
//! through, unknown keys in both the inline and `[[case.step]]` forms
//! rejected, wrong-typed known keys rejected without panicking,
//! kind-inapplicable fields rejected) is covered directly in
//! `model_tests.rs`, not just via the happy path.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCaseFile {
    #[serde(default)]
    constants: BTreeMap<String, String>,
    #[serde(default)]
    matrix: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    source: BTreeMap<String, String>,
    #[serde(default)]
    case: Vec<RawCase>,
}

// No `deny_unknown_fields` here: it cannot coexist with `#[serde(flatten)]`
// on `rest` (compile error). `rest` catches every key that isn't `name`,
// `rationale`, `ignore`, or `step`; `parse_case_file` is what turns those
// residual keys into a validated `Step` (or rejects them), so the
// unknown-key guarantee is not lost -- see the module doc comment.
#[derive(Debug, Deserialize)]
struct RawCase {
    name: String,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    ignore: bool,
    /// Multi-step form: `[[case.step]]`.
    #[serde(default)]
    step: Vec<RawStep>,
    /// Everything else written directly on `[[case]]` -- the single-step
    /// shorthand's raw material, converted to `Step` in `parse_case_file`.
    #[serde(flatten)]
    rest: toml::Table,
}

#[derive(Debug)]
pub struct CaseFile {
    pub constants: BTreeMap<String, String>,
    pub matrix: BTreeMap<String, Vec<String>>,
    pub source: BTreeMap<String, String>,
    pub case: Vec<Case>,
}

#[derive(Debug)]
pub struct Case {
    pub name: String,
    pub rationale: Option<String>,
    pub ignore: bool,
    /// Multi-step form: `[[case.step]]`.
    pub step: Vec<Step>,
    /// Single-step shorthand: step fields written directly on `[[case]]`.
    pub inline: Option<Step>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    #[default]
    Cli,
    FileJson,
    BrowserBundleHarness,
}

impl StepKind {
    /// The spelling an author would write in a case file, used in error
    /// messages so they read like the TOML the author wrote.
    fn as_str(self) -> &'static str {
        match self {
            StepKind::Cli => "cli",
            StepKind::FileJson => "file_json",
            StepKind::BrowserBundleHarness => "browser_bundle_harness",
        }
    }
}

// `Exit` is `#[serde(untagged)]` so both `exit = "success"` and `exit = 2`
// parse from the same field. Untagged unit variants only match a TOML unit
// value, not a string, so a bare `Success`/`Failure` pair of unit variants
// on `Exit` itself would *not* accept `exit = "success"`. `ExitStatusWord`
// exists to route around that: as a newtype payload, `Status(ExitStatusWord)`
// delegates to `ExitStatusWord`'s own (externally tagged) `Deserialize`,
// which *does* accept a bare string. `Exit::SUCCESS` / `Exit::FAILURE` are
// the ergonomic spelling call sites should use instead of spelling out
// `Exit::Status(ExitStatusWord::Success)`.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(untagged)]
pub enum Exit {
    Status(ExitStatusWord),
    Code(i32),
}

impl Exit {
    pub const SUCCESS: Exit = Exit::Status(ExitStatusWord::Success);
    pub const FAILURE: Exit = Exit::Status(ExitStatusWord::Failure);
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ExitStatusWord {
    Success,
    Failure,
}

// The two raw count-claim tables. `at_least` and `exact` are both `Option`
// here and collapsed into exactly one `CountBound` by `count_bound`: the
// format admits no other comparison, and requiring exactly one of the two to
// be spelled is what keeps `{ needle = "3\n" }` (a table that would assert
// nothing) from parsing. `deny_unknown_fields` applies to the table itself,
// so `{ needle = "3\n", atleast = 2 }` is rejected rather than read as a
// claim with no bound.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCountClaim {
    needle: String,
    #[serde(default)]
    at_least: Option<usize>,
    #[serde(default)]
    exact: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJsonCountClaim {
    path: String,
    needle: String,
    #[serde(default)]
    at_least: Option<usize>,
    #[serde(default)]
    exact: Option<usize>,
}

/// How many non-overlapping occurrences of a needle a surface must hold.
///
/// Deliberately closed at the two comparisons the migrated corpus actually
/// contains -- `count() >= n` (29 sites) and `count() == n` (3 sites). No
/// `<`, `<=`, `>`, or `!=`: no source assertion spells them, and an operator
/// no case can justify is a vocabulary the format has to keep honest forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountBound {
    AtLeast(usize),
    Exact(usize),
}

impl CountBound {
    /// Rendered into failure text, so it reads as prose: "expected at least
    /// 2 non-overlapping occurrences of ...".
    pub fn describe(self) -> String {
        match self {
            CountBound::AtLeast(n) => format!("at least {n}"),
            CountBound::Exact(n) => format!("exactly {n}"),
        }
    }

    pub fn holds(self, actual: usize) -> bool {
        match self {
            CountBound::AtLeast(n) => actual >= n,
            CountBound::Exact(n) => actual == n,
        }
    }
}

/// One `stdout_count` claim: how many non-overlapping occurrences of `needle`
/// the step's captured stdout must hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountClaim {
    pub needle: String,
    pub bound: CountBound,
}

/// One `json_count` claim: the same count, taken against the JSON string leaf
/// at `path` in the step's captured stdout rather than against raw stdout.
/// The two surfaces are separate keys, not one key with an optional `path`,
/// because the same helper in a migrated source file routinely asserts the
/// same count on *both* (the `--output json` branch reads `json["stdout"]`,
/// the text branch reads the process's stdout) and the case file must be able
/// to say which without the reader inferring it from a field's absence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonCountClaim {
    pub path: String,
    pub needle: String,
    pub bound: CountBound,
}

/// Collapse a raw claim's `at_least`/`exact` pair into the single bound the
/// evaluator sees, rejecting the two ways an author can write a table that
/// asserts nothing:
///
/// - neither spelled (`{ needle = "3\n" }`) -- there is no default bound to
///   fall back on that would not be a guess about the author's intent;
/// - both spelled -- the two could disagree, and there is no rule for which
///   one wins that is not silently discarding half of what was written.
///
/// `at_least = 0` is rejected for the same family of reason: every string
/// contains at least zero occurrences of anything, so it is a claim that can
/// never fail. `exact = 0` is *not* rejected -- "this needle appears zero
/// times" is a real, falsifiable claim (a stricter `stdout_absent`).
fn count_bound(
    at_least: Option<usize>,
    exact: Option<usize>,
    case_name: &str,
    key: &str,
) -> Result<CountBound, String> {
    match (at_least, exact) {
        (Some(_), Some(_)) => Err(format!(
            "case `{case_name}`: a `{key}` claim sets both `at_least` and `exact` -- set exactly \
             one"
        )),
        (None, None) => Err(format!(
            "case `{case_name}`: a `{key}` claim sets neither `at_least` nor `exact` -- set \
             exactly one, or the claim asserts nothing"
        )),
        (Some(0), None) => Err(format!(
            "case `{case_name}`: a `{key}` claim sets `at_least = 0`, which every possible output \
             satisfies -- use `exact = 0` to claim the needle is absent"
        )),
        (Some(n), None) => Ok(CountBound::AtLeast(n)),
        (None, Some(n)) => Ok(CountBound::Exact(n)),
    }
}

/// An empty needle is rejected rather than evaluated. Rust's `str::matches`
/// yields `haystack.chars().count() + 1` matches for `""` (one at every
/// character boundary, including both ends), which is a number no case author
/// writing `count() >= 2` ever meant; reproducing it faithfully would be
/// correct and useless, and special-casing it to 0 would silently diverge
/// from the `str::matches` semantics every migrated claim was written
/// against. Refusing it at parse time is the only option that does neither.
fn count_needle(needle: String, case_name: &str, key: &str) -> Result<String, String> {
    if needle.is_empty() {
        return Err(format!(
            "case `{case_name}`: a `{key}` claim has an empty `needle` -- `str::matches(\"\")` \
             matches at every character boundary, which is never the claim being made"
        ));
    }
    Ok(needle)
}

// `kind` is `Option` here, not defaulted straight to `StepKind::Cli` the way
// the public `Step` below has it. `finalize_step` is what applies the
// default -- and only when no kind-specific field (`path`/`fields`/
// `entry`/`body`) is present. That distinction is what turns "a step sets
// `entry`/`body` but forgot `kind = \"browser_bundle_harness\"`" into a hard
// error instead of a silently-misinterpreted `cli` step: see `finalize_step`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStep {
    #[serde(default)]
    kind: Option<StepKind>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    exit: Option<Exit>,
    #[serde(default)]
    stdout: Option<String>,
    #[serde(default)]
    stdout_contains: Vec<String>,
    #[serde(default)]
    stdout_absent: Vec<String>,
    #[serde(default)]
    stdout_count: Vec<RawCountClaim>,
    #[serde(default)]
    stderr: Option<String>,
    #[serde(default)]
    stderr_contains: Vec<String>,
    #[serde(default)]
    stderr_absent: Vec<String>,
    #[serde(default)]
    json: Option<toml::Value>,
    #[serde(default)]
    json_null: Vec<String>,
    #[serde(default)]
    json_count: Vec<RawJsonCountClaim>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    fields: Option<toml::Value>,
    #[serde(default)]
    entry: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug)]
pub struct Step {
    pub kind: StepKind,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub exit: Option<Exit>,
    pub stdout: Option<String>,
    pub stdout_contains: Vec<String>,
    pub stdout_absent: Vec<String>,
    /// Occurrence-count claims against the step's captured stdout, one per
    /// `{ needle = "...", at_least | exact = n }` table. Added when the
    /// `.matches(needle).count()` shape -- 32 sites across 13 otherwise
    /// un-migratable browser files -- turned out to have no expressible
    /// form: `stdout_contains` is satisfied by a *single* occurrence, so
    /// migrating `count() >= 2` to it silently weakens the claim to
    /// `count() >= 1`, and output that folded two constants into one would
    /// still pass. See `assertions::check_count` for the counting semantics.
    pub stdout_count: Vec<CountClaim>,
    /// Exact stderr equality, symmetric with `stdout` above. Added when a
    /// migrated source assertion (`stderr.is_empty()`, or any exact-stderr
    /// claim) had no expressible form: `stderr_contains`/`stderr_absent`
    /// are substring claims, neither of which can pin "stderr is exactly
    /// this string" (in particular "stderr is exactly empty" -- a stray
    /// unrelated diagnostic on stderr would satisfy every `stderr_absent`
    /// needle the source never wrote and still pass). See `check`'s doc
    /// comment for why this is a separate field from `stderr_contains`
    /// rather than an overload of it.
    pub stderr: Option<String>,
    pub stderr_contains: Vec<String>,
    pub stderr_absent: Vec<String>,
    pub json: Option<toml::Value>,
    /// Dotted paths (jsonpath.rs) that must resolve to a JSON `null` in the
    /// step's captured stdout. TOML has no null literal, so a claim like
    /// `json["stderr"].is_null()` cannot be written inside `json` (a
    /// `toml::Value` expectation) at all -- `values_equal` in jsonpath.rs
    /// hard-rejects every TOML type against a JSON null by construction.
    /// This is that claim's only expressible form; see `check`'s doc
    /// comment on why it is deliberately not folded into `json` itself.
    pub json_null: Vec<String>,
    /// `stdout_count`'s claim taken against a JSON string leaf of the
    /// captured stdout instead of raw stdout. Both surfaces exist because a
    /// single migrated helper asserts the same count on both branches of its
    /// `--output json` split (`browser_math_log2_log10.rs:177-179` against
    /// `json["stdout"].as_str()`, `:186` against the raw stdout); a key that
    /// covered only one surface would leave half of every such helper
    /// hand-written. A path that does not resolve, or resolves to a
    /// non-string, is a hard failure -- §5.10, the same rule `json_null`
    /// follows.
    pub json_count: Vec<JsonCountClaim>,
    /// `file_json` only.
    pub path: Option<String>,
    /// `file_json` only.
    pub fields: Option<toml::Value>,
    /// `browser_bundle_harness` only.
    pub entry: Option<String>,
    /// `browser_bundle_harness` only.
    pub body: Option<String>,
}

/// Resolves a `RawStep`'s effective `kind` and rejects fields that don't
/// apply to it.
///
/// Two failure modes this closes, both first reported against a real case
/// file rather than anticipated up front:
///
/// - A `file_json` step (reads a file, asserts on it) declaring `stdout*`,
///   `exit`, or other process-output assertions: the step never runs a
///   process, so those assertions would never be evaluated -- parses clean,
///   asserts nothing, exactly the degradation this format exists to close.
/// - A step setting `entry`/`body` (`browser_bundle_harness`-only fields)
///   or `path`/`fields` (`file_json`-only fields) without an explicit
///   `kind`: `kind` defaults to `cli`, so a forgotten `kind =
///   "browser_bundle_harness"` silently becomes a `cli` step that ignores
///   `entry`/`body` entirely. `kind` therefore only defaults to `cli` when
///   *no* kind-specific field is present; otherwise it must be spelled out.
fn finalize_step(raw: RawStep, case_name: &str) -> Result<Step, String> {
    let wants_file_json = raw.path.is_some() || raw.fields.is_some();
    let wants_browser = raw.entry.is_some() || raw.body.is_some();

    let kind = match raw.kind {
        Some(kind) => kind,
        None if wants_file_json && wants_browser => {
            return Err(format!(
                "case `{case_name}`: step sets both `path`/`fields` (file_json-only) and \
                 `entry`/`body` (browser_bundle_harness-only) without an explicit `kind`"
            ));
        }
        None if wants_file_json => {
            return Err(format!(
                "case `{case_name}`: step sets `path` or `fields`, which requires an explicit \
                 `kind = \"file_json\"` -- `kind` only defaults to `cli` when no kind-specific \
                 field is set"
            ));
        }
        None if wants_browser => {
            return Err(format!(
                "case `{case_name}`: step sets `entry` or `body`, which requires an explicit \
                 `kind = \"browser_bundle_harness\"` -- `kind` only defaults to `cli` when no \
                 kind-specific field is set"
            ));
        }
        None => StepKind::default(),
    };

    // Field applicability by kind: `cli` and `browser_bundle_harness` both
    // run a process and can assert on its exit/stdout/stderr (including
    // `json`/`json_null`/`json_count`, all read from that process's captured
    // stdout);
    // `file_json` reads a file off disk and never runs anything, so none of
    // that applies to it. `args` is `cli`-only (it's the argv passed to
    // `kali`). `path`/`fields` are `file_json`-only; `entry`/`body` are
    // `browser_bundle_harness`-only.
    let mut inapplicable: Vec<&'static str> = Vec::new();
    match kind {
        StepKind::Cli => {
            if raw.path.is_some() {
                inapplicable.push("path");
            }
            if raw.fields.is_some() {
                inapplicable.push("fields");
            }
            if raw.entry.is_some() {
                inapplicable.push("entry");
            }
            if raw.body.is_some() {
                inapplicable.push("body");
            }
        }
        StepKind::FileJson => {
            if !raw.args.is_empty() {
                inapplicable.push("args");
            }
            if !raw.env.is_empty() {
                inapplicable.push("env");
            }
            if raw.exit.is_some() {
                inapplicable.push("exit");
            }
            if raw.stdout.is_some() {
                inapplicable.push("stdout");
            }
            if !raw.stdout_contains.is_empty() {
                inapplicable.push("stdout_contains");
            }
            if !raw.stdout_absent.is_empty() {
                inapplicable.push("stdout_absent");
            }
            if !raw.stdout_count.is_empty() {
                inapplicable.push("stdout_count");
            }
            if raw.stderr.is_some() {
                inapplicable.push("stderr");
            }
            if !raw.stderr_contains.is_empty() {
                inapplicable.push("stderr_contains");
            }
            if !raw.stderr_absent.is_empty() {
                inapplicable.push("stderr_absent");
            }
            if raw.json.is_some() {
                inapplicable.push("json");
            }
            if !raw.json_null.is_empty() {
                inapplicable.push("json_null");
            }
            if !raw.json_count.is_empty() {
                inapplicable.push("json_count");
            }
            if raw.entry.is_some() {
                inapplicable.push("entry");
            }
            if raw.body.is_some() {
                inapplicable.push("body");
            }
        }
        StepKind::BrowserBundleHarness => {
            if !raw.args.is_empty() {
                inapplicable.push("args");
            }
            if raw.path.is_some() {
                inapplicable.push("path");
            }
            if raw.fields.is_some() {
                inapplicable.push("fields");
            }
        }
    }
    if !inapplicable.is_empty() {
        return Err(format!(
            "case `{case_name}`: step (kind = \"{}\") sets field(s) that do not apply to that \
             kind: {}",
            kind.as_str(),
            inapplicable.join(", ")
        ));
    }

    let stdout_count = raw
        .stdout_count
        .into_iter()
        .map(|claim| {
            Ok(CountClaim {
                needle: count_needle(claim.needle, case_name, "stdout_count")?,
                bound: count_bound(claim.at_least, claim.exact, case_name, "stdout_count")?,
            })
        })
        .collect::<Result<Vec<CountClaim>, String>>()?;
    let json_count = raw
        .json_count
        .into_iter()
        .map(|claim| {
            Ok(JsonCountClaim {
                path: claim.path,
                needle: count_needle(claim.needle, case_name, "json_count")?,
                bound: count_bound(claim.at_least, claim.exact, case_name, "json_count")?,
            })
        })
        .collect::<Result<Vec<JsonCountClaim>, String>>()?;

    Ok(Step {
        kind,
        args: raw.args,
        env: raw.env,
        exit: raw.exit,
        stdout: raw.stdout,
        stdout_contains: raw.stdout_contains,
        stdout_absent: raw.stdout_absent,
        stdout_count,
        stderr: raw.stderr,
        stderr_contains: raw.stderr_contains,
        stderr_absent: raw.stderr_absent,
        json: raw.json,
        json_null: raw.json_null,
        json_count,
        path: raw.path,
        fields: raw.fields,
        entry: raw.entry,
        body: raw.body,
    })
}

pub fn parse_case_file(text: &str) -> Result<CaseFile, String> {
    let raw: RawCaseFile = toml::from_str(text).map_err(|error| error.to_string())?;
    if raw.case.is_empty() {
        return Err("case file declares no [[case]] entries".to_string());
    }
    for (axis, values) in &raw.matrix {
        if values.is_empty() {
            return Err(format!("matrix axis `{axis}` has no values"));
        }
    }

    let mut cases = Vec::with_capacity(raw.case.len());
    let mut seen_names = std::collections::BTreeSet::new();
    for raw_case in raw.case {
        if !seen_names.insert(raw_case.name.clone()) {
            return Err(format!("duplicate case name `{}`", raw_case.name));
        }

        let inline_raw: Option<RawStep> = if raw_case.rest.is_empty() {
            None
        } else {
            let raw_step: RawStep = toml::Value::Table(raw_case.rest)
                .try_into()
                .map_err(|error| format!("case `{}`: {error}", raw_case.name))?;
            Some(raw_step)
        };

        let mut step = Vec::with_capacity(raw_case.step.len());
        for raw_step in raw_case.step {
            step.push(finalize_step(raw_step, &raw_case.name)?);
        }
        let inline = inline_raw
            .map(|raw_step| finalize_step(raw_step, &raw_case.name))
            .transpose()?;

        if step.is_empty() && inline.is_none() {
            return Err(format!("case `{}` declares no step", raw_case.name));
        }
        if !step.is_empty() && inline.is_some() {
            return Err(format!(
                "case `{}` mixes [[case.step]] with inline step fields",
                raw_case.name
            ));
        }
        cases.push(Case {
            name: raw_case.name,
            rationale: raw_case.rationale,
            ignore: raw_case.ignore,
            step,
            inline,
        });
    }

    Ok(CaseFile {
        constants: raw.constants,
        matrix: raw.matrix,
        source: raw.source,
        case: cases,
    })
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;
