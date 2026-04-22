use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use serde_json::{json, Value};
use wasmparser::{Operator, Parser, Payload};

use kali_runtime::split_command_spec;
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    fixture_root().join(relative)
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_artifact_metadata_provenance(
    metadata: &Value,
    artifact_kind: &str,
    expected_max_specializations: usize,
) {
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], artifact_kind);
    assert_eq!(metadata["runtimeProfiles"], json!([]));
    assert_eq!(metadata["maxSpecializations"], expected_max_specializations);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
}

fn assert_browser_runtime_rejection_text(text: &str) {
    assert!(text.contains("browser API surface"), "stderr: {text}");
    assert!(
        text.contains("selected host contract: browser-requested"),
        "stderr: {text}"
    );
    assert!(
        text.contains("current runtime backend: wasmtime"),
        "stderr: {text}"
    );
    assert!(
        text.contains("browser runtime host description: real browser host"),
        "stderr: {text}"
    );
    assert!(
        text.contains("browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"),
        "stderr: {text}"
    );
    assert!(
        text.contains("browser runtime contract scope: run and test only"),
        "stderr: {text}"
    );
    assert!(
        text.contains("standalone browser runtime contract"),
        "stderr: {text}"
    );
}

fn assert_browser_runtime_rejection_message(message: &str) {
    assert!(
        message.contains("browser API surface"),
        "message: {message}"
    );
    assert!(
        message.contains("selected host contract: browser-requested"),
        "message: {message}"
    );
    assert!(
        message.contains("standalone browser runtime contract"),
        "message: {message}"
    );
}

fn assert_browser_runtime_rejection_notes(notes: &[Value]) {
    assert!(
        notes
            .iter()
            .any(|note| note.as_str() == Some("selected host contract: browser-requested")),
        "notes: {notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|note| note.as_str() == Some("current runtime backend: wasmtime")),
        "notes: {notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|note| note.as_str() == Some("supported browser runtime commands: run, test")),
        "notes: {notes:?}"
    );
    assert!(
        notes.iter().any(
            |note| note.as_str() == Some("browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work")
        ),
        "notes: {notes:?}"
    );
    assert!(
        notes.iter().any(|note| note.as_str() == Some("browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness")),
        "notes: {notes:?}"
    );
    assert!(
        notes.iter().any(
            |note| note.as_str() == Some("browser runtime host description: real browser host")
        ),
        "notes: {notes:?}"
    );
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

fn read_artifact_bytes(paths: &[PathBuf]) -> BTreeMap<PathBuf, Vec<u8>> {
    paths
        .iter()
        .cloned()
        .map(|path| {
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!("failed to read artifact '{}': {}", path.display(), error)
            });
            (path, bytes)
        })
        .collect()
}

fn assert_artifact_bytes_stable(paths: &[PathBuf], first: &BTreeMap<PathBuf, Vec<u8>>) {
    let second = read_artifact_bytes(paths);
    assert_eq!(first, &second, "artifact outputs differed between builds");
}

fn count_i64_adds(bytes: &[u8]) -> usize {
    let mut count = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        if let Ok(Payload::CodeSectionEntry(body)) = payload {
            let mut reader = body.get_operators_reader().expect("operators reader");
            while !reader.eof() {
                if reader.read().expect("read operator") == Operator::I64Add {
                    count += 1
                }
            }
        }
    }
    count
}

fn count_tag_boxing_ops(bytes: &[u8]) -> usize {
    let mut count = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        if let Ok(Payload::CodeSectionEntry(body)) = payload {
            let mut reader = body.get_operators_reader().expect("operators reader");
            while !reader.eof() {
                match reader.read().expect("read operator") {
                    Operator::I64And | Operator::I64Eq | Operator::I64ShrS => count += 1,
                    _ => {}
                }
            }
        }
    }
    count
}

fn count_wasm_instructions(bytes: &[u8]) -> usize {
    let mut count = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        if let Ok(Payload::CodeSectionEntry(body)) = payload {
            let mut reader = body.get_operators_reader().expect("operators reader");
            while !reader.eof() {
                reader.read().expect("read operator");
                count += 1;
            }
        }
    }
    count
}

fn browser_bundle_harness_command_parts_for(command: Option<&str>) -> Vec<String> {
    kali_runtime::browser_harness_command_parts_for(command)
}

fn browser_bundle_harness_command_parts() -> Vec<String> {
    kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    )
}

#[test]
fn browser_bundle_harness_command_override_supports_quoted_arguments() {
    let parts = split_command_spec(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    )
    .expect("split valid browser harness command");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );
}

#[test]
fn browser_bundle_harness_command_override_preserves_empty_quoted_arguments() {
    let parts = split_command_spec(r#"browser-wrapper "" --flag '' trailing"#)
        .expect("split browser harness command with empty quoted arguments");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "".to_string(),
            "--flag".to_string(),
            "".to_string(),
            "trailing".to_string(),
        ]
    );
}

