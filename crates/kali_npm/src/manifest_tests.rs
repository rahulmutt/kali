use crate::*;
use crate::test_support::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn manifest_round_trip_is_deterministic() {
    let manifest = ProjectManifest {
        schema_version: crate::MANIFEST_SCHEMA,
        dependencies: BTreeMap::from([("lodash".to_string(), "4.17.21".to_string())]),
        ..ProjectManifest::default()
    };

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let parsed: ProjectManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.dependencies.get("lodash").unwrap(), "4.17.21");
}

#[test]
fn lock_round_trip_is_deterministic() {
    let lock = LockFile {
        version: crate::LOCK_VERSION,
        packages: BTreeMap::from([(
            "lodash@4.17.21".to_string(),
            LockedPackage {
                registry: "npm".to_string(),
                integrity: "sha512-demo".to_string(),
                resolved: "https://example.com/lodash.tgz".to_string(),
                dependencies: BTreeMap::new(),
            },
        )]),
        ..LockFile::default()
    };

    let json = serde_json::to_string_pretty(&lock).unwrap();
    let parsed: LockFile = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.packages.len(), 1);
}

#[test]
fn manifest_registry_collisions_are_rejected_before_install() {
    let manifest = ProjectManifest {
        dependencies: BTreeMap::from([("@scope/name".to_string(), "1.0.0".to_string())]),
        dev_dependencies: BTreeMap::from([("jsr:@scope/name".to_string(), "1.0.0".to_string())]),
        ..ProjectManifest::default()
    };

    let error = validate_manifest_registry_collisions(&manifest).unwrap_err();
    assert_eq!(error.len(), 1);
    let diagnostic = &error[0];
    assert_eq!(diagnostic.code, Some(e6::VERSION_MISMATCH as u32));
    assert!(diagnostic
        .message
        .contains("would both materialize to node_modules/@scope/name"));
}

#[test]
fn manifest_registry_collisions_allow_identical_identity_spelling() {
    let manifest = ProjectManifest {
        dependencies: BTreeMap::from([("lodash".to_string(), "1.0.0".to_string())]),
        dev_dependencies: BTreeMap::new(),
        ..ProjectManifest::default()
    };

    validate_manifest_registry_collisions(&manifest).expect("single dependency should be valid");
}

#[test]
fn ensure_project_ready_rejects_stale_lock_entries() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();
    let lock = LockFile {
        version: crate::LOCK_VERSION,
        packages: BTreeMap::from([(
            "lodash@4.17.21".to_string(),
            LockedPackage {
                registry: "npm".to_string(),
                integrity: "sha512-demo".to_string(),
                resolved: "https://example.com/lodash.tgz".to_string(),
                dependencies: BTreeMap::new(),
            },
        )]),
        ..LockFile::default()
    };
    fs::write(
        dir.path().join("kali.lock"),
        serde_json::to_string_pretty(&lock).unwrap(),
    )
    .unwrap();

    let error = ensure_project_ready(dir.path()).unwrap_err();
    assert_eq!(error.code, Some(e6::INSTALL_REQUIRED as u32));
}

#[test]
fn ensure_project_ready_rejects_missing_raw_url_cache() {
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
    let lock = load_lock(dir.path()).unwrap().unwrap();
    let cached = Path::new(&lock.raw_urls.get(&raw_url).unwrap().cached).to_path_buf();
    fs::remove_file(&cached).unwrap();

    let error = ensure_project_ready(dir.path()).unwrap_err();
    assert_eq!(error.code, Some(e6::INSTALL_REQUIRED as u32));
}
