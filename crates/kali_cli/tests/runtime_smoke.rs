use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::symlink;

use base64::Engine;
use flate2::{write::GzEncoder, Compression};
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use tar::Builder;
use wasmparser::{Operator, Parser, Payload};

use kali_common::{
    late_object_model_own_property_source as kali_common_late_object_model_own_property_source,
    math_abs_sign_frozen_callable_invocation_lines, math_cbrt_frozen_callable_aliases,
    math_floor_trunc_ceil_frozen_callable_aliases, math_pow_browser_alias_inventory_aliases,
    math_pow_browser_alias_inventory_invocation_source,
};
use kali_optimize::{ProfileData, ProfileSample, ProfileSampleKind};
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

fn write_browser_api_surface_manifest(dir: &Path) {
    fs::write(
        dir.join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");
}

fn browser_runtime_try_catch_and_finally_source() -> &'static str {
    r#"let caught = false;
try {
  throw 'boom';
} catch {
  caught = true;
}
if (!caught) {
  throw new Error('catch did not run');
}
try {
  0;
} finally {
  console.log(2);
}
console.log(1);
"#
}

fn browser_runtime_queue_microtask_source() -> &'static str {
    r#"async function main() {
  let microtaskRan = false;
  queueMicrotask(() => {
    microtaskRan = true;
  });
  if (microtaskRan) {
    throw new Error('microtask ran too early');
  }
  await Promise.resolve();
  if (!microtaskRan) {
    throw new Error('microtask did not run before the next turn');
  }
  console.log('queueMicrotask ok');
}
main();
"#
}

fn promise_all_sequencing_source() -> &'static str {
    r#"async function main() {
  const values = await Promise.all([Promise.resolve(1n), Promise.resolve(2n)]);
  if (values.length !== 2 || values[0] !== 1n || values[1] !== 2n) {
    throw new Error(`unexpected Promise.all result ${values.join(',')}`);
  }
}
main();
"#
}

fn browser_bundle_unary_prefix_semantics_source() -> &'static str {
    r#"// kali-tree-shake: unaryPrefixSmoke
export function unaryPrefixSmoke() {
  const notTrue = !true;
  if (notTrue !== false) {
    throw new Error('expected logical negation to invert the boolean');
  }
  const negative = -(1 + 2);
  if (negative !== -3) {
    throw new Error('expected unary minus to negate the value');
  }
  const positive = +(1 + 2);
  if (positive !== 3) {
    throw new Error('expected unary plus to preserve the numeric value');
  }
  const bitwiseNot = ~1;
  if (bitwiseNot !== -2) {
    throw new Error('expected bitwise not to invert integer bits');
  }
  const value = void (1 + 2);
  if (value !== void 0) {
    throw new Error('expected void to evaluate to undefined');
  }
  if (typeof value !== 'undefined') {
    throw new Error('expected void result to be undefined');
  }
  return 0n;
}
"#
}

fn browser_bundle_promise_all_sequencing_source() -> &'static str {
    r#"// kali-tree-shake: promiseAllSmoke
export async function promiseAllSmoke(left, right) {
  const values = await Promise.all([Promise.resolve(left), Promise.resolve(right)]);
  if (values.length !== 2 || values[0] !== left || values[1] !== right) {
    throw new Error(`unexpected Promise.all result ${values.join(',')}`);
  }
  return 0n;
}
"#
}

fn late_process_control_source() -> String {
    kali_common::late_process_control_single_quoted_process_source()
}

fn late_process_env_mutation_source() -> String {
    kali_common::late_process_env_mutation_source()
}

fn late_object_model_source() -> &'static str {
    kali_common::late_object_model_source()
}

fn late_object_model_own_property_source() -> &'static str {
    kali_common_late_object_model_own_property_source()
}

fn broader_intl_source() -> String {
    kali_common::broader_intl_source()
}

fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"]();"
}

fn threaded_runtime_source() -> &'static str {
    kali_common::late_threaded_runtime_source()
}

fn permission_escalation_source() -> &'static str {
    "Deno.permissions.request(); Deno.permissions[\"request\"](); Deno.permissions.revoke(); Deno.permissions[\"revoke\"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions[\"request\"](); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions[\"revoke\"](); globalThis[\"Deno\"].permissions.request(); globalThis[\"Deno\"].permissions.revoke(); globalThis[\"Deno\"].permissions[\"request\"](); globalThis[\"Deno\"].permissions[\"revoke\"]();"
}

fn permission_escalation_computed_source() -> &'static str {
    r#"globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#
}

fn permission_escalation_bracketed_source() -> &'static str {
    r#"Deno["permissions"]["request"](); Deno["permissions"]["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis.Deno["permissions"]["request"](); globalThis.Deno["permissions"]["revoke"]();"#
}

fn permission_escalation_mixed_bracketed_source() -> &'static str {
    r#"globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions.revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"]();"#
}

fn supported_permission_query_const_binding_source() -> &'static str {
    r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions.query({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: env_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });"#
}

fn supported_permission_query_const_binding_runtime_source() -> String {
    r#"async function main() {{
const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
await Deno.permissions.query({{ name: read_descriptor }});
await Deno.permissions.query({{ name: write_descriptor }});
await Deno.permissions.query({{ name: env_descriptor }});
await Deno.permissions.query({{ name: net_descriptor }});
await Deno.permissions["query"]({{ name: read_descriptor }});
await Deno.permissions["query"]({{ name: write_descriptor }});
await Deno.permissions["query"]({{ name: env_descriptor }});
await Deno.permissions["query"]({{ name: net_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: read_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: read_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: write_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: write_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: env_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: env_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: net_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: net_descriptor }});
  console.log("permission query const bindings ok");
}}
main();
"#
    .to_string()
}

fn supported_permission_query_const_binding_test_source() -> String {
    r#"async function main() {{
const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
await Deno.permissions.query({{ name: read_descriptor }});
await Deno.permissions.query({{ name: write_descriptor }});
await Deno.permissions.query({{ name: env_descriptor }});
await Deno.permissions.query({{ name: net_descriptor }});
await Deno.permissions["query"]({{ name: read_descriptor }});
await Deno.permissions["query"]({{ name: write_descriptor }});
await Deno.permissions["query"]({{ name: env_descriptor }});
await Deno.permissions["query"]({{ name: net_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: read_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: read_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: write_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: write_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: env_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: env_descriptor }});
await globalThis["Deno"]["permissions"].query({{ name: net_descriptor }});
await globalThis["Deno"]["permissions"]["query"]({{ name: net_descriptor }});
}}
Kali.test('permission query const bindings', () => main());
"#
    .to_string()
}

fn assert_permission_escalation_stderr(stderr: &str, expected: &[&str]) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in expected {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_permission_escalation_json(errors: &[Value], expected: &[&str], expected_len: usize) {
    assert_eq!(errors.len(), expected_len);
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    for expected in expected {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {:?}",
            errors
        );
    }
}

fn assert_deno_filesystem_apis_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    let test_path = dir.path().join(format!("main.test.{extension}"));
    fs::write(dir.path().join("input.txt"), "alpha").expect("write input");
    fs::write(dir.path().join("open.txt"), "beta").expect("write open input");
    fs::write(
        &source_path,
        "Deno.mkdir('./nested', false);\nDeno.rename('./input.txt', './nested/renamed.txt');\nDeno.lstat('./nested/renamed.txt');\nDeno.remove('./nested/renamed.txt');\nDeno.remove('./nested', true);\nDeno.open('./open.txt');\nDeno.create('./created.txt');\nconsole.log('done');\n",
    )
    .expect("write source");
    fs::write(
        &test_path,
        "Kali.test('filesystem', () => { Deno.mkdir('./nested', false); Deno.rename('./input.txt', './nested/renamed.txt'); Deno.lstat('./nested/renamed.txt'); Deno.remove('./nested/renamed.txt'); Deno.remove('./nested', true); Deno.open('./open.txt'); Deno.create('./created.txt'); console.log('done'); });\n",
    )
    .expect("write test source");

    for command in ["check", "build"] {
        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg(command)
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success(), "{command} failed: {:?}", output);
    }

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "done", "stdout: {stdout}");

    fs::write(dir.path().join("input.txt"), "alpha").expect("reset json run input");

    let json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&json_output.stdout),
        String::from_utf8_lossy(&json_output.stderr)
    );
    let json = parse_json_stdout(&json_output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("done"));
    assert_eq!(json["stderr"], "");

    fs::write(dir.path().join("input.txt"), "alpha").expect("reset test input");

    let test_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        test_output.status.success(),
        "test failed: {:?}",
        test_output
    );
    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    assert!(test_stdout.contains("done"), "stdout: {test_stdout}");

    let test_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_path)
        .output()
        .expect("run kali");

    assert!(
        test_json_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json_output.stdout),
        String::from_utf8_lossy(&test_json_output.stderr)
    );
    let test_json = parse_json_stdout(&test_json_output);
    assert_eq!(test_json["schemaVersion"], 1);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("test stdout")
            .contains("done"),
        "json test: {test_json}"
    );
}

fn browser_harness_bracketed_deno_pid_source() -> &'static str {
    "console.log(Deno[\"pid\"]);\nconsole.log(globalThis[\"Deno\"][\"pid\"]);\n"
}

fn structured_clone_and_event_primitives_source(test_mode: bool) -> String {
    let source = if test_mode {
        r#"Kali.test('web baseline', () => {
  const original = { nested: { count: 1 }, values: [1, 2, 3] };
  const cloned = structuredClone(original);
  if (cloned === original || cloned.nested === original.nested || cloned.values === original.values) {
    throw new Error('structuredClone should deep-clone object graphs');
  }
  original.nested.count = 2;
  original.values.push(4);
  if (cloned.nested.count !== 1 || cloned.values.join(',') !== '1,2,3') {
    throw new Error(`unexpected structuredClone result ${JSON.stringify(cloned)}`);
  }
  const controller = new AbortController();
  if (!(controller.signal instanceof AbortSignal)) {
    throw new Error('expected AbortSignal from AbortController');
  }
  const event = new Event('tick');
  if (event.type !== 'tick') {
    throw new Error(`unexpected Event behavior ${event.type}`);
  }
  const target = new EventTarget();
  let count = 0;
  target.addEventListener('tick', () => {
    count += 1;
    controller.abort();
  });
  const dispatched = target.dispatchEvent(new CustomEvent('tick'));
  if (!dispatched || count !== 1 || !controller.signal.aborted) {
    throw new Error('unexpected event primitive behavior');
  }
  const query = new URLSearchParams('alpha=1&beta=two+words');
  query.append('gamma', String(count));
  query.set('beta', String(count));
  if (query.get('alpha') !== '1' || query.get('beta') !== String(count) || query.getAll('beta').length !== 1 || !query.has('gamma')) {
    throw new Error(`unexpected URLSearchParams behavior ${query.toString()}`);
  }
  const browserUrl = new URL('https://example.com/browser?alpha=1#fragment');
  if (browserUrl.origin !== 'https://example.com' || browserUrl.pathname !== '/browser' || browserUrl.search !== '?alpha=1' || browserUrl.hash !== '#fragment' || browserUrl.searchParams.get('alpha') !== '1') {
    throw new Error(`unexpected URL behavior ${browserUrl.href}`);
  }
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const encoded = encoder.encode(String(count));
  if (decoder.decode(encoded) !== String(count)) {
    throw new Error('unexpected TextEncoder/TextDecoder behavior');
  }
});
"#
    } else {
        r#"const original = { nested: { count: 1 }, values: [1, 2, 3] };
const cloned = structuredClone(original);
if (cloned === original || cloned.nested === original.nested || cloned.values === original.values) {
  throw new Error('structuredClone should deep-clone object graphs');
}
original.nested.count = 2;
original.values.push(4);
if (cloned.nested.count !== 1 || cloned.values.join(',') !== '1,2,3') {
  throw new Error(`unexpected structuredClone result ${JSON.stringify(cloned)}`);
}
const controller = new AbortController();
if (!(controller.signal instanceof AbortSignal)) {
  throw new Error('expected AbortSignal from AbortController');
}
const event = new Event('tick');
if (event.type !== 'tick') {
  throw new Error(`unexpected Event behavior ${event.type}`);
}
const target = new EventTarget();
let count = 0;
target.addEventListener('tick', () => {
  count += 1;
  controller.abort();
});
const dispatched = target.dispatchEvent(new CustomEvent('tick'));
if (!dispatched || count !== 1 || !controller.signal.aborted) {
  throw new Error('unexpected event primitive behavior');
}
const query = new URLSearchParams('alpha=1&beta=two+words');
query.append('gamma', String(count));
query.set('beta', String(count));
if (query.get('alpha') !== '1' || query.get('beta') !== String(count) || query.getAll('beta').length !== 1 || !query.has('gamma')) {
  throw new Error(`unexpected URLSearchParams behavior ${query.toString()}`);
}
const browserUrl = new URL('https://example.com/browser?alpha=1#fragment');
if (browserUrl.origin !== 'https://example.com' || browserUrl.pathname !== '/browser' || browserUrl.search !== '?alpha=1' || browserUrl.hash !== '#fragment' || browserUrl.searchParams.get('alpha') !== '1') {
  throw new Error(`unexpected URL behavior ${browserUrl.href}`);
}
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const encoded = encoder.encode(String(count));
if (decoder.decode(encoded) !== String(count)) {
  throw new Error('unexpected TextEncoder/TextDecoder behavior');
}
console.log('web baseline ok');
"#
    };

    source.to_string()
}

fn assert_artifact_metadata_provenance(
    metadata: &Value,
    artifact_kind: &str,
    expected_max_specializations: usize,
    expected_profile_data_hash: Option<&str>,
) {
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["artifactKind"], artifact_kind);
    assert_eq!(metadata["runtimeProfiles"], json!([]));
    assert_eq!(metadata["maxSpecializations"], expected_max_specializations);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");

    match expected_profile_data_hash {
        Some(expected) => assert_eq!(metadata["profileDataHash"], expected),
        None => assert!(metadata.get("profileDataHash").is_none()),
    }
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
        text.contains("supported browser runtime commands: run, test"),
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
        notes.iter().any(|note| note.as_str() == Some("browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid")),
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

fn start_binary_response_server(
    body: Vec<u8>,
    content_type: &'static str,
) -> (
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind response server");
    listener.set_nonblocking(true).expect("set nonblocking");
    let addr = listener.local_addr().expect("response server address");
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
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut builder = Builder::new(encoder);

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

fn format_sha512(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(Sha512::digest(bytes))
}

fn kali_registry_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
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

fn browser_runtime_object_enumeration_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const fromEntries = Object.fromEntries([["b", 1], ["a", 2]]);
const fromEntriesKeys = Object.keys(fromEntries);
const fromEntriesEntries = Object.entries(fromEntries);
const fromEntriesValues = Object.values(fromEntries);
const wrappedEntries = ([["b", 1], ["a", 2]]);
const wrappedFromEntries = Object.fromEntries(wrappedEntries);
const wrappedFromEntriesKeys = Object.keys(wrappedFromEntries);
const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2]]));
const frozenFromEntriesKeys = Object.keys(frozenFromEntries);
const frozenFromEntriesEntries = Object.entries(frozenFromEntries);
const frozenFromEntriesValues = Object.values(frozenFromEntries);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3 ||
  fromEntriesKeys.length !== 2 ||
  fromEntriesKeys[0] !== 'b' ||
  fromEntriesKeys[1] !== 'a' ||
  fromEntriesEntries.length !== 2 ||
  fromEntriesEntries[0][0] !== 'b' ||
  fromEntriesEntries[0][1] !== 1 ||
  fromEntriesEntries[1][0] !== 'a' ||
  fromEntriesEntries[1][1] !== 2 ||
  fromEntriesValues.length !== 2 ||
  fromEntriesValues[0] !== 1 ||
  fromEntriesValues[1] !== 2 ||
  wrappedFromEntriesKeys.length !== 2 ||
  wrappedFromEntriesKeys[0] !== 'b' ||
  wrappedFromEntriesKeys[1] !== 'a' ||
  frozenFromEntriesKeys.length !== 2 ||
  frozenFromEntriesKeys[0] !== 'b' ||
  frozenFromEntriesKeys[1] !== 'a' ||
  frozenFromEntriesEntries.length !== 2 ||
  frozenFromEntriesEntries[0][0] !== 'b' ||
  frozenFromEntriesEntries[0][1] !== 1 ||
  frozenFromEntriesEntries[1][0] !== 'a' ||
  frozenFromEntriesEntries[1][1] !== 2 ||
  frozenFromEntriesValues.length !== 2 ||
  frozenFromEntriesValues[0] !== 1 ||
  frozenFromEntriesValues[1] !== 2
) {
  throw new Error('unexpected numeric-key ordering');
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw new Error('unexpected delete-reinsert ordering');
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#
}

