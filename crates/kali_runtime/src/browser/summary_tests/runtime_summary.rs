use super::*;

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(summary.host_contract, None);
    assert_eq!(summary.runtime_backend, None);
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        "not-json",
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(summary.host_contract, None);
    assert_eq!(summary.runtime_backend, None);
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        " \n\t\n",
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_empty() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path =
        kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime-summary.json", "");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_unreadable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::create_dir(&summary_path).expect("create unreadable summary path");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_is_incomplete() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_unexpected_keys() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["summary"],"tests":["browser extra key"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness","unexpected":true}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":9,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["stdout".to_string()]);
    assert_eq!(summary.tests_failed, Some(9));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_stdout_labels_when_summary_file_labels_are_invalid() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":4,"hostContract":"not-a-contract","runtimeBackend":"not-a-backend"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":9,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(4));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_stdout_metadata_when_summary_file_has_invalid_labels_and_invalid_tests_failed_type(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":"oops","hostContract":"not-a-contract","runtimeBackend":"not-a-backend"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":9,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["stdout".to_string()]);
    assert_eq!(summary.tests_failed, Some(9));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":[1],"tests":["browser invalid array items"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser invalid array items"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(
        summary.tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_blank_args_item() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["  "],"tests":["browser blank args item"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser blank args item"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser blank args item".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_blank_tests_item() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["summary"],"tests":["\t"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["summary"],"tests":["stdout"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["summary".to_string()]);
    assert_eq!(summary.tests, vec!["stdout".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_padded_args_item() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":[" summary "],"tests":["browser padded args item"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser padded args item"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser padded args item".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_padded_tests_item() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["summary"],"tests":[" browser padded tests item "],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["summary"],"tests":["stdout"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["summary".to_string()]);
    assert_eq!(summary.tests, vec!["stdout".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_null_args_field() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":null,"tests":["browser null args"],"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser null args"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser null args".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_falls_back_to_stdout_when_summary_file_has_null_tests_field() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["stdout"],"tests":null,"testsFailed":4,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser null tests"],"testsFailed":8,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["stdout".to_string()]);
    assert_eq!(summary.tests, vec!["browser null tests".to_string()]);
    assert_eq!(summary.tests_failed, Some(8));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(1),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":1,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(1));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_uses_stdout_labels_when_summary_file_lacks_them() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_uses_stdout_labels_when_summary_file_labels_are_empty() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"","runtimeBackend":""}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_uses_stdout_labels_when_summary_file_labels_are_whitespace_only() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":" \n\t ","runtimeBackend":"  \t"}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_trims_labels_before_normalizing() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":" browser-requested ","runtimeBackend":" browser-harness "}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":0,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_trims_labels_and_preserves_stdout_tests_failed_fallback() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["zeta"],"tests":["7"],"hostContract":" browser-requested ","runtimeBackend":" browser-harness "}"#,
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":11,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(11));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_prefers_the_last_json_line_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        "still-not-json",
    );
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: [
            "guest log line\n",
            r#"{"args":["ignored"],"tests":["1"],"testsFailed":9,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#,
            "\ntrailing non-json noise\n",
            r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
        ]
        .concat(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}

#[test]
fn browser_runtime_summary_prefers_the_last_json_line_from_a_noisy_summary_file() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    fs::write(
        &summary_path,
        [
            "summary log line\n",
            r#"{"args":["ignored"],"tests":["1"],"testsFailed":4,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#,
            "\nsummary trailing noise\n",
            r#"{"args":["zeta"],"tests":["7"],"testsFailed":0,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
        ]
        .concat(),
    )
    .expect("write noisy summary file");
    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["stdout"],"testsFailed":3,"hostContract":"kali-hosted","runtimeBackend":"wasmtime"}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["zeta".to_string()]);
    assert_eq!(summary.tests, vec!["7".to_string()]);
    assert_eq!(summary.tests_failed, Some(0));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
}
