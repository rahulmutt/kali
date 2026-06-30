use super::*;

#[test]
fn lifecycle_hooks_run_in_order_when_allowed() {
    let dir = kali_test_support::fixtures::tempdir();
    let marker = dir.path().join("hook-order.txt");
    let package = PackageJson {
        scripts: BTreeMap::from([
            (
                "preinstall".to_string(),
                append_marker_command(&marker, "pre"),
            ),
            (
                "install".to_string(),
                append_marker_command(&marker, "install"),
            ),
            (
                "postinstall".to_string(),
                append_marker_command(&marker, "post"),
            ),
        ]),
        ..PackageJson::default()
    };

    run_package_lifecycle_hooks(dir.path(), &package, true, true).unwrap();

    let contents = fs::read_to_string(&marker).unwrap();
    assert_eq!(contents, "pre\ninstall\npost\n");
}

#[test]
fn lifecycle_hooks_skip_blank_entries() {
    let dir = kali_test_support::fixtures::tempdir();
    let marker = dir.path().join("hook-skip.txt");
    let package = PackageJson {
        scripts: BTreeMap::from([("install".to_string(), "   ".to_string())]),
        ..PackageJson::default()
    };

    run_package_lifecycle_hooks(dir.path(), &package, true, true).unwrap();
    assert!(!marker.exists(), "blank lifecycle hook should be skipped");
}
