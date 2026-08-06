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
//! that table into `Step` with `toml::Value::Table(rest).try_into::<Step>()`
//! -- a plain `Deserialize` call, outside the flatten `Content` buffer,
//! which *does* honor `Step`'s `deny_unknown_fields`. That conversion is
//! the only hand-written part of this module; everything else is ordinary
//! derived `Deserialize`. Its correctness (every `Step` field actually
//! carried through, unknown keys in both the inline and `[[case.step]]`
//! forms rejected, wrong-typed known keys rejected without panicking) is
//! covered directly in `model_tests.rs`, not just via the happy path.

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
    step: Vec<Step>,
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

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    #[default]
    Cli,
    FileJson,
    BrowserBundleHarness,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(default)]
    pub kind: StepKind,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub exit: Option<Exit>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stdout_contains: Vec<String>,
    #[serde(default)]
    pub stdout_absent: Vec<String>,
    #[serde(default)]
    pub stderr_contains: Vec<String>,
    #[serde(default)]
    pub stderr_absent: Vec<String>,
    #[serde(default)]
    pub json: Option<toml::Value>,
    /// `file_json` only.
    #[serde(default)]
    pub path: Option<String>,
    /// `file_json` only.
    #[serde(default)]
    pub fields: Option<toml::Value>,
    /// `browser_bundle_harness` only.
    #[serde(default)]
    pub entry: Option<String>,
    /// `browser_bundle_harness` only.
    #[serde(default)]
    pub body: Option<String>,
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
    for raw_case in raw.case {
        let inline = if raw_case.rest.is_empty() {
            None
        } else {
            let step: Step = toml::Value::Table(raw_case.rest)
                .try_into()
                .map_err(|error| format!("case `{}`: {error}", raw_case.name))?;
            Some(step)
        };
        if raw_case.step.is_empty() && inline.is_none() {
            return Err(format!("case `{}` declares no step", raw_case.name));
        }
        if !raw_case.step.is_empty() && inline.is_some() {
            return Err(format!(
                "case `{}` mixes [[case.step]] with inline step fields",
                raw_case.name
            ));
        }
        cases.push(Case {
            name: raw_case.name,
            rationale: raw_case.rationale,
            ignore: raw_case.ignore,
            step: raw_case.step,
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
