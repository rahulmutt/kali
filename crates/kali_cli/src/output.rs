use kali_error::Diagnostic;
use serde_json::{json, Map, Value};
use std::path::Path;

use crate::{ColorChoice, OutputFormat};

#[derive(Clone, Debug)]
pub struct CliOutputOptions {
    pub format: OutputFormat,
    pub pretty: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub color: ColorChoice,
}

impl CliOutputOptions {
    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn emit_envelope_value(
    command: &str,
    success: bool,
    errors: Value,
    warnings: Value,
    payload: Value,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: i32,
) -> Value {
    if success && exit_code != 0 {
        panic!("CLI envelope success=true requires exitCode 0");
    }
    if !success && exit_code == 0 {
        panic!("CLI envelope success=false requires a non-zero exitCode");
    }

    let mut envelope = Map::new();
    envelope.insert("schemaVersion".to_string(), json!(1));
    envelope.insert("command".to_string(), json!(command));
    envelope.insert("success".to_string(), json!(success));
    envelope.insert("errors".to_string(), errors);
    envelope.insert("warnings".to_string(), warnings);
    envelope.insert("payload".to_string(), payload);
    envelope.insert(
        "stdout".to_string(),
        stdout.map_or(Value::Null, Value::String),
    );
    envelope.insert(
        "stderr".to_string(),
        stderr.map_or(Value::Null, Value::String),
    );
    envelope.insert("exitCode".to_string(), json!(exit_code));

    let value = Value::Object(envelope);
    validate_envelope_value(&value).expect("constructed CLI envelope must satisfy schema-v1 shape");
    value
}

pub(crate) fn validate_envelope_value(value: &Value) -> Result<(), String> {
    const REQUIRED_KEYS: [&str; 9] = [
        "schemaVersion",
        "command",
        "success",
        "errors",
        "warnings",
        "payload",
        "stdout",
        "stderr",
        "exitCode",
    ];

    let Some(object) = value.as_object() else {
        return Err("CLI envelope must be a JSON object".to_string());
    };

    for key in REQUIRED_KEYS {
        if !object.contains_key(key) {
            return Err(format!("CLI envelope is missing required key `{key}`"));
        }
    }

    match object.get("schemaVersion") {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope schemaVersion must be the numeric value 1, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    match object.get("command") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope command must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("success") {
        Some(Value::Bool(_)) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope success must be a boolean, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    validate_diagnostic_array(object.get("errors"), "errors")?;
    validate_diagnostic_array(object.get("warnings"), "warnings")?;

    match object.get("stdout") {
        Some(Value::Null) | Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope stdout must be string or null, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("stderr") {
        Some(Value::Null) | Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope stderr must be string or null, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    validate_timings_array(object.get("timings"))?;

    match object.get("exitCode") {
        Some(Value::Number(number)) if number.as_i64().is_some() || number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope exitCode must be an integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    let success = matches!(object.get("success"), Some(Value::Bool(true)));
    let exit_code_is_zero = matches!(object.get("exitCode"), Some(Value::Number(number)) if number.as_i64() == Some(0) || number.as_u64() == Some(0));

    if success && !exit_code_is_zero {
        return Err("CLI envelope success=true requires exitCode 0".to_string());
    }
    if !success && exit_code_is_zero {
        return Err("CLI envelope success=false requires a non-zero exitCode".to_string());
    }

    Ok(())
}

fn validate_diagnostic_array(value: Option<&Value>, field: &str) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("CLI envelope {field} must be an array"));
    };

    for (index, item) in items.iter().enumerate() {
        validate_diagnostic_value(item)
            .map_err(|err| format!("CLI envelope {field}[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

fn validate_timings_array(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };

    let Some(items) = value.as_array() else {
        return Err("CLI envelope timings must be an array".to_string());
    };

    for (index, item) in items.iter().enumerate() {
        validate_timing_value(item)
            .map_err(|err| format!("CLI envelope timings[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

fn validate_timing_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("timing must be a JSON object".to_string());
    };

    for key in ["phase", "milliseconds"] {
        if !object.contains_key(key) {
            return Err(format!("timing is missing required key `{key}`"));
        }
    }

    match object.get("phase") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("timing phase must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    match object.get("milliseconds") {
        Some(Value::Number(_)) => {}
        Some(other) => return Err(format!("timing milliseconds must be a number, got {other}")),
        None => unreachable!("validated above"),
    }

    Ok(())
}

fn validate_diagnostic_value(value: &Value) -> Result<(), String> {
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

    if let Some(context) = object.get("context") {
        validate_diagnostic_context(context)?;
    }

    Ok(())
}

fn is_positive_integer(value: &Value) -> bool {
    matches!(
        value,
        Value::Number(number)
            if number.as_u64().is_some_and(|value| value >= 1)
                || number.as_i64().is_some_and(|value| value >= 1)
    )
}

fn validate_source_span(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("span must be a JSON object".to_string());
    };

    for key in ["file", "line", "column", "endLine", "endColumn"] {
        if !object.contains_key(key) {
            return Err(format!("span is missing required key `{key}`"));
        }
    }

    match object.get("file") {
        Some(Value::String(_)) => {}
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

    Ok(())
}

fn validate_label_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("label must be a JSON object".to_string());
    };

    for key in ["span", "message", "style"] {
        if !object.contains_key(key) {
            return Err(format!("label is missing required key `{key}`"));
        }
    }

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

fn validate_diagnostic_label_array(value: Option<&Value>) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err("diagnostic labels must be an array".to_string());
    };

    for (index, item) in items.iter().enumerate() {
        validate_label_value(item)
            .map_err(|err| format!("diagnostic labels[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

fn validate_related_info_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("related item must be a JSON object".to_string());
    };

    for key in ["message", "span"] {
        if !object.contains_key(key) {
            return Err(format!("related item is missing required key `{key}`"));
        }
    }

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

fn validate_related_info_array(value: Option<&Value>) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err("diagnostic related must be an array".to_string());
    };

    for (index, item) in items.iter().enumerate() {
        validate_related_info_value(item)
            .map_err(|err| format!("diagnostic related[{index}] is invalid: {err}"))?;
    }

    Ok(())
}

fn validate_text_edit_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("text edit must be a JSON object".to_string());
    };

    for key in ["file", "start", "end", "newText"] {
        if !object.contains_key(key) {
            return Err(format!("text edit is missing required key `{key}`"));
        }
    }

    match object.get("file") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("text edit file must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    validate_source_location(
        object
            .get("start")
            .ok_or_else(|| "text edit is missing required key `start`".to_string())?,
    )?;
    validate_source_location(
        object
            .get("end")
            .ok_or_else(|| "text edit is missing required key `end`".to_string())?,
    )?;

    match object.get("newText") {
        Some(Value::String(_)) => {}
        Some(other) => return Err(format!("text edit newText must be a string, got {other}")),
        None => unreachable!("validated above"),
    }

    Ok(())
}

fn validate_source_location(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("source location must be a JSON object".to_string());
    };

    for key in ["file", "line", "column"] {
        if !object.contains_key(key) {
            return Err(format!("source location is missing required key `{key}`"));
        }
    }

    match object.get("file") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "source location file must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    for key in ["line", "column"] {
        match object.get(key) {
            Some(value) if is_positive_integer(value) => {}
            Some(other) => {
                return Err(format!(
                    "source location {key} must be a positive integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

fn validate_suggested_fix(value: Option<&Value>) -> Result<(), String> {
    match value {
        Some(Value::Null) | None => Ok(()),
        Some(Value::Object(object)) => {
            for key in ["message", "edits"] {
                if !object.contains_key(key) {
                    return Err(format!("suggested fix is missing required key `{key}`"));
                }
            }

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

fn validate_diagnostic_context(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("diagnostic context must be a JSON object".to_string());
    };

    match object.get("origin") {
        Some(Value::String(value))
            if matches!(value.as_str(), "cli" | "config" | "default" | "source") => {}
        Some(other) => {
            return Err(format!(
                "diagnostic context origin must be a canonical origin string, got {other}"
            ))
        }
        None => return Err("diagnostic context is missing required key `origin`".to_string()),
    }

    for key in ["configPath", "flag"] {
        if let Some(value) = object.get(key) {
            match value {
                Value::Null | Value::String(_) => {}
                other => {
                    return Err(format!(
                        "diagnostic context {key} must be string or null, got {other}"
                    ))
                }
            }
        }
    }

    // schema v1 allows requestedValue/effectiveValue to carry any JSON shape, so
    // validation is intentionally permissive here.
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_envelope(
    command: &str,
    success: bool,
    errors: &[Diagnostic],
    warnings: &[Diagnostic],
    payload: Value,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: i32,
    pretty: bool,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) {
    let errors = Value::Array(diagnostics_to_json(
        errors,
        source_path,
        source_text,
        "error",
    ));
    let warnings = Value::Array(diagnostics_to_json(
        warnings,
        source_path,
        source_text,
        "warning",
    ));
    let value = emit_envelope_value(
        command, success, errors, warnings, payload, stdout, stderr, exit_code,
    );
    if pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("serialize json envelope")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&value).expect("serialize json envelope")
        );
    }
}

pub fn diagnostic_to_text(diagnostic: &Diagnostic) -> String {
    diagnostic.to_string()
}

fn diagnostics_to_json(
    diagnostics: &[Diagnostic],
    source_path: Option<&Path>,
    source_text: Option<&str>,
    severity_fallback: &str,
) -> Vec<Value> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            diagnostic_to_json(diagnostic, source_path, source_text, severity_fallback)
        })
        .collect()
}

pub fn diagnostic_to_json(
    diagnostic: &Diagnostic,
    source_path: Option<&Path>,
    source_text: Option<&str>,
    severity_fallback: &str,
) -> Value {
    let severity = match diagnostic.severity {
        kali_error::Severity::Error => "error",
        kali_error::Severity::Warning => "warning",
        kali_error::Severity::Info => "info",
    };

    let code = diagnostic
        .code
        .map(|code| format!("{}{:04}", code_prefix(severity), code))
        .unwrap_or_else(|| format!("{}0000", code_prefix(severity_fallback)));

    let span = diagnostic
        .span
        .and_then(|span| source_path.and_then(|path| source_text.map(|text| (path, text, span))))
        .map(|(path, text, span)| span_to_json(path, text, span))
        .unwrap_or_else(|| synthetic_span(source_path));

    let mut diagnostic_json = Map::new();
    diagnostic_json.insert("severity".to_string(), json!(severity));
    diagnostic_json.insert("code".to_string(), json!(code));
    diagnostic_json.insert("message".to_string(), json!(diagnostic.message));
    diagnostic_json.insert("span".to_string(), span);
    diagnostic_json.insert("labels".to_string(), Value::Array(Vec::new()));
    diagnostic_json.insert("help".to_string(), json!(diagnostic.suggestion));
    diagnostic_json.insert("related".to_string(), Value::Array(Vec::new()));
    diagnostic_json.insert("fix".to_string(), Value::Null);
    diagnostic_json.insert("notes".to_string(), json!(diagnostic.notes));
    if let Some(context) = &diagnostic.context {
        diagnostic_json.insert(
            "context".to_string(),
            serde_json::to_value(context).expect("serialize diagnostic context"),
        );
    }

    Value::Object(diagnostic_json)
}

fn synthetic_span(source_path: Option<&Path>) -> Value {
    json!({
        "file": source_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<cli>".to_string()),
        "line": 1,
        "column": 1,
        "endLine": 1,
        "endColumn": 1,
    })
}

fn span_to_json(path: &Path, source: &str, span: kali_common::Span) -> Value {
    let (line, column) = byte_offset_to_line_col(source, span.start as usize);
    let (end_line, end_column) = byte_offset_to_line_col(source, span.end as usize);
    json!({
        "file": path.display().to_string(),
        "line": line,
        "column": column,
        "endLine": end_line,
        "endColumn": end_column,
    })
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    let mut consumed = 0usize;

    for ch in source.chars() {
        if consumed >= offset {
            break;
        }
        let len = ch.len_utf8();
        consumed += len;
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn code_prefix(severity: &str) -> char {
    match severity {
        "warning" => 'W',
        "info" => 'I',
        _ => 'E',
    }
}

pub fn json_source_path(path: impl AsRef<Path>) -> String {
    path.as_ref().display().to_string()
}

pub fn json_string_list(values: impl IntoIterator<Item = impl ToString>) -> Value {
    Value::Array(
        values
            .into_iter()
            .map(|value| Value::String(value.to_string()))
            .collect(),
    )
}
