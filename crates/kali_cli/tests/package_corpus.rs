use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_browser_runtime_json_output(
    output: &std::process::Output,
    command: &str,
    expected_stdout: &str,
) {
    let json = parse_json_stdout(output);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    if command == "test" {
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
    }
    assert!(
        json["stdout"]
            .as_str()
            .expect("json stdout")
            .contains(expected_stdout),
        "json: {json}"
    );
}

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn node_process_corpus_body() -> String {
    let process_kill_zero_probe_source = kali_common::process_kill_zero_probe_source();
    let mut body = String::from(
        "export default function root() { const zero = 0; const zeroAlias = zero; return process.cwd().length > 0 && typeof process[\"cwd\"] === \"function\" && typeof globalThis.process[\"cwd\"] === \"function\" && typeof globalThis[\"process\"][\"cwd\"] === \"function\" && typeof process.chdir === \"function\" && typeof process[\"chdir\"] === \"function\" && typeof globalThis.process[\"chdir\"] === \"function\" && typeof globalThis[\"process\"][\"chdir\"] === \"function\" && process.pid > 0 && process.kill(zeroAlias) && ",
    );
    body.push_str(process_kill_zero_probe_source.trim_end_matches(';'));
    body.push_str(
        " && typeof process.exit === \"function\" && typeof process[\"exit\"] === \"function\" && typeof globalThis.process[\"exit\"] === \"function\" && typeof globalThis[\"process\"][\"exit\"] === \"function\" ? 0 : 1; }\n",
    );
    body
}

fn assert_node_process_corpus_has_zero_probe_inventory(body: &str) {
    for expected in [
        "const zero = 0",
        "const zeroAlias = zero",
        "process.kill(zeroAlias)",
    ] {
        assert!(
            body.contains(expected),
            "node process corpus should confirm {expected}"
        );
    }

    let zero_probe_source = kali_common::process_kill_zero_probe_source();
    let zero_probe_source = zero_probe_source.trim_end_matches(';');
    assert!(
        body.contains(zero_probe_source),
        "node process corpus should confirm the shared zero-probe source"
    );
    assert_eq!(
        body.matches(zero_probe_source).count(),
        1,
        "node process corpus should embed the shared zero-probe source exactly once"
    );

    for alias in kali_common::process_kill_zero_probe_aliases() {
        assert!(
            body.contains(alias),
            "node process corpus should confirm {alias}"
        );
    }
}

fn write_manifest(root: &Path, api_surface: Option<&str>) {
    let manifest = match api_surface {
        Some(api_surface) => format!(
            r#"{{
  "schemaVersion": 1,
  "compilerOptions": {{
    "apiSurface": "{}"
  }}
}}"#,
            api_surface
        ),
        None => r#"{"schemaVersion": 1}"#.to_string(),
    };
    fs::write(root.join("kali.json"), manifest).expect("write manifest");
}

fn write_stub_package(root: &Path, name: &str, body: &str) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js"
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), body).expect("write package entry");
}

fn write_semver_style_package(root: &Path) {
    fs::create_dir_all(root.join("bin")).expect("create package bin dir");
    fs::write(
        root.join("package.json"),
        r#"{
  "name": "semver",
  "version": "7.7.4",
  "main": "index.js",
  "exports": "./index.js",
  "bin": {
    "semver": "bin/semver.js"
  }
}"#,
    )
    .expect("write package.json");
    fs::write(
        root.join("index.js"),
        r#"export function valid(v) { return v; }
export function satisfies(version, range) { return version === '1.2.3' && range === '^1.0.0'; }
export function minVersion(range) { return { version: '1.2.3' }; }
"#,
    )
    .expect("write package entry");
    fs::write(
        root.join("bin/semver.js"),
        "#!/usr/bin/env node\nconst pkg = require('../package.json');\nconsole.log(pkg.version);\n",
    )
    .expect("write package bin");
}

fn write_pi_coding_agent_style_package(root: &Path) {
    fs::create_dir_all(root.join("dist")).expect("create package dist dir");
    fs::write(
        root.join("package.json"),
        r#"{
  "name": "@mariozechner/pi-coding-agent",
  "version": "0.70.0",
  "main": "dist/index.js",
  "exports": "./dist/index.js",
  "bin": {
    "pi": "dist/cli.js",
    "pi-argv": "dist/argv.js"
  },
  "engines": {
    "node": ">=20.0.0"
  }
}"#,
    )
    .expect("write package.json");
    fs::write(
        root.join("dist/index.js"),
        "module.exports = function codingAgent() { return 'pi'; };\n",
    )
    .expect("write package entry");
    fs::write(
        root.join("dist/cli.js"),
        "#!/usr/bin/env node\nconst pkg = require('../package.json');\nconsole.log(pkg.version);\n",
    )
    .expect("write package bin");
    fs::write(
        root.join("dist/argv.js"),
        "#!/usr/bin/env node\nconsole.log(process.argv.slice(2).length);\n",
    )
    .expect("write package argv bin");
}