fn browser_runtime_object_enumeration_test_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const fromEntries = Object.fromEntries([["b", 1], ["a", 2]]);
const fromEntriesKeys = Object.keys(fromEntries);
const fromEntriesEntries = Object.entries(fromEntries);
const fromEntriesValues = Object.values(fromEntries);
const wrappedEntries = ([["b", 1], ["a", 2]]);
const wrappedFromEntries = Object.fromEntries(wrappedEntries);
const wrappedFromEntriesKeys = Object.keys(wrappedFromEntries);
const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2]]));
const frozenFromEntriesKeys = Object.keys(frozenFromEntries);
const frozenFromEntriesEntries = Object.entries(frozenFromEntries);
const frozenFromEntriesValues = Object.values(frozenFromEntries);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3 ||
  fromEntriesKeys.length !== 2 ||
  fromEntriesKeys[0] !== 'b' ||
  fromEntriesKeys[1] !== 'a' ||
  fromEntriesEntries.length !== 2 ||
  fromEntriesEntries[0][0] !== 'b' ||
  fromEntriesEntries[0][1] !== 1 ||
  fromEntriesEntries[1][0] !== 'a' ||
  fromEntriesEntries[1][1] !== 2 ||
  fromEntriesValues.length !== 2 ||
  fromEntriesValues[0] !== 1 ||
  fromEntriesValues[1] !== 2 ||
  wrappedFromEntriesKeys.length !== 2 ||
  wrappedFromEntriesKeys[0] !== 'b' ||
  wrappedFromEntriesKeys[1] !== 'a' ||
  frozenFromEntriesKeys.length !== 2 ||
  frozenFromEntriesKeys[0] !== 'b' ||
  frozenFromEntriesKeys[1] !== 'a' ||
  frozenFromEntriesEntries.length !== 2 ||
  frozenFromEntriesEntries[0][0] !== 'b' ||
  frozenFromEntriesEntries[0][1] !== 1 ||
  frozenFromEntriesEntries[1][0] !== 'a' ||
  frozenFromEntriesEntries[1][1] !== 2 ||
  frozenFromEntriesValues.length !== 2 ||
  frozenFromEntriesValues[0] !== 1 ||
  frozenFromEntriesValues[1] !== 2
) {
  throw new Error('unexpected numeric-key ordering');
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw new Error('unexpected delete-reinsert ordering');
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
Kali.test('browser runtime smoke', () => {});
"#
}

fn browser_runtime_reflect_own_keys_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const alias = obj;
const keys = Reflect.ownKeys(obj);
const aliasKeys = Reflect.ownKeys(alias);
const globalKeys = globalThis.Reflect.ownKeys(obj);
const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
const mixedBracketedRootKeys = globalThis["Reflect"]['ownKeys'](obj);
const mixedSingleQuotedRootKeys = globalThis['Reflect']["ownKeys"](obj);
const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])(alias);
const mixedKeys = globalThis.Reflect["ownKeys"](alias);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const frozenBracketedKeys = globalThis["Reflect"]["ownKeys"](alias);
const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)(alias);
const parenthesizedFrozenBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(alias);
const parenthesizedFrozenSingleQuotedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(alias);
const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(alias);
const parenthesizedFrozenMixedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(alias);
const parenthesizedFrozenMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))(alias);
const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(alias);
const frozenMixedBracketedCallableKeys = Object.freeze(globalThis.Reflect["ownKeys"])(alias);
const frozenBracketedCallableKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(alias);
const parenthesizedFrozenBracketedCallableKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(alias);
const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(alias);
const nullishKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);
const logicalAndKeys = Object.freeze((true && Reflect.ownKeys))(obj);
const logicalOrKeys = Object.freeze((false || Reflect.ownKeys))(alias);
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  aliasKeys.length !== 4 ||
  aliasKeys[0] !== '1' ||
  aliasKeys[1] !== '2' ||
  aliasKeys[2] !== 'b' ||
  aliasKeys[3] !== 'a' ||
  globalKeys.length !== 4 ||
  globalKeys[0] !== '1' ||
  globalKeys[1] !== '2' ||
  globalKeys[2] !== 'b' ||
  globalKeys[3] !== 'a' ||
  mixedRootKeys.length !== 4 ||
  mixedRootKeys[0] !== '1' ||
  mixedRootKeys[1] !== '2' ||
  mixedRootKeys[2] !== 'b' ||
  mixedRootKeys[3] !== 'a' ||
  mixedBracketedRootKeys.length !== 4 ||
  mixedBracketedRootKeys[0] !== '1' ||
  mixedBracketedRootKeys[1] !== '2' ||
  mixedBracketedRootKeys[2] !== 'b' ||
  mixedBracketedRootKeys[3] !== 'a' ||
  mixedSingleQuotedRootKeys.length !== 4 ||
  mixedSingleQuotedRootKeys[0] !== '1' ||
  mixedSingleQuotedRootKeys[1] !== '2' ||
  mixedSingleQuotedRootKeys[2] !== 'b' ||
  mixedSingleQuotedRootKeys[3] !== 'a' ||
  mixedKeys.length !== 4 ||
  mixedKeys[0] !== '1' ||
  mixedKeys[1] !== '2' ||
  mixedKeys[2] !== 'b' ||
  mixedKeys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a' ||
  fullyBracketedKeys.length !== 4 ||
  fullyBracketedKeys[0] !== '1' ||
  fullyBracketedKeys[1] !== '2' ||
  fullyBracketedKeys[2] !== 'b' ||
  fullyBracketedKeys[3] !== 'a' ||
  frozenBracketedKeys.length !== 4 ||
  frozenMixedRootKeys.length !== 4 ||
  parenthesizedFrozenBracketedRootKeys.length !== 4 ||
  frozenMixedBracketedCallableKeys.length !== 4 ||
  frozenBracketedCallableKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedKeys[0] !== '1' ||
  parenthesizedFrozenSingleQuotedKeys[1] !== '2' ||
  parenthesizedFrozenSingleQuotedKeys[2] !== 'b' ||
  parenthesizedFrozenSingleQuotedKeys[3] !== 'a' ||
  parenthesizedFrozenMixedKeys.length !== 4 ||
  parenthesizedFrozenMixedKeys[0] !== '1' ||
  parenthesizedFrozenMixedKeys[1] !== '2' ||
  parenthesizedFrozenMixedKeys[2] !== 'b' ||
  parenthesizedFrozenMixedKeys[3] !== 'a' ||
  parenthesizedFrozenMixedRootKeys.length !== 4 ||
  parenthesizedFrozenMixedRootKeys[0] !== '1' ||
  parenthesizedFrozenMixedRootKeys[1] !== '2' ||
  parenthesizedFrozenMixedRootKeys[2] !== 'b' ||
  parenthesizedFrozenMixedRootKeys[3] !== 'a' ||
  parenthesizedFrozenBracketedCallableKeys.length !== 4 ||
  frozenBracketedKeys[0] !== '1' ||
  frozenBracketedKeys[1] !== '2' ||
  frozenBracketedKeys[2] !== 'b' ||
  frozenBracketedKeys[3] !== 'a' ||
  frozenCallableKeys.length !== 4 ||
  frozenCallableKeys[0] !== '1' ||
  frozenCallableKeys[1] !== '2' ||
  frozenCallableKeys[2] !== 'b' ||
  frozenCallableKeys[3] !== 'a' ||
  nullishKeys.length !== 4 ||
  logicalAndKeys.length !== 4 ||
  logicalOrKeys.length !== 4
) {
  throw new Error('unexpected Reflect.ownKeys ordering');
}
for (const item of keys) { console.log(item); }
for (const item of aliasKeys) { console.log(item); }
for (const item of globalKeys) { console.log(item); }
for (const item of mixedKeys) { console.log(item); }
for (const item of bracketedKeys) { console.log(item); }
for (const item of frozenBracketedKeys) { console.log(item); }
for await (const item of keys) { console.log(item); }
for await (const item of aliasKeys) { console.log(item); }
for await (const item of globalKeys) { console.log(item); }
for await (const item of mixedKeys) { console.log(item); }
for await (const item of bracketedKeys) { console.log(item); }
for await (const item of frozenBracketedKeys) { console.log(item); }
for (const item of nullishKeys) { console.log(item); }
for (const item of logicalAndKeys) { console.log(item); }
for (const item of logicalOrKeys) { console.log(item); }
let breakContinueCount = 0;
for (const item of Reflect.ownKeys(obj)) {
  if (item === '1') {
    continue;
  }
  breakContinueCount += 1;
  break;
}
if (breakContinueCount !== 1) {
  throw new Error('unexpected Reflect.ownKeys break/continue semantics');
}
console.log(keys.length);
"#
}

fn browser_runtime_reflect_own_keys_test_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const alias = obj;
const keys = Reflect.ownKeys(obj);
const aliasKeys = Reflect.ownKeys(alias);
const globalKeys = globalThis.Reflect.ownKeys(obj);
const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])(alias);
const mixedKeys = globalThis.Reflect["ownKeys"](alias);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const frozenBracketedKeys = globalThis["Reflect"]["ownKeys"](alias);
const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)(alias);
const parenthesizedFrozenBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(alias);
const parenthesizedFrozenSingleQuotedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(alias);
const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(alias);
const parenthesizedFrozenMixedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(alias);
const parenthesizedFrozenMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))(alias);
const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(alias);
const frozenMixedBracketedCallableKeys = Object.freeze(globalThis.Reflect["ownKeys"])(alias);
const frozenBracketedCallableKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(alias);
const parenthesizedFrozenBracketedCallableKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(alias);
const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(alias);
const nullishKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);
const logicalAndKeys = Object.freeze((true && Reflect.ownKeys))(obj);
const logicalOrKeys = Object.freeze((false || Reflect.ownKeys))(alias);
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  aliasKeys.length !== 4 ||
  aliasKeys[0] !== '1' ||
  aliasKeys[1] !== '2' ||
  aliasKeys[2] !== 'b' ||
  aliasKeys[3] !== 'a' ||
  globalKeys.length !== 4 ||
  globalKeys[0] !== '1' ||
  globalKeys[1] !== '2' ||
  globalKeys[2] !== 'b' ||
  globalKeys[3] !== 'a' ||
  mixedRootKeys.length !== 4 ||
  mixedRootKeys[0] !== '1' ||
  mixedRootKeys[1] !== '2' ||
  mixedRootKeys[2] !== 'b' ||
  mixedRootKeys[3] !== 'a' ||
  mixedKeys.length !== 4 ||
  mixedKeys[0] !== '1' ||
  mixedKeys[1] !== '2' ||
  mixedKeys[2] !== 'b' ||
  mixedKeys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a' ||
  fullyBracketedKeys.length !== 4 ||
  fullyBracketedKeys[0] !== '1' ||
  fullyBracketedKeys[1] !== '2' ||
  fullyBracketedKeys[2] !== 'b' ||
  fullyBracketedKeys[3] !== 'a' ||
  frozenBracketedKeys.length !== 4 ||
  frozenMixedRootKeys.length !== 4 ||
  parenthesizedFrozenBracketedRootKeys.length !== 4 ||
  frozenMixedBracketedCallableKeys.length !== 4 ||
  frozenBracketedCallableKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedKeys[0] !== '1' ||
  parenthesizedFrozenSingleQuotedKeys[1] !== '2' ||
  parenthesizedFrozenSingleQuotedKeys[2] !== 'b' ||
  parenthesizedFrozenSingleQuotedKeys[3] !== 'a' ||
  parenthesizedFrozenMixedKeys.length !== 4 ||
  parenthesizedFrozenMixedKeys[0] !== '1' ||
  parenthesizedFrozenMixedKeys[1] !== '2' ||
  parenthesizedFrozenMixedKeys[2] !== 'b' ||
  parenthesizedFrozenMixedKeys[3] !== 'a' ||
  parenthesizedFrozenMixedRootKeys.length !== 4 ||
  parenthesizedFrozenMixedRootKeys[0] !== '1' ||
  parenthesizedFrozenMixedRootKeys[1] !== '2' ||
  parenthesizedFrozenMixedRootKeys[2] !== 'b' ||
  parenthesizedFrozenMixedRootKeys[3] !== 'a' ||
  parenthesizedFrozenBracketedCallableKeys.length !== 4 ||
  frozenBracketedKeys[0] !== '1' ||
  frozenBracketedKeys[1] !== '2' ||
  frozenBracketedKeys[2] !== 'b' ||
  frozenBracketedKeys[3] !== 'a' ||
  frozenCallableKeys.length !== 4 ||
  frozenCallableKeys[0] !== '1' ||
  frozenCallableKeys[1] !== '2' ||
  frozenCallableKeys[2] !== 'b' ||
  frozenCallableKeys[3] !== 'a' ||
  nullishKeys.length !== 4 ||
  logicalAndKeys.length !== 4 ||
  logicalOrKeys.length !== 4
) {
  throw new Error('unexpected Reflect.ownKeys ordering');
}
for (const item of keys) { console.log(item); }
for (const item of aliasKeys) { console.log(item); }
for (const item of globalKeys) { console.log(item); }
for (const item of mixedKeys) { console.log(item); }
for (const item of bracketedKeys) { console.log(item); }
for (const item of frozenBracketedKeys) { console.log(item); }
for await (const item of keys) { console.log(item); }
for await (const item of aliasKeys) { console.log(item); }
for await (const item of globalKeys) { console.log(item); }
for await (const item of mixedKeys) { console.log(item); }
for await (const item of bracketedKeys) { console.log(item); }
for await (const item of frozenBracketedKeys) { console.log(item); }
for (const item of nullishKeys) { console.log(item); }
for (const item of logicalAndKeys) { console.log(item); }
for (const item of logicalOrKeys) { console.log(item); }
let breakContinueCount = 0;
for (const item of Reflect.ownKeys(obj)) {
  if (item === '1') {
    continue;
  }
  breakContinueCount += 1;
  break;
}
if (breakContinueCount !== 1) {
  throw new Error('unexpected Reflect.ownKeys break/continue semantics');
}
Kali.test('browser runtime reflect ownKeys', () => {});
"#
}

fn browser_runtime_integer_like_object_enumeration_test_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw new Error('unexpected numeric-key ordering');
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
Kali.test('browser runtime smoke', () => {});
"#
}

fn object_enumeration_overwrite_ordering_source() -> &'static str {
    r#"const obj = { "a": 1, "b": 2 };
obj["a"] = 3;
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 3 ||
  values[1] !== 2
) {
  throw 'unexpected overwrite ordering';
}
const reinsertion = { "a": 1, "b": 2, "c": 3 };
delete reinsertion.b;
reinsertion.b = 4;
const reinsertionKeys = Object.keys(reinsertion);
const reinsertionEntries = Object.entries(reinsertion);
const reinsertionValues = Object.values(reinsertion);
if (
  reinsertionKeys.length !== 3 ||
  reinsertionKeys[0] !== 'a' ||
  reinsertionKeys[1] !== 'c' ||
  reinsertionKeys[2] !== 'b' ||
  reinsertionEntries.length !== 3 ||
  reinsertionEntries[0][0] !== 'a' ||
  reinsertionEntries[0][1] !== 1 ||
  reinsertionEntries[1][0] !== 'c' ||
  reinsertionEntries[1][1] !== 3 ||
  reinsertionEntries[2][0] !== 'b' ||
  reinsertionEntries[2][1] !== 4 ||
  reinsertionValues.length !== 3 ||
  reinsertionValues[0] !== 1 ||
  reinsertionValues[1] !== 3 ||
  reinsertionValues[2] !== 4
) {
  throw 'unexpected delete-reinsert ordering';
}
console.log(values.length);
"#
}

fn browser_bundle_object_enumeration_overwrite_ordering_source() -> &'static str {
    r##"// kali-tree-shake: enumSmoke
