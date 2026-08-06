use super::*;

// Honest re-pin (PR #16 rev2): kali fails closed/loud here;
// see docs/superpowers/followups/pr16-honest-repin-inventory.md.
//
// These are local, run-module-only variants of the shared
// assert_browser_requested_reflect_own_keys /
// assert_json_browser_requested_reflect_own_keys /
// assert_inherited_browser_api_surface_reflect_own_keys helpers defined in
// the parent `browser_reflect_own_keys.rs`. Those shared helpers still have
// green callers in `test.rs` (out of this batch), so per the shared-helper
// red-only rule they are left untouched; the command shape is copied here
// instead and the assertion is narrowed to the honest fail-closed result.

fn assert_browser_requested_reflect_own_keys_fails_closed(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "must fail closed: {output:?}");
}

fn assert_json_browser_requested_reflect_own_keys_fails_closed(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "must fail closed: {output:?}");
}

fn assert_inherited_browser_api_surface_reflect_own_keys_fails_closed(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys_fails_closed("run", "main.js");
}

#[test]
fn run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys_fails_closed("run", "main.ts");
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys_fails_closed("run", "main.jsx");
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys_fails_closed("run", "main.tsx");
}

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.js", false);
}

#[test]
fn run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.ts", false);
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.jsx", false);
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.tsx", false);
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.js", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.ts", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.jsx", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys_fails_closed("run", "main.tsx", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys_fails_closed("run", "main.js");
}

#[test]
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys_fails_closed("run", "main.ts");
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys_fails_closed("run", "main.jsx");
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys_fails_closed("run", "main.tsx");
}
