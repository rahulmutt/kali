use crate::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn validate_package_shape_rejects_install_time_scripts_without_allow_scripts() {
    let package = PackageJson {
        scripts: BTreeMap::from([
            ("preinstall".to_string(), "echo prep".to_string()),
            ("install".to_string(), "echo install".to_string()),
            ("postinstall".to_string(), "echo done".to_string()),
        ]),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, false).unwrap_err();
    assert_eq!(error[0].code, Some(e6::LIFECYCLE_SCRIPT_REJECTED as u32));
    assert!(error[0]
        .message
        .contains("npm install-time lifecycle scripts require `--allow-scripts`"));
}

#[test]
fn validate_package_shape_allows_non_install_scripts_without_allow_scripts() {
    let package = PackageJson {
        scripts: BTreeMap::from([
            ("test".to_string(), "echo test".to_string()),
            ("lint".to_string(), "echo lint".to_string()),
            ("postlint".to_string(), "echo postlint".to_string()),
            ("posttest".to_string(), "echo posttest".to_string()),
        ]),
        ..PackageJson::default()
    };

    validate_package_shape(&package, false)
        .expect("non-install lifecycle scripts should be treated as ordinary metadata");
}

#[test]
fn validate_package_shape_allows_semver_style_metadata_without_allow_scripts() {
    let package = PackageJson {
        name: Some("semver".to_string()),
        version: Some("7.7.4".to_string()),
        main: Some("index.js".to_string()),
        bin: Some(serde_json::json!({"semver": "bin/semver.js"})),
        scripts: BTreeMap::from([
            ("test".to_string(), "tap".to_string()),
            (
                "lint".to_string(),
                "eslint \"**/*.{js,cjs,ts,mjs,jsx}\"".to_string(),
            ),
            (
                "postlint".to_string(),
                "npm run test -- --ignore-scripts".to_string(),
            ),
            (
                "posttest".to_string(),
                "npm run lint -- --ignore-scripts".to_string(),
            ),
        ]),
        ..PackageJson::default()
    };

    validate_package_shape(&package, false)
        .expect("semver-style package metadata should not require `--allow-scripts`");
}

#[test]
fn validate_package_shape_rejects_node_gyp_install_time_scripts() {
    let package = PackageJson {
        scripts: BTreeMap::from([("install".to_string(), "node-gyp rebuild".to_string())]),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0]
        .message
        .contains("native or binary bootstrap lifecycle script and falls outside the pure JS/TS package contract"));
}

#[test]
fn validate_package_shape_rejects_prebuild_install_time_scripts() {
    let package = PackageJson {
        scripts: BTreeMap::from([(
            "install".to_string(),
            "prebuild-install --download || node-gyp rebuild".to_string(),
        )]),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0]
        .message
        .contains("native or binary bootstrap lifecycle script and falls outside the pure JS/TS package contract"));
}

#[test]
fn validate_package_shape_rejects_native_addon_entrypoints() {
    let package = PackageJson {
        main: Some("native.node".to_string()),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0]
        .message
        .contains("native addon entrypoint and falls outside the pure JS/TS package contract"));
}

#[test]
fn validate_package_shape_rejects_native_exports_entrypoints() {
    let package = PackageJson {
        exports: Some(serde_json::json!({
            "import": "index.js",
            "node": "native.node"
        })),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0]
        .message
        .contains("native addon exports target and falls outside the pure JS/TS package contract"));
}

#[test]
fn validate_package_shape_rejects_native_bin_entrypoints() {
    let package = PackageJson {
        bin: Some(serde_json::json!({"kali-native": "bin/native.node"})),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0].message.contains(
        "bin entry points to a native addon and falls outside the pure JS/TS package contract"
    ));
}

#[test]
fn validate_package_shape_allows_harmless_scripts_when_allowed() {
    let package = PackageJson {
        scripts: BTreeMap::from([("postinstall".to_string(), "echo ok".to_string())]),
        ..PackageJson::default()
    };

    validate_package_shape(&package, true).expect("allowed lifecycle scripts should pass");
}

#[test]
fn validate_package_host_fit_rejects_node_builtin_imports() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("index.js"),
        r#"import fs from "node:fs";
export default fs;
"#,
    )
    .unwrap();

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("fs"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_rejects_node_timers_imports() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("index.js"),
        r#"import timers from "node:timers";
export default timers;
"#,
    )
    .unwrap();

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("timers"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_rejects_node_timers_promises_imports() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("index.js"),
        r#"import timers from "node:timers/promises";
export default timers;
"#,
    )
    .unwrap();

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("timers/promises"));
    assert!(error.message.contains("Phase-3 Node compatibility target"));
}

#[test]
fn validate_package_host_fit_allows_node_builtin_imports_in_node_context() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("index.js"),
        r#"import crypto from "node:crypto";
export default crypto;
"#,
    )
    .unwrap();

    validate_package_host_fit(dir.path(), PackageHostFitContext::Node)
        .expect("node host fit should allow Node builtins");
}

#[test]
fn validate_package_host_fit_rejects_node_builtin_requires() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("index.cjs"),
        r#"const childProcess = require("child_process");
module.exports = childProcess;
"#,
    )
    .unwrap();

    let error = validate_package_host_fit(dir.path(), PackageHostFitContext::DefaultStandalone)
        .unwrap_err();
    assert_eq!(error.code, Some(e6::NODE_ONLY_HOST_APIS as u32));
    assert!(error.message.contains("child_process"));
}