#[test]
fn browser_bundle_harness_command_override_rejects_empty_executable_token() {
    assert_eq!(split_command_spec(r#"" --flag"#), None);
}

#[test]
fn browser_bundle_harness_command_override_rejects_unterminated_quotes() {
    assert_eq!(split_command_spec(r#"browser-wrapper "unterminated"#), None);
}

#[test]
fn browser_bundle_harness_command_override_rejects_malformed_environment_values() {
    assert!(std::panic::catch_unwind(|| {
        browser_bundle_harness_command_parts_for(Some(r#"" --flag"#))
    })
    .is_err());
    assert!(std::panic::catch_unwind(|| {
        browser_bundle_harness_command_parts_for(Some(r#"browser-wrapper "unterminated"#))
    })
    .is_err());
}

fn assert_browser_bundle_executes(bundle_root: &Path, export_name: &str) {
    let bundle_dir = bundle_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_root
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir,
        false,
        &format!(
            r#"const mod = await import(bundleJs.href);
const result = await mod.{export_name}(1n, 2n);
if (result !== 0n) {{
  throw new Error(`unexpected result ${{result}}`);
}}
console.log(String(result));
"#,
            export_name = export_name,
        ),
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(bundle_root)
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

fn assert_browser_bundle_dynamic_import_loader(bundle_root: &Path, specifier: &str) {
    let bundle_dir = bundle_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_root
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-dynamic-import-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir,
        true,
        &format!(
            r#"const mod = await import(bundleJs.href);
if (typeof mod.loadDynamicImport !== 'function') {{
  throw new Error('missing loadDynamicImport helper');
}}
const chunk = await mod.loadDynamicImport({specifier});
if (typeof chunk.lazyValue !== 'function') {{
  throw new Error('missing lazyValue export');
}}
const value = await chunk.lazyValue();
if (value !== 0n) {{
  throw new Error(`unexpected chunk result ${{value}}`);
}}
console.log(String(value));
"#,
            specifier = serde_json::to_string(specifier).expect("serialize specifier"),
        ),
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(bundle_root)
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

fn write_valid_policy(path: &Path) {
    fs::write(
        path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");
}

#[test]
fn check_accepts_a_resolved_file() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_uses_explicit_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_uses_inherited_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
console.log('ok');
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_rejects_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn check_rejects_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("wasm-threads"), "stderr: {stderr}");
}

#[test]
fn check_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn check_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn run_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn test_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(
        stderr.contains("globalThis.SharedArrayBuffer"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("globalThis.Atomics"), "stderr: {stderr}");
}

#[test]
fn check_rejects_late_host_control_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Deno.pid; globalThis.Deno.pid; globalThis.Deno.cwd; Deno.chdir('/tmp'); globalThis.Deno.chdir('/tmp'); globalThis.Deno.exit(0); process.pid; globalThis.process.pid; globalThis.process.cwd; process.chdir('/tmp'); globalThis.process.chdir('/tmp'); globalThis.process.exit(0);",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    for expected in [
        "Deno.pid",
        "globalThis.Deno.pid",
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "globalThis.process.exit",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_late_host_control_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Deno.pid; globalThis.Deno.pid; globalThis.Deno.cwd; Deno.chdir('/tmp'); globalThis.Deno.chdir('/tmp'); globalThis.Deno.exit(0); process.pid; globalThis.process.pid; globalThis.process.cwd; process.chdir('/tmp'); globalThis.process.chdir('/tmp'); globalThis.process.exit(0);",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 12);
    assert!(errors.iter().all(|error| error["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Deno.pid",
        "globalThis.Deno.pid",
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "globalThis.process.exit",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn check_rejects_broader_intl_support() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Intl; globalThis.Intl; globalThis.Intl.NumberFormat; Intl.NumberFormat;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("Intl"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Intl"), "stderr: {stderr}");
}

#[test]
fn check_rejects_broader_intl_support_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Intl; globalThis.Intl; globalThis.Intl.NumberFormat; Intl.NumberFormat;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty());
    assert_eq!(errors[0]["code"], "E5006");
    assert!(errors[0]["message"]
        .as_str()
        .expect("error message")
        .contains("Intl"));
    assert!(errors.iter().any(|error| error["message"]
        .as_str()
        .expect("error message")
        .contains("globalThis.Intl")));
}

#[test]
fn check_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("globalThis.Proxy"), "stderr: {stderr}");
    assert!(stderr.contains("WeakMap"), "stderr: {stderr}");
    assert!(stderr.contains("WeakSet"), "stderr: {stderr}");
    assert!(stderr.contains("FinalizationRegistry"), "stderr: {stderr}");
}

#[test]
fn check_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 8);
    assert!(errors.iter().all(|error| error["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn run_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn run_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 8);
    assert!(errors.iter().all(|error| error["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn test_rejects_late_object_model_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn test_rejects_late_object_model_globals_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 8);
    assert!(errors.iter().all(|error| error["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn check_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("check")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 3 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_reports_unresolved_identifiers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "missing;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
}

#[test]
fn run_executes_the_hello_fixture() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_the_explicit_deno_api_surface() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("deno")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_zero_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-threads")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
}

#[test]
fn run_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_accepts_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_and_test_accept_the_specialization_cap_override() {
    let dir = tempdir().expect("tempdir");
    let run_source = dir.path().join("main.ts");
    let test_source = dir.path().join("smoke.test.ts");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");
    fs::write(&run_source, "console.log('specialization-cap');").expect("write run source");
    fs::write(
        &test_source,
        "Kali.test('addition', () => {\n    1 + 2;\n});\n",
    )
    .expect("write test source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--max-specializations")
        .arg("4")
        .arg(&run_source)
        .output()
        .expect("run kali");

    assert!(
        run.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("specialization-cap"),
        "stdout: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--max-specializations")
        .arg("4")
        .arg(&test_source)
        .output()
        .expect("run kali");

    assert!(
        test.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
}

#[test]
fn run_rejects_declaration_only_fixture_entrypoints() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/decl.d.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5007"), "stderr: {stderr}");
}

#[test]
fn run_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn run_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--api");
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "config");
    assert_eq!(
        errors[0]["context"]["configPath"],
        "compilerOptions.apiSurface"
    );
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn run_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
}

#[test]
fn json_run_rejects_browser_api_surface_with_guest_args_in_phase_one() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("run/hello.ts"))
        .arg("--")
        .arg("guest-flag")
        .arg("guest-value")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_with_guest_args_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .arg("--")
        .arg("guest-flag")
        .arg("guest-value")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
}

#[test]
fn run_rejects_wasm_threads_runtime_profile() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--wasm-threads")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn run_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn run_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('threaded run');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
}

#[test]
fn test_reports_success_for_explicit_file_sets() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_the_explicit_deno_api_surface() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--api")
        .arg("deno")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
}

#[test]
fn test_accepts_zero_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_accepts_positive_spawned_process_budget_override() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--max-spawned-processes")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_rejects_wasm_threads_runtime_profile() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--wasm-threads")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn test_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn test_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
}

#[test]
fn test_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("test")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn fmt_check_reports_drift_without_rewriting() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function add(a,b){return a+b;}").expect("write source");

    let output = Command::new(kali_bin())
        .arg("fmt")
        .arg("--check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would format 1 file(s)"),
        "stdout: {stdout}"
    );
    let contents = fs::read_to_string(&source_path).expect("read source");
    assert_eq!(contents, "function add(a,b){return a+b;}");
}

#[test]
fn lint_fix_applies_structured_safe_fixes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "var x = 1; debugger; if (x == 1) { }").expect("write source");

    let output = Command::new(kali_bin())
        .arg("lint")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let contents = fs::read_to_string(&source_path).expect("read source");
    assert!(contents.contains("let x = 1;"));
    assert!(contents.contains("==="));
    assert!(!contents.contains("debugger"));
}

#[test]
fn test_filters_selected_files_before_execution() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(&keep, "1 + 2;").expect("write keep source");
    fs::write(&skip, "3 + 4;").expect("write skip source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn init_scaffolds_application_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("main.ts").exists());

    let manifest = fs::read_to_string(dir.path().join("kali.json")).expect("manifest");
    assert!(
        manifest.contains("\"schemaVersion\": 1"),
        "manifest: {manifest}"
    );
    let source = fs::read_to_string(dir.path().join("main.ts")).expect("source");
    assert!(source.contains("Hello, world!"), "source: {source}");
}

#[test]
fn init_scaffolds_library_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .arg("--lib")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("lib.ts").exists());

    let source = fs::read_to_string(dir.path().join("lib.ts")).expect("source");
    assert!(source.contains("export function add"), "source: {source}");
}

