use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn run_kali<I, S>(args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let dir = tempdir().expect("tempdir");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("run kali")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn start_registry_metadata_server(
    body: &'static str,
) -> (
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind registry metadata server");
    listener.set_nonblocking(true).expect("set nonblocking");
    let addr = listener.local_addr().expect("registry metadata address");
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
        format!("http://127.0.0.1:{}", addr.port()),
        hits,
        stop,
        handle,
    )
}

#[test]
fn package_audit_preview_short_circuits_before_malformed_target_validation_in_text_mode() {
    let output = run_kali(["package-audit", "--preview", "npm:lodash"]);

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "stderr: {stderr}"
    );
    assert!(
        !stderr.contains("npm:lodash"),
        "preview should short-circuit before malformed-target validation: {stderr}"
    );
}

#[test]
fn package_audit_preview_short_circuits_before_malformed_target_validation_in_json_mode() {
    let output = run_kali([
        "--output",
        "json",
        "package-audit",
        "--preview",
        "npm:lodash",
    ]);

    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
    assert!(json["stdout"].is_null());
    assert!(json["stderr"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(
        errors[0]["message"],
        "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"
    );
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_in_json_mode_with_valid_target() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_in_pretty_json_mode_with_valid_target(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert!(json["payload"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_in_pretty_json_mode_with_sandbox_and_valid_target(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("--pretty")
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--sandbox")
        .arg("policy.json")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert!(json["payload"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_in_json_mode_with_sandbox_and_valid_target(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--sandbox")
        .arg("policy.json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert!(json["payload"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--sandbox");
    assert_eq!(errors[0]["context"]["requestedValue"], "policy.json");
    assert_eq!(errors[0]["context"]["effectiveValue"], "policy.json");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("message string")
            .contains("package-analysis-specific flags"),
        "json: {json}"
    );
    assert_ne!(errors[0]["message"], "lodash");
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_in_json_mode_with_sandbox_and_package_analysis_flags(
) {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg("--sandbox")
        .arg("policy.json")
        .arg("--api")
        .arg("browser")
        .arg("--preview")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert!(json["payload"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
    assert_eq!(
        errors[0]["message"],
        "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"
    );
}

#[test]
fn package_audit_pretty_without_json_short_circuits_before_registry_lookup() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--pretty")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5508"), "stderr: {stderr}");
    assert!(
        stderr.contains("`--pretty` is only meaningful when JSON output is active"),
        "stderr: {stderr}"
    );
    assert!(
        !stderr.contains("lodash"),
        "pretty should short-circuit before target validation: {stderr}"
    );
}

#[test]
fn package_audit_preview_short_circuits_before_malformed_target_validation_with_package_analysis_flags_in_text_mode(
) {
    for args in [
        &["--api", "browser"][..],
        &["--compat", "eval"][..],
        &["--wasm-threads"][..],
    ] {
        let output = Command::new(kali_bin())
            .arg("package-audit")
            .args(args)
            .arg("--preview")
            .arg("lodash")
            .output()
            .expect("run kali");

        assert_eq!(output.status.code(), Some(5));
        assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(
                "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape",
            ),
            "stderr: {stderr}"
        );
        assert!(
            !stderr.contains("does not accept package-analysis-specific flags"),
            "preview should short-circuit before package-analysis flag validation: {stderr}"
        );
        assert!(
            !stderr.contains("lodash"),
            "preview should short-circuit before malformed-target validation: {stderr}"
        );
    }
}

#[test]
fn package_audit_preview_short_circuits_before_registry_lookup_with_package_analysis_flags_in_pretty_json_mode(
) {
    for args in [
        &["--api", "browser"][..],
        &["--compat", "eval"][..],
        &["--wasm-threads"][..],
    ] {
        let (registry_url, hits, stop, handle) =
            start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

        let output = Command::new(kali_bin())
            .env("KALI_REGISTRY", registry_url)
            .arg("package-audit")
            .args(args)
            .arg("--pretty")
            .arg("--output")
            .arg("json")
            .arg("--preview")
            .arg("lodash")
            .output()
            .expect("run kali");

        stop.store(true, Ordering::SeqCst);
        handle.join().expect("join registry server");

        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "registry should not be queried"
        );
        assert_eq!(output.status.code(), Some(5));
        assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "package-audit");
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 5);
        assert!(json["payload"].is_null());
        let errors = json["errors"].as_array().expect("errors array");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["code"], "E5508");
        assert_eq!(errors[0]["context"]["origin"], "cli");
        assert_eq!(errors[0]["context"]["flag"], "--preview");
        assert_eq!(errors[0]["context"]["requestedValue"], "true");
        assert_eq!(errors[0]["context"]["effectiveValue"], "true");
        assert_eq!(errors[0]["message"], "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape");
        assert!(
            !errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("package-analysis-specific flags"),
            "json: {json}"
        );
        assert!(
            !errors[0]["message"]
                .as_str()
                .expect("message string")
                .contains("lodash"),
            "preview should short-circuit before malformed-target validation: {json}"
        );
    }
}