fn write_native_addon_package(root: &Path) {
    fs::write(
        root.join("package.json"),
        r#"{
  "name": "native-addon",
  "main": "native.node"
}"#,
    )
    .expect("write native addon package.json");
    fs::write(root.join("native.node"), "placeholder").expect("write native addon entry");
}

fn write_web_baseline_interop_source(path: &Path, package: &str) {
    fs::write(
        path,
        format!(
            "import describe from '{package}';\nconst controller = new AbortController();\nconst signal = controller.signal;\nif (!(signal instanceof AbortSignal)) {{\n  throw new Error('expected AbortSignal from AbortController');\n}}\nsignal.addEventListener('abort', () => {{\n}});\nsignal.aborted;\nconst target = new EventTarget();\nlet count = 0;\ntarget.addEventListener('tick', () => {{\n  count += 1;\n  controller.abort();\n}});\nconst custom = new CustomEvent('tick');\ntarget.dispatchEvent(custom);\nconsole.log(custom.type);\ntarget.dispatchEvent(new Event('tick'));\nsignal.aborted;\nconst query = new URLSearchParams('alpha=1&beta=two+words');\nquery.append('gamma', String(count));\nquery.set('beta', describe(count));\nquery.get('alpha');\nquery.getAll('beta');\nquery.has('gamma');\nquery.toString();\nconst agent = navigator.userAgent;\nconst language = navigator.language;\nconst online = navigator.onLine;\nconsole.log(agent, language, online);\nconst now = performance.now();\nconsole.log(now);\nqueueMicrotask(() => {{\n  count += 0;\n}});\nconst encoder = new TextEncoder();\nconst encoded = encoder.encode(describe(count));\nconst decoder = new TextDecoder();\ndecoder.decode(encoded);\nconst encodedBinary = btoa(describe(count));\natob(encodedBinary);\nconst browserUrl = new URL('https://example.com/browser?alpha=1#fragment');\nbrowserUrl.pathname;\nbrowserUrl.search;\nbrowserUrl.hash;\nbrowserUrl.href;\nconst headers = new Headers();\nheaders.append('x-corpus', describe(count));\nheaders.set('accept', 'application/json');\nconst request = new Request('https://example.com/request');\nconst response = new Response('browser corpus');\nfetch('https://example.com/fetch');\nconsole.log(headers, request, response);\nconst blob = structuredClone(new Blob(['browser corpus']));\nconst file = structuredClone(new File(['browser corpus'], 'browser.txt'));\nconst form = new FormData();\nform.append('blob', blob);\nform.append('file', file);\nform.set('count', String(count));\nlocalStorage.clear();\nlocalStorage.setItem('count', String(count));\nlocalStorage.getItem('count');\nsessionStorage.clear();\nsessionStorage.setItem('count', String(count));\nsessionStorage.getItem('count');\nconst reader = new FileReader();\nreader.readAsText(blob);\nreader.readAsText(file);\nconst readableStream = new ReadableStream();\nreadableStream.getReader();\nconst writableStream = new WritableStream();\nwritableStream.getWriter();\nconst transformStream = new TransformStream();\ntransformStream.readable;\ntransformStream.writable;\nconst socket = new WebSocket('https://example.com/socket');\nsocket.sendText(describe(count));\nsocket.sendBytes(encoded);\nsocket.close();\nconst worker = new Worker('https://example.com/worker.js');\nworker.postMessage(describe(count));\nworker.terminate();\nconst channel = new BroadcastChannel('browser-corpus');\nchannel.postMessage(describe(count));\nchannel.close();\nif (typeof indexedDB !== 'undefined') {{\n  const database = indexedDB.open('browser-corpus');\n  database.put('events', 'count', String(count));\n  database.get('events', 'count');\n  database.storeNames();\n}}\nstructuredClone(blob);\nstructuredClone(file);\nblob.text();\nfile.text();\nconst legacyDigest = crypto.subtle.digest('SHA-1', encoded);\nconst digest = crypto.subtle.digest('SHA-256', encoded);\nconst strongerDigest384 = crypto.subtle.digest('SHA-384', encoded);\nconst strongerDigest512 = crypto.subtle.digest('SHA-512', encoded);\nconsole.log(legacyDigest, digest, strongerDigest384, strongerDigest512);\nconst randomUuid = crypto.randomUUID();\nconsole.log(randomUuid);\nconsole.log(describe(count));\n",
            package = package
        ),
    )
    .expect("write web baseline source");
}

