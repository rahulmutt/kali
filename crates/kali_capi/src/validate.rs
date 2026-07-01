//! Shared JSON field-validators used by the `metadata` and `manifest` families.
//!
//! Internal only — `pub(crate)`, intentionally NOT glob-exported by the facade.

use serde_json::Value;

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

pub(crate) fn validate_string_field(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<(), String> {
    if value.is_string() {
        Ok(())
    } else {
        Err(format!(
            "{} field '{}' must be a string",
            context, field_name
        ))
    }
}

pub(crate) fn validate_non_empty_string_field(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<(), String> {
    match value {
        Value::String(value) if !value.trim().is_empty() && value.trim() == value => Ok(()),
        Value::String(_) => Err(format!(
            "{} field '{}' must be a non-empty, non-whitespace string",
            context, field_name
        )),
        _ => Err(format!(
            "{} field '{}' must be a string",
            context, field_name
        )),
    }
}

pub(crate) fn validate_integer_field(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<(), String> {
    if value.as_i64().is_some() || value.as_u64().is_some() {
        Ok(())
    } else {
        Err(format!(
            "{} field '{}' must be an integer",
            context, field_name
        ))
    }
}

pub(crate) fn validate_non_negative_integer_field(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<(), String> {
    if value.as_u64().is_some() {
        Ok(())
    } else {
        Err(format!(
            "{} field '{}' must be a non-negative integer",
            context, field_name
        ))
    }
}

pub(crate) fn integer_value(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<i128, String> {
    if let Some(number) = value.as_i64() {
        Ok(number as i128)
    } else if let Some(number) = value.as_u64() {
        Ok(number as i128)
    } else {
        Err(format!(
            "{} field '{}' must be an integer",
            context, field_name
        ))
    }
}

pub(crate) fn validate_host_abi_version_window(
    host_abi_version: &Value,
    min_host_abi_version: Option<&Value>,
    context: &str,
) -> Result<Value, String> {
    validate_integer_field(host_abi_version, context, "hostAbiVersion")?;
    let host_abi_version_value = host_abi_version.clone();
    let host_abi_version = integer_value(host_abi_version, context, "hostAbiVersion")?;

    match min_host_abi_version {
        Some(min_host_abi_version) => {
            validate_integer_field(min_host_abi_version, context, "minHostAbiVersion")?;
            let min_host_abi_version_value = min_host_abi_version.clone();
            let min_host_abi_version =
                integer_value(min_host_abi_version, context, "minHostAbiVersion")?;
            if min_host_abi_version > host_abi_version {
                return Err(format!(
                    "{} field 'minHostAbiVersion' must not exceed field 'hostAbiVersion'",
                    context
                ));
            }
            Ok(min_host_abi_version_value)
        }
        None => Ok(host_abi_version_value),
    }
}

pub(crate) fn normalize_string_list_value(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<Value, String> {
    let items = value.as_array().ok_or_else(|| {
        format!(
            "{} field '{}' must be an array of strings",
            context, field_name
        )
    })?;

    let mut normalized = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let string = item
            .as_str()
            .ok_or_else(|| format!("{} field '{}' entries must be strings", context, field_name))?;
        if string.trim().is_empty() {
            return Err(format!(
                "{} field '{}[{}]' must be a non-empty, non-whitespace string",
                context, field_name, index
            ));
        }
        if string.trim() != string {
            return Err(format!(
                "{} field '{}[{}]' must not have leading or trailing whitespace",
                context, field_name, index
            ));
        }
        normalized.push(string.to_string());
    }

    normalized.sort();
    normalized.dedup();

    Ok(Value::Array(
        normalized.into_iter().map(Value::String).collect(),
    ))
}