async function enumSmoke(left, right) {
  const obj = Object.create(null);
  obj["a"] = 1;
  obj["b"] = 2;
  obj["a"] = 3;
  const keys = Object.keys(obj);
  const entries = Object.entries(obj);
  const values = Object.values(obj);
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2]]);
  const fromEntriesKeys = Object.keys(fromEntries);
  const fromEntriesEntries = Object.entries(fromEntries);
  const fromEntriesValues = Object.values(fromEntries);
  const wrappedEntries = ([["b", 1], ["a", 2]]);
  const wrappedFromEntries = Object.fromEntries(wrappedEntries);
  const wrappedFromEntriesKeys = Object.keys(wrappedFromEntries);
  const consumeArray = (items, value) => items[0] + items[1] + value;
  const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
  const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
  if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
    throw new Error('unexpected array literal arguments');
  }
  if (
    keys.length !== 2 ||
    keys[0] !== 'a' ||
    keys[1] !== 'b' ||
    entries.length !== 2 ||
    entries[0][0] !== 'a' ||
    entries[0][1] !== 3 ||
    entries[1][0] !== 'b' ||
    entries[1][1] !== 2 ||
    values.length !== 2 ||
    values[0] !== 3 ||
    values[1] !== 2 ||
    fromEntriesKeys.length !== 2 ||
    fromEntriesKeys[0] !== 'b' ||
    fromEntriesKeys[1] !== 'a' ||
    fromEntriesEntries.length !== 2 ||
    fromEntriesEntries[0][0] !== 'b' ||
    fromEntriesEntries[0][1] !== 1 ||
    fromEntriesEntries[1][0] !== 'a' ||
    fromEntriesEntries[1][1] !== 2 ||
    fromEntriesValues.length !== 2 ||
    fromEntriesValues[0] !== 1 ||
    fromEntriesValues[1] !== 2 ||
    wrappedFromEntriesKeys.length !== 2 ||
    wrappedFromEntriesKeys[0] !== 'b' ||
    wrappedFromEntriesKeys[1] !== 'a'
  ) {
    throw new Error('unexpected overwrite ordering');
  }
  const reinsertion = Object.create(null);
  reinsertion["a"] = 1;
  reinsertion["b"] = 2;
  reinsertion["c"] = 3;
  delete reinsertion.b;
  reinsertion.b = 4;
  const reinsertionKeys = Object.keys(reinsertion);
  const reinsertionEntries = Object.entries(reinsertion);
  const reinsertionValues = Object.values(reinsertion);
  if (
    reinsertionKeys.length !== 3 ||
    reinsertionKeys[0] !== 'a' ||
    reinsertionKeys[1] !== 'c' ||
    reinsertionKeys[2] !== 'b' ||
    reinsertionEntries.length !== 3 ||
    reinsertionEntries[0][0] !== 'a' ||
    reinsertionEntries[0][1] !== 1 ||
    reinsertionEntries[1][0] !== 'c' ||
    reinsertionEntries[1][1] !== 3 ||
    reinsertionEntries[2][0] !== 'b' ||
    reinsertionEntries[2][1] !== 4 ||
    reinsertionValues.length !== 3 ||
    reinsertionValues[0] !== 1 ||
    reinsertionValues[1] !== 3 ||
    reinsertionValues[2] !== 4
  ) {
    throw new Error('unexpected delete-reinsert ordering');
  }
  return left - left + right - right;
}
"##
}

fn browser_bundle_integer_like_object_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: enumSmoke
async function enumSmoke(left, right) {
  const obj = Object.create(null);
  obj["b"] = 1;
  obj["2"] = 2;
  obj["a"] = 3;
  obj["1"] = 4;
  const keys = Object.keys(obj);
  const entries = Object.entries(obj);
  const values = Object.values(obj);
  const consumeArray = (items, value) => items[0] + items[1] + value;
  const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
  const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
  if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
    throw new Error('unexpected array literal arguments');
  }
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3 ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3
  ) {
    throw new Error('unexpected numeric-key ordering');
  }
  return left - left + right - right;
}
"##
}

fn browser_bundle_string_primitive_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: stringPrimitiveSmoke
async function stringPrimitiveSmoke(left, right) {
  const stringKeys = Object.keys('ab');
  const globalThisStringKeys = globalThis.Object["keys"]('ab');
  const bracketedGlobalThisStringKeys = globalThis["Object"].keys('ab');
  const stringEntries = Object.entries('ab');
  const globalThisStringEntries = globalThis.Object["entries"]('ab');
  const bracketedGlobalThisStringEntries = globalThis["Object"].entries('ab');
  const stringValues = Object.values('ab');
  const globalThisStringValues = globalThis.Object["values"]('ab');
  const fullyBracketedGlobalThisStringValues = globalThis["Object"]["values"]('ab');
  const consumeArray = (items, value) => items[0] + items[1] + value;
  const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
  const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
  if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
    throw new Error('unexpected array literal arguments');
  }
  if (
    stringKeys.length !== 2 ||
    stringKeys[0] !== '0' ||
    stringKeys[1] !== '1' ||
    globalThisStringKeys.length !== 2 ||
    globalThisStringKeys[0] !== '0' ||
    globalThisStringKeys[1] !== '1' ||
    bracketedGlobalThisStringKeys.length !== 2 ||
    bracketedGlobalThisStringKeys[0] !== '0' ||
    bracketedGlobalThisStringKeys[1] !== '1' ||
    stringEntries.length !== 2 ||
    stringEntries[0][0] !== '0' ||
    stringEntries[0][1] !== 'a' ||
    stringEntries[1][0] !== '1' ||
    stringEntries[1][1] !== 'b' ||
    globalThisStringEntries.length !== 2 ||
    globalThisStringEntries[0][0] !== '0' ||
    globalThisStringEntries[0][1] !== 'a' ||
    globalThisStringEntries[1][0] !== '1' ||
    globalThisStringEntries[1][1] !== 'b' ||
    bracketedGlobalThisStringEntries.length !== 2 ||
    bracketedGlobalThisStringEntries[0][0] !== '0' ||
    bracketedGlobalThisStringEntries[0][1] !== 'a' ||
    bracketedGlobalThisStringEntries[1][0] !== '1' ||
    bracketedGlobalThisStringEntries[1][1] !== 'b' ||
    stringValues.length !== 2 ||
    stringValues[0] !== 'a' ||
    stringValues[1] !== 'b' ||
    globalThisStringValues.length !== 2 ||
    globalThisStringValues[0] !== 'a' ||
    globalThisStringValues[1] !== 'b' ||
    fullyBracketedGlobalThisStringValues.length !== 2 ||
    fullyBracketedGlobalThisStringValues[0] !== 'a' ||
    fullyBracketedGlobalThisStringValues[1] !== 'b'
  ) {
    throw new Error('unexpected string primitive enumeration');
  }
  return left - left + right - right;
}
"##
}

fn write_browser_runtime_package_fixture(package_dir: &Path, package_name: &str) {
    fs::create_dir_all(package_dir).expect("create browser package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{package_name}",
  "version": "1.0.0",
  "main": "index.js",
  "browser": "index.browser.js"
}}"#,
        ),
    )
    .expect("write browser package json");
    fs::write(
        package_dir.join("index.js"),
        "export default function describe() { return 1; }\n",
    )
    .expect("write browser package main entry");
    fs::write(
        package_dir.join("index.browser.js"),
        "export default function describe() { return 0; }\n",
    )
    .expect("write browser package browser entry");
}

fn write_browser_runtime_exports_package_fixture(package_dir: &Path, package_name: &str) {
    fs::create_dir_all(package_dir).expect("create browser package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{package_name}",
  "version": "1.0.0",
  "exports": {{
    ".": {{
      "browser": "./index.browser.js",
      "default": "./index.js"
    }}
  }}
}}"#,
        ),
    )
    .expect("write browser package json");
    fs::write(
        package_dir.join("index.js"),
        "export default function describe() { return 1; }\n",
    )
    .expect("write browser package main entry");
    fs::write(
        package_dir.join("index.browser.js"),
        "export default function describe() { return 0; }\n",
    )
    .expect("write browser package browser entry");
}

/// Runs the bundle's `export_name(1n, 2n)` through the node harness and pins
/// the result. `expected` is the decimal digits of the expected BigInt (i64
/// exports surface as BigInt through the raw wasm-export ABI). Pinning the
/// real source value proves the exported body actually executed; a blanket
/// `0n` pin is indistinguishable from a dead stub (b5c085401 precedent).
fn assert_browser_bundle_executes_with_result(
    bundle_root: &Path,
    export_name: &str,
    expected: &str,
) {
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
if (result !== {expected}n) {{
  throw new Error(`unexpected result ${{result}}`);
}}
console.log(String(result));
"#,
            export_name = export_name,
            expected = expected,
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
    assert!(stdout.contains(expected), "stdout: {stdout}");
}

/// Callers whose fixtures genuinely `return 0n;`.
fn assert_browser_bundle_executes(bundle_root: &Path, export_name: &str) {
    assert_browser_bundle_executes_with_result(bundle_root, export_name, "0");
}

fn assert_browser_bundle_promise_all_sequencing(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_promise_all_sequencing_source()).expect("write source");

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
        let envelope = parse_json_stdout(&output);
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "promiseAllSmoke");
}

fn assert_browser_bundle_unary_prefix_semantics(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_unary_prefix_semantics_source()).expect("write source");

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
        let envelope = parse_json_stdout(&output);
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "unaryPrefixSmoke");
}

fn assert_browser_bundle_string_primitive_enumeration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_string_primitive_enumeration_source(),
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
        "stdout: {}
stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope = parse_json_stdout(&output);
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
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

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "stringPrimitiveSmoke");
}

/// Every caller writes the chunk fixture `export function lazyValue() { return 7; }`,
/// so the harness expects `7n` (i64 chunk results surface as BigInt through the raw
/// wasm-export ABI). Expecting the real source value proves the chunk body actually
/// executed; the old `0n` pin was indistinguishable from a stub returning the i64 default.
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
if (value !== 7n) {{
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
    assert!(stdout.contains('7'), "stdout: {stdout}");
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

fn write_threaded_policy(path: &Path) {
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
    "maxThreads": 1
  }
}"#,
    )
    .expect("write threaded policy");
}

fn write_invalid_policy_schema(path: &Path) {
    fs::write(
        path,
        r#"{
  "schemaVersion": 1,
  "unknown": true
}"#,
    )
    .expect("write invalid policy");
}

fn assert_browser_requested_unary_prefix_semantics(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        unary_prefix_semantics_source(command == "test"),
    )
    .expect("write source");

    let mut cmd = Command::new(kali_bin());
    cmd.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cmd.arg("--output").arg("json");
    }
    let output = cmd
        .arg(command)
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

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
    }
}

fn assert_browser_requested_unary_prefix_semantics_with_inherited_browser_api_surface(
    command: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    write_browser_api_surface_manifest(dir.path());

    for ext in ["ts", "js"] {
        let source_path = dir
            .path()
            .join(format!("browser-unary-prefix-inherited-{command}.{ext}"));
        fs::write(
            &source_path,
            unary_prefix_semantics_source(command == "test"),
        )
        .expect("write source");

        let mut cmd = Command::new(kali_bin());
        cmd.current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        if json_output {
            cmd.arg("--output").arg("json");
        }
        let output = cmd
            .arg(command)
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

        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], true);
            assert_eq!(json["payload"]["hostContract"], "browser-requested");
            assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
            if command == "run" {
                assert_eq!(json["exitCode"], 0);
                assert_eq!(json["payload"]["exitCode"], 0);
            } else {
                assert_eq!(json["payload"]["total"], 1);
                assert_eq!(json["payload"]["passed"], 1);
                assert_eq!(json["payload"]["failed"], 0);
            }
        }
    }
}

fn browser_runtime_array_from_source() -> String {
    let source = kali_common::array_from_alias_inventory_source();
    format!(
        "const values = [1, 2];\n{}\n",
        source
            .trim_end_matches(';')
            .split("; ")
            .map(|alias| format!(
                "for (const value of {alias}(values)) {{\n  console.log(value);\n}}"
            ))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn assert_json_run_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_input_when_a_browser_harness_command_is_configured(
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, browser_runtime_reflect_own_keys_source()).expect("write source");
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\nb\na\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

fn assert_json_test_supports_reflect_own_keys_direct_iteration_when_browser_api_surface_is_inherited_in_input_when_a_browser_harness_command_is_configured(
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, browser_runtime_reflect_own_keys_test_source()).expect("write source");
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1\n2\nb\na\n"),
        "json: {json}"
    );
    assert_eq!(json["stderr"], "");
}

fn assert_browser_harness_supports_math_hypot_semantics(
    command: &str,
    source_name: &str,
    expected_stdout_fragment: &str,
    assert_payload_details: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, "console.log(Math.hypot(3, 4));\n").expect("write source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
    }
    if assert_payload_details {
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains(expected_stdout_fragment),
        "json: {json}"
    );
}

#[cfg(unix)]
fn shell_quote_path(path: &Path) -> String {
    let rendered = path.to_string_lossy().replace('\'', "'\\''");
    format!("'{}'", rendered)
}

#[cfg(unix)]
fn browser_entrypoint_smoke(
    command_name: &str,
    source_name: &str,
    source_contents: &str,
    stdout_marker: &str,
    browser_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, source_contents).expect("write source");

    let browser = dir.path().join(browser_name);
    symlink("/bin/sh", &browser).expect("link browser executable shim to /bin/sh");

    let browser_log = dir.path().join(format!("{browser_name}-args.txt"));
    let command = format!(
        r#"{} -c 'printf "%s\n" "$@" > "$KALI_BROWSER_SHIM_LOG"; printf "{{\"args\":[],\"tests\":[],\"testsFailed\":0}}\n" > "$KALI_BROWSER_HARNESS_SUMMARY_FILE"; printf "{}\n"; exit 0' _ --headless"#,
        shell_quote_path(&browser),
        stdout_marker
    );

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", command)
        .env("KALI_BROWSER_SHIM_LOG", &browser_log)
        .arg("--output")
        .arg("json")
        .arg(command_name)
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command_name);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .is_some_and(|stdout| stdout.contains(stdout_marker)),
        "json: {json}"
    );

    let browser_args = fs::read_to_string(&browser_log).expect("browser shim args");
    assert!(browser_args.contains("--headless"), "args: {browser_args}");
    assert!(browser_args.contains("file://"), "args: {browser_args}");
    assert!(
        browser_args.contains("browser-runtime.html"),
        "args: {browser_args}"
    );
}

#[cfg(unix)]
fn run_browser_entrypoint_smoke(browser_name: &str) {
    browser_entrypoint_smoke(
        "run",
        "main.ts",
        "console.log('browser run');",
        "browser run",
        browser_name,
    );
}

#[cfg(unix)]
fn test_browser_entrypoint_smoke(browser_name: &str) {
    browser_entrypoint_smoke(
        "test",
        "main.test.ts",
        "console.log('browser test');\nKali.test('browser test', () => { 1 + 1; });",
        "browser test",
        browser_name,
    );
}

fn assert_literal_string_dynamic_import_runtime_support(extension: &str, use_json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        dir.path().join(format!("lazy.{extension}")),
        "export const value = 7;",
    )
    .expect("write lazy chunk");
    fs::write(
        &source_path,
        format!(
            r#"async function main() {{
  await import("./lazy.{extension}");
  console.log("main loaded");
}}
main();
"#,
            extension = extension,
        ),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if use_json_output {
        command.arg("--output").arg("json");
    }
    let output = command
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

    if use_json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("main loaded"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main loaded"), "stdout: {stdout}");
}

fn browser_requested_web_crypto_get_random_values_source() -> &'static str {
    r#"const bytes = new globalThis["Uint8Array"](8);
const result = crypto.getRandomValues(bytes);
if (result !== bytes) {
  throw new Error('crypto.getRandomValues should return the provided buffer');
}
if (bytes.length !== 8 || bytes.byteLength !== 8) {
  throw new Error(`unexpected buffer length ${bytes.length}/${bytes.byteLength}`);
}
console.log('ok');
"#
}

fn assert_browser_requested_web_crypto_get_random_values_when_browser_api_surface_is_inherited(
    command: &str,
    filename: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_requested_web_crypto_get_random_values_source(),
    )
    .expect("write source");
    write_browser_api_surface_manifest(dir.path());

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
    }
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("ok"),
        "json: {json}"
    );
}