fn write_web_baseline_test_source(path: &Path, package: &str) {
    fs::create_dir_all(path.parent().expect("web baseline test directory"))
        .expect("create web baseline test directory");
    fs::write(
        path,
        format!(
            "import describe from '{package}';\nKali.test('web-baseline corpus', () => {{\n  let count = 0;\n  const controller = new AbortController();\n  const signal = controller.signal;\n  if (!(signal instanceof AbortSignal)) {{\n    throw new Error('expected AbortSignal from AbortController');\n  }}\n  signal.addEventListener('abort', () => {{\n    count += 0;\n  }});\n  signal.aborted;\n  const target = new EventTarget();\n  target.addEventListener('tick', () => {{\n    count += 1;\n    controller.abort();\n  }});\n  target.dispatchEvent(new CustomEvent('tick'));\n  const query = new URLSearchParams('alpha=1&beta=two+words');\n  query.set('beta', describe(count));\n  const headers = new Headers();\n  headers.set('accept', describe(count));\n  const request = new Request('https://example.com/request');\n  const response = new Response('browser corpus');\n  const blob = structuredClone(new Blob(['browser corpus']));\n  const file = structuredClone(new File(['browser corpus'], 'browser.txt'));\n  const encoder = new TextEncoder();\n  const encoded = encoder.encode(describe(count));\n  const decoder = new TextDecoder();\n  decoder.decode(encoded);\n  const randomUuid = crypto.randomUUID();\n  queueMicrotask(() => {{\n    count += 0;\n  }});\n  console.log(count, request, response, blob, file, randomUuid);\n}});\n",
            package = package
        ),
    )
    .expect("write web baseline test source");
}

fn write_node_assuming_package(root: &Path, name: &str, body: &str) {
    write_stub_package(root, name, body);
    write_types_stub_package(root, name);
}

fn write_types_stub_package(root: &Path, name: &str) {
    let types_name = if let Some(rest) = name.strip_prefix('@') {
        let mut parts = rest.splitn(2, '/');
        let scope = parts.next().unwrap_or(rest);
        let package = parts.next().unwrap_or("");
        if package.is_empty() {
            format!("@types/{}", scope)
        } else {
            format!("@types/{}__{}", scope, package)
        }
    } else {
        format!("@types/{}", name)
    };
    let package_dir = root.join("node_modules").join(&types_name);
    fs::create_dir_all(&package_dir).expect("create types package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "types": "index.d.ts"
}}"#,
            types_name
        ),
    )
    .expect("write types package.json");
    fs::write(
        package_dir.join("index.d.ts"),
        "declare const value: unknown;\n",
    )
    .expect("write types package entry");
}

fn write_deno_host_package(root: &Path, name: &str, body: &str) {
    write_stub_package(root, name, body);
    write_types_stub_package(root, name);
}

fn write_jsr_package(root: &Path, name: &str, body: &str) {
    let package_name = name
        .strip_prefix("jsr:")
        .expect("canonical jsr package identifier");
    write_stub_package(root, package_name, body);
    write_types_stub_package(root, package_name);
}

fn write_export_map_package(
    root: &Path,
    name: &str,
    root_body: &str,
    subpath: &str,
    subpath_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "exports": {{
    ".": "./index.js",
    "./{}": "./{}.js"
  }}
}}"#,
            name, subpath, subpath
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_body).expect("write package root");
    fs::write(package_dir.join(format!("{}.js", subpath)), subpath_body)
        .expect("write package subpath");
}

fn write_pattern_exports_package(
    root: &Path,
    name: &str,
    root_body: &str,
    subpath: &str,
    subpath_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "exports": {{
    ".": "./index.js",
    "./*": "./src/*.js"
  }}
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_body).expect("write package root");
    let subpath_dir = package_dir.join("src");
    fs::create_dir_all(&subpath_dir).expect("create package subpath dir");
    fs::write(subpath_dir.join(format!("{}.js", subpath)), subpath_body)
        .expect("write package subpath");
}

