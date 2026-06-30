use super::*;

#[test]
fn install_rejects_allow_scripts_without_effective_npm_work() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_manifest(dir.path(), r#"{"schemaVersion":1}"#);

    let error = install_project(
        dir.path(),
        InstallOptions {
            allow_scripts: true,
            ..InstallOptions::default()
        },
    )
    .unwrap_err();

    assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
    assert!(error[0]
        .message
        .contains("requires non-empty npm install work"));
}

#[test]
fn install_rejects_allow_scripts_for_jsr_targets() {
    let dir = kali_test_support::fixtures::tempdir();

    let error = install_project(
        dir.path(),
        InstallOptions {
            target: Some("jsr:@std/path".to_string()),
            allow_scripts: true,
            ..InstallOptions::default()
        },
    )
    .unwrap_err();

    assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
    assert!(error[0].message.contains("not valid for JSR targets"));
}

#[test]
fn install_rejects_allow_scripts_for_raw_url_targets() {
    let dir = kali_test_support::fixtures::tempdir();

    let error = install_project(
        dir.path(),
        InstallOptions {
            target: Some("https://example.com/mod.ts".to_string()),
            allow_scripts: true,
            ..InstallOptions::default()
        },
    )
    .unwrap_err();

    assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
    assert!(error[0].message.contains("not valid for raw-URL targets"));
}

#[test]
fn install_rejects_dev_without_explicit_target() {
    let dir = kali_test_support::fixtures::tempdir();

    let error = install_project(
        dir.path(),
        InstallOptions {
            dev: true,
            ..InstallOptions::default()
        },
    )
    .unwrap_err();

    assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
    assert!(error[0]
        .message
        .contains("requires an explicit registry package target"));
}

#[test]
fn install_rejects_dev_for_raw_url_targets() {
    let dir = kali_test_support::fixtures::tempdir();

    let error = install_project(
        dir.path(),
        InstallOptions {
            target: Some("https://example.com/mod.ts".to_string()),
            dev: true,
            ..InstallOptions::default()
        },
    )
    .unwrap_err();

    assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
    assert!(error[0].message.contains("not valid for raw-URL targets"));
}

#[test]
fn install_rejects_versioned_registry_targets() {
    for target in ["lodash@1.2.3", "jsr:@std/path@1.0.0"] {
        let dir = kali_test_support::fixtures::tempdir();

        let error = install_project(
            dir.path(),
            InstallOptions {
                target: Some(target.to_string()),
                ..InstallOptions::default()
            },
        )
        .unwrap_err();

        assert_eq!(error[0].code, Some(e5::INVALID_CLI_USAGE as u32));
        assert!(error[0]
            .message
            .contains("accepts only registry package identifiers, not explicit versions"));
        assert!(
            !dir.path().join("kali.json").exists(),
            "kali.json should not be created"
        );
        assert!(
            !dir.path().join("kali.lock").exists(),
            "kali.lock should not be created"
        );
    }
}
