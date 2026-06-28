//! CLI artifacts/timings/diagnostic-array validators.

use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

use super::diagnostic::*;
use super::schema::*;

pub(crate) fn validate_diagnostic_array(value: Option<&Value>, field: &str) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("CLI envelope {field} must be an array"));
    };

    let mut previous_sort_key: Option<(String, u64, u64, String)> = None;
    for (index, item) in items.iter().enumerate() {
        validate_diagnostic_value(item)
            .map_err(|err| format!("CLI envelope {field}[{index}] is invalid: {err}"))?;

        let sort_key = diagnostic_sort_key(item)
            .map_err(|err| format!("CLI envelope {field}[{index}] is invalid: {err}"))?;
        if previous_sort_key
            .as_ref()
            .is_some_and(|previous| previous > &sort_key)
        {
            let (previous_file, previous_line, previous_column, previous_code) =
                previous_sort_key.expect("validated above");
            let (current_file, current_line, current_column, current_code) = &sort_key;
            return Err(format!(
                "CLI envelope {field}[{index}] must be sorted by file, line, column, then code; got file `{current_file}`, line {current_line}, column {current_column}, code `{current_code}` after file `{previous_file}`, line {previous_line}, column {previous_column}, code `{previous_code}`"
            ));
        }
        previous_sort_key = Some(sort_key);
    }

    Ok(())
}

pub(crate) fn sort_diagnostic_array_value(value: &mut Value) {
    let Some(items) = value.as_array_mut() else {
        return;
    };

    if items.iter().all(|item| diagnostic_sort_key(item).is_ok()) {
        items.sort_by(|left, right| {
            diagnostic_sort_key(left)
                .expect("validated above")
                .cmp(&diagnostic_sort_key(right).expect("validated above"))
        });
    }
}

pub(crate) fn diagnostic_sort_key(value: &Value) -> Result<(String, u64, u64, String), String> {
    let Some(object) = value.as_object() else {
        return Err("diagnostic must be a JSON object".to_string());
    };
    let Some(span) = object.get("span").and_then(Value::as_object) else {
        return Err("diagnostic is missing required key `span`".to_string());
    };

    let file = span
        .get("file")
        .and_then(Value::as_str)
        .ok_or_else(|| "diagnostic span.file must be a string".to_string())?
        .to_string();
    let line = span
        .get("line")
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_i64().map(|value| value as u64))
        })
        .ok_or_else(|| "diagnostic span.line must be a positive integer".to_string())?;
    let column = span
        .get("column")
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_i64().map(|value| value as u64))
        })
        .ok_or_else(|| "diagnostic span.column must be a positive integer".to_string())?;
    let code = object
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| "diagnostic code must be a canonical code string".to_string())?
        .to_string();

    Ok((file, line, column, code))
}

pub(crate) fn validate_cli_artifacts_array(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };

    let Some(items) = value.as_array() else {
        return Err("CLI envelope artifacts must be an array".to_string());
    };

    let mut seen_primary_executable = false;
    let mut seen_primary_library = false;
    let mut seen_primary_component = false;
    let mut seen_kind_path_pairs = BTreeSet::new();
    let mut previous_sort_key: Option<(usize, String, String)> = None;

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!(
                "CLI envelope artifacts[{index}] must be a JSON object"
            ));
        };

        reject_unexpected_keys(
            object,
            &["path", "kind", "role", "bytes"],
            "CLI envelope artifact",
        )?;

        match object.get("path") {
            Some(Value::String(_)) => {
                validate_canonical_non_empty_string_value(
                    object.get("path"),
                    &format!("CLI envelope artifacts[{index}].path"),
                )?;
            }
            Some(other) => {
                return Err(format!(
                    "CLI envelope artifacts[{index}].path must be a string, got {other}"
                ))
            }
            None => {
                return Err(format!(
                    "CLI envelope artifacts[{index}] is missing required key `path`"
                ))
            }
        }
        match object.get("kind") {
            Some(Value::String(_)) => {
                validate_canonical_non_empty_string_value(
                    object.get("kind"),
                    &format!("CLI envelope artifacts[{index}].kind"),
                )?;
            }
            Some(other) => {
                return Err(format!(
                    "CLI envelope artifacts[{index}].kind must be a string, got {other}"
                ))
            }
            None => {
                return Err(format!(
                    "CLI envelope artifacts[{index}] is missing required key `kind`"
                ))
            }
        }
        match object.get("bytes") {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!(
                "CLI envelope artifacts[{index}].bytes must be a non-negative integer, got {other}"
            ))
            }
            None => {
                return Err(format!(
                    "CLI envelope artifacts[{index}] is missing required key `bytes`"
                ))
            }
        }

        let kind = object
            .get("kind")
            .and_then(Value::as_str)
            .expect("validated above");
        let path = object
            .get("path")
            .and_then(Value::as_str)
            .expect("validated above");
        if !seen_kind_path_pairs.insert((kind.to_string(), path.to_string())) {
            return Err(format!(
                "CLI envelope artifacts[{index}] duplicates artifact `{kind}` at `{path}`"
            ));
        }

        if let Some(role) = object.get("role") {
            match role {
                Value::String(role) => {
                    let role_value = Value::String(role.clone());
                    validate_canonical_non_empty_string_value(
                        Some(&role_value),
                        &format!("CLI envelope artifacts[{index}].role"),
                    )?;
                    if !is_canonical_artifact_role(role) {
                        return Err(format!(
                            "CLI envelope artifacts[{index}].role must be a canonical schema-v1 role, got `{role}`"
                        ));
                    }
                    match role.as_str() {
                        "primary-executable" => {
                            if seen_primary_executable {
                                return Err(format!("CLI envelope artifacts[{index}].role duplicates primary-executable"));
                            }
                            seen_primary_executable = true;
                        }
                        "primary-library" => {
                            if seen_primary_library {
                                return Err(format!("CLI envelope artifacts[{index}].role duplicates primary-library"));
                            }
                            seen_primary_library = true;
                        }
                        "primary-component" => {
                            if seen_primary_component {
                                return Err(format!("CLI envelope artifacts[{index}].role duplicates primary-component"));
                            }
                            seen_primary_component = true;
                        }
                        _ => {}
                    }
                }
                other => {
                    return Err(format!(
                        "CLI envelope artifacts[{index}].role must be a string, got {other}"
                    ))
                }
            }
        }

        let sort_key = artifact_sort_key(object);
        if let Some(previous_sort_key) = &previous_sort_key {
            if previous_sort_key >= &sort_key {
                return Err(format!(
                    "CLI envelope artifacts[{index}] must be sorted by role, kind, then path; got role rank {}, kind `{}`, path `{}` after role rank {}, kind `{}`, path `{}`",
                    sort_key.0,
                    sort_key.1,
                    sort_key.2,
                    previous_sort_key.0,
                    previous_sort_key.1,
                    previous_sort_key.2
                ));
            }
        }
        previous_sort_key = Some(sort_key);
    }

    Ok(())
}

