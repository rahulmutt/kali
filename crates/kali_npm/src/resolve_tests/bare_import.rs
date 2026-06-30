use super::*;

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
