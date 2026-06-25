use super::*;
use crate::test_support::*;
use std::fs;
use std::sync::atomic::Ordering;
use tempfile::tempdir;

use serde_json::json;


#[test]
fn lifecycle_hooks_run_in_order_when_allowed() {
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
    let marker = dir.path().join("hook-skip.txt");
    let package = PackageJson {
        scripts: BTreeMap::from([("install".to_string(), "   ".to_string())]),
        ..PackageJson::default()
    };

    run_package_lifecycle_hooks(dir.path(), &package, true, true).unwrap();
    assert!(!marker.exists(), "blank lifecycle hook should be skipped");
}


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
fn install_reconciles_raw_urls_from_source_import_map_rewrites() {
    let dir = tempdir().unwrap();
    let raw_url = start_raw_url_server("export default 1;");
    let raw_prefix = raw_url.trim_end_matches("mod.ts").to_string();
    fs::write(
        dir.path().join("kali.json"),
        format!(
            r#"{{
  "schemaVersion": 1,
  "imports": {{
    "raw/": "{}"
  }}
}}"#,
            raw_prefix
        ),
    )
    .unwrap();
    fs::write(dir.path().join("main.ts"), "import 'raw/mod.ts';\n").unwrap();

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();
    assert!(summary.lock_path.is_some());

    let lock = load_lock(dir.path()).unwrap().unwrap();
    assert!(lock.raw_urls.contains_key(&raw_url), "lock: {lock:#?}");
    let cached = Path::new(&lock.raw_urls.get(&raw_url).unwrap().cached).to_path_buf();
    assert!(cached.exists(), "cached raw url was not materialized");

    fs::write(dir.path().join("main.ts"), "export {};\n").unwrap();
    let manifest = load_manifest(dir.path()).unwrap().unwrap();
    let discovered = discover_install_time_raw_urls(dir.path(), &manifest).unwrap();
    assert!(
        discovered.is_empty(),
        "discovered raw urls: {:?}",
        discovered
    );
    install_project(dir.path(), InstallOptions::default()).unwrap();

    assert!(!dir.path().join("kali.lock").exists());
    assert!(!cached.exists(), "stale raw url cache was not pruned");
}

#[test]
fn install_is_idempotent_for_unchanged_raw_url_graph() {
    let dir = tempdir().unwrap();
    let raw_url = start_raw_url_server("export default 1;");
    let raw_prefix = raw_url.trim_end_matches("mod.ts").to_string();
    fs::write(
        dir.path().join("kali.json"),
        format!(
            r#"{{
  "schemaVersion": 1,
  "imports": {{
    "raw/": "{}"
  }}
}}"#,
            raw_prefix
        ),
    )
    .unwrap();
    fs::write(dir.path().join("main.ts"), "import 'raw/mod.ts';\n").unwrap();

    install_project(dir.path(), InstallOptions::default()).unwrap();
    let first_lock = fs::read(dir.path().join("kali.lock")).unwrap();

    install_project(dir.path(), InstallOptions::default()).unwrap();
    let second_lock = fs::read(dir.path().join("kali.lock")).unwrap();

    assert_eq!(
        first_lock, second_lock,
        "lock file changed across identical installs"
    );
}

#[test]
fn install_noops_without_manifest_or_dependencies() {
    let dir = tempdir().unwrap();

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();

    assert!(summary.manifest_path.is_none());
    assert!(summary.lock_path.is_none());
    assert!(summary.installed.is_empty());
    assert!(!dir.path().join("kali.json").exists());
    assert!(!dir.path().join("kali.lock").exists());
}

