use super::*;

#[test]
fn browser_runtime_unavailable_diagnostic_formats_command_context() {
    let command_diagnostic = browser_runtime_unavailable_diagnostic(Some("run"), None);
    assert!(
        command_diagnostic
            .message
            .contains("run does not support the browser API surface"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .message
            .contains("selected host contract: browser-requested"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .message
            .contains("Phase-1 browser-targeted command set"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "selected host contract: browser-requested"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == "supported browser runtime commands: run, test"),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::contract_scope_note()),
        "diagnostic: {command_diagnostic:?}"
    );
    assert!(
        command_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::host_description_note()),
        "diagnostic: {command_diagnostic:?}"
    );

    let test_diagnostic = browser_runtime_unavailable_diagnostic(Some("test"), None);
    assert!(
        test_diagnostic
            .message
            .contains("test does not support the browser API surface"),
        "diagnostic: {test_diagnostic:?}"
    );
    assert!(
        test_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {test_diagnostic:?}"
    );

    let runtime_diagnostic = browser_runtime_unavailable_diagnostic(None, None);
    assert!(
        runtime_diagnostic
            .message
            .contains("current runtime contract"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .message
            .contains("selected host contract: browser-requested"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .message
            .contains("Phase-1 browser-targeted command set"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "selected host contract: browser-requested"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note
                == "browser harness opt-in env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == "supported browser runtime commands: run, test"),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::contract_scope_note()),
        "diagnostic: {runtime_diagnostic:?}"
    );
    assert!(
        runtime_diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::host_description_note()),
        "diagnostic: {runtime_diagnostic:?}"
    );
}