pub(crate) fn is_canonical_artifact_role(role: &str) -> bool {
    matches!(
        role,
        "primary-executable"
            | "primary-library"
            | "primary-component"
            | "browser-glue"
            | "interface-wit"
            | "embedding-header"
            | "embedding-metadata"
            | "binding-package-manifest"
            | "debug-source-map"
    )
}

pub(crate) fn artifact_sort_key(object: &serde_json::Map<String, Value>) -> (usize, String, String) {
    let role_rank = object
        .get("role")
        .and_then(Value::as_str)
        .map(artifact_role_rank)
        .unwrap_or(usize::MAX);
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let path = object
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    (role_rank, kind, path)
}

pub(crate) fn artifact_role_rank(role: &str) -> usize {
    match role {
        "primary-executable" => 0,
        "primary-library" => 1,
        "primary-component" => 2,
        "browser-glue" => 3,
        "interface-wit" => 4,
        "embedding-header" => 5,
        "embedding-metadata" => 6,
        "binding-package-manifest" => 7,
        "debug-source-map" => 8,
        _ => usize::MAX,
    }
}

pub(crate) fn timing_sort_key(phase: &str) -> (usize, String) {
    (timing_phase_rank(phase), phase.to_string())
}

pub(crate) fn timing_phase_rank(phase: &str) -> usize {
    match phase {
        "parse" => 0,
        "typecheck" => 1,
        _ => usize::MAX,
    }
}

pub(crate) fn validate_timings_array(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };

    let Some(items) = value.as_array() else {
        return Err("CLI envelope timings must be an array".to_string());
    };

    let mut seen_phases = HashSet::new();
    let mut previous_sort_key: Option<(usize, String)> = None;
    let mut previous_phase: Option<String> = None;
    for (index, item) in items.iter().enumerate() {
        let phase = validate_timing_value(item)
            .map_err(|err| format!("CLI envelope timings[{index}] is invalid: {err}"))?;
        if !seen_phases.insert(phase.clone()) {
            return Err(format!(
                "CLI envelope timings[{index}] duplicates phase `{phase}`"
            ));
        }

        let sort_key = timing_sort_key(&phase);
        if previous_sort_key
            .as_ref()
            .is_some_and(|previous| previous > &sort_key)
        {
            let previous_phase =
                previous_phase.expect("previous phase is always set with previous sort key");
            return Err(format!(
                "CLI envelope timings[{index}] must be in canonical phase order, got `{previous_phase}` before `{phase}`"
            ));
        }
        previous_sort_key = Some(sort_key);
        previous_phase = Some(phase);
    }

    Ok(())
}

pub(crate) fn validate_timing_value(value: &Value) -> Result<String, String> {
    let Some(object) = value.as_object() else {
        return Err("timing must be a JSON object".to_string());
    };

    for key in ["phase", "milliseconds"] {
        if !object.contains_key(key) {
            return Err(format!("timing is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &["phase", "milliseconds"], "timing")?;

    let phase = match object.get("phase") {
        Some(Value::String(value)) if !value.trim().is_empty() && value.trim() == value => {
            value.clone()
        }
        Some(Value::String(_)) => {
            return Err("timing phase must be a non-empty, non-whitespace string".to_string())
        }
        Some(other) => return Err(format!("timing phase must be a string, got {other}")),
        None => unreachable!("validated above"),
    };

    match object.get("milliseconds") {
        Some(Value::Number(value))
            if value
                .as_f64()
                .is_some_and(|milliseconds| milliseconds.is_finite() && milliseconds >= 0.0) => {}
        Some(Value::Number(_)) => {
            return Err("timing milliseconds must be a finite non-negative number".to_string())
        }
        Some(other) => return Err(format!("timing milliseconds must be a number, got {other}")),
        None => unreachable!("validated above"),
    }

    Ok(phase)
}
