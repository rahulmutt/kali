//! Browser harness / browser-runtime-contract validators.

use serde_json::{Map, Value};
use std::collections::HashSet;

use super::schema::*;

#[cfg(test)]
use serde_json::json;

pub(crate) fn validate_browser_harness_value(value: Option<&Value>) -> Result<(), String> {
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
    if !trimmed_string_matches(
        object.get("envVar"),
        kali_runtime::BROWSER_HARNESS_COMMAND_ENV,
    ) {
        return Err(format!(
            "doctor browserHarness envVar must be `{}`",
            kali_runtime::BROWSER_HARNESS_COMMAND_ENV
        ));
    }

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

pub(crate) fn trimmed_string_matches(value: Option<&Value>, expected: &str) -> bool {
    matches!(value, Some(Value::String(value)) if value.trim() == expected)
}

pub(crate) fn validate_trimmed_string_field(
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

pub(crate) fn validate_browser_runtime_supported_commands_value(
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

pub(crate) fn browser_runtime_supported_commands_message() -> String {
    let commands = kali_runtime::BrowserRuntimeContract::supported_commands();
    format!("[`{}`, `{}`]", commands[0], commands[1])
}

pub(crate) fn browser_runtime_contract_notes_message() -> String {
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

pub(crate) fn validate_browser_runtime_diagnostic_notes_value(
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

pub(crate) fn validate_browser_runtime_contract_value(value: Option<&Value>) -> Result<(), String> {
    let Some(object) = value.and_then(Value::as_object) else {
        return Err("doctor browserRuntimeContract must be a JSON object".to_string());
    };

    for key in [
        "hostLabel",
        "hostDescription",
        "hostDescriptionNote",
        "supportedCommands",
        "diagnosticHint",
        "summaryNote",
        "contractScopeNote",
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
            "summaryNote",
            "contractScopeNote",
            "diagnosticNotes",
        ],
        "doctor browserRuntimeContract",
    )?;

    let browser_runtime_contract = kali_runtime::BrowserRuntimeContract::descriptor();

    for (key, expected) in [
        ("hostLabel", browser_runtime_contract.host_label),
        ("hostDescription", browser_runtime_contract.host_description),
        (
            "hostDescriptionNote",
            browser_runtime_contract.host_description_note,
        ),
        ("diagnosticHint", browser_runtime_contract.diagnostic_hint),
        ("summaryNote", browser_runtime_contract.summary_note),
        (
            "contractScopeNote",
            browser_runtime_contract.contract_scope_note,
        ),
    ] {
        validate_trimmed_string_field(object, key, expected, "doctor browserRuntimeContract")?;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn browser_runtime_contract_fixture() -> Value {
        kali_runtime::browser_runtime_contract_value()
    }

    #[test]
    fn browser_runtime_contract_rejects_empty_supported_commands() {
        let mut contract = browser_runtime_contract_fixture();
        contract["supportedCommands"] = json!([]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("empty supportedCommands should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract supportedCommands must contain at least one item"
        );
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
    fn browser_runtime_contract_rejects_duplicate_supported_commands_items_after_trim() {
        let mut contract = browser_runtime_contract_fixture();
        contract["supportedCommands"] = json!([" run ", "run"]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("duplicate supportedCommands item should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract supportedCommands must not contain duplicate item `run`"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_empty_diagnostic_notes() {
        let mut contract = browser_runtime_contract_fixture();
        contract["diagnosticNotes"] = json!([]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("empty diagnosticNotes should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract diagnosticNotes must contain at least one item"
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
    fn browser_runtime_contract_rejects_duplicate_diagnostic_notes_items_after_trim() {
        let mut contract = browser_runtime_contract_fixture();
        contract["diagnosticNotes"] = json!([
            " supported browser runtime commands: run, test ",
            "supported browser runtime commands: run, test",
            "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
            "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
            "browser runtime host description: real browser host"
        ]);

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("duplicate diagnosticNotes item should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract diagnosticNotes must not contain duplicate item `supported browser runtime commands: run, test`"
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
    fn browser_runtime_contract_accepts_trimmed_all_runtime_fields() {
        let mut contract = browser_runtime_contract_fixture();
        contract["hostLabel"] = json!(" browser-requested ");
        contract["hostDescription"] = json!(" real browser host ");
        contract["hostDescriptionNote"] =
            json!(" browser runtime host description: real browser host ");
        contract["supportedCommands"] = json!([" run ", " test "]);
        contract["diagnosticHint"] = json!(" Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work. ");
        contract["summaryNote"] = json!(" browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ");
        contract["contractScopeNote"] = json!(" browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ");
        contract["diagnosticNotes"] = json!([
            " supported browser runtime commands: run, test ",
            " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
            " browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ",
            " browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid ",
            " browser runtime host description: real browser host ",
        ]);

        validate_browser_runtime_contract_value(Some(&contract))
            .expect("trimmed browser runtime contract fields should validate");
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

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_diagnostic_hint() {
        let mut contract = browser_runtime_contract_fixture();
        contract["diagnosticHint"] = json!("   ");

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("whitespace-only diagnosticHint should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract diagnosticHint must be `Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.`"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_wrong_diagnostic_hint() {
        let mut contract = browser_runtime_contract_fixture();
        contract["diagnosticHint"] = json!("Use the browser-targeted command set.");

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("wrong diagnosticHint should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract diagnosticHint must be `Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.`"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_wrong_host_description_note() {
        let mut contract = browser_runtime_contract_fixture();
        contract["hostDescriptionNote"] = json!("browser runtime host description: browser host");

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("wrong hostDescriptionNote should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract hostDescriptionNote must be `browser runtime host description: real browser host`"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_summary_note() {
        let mut contract = browser_runtime_contract_fixture();
        contract["summaryNote"] = json!("   ");

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("whitespace-only summaryNote should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract summaryNote must be `browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work`"
        );
    }

    #[test]
    fn browser_runtime_contract_rejects_whitespace_only_contract_scope_note() {
        let mut contract = browser_runtime_contract_fixture();
        contract["contractScopeNote"] = json!("   ");

        let err = validate_browser_runtime_contract_value(Some(&contract))
            .expect_err("whitespace-only contractScopeNote should be rejected");

        assert_eq!(
            err,
            "doctor browserRuntimeContract contractScopeNote must be `browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness`"
        );
    }
}
