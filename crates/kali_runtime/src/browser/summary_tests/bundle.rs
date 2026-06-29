use super::*;

#[test]
fn browser_bundle_runtime_summary_merges_missing_tests_failed_from_stdout() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_merges_thread_topology_from_stdout_when_summary_file_is_missing_it(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_keeps_thread_topology_from_summary_file() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["alpha".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_thread_topology_is_invalid(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser merge\"],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\" https://example.com/thread.js \",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\",\"threadTopology\":{\"totalInstances\":1,\"terminatedInstances\":0,\"liveInstances\":[{\"instanceId\":0,\"scriptUrl\":\"https://example.com/stdout-thread.js\",\"postedMessages\":[],\"postedSharedBuffers\":[],\"wasTerminated\":false}]}}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.thread_topology.total_instances, 1);
    assert_eq!(outcome.thread_topology.terminated_instances, 0);
    assert_eq!(outcome.thread_topology.live_instances.len(), 1);
    assert_eq!(outcome.thread_topology.live_instances[0].instance_id, 0);
    assert_eq!(
        outcome.thread_topology.live_instances[0].script_url,
        "https://example.com/stdout-thread.js"
    );
    assert_eq!(
        outcome.thread_topology.snapshot_value(),
        serde_json::json!({
            "totalInstances": 1,
            "terminatedInstances": 0,
            "liveInstances": [{
                "instanceId": 0,
                "scriptUrl": "https://example.com/stdout-thread.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }]
        })
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_merges_stdout_tests_failed_when_summary_file_has_null_value() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"alpha\"],\"tests\":[\"browser merge\"],\"testsFailed\":null,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser merge\"],\"testsFailed\":1,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["alpha".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 1);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["browser merge".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_missing() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser missing\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser missing".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[cfg(unix)]
#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unreadable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, "{\"args\":[\"alpha\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); fs.chmodSync(summary, 0o000); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"browser unreadable\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser unreadable".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_whitespace_only() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); const summary = process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE; fs.writeFileSync(summary, " \n\t\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_is_unparseable() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "not-json"); process.stdout.write("{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_metadata_when_summary_file_has_invalid_labels() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":2,\"hostContract\":\"not-a-contract\",\"runtimeBackend\":\"not-a-backend\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_metadata_when_summary_file_has_whitespace_only_labels(
) {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":2,\"hostContract\":\"   \",\"runtimeBackend\":\"   \"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser whitespace labels\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 2);
    assert_eq!(outcome.reported_args, vec!["summary".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser whitespace labels".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome
            .stdout
            .contains("\"hostContract\":\"browser-requested\""),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome
            .stdout
            .contains("\"runtimeBackend\":\"browser-harness\""),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_falls_back_to_stdout_when_summary_file_has_invalid_array_items() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"summary\"],\"tests\":[1],\"testsFailed\":2,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"browser invalid array items\"],\"testsFailed\":8,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 8);
    assert_eq!(outcome.reported_args, vec!["stdout".to_string()]);
    assert_eq!(
        outcome.registered_tests,
        vec!["browser invalid array items".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert!(
        outcome.stdout.contains("\"testsFailed\":8"),
        "stdout: {}",
        outcome.stdout
    );
    assert_eq!(outcome.tests_run(), 1);
}

#[test]
fn browser_bundle_runtime_summary_uses_stdout_labels_when_summary_file_lacks_them() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let bundle_root = tempdir.path().join("browser-app");
    fs::create_dir_all(&bundle_root).expect("create bundle root");

    fs::write(
        bundle_root.join("browser-app.wasm"),
        compile_wat(
            r#"
                (module
                    (func (export "_start")))
            "#,
        ),
    )
    .expect("write bundle wasm");
    fs::write(
        bundle_root.join("browser-app.js"),
        r#"
const wasmUrl = new URL('./browser-app.wasm', import.meta.url);

export async function loadWithImports(importObject) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, importObject);
  return instance;
}
"#,
    )
    .expect("write bundle js");

    let command = r#"node -e 'const fs = require("fs"); fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, "{\"args\":[\"zeta\"],\"tests\":[\"7\"],\"testsFailed\":0}\n"); process.stdout.write("{\"args\":[\"stdout\"],\"tests\":[\"stdout\"],\"testsFailed\":0,\"hostContract\":\"browser-requested\",\"runtimeBackend\":\"browser-harness\"}\n");'"#;
    let outcome = browser_bundle_runtime_execute_checked(
        Some(command),
        &bundle_root,
        &["zeta".to_string()],
        false,
        true,
    )
    .expect("execute browser bundle runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert_eq!(outcome.tests_failed, 0);
    assert_eq!(outcome.reported_args, vec!["zeta".to_string()]);
    assert_eq!(outcome.registered_tests, vec!["7".to_string()]);
    assert_eq!(outcome.host_contract, RuntimeHostContract::BrowserRequested);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::BrowserHarness);
    assert_eq!(outcome.tests_run(), 1);
}
