//! Diagnostic value validators (diagnostic, span, labels, related, fixes, context).

use serde_json::Value;

use super::schema::*;

pub(crate) fn validate_diagnostic_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("diagnostic must be a JSON object".to_string());
    };

    for key in [
        "severity", "code", "message", "span", "labels", "related", "fix", "notes",
    ] {
        if !object.contains_key(key) {
            return Err(format!("diagnostic is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "severity", "code", "message", "span", "file", "labels", "help", "related", "fix",
            "notes", "context",
        ],
        "diagnostic",
    )?;

    match object.get("severity") {
        Some(Value::String(value))
            if matches!(value.as_str(), "error" | "warning" | "info" | "hint") => {}
        Some(other) => {
            return Err(format!(
                "diagnostic severity must be a canonical severity string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("code") {
        Some(Value::String(value))
            if value.len() == 5
                && matches!(value.as_bytes().first(), Some(b'E' | b'W' | b'I' | b'H'))
                && value[1..].chars().all(|ch| ch.is_ascii_digit()) => {}
        Some(other) => {
            return Err(format!(
                "diagnostic code must be a canonical code string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("message") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("diagnostic message must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    validate_source_span(
        object
            .get("span")
            .ok_or_else(|| "diagnostic is missing required key `span`".to_string())?,
    )?;

    if let Some(file) = object.get("file") {
        match file {
            Value::String(file)
                if object
                    .get("span")
                    .and_then(Value::as_object)
                    .and_then(|span| span.get("file"))
                    .and_then(Value::as_str)
                    .is_some_and(|span_file| span_file == file) => {}
            Value::String(file) => {
                return Err(format!(
                    "diagnostic file mirror must match span.file, got `{file}`"
                ))
            }
            other => {
                return Err(format!(
                    "diagnostic file mirror must be a string, got {other}"
                ))
            }
        }
    }

    validate_diagnostic_label_array(object.get("labels"))?;
    validate_related_info_array(object.get("related"))?;
    validate_suggested_fix(object.get("fix"))?;

    match object.get("notes") {
        Some(Value::Array(notes)) if notes.iter().all(|note| matches!(note, Value::String(_))) => {}
        Some(other) => {
            return Err(format!(
                "diagnostic notes must be an array of strings, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("help") {
        Some(Value::Null) | Some(Value::String(_)) | None => {}
        Some(other) => {
            return Err(format!(
                "diagnostic help must be a string or null, got {other}"
            ))
        }
    }

    if let Some(context) = object.get("context") {
        validate_diagnostic_context(context)?;
    }

    Ok(())
}

pub(crate) fn is_positive_integer(value: &Value) -> bool {
    matches!(
        value,
        Value::Number(number)
            if number.as_u64().is_some_and(|value| value >= 1)
                || number.as_i64().is_some_and(|value| value >= 1)
    )
}

pub(crate) fn positive_integer_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64().or_else(|| {
            number
                .as_i64()
                .and_then(|value| (value >= 0).then_some(value as u64))
        }),
        _ => None,
    }
}

pub(crate) fn validate_source_span(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("span must be a JSON object".to_string());
    };

    for key in ["file", "line", "column", "endLine", "endColumn"] {
        if !object.contains_key(key) {
            return Err(format!("span is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &["file", "line", "column", "endLine", "endColumn"],
        "span",
    )?;

    match object.get("file") {
        Some(Value::String(value)) if !value.trim().is_empty() => {}
        Some(Value::String(_)) => {
            return Err("span file must be a non-empty, non-whitespace string".to_string())
        }
        Some(other) => return Err(format!("span file must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    for key in ["line", "column", "endLine", "endColumn"] {
        match object.get(key) {
            Some(value) if is_positive_integer(value) => {}
            Some(other) => {
                return Err(format!(
                    "span {key} must be a positive integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    let line = positive_integer_value(object.get("line").expect("validated above"))
        .expect("validated above");
    let column = positive_integer_value(object.get("column").expect("validated above"))
        .expect("validated above");
    let end_line = positive_integer_value(object.get("endLine").expect("validated above"))
        .expect("validated above");
    let end_column = positive_integer_value(object.get("endColumn").expect("validated above"))
        .expect("validated above");

    if end_line < line || (end_line == line && end_column < column) {
        return Err("span end position must not precede its start position".to_string());
    }

    Ok(())
}

pub(crate) fn validate_label_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("label must be a JSON object".to_string());
    };

    for key in ["span", "message", "style"] {
        if !object.contains_key(key) {
            return Err(format!("label is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &["span", "message", "style"], "label")?;

    validate_source_span(
        object
            .get("span")
            .ok_or_else(|| "label is missing required key `span`".to_string())?,
    )?;

    match object.get("message") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("label message must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    match object.get("style") {
        Some(Value::String(value)) if matches!(value.as_str(), "primary" | "secondary") => {}
        Some(other) => {
            return Err(format!(
                "label style must be `primary` or `secondary`, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}

pub(crate) fn validate_diagnostic_label_array(value: Option<&Value>) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err("diagnostic labels must be an array".to_string());
    };

    for (index, item) in items.iter().enumerate() {
        validate_label_value(item)
            .map_err(|err| format!("diagnostic labels[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

pub(crate) fn validate_related_info_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("related item must be a JSON object".to_string());
    };

    for key in ["message", "span"] {
        if !object.contains_key(key) {
            return Err(format!("related item is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &["message", "span"], "related item")?;

    match object.get("message") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "related item message must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    validate_source_span(
        object
            .get("span")
            .ok_or_else(|| "related item is missing required key `span`".to_string())?,
    )?;
    Ok(())
}

pub(crate) fn validate_related_info_array(value: Option<&Value>) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err("diagnostic related must be an array".to_string());
    };

    for (index, item) in items.iter().enumerate() {
        validate_related_info_value(item)
            .map_err(|err| format!("diagnostic related[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

pub(crate) fn validate_text_edit_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("text edit must be a JSON object".to_string());
    };

    reject_unexpected_keys(object, &["file", "start", "end", "newText"], "text edit")?;

    for key in ["file", "start", "end", "newText"] {
        if !object.contains_key(key) {
            return Err(format!("text edit is missing required key `{key}`"));
        }
    }

    let file = match object.get("file") {
        Some(Value::String(file)) => file,
        Some(other) => return Err(format!("text edit file must be a string, got {other}")),
        None => unreachable!("validated above"),
    };

    let start = object
        .get("start")
        .ok_or_else(|| "text edit is missing required key `start`".to_string())?;
    validate_source_location_file_mirror(start, file, "text edit start")?;

    let end = object
        .get("end")
        .ok_or_else(|| "text edit is missing required key `end`".to_string())?;
    validate_source_location_file_mirror(end, file, "text edit end")?;
    validate_text_edit_location_order(start, end)?;

    match object.get("newText") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("text edit newText must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    Ok(())
}

pub(crate) fn validate_source_location_file_mirror(
    location: &Value,
    file: &str,
    location_name: &str,
) -> Result<(), String> {
    validate_source_location(location, location_name)?;

    let Some(location) = location.as_object() else {
        unreachable!("validated above")
    };

    match location.get("file") {
        Some(Value::String(location_file)) if location_file == file => {}
        Some(Value::String(location_file)) => {
            return Err(format!(
                "{location_name}.file must match text edit file, got `{location_file}`"
            ))
        }
        Some(other) => {
            return Err(format!(
                "{location_name}.file must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}

pub(crate) fn validate_source_location(value: &Value, context: &str) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err(format!("{context} must be a JSON object"));
    };

    for key in ["file", "line", "column"] {
        if !object.contains_key(key) {
            return Err(format!("{context} is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &["file", "line", "column"], context)?;

    match object.get("file") {
        Some(Value::String(value)) if !value.trim().is_empty() => {}
        Some(Value::String(_)) => {
            return Err(format!(
                "{context} source location file must be a non-empty, non-whitespace string"
            ))
        }
        Some(other) => {
            return Err(format!(
                "{context} source location file must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    for key in ["line", "column"] {
        match object.get(key) {
            Some(value) if is_positive_integer(value) => {}
            Some(other) => {
                return Err(format!(
                    "{context} source location {key} must be a positive integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

pub(crate) fn validate_text_edit_location_order(start: &Value, end: &Value) -> Result<(), String> {
    let start = source_location_position(start, "text edit start")?;
    let end = source_location_position(end, "text edit end")?;

    if end < start {
        Err("text edit end position must not precede its start position".to_string())
    } else {
        Ok(())
    }
}

pub(crate) fn validate_suggested_fix_edits_non_overlapping(edits: &[Value]) -> Result<(), String> {
    let mut ranges = Vec::with_capacity(edits.len());

    for (index, edit) in edits.iter().enumerate() {
        let Some(object) = edit.as_object() else {
            return Err(format!(
                "suggested fix edits[{index}] must be a JSON object"
            ));
        };

        let file = match object.get("file") {
            Some(Value::String(file)) => file.clone(),
            Some(other) => {
                return Err(format!(
                    "suggested fix edits[{index}].file must be a string, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        };

        let start = source_location_position(
            object.get("start").ok_or_else(|| {
                format!("suggested fix edits[{index}] is missing required key `start`")
            })?,
            "text edit start",
        )?;
        let end = source_location_position(
            object.get("end").ok_or_else(|| {
                format!("suggested fix edits[{index}] is missing required key `end`")
            })?,
            "text edit end",
        )?;

        ranges.push((file, start, end, index));
    }

    ranges.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(left.1.cmp(&right.1))
            .then(left.2.cmp(&right.2))
            .then(left.3.cmp(&right.3))
    });

    for pair in ranges.windows(2) {
        let (previous_file, previous_start, previous_end, previous_index) = &pair[0];
        let (current_file, current_start, current_end, current_index) = &pair[1];

        if previous_file == current_file
            && (current_start < previous_end
                || (previous_start == previous_end
                    && current_start == previous_end
                    && current_start == current_end))
        {
            return Err(format!(
                "suggested fix edits[{current_index}] overlaps with suggested fix edits[{previous_index}]"
            ));
        }
    }

    Ok(())
}

pub(crate) fn source_location_position(value: &Value, location_name: &str) -> Result<(u64, u64), String> {
    validate_source_location(value, location_name)?;

    let Some(object) = value.as_object() else {
        unreachable!("validated above")
    };

    let line =
        positive_integer_value(object.get("line").expect("validated above")).ok_or_else(|| {
            format!("{location_name} source location line must be a positive integer")
        })?;
    let column = positive_integer_value(object.get("column").expect("validated above"))
        .ok_or_else(|| {
            format!("{location_name} source location column must be a positive integer")
        })?;

    Ok((line, column))
}

pub(crate) fn validate_suggested_fix(value: Option<&Value>) -> Result<(), String> {
    match value {
        Some(Value::Null) | None => Ok(()),
        Some(Value::Object(object)) => {
            for key in ["message", "edits"] {
                if !object.contains_key(key) {
                    return Err(format!("suggested fix is missing required key `{key}`"));
                }
            }
            reject_unexpected_keys(object, &["message", "edits"], "suggested fix")?;

            match object.get("message") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "suggested fix message must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }

            match object.get("edits") {
                Some(Value::Array(edits)) => {
                    for (index, edit) in edits.iter().enumerate() {
                        validate_text_edit_value(edit).map_err(|err| {
                            format!("suggested fix edits[{index}] is invalid: {err}")
                        })?;
                    }
                    validate_suggested_fix_edits_non_overlapping(edits)?;
                }
                Some(other) => {
                    return Err(format!("suggested fix edits must be an array, got {other}"))
                }
                None => unreachable!("validated above"),
            }

            Ok(())
        }
        Some(other) => Err(format!(
            "suggested fix must be null or an object, got {other}"
        )),
    }
}

pub(crate) fn validate_diagnostic_context(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("diagnostic context must be a JSON object".to_string());
    };

    const CANONICAL_DIAGNOSTIC_CONTEXT_ORIGINS: [&str; 4] = ["cli", "config", "default", "source"];

    reject_unexpected_keys(
        object,
        &[
            "origin",
            "configPath",
            "flag",
            "requestedValue",
            "effectiveValue",
        ],
        "diagnostic context",
    )?;

    match object.get("origin") {
        Some(Value::String(value))
            if CANONICAL_DIAGNOSTIC_CONTEXT_ORIGINS.contains(&value.as_str()) => {}
        Some(other) => {
            return Err(format!(
                "diagnostic context origin must be a canonical origin string, got {other}"
            ))
        }
        None => return Err("diagnostic context is missing required key `origin`".to_string()),
    }

    for key in ["configPath", "flag"] {
        validate_optional_non_empty_string_value(
            object.get(key),
            &format!("diagnostic context {key}"),
        )?;
    }

    // schema v1 allows requestedValue/effectiveValue to carry any JSON shape, so
    // validation is intentionally permissive here.
    Ok(())
}
