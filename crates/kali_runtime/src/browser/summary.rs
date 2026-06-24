//! Browser runtime summary parsing and outcome helpers.
use crate::*;

#[derive(Default)]
pub(crate) struct BrowserRuntimeSummary {
    pub(crate) args: Vec<String>,
    pub(crate) tests: Vec<String>,
    pub(crate) tests_failed: Option<usize>,
    pub(crate) host_contract: Option<RuntimeHostContract>,
    pub(crate) runtime_backend: Option<RuntimeBackend>,
    pub(crate) thread_topology: Option<ThreadRuntimeShutdownReport>,
}

pub(crate) fn parse_non_blank_string_array_field(
    value: Option<&serde_json::Value>,
) -> Option<Vec<String>> {
    let items = value?.as_array()?;
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        let item = item.as_str()?;
        if item.trim().is_empty() || item.trim() != item {
            return None;
        }
        strings.push(item.to_owned());
    }
    Some(strings)
}

pub(crate) fn parse_browser_runtime_summary(stdout: &str) -> BrowserRuntimeSummary {
    parse_browser_runtime_summary_opt(stdout).unwrap_or_default()
}

pub(crate) fn parse_thread_runtime_instance_snapshot_value(
    value: &serde_json::Value,
) -> Option<ThreadRuntimeInstanceSnapshot> {
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "instanceId" | "scriptUrl" | "postedMessages" | "postedSharedBuffers" | "wasTerminated"
        )
    }) {
        return None;
    }

    let instance_id = object.get("instanceId")?.as_u64()? as usize;
    let script_url = object.get("scriptUrl")?.as_str()?;
    let trimmed_script_url = script_url.trim();
    if trimmed_script_url.is_empty() || trimmed_script_url != script_url {
        return None;
    }
    let parsed_script_url = url::Url::parse(trimmed_script_url).ok()?;
    if parsed_script_url.as_str() != script_url {
        return None;
    }

    let posted_messages = object.get("postedMessages")?.as_array()?.clone();
    let posted_shared_buffers = object
        .get("postedSharedBuffers")?
        .as_array()?
        .iter()
        .map(|buffer| {
            let bytes = buffer.as_array()?;
            let mut output = Vec::with_capacity(bytes.len());
            for byte in bytes {
                let byte = byte.as_u64()?;
                if byte > u8::MAX as u64 {
                    return None;
                }
                output.push(byte as u8);
            }
            Some(output)
        })
        .collect::<Option<Vec<Vec<u8>>>>()?;
    let was_terminated = object.get("wasTerminated")?.as_bool()?;

    Some(ThreadRuntimeInstanceSnapshot {
        instance_id,
        script_url: script_url.to_owned(),
        posted_messages,
        posted_shared_buffers,
        was_terminated,
    })
}

pub(crate) fn parse_thread_runtime_shutdown_report_value(
    value: Option<&serde_json::Value>,
) -> Option<ThreadRuntimeShutdownReport> {
    let value = value?;
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "totalInstances" | "terminatedInstances" | "liveInstances"
        )
    }) {
        return None;
    }

    let total_instances = object.get("totalInstances")?.as_u64()? as usize;
    let terminated_instances = object.get("terminatedInstances")?.as_u64()? as usize;
    let live_instances = object
        .get("liveInstances")?
        .as_array()?
        .iter()
        .map(parse_thread_runtime_instance_snapshot_value)
        .collect::<Option<Vec<_>>>()?;

    let mut previous_instance_id = None;
    let mut seen_instance_ids = BTreeSet::new();
    for instance in &live_instances {
        if !seen_instance_ids.insert(instance.instance_id) {
            return None;
        }
        if previous_instance_id.is_some_and(|previous| instance.instance_id < previous) {
            return None;
        }
        previous_instance_id = Some(instance.instance_id);
    }

    if total_instances != terminated_instances + live_instances.len() {
        return None;
    }

    Some(ThreadRuntimeShutdownReport {
        total_instances,
        terminated_instances,
        live_instances,
    })
}

pub(crate) fn parse_browser_runtime_summary_value(
    value: &serde_json::Value,
) -> Option<BrowserRuntimeSummary> {
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "args" | "tests" | "testsFailed" | "hostContract" | "runtimeBackend" | "threadTopology"
        )
    }) {
        return None;
    }

    let args = parse_non_blank_string_array_field(object.get("args"))?;
    let tests = parse_non_blank_string_array_field(object.get("tests"))?;
    let tests_failed = match object.get("testsFailed") {
        Some(value) => Some(value.as_u64()? as usize),
        None => None,
    };

    Some(BrowserRuntimeSummary {
        args,
        tests,
        tests_failed,
        host_contract: parse_optional_runtime_host_contract_label(object.get("hostContract")),
        runtime_backend: parse_optional_runtime_backend_label(object.get("runtimeBackend")),
        thread_topology: parse_thread_runtime_shutdown_report_value(object.get("threadTopology")),
    })
}

pub(crate) fn parse_browser_runtime_summary_opt(stdout: &str) -> Option<BrowserRuntimeSummary> {
    stdout.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let value = serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
        parse_browser_runtime_summary_value(&value)
    })
}

pub(crate) fn browser_runtime_summary_for_outcome(
    summary_path: &Path,
    outcome: &BrowserHarnessOutcome,
) -> BrowserRuntimeSummary {
    let stdout_summary = parse_browser_runtime_summary(&outcome.stdout);
    match fs::read_to_string(summary_path) {
        Ok(text) => {
            if text.trim().is_empty() {
                return stdout_summary;
            }

            match parse_browser_runtime_summary_opt(&text) {
                Some(mut summary) => {
                    if summary.tests_failed.is_none() {
                        summary.tests_failed = stdout_summary.tests_failed;
                    }
                    if summary.host_contract.is_none() {
                        summary.host_contract = stdout_summary.host_contract;
                    }
                    if summary.runtime_backend.is_none() {
                        summary.runtime_backend = stdout_summary.runtime_backend;
                    }
                    if summary.thread_topology.is_none() {
                        summary.thread_topology = stdout_summary.thread_topology;
                    }
                    summary
                }
                None => stdout_summary,
            }
        }
        Err(_) => stdout_summary,
    }
}