#[test]
fn test_reports_function_coverage_for_explicit_file_sets() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1 (coverage:"), "stdout: {stdout}");
}

#[test]
fn test_reports_function_coverage_in_json_output() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["coverage"]["mode"], "function");
    assert!(
        json["payload"]["coverage"]["summary"]["functionsTotal"]
            .as_u64()
            .expect("functionsTotal")
            >= 1
    );
}

#[test]
fn test_reports_function_coverage_with_normalized_relative_paths() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 1, "coverage files: {files:?}");
    let file = files[0]["file"].as_str().expect("coverage file path");
    assert!(
        file.ends_with("tests/fixtures/tests/smoke.test.ts"),
        "file path should be project-root relative: {file}"
    );
    assert!(
        !Path::new(file).is_absolute(),
        "file path should not be absolute: {file}"
    );
}

#[test]
fn test_reports_function_coverage_in_deterministic_file_order() {
    let dir = tempdir().expect("tempdir");
    let first_path = dir.path().join("z.test.ts");
    let second_path = dir.path().join("a.test.ts");
    fs::write(
        &first_path,
        r#"Kali.test("z", () => {
    1 + 1;
});
"#,
    )
    .expect("write first test file");
    fs::write(
        &second_path,
        r#"Kali.test("a", () => {
    2 + 2;
});
"#,
    )
    .expect("write second test file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg(&first_path)
        .arg(&second_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 2, "coverage files: {files:?}");

    let first_file = files[0]["file"].as_str().expect("first coverage file path");
    let second_file = files[1]["file"]
        .as_str()
        .expect("second coverage file path");

    assert!(
        first_file.ends_with("a.test.ts"),
        "coverage rows should be sorted deterministically: {first_file}"
    );
    assert!(
        second_file.ends_with("z.test.ts"),
        "coverage rows should be sorted deterministically: {second_file}"
    );
    assert!(
        !Path::new(first_file).is_absolute() && !Path::new(second_file).is_absolute(),
        "coverage file paths should be relative: {files:?}"
    );
}

#[test]
fn test_reports_function_coverage_respects_filter_selection() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(
        &keep,
        r#"Kali.test("math", () => {
    1 + 1;
});
"#,
    )
    .expect("write keep test file");
    fs::write(
        &skip,
        r#"Kali.test("strings", () => {
    2 + 2;
});
"#,
    )
    .expect("write skip test file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--output")
        .arg("json")
        .arg("--coverage")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let files = json["payload"]["coverage"]["files"]
        .as_array()
        .expect("coverage files");
    assert_eq!(files.len(), 1, "coverage files: {files:?}");
    let file = files[0]["file"].as_str().expect("coverage file path");
    assert!(
        file.ends_with("math.test.ts"),
        "coverage should only report filtered files: {file}"
    );
    assert!(
        !Path::new(file).is_absolute(),
        "coverage file path should be relative: {file}"
    );
}

#[test]
fn check_accepts_compat_eval_flag() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_accepts_inherited_compat_eval_feature_from_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compat": {
    "features": ["eval"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_rejects_eval_without_compat_eval() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "if (eval(\"1 + 2\") !== 3) { throw new Error('bad eval result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("compatibility feature 'eval'"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_evaluates_dynamic_eval_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const prefix = \"1\"; const suffix = \" + 2\"; const source = prefix + suffix; if (eval(source) !== 3) { throw new Error('bad eval result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_evaluates_dynamic_function_constructor_sources_when_compat_eval_is_enabled() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const bodyPrefix = \"return \"; const body = bodyPrefix + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--compat")
        .arg("eval")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_embeds_sandbox_policy_custom_section() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "1 + 2;").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let built = fs::read(dir.path().join("main.wasm")).expect("read wasm artifact");
    let policy_bytes = fs::read(&policy_path).expect("read policy bytes");
    let mut seen_policy = None;
    let mut seen_metadata = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:policy" {
                seen_policy = Some(section.data().to_vec());
            }
            if section.name() == "kali:metadata" {
                seen_metadata = true;
            }
        }
    }
    let embedded_policy = seen_policy.expect("custom section 'kali:policy' was not embedded");
    assert_eq!(
        embedded_policy, policy_bytes,
        "custom section 'kali:policy' should match the input policy bytes exactly"
    );
    assert!(
        seen_metadata,
        "custom section 'kali:metadata' was not embedded"
    );
}

#[test]
fn build_emits_library_artifacts_and_metadata() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--max-specializations")
        .arg("4")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.path().join("math.lib.wasm");
    let wit_path = dir.path().join("math.lib.wit");
    let meta_path = dir.path().join("math.lib.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));
    assert!(wit.contains("export add: func();"));

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "lib", 4);
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|entry| entry["name"] == "add"));

    let built = fs::read(&wasm_path).expect("read wasm artifact");
    let mut seen_metadata = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:metadata" {
                seen_metadata = true;
                break;
            }
        }
    }
    assert!(
        seen_metadata,
        "custom section 'kali:metadata' was not embedded"
    );
}

#[test]
fn build_rejects_library_sources_without_static_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(&source_path, "const value = 42; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5011"), "stderr: {stderr}");
    assert!(
        stderr.contains("no statically known export surface"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("math.lib.wasm").exists());
    assert!(!dir.path().join("math.lib.meta.json").exists());
}

#[test]
fn build_rejects_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_rejects_inherited_unknown_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["fiber-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("unsupported runtimeProfile"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn build_emits_browser_bundle_artifacts() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.js");
    let source_map_path = bundle_dir.join("app.js.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("instantiateStreaming"), "bundle js: {js}");
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        js.contains("sourceMappingURL=app.js.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.js");
    assert_eq!(source_map["sources"][0], "app.ts");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "greet");
}

#[test]
fn build_trees_shakes_unused_browser_bundle_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; } function unused() { return 1; }",
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(
        js.contains("export async function greet"),
        "bundle js: {js}"
    );
    assert!(
        !js.contains("export async function unused"),
        "bundle js: {js}"
    );

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    let exports = metadata["exports"].as_array().expect("exports array");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0]["name"], "greet");

    assert_browser_bundle_executes(&bundle_dir, "greet");
}

#[test]
fn build_emits_browser_bundle_crypto_web_apis() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: digestSmoke\nasync function digestSmoke(left, right) {\n  const bytes = new TextEncoder().encode(`browser crypto ${String(left + right)}`);\n  const digest = await crypto.subtle.digest('SHA-512', bytes);\n  const uuid = crypto.randomUUID();\n  if (digest.byteLength !== 64) {\n    throw new Error(`unexpected digest length ${digest.byteLength}`);\n  }\n  if (typeof uuid !== 'string' || uuid.length === 0) {\n    throw new Error(`unexpected uuid ${uuid}`);\n  }\n  return left - left;\n}\n",
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "digestSmoke");
}