fn assert_browser_requested_promise_all_sequencing(
    command: &str,
    filename: &str,
    json_output: bool,
    inherited_browser_api_surface: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, promise_all_sequencing_source()).expect("write source");
    if inherited_browser_api_surface {
        write_browser_api_surface_manifest(dir.path());
    }

    let mut command_line = Command::new(kali_bin());
    command_line.current_dir(dir.path());
    command_line.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    command_line
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path);

    let output = command_line.output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else if command == "test" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_run_supports_bigint_binary_semantics(
    extension: &str,
    expression: &str,
    expected_stdout: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, format!("console.log({expression});\n")).expect("write source");

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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), expected_stdout, "stdout: {stdout}");
}

fn assert_test_supports_bigint_binary_semantics(
    extension: &str,
    expression: &str,
    expected_stdout: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, format!("console.log({expression});\n")).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(
        stdout.contains(expected_stdout) && stdout.contains("ok 1"),
        "stdout: {stdout}"
    );
}

fn assert_browser_bundle_supports_bigint_binary_semantics(
    extension: &str,
    expression: &str,
    expected_value_source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("app.{extension}"));
    fs::write(
        &source_path,
        format!(
            "// kali-tree-shake: bigintSmoke\nfunction bigintSmoke() {{\n  const result = {expression};\n  if (result !== {expected_value_source}) {{\n    throw new Error('unexpected bigint');\n  }}\n  return 0n;\n}}\n"
        ),
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
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    assert_browser_bundle_executes(&bundle_dir, "bigintSmoke");
}

fn assert_object_string_primitive_enumeration_semantics(
    command: &str,
    filename: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

fn object_string_primitive_enumeration_semantics_source() -> &'static str {
    r#"const stringKeys = Object.keys('ab');
const globalThisStringKeys = globalThis.Object["keys"]('ab');
const bracketedGlobalThisStringKeys = globalThis["Object"].keys('ab');
const stringEntries = Object.entries('ab');
const globalThisStringEntries = globalThis.Object["entries"]('ab');
const bracketedGlobalThisStringEntries = globalThis["Object"].entries('ab');
const stringValues = Object.values('ab');
const conditionalStringValues = (true ? Object.values : Object.values)('ab');
const globalThisStringValues = globalThis.Object["values"]('ab');
const conditionalGlobalThisStringValues = (true ? globalThis.Object["values"] : globalThis.Object["values"])('ab');
const fullyBracketedGlobalThisStringValues = globalThis["Object"]["values"]('ab');
const conditionalFullyBracketedGlobalThisStringValues = (true ? globalThis["Object"]["values"] : globalThis["Object"]["values"])('ab');
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  stringKeys.length !== 2 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  globalThisStringKeys.length !== 2 ||
  globalThisStringKeys[0] !== '0' ||
  globalThisStringKeys[1] !== '1' ||
  bracketedGlobalThisStringKeys.length !== 2 ||
  bracketedGlobalThisStringKeys[0] !== '0' ||
  bracketedGlobalThisStringKeys[1] !== '1' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  globalThisStringEntries.length !== 2 ||
  globalThisStringEntries[0][0] !== '0' ||
  globalThisStringEntries[0][1] !== 'a' ||
  globalThisStringEntries[1][0] !== '1' ||
  globalThisStringEntries[1][1] !== 'b' ||
  bracketedGlobalThisStringEntries.length !== 2 ||
  bracketedGlobalThisStringEntries[0][0] !== '0' ||
  bracketedGlobalThisStringEntries[0][1] !== 'a' ||
  bracketedGlobalThisStringEntries[1][0] !== '1' ||
  bracketedGlobalThisStringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b' ||
  conditionalStringValues.length !== 2 ||
  conditionalStringValues[0] !== 'a' ||
  conditionalStringValues[1] !== 'b' ||
  globalThisStringValues.length !== 2 ||
  globalThisStringValues[0] !== 'a' ||
  globalThisStringValues[1] !== 'b' ||
  conditionalGlobalThisStringValues.length !== 2 ||
  conditionalGlobalThisStringValues[0] !== 'a' ||
  conditionalGlobalThisStringValues[1] !== 'b' ||
  fullyBracketedGlobalThisStringValues.length !== 2 ||
  fullyBracketedGlobalThisStringValues[0] !== 'a' ||
  fullyBracketedGlobalThisStringValues[1] !== 'b' ||
  conditionalFullyBracketedGlobalThisStringValues.length !== 2 ||
  conditionalFullyBracketedGlobalThisStringValues[0] !== 'a' ||
  conditionalFullyBracketedGlobalThisStringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
console.log(stringKeys.length);
"#
}

fn object_string_primitive_enumeration_semantics_test_source() -> &'static str {
    r#"const stringKeys = Object.keys('ab');
const globalThisStringKeys = globalThis.Object["keys"]('ab');
const bracketedGlobalThisStringKeys = globalThis["Object"].keys('ab');
const stringEntries = Object.entries('ab');
const globalThisStringEntries = globalThis.Object["entries"]('ab');
const bracketedGlobalThisStringEntries = globalThis["Object"].entries('ab');
const stringValues = Object.values('ab');
const conditionalStringValues = (true ? Object.values : Object.values)('ab');
const globalThisStringValues = globalThis.Object["values"]('ab');
const conditionalGlobalThisStringValues = (true ? globalThis.Object["values"] : globalThis.Object["values"])('ab');
const fullyBracketedGlobalThisStringValues = globalThis["Object"]["values"]('ab');
const conditionalFullyBracketedGlobalThisStringValues = (true ? globalThis["Object"]["values"] : globalThis["Object"]["values"])('ab');
if (
  stringKeys.length !== 2 ||
  stringKeys[0] !== '0' ||
  stringKeys[1] !== '1' ||
  globalThisStringKeys.length !== 2 ||
  globalThisStringKeys[0] !== '0' ||
  globalThisStringKeys[1] !== '1' ||
  bracketedGlobalThisStringKeys.length !== 2 ||
  bracketedGlobalThisStringKeys[0] !== '0' ||
  bracketedGlobalThisStringKeys[1] !== '1' ||
  stringEntries.length !== 2 ||
  stringEntries[0][0] !== '0' ||
  stringEntries[0][1] !== 'a' ||
  stringEntries[1][0] !== '1' ||
  stringEntries[1][1] !== 'b' ||
  globalThisStringEntries.length !== 2 ||
  globalThisStringEntries[0][0] !== '0' ||
  globalThisStringEntries[0][1] !== 'a' ||
  globalThisStringEntries[1][0] !== '1' ||
  globalThisStringEntries[1][1] !== 'b' ||
  bracketedGlobalThisStringEntries.length !== 2 ||
  bracketedGlobalThisStringEntries[0][0] !== '0' ||
  bracketedGlobalThisStringEntries[0][1] !== 'a' ||
  bracketedGlobalThisStringEntries[1][0] !== '1' ||
  bracketedGlobalThisStringEntries[1][1] !== 'b' ||
  stringValues.length !== 2 ||
  stringValues[0] !== 'a' ||
  stringValues[1] !== 'b' ||
  conditionalStringValues.length !== 2 ||
  conditionalStringValues[0] !== 'a' ||
  conditionalStringValues[1] !== 'b' ||
  globalThisStringValues.length !== 2 ||
  globalThisStringValues[0] !== 'a' ||
  globalThisStringValues[1] !== 'b' ||
  conditionalGlobalThisStringValues.length !== 2 ||
  conditionalGlobalThisStringValues[0] !== 'a' ||
  conditionalGlobalThisStringValues[1] !== 'b' ||
  fullyBracketedGlobalThisStringValues.length !== 2 ||
  fullyBracketedGlobalThisStringValues[0] !== 'a' ||
  fullyBracketedGlobalThisStringValues[1] !== 'b' ||
  conditionalFullyBracketedGlobalThisStringValues.length !== 2 ||
  conditionalFullyBracketedGlobalThisStringValues[0] !== 'a' ||
  conditionalFullyBracketedGlobalThisStringValues[1] !== 'b'
) {
  throw 'unexpected string primitive enumeration';
}
console.log(stringKeys.length);
Kali.test('string primitive enumeration', () => {});
"#
}

fn object_enumeration_semantics_source() -> &'static str {
    r#"const obj = { "a": 1, "b": 2 };
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
const arrayLiteralSecond = consumeArray([1n, 2n, 3n], 1n);
if (arrayLiteralFirst !== 4n || arrayLiteralSecond !== 4n) {
  throw 'unexpected array literal arguments';
}
if (
  keys.length !== 2 ||
  keys[0] !== 'a' ||
  keys[1] !== 'b' ||
  entries.length !== 2 ||
  entries[0][0] !== 'a' ||
  entries[0][1] !== 1 ||
  entries[1][0] !== 'b' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 1 ||
  values[1] !== 2
) {
  throw 'unexpected enumeration';
}
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#
}

fn object_from_entries_enumeration_source() -> &'static str {
    r#"function assertFromEntriesShape(fromEntries) {
  const keys = Object.keys(fromEntries);
  const entries = Object.entries(fromEntries);
  const values = Object.values(fromEntries);
  if (
    JSON.stringify(keys) !== '["b","a"]' ||
    JSON.stringify(entries) !== '[["b",1],["a",2]]' ||
    JSON.stringify(values) !== '[1,2]'
  ) {
    throw new Error('unexpected fromEntries enumeration');
  }
}

const obj = Object.fromEntries([["b", 1], ["a", 2]]);
const dotted = globalThis.Object.fromEntries([["b", 1], ["a", 2]]);
const mixedDotted = globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]);
const mixedBracketed = globalThis["Object"].fromEntries([["b", 1], ["a", 2]]);
const bracketed = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]);
assertFromEntriesShape(obj);
assertFromEntriesShape(dotted);
assertFromEntriesShape(mixedDotted);
assertFromEntriesShape(mixedBracketed);
assertFromEntriesShape(bracketed);
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#
}

fn browser_runtime_object_from_entries_satisfies_source() -> &'static str {
    r#"function assertFromEntriesShape(fromEntries) {
  const keys = Object.keys(fromEntries);
  const entries = Object.entries(fromEntries);
  const values = Object.values(fromEntries);
  if (
    JSON.stringify(keys) !== '["b","a"]' ||
    JSON.stringify(entries) !== '[["b",1],["a",2]]' ||
    JSON.stringify(values) !== '[1,2]'
  ) {
    throw new Error('unexpected fromEntries enumeration');
  }
}

const wrappedEntries = ([["b", 1], ["a", 2]] satisfies unknown);
const obj = Object.fromEntries(wrappedEntries);
const dotted = globalThis.Object.fromEntries(wrappedEntries);
const mixedDotted = globalThis.Object["fromEntries"](wrappedEntries);
const mixedBracketed = globalThis["Object"].fromEntries(wrappedEntries);
const bracketed = globalThis["Object"]["fromEntries"](wrappedEntries);
assertFromEntriesShape(obj);
assertFromEntriesShape(dotted);
assertFromEntriesShape(mixedDotted);
assertFromEntriesShape(mixedBracketed);
assertFromEntriesShape(bracketed);
const keys = Object.keys(obj);
const entries = Object.entries(obj);
const values = Object.values(obj);
console.log(keys.length);
console.log(entries.length);
console.log(values.length);
"#
}

fn browser_runtime_object_from_entries_satisfies_test_source() -> &'static str {
    r#"Kali.test('browser object.fromEntries satisfies', () => {
  function assertFromEntriesShape(fromEntries) {
    const keys = Object.keys(fromEntries);
    const entries = Object.entries(fromEntries);
    const values = Object.values(fromEntries);
    if (
      JSON.stringify(keys) !== '["b","a"]' ||
      JSON.stringify(entries) !== '[["b",1],["a",2]]' ||
      JSON.stringify(values) !== '[1,2]'
    ) {
      throw new Error('unexpected fromEntries enumeration');
    }
  }

  const wrappedEntries = ([["b", 1], ["a", 2]] satisfies unknown);
  const obj = Object.fromEntries(wrappedEntries);
  const dotted = globalThis.Object.fromEntries(wrappedEntries);
  const mixedDotted = globalThis.Object["fromEntries"](wrappedEntries);
  const mixedBracketed = globalThis["Object"].fromEntries(wrappedEntries);
  const bracketed = globalThis["Object"]["fromEntries"](wrappedEntries);
  assertFromEntriesShape(obj);
  assertFromEntriesShape(dotted);
  assertFromEntriesShape(mixedDotted);
  assertFromEntriesShape(mixedBracketed);
  assertFromEntriesShape(bracketed);
  const keys = Object.keys(obj);
  const entries = Object.entries(obj);
  const values = Object.values(obj);
  console.log(keys.length);
  console.log(entries.length);
  console.log(values.length);
});
"#
}

fn browser_runtime_object_from_entries_has_own_source() -> &'static str {
    r#"function main() {
  const frozen = Object.freeze(Object.fromEntries([["b", 1], ["a", 2]]));
  const awaited = frozen;
  const frozenHasOwn = Object.freeze(Object.hasOwn);
  const frozenHasOwnPropertyCall = Object.freeze(Object.prototype.hasOwnProperty.call);
  const frozenOptionalChainHasOwn = Object.freeze(globalThis?.Object.hasOwn);
  const frozenOptionalChainBracketedHasOwn = Object.freeze(globalThis?.Object["hasOwn"]);
  const frozenOptionalChainHasOwnPropertyCall = Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call);
  const frozenOptionalChainBracketedHasOwnPropertyCall = Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]);
  console.log(Object.hasOwn(frozen, "a"));
  console.log(Object.hasOwn(awaited, "a"));
  console.log(Object.prototype.hasOwnProperty.call(frozen, "a"));
  console.log(Object.prototype.hasOwnProperty.call(awaited, "a"));
  console.log(frozenHasOwn(frozen, "a"));
  console.log(frozenHasOwnPropertyCall(frozen, "a"));
  console.log(frozenOptionalChainHasOwn(frozen, "a"));
  console.log(frozenOptionalChainBracketedHasOwn(frozen, "a"));
  console.log(frozenOptionalChainHasOwnPropertyCall(frozen, "a"));
  console.log(frozenOptionalChainBracketedHasOwnPropertyCall(frozen, "a"));
}
main();
"#
}

fn browser_runtime_object_from_entries_has_own_test_source() -> &'static str {
    r#"Kali.test('browser object.fromEntries hasOwn', () => {
    const frozen = Object.freeze(Object.fromEntries([["b", 1], ["a", 2]]));
    const awaited = frozen;
    const frozenHasOwn = Object.freeze(Object.hasOwn);
    const frozenHasOwnPropertyCall = Object.freeze(Object.prototype.hasOwnProperty.call);
    const frozenOptionalChainHasOwn = Object.freeze(globalThis?.Object.hasOwn);
    const frozenOptionalChainBracketedHasOwn = Object.freeze(globalThis?.Object["hasOwn"]);
    const frozenOptionalChainHasOwnPropertyCall = Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call);
    const frozenOptionalChainBracketedHasOwnPropertyCall = Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]);
    console.log(Object.hasOwn(frozen, "a"));
    console.log(Object.hasOwn(awaited, "a"));
    console.log(Object.prototype.hasOwnProperty.call(frozen, "a"));
    console.log(Object.prototype.hasOwnProperty.call(awaited, "a"));
    console.log(frozenHasOwn(frozen, "a"));
    console.log(frozenHasOwnPropertyCall(frozen, "a"));
    console.log(frozenOptionalChainHasOwn(frozen, "a"));
    console.log(frozenOptionalChainBracketedHasOwn(frozen, "a"));
    console.log(frozenOptionalChainHasOwnPropertyCall(frozen, "a"));
    console.log(frozenOptionalChainBracketedHasOwnPropertyCall(frozen, "a"));
});
"#
}
fn browser_runtime_frozen_object_enumeration_spread_source() -> &'static str {
    r#"const frozen = Object.freeze({ "zed": 1, "alpha": 2 });
