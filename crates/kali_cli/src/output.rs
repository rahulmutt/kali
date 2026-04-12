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
    Value::Object(envelope)
}

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

    json!({
        "severity": severity,
        "code": code,
        "message": diagnostic.message,
        "span": span,
        "labels": [],
        "help": diagnostic.suggestion,
        "related": [],
        "fix": null,
        "notes": diagnostic.notes,
    })
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
