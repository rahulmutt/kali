use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_math_pow_optional_chain_run_source() -> &'static str {
    r#"const exponent = 3;
const alias = exponent;
console.log(globalThis?.Math.pow(2, alias));
console.log(globalThis?.Math["pow"](2, alias));
console.log(globalThis?.Math['pow'](2, alias));
console.log(globalThis?.["Math"].pow(2, alias));
console.log(globalThis?.["Math"]["pow"](2, alias));
console.log(globalThis?.["Math"]['pow'](2, alias));
console.log(globalThis?.['Math'].pow(2, alias));
console.log(globalThis?.['Math']["pow"](2, alias));
console.log(globalThis?.['Math']['pow'](2, alias));
console.log(Object.freeze(globalThis?.Math.pow)(2, alias));
console.log(Object.freeze((globalThis?.Math.pow))(2, alias));
console.log(Object.freeze(globalThis?.Math["pow"])(2, alias));
console.log(Object.freeze((globalThis?.Math["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.["Math"].pow))(2, alias));
console.log(Object.freeze((globalThis?.['Math'].pow))(2, alias));
console.log(Object.freeze((globalThis?.["Math"]["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.["Math"]['pow']))(2, alias));
console.log(Object.freeze((globalThis?.['Math']["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.['Math']['pow']))(2, alias));
"#
}

fn assert_browser_math_pow_optional_chain_rejection(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let mut cli = cli.arg(command);
    if command == "build" {
        cli = cli.arg("--bundle");
    }
    let output = cli
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .is_some_and(|message| message.contains("optional-chain wrappers"))),
            "unexpected errors: {errors:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("optional-chain wrappers"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn build_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_math_pow_optional_chain_rejection(
            "build",
            &format!("app.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            false,
        );
        assert_browser_math_pow_optional_chain_rejection(
            "build",
            &format!("app.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            true,
        );
    }
}

#[test]
fn check_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_math_pow_optional_chain_rejection(
            "check",
            &format!("main.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            false,
        );
        assert_browser_math_pow_optional_chain_rejection(
            "check",
            &format!("main.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            true,
        );
    }
}
