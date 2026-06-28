use super::*;

#[test]
fn json_install_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!([]));
    assert_eq!(json["payload"]["updated"], json!([]));
    assert_eq!(json["payload"]["removed"], json!([]));
}

#[test]
fn install_rejects_registry_path_collisions_before_materialization() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "dependencies": {
    "@scope/name": "1.0.0"
  },
  "devDependencies": {
    "jsr:@scope/name": "1.0.0"
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E6002"), "stderr: {stderr}");
    assert!(
        stderr.contains("would both materialize to node_modules/@scope/name"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_rejects_api_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--api")
        .arg("browser")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_rejects_sandbox_flag_with_usage_code() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--sandbox")
        .arg("kali.policy.json")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("does not accept `--api` or `--sandbox`"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_prunes_stale_registry_layout_without_repairing() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    fs::write(
        dir.path().join("kali.lock"),
        r#"{
  "version": 1,
  "packages": {
    "lodash@4.17.21": {
      "registry": "npm",
      "integrity": "sha512-demo",
      "resolved": "https://example.com/lodash.tgz",
      "dependencies": {}
    }
  }
}"#,
    )
    .expect("write lock");
    fs::create_dir_all(dir.path().join("node_modules/lodash")).expect("node_modules layout");
    fs::create_dir_all(dir.path().join(".kali-cache/packages/lodash@4.17.21"))
        .expect("package cache");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "stale lock file should be removed"
    );
    assert!(
        !dir.path().join("node_modules/lodash").exists(),
        "stale install path should be pruned"
    );
    assert!(
        !dir.path()
            .join(".kali-cache/packages/lodash@4.17.21")
            .exists(),
        "stale package cache should be pruned"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Installed 0 package(s)"),
        "stdout: {stdout}"
    );
}

#[test]
fn install_prunes_stale_registry_layout_and_reports_removed_entries_in_json() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    fs::write(
        dir.path().join("kali.lock"),
        r#"{
  "version": 1,
  "packages": {
    "lodash@4.17.21": {
      "registry": "npm",
      "integrity": "sha512-demo",
      "resolved": "https://example.com/lodash.tgz",
      "dependencies": {}
    }
  }
}"#,
    )
    .expect("write lock");
    fs::create_dir_all(dir.path().join("node_modules/lodash")).expect("node_modules layout");
    fs::create_dir_all(dir.path().join(".kali-cache/packages/lodash@4.17.21"))
        .expect("package cache");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!([]));
    assert_eq!(json["payload"]["updated"], json!([]));
    assert_eq!(json["payload"]["removed"], json!(["lodash@4.17.21"]));
    assert!(json["payload"]["manifestPath"].is_null());
    assert!(json["payload"]["lockPath"].is_null());
    assert!(
        !dir.path().join("kali.lock").exists(),
        "stale lock file should be removed"
    );
    assert!(
        !dir.path().join("node_modules/lodash").exists(),
        "stale install path should be pruned"
    );
    assert!(
        !dir.path()
            .join(".kali-cache/packages/lodash@4.17.21")
            .exists(),
        "stale package cache should be pruned"
    );
}

#[test]
fn install_noops_without_manifest_or_dependencies_on_the_cli() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!([]));
    assert_eq!(json["payload"]["removed"], json!([]));
    assert_eq!(json["payload"]["updated"], json!([]));
    assert!(json["payload"]["manifestPath"].is_null());
    assert!(json["payload"]["lockPath"].is_null());
    assert!(
        !dir.path().join("kali.json").exists(),
        "install should not scaffold a placeholder manifest"
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "install should not materialize a lockfile on an empty workspace"
    );
}

