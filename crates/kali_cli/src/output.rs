use kali_error::Diagnostic;
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};
use url::Url;

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

pub fn validate_envelope_value(value: &Value) -> Result<(), String> {
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
        Some(Value::Number(number))
            if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {}
        Some(other) => {
            return Err(format!(
                "CLI envelope exitCode must be a non-negative integer, got {other}"
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

pub fn validate_doctor_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("doctor payload must be a JSON object".to_string());
    };

    for key in ["browserHarness", "browserRuntimeContract"] {
        if !object.contains_key(key) {
            return Err(format!("doctor payload is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &["browserHarness", "browserRuntimeContract"],
        "doctor payload",
    )?;

    validate_browser_harness_value(object.get("browserHarness"))?;
    validate_browser_runtime_contract_value(object.get("browserRuntimeContract"))?;
    Ok(())
}

pub fn validate_effects_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("effects payload must be a JSON object".to_string());
    };

    for key in [
        "schemaVersion",
        "analysisContext",
        "entryPoints",
        "effects",
        "dynamicEffects",
        "dynamicReasons",
    ] {
        if !object.contains_key(key) {
            return Err(format!("effects payload is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "schemaVersion",
            "analysisContext",
            "entryPoints",
            "effects",
            "dynamicEffects",
            "dynamicReasons",
        ],
        "effects payload",
    )?;

    validate_schema_version_one(object.get("schemaVersion"), "effects payload")?;
    validate_analysis_context_value(
        object.get("analysisContext"),
        "effects payload analysisContext",
    )?;
    validate_unique_string_array_value(
        object.get("entryPoints"),
        "effects payload entryPoints",
        true,
    )?;
    validate_effect_occurrences_value(object.get("effects"), "effects payload effects")?;

    let dynamic_effects = match object.get("dynamicEffects") {
        Some(Value::Bool(value)) => *value,
        Some(other) => {
            return Err(format!(
                "effects payload dynamicEffects must be a boolean, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    };

    validate_sorted_string_array_value(
        object.get("dynamicReasons"),
        "effects payload dynamicReasons",
        true,
    )?;

    if !dynamic_effects
        && object
            .get("dynamicReasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| !reasons.is_empty())
    {
        return Err(
            "effects payload dynamicReasons must be empty when dynamicEffects is false".to_string(),
        );
    }

    Ok(())
}

pub fn validate_package_effects_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("package-effects payload must be a JSON object".to_string());
    };

    for key in ["schemaVersion", "package", "report"] {
        if !object.contains_key(key) {
            return Err(format!(
                "package-effects payload is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        object,
        &["schemaVersion", "package", "report"],
        "package-effects payload",
    )?;

    validate_schema_version_one(object.get("schemaVersion"), "package-effects payload")?;
    validate_package_coordinate_value(object.get("package"))?;
    let report = object
        .get("report")
        .expect("validated package-effects payload report key");
    validate_effects_payload_value(report)?;
    let Some(report_object) = report.as_object() else {
        return Err("package-effects payload report must be a JSON object".to_string());
    };
    let Some(entry_points) = report_object.get("entryPoints").and_then(Value::as_array) else {
        unreachable!("validated by validate_effects_payload_value")
    };
    if entry_points.len() != 1 {
        return Err(
            "package-effects payload report entryPoints must contain exactly one item".to_string(),
        );
    }
    Ok(())
}

pub fn validate_init_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("init payload must be a JSON object".to_string());
    };

    for key in ["root", "manifestPath", "sourcePath", "library"] {
        if !object.contains_key(key) {
            return Err(format!("init payload is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &["root", "manifestPath", "sourcePath", "library"],
        "init payload",
    )?;

    for key in ["root", "manifestPath", "sourcePath"] {
        validate_non_empty_string_value(object.get(key), &format!("init payload {key}"))?;
    }

    match object.get("library") {
        Some(Value::Bool(_)) => {}
        Some(other) => {
            return Err(format!(
                "init payload library must be a boolean, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}

pub fn validate_fmt_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("fmt payload must be a JSON object".to_string());
    };

    for key in ["filesFormatted", "filesChecked"] {
        if !object.contains_key(key) {
            return Err(format!("fmt payload is missing required key `{key}`"));
        }
    }

    for key in ["filesFormatted", "filesChecked"] {
        match object.get(key) {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!(
                    "fmt payload {key} must be a non-negative integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

pub fn validate_lint_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("lint payload must be a JSON object".to_string());
    };

    for key in ["filesLinted", "errorCount", "warningCount", "fixedCount"] {
        if !object.contains_key(key) {
            return Err(format!("lint payload is missing required key `{key}`"));
        }
    }

    for key in ["filesLinted", "errorCount", "warningCount", "fixedCount"] {
        match object.get(key) {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!(
                    "lint payload {key} must be a non-negative integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

pub fn validate_install_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("install payload must be a JSON object".to_string());
    };

    for key in ["installed", "updated", "removed"] {
        if !object.contains_key(key) {
            return Err(format!("install payload is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "installed",
            "updated",
            "removed",
            "manifestPath",
            "lockPath",
        ],
        "install payload",
    )?;

    for key in ["manifestPath", "lockPath"] {
        if let Some(other) = object.get(key) {
            match other {
                Value::Null | Value::String(_) => {}
                _ => {
                    return Err(format!(
                        "install payload {key} must be a string or null, got {other}"
                    ))
                }
            }
        }
    }

    for key in ["installed", "updated", "removed"] {
        validate_string_array_value(object.get(key), &format!("install payload {key}"), true)?;
    }

    Ok(())
}

pub fn validate_package_audit_payload_value(value: &Value) -> Result<(), String> {
    if value.is_null() {
        Ok(())
    } else {
        Err(format!("package-audit payload must be null, got {value}"))
    }
}

pub fn validate_check_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("check payload must be a JSON object".to_string());
    };

    for key in ["filesChecked", "errorCount", "warningCount"] {
        if !object.contains_key(key) {
            return Err(format!("check payload is missing required key `{key}`"));
        }
    }

    for key in ["filesChecked", "errorCount", "warningCount"] {
        match object.get(key) {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!(
                    "check payload {key} must be a non-negative integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

pub fn validate_run_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("run payload must be a JSON object".to_string());
    };

    for key in ["exitCode", "runtimeMs"] {
        if !object.contains_key(key) {
            return Err(format!("run payload is missing required key `{key}`"));
        }
    }

    match object.get("exitCode") {
        Some(Value::Number(number)) if number.as_i64().is_some() || number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "run payload exitCode must be an integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("runtimeMs") {
        Some(Value::Number(number))
            if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {}
        Some(other) => {
            return Err(format!(
                "run payload runtimeMs must be a non-negative integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    for key in ["hostContract", "runtimeBackend"] {
        validate_optional_non_empty_string_value(object.get(key), &format!("run payload {key}"))?;
    }

    validate_thread_topology_snapshot_value(object.get("threadTopology"))?;
    Ok(())
}

pub fn validate_test_payload_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("test payload must be a JSON object".to_string());
    };

    for key in ["total", "passed", "failed", "skipped", "runtimeMs"] {
        if !object.contains_key(key) {
            return Err(format!("test payload is missing required key `{key}`"));
        }
    }

    for key in ["total", "passed", "failed", "skipped", "runtimeMs"] {
        match object.get(key) {
            Some(Value::Number(number))
                if number.as_u64().is_some() || number.as_i64().is_some_and(|value| value >= 0) => {
            }
            Some(other) => {
                return Err(format!(
                    "test payload {key} must be a non-negative integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    for key in ["hostContract", "runtimeBackend"] {
        validate_optional_non_empty_string_value(object.get(key), &format!("test payload {key}"))?;
    }

    validate_thread_topology_snapshot_value(object.get("threadTopology"))?;
    validate_test_payload_coverage_value(object.get("coverage"))?;
    Ok(())
}

pub fn merge_thread_topology_snapshot_values(target: &mut Value, source: &Value) {
    let Some(target_object) = target.as_object_mut() else {
        return;
    };
    let Some(source_object) = source.as_object() else {
        return;
    };

    for key in ["totalInstances", "terminatedInstances"] {
        let merged_value = target_object
            .get(key)
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .saturating_add(source_object.get(key).and_then(Value::as_u64).unwrap_or(0));
        target_object.insert(key.to_string(), json!(merged_value));
    }

    let Some(target_live_instances) = target_object
        .get_mut("liveInstances")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let Some(source_live_instances) = source_object.get("liveInstances").and_then(Value::as_array)
    else {
        return;
    };

    let mut next_instance_id = target_live_instances
        .iter()
        .filter_map(|item| item.get("instanceId").and_then(Value::as_u64))
        .max()
        .map_or(0, |max| max.saturating_add(1));

    for item in source_live_instances {
        let mut item = item.clone();
        if let Some(object) = item.as_object_mut() {
            object.insert("instanceId".to_string(), json!(next_instance_id));
        }
        next_instance_id = next_instance_id.saturating_add(1);
        target_live_instances.push(item);
    }

    target_live_instances.sort_by_key(|item| {
        item.get("instanceId")
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX)
    });
}

fn validate_thread_topology_snapshot_value(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Err(format!("threadTopology must be a JSON object, got {value}"));
    };

    for key in ["totalInstances", "terminatedInstances", "liveInstances"] {
        if !object.contains_key(key) {
            return Err(format!("threadTopology is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &["totalInstances", "terminatedInstances", "liveInstances"],
        "threadTopology",
    )?;

    let total_instances = match object.get("totalInstances") {
        Some(value) => positive_integer_value(value).ok_or_else(|| {
            format!("threadTopology totalInstances must be a non-negative integer, got {value}")
        })?,
        None => unreachable!("validated above"),
    };
    let terminated_instances = match object.get("terminatedInstances") {
        Some(value) => positive_integer_value(value).ok_or_else(|| {
            format!(
                "threadTopology terminatedInstances must be a non-negative integer, got {value}"
            )
        })?,
        None => unreachable!("validated above"),
    };

    let Some(Value::Array(items)) = object.get("liveInstances") else {
        return Err(format!(
            "threadTopology liveInstances must be an array, got {}",
            object.get("liveInstances").unwrap()
        ));
    };

    let mut previous_instance_id = None;
    let mut seen_instance_ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let instance_id = validate_thread_topology_instance_snapshot_value(item)
            .map_err(|error| format!("threadTopology liveInstances[{index}] {error}"))?;
        if !seen_instance_ids.insert(instance_id) {
            return Err(format!(
                "threadTopology liveInstances[{index}] instanceId must be unique, got {instance_id}"
            ));
        }
        if previous_instance_id.is_some_and(|previous| instance_id < previous) {
            return Err(format!(
                "threadTopology liveInstances[{index}] instanceId must be ordered by ascending instanceId"
            ));
        }
        previous_instance_id = Some(instance_id);
    }

    let live_instances = items.len() as u64;
    if total_instances != terminated_instances + live_instances {
        return Err(format!(
            "threadTopology totalInstances must equal terminatedInstances + liveInstances.len(), got totalInstances={total_instances}, terminatedInstances={terminated_instances}, liveInstances={live_instances}"
        ));
    }

    Ok(())
}

fn validate_thread_topology_instance_snapshot_value(value: &Value) -> Result<u64, String> {
    let Some(object) = value.as_object() else {
        return Err(format!("must be a JSON object, got {value}"));
    };

    for key in [
        "instanceId",
        "scriptUrl",
        "postedMessages",
        "postedSharedBuffers",
        "wasTerminated",
    ] {
        if !object.contains_key(key) {
            return Err(format!("is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "instanceId",
            "scriptUrl",
            "postedMessages",
            "postedSharedBuffers",
            "wasTerminated",
        ],
        "threadTopology liveInstances item",
    )?;

    let instance_id = match object.get("instanceId") {
        Some(value) => positive_integer_value(value)
            .ok_or_else(|| format!("instanceId must be a non-negative integer, got {value}"))?,
        None => unreachable!("validated above"),
    };

    validate_canonical_absolute_url_string_value(object.get("scriptUrl"), "scriptUrl")?;

    match object.get("postedMessages") {
        Some(Value::Array(_)) => {}
        Some(other) => return Err(format!("postedMessages must be an array, got {other}")),
        None => unreachable!("validated above"),
    }

    let Some(Value::Array(shared_buffers)) = object.get("postedSharedBuffers") else {
        return Err(format!(
            "postedSharedBuffers must be an array, got {}",
            object.get("postedSharedBuffers").unwrap()
        ));
    };
    for (buffer_index, buffer) in shared_buffers.iter().enumerate() {
        let Some(Value::Array(bytes)) = Some(buffer) else {
            return Err(format!(
                "postedSharedBuffers[{buffer_index}] must be an array, got {buffer}"
            ));
        };
        for (byte_index, byte) in bytes.iter().enumerate() {
            match positive_integer_value(byte) {
                Some(value) if value <= 255 => {}
                Some(_) | None => {
                    return Err(format!(
                        "postedSharedBuffers[{buffer_index}][{byte_index}] must be a byte value, got {byte}"
                    ))
                }
            }
        }
    }

    match object.get("wasTerminated") {
        Some(Value::Bool(_)) => {}
        Some(other) => return Err(format!("wasTerminated must be a boolean, got {other}")),
        None => unreachable!("validated above"),
    }

    Ok(instance_id)
}

fn validate_test_payload_coverage_value(value: Option<&Value>) -> Result<(), String> {
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

fn reject_unexpected_keys(
    object: &serde_json::Map<String, Value>,
    allowed_keys: &[&str],
    context: &str,
) -> Result<(), String> {
    for key in object.keys() {
        if !allowed_keys.contains(&key.as_str()) {
            return Err(format!("{context} contains unexpected key `{key}`"));
        }
    }
    Ok(())
}

fn validate_schema_version_one(value: Option<&Value>, context: &str) -> Result<(), String> {
    match value {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => Ok(()),
        Some(other) => Err(format!(
            "{context} schemaVersion must be the numeric value 1, got {other}"
        )),
        None => Err(format!("{context} is missing required key `schemaVersion`")),
    }
}

fn validate_string_array_value(
    value: Option<&Value>,
    context: &str,
    allow_empty: bool,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    if !allow_empty && items.is_empty() {
        return Err(format!("{context} must contain at least one item"));
    }

    for (index, item) in items.iter().enumerate() {
        validate_non_empty_string_value(Some(item), &format!("{context}[{index}]"))?;
    }

    Ok(())
}

fn validate_unique_string_array_value(
    value: Option<&Value>,
    context: &str,
    allow_empty: bool,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    if !allow_empty && items.is_empty() {
        return Err(format!("{context} must contain at least one item"));
    }

    let mut seen = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let Some(item) = item.as_str() else {
            return Err(format!("{context}[{index}] must be a string, got {item}"));
        };
        if item.trim().is_empty() {
            return Err(format!(
                "{context}[{index}] must be a non-empty, non-whitespace string"
            ));
        }
        if item.trim() != item {
            return Err(format!(
                "{context}[{index}] must not have leading or trailing whitespace"
            ));
        }

        if !seen.insert(item) {
            return Err(format!(
                "{context} must not contain duplicate item `{item}`"
            ));
        }
    }

    Ok(())
}

fn validate_non_empty_string_value(value: Option<&Value>, context: &str) -> Result<(), String> {
    match value {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
        Some(Value::String(_)) => Err(format!(
            "{context} must be a non-empty, non-whitespace string"
        )),
        Some(other) => Err(format!("{context} must be a string, got {other}")),
        None => unreachable!("validated above"),
    }
}

fn validate_canonical_absolute_url_string_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::String(value)) = value else {
        return validate_non_empty_string_value(value, context);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "{context} must be a non-empty, non-whitespace string"
        ));
    }

    Url::parse(trimmed)
        .map_err(|_| format!("{context} must be a valid absolute URL, got {value}"))?;
    Ok(())
}

fn validate_optional_non_empty_string_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    if let Some(value) = value {
        validate_non_empty_string_value(Some(value), context)?;
    }
    Ok(())
}

pub(crate) fn validate_sorted_string_array_value(
    value: Option<&Value>,
    context: &str,
    allow_empty: bool,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    if !allow_empty && items.is_empty() {
        return Err(format!("{context} must contain at least one item"));
    }

    let mut previous: Option<&str> = None;
    for (index, item) in items.iter().enumerate() {
        let Some(item) = item.as_str() else {
            return Err(format!("{context}[{index}] must be a string, got {item}"));
        };
        if item.trim().is_empty() {
            return Err(format!(
                "{context}[{index}] must be a non-empty, non-whitespace string"
            ));
        }
        if item.trim() != item {
            return Err(format!(
                "{context}[{index}] must not have leading or trailing whitespace"
            ));
        }

        if let Some(previous) = previous {
            if previous >= item {
                return Err(format!(
                    "{context} must be deduplicated and sorted in lexical order, got `{previous}` before `{item}`"
                ));
            }
        }

        previous = Some(item);
    }

    Ok(())
}

fn validate_analysis_context_value(value: Option<&Value>, context: &str) -> Result<(), String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Err(format!("{context} must be a JSON object"));
    };

    for key in ["apiSurface", "runtimeProfiles", "compatFeatures"] {
        if !object.contains_key(key) {
            return Err(format!("{context} is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(
        object,
        &["apiSurface", "runtimeProfiles", "compatFeatures"],
        context,
    )?;

    validate_non_empty_string_value(object.get("apiSurface"), &format!("{context} apiSurface"))?;

    validate_sorted_string_array_value(
        object.get("runtimeProfiles"),
        &format!("{context} runtimeProfiles"),
        true,
    )?;
    validate_sorted_string_array_value(
        object.get("compatFeatures"),
        &format!("{context} compatFeatures"),
        true,
    )?;
    Ok(())
}

fn validate_effect_location_value(value: &Value, context: &str) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err(format!("{context} must be a JSON object"));
    };

    for key in ["file", "line", "column"] {
        if !object.contains_key(key) {
            return Err(format!("{context} is missing required key `{key}`"));
        }
    }
    reject_unexpected_keys(object, &["file", "line", "column", "function"], context)?;

    validate_non_empty_string_value(object.get("file"), &format!("{context} file"))?;

    for key in ["line", "column"] {
        match object.get(key) {
            Some(value) if is_positive_integer(value) => {}
            Some(other) => {
                return Err(format!(
                    "{context} {key} must be a positive integer, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    if let Some(other) = object.get("function") {
        match other {
            Value::String(value) if !value.trim().is_empty() => {}
            Value::String(_) => {
                return Err(format!(
                    "{context} function must be a non-empty, non-whitespace string"
                ));
            }
            _ => {
                return Err(format!("{context} function must be a string, got {other}"));
            }
        }
    }

    Ok(())
}

fn validate_effect_occurrences_value(value: Option<&Value>, context: &str) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!(
                "{context}[{index}] must be a JSON object, got {item}"
            ));
        };
        for key in ["kind", "locations"] {
            if !object.contains_key(key) {
                return Err(format!(
                    "{context}[{index}] is missing required key `{key}`"
                ));
            }
        }
        reject_unexpected_keys(
            object,
            &["kind", "locations"],
            &format!("{context}[{index}]"),
        )?;

        match object.get("kind") {
            Some(Value::String(value)) if !value.trim().is_empty() => {}
            Some(Value::String(_)) => {
                return Err(format!(
                    "{context}[{index}] kind must be a non-empty, non-whitespace string"
                ))
            }
            Some(other) => {
                return Err(format!(
                    "{context}[{index}] kind must be a string, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }

        let Some(Value::Array(locations)) = object.get("locations") else {
            return Err(format!("{context}[{index}] locations must be an array"));
        };
        for (location_index, location) in locations.iter().enumerate() {
            validate_effect_location_value(
                location,
                &format!("{context}[{index}].locations[{location_index}]"),
            )?;
        }
    }

    Ok(())
}

fn validate_package_coordinate_value(value: Option<&Value>) -> Result<(), String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Err("package-effects payload package must be a JSON object".to_string());
    };

    for key in ["name", "version", "registry"] {
        if !object.contains_key(key) {
            return Err(format!(
                "package-effects payload package is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        object,
        &["name", "version", "registry"],
        "package-effects payload package",
    )?;

    for key in ["name", "version", "registry"] {
        match object.get(key) {
            Some(Value::String(value)) if !value.trim().is_empty() => {}
            Some(Value::String(_)) => {
                return Err(format!(
                "package-effects payload package {key} must be a non-empty, non-whitespace string"
            ))
            }
            Some(other) => {
                return Err(format!(
                    "package-effects payload package {key} must be a string, got {other}"
                ))
            }
            None => unreachable!("validated above"),
        }
    }

    Ok(())
}

fn validate_browser_harness_value(value: Option<&Value>) -> Result<(), String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Err("doctor browserHarness must be a JSON object".to_string());
    };

    for key in [
        "envVar",
        "source",
        "override",
        "command",
        "executable",
        "args",
        "executableAvailable",
    ] {
        if !object.contains_key(key) {
            return Err(format!(
                "doctor browserHarness is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "envVar",
            "source",
            "override",
            "command",
            "executable",
            "args",
            "executableAvailable",
        ],
        "doctor browserHarness",
    )?;

    validate_non_empty_string_value(object.get("envVar"), "doctor browserHarness envVar")?;

    let source = match object.get("source") {
        Some(Value::String(value)) if !value.trim().is_empty() => value,
        Some(Value::String(_)) => {
            return Err(
                "doctor browserHarness source must be a non-empty, non-whitespace string"
                    .to_string(),
            )
        }
        Some(other) => {
            return Err(format!(
                "doctor browserHarness source must be `env` or `auto`, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    };

    match source.as_str() {
        "env" | "auto" => {}
        _ => {
            return Err(format!(
                "doctor browserHarness source must be `env` or `auto`, got `{source}`"
            ))
        }
    }

    let override_value = match object.get("override") {
        Some(Value::Null) | Some(Value::String(_)) => {
            object.get("override").expect("validated above")
        }
        Some(other) => {
            return Err(format!(
                "doctor browserHarness override must be string or null, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    };

    match (source.as_str(), override_value) {
        ("env", Value::String(_)) => {}
        ("auto", Value::Null) => {}
        ("env", other) => {
            return Err(format!(
                "doctor browserHarness override must be a string when source is `env`, got {other}"
            ))
        }
        ("auto", other) => {
            return Err(format!(
                "doctor browserHarness override must be null when source is `auto`, got {other}"
            ))
        }
        _ => unreachable!("validated above"),
    }

    if matches!(source.as_str(), "env") {
        validate_non_empty_string_value(object.get("override"), "doctor browserHarness override")?;
    }

    match object.get("command") {
        Some(Value::Array(items)) if !items.is_empty() => {
            for (index, item) in items.iter().enumerate() {
                validate_non_empty_string_value(
                    Some(item),
                    &format!("doctor browserHarness command[{index}]"),
                )?;
            }
        }
        Some(Value::Array(_)) => {
            return Err("doctor browserHarness command must contain at least one item".to_string())
        }
        Some(other) => {
            return Err(format!(
                "doctor browserHarness command must be an array, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    validate_non_empty_string_value(object.get("executable"), "doctor browserHarness executable")?;

    match object.get("args") {
        Some(Value::Array(items)) => {
            for (index, item) in items.iter().enumerate() {
                validate_non_empty_string_value(
                    Some(item),
                    &format!("doctor browserHarness args[{index}]"),
                )?;
            }
        }
        Some(other) => {
            return Err(format!(
                "doctor browserHarness args must be an array, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    let command_items = object
        .get("command")
        .and_then(Value::as_array)
        .expect("validated above");
    let command_items = command_items
        .iter()
        .map(|item| item.as_str().expect("validated above"))
        .collect::<Vec<_>>();
    let executable = object
        .get("executable")
        .and_then(Value::as_str)
        .expect("validated above");
    let args_items = object
        .get("args")
        .and_then(Value::as_array)
        .expect("validated above");
    let args_items = args_items
        .iter()
        .map(|item| item.as_str().expect("validated above"))
        .collect::<Vec<_>>();
    if command_items.first().copied() != Some(executable) {
        return Err("doctor browserHarness executable must match command[0]".to_string());
    }
    if args_items.as_slice() != &command_items[1..] {
        return Err("doctor browserHarness args must match command[1..]".to_string());
    }

    match object.get("executableAvailable") {
        Some(Value::Bool(_)) => {}
        Some(other) => {
            return Err(format!(
                "doctor browserHarness executableAvailable must be a boolean, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}

fn trimmed_string_matches(value: Option<&Value>, expected: &str) -> bool {
    matches!(value, Some(Value::String(value)) if value.trim() == expected)
}

fn validate_trimmed_string_field(
    object: &Map<String, Value>,
    key: &str,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    if !trimmed_string_matches(object.get(key), expected) {
        return Err(format!("{context} {key} must be `{expected}`"));
    }

    Ok(())
}

fn validate_browser_runtime_supported_commands_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    let expected_commands = kali_runtime::BrowserRuntimeContract::supported_commands();

    if items.is_empty() {
        return Err(format!("{context} must contain at least one item"));
    }
    if items.len() != expected_commands.len() {
        return Err(format!(
            "{context} must be exactly {} in that order",
            browser_runtime_supported_commands_message()
        ));
    }

    let mut seen = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let Some(item) = item.as_str() else {
            return Err(format!("{context}[{index}] must be a string, got {item}"));
        };
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "{context}[{index}] must be a non-empty, non-whitespace string"
            ));
        }
        if !seen.insert(trimmed) {
            return Err(format!(
                "{context} must not contain duplicate item `{trimmed}`"
            ));
        }
        match trimmed {
            "run" if index == 0 => {}
            "test" if index == 1 => {}
            "run" | "test" => {
                return Err(format!(
                    "{context} must be exactly [`run`, `test`] in that order"
                ));
            }
            other => {
                return Err(format!(
                    "{context}[{index}] must be `run` or `test`, got {other}"
                ));
            }
        }
    }

    Ok(())
}

fn browser_runtime_supported_commands_message() -> String {
    let commands = kali_runtime::BrowserRuntimeContract::supported_commands();
    format!("[`{}`, `{}`]", commands[0], commands[1])
}

fn browser_runtime_contract_notes_message() -> String {
    let notes = kali_runtime::BrowserRuntimeContract::diagnostic_notes();
    format!(
        "[{}]",
        notes
            .iter()
            .map(|note| format!("`{note}`"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn validate_browser_runtime_diagnostic_notes_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    if items.is_empty() {
        return Err(format!("{context} must contain at least one item"));
    }

    let mut seen = HashSet::new();
    let expected_notes = kali_runtime::BrowserRuntimeContract::diagnostic_notes();

    for (index, (item, expected_item)) in items.iter().zip(expected_notes.iter()).enumerate() {
        let Some(item) = item.as_str() else {
            return Err(format!("{context}[{index}] must be a string, got {item}"));
        };
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "{context}[{index}] must be a non-empty, non-whitespace string"
            ));
        }
        if !seen.insert(trimmed) {
            return Err(format!(
                "{context} must not contain duplicate item `{trimmed}`"
            ));
        }
        if trimmed != *expected_item {
            return Err(format!(
                "{context} must be exactly {} in that order",
                browser_runtime_contract_notes_message()
            ));
        }
    }

    if items.len() != expected_notes.len() {
        return Err(format!(
            "{context} must be exactly {} in that order",
            browser_runtime_contract_notes_message()
        ));
    }

    Ok(())
}

fn validate_browser_runtime_contract_value(value: Option<&Value>) -> Result<(), String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Err("doctor browserRuntimeContract must be a JSON object".to_string());
    };

    for key in [
        "hostLabel",
        "hostDescription",
        "hostDescriptionNote",
        "supportedCommands",
        "diagnosticHint",
        "diagnosticNotes",
    ] {
        if !object.contains_key(key) {
            return Err(format!(
                "doctor browserRuntimeContract is missing required key `{key}`"
            ));
        }
    }
    reject_unexpected_keys(
        object,
        &[
            "hostLabel",
            "hostDescription",
            "hostDescriptionNote",
            "supportedCommands",
            "diagnosticHint",
            "diagnosticNotes",
        ],
        "doctor browserRuntimeContract",
    )?;

    let browser_runtime_contract = kali_runtime::BrowserRuntimeContract::descriptor();

    validate_trimmed_string_field(
        object,
        "hostLabel",
        browser_runtime_contract.host_label,
        "doctor browserRuntimeContract",
    )?;

    validate_trimmed_string_field(
        object,
        "hostDescription",
        browser_runtime_contract.host_description,
        "doctor browserRuntimeContract",
    )?;

    validate_trimmed_string_field(
        object,
        "diagnosticHint",
        browser_runtime_contract.diagnostic_hint,
        "doctor browserRuntimeContract",
    )?;

    validate_trimmed_string_field(
        object,
        "hostDescriptionNote",
        browser_runtime_contract.host_description_note,
        "doctor browserRuntimeContract",
    )?;

    validate_browser_runtime_supported_commands_value(
        object.get("supportedCommands"),
        "doctor browserRuntimeContract supportedCommands",
    )?;

    validate_browser_runtime_diagnostic_notes_value(
        object.get("diagnosticNotes"),
        "doctor browserRuntimeContract diagnosticNotes",
    )?;

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

fn validate_cli_artifacts_array(value: Option<&Value>) -> Result<(), String> {
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
            Some(Value::String(_)) => {}
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
            Some(Value::String(_)) => {}
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

fn is_canonical_artifact_role(role: &str) -> bool {
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

fn artifact_sort_key(object: &serde_json::Map<String, Value>) -> (usize, String, String) {
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

fn artifact_role_rank(role: &str) -> usize {
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

fn timing_sort_key(phase: &str) -> (usize, String) {
    (timing_phase_rank(phase), phase.to_string())
}

fn timing_phase_rank(phase: &str) -> usize {
    match phase {
        "parse" => 0,
        "typecheck" => 1,
        _ => usize::MAX,
    }
}

fn validate_timings_array(value: Option<&Value>) -> Result<(), String> {
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

fn validate_timing_value(value: &Value) -> Result<String, String> {
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
        Some(Value::String(value)) if !value.trim().is_empty() => value.clone(),
        Some(Value::String(_)) => return Err("timing phase must be a non-empty string".to_string()),
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

fn is_positive_integer(value: &Value) -> bool {
    matches!(
        value,
        Value::Number(number)
            if number.as_u64().is_some_and(|value| value >= 1)
                || number.as_i64().is_some_and(|value| value >= 1)
    )
}

fn positive_integer_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64().or_else(|| {
            number
                .as_i64()
                .and_then(|value| (value >= 0).then_some(value as u64))
        }),
        _ => None,
    }
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

fn validate_label_value(value: &Value) -> Result<(), String> {
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

fn validate_source_location_file_mirror(
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

fn validate_source_location(value: &Value, context: &str) -> Result<(), String> {
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

fn validate_text_edit_location_order(start: &Value, end: &Value) -> Result<(), String> {
    let start = source_location_position(start, "text edit start")?;
    let end = source_location_position(end, "text edit end")?;

    if end < start {
        Err("text edit end position must not precede its start position".to_string())
    } else {
        Ok(())
    }
}

fn validate_suggested_fix_edits_non_overlapping(edits: &[Value]) -> Result<(), String> {
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

fn source_location_position(value: &Value, location_name: &str) -> Result<(u64, u64), String> {
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

fn validate_suggested_fix(value: Option<&Value>) -> Result<(), String> {
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

fn validate_diagnostic_context(value: &Value) -> Result<(), String> {
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
    let file = span
        .get("file")
        .and_then(Value::as_str)
        .map(|file| json!(file));

    let mut diagnostic_json = Map::new();
    diagnostic_json.insert("severity".to_string(), json!(severity));
    diagnostic_json.insert("code".to_string(), json!(code));
    diagnostic_json.insert("message".to_string(), json!(diagnostic.message));
    if let Some(file) = file {
        diagnostic_json.insert("file".to_string(), file);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn browser_runtime_contract_fixture() -> Value {
        kali_runtime::browser_runtime_contract_value()
    }

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_supported_commands_items() {
        let mut contract = browser_runtime_contract_fixture();
        contract["supportedCommands"] = json!(["run", "  "]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("whitespace-only supportedCommands item should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract supportedCommands[1] must be a non-empty, non-whitespace string"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_diagnostic_notes_items() {
        let mut contract = browser_runtime_contract_fixture();
        contract["diagnosticNotes"] = json!([
            "supported browser runtime commands: run, test",
            " ",
            "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
            "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
            "browser runtime host description: real browser host"
        ]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("whitespace-only diagnosticNotes item should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract diagnosticNotes[1] must be a non-empty, non-whitespace string"
        );
    }

    #[test]
    fn browser_runtime_contract_accepts_trimmed_canonical_labels() {
        let mut contract = browser_runtime_contract_fixture();
        contract["hostLabel"] = json!(" browser-requested ");
        contract["hostDescription"] = json!(" real browser host ");
        contract["hostDescriptionNote"] =
            json!(" browser runtime host description: real browser host ");

        validate_browser_runtime_contract_value(Some(&contract))
            .expect("trimmed canonical labels should still validate");
    }

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_canonical_labels() {
        for (field, expected) in [
            (
                "hostLabel",
                "doctor browserRuntimeContract hostLabel must be `browser-requested`",
            ),
            (
                "hostDescription",
                "doctor browserRuntimeContract hostDescription must be `real browser host`",
            ),
            (
                "hostDescriptionNote",
                "doctor browserRuntimeContract hostDescriptionNote must be `browser runtime host description: real browser host`",
            ),
        ] {
            let mut contract = browser_runtime_contract_fixture();
            contract[field] = json!("   ");

            let err = validate_browser_runtime_contract_value(Some(&contract))
                .expect_err("whitespace-only canonical label should be rejected");

            assert_eq!(err, expected);
        }
    }
}