#[test]
fn install_stops_at_nested_child_project_roots() {
    let dir = tempdir().unwrap();
    let root_raw_url = start_raw_url_server("export default 'root';\n");
    let child_raw_url = start_raw_url_server("export default 'child';\n");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1
}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("main.ts"),
        format!("import '{}';\n", root_raw_url),
    )
    .unwrap();

    let child_dir = dir.path().join("child");
    fs::create_dir(&child_dir).unwrap();
    fs::write(child_dir.join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();
    fs::write(
        child_dir.join("main.ts"),
        format!("import '{}';\n", child_raw_url),
    )
    .unwrap();

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

#[test]
fn install_rejects_allow_scripts_without_effective_npm_work() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();

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
    let dir = tempdir().unwrap();

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
    let dir = tempdir().unwrap();

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
    let dir = tempdir().unwrap();

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
    let dir = tempdir().unwrap();

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
        let dir = tempdir().unwrap();

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


#[test]
fn install_reconciles_semver_style_package_without_allow_scripts() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().unwrap();

    let package_json = json!({
        "name": "semver",
        "version": "7.7.4",
        "main": "index.js",
        "bin": { "semver": "bin/semver.js" },
        "scripts": {
            "test": "tap",
            "lint": "eslint \"**/*.{js,cjs,ts,mjs,jsx}\"",
            "postlint": "npm run test -- --ignore-scripts",
            "posttest": "npm run lint -- --ignore-scripts"
        }
    });
    let package_json_bytes = serde_json::to_vec_pretty(&package_json).unwrap();
    let tarball_bytes = build_package_tarball(&[
        ("package/package.json", package_json_bytes.as_slice()),
        ("package/index.js", b"module.exports = {};\n"),
        (
            "package/bin/semver.js",
            b"#!/usr/bin/env node\nconsole.log('semver');\n",
        ),
    ]);
    let tarball_integrity = format!("sha512-{}", format_sha512(&tarball_bytes));
    let (tarball_base, tarball_hits, tarball_stop, tarball_handle) =
        start_response_server(tarball_bytes, "application/octet-stream");

    let metadata = json!({
        "versions": {
            "7.7.4": {
                "dist": {
                    "tarball": format!("{}/semver-7.7.4.tgz", tarball_base),
                    "integrity": tarball_integrity
                }
            }
        }
    });
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_response_server(serde_json::to_vec(&metadata).unwrap(), "application/json");
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"dependencies":{"semver":"7.7.4"}}"#,
    )
    .unwrap();

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    let lock_path = dir.path().join("kali.lock");
    assert_eq!(summary.lock_path.as_deref(), Some(lock_path.as_path()));
    assert_eq!(summary.installed, vec![package_key("semver", "7.7.4")]);

    let lock = load_lock(dir.path()).unwrap().unwrap();
    assert!(
        lock.packages.contains_key("semver@7.7.4"),
        "lock: {lock:#?}"
    );
    assert!(dir.path().join("node_modules/semver/package.json").exists());
    assert!(dir
        .path()
        .join(".kali-cache/packages/semver@7.7.4/package/package.json")
        .exists());

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().unwrap();
    registry_handle.join().unwrap();
    assert_eq!(tarball_hits.load(Ordering::SeqCst), 1);
    assert_eq!(registry_hits.load(Ordering::SeqCst), 1);
}

#[test]
fn install_reconciles_semver_style_package_with_allow_scripts_noop() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().unwrap();

    let package_json = json!({
        "name": "semver",
        "version": "7.7.4",
        "main": "index.js",
        "bin": { "semver": "bin/semver.js" },
        "scripts": {
            "test": "tap",
            "lint": "eslint \"**/*.{js,cjs,ts,mjs,jsx}\"",
            "postlint": "npm run test -- --ignore-scripts",
            "posttest": "npm run lint -- --ignore-scripts"
        }
    });
    let package_json_bytes = serde_json::to_vec_pretty(&package_json).unwrap();
    let tarball_bytes = build_package_tarball(&[
        ("package/package.json", package_json_bytes.as_slice()),
        ("package/index.js", b"module.exports = {};\n"),
        (
            "package/bin/semver.js",
            b"#!/usr/bin/env node\nconsole.log('semver');\n",
        ),
    ]);
    let tarball_integrity = format!("sha512-{}", format_sha512(&tarball_bytes));
    let (tarball_base, tarball_hits, tarball_stop, tarball_handle) =
        start_response_server(tarball_bytes, "application/octet-stream");

    let metadata = json!({
        "versions": {
            "7.7.4": {
                "dist": {
                    "tarball": format!("{}/semver-7.7.4.tgz", tarball_base),
                    "integrity": tarball_integrity
                }
            }
        }
    });
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_response_server(serde_json::to_vec(&metadata).unwrap(), "application/json");
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"dependencies":{"semver":"7.7.4"}}"#,
    )
    .unwrap();

    let summary = install_project(
        dir.path(),
        InstallOptions {
            allow_scripts: true,
            ..InstallOptions::default()
        },
    )
    .unwrap();

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    let lock_path = dir.path().join("kali.lock");
    assert_eq!(summary.lock_path.as_deref(), Some(lock_path.as_path()));
    assert_eq!(summary.installed, vec![package_key("semver", "7.7.4")]);

    let lock = load_lock(dir.path()).unwrap().unwrap();
    assert!(
        lock.packages.contains_key("semver@7.7.4"),
        "lock: {lock:#?}"
    );
    assert!(dir.path().join("node_modules/semver/package.json").exists());
    assert!(dir
        .path()
        .join(".kali-cache/packages/semver@7.7.4/package/package.json")
        .exists());

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().unwrap();
    registry_handle.join().unwrap();
    assert_eq!(tarball_hits.load(Ordering::SeqCst), 1);
    assert_eq!(registry_hits.load(Ordering::SeqCst), 1);
}