fn write_typed_export_map_package(
    root: &Path,
    name: &str,
    root_body: &str,
    subpath: &str,
    subpath_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "exports": {{
    ".": {{
      "types": "./index.d.ts",
      "default": "./index.js"
    }},
    "./{}": {{
      "types": "./{}.d.ts",
      "default": "./{}.js"
    }}
  }}
}}"#,
            name, subpath, subpath, subpath
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_body).expect("write package root");
    fs::write(
        package_dir.join("index.d.ts"),
        "export declare const value: string;\n",
    )
    .expect("write package root types");
    fs::write(package_dir.join(format!("{}.js", subpath)), subpath_body)
        .expect("write package subpath");
    fs::write(
        package_dir.join(format!("{}.d.ts", subpath)),
        "export declare const value: string;\n",
    )
    .expect("write package subpath types");
}

fn write_dual_exports_package(
    root: &Path,
    name: &str,
    root_import_body: &str,
    root_require_body: &str,
    subpath: &str,
    subpath_import_body: &str,
    subpath_require_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "exports": {{
    ".": {{
      "import": "./index.mjs",
      "require": "./index.cjs"
    }},
    "./{}": {{
      "import": "./{}.mjs",
      "require": "./{}.cjs"
    }}
  }}
}}"#,
            name, subpath, subpath, subpath
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.mjs"), root_import_body).expect("write package root import");
    fs::write(package_dir.join("index.cjs"), root_require_body)
        .expect("write package root require");
    fs::write(
        package_dir.join(format!("{}.mjs", subpath)),
        subpath_import_body,
    )
    .expect("write package subpath import");
    fs::write(
        package_dir.join(format!("{}.cjs", subpath)),
        subpath_require_body,
    )
    .expect("write package subpath require");
}

fn write_mixed_format_package(
    root: &Path,
    name: &str,
    root_cjs_body: &str,
    root_esm_body: &str,
    subpath: &str,
    subpath_cjs_body: &str,
    subpath_esm_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.cjs",
  "module": "index.mjs"
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.cjs"), root_cjs_body).expect("write package root cjs");
    fs::write(package_dir.join("index.mjs"), root_esm_body).expect("write package root esm");
    fs::write(
        package_dir.join(format!("{}.cjs", subpath)),
        subpath_cjs_body,
    )
    .expect("write package subpath cjs");
    fs::write(
        package_dir.join(format!("{}.mjs", subpath)),
        subpath_esm_body,
    )
    .expect("write package subpath esm");
}

fn write_module_only_package(root: &Path, name: &str, body: &str) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "module": "index.mjs"
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.mjs"), body).expect("write package module entry");
}

#[allow(clippy::too_many_arguments)]
fn write_browser_condition_exports_package(
    root: &Path,
    name: &str,
    root_browser_body: &str,
    root_import_body: &str,
    root_require_body: &str,
    subpath: &str,
    subpath_browser_body: &str,
    subpath_import_body: &str,
    subpath_require_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "./index.js",
  "exports": {{
    ".": {{
      "browser": "./index.browser.js",
      "import": "./index.import.js",
      "require": "./index.require.cjs",
      "default": "./index.import.js"
    }},
    "./{}": {{
      "browser": "./{}.browser.js",
      "import": "./{}.import.js",
      "require": "./{}.require.cjs",
      "default": "./{}.import.js"
    }}
  }}
}}"#,
            name, subpath, subpath, subpath, subpath, subpath
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_browser_body).expect("write package root main");
    fs::write(package_dir.join("index.browser.js"), root_browser_body)
        .expect("write package root browser");
    fs::write(package_dir.join("index.import.js"), root_import_body)
        .expect("write package root import");
    fs::write(package_dir.join("index.require.cjs"), root_require_body)
        .expect("write package root require");
    fs::write(
        package_dir.join(format!("{}.browser.js", subpath)),
        subpath_browser_body,
    )
    .expect("write package subpath browser");
    fs::write(
        package_dir.join(format!("{}.import.js", subpath)),
        subpath_import_body,
    )
    .expect("write package subpath import");
    fs::write(
        package_dir.join(format!("{}.require.cjs", subpath)),
        subpath_require_body,
    )
    .expect("write package subpath require");
}

fn write_browser_and_deno_condition_package(
    root: &Path,
    name: &str,
    browser_body: &str,
    deno_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "exports": {{
    ".": {{
      "browser": "./index.browser.js",
      "deno": "./index.deno.js",
      "default": "./index.deno.js"
    }}
  }}
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.browser.js"), browser_body)
        .expect("write package browser entry");
    fs::write(package_dir.join("index.deno.js"), deno_body).expect("write package deno entry");
}

