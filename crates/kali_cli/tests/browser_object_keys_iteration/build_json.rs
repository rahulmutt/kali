use super::*;

#[test]
fn json_build_emits_object_keys_iteration_semantics_in_js_input() {
    assert_browser_bundle_object_keys_iteration("app.js", true);
}

#[test]
fn json_build_emits_object_keys_iteration_semantics_in_ts_input() {
    assert_browser_bundle_object_keys_iteration("app.ts", true);
}

#[test]
fn json_build_emits_object_keys_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_object_keys_iteration("app.jsx", true);
}

#[test]
fn json_build_emits_object_keys_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_object_keys_iteration("app.tsx", true);
}

#[test]
fn json_build_emits_direct_object_keys_iteration_semantics_in_js_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.js", true);
}

#[test]
fn json_build_emits_direct_object_keys_iteration_semantics_in_ts_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.ts", true);
}

#[test]
fn json_build_emits_direct_object_keys_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.jsx", true);
}

#[test]
fn json_build_emits_direct_object_keys_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_direct_object_keys_iteration("app.tsx", true);
}

#[test]
fn json_build_emits_global_object_keys_iteration_semantics_in_js_input() {
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
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_build_emits_global_object_keys_iteration_semantics_in_ts_jsx_tsx_input() {
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
            .arg("--output")
            .arg("json")
            .arg(&source_path)
            .output()
            .expect("run kali");

        // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!output.status.success(), "must fail closed: {output:?}");
    }
}
