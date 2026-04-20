use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::tempdir;

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

fn write_web_baseline_interop_source(path: &Path, package: &str) {
    fs::write(
        path,
        format!(
            "import describe from '{package}';\nconst controller = new AbortController();\nconst target = new EventTarget();\nlet count = 0;\ntarget.addEventListener('tick', () => {{\n  count += 1;\n  controller.abort();\n}});\nconst custom = new CustomEvent('tick');\ntarget.dispatchEvent(custom);\nconsole.log(custom.type);\ntarget.dispatchEvent(new Event('tick'));\nconst query = new URLSearchParams('alpha=1&beta=two+words');\nquery.append('gamma', String(count));\nquery.set('beta', describe(count));\nquery.get('alpha');\nquery.getAll('beta');\nquery.has('gamma');\nquery.toString();\nconst agent = navigator.userAgent;\nconst language = navigator.language;\nconst online = navigator.onLine;\nconsole.log(agent, language, online);\nconst now = performance.now();\nconsole.log(now);\nqueueMicrotask(() => {{\n  count += 0;\n}});\nconst encoder = new TextEncoder();\nconst encoded = encoder.encode(describe(count));\nconst decoder = new TextDecoder();\ndecoder.decode(encoded);\nconst encodedBinary = btoa(describe(count));\natob(encodedBinary);\nconst browserUrl = new URL('https://example.com/browser?alpha=1#fragment');\nbrowserUrl.pathname;\nbrowserUrl.search;\nbrowserUrl.hash;\nbrowserUrl.href;\nconst headers = new Headers();\nheaders.append('x-corpus', describe(count));\nheaders.set('accept', 'application/json');\nconst request = new Request('https://example.com/request');\nconst response = new Response('browser corpus');\nfetch('https://example.com/fetch');\nconsole.log(headers, request, response);\nconst blob = structuredClone(new Blob(['browser corpus']));\nconst file = structuredClone(new File(['browser corpus'], 'browser.txt'));\nconst form = new FormData();\nform.append('blob', blob);\nform.append('file', file);\nform.set('count', String(count));\nlocalStorage.clear();\nlocalStorage.setItem('count', String(count));\nlocalStorage.getItem('count');\nsessionStorage.clear();\nsessionStorage.setItem('count', String(count));\nsessionStorage.getItem('count');\nconst reader = new FileReader();\nreader.readAsText(blob);\nreader.readAsText(file);\nconst socket = new WebSocket('https://example.com/socket');\nsocket.sendText(describe(count));\nsocket.close();\nconst worker = new Worker('https://example.com/worker.js');\nworker.postMessage(describe(count));\nworker.terminate();\nconst channel = new BroadcastChannel('browser-corpus');\nchannel.postMessage(describe(count));\nchannel.close();\nif (typeof indexedDB !== 'undefined') {{\n  const database = indexedDB.open('browser-corpus');\n  database.put('events', 'count', String(count));\n  database.get('events', 'count');\n  database.storeNames();\n}}\nstructuredClone(blob);\nstructuredClone(file);\nconst randomUuid = crypto.randomUUID();\nconsole.log(randomUuid);\nconsole.log(describe(count));\n",
            package = package
        ),
    )
    .expect("write web baseline source");
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
fn browser_corpus_packages_with_exports_maps_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("svelte", "compiler"),
        ("lit", "decorators"),
        ("solid-js", "store"),
        ("vue-router", "history"),
        ("react-router", "dom"),
        ("jotai", "store"),
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
fn browser_corpus_packages_with_pattern_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
        ("vue-router", "matcher"),
        ("react-router", "routes"),
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
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
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
fn browser_corpus_packages_with_browser_replacement_maps_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
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
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
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
        "@storybook/react",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
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
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
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
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@remix-run/react", "links"),
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
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("zustand", "vanilla"),
        ("@remix-run/react", "links"),
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
        "@jridgewell/sourcemap-codec",
        "@emotion/react",
        "@emotion/styled",
        "@mantine/core",
        "vite",
        "@tanstack/router",
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
fn utility_corpus_packages_with_pattern_exports_remain_executable_on_the_default_standalone_surface(
) {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("redux", "createStore"),
        ("reselect", "selectors"),
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
            "utility pattern-export package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility pattern-export package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
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
            "utility module-only package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility module-only package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
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
            "utility module-chain package {package} should resolve its internal module dependency\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility module-chain package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
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
fn utility_corpus_scoped_packages_remain_executable_on_the_default_standalone_surface() {
    for package in [
        "@babel/runtime",
        "@npmcli/package-json",
        "@jridgewell/sourcemap-codec",
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
fn node_runner_corpus_packages_require_the_node_context_but_remain_executable_there() {
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
            "node package {package} should execute under the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
    }
}

#[test]
fn node_runner_corpus_packages_with_exports_maps_require_the_node_context_but_remain_executable_there(
) {
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
            "node package {package} with exports map should execute under the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
    }
}

#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_require_the_node_context_but_remain_executable_there(
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
            "node mixed-format package {package} should execute under the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
    }
}

#[test]
fn node_assuming_corpus_packages_require_the_node_context_but_remain_executable_there() {
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
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_node_assuming_package(dir.path(), package, body);
        let source_path = dir.path().join("main.ts");
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
            "node package {package} should check under the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node package {package} should execute under the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}
