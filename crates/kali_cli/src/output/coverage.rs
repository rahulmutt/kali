//! Test-payload coverage validator.

use serde_json::Value;
use std::collections::HashSet;

use super::schema::*;

pub(crate) fn validate_test_payload_coverage_value(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Err(format!(
            "test payload coverage must be a JSON object, got {value}"
        ));
    };

    for key in ["mode", "files", "summary"] {
        if !object.contains_key(key) {
            return Err(format!(
                "test payload coverage is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        object,
        &["mode", "files", "summary"],
        "test payload coverage",
    )?;

    match object.get("mode") {
        Some(Value::String(mode)) if mode == "function" => {}
        Some(Value::String(mode)) => {
            return Err(format!(
                "test payload coverage mode must be 'function', got '{mode}'"
            ));
        }
        Some(other) => {
            return Err(format!(
                "test payload coverage mode must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    let Some(Value::Array(items)) = object.get("files") else {
        return Err(format!(
            "test payload coverage files must be an array, got {}",
            object.get("files").unwrap()
        ));
    };
    let mut seen_files = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let Some(file) = item.as_object() else {
            return Err(format!(
                "test payload coverage files[{index}] must be an object, got {item}"
            ));
        };
        for key in [
            "file",
            "functionsTotal",
            "functionsCovered",
            "functionsMissed",
        ] {
            if !file.contains_key(key) {
                return Err(format!(
                    "test payload coverage files[{index}] is missing required key `{key}`"
                ));
            }
        }
        let file_context = format!("test payload coverage files[{index}]");
        reject_unexpected_keys(
            file,
            &[
                "file",
                "functionsTotal",
                "functionsCovered",
                "functionsMissed",
            ],
            &file_context,
        )?;
        let Some(Value::String(file_path)) = file.get("file") else {
            match file.get("file") {
                Some(other) => {
                    return Err(format!(
                        "test payload coverage files[{index}].file must be a string, got {other}"
                    ));
                }
                None => unreachable!("validated above"),
            }
        };
        validate_non_empty_string_value(
            file.get("file"),
            &format!("test payload coverage files[{index}].file"),
        )?;
        if !seen_files.insert(file_path.clone()) {
            return Err(format!(
                "test payload coverage files[{index}].file must be unique, got `{file_path}` twice"
            ));
        }
        for key in ["functionsTotal", "functionsCovered", "functionsMissed"] {
            match file.get(key) {
                Some(Value::Number(number))
                    if number.as_u64().is_some()
                        || number.as_i64().is_some_and(|value| value >= 0) => {}
                Some(other) => {
                    return Err(format!("test payload coverage files[{index}].{key} must be a non-negative integer, got {other}"));
                }
                None => unreachable!("validated above"),
            }
        }
    }

    let Some(summary) = object.get("summary") else {
        unreachable!("validated above")
    };
    let Some(summary) = summary.as_object() else {
        return Err(format!(
            "test payload coverage summary must be a JSON object, got {summary}"
        ));
    };
    for key in [
        "functionsTotal",
        "functionsCovered",
        "functionsMissed",
        "coveragePercent",
    ] {
        if !summary.contains_key(key) {
            return Err(format!(
                "test payload coverage summary is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        summary,
        &[
            "functionsTotal",
            "functionsCovered",
            "functionsMissed",
            "coveragePercent",
        ],
        "test payload coverage summary",
    )?;
    for key in ["functionsTotal", "functionsCovered", "functionsMissed"] {
        match summary.get(key) {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!("test payload coverage summary {key} must be a non-negative integer, got {other}"));
            }
            None => unreachable!("validated above"),
        }
    }
    match summary.get("coveragePercent") {
        Some(Value::Number(number)) if number.as_f64().is_some_and(|value| value >= 0.0) => {}
        Some(other) => {
            return Err(format!("test payload coverage summary coveragePercent must be a non-negative number, got {other}"));
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}
