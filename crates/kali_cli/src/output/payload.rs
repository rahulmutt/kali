//! Per-command payload validators (public surface).

use serde_json::Value;

use super::browser_runtime::{validate_browser_harness_value, validate_browser_runtime_contract_value};
use super::coverage::validate_test_payload_coverage_value;
use super::schema::*;
use super::thread_topology::validate_thread_topology_snapshot_value;

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

    let expected_entry_point = match (
        object
            .get("package")
            .and_then(Value::as_object)
            .and_then(|package| package.get("registry"))
            .and_then(Value::as_str),
        object
            .get("package")
            .and_then(Value::as_object)
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str),
    ) {
        (Some("jsr"), Some(name)) => format!("jsr:{name}"),
        (Some(_), Some(name)) => name.to_string(),
        _ => unreachable!("validated package coordinate should contain registry and name"),
    };

    let actual_entry_point = entry_points[0]
        .as_str()
        .expect("validated entryPoints item should be a string");
    if actual_entry_point.trim().is_empty() {
        return Err(
            "package-effects payload report entryPoints[0] must be a non-empty, non-whitespace string"
                .to_string(),
        );
    }
    if actual_entry_point.trim() != actual_entry_point {
        return Err(
            "package-effects payload report entryPoints[0] must not have leading or trailing whitespace"
                .to_string(),
        );
    }
    if actual_entry_point != expected_entry_point {
        return Err(format!(
            "package-effects payload report entryPoints[0] must match the canonical registry package identifier `{expected_entry_point}`, got `{actual_entry_point}`"
        ));
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
        validate_canonical_non_empty_string_value(object.get(key), &format!("init payload {key}"))?;
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
                Value::Null => {}
                Value::String(_) => {
                    validate_canonical_non_empty_string_value(
                        Some(other),
                        &format!("install payload {key}"),
                    )?;
                }
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
