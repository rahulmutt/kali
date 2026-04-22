use super::*;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

use serde_json::json;

#[test]
fn manifest_round_trip_is_deterministic() {
    let manifest = ProjectManifest {
        schema_version: MANIFEST_SCHEMA,
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
        version: LOCK_VERSION,
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

#[cfg(unix)]
fn append_marker_command(marker: &std::path::Path, label: &str) -> String {
    format!("printf '%s\\n' '{}' >> '{}'", label, marker.display())
}

#[cfg(windows)]
fn append_marker_command(marker: &std::path::Path, label: &str) -> String {
    format!("echo {}>>\"{}\"", label, marker.display())
}

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
fn requested_version_ranges_select_highest_matching_release() {
    let mut versions = serde_json::Map::new();
    versions.insert("1.0.0".to_string(), serde_json::Value::Null);
    versions.insert("1.2.0".to_string(), serde_json::Value::Null);
    versions.insert("2.0.0".to_string(), serde_json::Value::Null);

    assert_eq!(
        select_registry_version("lodash", &versions, Some("^1.0.0")).unwrap(),
        "1.2.0"
    );
    assert_eq!(
        select_registry_version("lodash", &versions, None).unwrap(),
        "2.0.0"
    );
}

#[test]
fn registry_metadata_is_cached_within_a_process() {
    let metadata = r#"{
  "versions": {
    "1.0.0": {
      "dist": {
        "tarball": "https://example.com/lodash-1.0.0.tgz",
        "integrity": "sha512-demo"
      }
    },
    "1.2.0": {
      "dist": {
        "tarball": "https://example.com/lodash-1.2.0.tgz",
        "integrity": "sha512-demo"
      }
    }
  }
}"#;
    let (metadata_url, hits, stop, handle) = start_metadata_server(metadata);

    let resolved_first =
        resolve_npm_like_package("npm", "lodash", "lodash", &metadata_url, Some("^1.0.0")).unwrap();
    let resolved_second =
        resolve_npm_like_package("npm", "lodash", "lodash", &metadata_url, Some("^1.0.0")).unwrap();

    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();

    assert_eq!(resolved_first.version, "1.2.0");
    assert_eq!(resolved_second.version, "1.2.0");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
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
fn ensure_project_ready_rejects_stale_lock_entries() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();
    let lock = LockFile {
        version: LOCK_VERSION,
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

fn start_raw_url_server(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/typescript\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    format!("http://127.0.0.1:{}/mod.ts", addr.port())
}

fn start_response_server(
    body: Vec<u8>,
    content_type: &'static str,
) -> (
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let hits_thread = hits.clone();
    let stop_thread = stop.clone();
    let handle = thread::spawn(move || loop {
        if stop_thread.load(Ordering::SeqCst) {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                hits_thread.fetch_add(1, Ordering::SeqCst);
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
                    body.len(),
                    content_type
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    });
    (
        format!("http://127.0.0.1:{}", addr.port()),
        hits,
        stop,
        handle,
    )
}

fn build_package_tarball(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);

    for (path, contents) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_cksum();
        builder.append(&header, *contents).unwrap();
    }

    let encoder = builder.into_inner().unwrap();
    encoder.finish().unwrap()
}

fn kali_registry_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn start_metadata_server(
    body: &'static str,
) -> (
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let hits_thread = hits.clone();
    let stop_thread = stop.clone();
    let handle = thread::spawn(move || loop {
        if stop_thread.load(Ordering::SeqCst) {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                hits_thread.fetch_add(1, Ordering::SeqCst);
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    });
    (
        format!("http://127.0.0.1:{}/package", addr.port()),
        hits,
        stop,
        handle,
    )
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
fn install_rejects_allow_scripts_without_npm_work() {
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
        .contains("requires effective npm-scriptable install work"));
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
fn validate_package_shape_rejects_node_gyp_install_time_scripts() {
    let package = PackageJson {
        scripts: BTreeMap::from([("install".to_string(), "node-gyp rebuild".to_string())]),
        ..PackageJson::default()
    };

    let error = validate_package_shape(&package, true).unwrap_err();
    assert_eq!(error[0].code, Some(e6::INCOMPATIBLE_PACKAGE as u32));
    assert!(error[0]
        .message
        .contains("node-gyp lifecycle script and falls outside the pure JS/TS package contract"));
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