#[test]
fn install_noops_without_manifest_or_dependencies_are_deterministic_across_repeated_json_invocations(
) {
    let dir = tempdir().expect("tempdir");

    let run = || {
        Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("install")
            .output()
            .expect("run kali")
    };

    let first = run();
    assert!(
        first.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let first_json = parse_json_stdout(&first);

    let second = run();
    assert!(
        second.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let second_json = parse_json_stdout(&second);

    assert_eq!(
        first.stdout, second.stdout,
        "stdout should be deterministic across repeated install no-op invocations"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "stderr should be deterministic across repeated install no-op invocations"
    );
    assert_eq!(
        first_json, second_json,
        "JSON output should be deterministic across repeated install no-op invocations"
    );
    assert_eq!(first_json["command"], "install");
    assert_eq!(first_json["success"], true);
    assert_eq!(first_json["exitCode"], 0);
    assert_eq!(first_json["payload"]["installed"], json!([]));
    assert_eq!(first_json["payload"]["removed"], json!([]));
    assert_eq!(first_json["payload"]["updated"], json!([]));
    assert!(first_json["payload"]["manifestPath"].is_null());
    assert!(first_json["payload"]["lockPath"].is_null());
    assert!(
        !dir.path().join("kali.json").exists(),
        "install should not scaffold a placeholder manifest"
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "install should not materialize a lockfile on an empty workspace"
    );
}

#[test]
fn install_allow_scripts_rejects_jsr_targets() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--allow-scripts")
        .arg("jsr:@std/path")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("JSR targets"), "stderr: {stderr}");
}

#[test]
fn install_allow_scripts_rejects_jsr_targets_in_json() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .arg("jsr:@std/path")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5508");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("JSR targets"),
        "json: {json}"
    );
}

#[test]
fn install_allow_scripts_rejects_bootstrap_heavy_registry_targets_on_the_cli() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().expect("tempdir");

    let package_json = json!({
        "name": "bootstrap-heavy",
        "version": "1.0.0",
        "main": "index.js",
        "scripts": {
            "install": "node-gyp rebuild"
        }
    });
    let package_json_bytes =
        serde_json::to_vec_pretty(&package_json).expect("serialize package json");
    let tarball_bytes = build_package_tarball(&[
        ("package/package.json", package_json_bytes.as_slice()),
        ("package/index.js", b"module.exports = {};\n"),
    ]);
    let tarball_integrity = format!("sha512-{}", format_sha512(&tarball_bytes));
    let (tarball_base, tarball_hits, tarball_stop, tarball_handle) =
        start_binary_response_server(tarball_bytes, "application/octet-stream");
    let metadata = json!({
        "versions": {
            "1.0.0": {
                "dist": {
                    "tarball": format!("{}/bootstrap-heavy-1.0.0.tgz", tarball_base),
                    "integrity": tarball_integrity
                }
            }
        }
    });
    let metadata = Box::leak(metadata.to_string().into_boxed_str());
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_registry_metadata_server(metadata);
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .arg("bootstrap-heavy")
        .output()
        .expect("run kali");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().expect("join tarball server");
    registry_handle.join().expect("join registry server");

    assert!(
        tarball_hits.load(Ordering::SeqCst) > 0,
        "tarball server should be queried"
    );
    assert!(
        registry_hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(!output.status.success());
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 1);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E6005");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("native or binary bootstrap lifecycle script"),
        "json: {json}"
    );
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("falls outside the pure JS/TS package contract"),
        "json: {json}"
    );
}

#[test]
fn install_rejects_versioned_registry_targets() {
    let _guard = kali_registry_lock().lock().unwrap();
    let (registry_base, hits, stop, handle) = start_registry_metadata_server(
        r#"{"versions":{"1.2.3":{"dist":{"tarball":"https://example.com/lodash-1.2.3.tgz"}}}}"#,
    );
    let dir = tempdir().expect("tempdir");
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("lodash@1.2.3")
        .output()
        .expect("run kali");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "versioned install target should not hit the registry"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(stderr.contains("explicit versions"), "stderr: {stderr}");
}

#[test]
fn install_allow_scripts_rejects_when_no_npm_work_exists() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--allow-scripts")
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-empty npm install work"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_allow_scripts_rejects_when_no_npm_work_exists_in_json_on_a_clean_workspace() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5508");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("non-empty npm install work"),
        "json: {json}"
    );
    assert!(
        !dir.path().join("kali.json").exists(),
        "install should not scaffold a placeholder manifest on a rejected no-op"
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "install should not materialize a lockfile on a rejected no-op"
    );
}