for (const value of [...globalThis["Object"]["values"](frozen)]) { console.log(value); }
for (const key of [...globalThis.Object["keys"](frozen)]) { console.log(key); }
for (const entry of [...globalThis["Object"].entries(frozen)]) { console.log(entry[0]); console.log(entry[1]); }
const frozenKeys = Reflect.ownKeys(frozen);
const frozenGlobalKeys = globalThis['Reflect']['ownKeys'](frozen);
const frozenCallableValues = Object.freeze(Object.values)(frozen);
const frozenCallableGlobalValues = Object.freeze(globalThis.Object.values)(frozen);
const frozenCallableBracketedValues = Object.freeze(globalThis["Object"]["values"])(frozen);
const bracketRootValues = Object.freeze((globalThis["Object"]))["values"](frozen);
const bracketRootSingleQuotedValues = Object.freeze((globalThis["Object"])['values'])(frozen);
const bracketRootEntries = Object.freeze((globalThis["Object"]))["entries"](frozen);
const bracketRootSingleQuotedEntries = Object.freeze((globalThis["Object"])['entries'])(frozen);
if (
  bracketRootValues.length !== 2 ||
  bracketRootValues[0] !== 1 ||
  bracketRootValues[1] !== 2 ||
  bracketRootSingleQuotedValues.length !== 2 ||
  bracketRootSingleQuotedValues[0] !== 1 ||
  bracketRootSingleQuotedValues[1] !== 2 ||
  bracketRootEntries.length !== 2 ||
  bracketRootEntries[0][0] !== "zed" ||
  bracketRootEntries[0][1] !== 1 ||
  bracketRootEntries[1][0] !== "alpha" ||
  bracketRootEntries[1][1] !== 2 ||
  bracketRootSingleQuotedEntries.length !== 2 ||
  bracketRootSingleQuotedEntries[0][0] !== "zed" ||
  bracketRootSingleQuotedEntries[0][1] !== 1 ||
  bracketRootSingleQuotedEntries[1][0] !== "alpha" ||
  bracketRootSingleQuotedEntries[1][1] !== 2
) {
  throw new Error('unexpected bracket-root object enumeration semantics');
}
const frozenCallableEntries = Object.freeze(Object.entries)(frozen);
const frozenCallableGlobalEntries = Object.freeze(globalThis.Object.entries)(frozen);
const frozenCallableBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(frozen);
const parenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(frozen);
const parenthesizedSingleQuotedBracketedValues = Object.freeze((globalThis['Object'])["values"])(frozen);
const parenthesizedSingleQuotedReceiverBracketedValues = Object.freeze((globalThis['Object'])['values'])(frozen);
const parenthesizedBracketedKeys = Object.freeze((globalThis["Object"]).keys)(frozen);
const parenthesizedSingleQuotedReceiverBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(frozen);
const parenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(frozen);
const parenthesizedSingleQuotedBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(frozen);
const nullishBracketedEntries = Object.freeze((null ?? globalThis["Object"]["entries"]))(frozen);
const logicalAndBracketedEntries = Object.freeze((true && globalThis["Object"]["entries"]))(frozen);
const logicalOrBracketedEntries = Object.freeze((false || globalThis["Object"]["entries"]))(frozen);
const parenthesizedReceiverBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(frozen);
const parenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(frozen);
for (const value of [...parenthesizedBracketedValues]) { console.log(value); }
for (const value of [...parenthesizedSingleQuotedBracketedValues]) { console.log(value); }
for (const value of [...parenthesizedSingleQuotedReceiverBracketedValues]) { console.log(value); }
for (const key of [...parenthesizedBracketedKeys]) { console.log(key); }
for (const entry of [...parenthesizedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...parenthesizedSingleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...frozenCallableEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...frozenCallableGlobalEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...frozenCallableBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...logicalAndBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...logicalOrBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...parenthesizedReceiverBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...parenthesizedSingleQuotedReceiverBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
const frozenCallableKeys = Object.freeze(Reflect.ownKeys)(frozen);
const frozenCallableGlobalKeys = Object.freeze(globalThis.Reflect.ownKeys)(frozen);
const frozenCallableBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])(frozen);
const frozenBracketRootKeys = Object.freeze((globalThis["Object"]))["keys"](frozen);
const frozenSingleQuotedBracketRootKeys = Object.freeze((globalThis["Object"])['keys'])(frozen);
for (const value of [...frozenCallableValues]) { console.log(value); }
for (const value of [...frozenCallableGlobalValues]) { console.log(value); }
for (const value of [...frozenCallableBracketedValues]) { console.log(value); }
for (const key of frozenKeys) { console.log(key); }
for (const key of frozenBracketRootKeys) { console.log(key); }
for (const key of frozenSingleQuotedBracketRootKeys) { console.log(key); }
for (const key of frozenCallableKeys) { console.log(key); }
for (const key of frozenCallableGlobalKeys) { console.log(key); }
for (const key of frozenCallableBracketedKeys) { console.log(key); }
for await (const key of frozenGlobalKeys) { console.log(key); }
"#
}
fn browser_runtime_frozen_object_enumeration_spread_test_source() -> &'static str {
    r#"Kali.test('browser frozen object enumeration spread', () => {
  const frozen = Object.freeze({ "zed": 1, "alpha": 2 });
  for (const value of [...globalThis["Object"]["values"](frozen)]) { console.log(value); }
  for (const key of [...globalThis.Object["keys"](frozen)]) { console.log(key); }
  for (const entry of [...globalThis["Object"].entries(frozen)]) { console.log(entry[0]); console.log(entry[1]); }
  const frozenKeys = Reflect.ownKeys(frozen);
  const frozenGlobalKeys = globalThis['Reflect']['ownKeys'](frozen);
  const frozenCallableValues = Object.freeze(Object.values)(frozen);
  const frozenCallableGlobalValues = Object.freeze(globalThis.Object.values)(frozen);
  const frozenCallableBracketedValues = Object.freeze(globalThis["Object"]["values"])(frozen);
  const bracketRootValues = Object.freeze((globalThis["Object"]))["values"](frozen);
  const bracketRootSingleQuotedValues = Object.freeze((globalThis["Object"])['values'])(frozen);
  const bracketRootEntries = Object.freeze((globalThis["Object"]))["entries"](frozen);
  const bracketRootSingleQuotedEntries = Object.freeze((globalThis["Object"])['entries'])(frozen);
  if (
    bracketRootValues.length !== 2 ||
    bracketRootValues[0] !== 1 ||
    bracketRootValues[1] !== 2 ||
    bracketRootSingleQuotedValues.length !== 2 ||
    bracketRootSingleQuotedValues[0] !== 1 ||
    bracketRootSingleQuotedValues[1] !== 2 ||
    bracketRootEntries.length !== 2 ||
    bracketRootEntries[0][0] !== "zed" ||
    bracketRootEntries[0][1] !== 1 ||
    bracketRootEntries[1][0] !== "alpha" ||
    bracketRootEntries[1][1] !== 2 ||
    bracketRootSingleQuotedEntries.length !== 2 ||
    bracketRootSingleQuotedEntries[0][0] !== "zed" ||
    bracketRootSingleQuotedEntries[0][1] !== 1 ||
    bracketRootSingleQuotedEntries[1][0] !== "alpha" ||
    bracketRootSingleQuotedEntries[1][1] !== 2
  ) {
    throw new Error('unexpected bracket-root object enumeration semantics');
  }
  const frozenCallableEntries = Object.freeze(Object.entries)(frozen);
  const frozenCallableGlobalEntries = Object.freeze(globalThis.Object.entries)(frozen);
  const frozenCallableBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(frozen);
  const parenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(frozen);
  const parenthesizedSingleQuotedBracketedValues = Object.freeze((globalThis['Object'])["values"])(frozen);
  const parenthesizedSingleQuotedReceiverBracketedValues = Object.freeze((globalThis['Object'])['values'])(frozen);
  const parenthesizedBracketedKeys = Object.freeze((globalThis["Object"]).keys)(frozen);
  const parenthesizedSingleQuotedReceiverBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(frozen);
  const parenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(frozen);
  const parenthesizedSingleQuotedBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(frozen);
  const nullishBracketedEntries = Object.freeze((null ?? globalThis["Object"]["entries"]))(frozen);
  const logicalAndBracketedEntries = Object.freeze((true && globalThis["Object"]["entries"]))(frozen);
  const logicalOrBracketedEntries = Object.freeze((false || globalThis["Object"]["entries"]))(frozen);
  const parenthesizedReceiverBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(frozen);
  const parenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(frozen);
  for (const value of [...parenthesizedBracketedValues]) { console.log(value); }
  for (const value of [...parenthesizedSingleQuotedBracketedValues]) { console.log(value); }
  for (const value of [...parenthesizedSingleQuotedReceiverBracketedValues]) { console.log(value); }
  for (const key of [...parenthesizedBracketedKeys]) { console.log(key); }
  for (const key of [...parenthesizedSingleQuotedReceiverBracketedKeys]) { console.log(key); }
  for (const entry of [...parenthesizedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...parenthesizedSingleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...frozenCallableEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...frozenCallableGlobalEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...frozenCallableBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...logicalAndBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...logicalOrBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...parenthesizedReceiverBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  for (const entry of [...parenthesizedSingleQuotedReceiverBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
  const frozenCallableKeys = Object.freeze(Reflect.ownKeys)(frozen);
  const frozenCallableGlobalKeys = Object.freeze(globalThis.Reflect.ownKeys)(frozen);
  const frozenCallableBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])(frozen);
  for (const value of [...frozenCallableValues]) { console.log(value); }
  for (const value of [...frozenCallableGlobalValues]) { console.log(value); }
  for (const value of [...frozenCallableBracketedValues]) { console.log(value); }
  for (const key of frozenKeys) { console.log(key); }
  for (const key of frozenCallableKeys) { console.log(key); }
  for (const key of frozenCallableGlobalKeys) { console.log(key); }
  for (const key of frozenCallableBracketedKeys) { console.log(key); }
  for await (const key of frozenGlobalKeys) { console.log(key); }
});
"#
}

fn assert_json_browser_runtime_frozen_object_enumeration_spread_semantics_in_input(
    command: &str,
    filename: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg("--output")
        .arg("json")
        .arg(command)
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

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    let expected_stdout = if command == "test"
        && (filename.ends_with(".jsx") || filename.ends_with(".tsx"))
    {
        "1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\n"
    } else if command == "test" {
        "1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\nalpha\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\n"
    } else {
        "1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\n"
    };
    if filename.ends_with(".jsx") || filename.ends_with(".tsx") {
        assert!(json["stdout"]
            .as_str()
            .expect("stdout string")
            .starts_with("1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n1\n2\n1\n2\n1\n2\nzed\nalpha\n"));
    } else {
        assert_eq!(json["stdout"], expected_stdout);
    }
    assert_eq!(json["stderr"], "");
}

fn assert_json_object_enumeration_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_enumeration_semantics_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    assert_eq!(json["stdout"], "2\n2\n2\n");
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_json_object_from_entries_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_from_entries_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    assert_eq!(json["stdout"], "2\n2\n2\n");
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_json_frozen_object_enumeration_spread_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_runtime_frozen_object_enumeration_spread_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    let stdout = json["stdout"].as_str().expect("stdout string");
    assert!(
        stdout.starts_with("1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("1\n2\n1\n2\n1\n2\nzed\nalpha\nzed\n1\nalpha\n2\n"),
        "stdout: {stdout}"
    );
    assert!(
        stdout
            .ends_with("zed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\nzed\nalpha\n"),
        "stdout: {stdout}"
    );
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_browser_runtime_object_from_entries_has_own_semantics_in_input(
    command: &str,
    filename: &str,
    source: &str,
    assert_stdout: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .arg(command)
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
    assert_eq!(output.status.code(), Some(0));
    if assert_stdout {
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n"
        );
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_json_object_string_primitive_enumeration_semantics(
    command: &str,
    filename: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    assert_eq!(json["stdout"], "2\n");
    assert_eq!(json["stderr"], "");
}

fn object_property_deletion_semantics_source() -> &'static str {
    r#"const obj = { a: 1, b: 2 };
if (!('a' in obj) || !('b' in obj)) {
  throw 'missing property';
}
if (delete obj.a !== true) {
  throw 'unexpected delete result';
}
if ('a' in obj) {
  throw 'delete failed';
}
"#
}

fn assert_object_property_deletion_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
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

fn assert_json_object_property_deletion_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn assert_browser_requested_object_property_deletion_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
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

fn assert_json_browser_requested_object_property_deletion_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn assert_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
    command: &str,
    filename: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");
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
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
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

fn assert_json_browser_requested_object_property_deletion_semantics_when_browser_api_surface_is_inherited(
    command: &str,
    filename: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_property_deletion_semantics_source()).expect("write source");
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
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg(command)
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn browser_bundle_object_property_deletion_semantics_source() -> &'static str {
    r#"// kali-tree-shake: objectPropertyDeletionSmoke
async function objectPropertyDeletionSmoke() {
  const obj = { a: 1, b: 2 };
  if (!("a" in obj) || !("b" in obj)) {
    throw new Error('missing property');
  }
  if (delete obj.a !== true) {
    throw new Error('unexpected delete result');
  }
  if ("a" in obj) {
    throw new Error('delete failed');
  }
  console.log('object deletion ok');
  return 0n;
}
"#
}

fn object_type_and_constructor_semantics_source(test_mode: bool) -> String {
    if test_mode {
        return r#"function Box() {}
Kali.test('object type and constructor semantics', () => {
  const box = new Box();
  if (typeof box !== 'object') {
    throw new Error('expected object from constructor');
  }
  if (typeof Box !== 'function') {
    throw new Error('expected constructor function');
  }
  if (typeof null !== 'object') {
    throw new Error('expected typeof null to be object');
  }
  if (!(box instanceof Box)) {
    throw new Error('expected instanceof to succeed');
  }
});
"#
        .to_string();
    }

    r#"function Box() {}
const box = new Box();
if (typeof box !== 'object') {
  throw new Error('expected object from constructor');
}
if (typeof Box !== 'function') {
  throw new Error('expected constructor function');
}
if (typeof null !== 'object') {
  throw new Error('expected typeof null to be object');
}
if (!(box instanceof Box)) {
  throw new Error('expected instanceof to succeed');
}
console.log('object type ok');
"#
    .to_string()
}

fn assert_json_object_type_and_constructor_semantics(
    command: &str,
    filename: &str,
    test_mode: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("object type ok"),
            "json: {json}"
        );
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
        assert_eq!(json["stdout"], "");
    }
    assert_eq!(json["stderr"], "");
}

fn assert_json_browser_requested_object_type_and_constructor_semantics(
    command: &str,
    filename: &str,
    test_mode: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        object_type_and_constructor_semantics_source(test_mode),
    )
    .expect("write source");

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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("object type ok"),
            "json: {json}"
        );
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["stdout"], "");
    }
    assert_eq!(json["stderr"], "");
}

fn browser_bundle_object_type_and_constructor_semantics_source() -> &'static str {
    r#"// kali-tree-shake: objectTypeSmoke
function Box() {}
async function objectTypeSmoke() {
  const box = new Box();
  if (typeof box !== 'object') {
    throw new Error('expected object from constructor');
  }
  if (typeof Box !== 'function') {
    throw new Error('expected constructor function');
  }
  if (typeof null !== 'object') {
    throw new Error('expected typeof null to be object');
  }
  if (!(box instanceof Box)) {
    throw new Error('expected instanceof to succeed');
  }
  console.log('object type ok');
  return 0n;
}
"#
}

fn unary_prefix_semantics_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('unary prefix semantics', () => {
  const notTrue = !true;
  if (notTrue !== false) {
    throw new Error('expected logical negation to invert the boolean');
  }
  const negative = -(1 + 2);
  if (negative !== -3) {
    throw new Error('expected unary minus to negate the value');
  }
  const positive = +(1 + 2);
  if (positive !== 3) {
    throw new Error('expected unary plus to preserve the numeric value');
  }
  const bitwiseNot = ~1;
  if (bitwiseNot !== -2) {
    throw new Error('expected bitwise not to invert integer bits');
  }
  let counter = 1;
  const prefix = ++counter;
  if (prefix !== 2 || counter !== 2) {
    throw new Error('expected prefix update expressions to return the incremented value');
  }
  const postfix = counter--;
  if (postfix !== 2 || counter !== 1) {
    throw new Error('expected postfix update expressions to return the previous value');
  }
  const value = void (1 + 2);
  if (value !== void 0) {
    throw new Error('expected void to evaluate to undefined');
  }
  if (typeof value !== 'undefined') {
    throw new Error('expected void result to be undefined');
  }
});
"#
        .to_string();
    }

    r#"const notTrue = !true;
