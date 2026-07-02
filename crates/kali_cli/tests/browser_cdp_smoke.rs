//! Gated end-to-end smoke: build a real browser bundle with `kali build
//! --bundle --api browser`, serve it (plus a harness page) over localhost HTTP,
//! and run it in one shared real Chromium via the test-only CDP driver.
//!
//! Why an HTTP server? The emitted bundle glue (`app/app.js`) resolves its
//! wasm with `new URL("./app.wasm", import.meta.url)` and `fetch()`. Chromium
//! blocks `fetch()` of `file://` URLs, so we serve the bundle dir from a tiny
//! localhost server and navigate to a real HTML harness page that imports the
//! glue. The harness page itself is no longer hand-written here — it comes
//! from the production `kali_runtime::browser_bundle_harness_page` generator,
//! so this test proves that page is genuinely browser-loadable. No extra
//! Chromium flags, no production-code changes, no driver changes.
mod cdp_driver;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use tempfile::tempdir;

use cdp_driver::{CdpBrowser, CdpPageOutcome};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn chromium() -> Option<String> {
    ["chromium", "chromium-browser", "google-chrome", "chrome"]
        .into_iter()
        .find(|&exe| cdp_driver::chromium_available(exe))
        .map(str::to_owned)
}

/// Map a file extension to a minimal, correct content-type for module loading.
fn content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("js") => "text/javascript",
        Some("wasm") => "application/wasm",
        Some("html") => "text/html",
        Some("map") | Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Serve `root` over a fresh `127.0.0.1` port on a detached thread and return the
/// bound port. Handles just enough of HTTP/1.1 GET to load a module graph:
/// request line -> file under `root` (query stripped, traversal rejected),
/// answered with a correct content-type and `Connection: close`.
fn serve_dir(root: PathBuf) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind localhost server");
    let port = listener.local_addr().expect("server addr").port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }
            // Drain the remaining request headers so the client isn't reset early.
            let mut header = String::new();
            while reader
                .read_line(&mut header)
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                if header == "\r\n" || header == "\n" {
                    break;
                }
                header.clear();
            }

            let target = request_line.split_whitespace().nth(1).unwrap_or("/");
            let path = target.split(['?', '#']).next().unwrap_or("/");
            let relative = path.trim_start_matches('/');
            let file = root.join(relative);

            let response = if relative.contains("..") || !file.starts_with(&root) {
                b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
            } else if let Ok(bytes) = fs::read(&file) {
                let mut head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    content_type(&file),
                    bytes.len()
                )
                .into_bytes();
                head.extend_from_slice(&bytes);
                head
            } else {
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
            };
            let _ = stream.write_all(&response);
            let _ = stream.flush();
        }
    });
    port
}

#[test]
#[ignore = "requires a real Chromium; run with `-- --ignored`"]
fn real_chromium_runs_a_browser_bundle_and_captures_console() {
    let Some(chromium_exe) = chromium() else {
        eprintln!("skipping: no Chromium available");
        return;
    };

    // 1. Build a browser bundle from a program with BOTH entry shapes: a bare
    //    top-level statement (runs via the glue's `start()` helper and must
    //    route console.log through the `console_log` import) and an exported
    //    function (runs via the per-export wrapper). The tree-shake marker is
    //    required for the export wrapper to be emitted.
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("app.ts");
    fs::write(
        &source,
        "// kali-tree-shake: smoke\n\
console.log(1 + 2);\n\
export async function smoke(left, right) {\n\
  console.log(6 + 1);\n\
  return left - left + right - right;\n\
}\n",
    )
    .expect("write source");
    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source)
        .output()
        .expect("run kali build");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    // 2. Locate the emitted bundle dir (kali writes `<stem>/` next to the source).
    let bundle_dir = dir.path().join("app");
    assert!(bundle_dir.join("app.js").exists(), "bundle glue missing");
    assert!(bundle_dir.join("app.wasm").exists(), "bundle wasm missing");

    // 3. Generate the browser-native harness page from the production API:
    //    run the top-level program via start(), then the exported wrapper.
    let harness_path = dir.path().join("cdp-harness.html");
    let harness = kali_runtime::browser_bundle_harness_page(
        "app",
        "const mod = await import(bundleJs.href);\n\
await mod.start();\n\
await mod.smoke(1n, 2n);\n",
    );
    fs::write(&harness_path, harness).expect("write harness");

    // 4. Serve the bundle dir + harness page over localhost so the browser can
    //    `fetch()` the module graph and wasm (blocked under file://).
    let port = serve_dir(dir.path().to_path_buf());

    // 5. Drive it through a single shared Chromium via CDP.
    let mut browser =
        CdpBrowser::launch(&chromium_exe, Duration::from_secs(20)).expect("launch chromium");
    let url = format!("http://127.0.0.1:{port}/cdp-harness.html");
    let outcome: CdpPageOutcome = browser
        .run_page(&url, Duration::from_secs(30))
        .expect("run page");
    browser.close().expect("close");

    // 6. Assert the real browser produced BOTH programs' console output, in
    //    order: top-level `console.log(1 + 2)` via start(), then the export's
    //    `console.log(6 + 1)` via the wrapper.
    assert!(outcome.completed, "harness did not signal completion");
    assert_eq!(outcome.stdout(), "3\n7\n", "console: {:?}", outcome.console);
}