#[test]
fn install_allow_scripts_rejects_when_only_raw_url_install_work_exists() {
    let dir = tempdir().expect("tempdir");
    let (raw_url_base, hits, stop, handle) =
        start_binary_response_server(b"export default 1;".to_vec(), "application/typescript");
    let raw_url = format!("{raw_url_base}/mod.ts");
    fs::write(
        dir.path().join("main.ts"),
        format!("import \"{raw_url}\";\n"),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--allow-scripts")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join raw-url server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "raw URL should not be fetched when npm install work is absent"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-empty npm install work"),
        "stderr: {stderr}"
    );
    assert!(
        !dir.path().join("kali.json").exists(),
        "install should not scaffold a placeholder manifest on a rejected no-op"
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "install should not materialize a lockfile on a rejected no-op"
    );
}

#[test]
fn install_allow_scripts_rejects_when_only_raw_url_install_work_exists_in_json() {
    let dir = tempdir().expect("tempdir");
    let (raw_url_base, hits, stop, handle) =
        start_binary_response_server(b"export default 1;".to_vec(), "application/typescript");
    let raw_url = format!("{raw_url_base}/mod.ts");
    fs::write(
        dir.path().join("main.ts"),
        format!("import \"{raw_url}\";\n"),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join raw-url server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "raw URL should not be fetched when npm install work is absent"
    );
    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5508");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("non-empty npm install work"),
        "json: {json}"
    );
    assert!(
        !dir.path().join("kali.json").exists(),
        "install should not scaffold a placeholder manifest on a rejected no-op"
    );
    assert!(
        !dir.path().join("kali.lock").exists(),
        "install should not materialize a lockfile on a rejected no-op"
    );
}

#[test]
fn install_reconciles_semver_style_package_without_allow_scripts_on_the_cli() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"dependencies":{"semver":"7.7.4"}}"#,
    )
    .expect("write manifest");

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
    let package_json_bytes =
        serde_json::to_vec_pretty(&package_json).expect("serialize package json");
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
        start_binary_response_server(tarball_bytes, "application/octet-stream");
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
    let metadata = Box::leak(metadata.to_string().into_boxed_str());
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_registry_metadata_server(metadata);
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .output()
        .expect("run kali");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().expect("join tarball server");
    registry_handle.join().expect("join registry server");

    assert!(
        tarball_hits.load(Ordering::SeqCst) > 0,
        "tarball server should be queried"
    );
    assert!(
        registry_hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!(["semver@7.7.4"]));
    assert!(
        dir.path().join("node_modules/semver/package.json").exists(),
        "semver package should be materialized"
    );
}

#[test]
fn install_allow_scripts_accepts_registry_targets_with_empty_lifecycle_scripts_on_the_cli() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().expect("tempdir");

    let package_json = json!({
        "name": "semver",
        "version": "7.7.4",
        "main": "index.js",
        "bin": { "semver": "bin/semver.js" },
        "scripts": {}
    });
    let package_json_bytes =
        serde_json::to_vec_pretty(&package_json).expect("serialize package json");
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
        start_binary_response_server(tarball_bytes, "application/octet-stream");
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
    let metadata = Box::leak(metadata.to_string().into_boxed_str());
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_registry_metadata_server(metadata);
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .arg("semver")
        .output()
        .expect("run kali");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().expect("join tarball server");
    registry_handle.join().expect("join registry server");

    assert!(
        tarball_hits.load(Ordering::SeqCst) > 0,
        "tarball server should be queried"
    );
    assert!(
        registry_hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!(["semver@7.7.4"]));

    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("kali.json")).expect("read manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["dependencies"]["semver"], "7.7.4");
    assert!(
        manifest["devDependencies"].get("semver").is_none(),
        "semver should be recorded only in dependencies"
    );
    assert!(
        dir.path().join("kali.lock").exists(),
        "install should materialize a lockfile"
    );
    assert!(
        dir.path().join("node_modules/semver/package.json").exists(),
        "semver package should be materialized"
    );
}

