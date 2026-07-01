//! Cross-cutting envelope/payload schema validators (crate-internal).

use semver::Version;
use serde_json::Value;
use std::collections::HashSet;
use url::Url;

use super::diagnostic::is_positive_integer;

pub(crate) fn reject_unexpected_keys(
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

pub(crate) fn validate_schema_version_one(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    match value {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => Ok(()),
        Some(other) => Err(format!(
            "{context} schemaVersion must be the numeric value 1, got {other}"
        )),
        None => Err(format!("{context} is missing required key `schemaVersion`")),
    }
}

pub(crate) fn validate_string_array_value(
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

pub(crate) fn validate_unique_string_array_value(
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

pub(crate) fn validate_non_empty_string_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    match value {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
        Some(Value::String(_)) => Err(format!(
            "{context} must be a non-empty, non-whitespace string"
        )),
        Some(other) => Err(format!("{context} must be a string, got {other}")),
        None => unreachable!("validated above"),
    }
}

pub(crate) fn validate_canonical_non_empty_string_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::String(value)) = value else {
        return validate_non_empty_string_value(value, context);
    };

    if value.trim().is_empty() {
        return Err(format!(
            "{context} must be a non-empty, non-whitespace string"
        ));
    }
    if value.trim() != value {
        return Err(format!(
            "{context} must not have leading or trailing whitespace"
        ));
    }

    Ok(())
}

pub(crate) fn validate_registry_package_name_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    validate_canonical_non_empty_string_value(value, context)?;

    let Some(Value::String(value)) = value else {
        unreachable!("validated above")
    };

    if value.chars().any(char::is_whitespace) {
        return Err(format!(
            "{context} must be a registry-native package name without whitespace"
        ));
    }

    Ok(())
}

pub(crate) fn validate_canonical_absolute_url_string_value(
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
    if trimmed != value {
        return Err(format!(
            "{context} must not have leading or trailing whitespace"
        ));
    }

    let parsed = Url::parse(trimmed)
        .map_err(|_| format!("{context} must be a valid absolute URL, got {value}"))?;
    if parsed.as_str() != trimmed {
        return Err(format!(
            "{context} must be a canonical absolute URL, got {value}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_optional_non_empty_string_value(
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

pub(crate) fn validate_analysis_context_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
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

    validate_canonical_non_empty_string_value(
        object.get("apiSurface"),
        &format!("{context} apiSurface"),
    )?;

    match object.get("apiSurface").and_then(Value::as_str) {
        Some("default") | Some("deno") | Some("node") | Some("browser") => {}
        Some(other) => {
            return Err(format!(
            "{context} apiSurface must be `default`, `deno`, `node`, or `browser`, got `{other}`"
        ))
        }
        None => unreachable!("validated above"),
    }

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

pub(crate) fn validate_effect_location_value(value: &Value, context: &str) -> Result<(), String> {
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

pub(crate) fn validate_effect_occurrences_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
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

pub(crate) fn validate_package_coordinate_value(value: Option<&Value>) -> Result<(), String> {
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

    validate_registry_package_name_value(
        object.get("name"),
        "package-effects payload package name",
    )?;
    for key in ["version", "registry"] {
        validate_canonical_non_empty_string_value(
            object.get(key),
            &format!("package-effects payload package {key}"),
        )?;
    }

    validate_stable_semver_version_value(
        object.get("version"),
        "package-effects payload package version",
    )?;

    match object.get("name").and_then(Value::as_str) {
        Some(name) if !name.contains(':') => {}
        Some(other) => {
            return Err(format!(
                "package-effects payload package name must be registry-native and must not include a registry prefix, got `{other}`"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("registry").and_then(Value::as_str) {
        Some("npm") | Some("jsr") => {}
        Some(other) => {
            return Err(format!(
                "package-effects payload package registry must be `npm` or `jsr`, got `{other}`"
            ))
        }
        None => unreachable!("validated above"),
    }

    Ok(())
}

pub(crate) fn validate_stable_semver_version_value(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::String(version)) = value else {
        return Err(format!(
            "{context} must be a string, got {}",
            value.unwrap_or(&Value::Null)
        ));
    };

    let parsed = Version::parse(version).map_err(|_| {
        format!("{context} must be a stable SemVer release string, got `{version}`")
    })?;
    if !parsed.pre.is_empty() {
        return Err(format!(
            "{context} must be a stable SemVer release string, got `{version}`"
        ));
    }

    Ok(())
}