#[test]
fn build_emits_browser_bundle_chunks_for_literal_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import(\"./lazy.ts\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    let chunk_dirs: Vec<_> = fs::read_dir(&chunk_root)
        .expect("read chunk root")
        .map(|entry| entry.expect("chunk entry").path())
        .collect();
    assert!(
        !chunk_dirs.is_empty(),
        "expected at least one emitted chunk"
    );
    for chunk_dir in chunk_dirs {
        assert!(
            chunk_dir.is_dir(),
            "chunk entry should be a directory: {}",
            chunk_dir.display()
        );
    }
}

#[test]
fn build_emits_browser_bundle_chunks_for_simple_string_concat_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import(\"./\" + \"lazy.ts\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );

    let chunk_dirs: Vec<_> = fs::read_dir(&chunk_root)
        .expect("read chunk root")
        .map(|entry| entry.expect("chunk entry").path())
        .collect();
    assert!(
        !chunk_dirs.is_empty(),
        "expected at least one emitted chunk"
    );
    for chunk_dir in chunk_dirs {
        assert!(
            chunk_dir.is_dir(),
            "chunk entry should be a directory: {}",
            chunk_dir.display()
        );
    }
}

#[test]
fn build_emits_browser_bundle_chunks_for_const_bound_dynamic_imports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const root = \"./\"; const name = \"lazy.ts\"; const lazy = import((root + name));\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let chunk_root = bundle_dir.join("chunks");
    assert!(chunk_root.exists(), "missing {}", chunk_root.display());

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"chunk-wasm"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"chunk-js"), "artifacts: {artifacts:?}");
    assert!(
        kinds.contains(&"chunk-source-map"),
        "artifacts: {artifacts:?}"
    );
    assert!(
        kinds.contains(&"chunk-meta-json"),
        "artifacts: {artifacts:?}"
    );
}

#[test]
fn browser_bundle_js_exposes_runtime_dynamic_import_loader() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import(\"./\" + \"lazy.ts\");\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js = fs::read_to_string(bundle_dir.join("app.js")).expect("read bundle js");
    assert!(js.contains("loadDynamicImport"), "bundle js: {js}");
    assert!(js.contains("loadWithImports"), "bundle js: {js}");
    assert!(js.contains("lazy.ts"), "bundle js: {js}");

    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./lazy.ts");
}

#[test]
fn browser_bundle_js_normalizes_runtime_dynamic_import_specifiers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(
        &source_path,
        "const lazy = import((\"./\" + \"lazy.ts\"));\nfunction greet(name) { return name; }",
    )
    .expect("write source");
    fs::write(&chunk_path, "export function lazyValue() { return 7; }")
        .expect("write chunk source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    assert_browser_bundle_dynamic_import_loader(&bundle_dir, "./sub/../lazy.ts");
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs")
        .arg("--api")
        .arg("browser")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let wasm_path = bundle_dir.join("app.wasm");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("pathToFileURL"), "bundle js: {js}");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], "app.ts");

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16);
    assert_eq!(metadata["apiSurface"], "browser");

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "cjs");
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    let kinds: Vec<_> = artifacts
        .iter()
        .map(|artifact| artifact["kind"].as_str().expect("artifact kind"))
        .collect();
    assert!(kinds.contains(&"wasm-module"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"js-glue"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"source-map"), "artifacts: {artifacts:?}");
    assert!(kinds.contains(&"meta-json"), "artifacts: {artifacts:?}");
}

#[test]
fn release_build_constant_folds_literal_expressions() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function main() { return 1 + 2 + 3; } main();",
    )
    .expect("write source");

    let fast_dir = dir.path().join("fast");
    let fast_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--fast")
        .arg("--out-dir")
        .arg(&fast_dir)
        .arg(&source_path)
        .output()
        .expect("run kali fast build");
    assert!(
        fast_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&fast_output.stdout),
        String::from_utf8_lossy(&fast_output.stderr)
    );

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let fast_wasm = fs::read(fast_dir.join("math.wasm")).expect("read fast wasm");
    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let fast_adds = count_i64_adds(&fast_wasm);
    let release_adds = count_i64_adds(&release_wasm);

    assert!(
        release_adds < fast_adds,
        "expected release build to reduce add instructions (fast={fast_adds}, release={release_adds})"
    );
}

#[test]
fn release_hot_paths_stay_unboxed_without_tag_checks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function hot(a, b) { return a + b; } hot(1, 2);",
    )
    .expect("write source");

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let release_adds = count_i64_adds(&release_wasm);
    let release_tag_ops = count_tag_boxing_ops(&release_wasm);

    assert!(
        release_adds > 0,
        "expected a numeric hot path in the optimized wasm"
    );
    assert_eq!(
        release_tag_ops, 0,
        "expected the specialized hot path to avoid tag-check / untag boxing ops"
    );
}

#[test]
fn optimization_benchmark_suite_tracks_compile_time_size_and_speed() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        r#"
function dead0(x) { return (x + 0) + (0 + x); }
function dead1(x) { return (x + 0) + (0 + x); }
function dead2(x) { return (x + 0) + (0 + x); }
function dead3(x) { return (x + 0) + (0 + x); }
function dead4(x) { return (x + 0) + (0 + x); }
function dead5(x) { return (x + 0) + (0 + x); }

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return ((x + 0) + (y + 0)) + folded;
}

