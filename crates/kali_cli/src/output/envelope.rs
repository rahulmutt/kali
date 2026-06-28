//! CLI envelope construction + shape validation (public surface).

use kali_error::Diagnostic;
use serde_json::{json, Map, Value};
use std::path::Path;

use super::artifacts_timings::*;
use super::schema::*;
use super::serialize::diagnostics_to_json;

#[allow(clippy::too_many_arguments)]
pub fn emit_envelope_value(
    command: &str,
    success: bool,
    mut errors: Value,
    mut warnings: Value,
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

    sort_diagnostic_array_value(&mut errors);
    sort_diagnostic_array_value(&mut warnings);

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

pub fn validate_envelope_value(value: &Value) -> Result<(), String> {
    const REQUIRED_KEYS: [&str; 6] = [
        "schemaVersion",
        "command",
        "success",
        "errors",
        "warnings",
        "payload",
    ];
    const ALL_KEYS: [&str; 11] = [
        "schemaVersion",
        "command",
        "success",
        "errors",
        "warnings",
        "payload",
        "stdout",
        "stderr",
        "exitCode",
        "artifacts",
        "timings",
    ];

    let Some(object) = value.as_object() else {
        return Err("CLI envelope must be a JSON object".to_string());
    };

    for key in REQUIRED_KEYS {
        if !object.contains_key(key) {
            return Err(format!("CLI envelope is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &ALL_KEYS, "CLI envelope")?;

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
        Some(Value::String(value)) if !value.trim().is_empty() => {}
        Some(Value::String(_)) => {
            return Err(
                "CLI envelope command must be a non-empty, non-whitespace string".to_string(),
            )
        }
        Some(other) => {
            return Err(format!(
                "CLI envelope command must be a non-empty, non-whitespace string, got {other}"
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
    validate_cli_artifacts_array(object.get("artifacts"))?;

    if let Some(stdout) = object.get("stdout") {
        match stdout {
            Value::Null | Value::String(_) => {}
            other => {
                return Err(format!(
                    "CLI envelope stdout must be string or null, got {other}"
                ))
            }
        }
    }

    if let Some(stderr) = object.get("stderr") {
        match stderr {
            Value::Null | Value::String(_) => {}
            other => {
                return Err(format!(
                    "CLI envelope stderr must be string or null, got {other}"
                ))
            }
        }
    }

    validate_timings_array(object.get("timings"))?;

    let exit_code_is_zero = match object.get("exitCode") {
        Some(Value::Number(number))
            if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) =>
        {
            number.as_i64() == Some(0) || number.as_u64() == Some(0)
        }
        Some(other) => {
            return Err(format!(
                "CLI envelope exitCode must be a non-negative integer, got {other}"
            ))
        }
        None => false,
    };

    let success = matches!(object.get("success"), Some(Value::Bool(true)));
    if success && matches!(object.get("exitCode"), Some(Value::Number(_))) && !exit_code_is_zero {
        return Err("CLI envelope success=true requires exitCode 0".to_string());
    }
    if !success && matches!(object.get("exitCode"), Some(Value::Number(_))) && exit_code_is_zero {
        return Err("CLI envelope success=false requires a non-zero exitCode".to_string());
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
