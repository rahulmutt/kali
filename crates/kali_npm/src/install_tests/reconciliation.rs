use super::*;

#[test]
fn install_reconciles_raw_urls_from_source_import_map_rewrites() {
    let dir = kali_test_support::fixtures::tempdir();
    let raw_url = start_raw_url_server("export default 1;");
    let raw_prefix = raw_url.trim_end_matches("mod.ts").to_string();
    kali_test_support::fixtures::write_manifest(
        dir.path(),
        &format!(
            r#"{{
  "schemaVersion": 1,
  "imports": {{
    "raw/": "{}"
  }}
}}"#,
            raw_prefix
        ),
    );
    kali_test_support::fixtures::write_file(dir.path(), "main.ts", "import 'raw/mod.ts';\n");

    let summary = install_project(dir.path(), InstallOptions::default()).unwrap();
    assert!(summary.lock_path.is_some());

    let lock = load_lock(dir.path()).unwrap().unwrap();
    assert!(lock.raw_urls.contains_key(&raw_url), "lock: {lock:#?}");
    let cached = Path::new(&lock.raw_urls.get(&raw_url).unwrap().cached).to_path_buf();
    assert!(cached.exists(), "cached raw url was not materialized");

    kali_test_support::fixtures::write_file(dir.path(), "main.ts", "export {};\n");
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
    let dir = kali_test_support::fixtures::tempdir();
    let raw_url = start_raw_url_server("export default 1;");
    let raw_prefix = raw_url.trim_end_matches("mod.ts").to_string();
    kali_test_support::fixtures::write_manifest(
        dir.path(),
        &format!(
            r#"{{
  "schemaVersion": 1,
  "imports": {{
    "raw/": "{}"
  }}
}}"#,
            raw_prefix
        ),
    );
    kali_test_support::fixtures::write_file(dir.path(), "main.ts", "import 'raw/mod.ts';\n");

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
fn install_reconciles_semver_style_package_without_allow_scripts() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = kali_test_support::fixtures::tempdir();

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

    kali_test_support::fixtures::write_manifest(
        dir.path(),
        r#"{"schemaVersion":1,"dependencies":{"semver":"7.7.4"}}"#,
    );

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
    let dir = kali_test_support::fixtures::tempdir();

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

    kali_test_support::fixtures::write_manifest(
        dir.path(),
        r#"{"schemaVersion":1,"dependencies":{"semver":"7.7.4"}}"#,
    );

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