hot(1, 2);
"#,
    )
    .expect("write source");

    let benchmark = |mode_flag: &str, out_dir_name: &str| {
        let out_dir = dir.path().join(out_dir_name);
        let started = Instant::now();
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg(mode_flag)
            .arg("--out-dir")
            .arg(&out_dir)
            .arg(&source_path)
            .output()
            .expect("run kali build");
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = out_dir.join("math.wasm");
        let wasm_bytes = fs::read(&wasm_path).expect("read benchmark wasm");
        let compile_ms = started.elapsed().as_millis();
        let wasm_size = wasm_bytes.len();
        let instruction_count = count_wasm_instructions(&wasm_bytes);
        let add_count = count_i64_adds(&wasm_bytes);
        let tag_count = count_tag_boxing_ops(&wasm_bytes);

        eprintln!(
            "{}: compile={}ms size={} instructions={} adds={} tag_ops={}",
            mode_flag, compile_ms, wasm_size, instruction_count, add_count, tag_count
        );

        (
            compile_ms,
            wasm_size,
            instruction_count,
            add_count,
            tag_count,
        )
    };

    let (fast_compile_ms, fast_size, fast_instructions, fast_adds, fast_tag_ops) =
        benchmark("--fast", "fast");
    let (release_compile_ms, release_size, release_instructions, release_adds, release_tag_ops) =
        benchmark("--release", "release");
    let (
        advanced_compile_ms,
        advanced_size,
        advanced_instructions,
        advanced_adds,
        advanced_tag_ops,
    ) = benchmark("--release-advanced", "advanced");

    assert!(
        fast_compile_ms > 0,
        "fast build should measure compile time"
    );
    assert!(
        release_compile_ms > 0,
        "release build should measure compile time"
    );
    assert!(
        advanced_compile_ms > 0,
        "release-advanced build should measure compile time"
    );

    assert!(
        release_size < fast_size
            || release_instructions < fast_instructions
            || release_adds < fast_adds,
        "expected release build to improve at least one footprint metric (fast size={fast_size}, release size={release_size}; fast instructions={fast_instructions}, release instructions={release_instructions}; fast adds={fast_adds}, release adds={release_adds})"
    );
    assert!(
        advanced_size < release_size
            || advanced_instructions < release_instructions
            || advanced_adds < release_adds,
        "expected release-advanced build to improve at least one footprint metric further (release size={release_size}, advanced size={advanced_size}; release instructions={release_instructions}, advanced instructions={advanced_instructions}; release adds={release_adds}, advanced adds={advanced_adds})"
    );

    assert!(
        release_adds <= fast_adds,
        "expected release build to avoid more add instructions than fast (fast={fast_adds}, release={release_adds})"
    );
    assert!(
        advanced_adds <= release_adds,
        "expected release-advanced build to avoid more add instructions than release (release={release_adds}, advanced={advanced_adds})"
    );

    assert_eq!(
        fast_tag_ops, 0,
        "benchmark fast path should not box numeric ops"
    );
    assert_eq!(
        release_tag_ops, 0,
        "benchmark release path should not box numeric ops"
    );
    assert_eq!(
        advanced_tag_ops, 0,
        "benchmark release-advanced path should not box numeric ops"
    );
}

#[test]
fn release_advanced_strengthens_algebraic_simplification() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function addZero(x) { return x + 0; } addZero(1);",
    )
    .expect("write source");

    let release_dir = dir.path().join("release");
    let release_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release")
        .arg("--out-dir")
        .arg(&release_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release build");
    assert!(
        release_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&release_output.stdout),
        String::from_utf8_lossy(&release_output.stderr)
    );

    let advanced_dir = dir.path().join("advanced");
    let advanced_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--release-advanced")
        .arg("--out-dir")
        .arg(&advanced_dir)
        .arg(&source_path)
        .output()
        .expect("run kali release-advanced build");
    assert!(
        advanced_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&advanced_output.stdout),
        String::from_utf8_lossy(&advanced_output.stderr)
    );

    let release_wasm = fs::read(release_dir.join("math.wasm")).expect("read release wasm");
    let advanced_wasm = fs::read(advanced_dir.join("math.wasm")).expect("read advanced wasm");
    let release_adds = count_i64_adds(&release_wasm);
    let advanced_adds = count_i64_adds(&advanced_wasm);

    assert!(
        advanced_adds < release_adds,
        "expected release-advanced build to reduce add instructions further (release={release_adds}, advanced={advanced_adds})"
    );
}

#[test]
fn build_artifacts_are_deterministic_across_repeated_invocations() {
    let dir = tempdir().expect("tempdir");

    let executable_source = dir.path().join("main.ts");
    fs::write(&executable_source, "console.log(1);").expect("write executable source");
    let executable_output = dir.path().join("main.wasm");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&executable_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let executable_first = read_artifact_bytes(std::slice::from_ref(&executable_output));

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&executable_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(std::slice::from_ref(&executable_output), &executable_first);

    let library_source = dir.path().join("lib.ts");
    fs::write(
        &library_source,
        "export function add(a, b) { return a + b; }",
    )
    .expect("write library source");
    let library_wasm = dir.path().join("lib.lib.wasm");
    let library_meta = dir.path().join("lib.lib.meta.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&library_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let library_first = read_artifact_bytes(&[library_wasm.clone(), library_meta.clone()]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg(&library_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[library_wasm.clone(), library_meta.clone()],
        &library_first,
    );

    let browser_source = dir.path().join("app.ts");
    fs::write(&browser_source, "function greet(name) { return name; }")
        .expect("write browser source");
    let browser_root = dir.path().join("app");
    let browser_wasm = browser_root.join("app.wasm");
    let browser_js = browser_root.join("app.js");
    let browser_meta = browser_root.join("app.meta.json");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&browser_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let browser_first = read_artifact_bytes(&[
        browser_wasm.clone(),
        browser_js.clone(),
        browser_meta.clone(),
    ]);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&browser_source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_artifact_bytes_stable(
        &[
            browser_wasm.clone(),
            browser_js.clone(),
            browser_meta.clone(),
        ],
        &browser_first,
    );
}

#[test]
fn build_with_profile_data_is_deterministic_across_repeated_invocations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("math.ts");
    fs::write(
        &source_path,
        "function hot(a, b) { return a + b; } hot(1, 2);",
    )
    .expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":"hot","weight":8}]}"#,
    )
    .expect("write profile");
    let out_dir = dir.path().join("out");

    let build = |json_output: bool| {
        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--release")
            .arg("--profile")
            .arg(&profile_path)
            .arg("--out-dir")
            .arg(&out_dir)
            .arg(&source_path);
        if json_output {
            command.arg("--output").arg("json");
        }

        let output = command.output().expect("run kali build with profile");
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        (
            output,
            fs::read(out_dir.join("math.wasm")).expect("read profiled wasm"),
        )
    };

    let (text_first, first) = build(false);
    let (text_second, second) = build(false);
    assert_eq!(
        text_first.stdout, text_second.stdout,
        "PGO build output should be deterministic across repeated text-mode invocations"
    );
    assert_eq!(
        first, second,
        "PGO builds should be deterministic across repeated invocations"
    );

    let (json_first, json_first_wasm) = build(true);
    let (json_second, json_second_wasm) = build(true);
    assert_eq!(
        json_first.stdout, json_second.stdout,
        "PGO build JSON output should be deterministic across repeated invocations"
    );
    assert_eq!(
        json_first_wasm, json_second_wasm,
        "PGO builds should be deterministic across repeated JSON invocations"
    );

    let envelope = parse_json_stdout(&json_first);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["exitCode"], 0);
    assert!(envelope["payload"].is_object(), "envelope: {envelope:?}");
}

#[test]
fn build_rejects_unsupported_pgo_profile_data_version() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with unsupported profile version");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("unsupported PGO profile data version"),
        "stderr: {stderr}"
    );
}

