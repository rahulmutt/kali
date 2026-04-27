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

fn write_web_baseline_interop_source(path: &Path, package: &str) {
    fs::write(
        path,
        format!(
            "import describe from '{package}';\nconst controller = new AbortController();\nconst signal = controller.signal;\nsignal.addEventListener('abort', () => {{\n}});\nsignal.aborted;\nconst target = new EventTarget();\nlet count = 0;\ntarget.addEventListener('tick', () => {{\n  count += 1;\n  controller.abort();\n}});\nconst custom = new CustomEvent('tick');\ntarget.dispatchEvent(custom);\nconsole.log(custom.type);\ntarget.dispatchEvent(new Event('tick'));\nsignal.aborted;\nconst query = new URLSearchParams('alpha=1&beta=two+words');\nquery.append('gamma', String(count));\nquery.set('beta', describe(count));\nquery.get('alpha');\nquery.getAll('beta');\nquery.has('gamma');\nquery.toString();\nconst agent = navigator.userAgent;\nconst language = navigator.language;\nconst online = navigator.onLine;\nconsole.log(agent, language, online);\nconst now = performance.now();\nconsole.log(now);\nqueueMicrotask(() => {{\n  count += 0;\n}});\nconst encoder = new TextEncoder();\nconst encoded = encoder.encode(describe(count));\nconst decoder = new TextDecoder();\ndecoder.decode(encoded);\nconst encodedBinary = btoa(describe(count));\natob(encodedBinary);\nconst browserUrl = new URL('https://example.com/browser?alpha=1#fragment');\nbrowserUrl.pathname;\nbrowserUrl.search;\nbrowserUrl.hash;\nbrowserUrl.href;\nconst headers = new Headers();\nheaders.append('x-corpus', describe(count));\nheaders.set('accept', 'application/json');\nconst request = new Request('https://example.com/request');\nconst response = new Response('browser corpus');\nfetch('https://example.com/fetch');\nconsole.log(headers, request, response);\nconst blob = structuredClone(new Blob(['browser corpus']));\nconst file = structuredClone(new File(['browser corpus'], 'browser.txt'));\nconst form = new FormData();\nform.append('blob', blob);\nform.append('file', file);\nform.set('count', String(count));\nlocalStorage.clear();\nlocalStorage.setItem('count', String(count));\nlocalStorage.getItem('count');\nsessionStorage.clear();\nsessionStorage.setItem('count', String(count));\nsessionStorage.getItem('count');\nconst reader = new FileReader();\nreader.readAsText(blob);\nreader.readAsText(file);\nconst readableStream = new ReadableStream();\nreadableStream.getReader();\nconst writableStream = new WritableStream();\nwritableStream.getWriter();\nconst transformStream = new TransformStream();\ntransformStream.readable;\ntransformStream.writable;\nconst socket = new WebSocket('https://example.com/socket');\nsocket.sendText(describe(count));\nsocket.sendBytes(encoded);\nsocket.close();\nconst worker = new Worker('https://example.com/worker.js');\nworker.postMessage(describe(count));\nworker.terminate();\nconst channel = new BroadcastChannel('browser-corpus');\nchannel.postMessage(describe(count));\nchannel.close();\nif (typeof indexedDB !== 'undefined') {{\n  const database = indexedDB.open('browser-corpus');\n  database.put('events', 'count', String(count));\n  database.get('events', 'count');\n  database.storeNames();\n}}\nstructuredClone(blob);\nstructuredClone(file);\nblob.text();\nfile.text();\nconst legacyDigest = crypto.subtle.digest('SHA-1', encoded);\nconst digest = crypto.subtle.digest('SHA-256', encoded);\nconst strongerDigest384 = crypto.subtle.digest('SHA-384', encoded);\nconst strongerDigest512 = crypto.subtle.digest('SHA-512', encoded);\nconsole.log(legacyDigest, digest, strongerDigest384, strongerDigest512);\nconst randomUuid = crypto.randomUUID();\nconsole.log(randomUuid);\nconsole.log(describe(count));\n",
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
            "import describe from '{package}';\nKali.test('web-baseline corpus', () => {{\n  let count = 0;\n  const controller = new AbortController();\n  const signal = controller.signal;\n  signal.addEventListener('abort', () => {{\n    count += 0;\n  }});\n  signal.aborted;\n  const target = new EventTarget();\n  target.addEventListener('tick', () => {{\n    count += 1;\n    controller.abort();\n  }});\n  target.dispatchEvent(new CustomEvent('tick'));\n  const query = new URLSearchParams('alpha=1&beta=two+words');\n  query.set('beta', describe(count));\n  const headers = new Headers();\n  headers.set('accept', describe(count));\n  const request = new Request('https://example.com/request');\n  const response = new Response('browser corpus');\n  const blob = structuredClone(new Blob(['browser corpus']));\n  const file = structuredClone(new File(['browser corpus'], 'browser.txt'));\n  const encoder = new TextEncoder();\n  const encoded = encoder.encode(describe(count));\n  const decoder = new TextDecoder();\n  decoder.decode(encoded);\n  const randomUuid = crypto.randomUUID();\n  queueMicrotask(() => {{\n    count += 0;\n  }});\n  console.log(count, request, response, blob, file, randomUuid);\n}});\n",
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

#[test]
fn browser_corpus_packages_remain_checkable_and_deployable_through_host() {
    for package in [
        "react",
        "preact",
        "vue",
        "react-dom",
        "chart.js",
        "framer-motion",
        "clsx",
        "classnames",
        "reselect",
        "rxjs",
        "vue-router",
        "@testing-library/react",
        "@testing-library/dom",
        "nanostores",
        "@storybook/react",
        "@remix-run/react",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function widget() { return 'ok'; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import '{}';\nconsole.log('browser corpus: {}');\n",
                package, package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_semver_style_package_remains_checkable_and_deployable_through_host_on_js_input() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser semver package should be checkable on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser semver package should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn browser_corpus_pi_coding_agent_style_package_remains_checkable_and_deployable_through_host_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser JS source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be deployable-through-host via browser bundle in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_corpus_packages_with_exports_maps_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("hono", "client"),
        ("svelte", "compiler"),
        ("lit", "decorators"),
        ("solid-js", "store"),
        ("vue-router", "history"),
        ("react-router", "dom"),
        ("react-router-dom", "dom"),
        ("jotai", "store"),
        ("xstate", "react"),
        ("@remix-run/react", "links"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with exports map should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_exports_maps_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("hono", "client"),
        ("svelte", "compiler"),
        ("lit", "decorators"),
        ("solid-js", "store"),
        ("vue-router", "history"),
        ("react-router", "dom"),
        ("react-router-dom", "dom"),
        ("jotai", "store"),
        ("xstate", "react"),
        ("@remix-run/react", "links"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with exports map should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with exports map should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_exports_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [("date-fns", "formatISO"), ("@vueuse/core", "helpers")] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser exports-map package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let json = parse_json_stdout(&run);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"].as_str().expect("stdout").contains("0\n"),
            "json: {json}"
        );

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser exports-map package', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser exports-map package {package} should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let json = parse_json_stdout(&test);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"].as_str().expect("stdout").contains("0\n0\n"),
            "json: {json}"
        );
    }
}

#[test]
fn browser_corpus_packages_with_pattern_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
        ("vue-router", "matcher"),
        ("react-router", "routes"),
        ("react-router-dom", "dom"),
        ("xstate", "react"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser pattern-export package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser pattern-export package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_pattern_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
        ("vue-router", "matcher"),
        ("react-router", "routes"),
        ("react-router-dom", "dom"),
        ("xstate", "react"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser pattern-export package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser pattern-export package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_typed_export_branches_remain_checkable_and_deployable_through_host()
{
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with typed export branches should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with typed export branches should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_typed_export_branches_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with typed export branches should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with typed export branches should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_dual_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "module.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser dual package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser dual package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_dual_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "module.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser dual package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser dual package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("jotai", "store"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser conditional-exports package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser conditional-exports package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("jotai", "store"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser conditional-exports package {package} should resolve its browser branch on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser conditional-exports package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entries_remain_checkable_and_deployable_through_host() {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-only package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entries_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-only package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_module_only_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_with_json_output_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entry_chains_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-chain package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-chain package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_replacement_maps_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser replacement-map package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser replacement-map package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_js_entrypoints_with_browser_replacement_maps_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser JS entrypoint package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser JS entrypoint package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_js_entrypoints_with_minimized_cjs_esm_interop_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "module.exports = function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser mixed-format package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser mixed-format package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_explicit_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_mixed_format_package(
            dir.path(),
            package,
            "module.exports = function root() { return 0; }\n",
            "export default function root() { return 0; }\n",
            subpath,
            "module.exports = function subpath() { return 0; }\n",
            "export default function subpath() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser mixed-format package', () => {{\n  if (root() !== 0 || subpath() !== 0) {{\n    throw new Error('browser mixed-format package export mismatch');\n  }}\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_mixed_format_package(
            dir.path(),
            package,
            "module.exports = function root() { return 0; }\n",
            "export default function root() { return 0; }\n",
            subpath,
            "module.exports = function subpath() { return 0; }\n",
            "export default function subpath() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser mixed-format package', () => {{\n  if (root() !== 0 || subpath() !== 0) {{\n    throw new Error('browser mixed-format package export mismatch');\n  }}\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_remain_checkable_and_deployable_through_host(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-entry package {package} should resolve its browser override\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-entry package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-entry package {package} should resolve its browser override on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-entry package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_string_exports_remain_checkable_and_deployable_through_host() {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-exports package {package} should resolve its exports string\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-exports package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_string_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-exports package {package} should resolve its exports string on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-exports package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_string_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-exports package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-exports package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
        "d3",
        "recharts",
        "luxon",
        "graphql",
        "lodash-es",
        "nanoid",
        "ramda",
        "rxjs",
        "uuid",
        "clsx",
        "react-router",
        "zustand",
        "zod",
        "svelte",
        "lit",
        "axios",
        "ajv",
        "immer",
        "next",
        "react-helmet-async",
        "hono",
        "@vueuse/core",
        "@apollo/client",
        "@emotion/react",
        "@reduxjs/toolkit",
        "@floating-ui/react",
        "@headlessui/react",
        "@chakra-ui/react",
        "@mantine/core",
        "@emotion/styled",
        "@heroicons/react",
        "lucide-react",
        "@storybook/react",
        "@stripe/react-stripe-js",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
        "@tanstack/react-virtual",
        "@testing-library/dom",
        "@testing-library/user-event",
        "@playwright/test",
        "mobx",
        "redux",
        "recoil",
        "mitt",
        "swr",
        "formik",
        "jotai",
        "pinia",
        "xstate",
        "valtio",
        "superjson",
        "@jridgewell/sourcemap-codec",
        "@babel/runtime",
        "@npmcli/package-json",
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
        "@tanstack/react-router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        write_web_baseline_interop_source(&source_path, package);

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser web-baseline package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser web-baseline package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
        "d3",
        "recharts",
        "luxon",
        "graphql",
        "lodash-es",
        "nanoid",
        "ramda",
        "rxjs",
        "uuid",
        "clsx",
        "react-router",
        "zustand",
        "zod",
        "svelte",
        "lit",
        "axios",
        "ajv",
        "immer",
        "next",
        "react-helmet-async",
        "hono",
        "@vueuse/core",
        "@apollo/client",
        "@emotion/react",
        "@reduxjs/toolkit",
        "@floating-ui/react",
        "@headlessui/react",
        "@chakra-ui/react",
        "@mantine/core",
        "@emotion/styled",
        "@heroicons/react",
        "lucide-react",
        "@storybook/react",
        "@stripe/react-stripe-js",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
        "@tanstack/react-virtual",
        "@testing-library/dom",
        "@testing-library/user-event",
        "@playwright/test",
        "mobx",
        "redux",
        "recoil",
        "mitt",
        "swr",
        "formik",
        "jotai",
        "pinia",
        "xstate",
        "valtio",
        "superjson",
        "@jridgewell/sourcemap-codec",
        "@babel/runtime",
        "@npmcli/package-json",
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
        "@tanstack/react-router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser web-baseline package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser web-baseline package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_internal_browser_rewrites_remain_checkable_and_deployable_through_host(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function root() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function root() { return 'browser:' + helper(); }\n",
            "internal",
            &format!(
                "export default function helper() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function helper() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser internal-browser-rewrite package {package} should resolve its browser rewrite chain\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser internal-browser-rewrite package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_exports_maps_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
        ("chart.js", "auto"),
        ("zustand", "vanilla"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with exports map should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_exports_maps_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
        ("chart.js", "auto"),
        ("zustand", "vanilla"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with exports map should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with exports map should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_internal_browser_rewrites_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function root() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function root() { return 'browser:' + helper(); }\n",
            "internal",
            &format!(
                "export default function helper() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function helper() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser internal-browser-rewrite package {package} should resolve its browser rewrite chain on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser internal-browser-rewrite package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_browser_condition_exports_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("zustand", "vanilla"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with browser condition exports should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with browser condition exports should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_browser_condition_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("zustand", "vanilla"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with browser condition exports should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with browser condition exports should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return Deno.env.get('HOME') ? 1 : 2; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser condition package browser-deno should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return Deno.env.get('HOME') ? 1 : 2; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser condition package browser-deno should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_runtime_corpus_packages_remain_executable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_remain_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured_in_js_run(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_run(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_test(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_test(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_with_exports_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [("date-fns", "formatISO"), ("@vueuse/core", "helpers")] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser exports-map package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser exports-map package', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser exports-map package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_pattern_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser pattern-export package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser pattern-export package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_pattern_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser pattern-export package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser pattern-export package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-entry package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-entry package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-entry package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            "export default function root() { return 1; }\n",
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-entry package {package} should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-entry package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-entry package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_string_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-exports package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-exports package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.ts");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_dual_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!("export default function root() {{ return '{package}:import'; }}\n", package = package),
            &format!("module.exports = function root() {{ return '{package}:require'; }};\n", package = package),
            subpath,
            &format!("export default function subpath() {{ return '{package}:{subpath}:import'; }}\n", package = package, subpath = subpath),
            &format!("module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n", package = package, subpath = subpath),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser dual package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser dual package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_dual_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!("export default function root() {{ return '{package}:import'; }}\n", package = package),
            &format!("module.exports = function root() {{ return '{package}:require'; }};\n", package = package),
            subpath,
            &format!("export default function subpath() {{ return '{package}:{subpath}:import'; }}\n", package = package, subpath = subpath),
            &format!("module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n", package = package, subpath = subpath),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser dual package {package} should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");
        assert_eq!(run_stdout.lines().count(), 1, "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser dual package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_typed_export_branches_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser package {package} with typed export branches should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser package {package} with typed export branches should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_typed_export_branches_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser package {package} with typed export branches should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser package {package} with typed export branches should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(stdout.contains("0"), "stdout: {stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn browser_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context() {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_blocked_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
        );
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            !check.status.success(),
            "browser-blocked package {package} should be rejected in browser context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let stderr = String::from_utf8_lossy(&check.stderr);
        assert!(
            stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the import-resolution failure\nstderr: {}",
            stderr
        );
        assert!(
            stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry\nstderr: {}",
            stderr
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            !build.status.success(),
            "browser-blocked package {package} should also be rejected during bundle emission\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            build_stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the bundle-time import-resolution failure\nstderr: {}",
            build_stderr
        );

        let json_check = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_check, "check");

        let json_build = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_build, "build");
    }
}

#[test]
fn browser_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context_on_js_input(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_blocked_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
        );
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser JS source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            !check.status.success(),
            "browser-blocked package {package} should be rejected in browser context on JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let stderr = String::from_utf8_lossy(&check.stderr);
        assert!(
            stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the import-resolution failure on JS input\nstderr: {}",
            stderr
        );
        assert!(
            stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry on JS input\nstderr: {}",
            stderr
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            !build.status.success(),
            "browser-blocked package {package} should also be rejected during bundle emission on JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            build_stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the bundle-time import-resolution failure on JS input\nstderr: {}",
            build_stderr
        );

        let json_check = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_check, "check");

        let json_build = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_build, "build");
    }
}

#[test]
fn utility_corpus_packages_remain_executable_on_the_default_standalone_surface() {
    for package in [
        "react",
        "preact",
        "ramda",
        "rambda",
        "rxjs",
        "immer",
        "uuid",
        "typescript",
        "esbuild",
        "date-fns",
        "dayjs",
        "luxon",
        "axios",
        "camelcase",
        "deepmerge",
        "cheerio",
        "graphql",
        "lodash",
        "lodash-es",
        "commander",
        "redux",
        "reselect",
        "recoil",
        "clsx",
        "classnames",
        "zustand",
        "mitt",
        "query-string",
        "formik",
        "jotai",
        "yup",
        "yaml",
        "xstate",
        "valtio",
        "react-hook-form",
        "msw",
        "superjson",
        "chart.js",
        "recharts",
        "d3",
        "@jridgewell/sourcemap-codec",
        "@emotion/react",
        "@emotion/styled",
        "@mantine/core",
        "lucide-react",
        "vite",
        "tailwindcss",
        "@tanstack/router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            "export default function widget() { return 'ok'; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import '{}';\nconsole.log('utility corpus: {}');\n",
                package, package
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_exports_maps_remain_executable_on_the_default_standalone_surface() {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("redux", "createStore"),
        ("reselect", "selectors"),
        ("xstate", "react"),
        ("lodash", "get"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "utility package {package} with exports map should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility package {package} with exports map should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_string_exports_remain_executable_on_the_default_standalone_surface()
{
    for package in ["ramda", "rxjs", "uuid", "commander", "redux", "lodash"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility string-exports package {package} should resolve its exports string\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility string-exports package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_string_exports_remain_testable_on_the_default_standalone_surface_in_js_input(
) {
    for package in ["ramda", "rxjs", "uuid", "commander", "redux", "lodash"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir.path().join("smoke.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('string exports corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write utility test source");

        let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "utility string-exports package {package} should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write semver source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "semver corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write semver source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
Kali.test('semver corpus', () => {
  console.log(valid('1.2.3'));
  console.log(satisfies('1.2.3', '^1.0.0'));
  console.log(minVersion('^1.2.3')?.version);
});
"#,
    )
    .expect("write semver test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "semver corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "semver corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("1.2.3"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_executable_and_testable_on_the_node_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write semver source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
Kali.test('semver corpus', () => {
  console.log(valid('1.2.3'));
  console.log(satisfies('1.2.3', '^1.0.0'));
  console.log(minVersion('^1.2.3')?.version);
});
"#,
    )
    .expect("write semver test source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "node", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build_out_dir = dir.path().join("build");
    let build = run_kali(
        dir.path(),
        [
            "build",
            "--api",
            "node",
            "--out-dir",
            build_out_dir.to_str().unwrap(),
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "semver corpus package should be buildable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "node", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");

    let test = run_kali(
        dir.path(),
        ["test", "--api", "node", test_path.to_str().unwrap()],
    );
    assert!(
        test.status.success(),
        "semver corpus package should be testable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("1.2.3"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_date_fns_style_package_remains_checkable_buildable_testable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "date-fns",
        "export function addDays(date, amount) { return 0; }\nexport function format(date) { return 0; }\n",
        "formatISO",
        "export function formatISO(date) { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "date-fns");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
console.log(addDays('2024-01-01', 3));
console.log(format('2024-01-01'));
console.log(formatISO('2024-01-01'));
"#,
    )
    .expect("write date-fns source");
    let test_path = dir.path().join("tests").join("date-fns.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns test source");
    let ts_test_path = dir.path().join("tests").join("date-fns.test.ts");
    fs::write(
        &ts_test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns TS test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "date-fns corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "date-fns corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "date-fns corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n0\n0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");

    let ts_test = run_kali(dir.path(), ["test", ts_test_path.to_str().unwrap()]);
    assert!(
        ts_test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ts_test.stdout),
        String::from_utf8_lossy(&ts_test.stderr)
    );
    let ts_test_stdout = String::from_utf8_lossy(&ts_test.stdout);
    assert!(ts_test_stdout.contains("ok 1"), "stdout: {ts_test_stdout}");
    assert!(ts_test_stdout.contains("0"), "stdout: {ts_test_stdout}");
}

#[test]
fn utility_corpus_date_fns_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "date-fns",
        "export function addDays(date, amount) { return 0; }\nexport function format(date) { return 0; }\n",
        "formatISO",
        "export function formatISO(date) { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "date-fns");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
console.log(addDays('2024-01-01', 3));
console.log(format('2024-01-01'));
console.log(formatISO('2024-01-01'));
"#,
    )
    .expect("write date-fns JS source");
    let test_path = dir.path().join("tests").join("date-fns.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "date-fns corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "date-fns corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "date-fns corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n0\n0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_zod_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "zod",
        "export default function zod() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "zod");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import zod from 'zod';
console.log(zod());
"#,
    )
    .expect("write zod source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "zod corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "zod corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "zod corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_plimit_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "p-limit",
        "export default function pLimit() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "p-limit");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import pLimit from 'p-limit';
console.log(pLimit());
"#,
    )
    .expect("write p-limit source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "p-limit corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "p-limit corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "p-limit corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_ms_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "ms",
        "export default function ms() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "ms");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import ms from 'ms';
console.log(ms());
"#,
    )
    .expect("write ms source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "ms corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "ms corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "ms corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_zod_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "zod",
        "export default function zod() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "zod");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import zod from 'zod';
console.log(zod());
"#,
    )
    .expect("write zod JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import zod from 'zod';
Kali.test('zod corpus', () => {
  console.log(zod());
});
"#,
    )
    .expect("write zod test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "zod corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "zod corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "zod corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "zod corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_plimit_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "p-limit",
        "export default function pLimit() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "p-limit");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import pLimit from 'p-limit';
console.log(pLimit());
"#,
    )
    .expect("write p-limit JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import pLimit from 'p-limit';
Kali.test('p-limit corpus', () => {
  console.log(pLimit());
});
"#,
    )
    .expect("write p-limit test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "p-limit corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "p-limit corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "p-limit corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "p-limit corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_ms_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "ms",
        "export default function ms() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "ms");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import ms from 'ms';
console.log(ms());
"#,
    )
    .expect("write ms JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import ms from 'ms';
Kali.test('ms corpus', () => {
  console.log(ms());
});
"#,
    )
    .expect("write ms test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "ms corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "ms corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "ms corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "ms corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn node_runner_corpus_semver_style_package_bin_executes_on_the_node_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("node"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("bin/semver.js").to_str().unwrap(),
        ],
    );
    assert!(
        run.status.success(),
        "semver corpus package bin should execute on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_remains_checkable_and_buildable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_remains_checkable_and_buildable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent JS source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_is_executable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent JS source");

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_is_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
export function describeAgent() {
  return codingAgent();
}
"#,
    )
    .expect("write pi-coding-agent JS source");
    let test_path = dir.path().join("tests").join("pi-coding-agent.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { describeAgent } from '../main.js';
Kali.test('pi-coding-agent corpus', () => {
  console.log(describeAgent());
});
"#,
    )
    .expect("write pi-coding-agent JS test source");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn binary_entrypoint_corpus_pi_coding_agent_style_package_executes_on_the_node_surface_and_is_rejected_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let standalone_run = run_kali(
        dir.path(),
        ["run", package_dir.join("dist/cli.js").to_str().unwrap()],
    );
    assert!(
        !standalone_run.status.success(),
        "pi-coding-agent corpus package bin should stay rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&standalone_run.stdout),
        String::from_utf8_lossy(&standalone_run.stderr)
    );
    let standalone_stderr = String::from_utf8_lossy(&standalone_run.stderr);
    assert!(
        standalone_stderr.contains("E5506"),
        "stderr: {standalone_stderr}"
    );
    assert!(
        standalone_stderr.contains("Node.js CLI features")
            && standalone_stderr.contains("unavailable on the 'deno' API surface"),
        "stderr: {standalone_stderr}"
    );

    let node_run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("dist/cli.js").to_str().unwrap(),
        ],
    );
    assert!(
        node_run.status.success(),
        "pi-coding-agent corpus package bin should execute on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&node_run.stdout),
        String::from_utf8_lossy(&node_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node_run.stdout), "0\n");
}

#[test]
fn binary_entrypoint_corpus_pi_coding_agent_style_package_preserves_node_arguments_on_the_node_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let standalone_run = run_kali(
        dir.path(),
        ["run", package_dir.join("dist/argv.js").to_str().unwrap()],
    );
    assert!(
        !standalone_run.status.success(),
        "pi-coding-agent argv probe should stay rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&standalone_run.stdout),
        String::from_utf8_lossy(&standalone_run.stderr)
    );
    let standalone_stderr = String::from_utf8_lossy(&standalone_run.stderr);
    assert!(
        standalone_stderr.contains("E5506"),
        "stderr: {standalone_stderr}"
    );

    let node_run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("dist/argv.js").to_str().unwrap(),
            "--",
            "alpha",
        ],
    );
    assert!(
        node_run.status.success(),
        "pi-coding-agent argv probe should preserve Node arguments on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&node_run.stdout),
        String::from_utf8_lossy(&node_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node_run.stdout), "1\n");
}

#[test]
fn node_assuming_corpus_packages_are_rejected_on_the_default_standalone_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_node_assuming_package(
        dir.path(),
        "chalk",
        r#"import { createHash } from "node:crypto";
export default function chalk() {
    createHash("sha256").update("chalk").digest("hex");
    return "chalk";
}
"#,
    );
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import chalk from 'chalk';
console.log(chalk());
"#,
    )
    .expect("write node package source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        !check.status.success(),
        "node-assuming package should be rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E6005"), "stderr: {check_stderr}");
    assert!(
        check_stderr.contains("Node-only host API"),
        "stderr: {check_stderr}"
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        !run.status.success(),
        "node-assuming package should stay rejected at runtime on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run_stderr.contains("E6005"), "stderr: {run_stderr}");
    assert!(
        run_stderr.contains("Node-only host API"),
        "stderr: {run_stderr}"
    );

    let test_source = dir.path().join("tests").join("main.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        r#"import chalk from 'chalk';
Kali.test('node-assuming corpus', () => {
  console.log(chalk());
});
"#,
    )
    .expect("write node package test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        !test.status.success(),
        "node-assuming package should stay rejected at test time on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test.stderr);
    assert!(test_stderr.contains("E6005"), "stderr: {test_stderr}");
    assert!(
        test_stderr.contains("Node-only host API"),
        "stderr: {test_stderr}"
    );
}

#[test]
fn utility_corpus_packages_with_pattern_exports_remain_executable_on_the_default_standalone_surface(
) {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("redux", "createStore"),
        ("reselect", "selectors"),
        ("xstate", "react"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                    package = package,
                    subpath = subpath
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility pattern-export package {package} should be checkable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility pattern-export package {package} should be buildable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility pattern-export package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_module_entries_remain_executable_on_the_default_standalone_surface()
{
    for package in [
        "ramda",
        "rambda",
        "rxjs",
        "uuid",
        "commander",
        "immer",
        "typescript",
        "esbuild",
        "luxon",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nconsole.log(root());\n",
                    package = package
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility module-only package {package} should be checkable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility module-only package {package} should be buildable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility module-only package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_module_entry_chains_remain_executable_on_the_default_standalone_surface(
) {
    for package in [
        "ramda",
        "uuid",
        "commander",
        "immer",
        "typescript",
        "esbuild",
        "dayjs",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:internal'; }}\n",
                package = package
            ),
        )
        .expect("write utility internal module");
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nconsole.log(root());\n",
                    package = package
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility module-chain package {package} should resolve its internal module dependency on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility module-chain package {package} should build while resolving its internal module dependency on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility module-chain package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_web_baseline_primitives_remain_executable_on_the_default_standalone_surface(
) {
    for package in [
        "ramda",
        "uuid",
        "rxjs",
        "dayjs",
        "luxon",
        "zod",
        "nanoid",
        "axios",
        "ajv",
        "redux",
        "mitt",
        "swr",
        "nanostores",
        "pinia",
        "superjson",
        "yup",
        "chart.js",
        "recharts",
        "@emotion/styled",
        "@storybook/react",
        "@tanstack/react-table",
        "lodash",
        "vite",
        "yaml",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        write_web_baseline_interop_source(&source_path, package);

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility web-baseline package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "utility web-baseline package {package} should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility web-baseline package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_web_baseline_primitives_remain_checkable_executable_and_testable_on_the_default_standalone_surface_in_js_input(
) {
    for package in ["ramda", "uuid", "dayjs", "zod", "lodash", "yaml"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("tests").join("web-baseline.test.js");
        write_web_baseline_test_source(&test_path, package);

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility web-baseline package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "utility web-baseline package {package} should be buildable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility web-baseline package {package} should stay executable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "utility web-baseline package {package} should be testable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_mixed_format_entries_remain_executable_on_the_default_standalone_surface(
) {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("immer", "produce"),
        ("typescript", "tsc"),
        ("esbuild", "build"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "module.exports = function root() {{ return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility mixed-format package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility mixed-format package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import root from 'interop-demo';\nimport feature from 'interop-demo/feature';\nif (feature() !== 0) { throw new Error('interop-demo feature export mismatch'); }\nconsole.log(root());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility mixed-format package interop-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility mixed-format package interop-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility mixed-format package interop-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import root from 'interop-demo';\nimport feature from 'interop-demo/feature';\nif (feature() !== 0) { throw new Error('interop-demo feature export mismatch'); }\nconsole.log(root());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility mixed-format package interop-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility mixed-format package interop-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility mixed-format package interop-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_exports_map_and_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "interop-export-map-demo",
        "module.exports = function root() { return 0; }\n",
        "feature",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-export-map-demo");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import root from 'interop-export-map-demo';\nimport feature from 'interop-export-map-demo/feature';\nif (root() !== 0 || feature() !== 0) { throw new Error('interop-export-map-demo export mismatch'); }\nconsole.log(root() + feature());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_exports_map_and_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "interop-export-map-demo",
        "module.exports = function root() { return 0; }\n",
        "feature",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-export-map-demo");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import root from 'interop-export-map-demo';\nimport feature from 'interop-export-map-demo/feature';\nif (root() !== 0 || feature() !== 0) { throw new Error('interop-export-map-demo export mismatch'); }\nconsole.log(root() + feature());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be checkable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be buildable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should stay executable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_testable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo-test",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo-test");
    let test_source = dir.path().join("tests").join("interop-demo.test.ts");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import root from 'interop-demo-test';\nimport feature from 'interop-demo-test/feature';\nKali.test('interop-demo-test corpus', () => {\n  if (root() !== 0 || feature() !== 0) { throw new Error('interop-demo-test export mismatch'); }\n  console.log(root() + feature());\n});\n",
    )
    .expect("write utility test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "utility mixed-format package interop-demo-test should be testable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_testable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo-test",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo-test");
    let test_source = dir.path().join("tests").join("interop-demo.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import root from 'interop-demo-test';\nimport feature from 'interop-demo-test/feature';\nKali.test('interop-demo-test corpus', () => {\n  if (root() !== 0 || feature() !== 0) { throw new Error('interop-demo-test export mismatch'); }\n  console.log(root() + feature());\n});\n",
    )
    .expect("write utility test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "utility mixed-format package interop-demo-test should be testable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn utility_corpus_scoped_packages_remain_executable_on_the_default_standalone_surface() {
    for package in [
        "@babel/runtime",
        "@npmcli/package-json",
        "@jridgewell/sourcemap-codec",
        "@reduxjs/toolkit",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "scoped utility package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "scoped utility package {package} should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "scoped utility package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_scoped_packages_remain_checkable_and_executable_on_the_default_standalone_surface_on_js_input(
) {
    for package in [
        "@babel/runtime",
        "@npmcli/package-json",
        "@jridgewell/sourcemap-codec",
        "@reduxjs/toolkit",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write utility JS source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "scoped utility package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "scoped utility package {package} should be buildable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "scoped utility package {package} should stay executable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");
    }
}

#[test]
fn deno_host_corpus_packages_remain_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    for (package, body) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write deno host source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "deno host package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "deno",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "deno host package {package} should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "deno host package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn jsr_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import joinPath from '@std/path';\nconsole.log(joinPath('alpha', 'beta'));\n",
    )
    .expect("write jsr source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "jsr package should be checkable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build_out_dir = dir.path().join("build");
    let build = run_kali(
        dir.path(),
        [
            "build",
            "--api",
            "deno",
            "--out-dir",
            build_out_dir.to_str().unwrap(),
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "jsr package should be buildable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "jsr package should stay executable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn deno_host_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface_in_js_input(
) {
    for (package, body) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write deno host JS source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "deno host package {package} should be checkable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "deno",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "deno host package {package} should be buildable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "deno host package {package} should stay executable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn jsr_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import joinPath from '@std/path';\nconsole.log(joinPath('alpha', 'beta'));\n",
    )
    .expect("write jsr JS source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "jsr package should be checkable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build_out_dir = dir.path().join("build");
    let build = run_kali(
        dir.path(),
        [
            "build",
            "--api",
            "deno",
            "--out-dir",
            build_out_dir.to_str().unwrap(),
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "jsr package should be buildable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "jsr package should stay executable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn node_runner_corpus_packages_remain_gated_on_the_node_surface() {
    for package in ["vitest", "jest", "mocha", "ava"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_stub_package(
            dir.path(),
            package,
            r#"import assert from "node:assert";
export default assert;
"#,
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import '{}';\nKali.test('{} corpus', () => {{\n  console.log('node corpus: {}');\n}});\n",
                package, package, package
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(
            stdout.contains(&format!("node corpus: {}", package)),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn node_runner_corpus_packages_with_exports_maps_remain_gated_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} with exports map should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_gated_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn node_assuming_corpus_packages_remain_gated_on_the_node_surface() {
    for (package, body) in [
        (
            "axios",
            r#"import path from "node:path";
export default function axios() {
    return path.basename("/tmp/axios.js");
}
"#,
        ),
        (
            "express",
            r#"import assert from "node:assert";
export default function express() {
    assert.ok(true);
    return "express";
}
"#,
        ),
        (
            "chalk",
            r#"import { createHash } from "node:crypto";
export default function chalk() {
    createHash("sha256").update("chalk").digest("hex");
    return "chalk";
}
"#,
        ),
        (
            "dotenv",
            r#"import fs from "node:fs";
export default function dotenv() {
    return fs.existsSync("/tmp/dotenv");
}
"#,
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_node_assuming_package(dir.path(), package, body);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import {package} from '{package}';\nconsole.log({package}());\n",
                package = package
            ),
        )
        .expect("write node package source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "node package {package} should check on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "node",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "node package {package} should build on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node package {package} should execute on the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test_source = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import {package} from '{package}';\nKali.test('{package} corpus', () => {{\n  console.log({package}());\n}});\n",
                package = package
            ),
        )
        .expect("write node package test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert!(
            String::from_utf8_lossy(&test.stdout).contains("ok 1"),
            "stdout: {}",
            String::from_utf8_lossy(&test.stdout)
        );
    }
}

#[test]
fn node_builtin_corpus_packages_remain_checkable_buildable_executable_and_testable_on_the_node_surface_on_js_input(
) {
    for (package, body, expected) in [
        (
            "node-buffer-corpus",
            "import { Buffer } from \"node:buffer\";\nexport default function root() { Buffer.from(\"node\"); return 0; }\n",
            "0",
        ),
        (
            "node-path-corpus",
            "import path from \"node:path\";\nexport default function root() { path.basename(\"/tmp/node-corpus.txt\"); return 0; }\n",
            "0",
        ),
        (
            "node-os-corpus",
            "import os from \"node:os\";\nexport default function root() { return typeof os.platform === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-crypto-corpus",
            "import { createHash } from \"node:crypto\";\nexport default function root() { createHash(\"sha256\").update(\"node-corpus\").digest(\"hex\"); return 0; }\n",
            "0",
        ),
        (
            "node-fs-corpus",
            "import fs from \"node:fs\";\nexport default function root() { return typeof fs.existsSync === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-fs-promises-corpus",
            "import { readFile } from \"node:fs/promises\";\nexport default function root() { return typeof readFile === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-url-corpus",
            "import { fileURLToPath } from \"node:url\";\nexport default function root() { fileURLToPath(new URL(\"file:///tmp/node-corpus.txt\")); return 0; }\n",
            "0",
        ),
        (
            "node-util-corpus",
            "import util from \"node:util\";\nexport default function root() { util.format(\"%s-%s\", \"node\", \"util\"); return 0; }\n",
            "0",
        ),
        (
            "node-http-corpus",
            "import http from \"node:http\";\nexport default function root() { return typeof http.get === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-timers-corpus",
            "import timers from \"node:timers\";\nexport default function root() { return typeof timers.setTimeout === \"function\" && typeof timers.clearTimeout === \"function\" && typeof timers.setInterval === \"function\" && typeof timers.clearInterval === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-events-corpus",
            "import { EventEmitter } from \"node:events\";\nexport default function root() { const emitter = new EventEmitter(); return typeof emitter.on === \"function\" && typeof emitter.emit === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-stream-corpus",
            "import { Readable } from \"node:stream\";\nexport default function root() { return typeof Readable === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-assert-corpus",
            "import assert from \"node:assert\";\nexport default function root() { assert.ok(true); return 0; }\n",
            "0",
        ),
        (
            "node-child-process-corpus",
            "import { spawn } from \"node:child_process\";\nexport default function root() { return typeof spawn === \"function\" ? 0 : 1; }\n",
            "0",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_module_only_package(dir.path(), package, body);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write node built-in source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "node built-in package {package} should check on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "node",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "node built-in package {package} should build on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node built-in package {package} should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), format!("{expected}\n"));

        let test_source = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nKali.test('{package} corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write node built-in test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node built-in package {package} should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains(expected), "stdout: {test_stdout}");
    }
}

#[test]
fn node_runner_corpus_packages_with_inherited_api_surface_remain_executable_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let run_source = dir.path().join("main.ts");
        fs::write(
            &run_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write inherited node run source");

        let run = run_kali(dir.path(), ["run", run_source.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "node package {package} with exports map should execute on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(run_stdout.trim(), "0", "stdout: {run_stdout}");

        let test_source = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write inherited node test source");

        let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "node package {package} with exports map should be testable on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn node_runner_corpus_packages_with_exports_maps_remain_executable_on_the_node_surface_in_js_input()
{
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nimport {{ Buffer }} from \"node:buffer\";\nexport default function root() {{ assert.ok(true); Buffer.from('node'); return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from \"node:assert\";\nimport {{ Buffer }} from \"node:buffer\";\nexport default function subpath() {{ assert.ok(true); Buffer.from('node'); return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let run_source = dir.path().join("main.js");
        fs::write(
            &run_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node run source");

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", run_source.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node package {package} with exports map should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(run_stdout.trim(), "0", "stdout: {run_stdout}");

        let test_source = dir
            .path()
            .join("tests")
            .join(format!("{}.test.js", package));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} with exports map should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_executable_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_executable_on_the_node_surface_in_js_input(
) {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.js", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn default_standalone_corpus_rejects_semver_style_package_bin_entrypoint() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let output = run_kali(
        dir.path(),
        ["run", package_dir.join("bin/semver.js").to_str().unwrap()],
    );

    assert!(
        !output.status.success(),
        "expected the default standalone surface to reject a Node-style package bin entrypoint\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("Node.js CLI features")
            && stderr.contains("unavailable on the 'deno' API surface"),
        "stderr: {stderr}"
    );
}

#[test]
fn node_corpus_executes_semver_style_package_bin_entrypoint() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let output = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("bin/semver.js").to_str().unwrap(),
            "--",
            "1.2.3",
        ],
    );

    assert!(
        output.status.success(),
        "expected the Node surface to execute a published package bin entrypoint\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
