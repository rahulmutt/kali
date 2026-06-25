use crate::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn bare_import_resolves_from_materialized_package() {
    let dir = tempdir().unwrap();
    let package_dir = dir.path().join("node_modules/lodash");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{"name":"lodash","main":"lodash.js"}"#,
    )
    .unwrap();
    fs::write(package_dir.join("lodash.js"), "export default 1;").unwrap();

    let resolved = resolve_materialized_import(dir.path(), "lodash");
    assert_eq!(resolved.unwrap(), package_dir.join("lodash.js"));
}

#[test]
fn bare_import_resolves_via_types_package_dependency() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "devDependencies": {
    "@types/lodash": "1.0.0"
  }
}"#,
    )
    .unwrap();

    let types_dir = dir.path().join("node_modules/@types/lodash");
    fs::create_dir_all(&types_dir).unwrap();
    fs::write(
        types_dir.join("package.json"),
        r#"{"name":"@types/lodash","types":"index.d.ts"}"#,
    )
    .unwrap();
    fs::write(types_dir.join("index.d.ts"), "declare const _: number;").unwrap();

    let resolved = resolve_materialized_import(dir.path(), "lodash");
    assert_eq!(resolved.unwrap(), types_dir.join("index.d.ts"));
}

#[test]
fn browser_replacement_maps_rewrite_selected_root_entries() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .unwrap();

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": "./index.browser.js"
  }
}"#,
    )
    .unwrap();
    fs::write(package_dir.join("index.js"), "export default 'node';").unwrap();
    fs::write(
        package_dir.join("index.browser.js"),
        "export default 'browser';",
    )
    .unwrap();

    let resolved = resolve_materialized_import(dir.path(), "widget");
    assert_eq!(resolved.unwrap(), package_dir.join("index.browser.js"));
}

#[test]
fn browser_replacement_maps_rewrite_selected_root_entries_from_explicit_context() {
    let dir = tempdir().unwrap();

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": "./index.browser.js"
  }
}"#,
    )
    .unwrap();
    fs::write(package_dir.join("index.js"), "export default 'node';").unwrap();
    fs::write(
        package_dir.join("index.browser.js"),
        "export default 'browser';",
    )
    .unwrap();

    let resolved = resolve_materialized_import_with_browser_context(dir.path(), "widget", true);
    assert_eq!(resolved.unwrap(), package_dir.join("index.browser.js"));
}

#[test]
fn browser_replacement_maps_can_block_selected_root_entries() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .unwrap();

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "widget",
  "main": "index.js",
  "browser": {
    "./index.js": false
  }
}"#,
    )
    .unwrap();
    fs::write(package_dir.join("index.js"), "export default 'node';").unwrap();

    let resolved = resolve_materialized_import(dir.path(), "widget");
    assert!(
        resolved.is_none(),
        "browser-disabled root entry should not resolve"
    );
}

#[test]
fn browser_replacement_maps_rewrite_selected_subpaths() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .unwrap();

    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "widget",
  "browser": {
    "./feature.js": "./feature.browser.js"
  }
}"#,
    )
    .unwrap();
    fs::write(package_dir.join("feature.js"), "export default 'node';").unwrap();
    fs::write(
        package_dir.join("feature.browser.js"),
        "export default 'browser';",
    )
    .unwrap();

    let resolved = resolve_materialized_import(dir.path(), "widget/feature");
    assert_eq!(resolved.unwrap(), package_dir.join("feature.browser.js"));
}

#[test]
fn exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions() {
    let dir = tempdir().unwrap();
    let package_dir = dir.path().join("node_modules/widget");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("package.json"),
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
    )
    .unwrap();
    fs::write(package_dir.join("legacy.js"), "export default 'legacy';").unwrap();
    fs::write(package_dir.join("entry.deno.js"), "export default 'deno';").unwrap();
    fs::write(
        package_dir.join("entry.browser.js"),
        "export default 'browser';",
    )
    .unwrap();
    fs::write(
        package_dir.join("entry.default.js"),
        "export default 'default';",
    )
    .unwrap();

    let resolved_deno = resolve_materialized_import(dir.path(), "widget");
    assert_eq!(resolved_deno.unwrap(), package_dir.join("entry.deno.js"));

    let resolved_browser =
        resolve_materialized_import_with_browser_context(dir.path(), "widget", true);
    assert_eq!(
        resolved_browser.unwrap(),
        package_dir.join("entry.browser.js")
    );
}