#[test]
fn json_build_rejects_unsupported_pgo_profile_data_version() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    let profile_path = dir.path().join("profile.json");
    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("write profile");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--profile")
        .arg(&profile_path)
        .arg(&source_path)
        .output()
        .expect("run kali build with unsupported profile version");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5009");
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("json build rejection message")
            .contains("unsupported PGO profile data version"),
        "errors: {errors:?}"
    );
}

#[test]
fn build_uses_inherited_browser_api_surface_for_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }",
    )
    .expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_browser_bundle_executes(&dir.path().join("app"), "greet");
}

#[test]
fn build_rejects_bundle_without_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_explicit_browser_api_surface_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_explicit_node_bundle_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn json_build_rejects_inherited_browser_bundle_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["errors"][0]["code"], "E5006");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("runtime profile"),
        "json: {json}"
    );
}

#[test]
fn build_rejects_bundle_format_without_bundle() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "function greet(name) { return name; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--format")
        .arg("cjs")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("--format"), "stderr: {stderr}");
}

#[test]
fn build_accepts_explicit_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
console.log(1);
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn node_cross_module_inference_stays_within_the_phase_3_budget() {
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        &math_path,
        r#"export function double(value) {
    return value + value;
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { double } from './math.ts';

export function quadruple(value) {
    return double(double(value));
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { quadruple } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { quadruple } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { quadruple } from './public.ts';

console.log(quadruple(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn node_cross_module_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget()
{
    let dir = tempdir().expect("tempdir");
    let math_path = dir.path().join("math.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &math_path,
        r#"export function makePair(value) {
    return { left: value, right: value + value };
}
"#,
    )
    .expect("write math module");
    fs::write(
        &helper_path,
        r#"import { makePair } from './math.ts';

export function projectLeft(value) {
    return makePair(value).left;
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectLeft } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectLeft } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectLeft } from './public.ts';

console.log(projectLeft(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn cross_module_higher_order_inference_with_an_explicit_specialization_cap_stays_within_the_phase_3_budget(
) {
    let dir = tempdir().expect("tempdir");
    let factory_path = dir.path().join("factory.ts");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let public_path = dir.path().join("public.ts");
    let source_path = dir.path().join("main.ts");

    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "maxSpecializations": 1
  }
}"#,
    )
    .expect("write manifest");

    fs::write(
        &factory_path,
        r#"export function makeProjector(value) {
    return function project() {
        return value + value;
    };
}
"#,
    )
    .expect("write factory module");
    fs::write(
        &helper_path,
        r#"import { makeProjector } from './factory.ts';

export function projectValue(value) {
    const project = makeProjector(value);
    return project();
}
"#,
    )
    .expect("write helper module");
    fs::write(
        &bridge_path,
        r#"export { projectValue } from './helper.ts';
"#,
    )
    .expect("write bridge module");
    fs::write(
        &public_path,
        r#"export { projectValue } from './bridge.ts';
"#,
    )
    .expect("write public module");
    fs::write(
        &source_path,
        r#"import { projectValue } from './public.ts';

console.log(projectValue(21));
"#,
    )
    .expect("write source");

    let check = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali check");

    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali build");

    assert!(
        build.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    assert!(source_path.with_file_name("main.wasm").exists());
}

#[test]
fn build_rejects_explicit_browser_library_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--lib")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("browser API surface"), "stderr: {stderr}");
}

#[test]
fn build_rejects_multiple_source_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let extra_path = dir.path().join("extra.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");
    fs::write(&extra_path, "console.log(2);").expect("write extra source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .arg(&extra_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(
        stderr.contains("only one primary source file"),
        "stderr: {stderr}"
    );
    assert!(!dir.path().join("main.wasm").exists());
}

#[test]
fn json_init_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"]["library"], false);
    assert_eq!(json["exitCode"], 0);
}

#[test]
fn json_check_emits_diagnostic_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "missing;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert_eq!(json["payload"]["filesChecked"], 1);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn check_rejects_fix_flag_as_later_compatibility() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("lint --fix"), "stderr: {stderr}");
}

#[test]
fn json_check_rejects_fix_flag_as_later_compatibility() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--fix")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("error message")
        .contains("lint --fix"));
}

#[test]
fn json_check_rejects_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn json_check_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5009");
    assert!(errors[0]["message"]
        .as_str()
        .expect("error message")
        .contains("duplicate runtimeProfile"));
}

#[test]
fn json_check_rejects_browser_api_surface_with_wasm_threads() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn json_check_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.len() >= 2, "errors: {errors:?}");
    let messages = errors
        .iter()
        .map(|entry| entry["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.SharedArrayBuffer")),
        "messages: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.Atomics")),
        "messages: {messages:?}"
    );
}

#[test]
fn json_run_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.iter().any(|entry| entry["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|entry| entry["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.SharedArrayBuffer")),
        "messages: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.Atomics")),
        "messages: {messages:?}"
    );
}

#[test]
fn json_test_rejects_threaded_runtime_globals() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(
        &source_path,
        "globalThis.SharedArrayBuffer; globalThis.Atomics;",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.iter().any(|entry| entry["code"] == "E5006"));
    let messages = errors
        .iter()
        .map(|entry| entry["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.SharedArrayBuffer")),
        "messages: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("globalThis.Atomics")),
        "messages: {messages:?}"
    );
}

#[test]
fn json_run_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn json_test_rejects_positive_thread_budget_override() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--max-threads")
        .arg("1")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn json_fmt_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function add(a,b){return a+b;}").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("fmt")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "fmt");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(
        json["payload"],
        json!({"filesChecked": 1, "filesFormatted": 1})
    );
}

#[test]
fn json_lint_emits_a_command_envelope() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const x = 1; x;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("lint")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "lint");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(
        json["payload"],
        json!({"filesLinted": 1, "errorCount": 0, "warningCount": 0, "fixedCount": 0})
    );
}

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
fn json_run_emits_a_command_envelope() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn json_test_emits_a_command_envelope() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

#[test]
fn pretty_without_json_exits_with_usage_code() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--pretty")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
}

#[test]
fn init_rejects_non_empty_directory_with_usage_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("notes.txt"), "keep me").expect("write file");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
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
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(stderr.contains("JSR targets"), "stderr: {stderr}");
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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(
        stderr.contains("effective npm-scriptable install work"),
        "stderr: {stderr}"
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
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(
        stderr.contains("explicit registry package target"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import 'node:path';
1 + 2;
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_accepts_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_dir = dir.path().join("tests");
    fs::create_dir_all(&source_dir).expect("create test dir");
    let test_path = source_dir.join("node.test.ts");
    fs::write(
        &test_path,
        r#"import 'node:path';
Kali.test('node', () => {
    console.log('node test ok');
});
"#,
    )
    .expect("write test source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("node")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok"), "stdout: {stdout}");
}

#[test]
fn test_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn test_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_browser_api_surface_in_phase_one() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["flag"], "--api");
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn json_test_rejects_inherited_browser_api_surface_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
    assert_eq!(errors[0]["context"]["origin"], "config");
    assert_eq!(
        errors[0]["context"]["configPath"],
        "compilerOptions.apiSurface"
    );
    assert_eq!(errors[0]["context"]["requestedValue"], "browser");
    assert_eq!(errors[0]["context"]["effectiveValue"], "browser");
}

