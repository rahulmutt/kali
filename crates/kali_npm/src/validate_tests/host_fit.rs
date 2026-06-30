use super::*;

#[test]
fn validate_package_host_fit_rejects_node_builtin_imports() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "index.js",
        r#"import fs from "node:fs";
export default fs;
"#,
    );

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("fs"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_rejects_node_timers_imports() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "index.js",
        r#"import timers from "node:timers";
export default timers;
"#,
    );

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("timers"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_rejects_node_timers_promises_imports() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "index.js",
        r#"import timers from "node:timers/promises";
export default timers;
"#,
    );

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("timers/promises"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_allows_node_builtin_imports_in_node_context() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "index.js",
        r#"import crypto from "node:crypto";
export default crypto;
"#,
    );

    validate_package_host_fit(dir.path(), PackageHostFitContext::Node)
        .expect("node host fit should allow Node builtins");
}

#[test]
fn validate_package_host_fit_rejects_node_builtin_requires() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "index.cjs",
        r#"const childProcess = require("child_process");
module.exports = childProcess;
"#,
    );

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("child_process"));
}
