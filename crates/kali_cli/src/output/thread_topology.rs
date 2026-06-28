//! Thread-topology snapshot validators (merge + validate).

use serde_json::{json, Value};
use std::collections::BTreeSet;

use super::diagnostic::*;
use super::schema::*;

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

pub(crate) fn validate_thread_topology_snapshot_value(value: Option<&Value>) -> Result<(), String> {
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

pub(crate) fn validate_thread_topology_instance_snapshot_value(value: &Value) -> Result<u64, String> {
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