if (notTrue !== false) {
  throw new Error('expected logical negation to invert the boolean');
}
const negative = -(1 + 2);
if (negative !== -3) {
  throw new Error('expected unary minus to negate the value');
}
const positive = +(1 + 2);
if (positive !== 3) {
  throw new Error('expected unary plus to preserve the numeric value');
}
const bitwiseNot = ~1;
if (bitwiseNot !== -2) {
  throw new Error('expected bitwise not to invert integer bits');
}
let counter = 1;
const prefix = ++counter;
if (prefix !== 2 || counter !== 2) {
  throw new Error('expected prefix update expressions to return the incremented value');
}
const postfix = counter--;
if (postfix !== 2 || counter !== 1) {
  throw new Error('expected postfix update expressions to return the previous value');
}
const value = void (1 + 2);
if (value !== void 0) {
  throw new Error('expected void to evaluate to undefined');
}
if (typeof value !== 'undefined') {
  throw new Error('expected void result to be undefined');
}
"#
    .to_string()
}

fn assert_unary_prefix_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, unary_prefix_semantics_source(false)).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
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

fn assert_json_unary_prefix_semantics(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, unary_prefix_semantics_source(false)).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "kali-hosted");
        assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn browser_harness_unary_prefix_semantics_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('unary prefix semantics', () => {
  let counter = 1;
  const prefix = ++counter;
  if (prefix !== 2 || counter !== 2) {
    throw new Error('expected prefix update expressions to return the incremented value');
  }
  const postfix = counter--;
  if (postfix !== 2 || counter !== 1) {
    throw new Error('expected postfix update expressions to return the previous value');
  }
  const bitwiseNot = ~1;
  if (bitwiseNot !== -2) {
    throw new Error('expected bitwise not to invert integer bits');
  }
});
"#
        .to_string();
    }

    r#"let counter = 1;
const prefix = ++counter;
if (prefix !== 2 || counter !== 2) {
  throw new Error('expected prefix update expressions to return the incremented value');
}
const postfix = counter--;
if (postfix !== 2 || counter !== 1) {
  throw new Error('expected postfix update expressions to return the previous value');
}
const bitwiseNot = ~1;
if (bitwiseNot !== -2) {
  throw new Error('expected bitwise not to invert integer bits');
}
"#
    .to_string()
}

fn assert_browser_unary_prefix_semantics(command: &str, filename: &str, test_mode: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_unary_prefix_semantics_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
}

fn assert_json_browser_unary_prefix_semantics(command: &str, filename: &str, test_mode: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_unary_prefix_semantics_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn wrapped_mutable_update_targets_source() -> String {
    "let value = 1 as number;\n((value as number)) += 2;\nconst prefix = ++(value satisfies number);\nif (value !== 4 || prefix !== 4) {\n  throw new Error(`unexpected wrapped update result ${value} ${prefix}`);\n}\n".to_string()
}

fn browser_harness_wrapped_mutable_update_targets_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('wrapped mutable update targets', () => {
  let value = 1 as number;
  ((value as number)) += 2;
  const prefix = ++(value satisfies number);
  if (value !== 4 || prefix !== 4) {
    throw new Error(`unexpected wrapped update result ${value} ${prefix}`);
  }
});
"#
        .to_string();
    }

    wrapped_mutable_update_targets_source()
}

fn wrapped_mutable_compound_assignment_targets_source() -> String {
    "let value = 1 as number;\n((value as number)) += 2;\n((value as number)) -= 1;\n((value as number)) *= 5;\n((value as number)) /= 2;\n((value as number)) %= 4;\n((value as number)) **= 3;\nif (value !== 1) {\n  throw new Error(`unexpected wrapped compound result ${value}`);\n}\n".to_string()
}

fn browser_harness_wrapped_mutable_compound_assignment_targets_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('wrapped mutable compound assignment targets', () => {
  let value = 1 as number;
  ((value as number)) += 2;
  ((value as number)) -= 1;
  ((value as number)) *= 5;
  ((value as number)) /= 2;
  ((value as number)) %= 4;
  ((value as number)) **= 3;
  if (value !== 1) {
    throw new Error(`unexpected wrapped compound result ${value}`);
  }
});
"#
        .to_string();
    }

    wrapped_mutable_compound_assignment_targets_source()
}

fn assert_wrapped_mutable_update_targets(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, wrapped_mutable_update_targets_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);
}

fn assert_browser_wrapped_mutable_update_targets(command: &str, filename: &str, test_mode: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_wrapped_mutable_update_targets_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
}

fn assert_json_browser_wrapped_mutable_update_targets(
    command: &str,
    filename: &str,
    test_mode: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_wrapped_mutable_update_targets_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn assert_wrapped_mutable_compound_assignment_targets(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        wrapped_mutable_compound_assignment_targets_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);
}

fn assert_browser_wrapped_mutable_compound_assignment_targets(
    command: &str,
    filename: &str,
    test_mode: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_wrapped_mutable_compound_assignment_targets_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);
}

fn assert_json_browser_wrapped_mutable_compound_assignment_targets(
    command: &str,
    filename: &str,
    test_mode: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_wrapped_mutable_compound_assignment_targets_source(test_mode),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
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
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    }
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}

fn math_floor_trunc_ceil_const_numeric_alias_chain_source() -> String {
    let frozen_callable_aliases = math_floor_trunc_ceil_frozen_callable_aliases();
    let variable_names = frozen_callable_aliases
        .iter()
        .enumerate()
        .map(|(index, _)| format!("frozenMathAlias{index}"))
        .collect::<Vec<_>>();

    let declarations = variable_names
        .iter()
        .zip(frozen_callable_aliases.iter())
        .map(|(name, alias)| format!("const {name} = {alias};"))
        .collect::<Vec<_>>()
        .join(" ");
    let invocations = variable_names
        .iter()
        .map(|name| format!("console.log({name}(alias));"))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        "const value = 1.6; const alias = value; {declarations} console.log(Math.floor(alias)); console.log(Math.trunc(alias)); console.log(Math.ceil(alias)); {invocations}\n"
    )
}

fn assert_embeds_policy_custom_section(artifact_path: &Path, policy_path: &Path) {
    let built = fs::read(artifact_path).expect("read wasm artifact");
    let policy_bytes = fs::read(policy_path).expect("read policy bytes");
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

fn assert_build_supports_function_declaration_export_aliases_for_library_artifact(
    extension: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("math.{extension}"));
    fs::write(
        &source_path,
        "export function main(input) { return 1; } export { main as alias };",
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path()).arg("build").arg("--lib");
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
        let envelope = parse_json_stdout(&output);
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "lib");
        let exports = payload["exports"].as_array().expect("exports array");
        assert!(
            exports.iter().any(|export| {
                export["name"] == "alias" && export["signature"] == "(input) => number"
            }),
            "expected alias export in {exports:?}"
        );
        assert!(
            exports.iter().any(|export| {
                export["name"] == "main" && export["signature"] == "(input) => number"
            }),
            "expected main export in {exports:?}"
        );
    }

    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("math.lib.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["artifactKind"], "lib");
    let exports = metadata["exports"].as_array().expect("exports array");
    assert!(
        exports.iter().any(|export| {
            export["name"] == "alias" && export["signature"] == "(input) => number"
        }),
        "expected alias export in {exports:?}"
    );
    assert!(
        exports.iter().any(|export| {
            export["name"] == "main" && export["signature"] == "(input) => number"
        }),
        "expected main export in {exports:?}"
    );

    assert!(dir.path().join("math.lib.wasm").exists());
    assert!(dir.path().join("math.lib.wit").exists());
    assert!(dir.path().join("math.lib.meta.json").exists());
}

fn browser_bundle_web_baseline_source() -> String {
    r#"// kali-tree-shake: webBaselineSmoke
function webBaselineSmoke(left, right) {
  const original = { nested: { count: 1 }, values: [1, 2, 3] };
  const cloned = structuredClone(original);
  if (cloned === original || cloned.nested === original.nested || cloned.values === original.values) {
    throw new Error('structuredClone should deep-clone object graphs');
  }
  original.nested.count = 2;
  original.values.push(4);
  if (cloned.nested.count !== 1 || cloned.values.join(',') !== '1,2,3') {
    throw new Error(`unexpected structuredClone result ${JSON.stringify(cloned)}`);
  }
  const controller = new AbortController();
  if (!(controller.signal instanceof AbortSignal)) {
    throw new Error('expected AbortSignal from AbortController');
  }
  const target = new EventTarget();
  let count = 0;
  target.addEventListener('tick', () => {
    count += 1;
    controller.abort();
  });
  const dispatched = target.dispatchEvent(new CustomEvent('tick'));
  if (!dispatched || count !== 1 || !controller.signal.aborted) {
    throw new Error('unexpected event primitive behavior');
  }
  const query = new URLSearchParams('alpha=1&beta=two+words');
  query.append('gamma', String(left + right));
  query.set('beta', String(left));
  if (query.get('alpha') !== '1' || query.get('beta') !== String(left) || query.getAll('beta').length !== 1 || !query.has('gamma')) {
    throw new Error(`unexpected URLSearchParams behavior ${query.toString()}`);
  }
  const browserUrl = new URL('https://example.com/browser?alpha=1#fragment');
  if (browserUrl.origin !== 'https://example.com' || browserUrl.pathname !== '/browser' || browserUrl.search !== '?alpha=1' || browserUrl.hash !== '#fragment' || browserUrl.searchParams.get('alpha') !== '1') {
    throw new Error(`unexpected URL behavior ${browserUrl.href}`);
  }
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const encoded = encoder.encode(String(left + right));
  if (decoder.decode(encoded) !== String(left + right)) {
    throw new Error('unexpected TextEncoder/TextDecoder behavior');
  }
  return left - left;
}
"#.to_string()
}

fn assert_browser_bundle_console_level_routing_in_extension(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("app.{extension}"));
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.info('info');\n  console.debug('debug');\n  console.error('err');\n  console.warn('warn');\n  console.log(-1);\n  return 0n;\n}\n",
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
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("info"), "stdout: {stdout}");
    assert!(stdout.contains("debug"), "stdout: {stdout}");
    assert!(stdout.contains("-1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("err"), "stderr: {stderr}");
    assert!(stderr.contains("warn"), "stderr: {stderr}");
}

fn assert_browser_bundle_console_assert_routing_in_extension(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("app.{extension}"));
    fs::write(
        &source_path,
        "// kali-tree-shake: consoleSmoke\nasync function consoleSmoke() {\n  console.assert(false, 'assert failed');\n  return 0n;\n}\n",
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
    assert_artifact_metadata_provenance(&metadata, "bundle", 16, None);
    assert_eq!(metadata["apiSurface"], "browser");

    let bundle_dir_name = bundle_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-console-assert-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir_name,
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.consoleSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stderr.contains("assert failed"), "stderr: {stderr}");
}

fn assert_non_literal_dynamic_import_rejection_text(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

fn assert_non_literal_dynamic_import_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("non-literal dynamic import()")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("statically known import specifier")),
        "missing non-literal dynamic import in {errors:?}"
    );
}

fn promise_all_settled_source_variants() -> [&'static str; 28] {
    [
        "console.log(Promise.allSettled([1, 2]));\n",
        "console.log(Promise[\"allSettled\"]([1, 2]));\n",
        "console.log(Promise['allSettled']([1, 2]));\n",
        "console.log(globalThis.Promise.allSettled([1, 2]));\n",
        "console.log(globalThis.Promise[\"allSettled\"]([1, 2]));\n",
        "console.log(globalThis.Promise['allSettled']([1, 2]));\n",
        "console.log(globalThis[\"Promise\"].allSettled([1, 2]));\n",
        "console.log(globalThis['Promise'].allSettled([1, 2]));\n",
        "console.log(globalThis[\"Promise\"][\"allSettled\"]([1, 2]));\n",
        "console.log(globalThis['Promise']['allSettled']([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise)[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze(Promise[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((Promise[\"allSettled\"]))([1, 2]));\n",
        "console.log(Object.freeze(Promise.allSettled)([1, 2]));\n",
        "console.log(Object.freeze((Promise.allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis.Promise.allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise.allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis.Promise[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise[\"allSettled\"]))([1, 2]));\n",
        "console.log(Object.freeze(globalThis[\"Promise\"].allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"].allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis['Promise'].allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise'].allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis['Promise']['allSettled'])([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise']['allSettled']))([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"]).allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise']).allSettled)([1, 2]));\n",
    ]
}

fn nullish_assignment_source() -> &'static str {
    "let value = null; value ??= 1; console.log(value);\n"
}

fn compound_assignment_non_local_source() -> &'static str {
    "let target = { value: 1 }; target.value += 2;\n"
}

fn compound_assignment_immutable_source() -> &'static str {
    "const value = 1; value += 2;\n"
}

fn assert_compound_assignment_rejection_text(stderr: &str, expected: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains(expected), "stderr: {stderr}");
}

fn assert_compound_assignment_rejection_json(errors: &[Value], expected: &str) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains(expected)),
        "missing compound-assignment diagnostic in {errors:?}"
    );
}

fn assert_object_is_same_reference_alias_chain_in_browser_harness(
    command: &str,
    extension: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(match command {
        "test" => format!("smoke.test.{extension}"),
        _ => format!("main.{extension}"),
    });
    let source = match command {
        "test" => r#"Kali.test('browser object.is references', () => { const object = { a: 1 }; const alias = object; const frozen = Object.freeze(object); const array = [1, 2]; const arrayAlias = array; const frozenArray = Object.freeze(array); console.log(Object.is(alias, object)); console.log(Object.is(frozen, object)); console.log(Object.is(arrayAlias, array)); console.log(Object.is(frozenArray, array)); });
"#.to_string(),
        _ => r#"const object = { a: 1 }; const alias = object; const frozen = Object.freeze(object); const array = [1, 2]; const arrayAlias = array; const frozenArray = Object.freeze(array); console.log(Object.is(alias, object)); console.log(Object.is(frozen, object)); console.log(Object.is(arrayAlias, array)); console.log(Object.is(frozenArray, array));
"#.to_string(),
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(&source_path);

    let output = cli.output().expect("run kali");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["stdout"], "1\n1\n1\n1\n");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\n1\n1\n1"), "stdout: {stdout}");
    }
}

fn assert_build_supports_math_log2_and_log10_const_alias_chains(filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        "const log2Value = 8; const log2Alias = log2Value; console.log(Math.log2(log2Alias)); console.log(Object.freeze(Math.log2)(log2Alias));\nconst log10Value = 1000; const log10Alias = log10Value; console.log(Math.log10(log10Alias)); console.log(Object.freeze(Math.log10)(log10Alias));\n",
    )
    .expect("write source");

    for output_json in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if output_json {
            output.arg("--output").arg("json");
        }
        let output = output
            .arg("build")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(0));

        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], true);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("Built executable artifact at"),
                "stdout: {stdout}"
            );
        }
    }

    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

fn assert_build_supports_math_hypot_on_perfect_square_integer_literal_sums(filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, "console.log(Math.hypot(3, 4));\n").expect("write source");

    for output_json in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if output_json {
            output.arg("--output").arg("json");
        }
        let output = output
            .arg("build")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(0));

        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], true);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("Built executable artifact at"),
                "stdout: {stdout}"
            );
        }
    }

    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

fn assert_unsupported_math_member_calls_rejection_text(stderr: &str) {
    assert_unsupported_math_member_calls_rejection_text_for_method(stderr, "Math.sqrt");
}

fn assert_unsupported_math_member_calls_rejection_text_for_method(
    stderr: &str,
    expected_method: &str,
) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains(expected_method), "stderr: {stderr}");
    assert!(stderr.contains("later compatibility"), "stderr: {stderr}");
}

fn assert_unsupported_math_member_calls_rejection_json(errors: &[Value]) {
    assert_unsupported_math_member_calls_rejection_json_for_method(errors, "Math.sqrt");
}

fn assert_unsupported_math_member_calls_rejection_json_for_method(
    errors: &[Value],
    expected_method: &str,
) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            let message = error["message"].as_str().expect("error message");
            message.contains(expected_method)
        }),
        "missing unsupported Math member call in {errors:?}"
    );
}

