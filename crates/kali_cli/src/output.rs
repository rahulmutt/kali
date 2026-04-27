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

    if object.len() != REQUIRED_KEYS.len() {
        return Err(format!(
            "CLI envelope must contain exactly {} top-level keys",
            REQUIRED_KEYS.len()
        ));
    }

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

    match object.get("errors") {
        Some(Value::Array(_)) => {}
        Some(other) => return Err(format!("CLI envelope errors must be an array, got {other}")),
        None => unreachable!("validated above"),
    }

    match object.get("warnings") {
        Some(Value::Array(_)) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope warnings must be an array, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

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

    match object.get("exitCode") {
        Some(Value::Number(number)) if number.as_i64().is_some() || number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope exitCode must be an integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

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
