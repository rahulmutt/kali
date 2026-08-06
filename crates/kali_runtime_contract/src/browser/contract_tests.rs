use crate::*;

#[test]
fn browser_runtime_contract_documents_the_future_execution_surface() {
    let descriptor = BrowserRuntimeContract::descriptor();

    assert!(browser_runtime_contract_descriptor_is_canonical(
        &descriptor
    ));
    assert_eq!(descriptor.host_label, "browser-requested");
    assert_eq!(descriptor.host_description, "real browser host");
    assert_eq!(
        descriptor.host_description_note,
        "browser runtime host description: real browser host"
    );
    assert_eq!(descriptor.supported_commands, &["run", "test"]);
    assert_eq!(
        descriptor.supported_commands_note,
        "supported browser runtime commands: run, test"
    );
    assert_eq!(
        descriptor.summary_note,
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    );
    assert_eq!(
        descriptor.contract_scope_note,
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    );
    assert_eq!(
        BrowserRuntimeContract::summary_file_fallback_note(),
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    );
    let contract = browser_runtime_contract_value();
    assert_eq!(contract["hostLabel"], "browser-requested");
    assert_eq!(contract["hostDescription"], "real browser host");
    assert_eq!(
        contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        contract["supportedCommands"]
            .as_array()
            .expect("supportedCommands array"),
        &[
            serde_json::Value::String("run".to_string()),
            serde_json::Value::String("test".to_string()),
        ]
    );
    assert_eq!(
        contract["diagnosticHint"],
        "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work."
    );
    assert_eq!(
        contract["diagnosticNotes"]
            .as_array()
            .expect("diagnosticNotes array"),
        &[
            serde_json::Value::String(
                "supported browser runtime commands: run, test".to_string(),
            ),
            serde_json::Value::String(
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work".to_string(),
            ),
            serde_json::Value::String(
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness".to_string(),
            ),
            serde_json::Value::String(
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid".to_string(),
            ),
            serde_json::Value::String(
                "browser runtime host description: real browser host".to_string(),
            ),
        ]
    );
    assert_eq!(
        BrowserRuntimeContract::diagnostic_notes(),
        &[
            BrowserRuntimeContract::supported_commands_note(),
            BrowserRuntimeContract::summary_note(),
            BrowserRuntimeContract::contract_scope_note(),
            BrowserRuntimeContract::summary_file_fallback_note(),
            BrowserRuntimeContract::host_description_note(),
        ]
    );
    assert_eq!(
        contract,
        serde_json::json!({
            "hostLabel": BrowserRuntimeContract::host_label(),
            "hostDescription": BrowserRuntimeContract::host_description(),
            "hostDescriptionNote": BrowserRuntimeContract::host_description_note(),
            "supportedCommands": BrowserRuntimeContract::supported_commands(),
            "diagnosticHint": BrowserRuntimeContract::diagnostic_hint(),
            "summaryNote": BrowserRuntimeContract::summary_note(),
            "contractScopeNote": BrowserRuntimeContract::contract_scope_note(),
            "diagnosticNotes": BrowserRuntimeContract::diagnostic_notes(),
        })
    );
    assert!(descriptor
        .diagnostic_hint
        .contains("kali check --api browser"));
    assert!(descriptor
        .diagnostic_hint
        .contains("kali build --bundle --api browser"));
}

#[test]
fn browser_runtime_contract_descriptor_rejects_duplicate_or_whitespace_values() {
    let invalid_label = BrowserRuntimeContractDescriptor {
        host_label: " browser-requested",
        host_description: "real browser host",
        host_description_note: "browser runtime host description: real browser host",
        supported_commands: &["run", "test"],
        supported_commands_note: "supported browser runtime commands: run, test",
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_description = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: " real browser host",
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: &["run", "test"],
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_description_note = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: " browser runtime host description: real browser host ",
        supported_commands: BrowserRuntimeContract::supported_commands(),
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_command = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: &["run", "run"],
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_command_whitespace = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: &["run", " test "],
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_hint = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: BrowserRuntimeContract::supported_commands(),
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: " ",
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_supported_commands_note = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: BrowserRuntimeContract::supported_commands(),
        supported_commands_note: " supported browser runtime commands: run, test ",
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_summary_note = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: BrowserRuntimeContract::supported_commands(),
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
        contract_scope_note: BrowserRuntimeContract::contract_scope_note(),
    };
    let invalid_contract_scope_note = BrowserRuntimeContractDescriptor {
        host_label: BrowserRuntimeContract::host_label(),
        host_description: BrowserRuntimeContract::host_description(),
        host_description_note: BrowserRuntimeContract::host_description_note(),
        supported_commands: BrowserRuntimeContract::supported_commands(),
        supported_commands_note: BrowserRuntimeContract::supported_commands_note(),
        diagnostic_hint: BrowserRuntimeContract::diagnostic_hint(),
        summary_note: BrowserRuntimeContract::summary_note(),
        contract_scope_note: " browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ",
    };

    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_label
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_description
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_description_note
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_command
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_command_whitespace
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_hint
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_supported_commands_note
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_summary_note
    ));
    assert!(!browser_runtime_contract_descriptor_is_canonical(
        &invalid_contract_scope_note
    ));
    assert!(browser_runtime_contract_descriptor_is_canonical(
        &BrowserRuntimeContract::descriptor()
    ));
}