fn assert_optional_chain_math_pow_rejection_text(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Math.pow"), "stderr: {stderr}");
    assert!(
        stderr.contains("optional-chain wrappers"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("later compatibility"), "stderr: {stderr}");
}

fn assert_optional_chain_math_pow_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error["message"]
                .as_str()
                .expect("error message")
                .contains("Math.pow is unavailable through optional-chain wrappers")
        }),
        "missing optional-chain Math.pow rejection in {errors:?}"
    );
}

fn assert_browser_for_of_array_iteration(output: &str) {
    assert!(output.contains("1"), "output: {output}");
    assert!(output.contains("2"), "output: {output}");
}

fn assert_browser_for_of_array_iteration_json(success: bool) {
    assert!(success);
}

fn assert_browser_for_await_array_iteration(output: &str) {
    assert!(output.contains("1"), "output: {output}");
    assert!(output.contains("2"), "output: {output}");
}

fn assert_browser_for_await_array_iteration_json(success: bool) {
    assert!(success);
}

fn assert_set_and_map_iteration(stdout: &str) {
    assert!(
        stdout.contains("set and map constructor iteration ok"),
        "stdout: {stdout}"
    );
}

fn set_and_map_iteration_run_source() -> &'static str {
    r##"function assertSetIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Set constructor iteration semantics');
  }
}

function assertMapIteration(values) {
  if (
    values.length !== 2 ||
    values[0][0] !== 1 ||
    values[0][1] !== 3 ||
    values[1][0] !== 4 ||
    values[1][1] !== 5
  ) {
    throw new Error('unexpected Map constructor iteration semantics');
  }
}

function setAndMapIteration() {
  const values = [1, 2, 1];
  let setReturnFinally = false;
  function setReturnProbe() {
    try {
      for (const value of new Set(values)) {
        return value;
      }
      throw new Error('unexpected empty Set constructor iteration');
    } finally {
      setReturnFinally = true;
    }
  }
  const setReturnValue = setReturnProbe();
  if (setReturnValue !== 1 || !setReturnFinally) {
    throw new Error('unexpected Set constructor return/finally semantics');
  }

  let mapThrowFinally = false;
  function mapThrowProbe() {
    try {
      for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
        if (entry[0] === 1) {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Map constructor iteration');
    } finally {
      mapThrowFinally = true;
    }
  }
  let mapThrew = false;
  try {
    mapThrowProbe();
  } catch {
    mapThrew = true;
  }
  if (!mapThrew || !mapThrowFinally) {
    throw new Error('unexpected Map constructor throw/finally semantics');
  }

  const setAlias = Set;
  const wrappedSetAlias = (setAlias);
  const aliasValues = (values);
  const direct = [];
  for (const value of new Set(values)) {
    direct.push(value);
  }
  const arrayFromSet = [];
  for (const value of Array.from(new Set(values))) {
    arrayFromSet.push(value);
  }
  const alias = [];
  for (const value of new setAlias(aliasValues)) {
    alias.push(value);
  }
  const wrappedAlias = [];
  for (const value of new (wrappedSetAlias)(aliasValues)) {
    wrappedAlias.push(value);
  }
  const globalDirect = [];
  for (const value of new globalThis.Set(values)) {
    globalDirect.push(value);
  }
  const bracketed = [];
  for (const value of new globalThis["Set"](values)) {
    bracketed.push(value);
  }
  const singleBracketed = [];
  for (const value of new globalThis['Set'](values)) {
    singleBracketed.push(value);
  }
  const parenthesizedBracketed = [];
  for (const value of new (globalThis["Set"])(values)) {
    parenthesizedBracketed.push(value);
  }
  const parenthesizedSingleBracketed = [];
  for (const value of new (globalThis['Set'])(values)) {
    parenthesizedSingleBracketed.push(value);
  }
  const frozenValues = Object.freeze(aliasValues);
  const frozenDirect = [];
  for (const value of new Set(frozenValues)) {
    frozenDirect.push(value);
  }

  assertSetIteration(direct);
  assertSetIteration(arrayFromSet);
  assertSetIteration(alias);
  assertSetIteration(wrappedAlias);
  assertSetIteration(globalDirect);
  assertSetIteration(bracketed);
  assertSetIteration(singleBracketed);
  assertSetIteration(parenthesizedBracketed);
  assertSetIteration(parenthesizedSingleBracketed);
  assertSetIteration(frozenDirect);

  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const mapAlias = Map;
  const wrappedMapAlias = (mapAlias);
  const mapDirect = [];
  for (const entry of new Map(mapValues)) {
    mapDirect.push(entry);
  }
  const arrayFromMap = [];
  for (const entry of Array.from(new Map(mapValues))) {
    arrayFromMap.push(entry);
  }
  const mapAliasValues = [];
  for (const entry of new mapAlias(mapValues)) {
    mapAliasValues.push(entry);
  }
  const wrappedMapAliasValues = [];
  for (const entry of new (wrappedMapAlias)(mapValues)) {
    wrappedMapAliasValues.push(entry);
  }
  const globalMapDirect = [];
  for (const entry of new globalThis.Map(mapValues)) {
    globalMapDirect.push(entry);
  }
  const bracketedMap = [];
  for (const entry of new globalThis["Map"](mapValues)) {
    bracketedMap.push(entry);
  }
  const singleBracketedMap = [];
  for (const entry of new globalThis['Map'](mapValues)) {
    singleBracketedMap.push(entry);
  }
  const parenthesizedBracketedMap = [];
  for (const entry of new (globalThis["Map"])(mapValues)) {
    parenthesizedBracketedMap.push(entry);
  }
  const parenthesizedSingleBracketedMap = [];
  for (const entry of new (globalThis['Map'])(mapValues)) {
    parenthesizedSingleBracketedMap.push(entry);
  }

  const frozenMapValues = Object.freeze(mapValues);
  const frozenMapDirect = [];
  for (const entry of new Map(frozenMapValues)) {
    frozenMapDirect.push(entry);
  }

  assertMapIteration(mapDirect);
  assertMapIteration(arrayFromMap);
  assertMapIteration(mapAliasValues);
  assertMapIteration(wrappedMapAliasValues);
  assertMapIteration(globalMapDirect);
  assertMapIteration(bracketedMap);
  assertMapIteration(singleBracketedMap);
  assertMapIteration(parenthesizedBracketedMap);
  assertMapIteration(parenthesizedSingleBracketedMap);
  assertMapIteration(frozenMapDirect);

  let setBreakContinueCount = 0;
  let setBreakContinueFinally = false;
  try {
    for (const value of new Set(values)) {
      if (value === 1) {
        continue;
      }
      setBreakContinueCount += 1;
      break;
    }
  } finally {
    setBreakContinueFinally = true;
  }
  if (setBreakContinueCount !== 1 || !setBreakContinueFinally) {
    throw new Error('unexpected Set constructor break/continue semantics');
  }

  let mapBreakContinueCount = 0;
  let mapBreakContinueFinally = false;
  try {
    for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
      if (entry[0] === 1) {
        continue;
      }
      mapBreakContinueCount += 1;
      break;
    }
  } finally {
    mapBreakContinueFinally = true;
  }
  if (mapBreakContinueCount !== 1 || !mapBreakContinueFinally) {
    throw new Error('unexpected Map constructor break/continue semantics');
  }

  console.log('set and map constructor iteration ok');
}

setAndMapIteration();
"##
}

fn set_and_map_iteration_test_source() -> &'static str {
    r##"Kali.test('set and map iteration', () => {
  function assertSetIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Set constructor iteration semantics');
    }
  }

  function assertMapIteration(values) {
    if (
      values.length !== 2 ||
      values[0][0] !== 1 ||
      values[0][1] !== 3 ||
      values[1][0] !== 4 ||
      values[1][1] !== 5
    ) {
      throw new Error('unexpected Map constructor iteration semantics');
    }
  }

  function setAndMapIteration() {
    const values = [1, 2, 1];
    let setReturnFinally = false;
    function setReturnProbe() {
      try {
        for (const value of new Set(values)) {
          return value;
        }
        throw new Error('unexpected empty Set constructor iteration');
      } finally {
        setReturnFinally = true;
      }
    }
    const setReturnValue = setReturnProbe();
    if (setReturnValue !== 1 || !setReturnFinally) {
      throw new Error('unexpected Set constructor return/finally semantics');
    }

    let mapThrowFinally = false;
    function mapThrowProbe() {
      try {
        for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
          if (entry[0] === 1) {
            throw new Error('boom');
          }
        }
        throw new Error('unexpected empty Map constructor iteration');
      } finally {
        mapThrowFinally = true;
      }
    }
    let mapThrew = false;
    try {
      mapThrowProbe();
    } catch {
      mapThrew = true;
    }
    if (!mapThrew || !mapThrowFinally) {
      throw new Error('unexpected Map constructor throw/finally semantics');
    }

    const setAlias = Set;
    const wrappedSetAlias = (setAlias);
    const frozenSetAlias = Object.freeze(Set);
    const wrappedFrozenSetAlias = Object.freeze((Set));
    const frozenGlobalSetAlias = Object.freeze(globalThis.Set);
    const wrappedFrozenGlobalSetAlias = Object.freeze((globalThis.Set));
    const frozenBracketedSetAlias = Object.freeze(globalThis["Set"]);
    const wrappedFrozenBracketedSetAlias = Object.freeze((globalThis["Set"]));
    const frozenSingleBracketedSetAlias = Object.freeze(globalThis['Set']);
    const wrappedFrozenSingleBracketedSetAlias = Object.freeze((globalThis['Set']));
    const aliasValues = (values);
    const direct = [];
    for (const value of new Set(values)) {
      direct.push(value);
    }
    const arrayFromSet = [];
    for (const value of Array.from(new Set(values))) {
      arrayFromSet.push(value);
    }
    const alias = [];
    for (const value of new setAlias(aliasValues)) {
      alias.push(value);
    }
    const wrappedAlias = [];
    for (const value of new (wrappedSetAlias)(aliasValues)) {
      wrappedAlias.push(value);
    }
    const globalDirect = [];
    for (const value of new globalThis.Set(values)) {
      globalDirect.push(value);
    }
    const bracketed = [];
    for (const value of new globalThis["Set"](values)) {
      bracketed.push(value);
    }
    const singleBracketed = [];
    for (const value of new globalThis['Set'](values)) {
      singleBracketed.push(value);
    }
    const parenthesizedBracketed = [];
    for (const value of new (globalThis["Set"])(values)) {
      parenthesizedBracketed.push(value);
    }
    const parenthesizedSingleBracketed = [];
    for (const value of new (globalThis['Set'])(values)) {
      parenthesizedSingleBracketed.push(value);
    }
    const frozenValues = Object.freeze(aliasValues);
    const frozenDirect = [];
    for (const value of new frozenSetAlias(frozenValues)) {
      frozenDirect.push(value);
    }
    const wrappedFrozenDirect = [];
    for (const value of new (wrappedFrozenSetAlias)(frozenValues)) {
      wrappedFrozenDirect.push(value);
    }
    const frozenGlobalDirect = [];
    for (const value of new frozenGlobalSetAlias(values)) {
      frozenGlobalDirect.push(value);
    }
    const wrappedFrozenGlobalDirect = [];
    for (const value of new (wrappedFrozenGlobalSetAlias)(values)) {
      wrappedFrozenGlobalDirect.push(value);
    }
    const frozenBracketedDirect = [];
    for (const value of new frozenBracketedSetAlias(values)) {
      frozenBracketedDirect.push(value);
    }
    const wrappedFrozenBracketedDirect = [];
    for (const value of new (wrappedFrozenBracketedSetAlias)(values)) {
      wrappedFrozenBracketedDirect.push(value);
    }
    const frozenSingleBracketedDirect = [];
    for (const value of new frozenSingleBracketedSetAlias(values)) {
      frozenSingleBracketedDirect.push(value);
    }
    const wrappedFrozenSingleBracketedDirect = [];
    for (const value of new (wrappedFrozenSingleBracketedSetAlias)(values)) {
      wrappedFrozenSingleBracketedDirect.push(value);
    }

    assertSetIteration(direct);
    assertSetIteration(arrayFromSet);
    assertSetIteration(alias);
    assertSetIteration(wrappedAlias);
    assertSetIteration(globalDirect);
    assertSetIteration(bracketed);
    assertSetIteration(singleBracketed);
    assertSetIteration(parenthesizedBracketed);
    assertSetIteration(parenthesizedSingleBracketed);
    assertSetIteration(frozenDirect);
    assertSetIteration(wrappedFrozenDirect);
    assertSetIteration(frozenGlobalDirect);
    assertSetIteration(wrappedFrozenGlobalDirect);
    assertSetIteration(frozenBracketedDirect);
    assertSetIteration(wrappedFrozenBracketedDirect);
    assertSetIteration(frozenSingleBracketedDirect);
    assertSetIteration(wrappedFrozenSingleBracketedDirect);

    const mapValues = [[1, 2], [1, 3], [4, 5]];
    const mapAlias = Map;
    const wrappedMapAlias = (mapAlias);
    const frozenMapAlias = Object.freeze(Map);
    const wrappedFrozenMapAlias = Object.freeze((Map));
    const frozenGlobalMapAlias = Object.freeze(globalThis.Map);
    const wrappedFrozenGlobalMapAlias = Object.freeze((globalThis.Map));
    const frozenBracketedMapAlias = Object.freeze(globalThis["Map"]);
    const wrappedFrozenBracketedMapAlias = Object.freeze((globalThis["Map"]));
    const frozenSingleBracketedMapAlias = Object.freeze(globalThis['Map']);
    const wrappedFrozenSingleBracketedMapAlias = Object.freeze((globalThis['Map']));
    const mapDirect = [];
    for (const entry of new Map(mapValues)) {
      mapDirect.push(entry);
    }
    const arrayFromMap = [];
    for (const entry of Array.from(new Map(mapValues))) {
      arrayFromMap.push(entry);
    }
    const mapAliasValues = [];
    for (const entry of new mapAlias(mapValues)) {
      mapAliasValues.push(entry);
    }
    const wrappedMapAliasValues = [];
    for (const entry of new (wrappedMapAlias)(mapValues)) {
      wrappedMapAliasValues.push(entry);
    }
    const globalMapDirect = [];
    for (const entry of new globalThis.Map(mapValues)) {
      globalMapDirect.push(entry);
    }
    const bracketedMap = [];
    for (const entry of new globalThis["Map"](mapValues)) {
      bracketedMap.push(entry);
    }
    const singleBracketedMap = [];
    for (const entry of new globalThis['Map'](mapValues)) {
      singleBracketedMap.push(entry);
    }
    const parenthesizedBracketedMap = [];
    for (const entry of new (globalThis["Map"])(mapValues)) {
      parenthesizedBracketedMap.push(entry);
    }
    const parenthesizedSingleBracketedMap = [];
    for (const entry of new (globalThis['Map'])(mapValues)) {
      parenthesizedSingleBracketedMap.push(entry);
    }

    const frozenMapValues = Object.freeze(mapValues);
    const frozenMapDirect = [];
    for (const entry of new frozenMapAlias(frozenMapValues)) {
      frozenMapDirect.push(entry);
    }
    const wrappedFrozenMapDirect = [];
    for (const entry of new (wrappedFrozenMapAlias)(frozenMapValues)) {
      wrappedFrozenMapDirect.push(entry);
    }
    const frozenGlobalMapDirect = [];
    for (const entry of new frozenGlobalMapAlias(mapValues)) {
      frozenGlobalMapDirect.push(entry);
    }
    const wrappedFrozenGlobalMapDirect = [];
    for (const entry of new (wrappedFrozenGlobalMapAlias)(mapValues)) {
      wrappedFrozenGlobalMapDirect.push(entry);
    }
    const frozenBracketedMapDirect = [];
    for (const entry of new frozenBracketedMapAlias(mapValues)) {
      frozenBracketedMapDirect.push(entry);
    }
    const wrappedFrozenBracketedMapDirect = [];
    for (const entry of new (wrappedFrozenBracketedMapAlias)(mapValues)) {
      wrappedFrozenBracketedMapDirect.push(entry);
    }
    const frozenSingleBracketedMapDirect = [];
    for (const entry of new frozenSingleBracketedMapAlias(mapValues)) {
      frozenSingleBracketedMapDirect.push(entry);
    }
    const wrappedFrozenSingleBracketedMapDirect = [];
    for (const entry of new (wrappedFrozenSingleBracketedMapAlias)(mapValues)) {
      wrappedFrozenSingleBracketedMapDirect.push(entry);
    }

    assertMapIteration(mapDirect);
    assertMapIteration(arrayFromMap);
    assertMapIteration(mapAliasValues);
    assertMapIteration(wrappedMapAliasValues);
    assertMapIteration(globalMapDirect);
    assertMapIteration(bracketedMap);
    assertMapIteration(singleBracketedMap);
    assertMapIteration(parenthesizedBracketedMap);
    assertMapIteration(parenthesizedSingleBracketedMap);
    assertMapIteration(frozenMapDirect);
    assertMapIteration(wrappedFrozenMapDirect);
    assertMapIteration(frozenGlobalMapDirect);
    assertMapIteration(wrappedFrozenGlobalMapDirect);
    assertMapIteration(frozenBracketedMapDirect);
    assertMapIteration(wrappedFrozenBracketedMapDirect);
    assertMapIteration(frozenSingleBracketedMapDirect);
    assertMapIteration(wrappedFrozenSingleBracketedMapDirect);

    let setBreakContinueCount = 0;
    let setBreakContinueFinally = false;
    try {
      for (const value of new Set(values)) {
        if (value === 1) {
          continue;
        }
        setBreakContinueCount += 1;
        break;
      }
    } finally {
      setBreakContinueFinally = true;
    }
    if (setBreakContinueCount !== 1 || !setBreakContinueFinally) {
      throw new Error('unexpected Set constructor break/continue semantics');
    }

    let mapBreakContinueCount = 0;
    let mapBreakContinueFinally = false;
    try {
      for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
        if (entry[0] === 1) {
          continue;
        }
        mapBreakContinueCount += 1;
        break;
      }
    } finally {
      mapBreakContinueFinally = true;
    }
    if (mapBreakContinueCount !== 1 || !mapBreakContinueFinally) {
      throw new Error('unexpected Map constructor break/continue semantics');
    }

    console.log('set and map constructor iteration ok');
  }

  setAndMapIteration();
});
"##
}

