use super::*;

#[test]
fn browser_replacement_maps_rewrite_selected_root_entries() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "kali.json",
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    );

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/package.json",
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": "./index.browser.js"
  }
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/index.js",
        "export default 'node';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/index.browser.js",
        "export default 'browser';",
    );

    let resolved = resolve_materialized_import(dir.path(), "widget");
    assert_eq!(resolved.unwrap(), package_dir.join("index.browser.js"));
}

#[test]
fn browser_replacement_maps_rewrite_selected_root_entries_from_explicit_context() {
    let dir = kali_test_support::fixtures::tempdir();

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/package.json",
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": "./index.browser.js"
  }
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/index.js",
        "export default 'node';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/index.browser.js",
        "export default 'browser';",
    );

    let resolved = resolve_materialized_import_with_browser_context(dir.path(), "widget", true);
    assert_eq!(resolved.unwrap(), package_dir.join("index.browser.js"));
}

#[test]
fn browser_replacement_maps_can_block_selected_root_entries() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "kali.json",
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    );

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/package.json",
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": false
  }
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/index.js",
        "export default 'node';",
    );

    let resolved = resolve_materialized_import(dir.path(), "widget");
    assert!(
        resolved.is_none(),
        "browser-disabled root entry should not resolve"
    );
}

#[test]
fn browser_replacement_maps_rewrite_selected_subpaths() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "kali.json",
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    );

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/package.json",
        r#"{
  "name": "widget",
  "browser": {
    "./feature.js": "./feature.browser.js"
  }
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/feature.js",
        "export default 'node';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/feature.browser.js",
        "export default 'browser';",
    );

    let resolved = resolve_materialized_import(dir.path(), "widget/feature");
    assert_eq!(resolved.unwrap(), package_dir.join("feature.browser.js"));
}
