use crate::*;
use std::fs;

#[test]
fn bare_import_resolves_from_materialized_package() {
    let dir = kali_test_support::fixtures::tempdir();
    let package_dir = dir.path().join("node_modules/lodash");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/lodash/package.json",
        r#"{"name":"lodash","main":"lodash.js"}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/lodash/lodash.js",
        "export default 1;",
    );

    let resolved = resolve_materialized_import(dir.path(), "lodash");
    assert_eq!(resolved.unwrap(), package_dir.join("lodash.js"));
}

#[test]
fn bare_import_resolves_via_types_package_dependency() {
    let dir = kali_test_support::fixtures::tempdir();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "kali.json",
        r#"{
  "schemaVersion": 1,
  "devDependencies": {
    "@types/lodash": "1.0.0"
  }
}"#,
    );

    let types_dir = dir.path().join("node_modules/@types/lodash");
    fs::create_dir_all(&types_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/@types/lodash/package.json",
        r#"{"name":"@types/lodash","types":"index.d.ts"}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/@types/lodash/index.d.ts",
        "declare const _: number;",
    );

    let resolved = resolve_materialized_import(dir.path(), "lodash");
    assert_eq!(resolved.unwrap(), types_dir.join("index.d.ts"));
}

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

#[test]
fn exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions() {
    let dir = kali_test_support::fixtures::tempdir();
    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/package.json",
        r#"{
  "name": "widget",
  "main": "legacy.js",
  "exports": {
    ".": {
      "deno": "./entry.deno.js",
      "browser": "./entry.browser.js",
      "default": "./entry.default.js"
    }
  }
}"#,
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/legacy.js",
        "export default 'legacy';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/entry.deno.js",
        "export default 'deno';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/entry.browser.js",
        "export default 'browser';",
    );
    kali_test_support::fixtures::write_file(
        dir.path(),
        "node_modules/widget/entry.default.js",
        "export default 'default';",
    );

    let resolved_deno = resolve_materialized_import(dir.path(), "widget");
    assert_eq!(resolved_deno.unwrap(), package_dir.join("entry.deno.js"));

    let resolved_browser =
        resolve_materialized_import_with_browser_context(dir.path(), "widget", true);
    assert_eq!(
        resolved_browser.unwrap(),
        package_dir.join("entry.browser.js")
    );
}