fn assert_for_of_template_literal_iteration(stdout: &str) {
    let lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines, ["h", "e", "l", "l", "o"], "stdout: {stdout}");
}

fn assert_for_await_object_enumeration(stdout: &str) {
    let lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(
        lines,
        ["3", "2", "zed", "alpha", "zed", "3", "alpha", "2"],
        "stdout: {stdout}"
    );
}

fn assert_test_for_await_object_enumeration(stdout: &str) {
    let mut lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    if lines.last() == Some(&"ok 1") {
        lines.pop();
    }
    assert_eq!(
        lines,
        ["3", "2", "zed", "alpha", "zed", "3", "alpha", "2"],
        "stdout: {stdout}"
    );
}

fn assert_browser_for_await_object_enumeration(stdout: &str) {
    let mut lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    if lines.last() == Some(&"ok 1") {
        lines.pop();
    }
    assert_eq!(
        lines,
        ["3", "2", "zed", "alpha", "zed", "3", "alpha", "2"],
        "stdout: {stdout}"
    );
}

fn browser_spread_of_object_enumeration_in_for_await_array_iteration_source() -> &'static str {
    "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }\n"
}

fn assert_generator_function_lowering_rejection(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

fn assert_json_generator_function_lowering_rejection(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "function* main() { yield 1; }\nmain();").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(errors.iter().any(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("message"))
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("generator function lowering")
                || message.contains("yield expressions")),
        "messages: {messages:?}"
    );
}

fn assert_generator_function_lowering_rejection_in_browser_context(
    command: &str,
    bundle: bool,
    extension: &str,
    source_contents: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
        "stderr: {stderr}"
    );
}

fn assert_generator_function_lowering_rejection_when_browser_harness_is_configured(
    command: &str,
    extension: &str,
    json_output: bool,
    source_contents: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(
            messages
                .iter()
                .any(|message| message.contains("generator function lowering")
                    || message.contains("yield expressions")),
            "messages: {messages:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("generator function lowering") || stderr.contains("yield expressions"),
            "stderr: {stderr}"
        );
    }
}

fn array_callback_iteration_sources() -> [&'static str; 8] {
    [
        "const values = [1, 2]; for (const item of values.find((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findIndex((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findLast((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findLastIndex((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.some((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.every((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.reduce((acc, value) => acc + value, 0)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.reduceRight((acc, value) => acc + value, 0)) { console.log(item); }\n",
    ]
}

fn assert_class_generator_method_lowering_rejection(
    command: &str,
    json_output: bool,
    extension: &str,
    source_contents: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command).arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let expected_message = match (
        source_contents.contains("async *"),
        source_contents.contains("yield*"),
    ) {
        (true, true) => {
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (true, false) => {
            "async-generator class method lowering is unavailable in the direct runtime path"
        }
        (false, true) => {
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (false, false) => {
            "generator class method lowering is unavailable in the direct runtime path"
        }
    };

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|message| { message.contains(expected_message) }));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(stderr.contains(expected_message), "stderr: {stderr}");
    }
}

fn assert_class_generator_method_lowering_rejection_in_browser_context(
    command: &str,
    json_output: bool,
    bundle: bool,
    extension: &str,
    source_contents: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let expected_message = match (
        source_contents.contains("async *"),
        source_contents.contains("yield*"),
    ) {
        (true, true) => {
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (true, false) => {
            "async-generator class method lowering is unavailable in the direct runtime path"
        }
        (false, true) => {
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (false, false) => {
            "generator class method lowering is unavailable in the direct runtime path"
        }
    };

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|message| { message.contains(expected_message) }));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(stderr.contains(expected_message), "stderr: {stderr}");
    }
}

fn assert_class_generator_method_lowering_rejection_when_browser_harness_is_configured(
    command: &str,
    json_output: bool,
    extension: &str,
    source_contents: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let expected_message = match (
        source_contents.contains("async *"),
        source_contents.contains("yield*"),
    ) {
        (true, true) => {
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (true, false) => {
            "async-generator class method lowering is unavailable in the direct runtime path"
        }
        (false, true) => {
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        }
        (false, false) => {
            "generator class method lowering is unavailable in the direct runtime path"
        }
    };

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|message| { message.contains(expected_message) }));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(stderr.contains(expected_message), "stderr: {stderr}");
    }
}

fn assert_runtime_entrypoint_rejection(
    command: &str,
    json_output: bool,
    extension: &str,
    source_contents: &str,
    expected_message: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command).arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(
            messages
                .iter()
                .any(|message| message.contains(expected_message)),
            "messages: {messages:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(stderr.contains(expected_message), "stderr: {stderr}");
    }
}

fn assert_runtime_entrypoint_rejection_when_browser_harness_is_configured(
    command: &str,
    json_output: bool,
    extension: &str,
    source_contents: &str,
    expected_messages: &[&str],
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source_contents).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path);
    let output = cli.output().expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(errors.iter().any(|error| error["code"] == "E5506"));
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("message"))
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| {
                expected_messages
                    .iter()
                    .any(|expected_message| message.contains(expected_message))
            }),
            "messages: {messages:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            expected_messages
                .iter()
                .any(|expected_message| stderr.contains(expected_message)),
            "stderr: {stderr}"
        );
    }
}

fn assert_optimization_benchmark_fixture(fixture_stem: &str, benchmark_name: &str) {
    let dir = tempdir().expect("tempdir");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(fixture_path(format!("benchmarks/{fixture_stem}.json")))
            .expect("read benchmark metadata"),
    )
    .expect("parse benchmark metadata");
    let source_file_name = metadata["sourceFile"]
        .as_str()
        .expect("benchmark source file name");
    let source_fixture = fixture_path(format!("benchmarks/{source_file_name}"));
    let source = fs::read_to_string(&source_fixture).expect("read benchmark source");
    let source_hash = format!("sha256-{:x}", Sha256::digest(source.as_bytes()));

    assert_eq!(metadata["benchmark"], benchmark_name);
    assert_eq!(metadata["version"], 1);
    assert!(
        metadata["sourceFile"] == json!(format!("{fixture_stem}.ts"))
            || metadata["sourceFile"] == json!(format!("{fixture_stem}.js")),
        "unexpected benchmark sourceFile for {fixture_stem}: {}",
        metadata["sourceFile"]
    );
    assert_eq!(metadata["sourceSha256"], source_hash);
    assert_eq!(
        metadata["buildModes"],
        json!(["--fast", "--release", "--release-advanced"])
    );

    let source_path = dir.path().join(source_file_name);
    fs::write(&source_path, source).expect("write benchmark source");

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

        let wasm_path = out_dir.join(format!(
            "{}.wasm",
            source_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("benchmark source stem")
        ));
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

    if !matches!(
        benchmark_name,
        "nullish-specialization"
            | "object-enumeration-delete-reinsert"
            | "reflect-own-keys-folding"
            | "reflect-own-keys-const-bound-literal"
            | "reflect-own-keys-alias-chain"
            | "array-literal-arguments"
            | "numeric-literal-arguments"
            | "math-max-min-builtin-js"
            | "spectral-norm"
            | "nbody"
    ) {
        assert!(
            release_size < fast_size
                || release_instructions < fast_instructions
                || release_adds < fast_adds,
            "expected release build to improve at least one footprint metric for {benchmark_name} (fast size={fast_size}, release size={release_size}; fast instructions={fast_instructions}, release instructions={release_instructions}; fast adds={fast_adds}, release adds={release_adds})"
        );
    }
    if !matches!(
        benchmark_name,
        "array-literal-arguments"
            | "numeric-literal-arguments"
            | "nested-call-inlining-chain"
            | "math-max-min-builtin-js"
    ) {
        assert!(
            advanced_size < release_size
                || advanced_instructions < release_instructions
                || advanced_adds < release_adds,
            "expected release-advanced build to improve at least one footprint metric further (release size={release_size}, advanced size={advanced_size}; release instructions={release_instructions}, advanced instructions={advanced_instructions}; release adds={release_adds}, advanced adds={advanced_adds})"
        );
    }

    if !matches!(
        benchmark_name,
        "array-literal-arguments"
            | "numeric-literal-arguments"
            | "nested-call-inlining-chain"
            | "math-max-min-builtin-js"
            | "spectral-norm"
            | "nbody"
    ) {
        assert!(
            release_adds <= fast_adds,
            "expected release build to avoid more add instructions than fast (fast={fast_adds}, release={release_adds})"
        );
    }
    assert!(
        advanced_adds <= release_adds,
        "expected release-advanced build to avoid more add instructions than release (release={release_adds}, advanced={advanced_adds})"
    );

    if benchmark_name != "nullish-specialization" {
        assert_eq!(
            fast_tag_ops, 0,
            "benchmark fast path should not box numeric ops"
        );
    }
    assert_eq!(
        release_tag_ops, 0,
        "benchmark release path should not box numeric ops"
    );
    assert_eq!(
        advanced_tag_ops, 0,
        "benchmark release-advanced path should not box numeric ops"
    );
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

fn assert_effects_rejects_wasm_threads_runtime_profile_in_browser_analysis_context(
    source_name: &str,
    explicit_browser_api_surface: bool,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(
        &source_path,
        "console.log('ok');\nfetch('https://example.com');",
    )
    .expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser",
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg("effects");
    if explicit_browser_api_surface {
        cli.arg("--api").arg("browser");
        cli.arg("--wasm-threads");
    }
    cli.arg(&source_path);

    let output = cli.output().expect("run kali");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(5));

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "effects");
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 5);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors: {errors:?}");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("runtime profile")
                || errors[0]["message"]
                    .as_str()
                    .expect("error message")
                    .contains("wasm-threads"),
            "json: {json}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("runtime profile") || stderr.contains("wasm-threads"),
            "stderr: {stderr}"
        );
    }
}

fn phase_three_deno_host_effects_source() -> &'static str {
    "Deno.env.set('KALI_CORPUS_FLAG', 'set');\nnew Deno.Command('sh').spawn();\nnew Deno[\"Command\"]('sh').spawn();\nDeno.connect('127.0.0.1', 1);\nglobalThis.Deno.connect('127.0.0.1', 1);\nglobalThis.Deno[\"connect\"]('127.0.0.1', 1);\nglobalThis[\"Deno\"].connect('127.0.0.1', 1);\nglobalThis[\"Deno\"][\"connect\"]('127.0.0.1', 1);\nDeno.listen('127.0.0.1', 0);\nglobalThis.Deno.listen('127.0.0.1', 0);\nglobalThis.Deno[\"listen\"]('127.0.0.1', 0);\nglobalThis[\"Deno\"].listen('127.0.0.1', 0);\nglobalThis[\"Deno\"][\"listen\"]('127.0.0.1', 0);\nDeno.serve('127.0.0.1', 0);\nglobalThis.Deno.serve('127.0.0.1', 0);\nglobalThis.Deno[\"serve\"]('127.0.0.1', 0);\nglobalThis[\"Deno\"].serve('127.0.0.1', 0);\nglobalThis[\"Deno\"][\"serve\"]('127.0.0.1', 0);\n"
}

fn deno_command_spawn_source() -> &'static str {
    "new Deno.Command('sh').spawn();\nnew Deno[\"Command\"]('sh').spawn();\n"
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

fn package_audit_metadata_body_with_multiple_findings() -> &'static str {
    Box::leak(
        json!({
            "versions": {
                "1.0.0": {
                    "name": "lodash",
                    "version": "1.0.0",
                    "main": "native.node",
                    "exports": "./native.node",
                    "bin": "./native.node",
                    "gypfile": true,
                    "scripts": {
                        "postinstall": "echo ok"
                    },
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash.tgz",
                        "integrity": "sha512-demo"
                    }
                }
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

fn package_audit_metadata_body_with_lifecycle_scripts() -> &'static str {
    Box::leak(
        json!({
            "versions": {
                "1.0.0": {
                    "name": "lodash",
                    "version": "1.0.0",
                    "scripts": {
                        "preinstall": "echo prep",
                        "install": "echo install",
                        "postinstall": "echo done"
                    },
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash.tgz",
                        "integrity": "sha512-demo"
                    }
                }
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

fn package_audit_metadata_body_with_stable_and_prerelease_versions() -> &'static str {
    Box::leak(
        json!({
            "versions": {
                "2.0.0-beta.1": {
                    "name": "lodash",
                    "version": "2.0.0-beta.1",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-prerelease.tgz",
                        "integrity": "sha512-demo"
                    }
                },
                "1.0.0": {
                    "name": "lodash",
                    "version": "1.0.0",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-stable.tgz",
                        "integrity": "sha512-demo"
                    }
                }
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

fn package_audit_metadata_body_with_multiple_stable_and_prerelease_versions() -> &'static str {
    Box::leak(
        json!({
            "versions": {
                "2.0.0-beta.1": {
                    "name": "lodash",
                    "version": "2.0.0-beta.1",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-prerelease.tgz",
                        "integrity": "sha512-demo"
                    }
                },
                "1.0.0": {
                    "name": "lodash",
                    "version": "1.0.0",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-stable-old.tgz",
                        "integrity": "sha512-demo"
                    }
                },
                "1.2.0": {
                    "name": "lodash",
                    "version": "1.2.0",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-stable-new.tgz",
                        "integrity": "sha512-demo"
                    }
                }
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

fn package_audit_metadata_body_with_prerelease_only_versions() -> &'static str {
    Box::leak(
        json!({
            "versions": {
                "2.0.0-beta.1": {
                    "name": "lodash",
                    "version": "2.0.0-beta.1",
                    "main": "index.js",
                    "dist": {
                        "tarball": "http://127.0.0.1:0/lodash-prerelease.tgz",
                        "integrity": "sha512-demo"
                    }
                }
            }
        })
        .to_string()
        .into_boxed_str(),
    )
}

#[path = "runtime_smoke/run.rs"]
mod run;

#[path = "runtime_smoke/test.rs"]
mod test;

#[path = "runtime_smoke/build.rs"]
mod build;

#[path = "runtime_smoke/check.rs"]
mod check;

#[path = "runtime_smoke/package.rs"]
mod package;

#[path = "runtime_smoke/effects.rs"]
mod effects;

#[path = "runtime_smoke/install.rs"]
mod install;

#[path = "runtime_smoke/misc.rs"]
mod misc;