fn write_browser_replacement_map_package(
    root: &Path,
    name: &str,
    root_node_body: &str,
    root_browser_body: &str,
    subpath: &str,
    subpath_node_body: &str,
    subpath_browser_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "browser": {{
    "./index.js": "./index.browser.js",
    "./{}.js": "./{}.browser.js"
  }}
}}"#,
            name, subpath, subpath
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_node_body).expect("write package root node");
    fs::write(package_dir.join("index.browser.js"), root_browser_body)
        .expect("write package root browser");
    fs::write(
        package_dir.join(format!("{}.js", subpath)),
        subpath_node_body,
    )
    .expect("write package subpath node");
    fs::write(
        package_dir.join(format!("{}.browser.js", subpath)),
        subpath_browser_body,
    )
    .expect("write package subpath browser");
}

fn write_browser_string_package(
    root: &Path,
    name: &str,
    root_node_body: &str,
    root_browser_body: &str,
) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "browser": "./index.browser.js"
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_node_body).expect("write package root node");
    fs::write(package_dir.join("index.browser.js"), root_browser_body)
        .expect("write package root browser");
}

fn write_browser_string_web_baseline_package(root: &Path, name: &str) {
    write_browser_string_package(
        root,
        name,
        &format!(
            "export default function root() {{ return '{name}:node'; }}\n",
            name = name
        ),
        &format!(
            "const controller = new AbortController();\nconst signal = controller.signal;\nsignal.addEventListener('abort', () => {{\n}});\nconst target = new EventTarget();\ntarget.addEventListener('tick', () => {{\n  controller.abort();\n}});\ntarget.dispatchEvent(new CustomEvent('tick'));\nconst query = new URLSearchParams('alpha=1&beta=two+words');\nquery.set('beta', '{name}');\nstructuredClone(new Blob(['browser corpus']));\nconst encoder = new TextEncoder();\nencoder.encode('browser corpus');\nexport default function root() {{ return '{name}:browser'; }}\n",
            name = name
        ),
    );
}

fn write_string_exports_package(root: &Path, name: &str, root_body: &str) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "exports": "./index.js"
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), root_body).expect("write package root");
}

fn write_browser_blocked_package(root: &Path, name: &str, body: &str) {
    let package_dir = root.join("node_modules").join(name);
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        package_dir.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "main": "index.js",
  "browser": {{
    "./index.js": false
  }}
}}"#,
            name
        ),
    )
    .expect("write package.json");
    fs::write(package_dir.join("index.js"), body).expect("write package entry");
}

fn run_kali<I, S>(root: &Path, args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(kali_bin())
        .current_dir(root)
        .args(args)
        .output()
        .expect("run kali")
}

fn assert_browser_blocked_package_json_rejection(output: &std::process::Output, command: &str) {
    assert!(!output.status.success(), "expected {command} to fail");
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        errors.iter().any(|error| error["code"] == "E3000"),
        "missing E3000 in {json}"
    );
    assert!(
        errors.iter().any(|error| {
            error["message"]
                .as_str()
                .expect("error message")
                .contains("could not be resolved")
        }),
        "missing resolution failure in {json}"
    );
}

fn assert_package_analysis_specific_flag_json_rejection(
    output: &std::process::Output,
    command: &str,
    expected_flag: &str,
) {
    assert!(!output.status.success(), "expected {command} to fail");
    assert_eq!(output.status.code(), Some(5));

    let json = parse_json_stdout(output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], false);
    assert_eq!(json["exitCode"], 5);
    assert_eq!(json["payload"], serde_json::Value::Null);

    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1, "json: {json}");
    let error = errors.first().expect("first error");
    assert_eq!(error["code"], "E5508");
    assert!(error["message"]
        .as_str()
        .expect("error message")
        .contains("package-analysis-specific flags"));
    assert_eq!(error["context"]["origin"], "cli");
    assert_eq!(error["context"]["flag"], expected_flag);
}

#[path = "package_corpus/browser_runtime.rs"]
mod browser_runtime;

#[path = "package_corpus/browser_corpus.rs"]
mod browser_corpus;

#[path = "package_corpus/utility.rs"]
mod utility;

#[path = "package_corpus/node.rs"]
mod node;

#[path = "package_corpus/misc.rs"]
mod misc;
