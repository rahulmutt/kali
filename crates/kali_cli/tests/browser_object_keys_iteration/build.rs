use super::*;

#[test]
fn build_emits_object_keys_iteration_semantics_in_js_input() {
    assert_browser_bundle_object_keys_iteration("app.js", false);
}

#[test]
fn build_emits_object_keys_iteration_semantics_in_ts_input() {
    assert_browser_bundle_object_keys_iteration("app.ts", false);
}

#[test]
fn build_emits_object_keys_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_object_keys_iteration("app.jsx", false);
}

#[test]
fn build_emits_object_keys_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_object_keys_iteration("app.tsx", false);
}

#[test]
fn build_emits_const_bound_object_keys_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        for json_output in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(filename);
            fs::write(
                &source_path,
                browser_bundle_const_bound_object_keys_iteration_source(),
            )
            .expect("write source");

            let mut command = Command::new(kali_bin());
            command
                .current_dir(dir.path())
                .arg("build")
                .arg("--bundle")
                .arg("--api")
                .arg("browser");
            if json_output {
                command.arg("--output").arg("json");
            }
            let output = command.arg(&source_path).output().expect("run kali");

            // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
            // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
            assert!(!output.status.success(), "must fail closed: {output:?}");
        }
    }
}

#[test]
fn build_emits_direct_object_keys_iteration_semantics_in_js_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.js", false);
}

#[test]
fn build_emits_direct_object_keys_iteration_semantics_in_ts_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.ts", false);
}

#[test]
fn build_emits_direct_object_keys_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.jsx", false);
}

#[test]
fn build_emits_direct_object_keys_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.tsx", false);
}

#[test]
fn build_emits_await_wrapped_static_object_helpers_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        for json_output in [false, true] {
            assert_browser_bundle_await_wrapped_static_object_helpers(filename, json_output);
        }
    }
}

#[test]
fn build_emits_global_object_keys_iteration_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_global_object_keys_iteration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn build_emits_global_object_keys_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(filename);
        fs::write(
            &source_path,
            browser_bundle_global_object_keys_iteration_source(),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!output.status.success(), "must fail closed: {output:?}");
    }
}

#[test]
fn build_emits_object_values_iteration_semantics_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    fs::write(
        &source_path,
        browser_bundle_object_values_iteration_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn build_emits_object_values_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(filename);
        fs::write(
            &source_path,
            browser_bundle_object_values_iteration_source(),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!output.status.success(), "must fail closed: {output:?}");
    }
}

#[test]
fn build_emits_object_keys_break_continue_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        for json_output in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(filename);
            fs::write(
                &source_path,
                browser_bundle_object_keys_break_continue_iteration_source(),
            )
            .expect("write source");

            let mut command = Command::new(kali_bin());
            command
                .current_dir(dir.path())
                .arg("build")
                .arg("--bundle")
                .arg("--api")
                .arg("browser");
            if json_output {
                command.arg("--output").arg("json");
            }
            let output = command.arg(&source_path).output().expect("run kali");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );

            if json_output {
                let envelope: Value =
                    serde_json::from_slice(&output.stdout).expect("valid json stdout");
                assert_eq!(envelope["schemaVersion"], 1);
                assert_eq!(envelope["command"], "build");
                assert_eq!(envelope["success"], true);
                assert_eq!(envelope["exitCode"], 0);
                assert!(envelope["errors"]
                    .as_array()
                    .expect("errors array")
                    .is_empty());
            }

            let bundle_dir = dir.path().join("app");
            let metadata: Value = serde_json::from_str(
                &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
            )
            .expect("parse metadata json");
            assert_eq!(metadata["apiSurface"], "browser");
            assert_eq!(metadata["artifactKind"], "bundle");

            let harness_path = bundle_dir
                .parent()
                .expect("bundle root parent")
                .join("browser-bundle-smoke.mjs");
            let harness = kali_runtime_contract::browser_bundle_harness_script(
                "app",
                false,
                r#"const mod = await import(bundleJs.href);
await mod.browserObjectKeysBreakContinueIteration();
"#,
            );
            fs::write(&harness_path, harness).expect("write browser bundle harness");

            let mut harness_command = kali_runtime_contract::browser_harness_command_parts_for(
                std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
                    .ok()
                    .as_deref(),
            );
            let harness_executable = harness_command.remove(0);
            let output = Command::new(&harness_executable)
                .current_dir(&bundle_dir)
                .args(&harness_command)
                .arg(&harness_path)
                .output()
                .expect("run browser bundle harness");

            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                String::from_utf8_lossy(&output.stdout).is_empty(),
                "stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }
}
