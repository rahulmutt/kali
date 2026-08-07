//! Matrix expansion and `${...}` substitution.
//!
//! Substitution is closed at exactly two forms -- `${matrix_axis}` and
//! `${CONSTANT}` -- with no conditionals and no expressions. Variation that
//! changes assertions rather than substituting uniformly (text vs JSON output,
//! for instance) belongs in sibling `[[case]]` blocks, not here.

use crate::model::{Case, CaseFile, Step};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Trial {
    pub id: String,
    pub rationale: Option<String>,
    pub ignore: bool,
    pub source: BTreeMap<String, String>,
    pub steps: Vec<Step>,
}

/// Substitute every `${name}` from `bindings`; error on any survivor.
fn substitute(text: &str, bindings: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("unterminated `${{` in {text:?}"))?;
        let name = &after[..end];
        let value = bindings
            .get(name)
            .ok_or_else(|| format!("unresolved placeholder `${{{name}}}` in {text:?}"))?;
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Substitute string leaves (both keys and values) throughout a `toml::Value`
/// tree, recursing through tables and arrays. Non-string leaves (integers,
/// booleans, floats, datetimes) pass through untouched.
///
/// `json` and `fields` are two of the ten assertion keys (design spec
/// §5.4), and §5.10's unresolved-placeholder hard-failure
/// rule applies to them exactly as it does to every other string-bearing
/// field -- a `${...}` in a `json`/`fields` key is the more dangerous case,
/// since it would otherwise silently produce a JSON path that never matches
/// anything, letting the assertion pass while asserting nothing.
/// `json_null`'s dotted-path *strings* go through the plain `list()` helper
/// in `substitute_step`, not this function -- they are path expressions,
/// not JSON tree leaves.
fn substitute_value(
    value: &toml::Value,
    bindings: &BTreeMap<String, String>,
) -> Result<toml::Value, String> {
    match value {
        toml::Value::String(text) => Ok(toml::Value::String(substitute(text, bindings)?)),
        toml::Value::Array(items) => {
            let items = items
                .iter()
                .map(|item| substitute_value(item, bindings))
                .collect::<Result<Vec<toml::Value>, String>>()?;
            Ok(toml::Value::Array(items))
        }
        toml::Value::Table(table) => {
            let mut out = toml::Table::new();
            for (key, value) in table {
                out.insert(
                    substitute(key, bindings)?,
                    substitute_value(value, bindings)?,
                );
            }
            Ok(toml::Value::Table(out))
        }
        other => Ok(other.clone()),
    }
}

fn substitute_step(step: &Step, bindings: &BTreeMap<String, String>) -> Result<Step, String> {
    let list = |values: &Vec<String>| -> Result<Vec<String>, String> {
        values.iter().map(|v| substitute(v, bindings)).collect()
    };
    let opt = |value: &Option<String>| -> Result<Option<String>, String> {
        value
            .as_deref()
            .map(|v| substitute(v, bindings))
            .transpose()
    };
    let opt_value = |value: &Option<toml::Value>| -> Result<Option<toml::Value>, String> {
        value
            .as_ref()
            .map(|v| substitute_value(v, bindings))
            .transpose()
    };
    let mut env = BTreeMap::new();
    for (key, value) in &step.env {
        env.insert(substitute(key, bindings)?, substitute(value, bindings)?);
    }
    Ok(Step {
        kind: step.kind,
        args: list(&step.args)?,
        env,
        exit: step.exit,
        stdout: opt(&step.stdout)?,
        stdout_contains: list(&step.stdout_contains)?,
        stdout_absent: list(&step.stdout_absent)?,
        stderr: opt(&step.stderr)?,
        stderr_contains: list(&step.stderr_contains)?,
        stderr_absent: list(&step.stderr_absent)?,
        json: opt_value(&step.json)?,
        json_null: list(&step.json_null)?,
        path: opt(&step.path)?,
        fields: opt_value(&step.fields)?,
        entry: opt(&step.entry)?,
        body: opt(&step.body)?,
    })
}

/// Every combination of the matrix axes, sorted by axis name so trial ids are
/// deterministic across runs.
fn matrix_cells(matrix: &BTreeMap<String, Vec<String>>) -> Vec<Vec<(String, String)>> {
    let mut cells: Vec<Vec<(String, String)>> = vec![Vec::new()];
    for (axis, values) in matrix {
        let mut next = Vec::with_capacity(cells.len() * values.len());
        for cell in &cells {
            for value in values {
                let mut extended = cell.clone();
                extended.push((axis.clone(), value.clone()));
                next.push(extended);
            }
        }
        cells = next;
    }
    cells
}

fn steps_of(case: &Case) -> Vec<&Step> {
    if case.step.is_empty() {
        case.inline.iter().collect()
    } else {
        case.step.iter().collect()
    }
}

pub fn expand(stem: &str, file: &CaseFile) -> Result<Vec<Trial>, String> {
    let mut trials = Vec::new();
    for cell in matrix_cells(&file.matrix) {
        let mut bindings = file.constants.clone();
        for (axis, value) in &cell {
            bindings.insert(axis.clone(), value.clone());
        }

        let suffix = if cell.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = cell.iter().map(|(a, v)| format!("{a}={v}")).collect();
            format!("[{}]", pairs.join(","))
        };

        let mut source = BTreeMap::new();
        for (name, body) in &file.source {
            source.insert(substitute(name, &bindings)?, substitute(body, &bindings)?);
        }

        for case in &file.case {
            let steps = steps_of(case)
                .into_iter()
                .map(|step| substitute_step(step, &bindings))
                .collect::<Result<Vec<Step>, String>>()
                .map_err(|error| format!("{stem}{suffix}::{}: {error}", case.name))?;
            trials.push(Trial {
                id: format!("{stem}{suffix}::{}", case.name),
                rationale: case.rationale.clone(),
                ignore: case.ignore,
                source: source.clone(),
                steps,
            });
        }
    }
    Ok(trials)
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod expand_tests;