#[test]
fn test_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_browser_api_surface_with_sandbox_in_phase_one() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors: {errors:?}");
    assert_eq!(errors[0]["code"], "E5006");
    assert_browser_runtime_rejection_message(
        errors[0]["message"]
            .as_str()
            .expect("browser rejection message"),
    );
    assert_browser_runtime_rejection_notes(
        errors[0]["notes"]
            .as_array()
            .expect("browser rejection notes"),
    );
}

#[test]
fn build_emits_capi_artifacts_and_header_compiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--capi")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = source_path.with_file_name("lib.capi.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let header_path = source_path.with_file_name("lib.h");
    let meta_path = source_path.with_file_name("lib.capi.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(wasm_path.exists(), "missing {}", wasm_path.display());
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(header_path.exists(), "missing {}", header_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], 2);
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(
        binding_package["moduleName"],
        source_path.display().to_string()
    );
    assert_eq!(binding_package["artifacts"]["library"], "lib.capi.wasm");
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.capi.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.h");

    let header_check = dir.path().join("header-check.c");
    fs::write(
        &header_check,
        "#include \"lib.h\"\nint main(void) { return 0; }\n",
    )
    .expect("write header check");
    let compile = Command::new("cc")
        .current_dir(dir.path())
        .arg("-fsyntax-only")
        .arg(&header_check)
        .output()
        .expect("run cc");
    assert!(
        compile.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
}

#[test]
fn build_emits_component_artifacts_and_valid_component_bytes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let component_path = source_path.with_file_name("lib.component.wasm");
    let wit_path = source_path.with_file_name("lib.wit");
    let meta_path = source_path.with_file_name("lib.component.meta.json");
    let binding_package_path = source_path.with_file_name("lib.binding-package.json");
    assert!(
        component_path.exists(),
        "missing {}",
        component_path.display()
    );
    assert!(wit_path.exists(), "missing {}", wit_path.display());
    assert!(meta_path.exists(), "missing {}", meta_path.display());
    assert!(
        binding_package_path.exists(),
        "missing {}",
        binding_package_path.display()
    );

    let wit = fs::read_to_string(&wit_path).expect("read wit sidecar");
    assert!(wit.contains("package kali:embed;"));

    let binding_package: Value = serde_json::from_str(
        &fs::read_to_string(&binding_package_path).expect("read binding package manifest"),
    )
    .expect("parse binding package manifest json");
    assert_eq!(binding_package["schemaVersion"], 1);
    assert_eq!(binding_package["kind"], "binding-package");
    assert_eq!(
        binding_package["artifacts"]["library"],
        "lib.component.wasm"
    );
    assert_eq!(
        binding_package["artifacts"]["metadata"],
        "lib.component.meta.json"
    );
    assert_eq!(binding_package["artifacts"]["exportsHeader"], "lib.wit");

    let component_bytes = fs::read(&component_path).expect("read component bytes");
    wasmparser::Validator::new()
        .validate_all(&component_bytes)
        .expect("generated component should validate");
}

#[test]
fn build_emits_component_json_artifacts_for_binding_package_manifest() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        source_path.with_file_name("lib.binding-package.json")
    );
    let artifacts = payload["artifacts"].as_array().expect("artifacts array");
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["kind"] == "binding-package"));
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["role"] == "binding-package-manifest"));
}

#[test]
fn build_emits_component_artifacts_into_an_explicit_output_directory() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    let out_dir = dir.path().join("dist");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--component")
        .arg("--output")
        .arg("json")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_json_stdout(&output);
    let payload = envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "component");
    assert_eq!(
        PathBuf::from(
            payload["outputPath"]
                .as_str()
                .expect("component output path")
        ),
        out_dir.join("lib.component.wasm")
    );
    assert_eq!(
        PathBuf::from(
            payload["bindingPackagePath"]
                .as_str()
                .expect("binding package path")
        ),
        out_dir.join("lib.binding-package.json")
    );

    for artifact in payload["artifacts"].as_array().expect("artifacts array") {
        let artifact_path = PathBuf::from(artifact["path"].as_str().expect("artifact path string"));
        assert!(
            artifact_path.starts_with(&out_dir),
            "artifact path should stay in the requested output directory: {artifact_path:?}"
        );
    }

    assert!(out_dir.join("lib.component.wasm").exists());
    assert!(out_dir.join("lib.wit").exists());
    assert!(out_dir.join("lib.component.meta.json").exists());
    assert!(out_dir.join("lib.binding-package.json").exists());
}

#[test]
fn run_surfaces_console_stdout_for_numeric_logs() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

#[test]
fn run_executes_package_bin_entrypoints_with_shebangs_after_stripping_the_shebang_line() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/hello-bin");
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "hello-bin",
  "version": "1.0.0",
  "bin": "bin/hello.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/hello.js"),
        "#!/usr/bin/env node\nconsole.log(1);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/hello.js"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}

fn write_semver_style_package_fixture(package_dir: &Path) {
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "1.2.3",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/semver.js"),
        r#"#!/usr/bin/env node
function help() {
  console.log('Usage: semver [options] <version> [<version> [...]]');
}

if (process.argv.length == 2) {
  help();
} else {
  console.log(process.argv.length);
}
"#,
    )
    .expect("write semver bin");
}

fn write_semver_package_json_probe_fixture(package_dir: &Path) {
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "1.0.0",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/semver.js"),
        r#"#!/usr/bin/env node
console.log(require('../package.json').version);
console.log(process.argv.length);
"#,
    )
    .expect("write semver bin");
}

#[test]
fn run_executes_semver_style_package_bin_help_path_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package_fixture(&package_dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage: semver [options] <version> [<version> [...]]"),
        "stdout: {stdout}"
    );
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn run_executes_semver_style_package_bin_argument_passthrough_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "1.0.0",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/semver.js"),
        "#!/usr/bin/env node\nconst argv = process.argv.slice(2);\nconst helper = require('../lib/helper');\nconsole.log(argv.length, helper);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(
        stderr.contains("npm package bin 'semver'"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("CommonJS require()"), "stderr: {stderr}");
    assert!(stderr.contains("Node process global"), "stderr: {stderr}");
}

