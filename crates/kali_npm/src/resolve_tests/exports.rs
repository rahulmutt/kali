use super::*;

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