#[test]
fn install_records_registry_targets_in_dev_dependencies_on_a_configless_project() {
    let _guard = kali_registry_lock().lock().unwrap();
    let dir = tempdir().expect("tempdir");

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
    let package_json_bytes =
        serde_json::to_vec_pretty(&package_json).expect("serialize package json");
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
        start_binary_response_server(tarball_bytes, "application/octet-stream");
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
    let metadata = Box::leak(metadata.to_string().into_boxed_str());
    let (registry_base, registry_hits, registry_stop, registry_handle) =
        start_registry_metadata_server(metadata);
    let previous_registry = std::env::var_os("KALI_REGISTRY");
    std::env::set_var("KALI_REGISTRY", &registry_base);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--dev")
        .arg("semver")
        .output()
        .expect("run kali");

    if let Some(previous_registry) = previous_registry {
        std::env::set_var("KALI_REGISTRY", previous_registry);
    } else {
        std::env::remove_var("KALI_REGISTRY");
    }

    tarball_stop.store(true, Ordering::SeqCst);
    registry_stop.store(true, Ordering::SeqCst);
    tarball_handle.join().expect("join tarball server");
    registry_handle.join().expect("join registry server");

    assert!(
        tarball_hits.load(Ordering::SeqCst) > 0,
        "tarball server should be queried"
    );
    assert!(
        registry_hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!(["semver@7.7.4"]));

    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("kali.json")).expect("read manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["devDependencies"]["semver"], "7.7.4");
    assert!(
        manifest["dependencies"].get("semver").is_none(),
        "semver should be recorded only in devDependencies"
    );
    assert!(
        dir.path().join("kali.lock").exists(),
        "install should materialize a lockfile"
    );
    assert!(
        dir.path().join("node_modules/semver/package.json").exists(),
        "semver package should be materialized"
    );
}

#[test]
fn install_dev_requires_an_explicit_registry_target() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--dev")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("explicit registry package target"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_materializes_raw_url_targets_without_scaffolding_a_placeholder_manifest() {
    let dir = tempdir().expect("tempdir");
    let (raw_url_base, hits, stop, handle) =
        start_binary_response_server(b"export default 1;".to_vec(), "application/typescript");
    let raw_url = format!("{raw_url_base}/mod.ts");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg(&raw_url)
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join raw-url server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "raw URL should be fetched during install"
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["installed"], json!([raw_url]));
    assert!(
        json["payload"]["manifestPath"].is_null(),
        "install should not scaffold a placeholder manifest"
    );
    assert!(json["payload"]["lockPath"].is_string());

    assert!(
        !dir.path().join("kali.json").exists(),
        "raw URL install should not create a placeholder manifest"
    );
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("kali.lock")).expect("read lock"))
            .expect("parse lock");
    let cached = lock["rawUrls"]
        .get(&raw_url)
        .and_then(|entry| entry.get("cached"))
        .and_then(|cached| cached.as_str())
        .expect("raw URL cache entry");
    assert!(
        Path::new(cached).exists(),
        "cached raw URL was not materialized"
    );
}

#[test]
fn install_allow_scripts_rejects_raw_url_targets() {
    let dir = tempdir().expect("tempdir");
    let (raw_url_base, hits, stop, handle) =
        start_binary_response_server(b"export default 1;".to_vec(), "application/typescript");
    let raw_url = format!("{raw_url_base}/mod.ts");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("install")
        .arg("--allow-scripts")
        .arg(&raw_url)
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join raw-url server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "raw URL should be rejected before fetch"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("not valid for raw-URL targets"),
        "stderr: {stderr}"
    );
}

#[test]
fn install_allow_scripts_rejects_raw_url_targets_in_json() {
    let dir = tempdir().expect("tempdir");
    let (raw_url_base, hits, stop, handle) =
        start_binary_response_server(b"export default 1;".to_vec(), "application/typescript");
    let raw_url = format!("{raw_url_base}/mod.ts");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("install")
        .arg("--allow-scripts")
        .arg(&raw_url)
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join raw-url server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "raw URL should be rejected before fetch"
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "install");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5508");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("error message")
            .contains("not valid for raw-URL targets"),
        "json: {json}"
    );
}