#[test]
fn run_executes_semver_style_package_bin_package_json_require_on_node_api_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_package_json_probe_fixture(&package_dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(package_dir.join("bin/semver.js"))
        .arg("--")
        .arg("1.2.3")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.0.0\n3\n");
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn run_executes_semver_package_consumer_calls_on_the_default_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js"
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("index.js"),
        r#"export function valid(v) { return v; }
export function satisfies(version, range) { return version === '1.2.3' && range === '^1.0.0'; }
export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        dir.path().join("main.ts"),
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write consumer source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(dir.path().join("main.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2.3\n1\n1.2.3\n", "stdout: {stdout}");
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn regression_package_bin_entrypoints_requiring_package_json_still_fail_on_default_surface() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    fs::create_dir_all(package_dir.join("bin")).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "semver",
  "version": "1.0.0",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package json");
    fs::write(
        package_dir.join("bin/semver.js"),
        "#!/usr/bin/env node\nconst pkg = require('../package.json');\nconsole.log(pkg.version);\n",
    )
    .expect("write package bin");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(package_dir.join("bin/semver.js"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(
        stderr.contains("npm package bin 'semver'"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("CommonJS require()"), "stderr: {stderr}");
}

#[test]
fn effects_command_emits_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"
fetch("https://api.example.com/data");
console.log("hello");
"#,
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(
        json["entryPoints"],
        json!([source_path.display().to_string()])
    );
    assert_eq!(json["dynamicEffects"], false);
    assert_eq!(json["dynamicReasons"], json!([]));
    let kinds = json["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
    assert!(kinds.contains(&"Network.Fetch"));
}

#[test]
fn effects_command_marks_proxy_constructor_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "new Proxy({}, {});\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["proxy-traps"]));
    assert!(json["effects"]
        .as_array()
        .expect("effects array")
        .is_empty());
}

#[test]
fn effects_command_marks_proxy_revocable_calls_as_dynamic() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Proxy.revocable({}, {});\nglobalThis.Proxy.revocable({}, {});\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["dynamicEffects"], true);
    assert_eq!(json["dynamicReasons"], json!(["proxy-traps"]));
    assert!(json["effects"]
        .as_array()
        .expect("effects array")
        .is_empty());
}

#[test]
fn effects_uses_explicit_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_uses_inherited_browser_api_surface() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log('hello');\nfetch('https://example.com');",
    )
    .expect("write source");
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

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["analysisContext"]["apiSurface"], "browser");
    assert_eq!(json["dynamicEffects"], false);
}

#[test]
fn effects_rejects_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn effects_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn json_effects_rejects_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
        .arg("--wasm-threads")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn json_effects_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('ok');").expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("effects")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn package_effects_command_emits_native_json_payload() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["package"]["name"], "purepkg");
    assert_eq!(json["package"]["version"], "1.0.0");
    assert_eq!(json["package"]["registry"], "npm");
    assert_eq!(json["report"]["entryPoints"], json!(["purepkg"]));
    let kinds = json["report"]["effects"]
        .as_array()
        .expect("effects array")
        .iter()
        .map(|entry| entry["kind"].as_str().expect("kind string"))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"Console.Write"));
}

#[test]
fn package_effects_rejects_inherited_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads", "wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5009"), "stderr: {stderr}");
    assert!(
        stderr.contains("duplicate runtimeProfile"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_effects_rejects_inherited_wasm_threads_runtime_profile() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/purepkg");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "purepkg",
  "version": "1.0.0",
  "main": "index.js"
}"#,
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), "console.log('hello');").expect("write package entry");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("package-effects")
        .arg("purepkg")
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("wasm-threads"), "stderr: {stderr}");
}

#[test]
fn check_with_sandbox_rejects_inferred_effects() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "fetch('https://api.example.com/data');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
}

#[test]
fn check_with_sandbox_rejects_phase_three_deno_host_effects() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "Deno.env.set('KALI_CORPUS_FLAG', 'set');\nnew Deno.Command('sh').spawn();\nDeno.connect('127.0.0.1', 1);\nDeno.listen('127.0.0.1', 0);\nDeno.serve('127.0.0.1', 0);\n",
    )
    .expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E9007"), "stderr: {stderr}");
    assert!(
        stderr.contains("Process.EnvWrite")
            || stderr.contains("Process.Spawn")
            || stderr.contains("Network.Connect")
            || stderr.contains("Network.Listen"),
        "stderr: {stderr}"
    );
}

#[test]
fn check_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
}

#[test]
fn json_check_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn test_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    fs::write(&source_path, "Kali.test(\"addition\", () => { 1 + 2; });").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

#[test]
fn run_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxThreads"), "stderr: {stderr}");
}

#[test]
fn json_run_with_sandbox_rejects_positive_thread_budget_policy() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log('thread policy');").expect("write source");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(
        &policy_path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 1
  }
}"#,
    )
    .expect("write policy");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    assert!(!json["errors"].as_array().expect("errors array").is_empty());
    assert_eq!(json["errors"][0]["code"], "E5006");
}

fn package_audit_metadata_body(
    postinstall_script: Option<&str>,
    native_addon: bool,
) -> &'static str {
    let mut version = json!({
        "name": "lodash",
        "version": "1.0.0",
        "main": if native_addon { "native.node" } else { "index.js" },
        "dist": {
            "tarball": "http://127.0.0.1:0/lodash.tgz",
            "integrity": "sha512-demo"
        }
    });

    if let Some(script) = postinstall_script {
        version["scripts"] = json!({"postinstall": script});
    }

    Box::leak(
        json!({
            "versions": {
                "1.0.0": version
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

#[test]
fn package_audit_command_emits_envelope() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no security findings were computed"),
        "stdout: {stdout}"
    );
}

#[test]
fn package_audit_rejects_preview_compatibility_shim() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
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
        "registry server should not be queried"
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5008"), "stderr: {stderr}");
    assert!(
        stderr.contains("`--preview` is no longer accepted for package-audit"),
        "stderr: {stderr}"
    );
}

#[test]
fn package_audit_command_emits_json_envelope() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(None, false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("no security findings were computed"));
    assert_eq!(json["warnings"], serde_json::Value::Array(vec![]));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
}

#[test]
fn package_audit_command_reports_findings() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(package_audit_metadata_body(Some("echo ok"), false));

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("package-audit")
        .arg("--output")
        .arg("json")
        .arg("lodash")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "registry server should be queried"
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], true);
    assert_eq!(json["payload"], serde_json::Value::Null);
    assert!(json["stdout"]
        .as_str()
        .expect("stdout string")
        .contains("0 error(s), 1 warning(s)"));
    assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
    assert_eq!(
        json["warnings"].as_array().expect("warnings array").len(),
        1
    );
    assert_eq!(json["warnings"][0]["code"], "W6006");
}
