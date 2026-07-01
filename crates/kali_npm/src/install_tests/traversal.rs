use super::*;

#[test]
fn collect_reachable_registry_packages_rejects_install_path_conflicts() {
    let lock = LockFile {
        version: LOCK_VERSION,
        packages: BTreeMap::from([
            (
                "@scope/name@1.0.0".to_string(),
                LockedPackage {
                    registry: "npm".to_string(),
                    integrity: "sha512-demo".to_string(),
                    resolved: "https://example.com/scope-name.tgz".to_string(),
                    dependencies: BTreeMap::new(),
                },
            ),
            (
                "jsr:@scope/name@1.0.0".to_string(),
                LockedPackage {
                    registry: "jsr".to_string(),
                    integrity: "sha512-demo".to_string(),
                    resolved: "https://example.com/jsr-scope-name.tgz".to_string(),
                    dependencies: BTreeMap::new(),
                },
            ),
        ]),
        ..LockFile::default()
    };

    let error = collect_reachable_registry_packages(
        &lock,
        &[
            "@scope/name@1.0.0".to_string(),
            "jsr:@scope/name@1.0.0".to_string(),
        ],
    )
    .unwrap_err();
    assert_eq!(error.code, Some(e6::VERSION_MISMATCH as u32));
}

#[test]
fn install_noops_without_manifest_or_dependencies() {
    let dir = kali_test_support::fixtures::tempdir();

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();

    assert!(summary.manifest_path.is_none());
    assert!(summary.lock_path.is_none());
    assert!(summary.installed.is_empty());
    assert!(!dir.path().join("kali.json").exists());
    assert!(!dir.path().join("kali.lock").exists());
}

#[test]
fn install_stops_at_nested_child_project_roots() {
    let dir = kali_test_support::fixtures::tempdir();
    let root_raw_url = start_raw_url_server("export default 'root';\n");
    let child_raw_url = start_raw_url_server("export default 'child';\n");

    kali_test_support::fixtures::write_manifest(
        dir.path(),
        r#"{
  "schemaVersion": 1
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "main.ts",
        &format!("import '{}';\n", root_raw_url),
    );

    let child_dir = dir.path().join("child");
    fs::create_dir(&child_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "child/kali.json",
        r#"{"schemaVersion":1}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "child/main.ts",
        &format!("import '{}';\n", child_raw_url),
    );

    let manifest = load_manifest(dir.path()).unwrap().unwrap();
    let discovered = discover_install_time_raw_urls(dir.path(), &manifest).unwrap();
    assert!(
        discovered.contains(&root_raw_url),
        "discovered: {discovered:?}"
    );
    assert!(
        !discovered.contains(&child_raw_url),
        "discovered: {discovered:?}"
    );

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();
    assert!(summary.lock_path.is_some());

    let lock = load_lock(dir.path()).unwrap().unwrap();
    assert!(lock.raw_urls.contains_key(&root_raw_url), "lock: {lock:#?}");
    assert!(
        !lock.raw_urls.contains_key(&child_raw_url),
        "lock: {lock:#?}"
    );
}
