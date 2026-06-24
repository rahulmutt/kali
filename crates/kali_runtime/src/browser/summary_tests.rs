use crate::test_support::*;
use crate::*;
use std::fs;

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

#[test]
fn browser_bundle_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_merges_thread_topology_from_stdout_when_summary_file_is_missing_it(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_keeps_thread_topology_from_summary_file() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_thread_topology_is_invalid(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\" https://example.com/thread.js \",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_null_value() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":null,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_keeps_thread_topology_from_summary_file() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_thread_topology_from_stdout_when_summary_file_is_missing_it(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let summary_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary.json",
        r#"{"args":["alpha"],"tests":["browser merge"],"testsFailed":2,"hostContract":"browser-requested","runtimeBackend":"browser-harness"}"#,
    );

    let outcome = BrowserHarnessOutcome {
        command: vec!["node".to_string()],
        status: browser_exit_status(0),
        stdout: r#"{"args":["stdout"],"tests":["browser merge"],"testsFailed":1,"hostContract":"browser-requested","runtimeBackend":"browser-harness","threadTopology":{"totalInstances":1,"terminatedInstances":0,"liveInstances":[{"instanceId":0,"scriptUrl":"https://example.com/stdout-thread.js","postedMessages":[],"postedSharedBuffers":[],"wasTerminated":false}]}}"#.to_string(),
        stderr: String::new(),
    };

    let summary = super::browser_runtime_summary_for_outcome(&summary_path, &outcome);
    assert_eq!(summary.args, vec!["alpha".to_string()]);
    assert_eq!(summary.tests, vec!["browser merge".to_string()]);
    assert_eq!(summary.tests_failed, Some(2));
    assert_eq!(
        summary.host_contract,
        Some(RuntimeHostContract::BrowserRequested)
    );
    assert_eq!(
        summary.runtime_backend,
        Some(RuntimeBackend::BrowserHarness)
    );
    assert_eq!(
        summary.thread_topology.unwrap().snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
}

#[test]
fn browser_requested_runtime_summary_merges_thread_topology_from_stdout_end_to_end_when_summary_file_is_missing_it(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_thread_topology_is_invalid(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\" https://example.com/thread.js \",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#),
        &wasm,
        &["stdout".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_trims_labels_before_falling_back_to_stdout_thread_topology_when_summary_file_thread_topology_is_invalid(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\" browser-requested \",\"runtimeBackend\":\" browser-harness \",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"worker.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#),
        &wasm,
        &["stdout".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_thread_topology_script_urls_are_whitespace_only(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"   \",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#),
        &wasm,
        &["stdout".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_thread_topology_script_urls_are_relative(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"worker.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#),
        &wasm,
        &["stdout".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_reports_thread_topology_from_thread_spawn() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "https://example.com/thread.js")
                (func (export "_start")
                    i32.const 0
                    i32.const 29
                    call $thread_spawn
                    drop))
        "#,
    );

    let outcome = browser_runtime_execute_checked(Some("node"), &wasm, &[], tempdir.path(), false)
        .expect("execute browser requested runtime harness with thread spawn");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    let instance = &outcome.thread_topology.live_instances[0];
    assert_eq!(instance.instance_id, 0);
    assert_eq!(instance.script_url, "https://example.com/thread.js");
    assert!(instance.posted_messages.is_empty());
    assert!(instance.posted_shared_buffers.is_empty());
    assert!(!instance.was_terminated);
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
}

#[test]
fn browser_requested_runtime_rejects_whitespace_thread_spawn_script_url() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "thread_spawn" (func $thread_spawn (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) " https://e.co/padded.js ")
                (func (export "_start")
                    i32.const 0
                    i32.const 24
                    call $thread_spawn
                    drop))
        "#,
    );

    let outcome = browser_runtime_execute_checked(Some("node"), &wasm, &[], tempdir.path(), false)
        .expect("execute browser requested runtime harness with invalid thread spawn url");

    assert_ne!(outcome.status.code(), Some(0));
    assert!(
        outcome
            .stderr
            .contains("browser runtime thread_spawn scriptUrl must be a canonical absolute URL"),
        "stderr: {}",
        outcome.stderr
    );
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_invalid_type()
{
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_tests_failed(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"file test\"],\"testsFailed\":\"oops\",\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"stdout test\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["stdout test".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_non_integer_number(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":1.5,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_negative_number(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":-1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_null_value() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":null,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":7,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["alpha".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 7);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":7"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser missing\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser missing".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser unparseable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser unparseable".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 4);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_whitespace_only_labels(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":4,\"hostContract\":\"   \",\"runtimeBackend\":\"   \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 4);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels_and_is_missing_tests_failed(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":9"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid array items\"],\"testsFailed\":4,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":9"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_requested_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels_and_invalid_args(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );

    let outcome = browser_runtime_execute_checked(
        Some(r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[1],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":4,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels and args\"],\"testsFailed\":9,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#),
        &wasm,
        &["zeta".to_string()],
        tempdir.path(),
        false,
    )
    .expect("execute browser requested runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 9);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels and args".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser missing\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser missing".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[cfg(unix)]
#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unreadable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser unreadable".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":2,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_metadata_when_summary_file_has_whitespace_only_labels(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":2,\"hostContract\":\"   \",\"runtimeBackend\":\"   \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[1],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 8);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":8"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_labels_when_summary_file_lacks_them() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"stdout\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.tests_run(), 1);
}
