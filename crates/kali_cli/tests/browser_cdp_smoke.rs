//! Gated end-to-end smoke: build a real browser bundle with `kali build
//! --bundle --api browser`, serve it (plus a harness page) over localhost HTTP,
//! and run it in one shared real Chromium via the test-only CDP driver.
//!
//! Why an HTTP server instead of the plan's `browser_bundle_harness_script`?
//! That production helper emits the *node* harness (its prelude imports
//! `node:fs/promises` / `node:url`), which neither renders nor resolves in a
//! browser. The emitted bundle glue (`app/app.js`) instead resolves its wasm
//! with `new URL("./app.wasm", import.meta.url)` and `fetch()`. Chromium blocks
//! `fetch()` of `file://` URLs, so we serve the bundle dir from a tiny localhost
//! server and navigate to a real HTML harness page that imports the glue. No
//! extra Chromium flags, no production-code changes, no driver changes.
mod cdp_driver;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use tempfile::tempdir;

use cdp_driver::{CdpBrowser, CdpConsoleLine, CdpPageOutcome};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn chromium() -> Option<String> {
    for exe in ["chromium", "chromium-browser", "google-chrome", "chrome"] {
        if Command::new(exe).arg("--version").output().is_ok() {
            return Some(exe.to_owned());
        }
    }
    None
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
            // Best-effort: let the client read before the socket drops.
            let _ = stream.read(&mut [0u8; 0]);
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

    // 1. Build a browser bundle from a program that logs a known value (1 + 2).
    //    We export a named function (matching how the repo's own browser-bundle
    //    tests drive bundles): its body's `console.log(1 + 2)` routes through the
    //    glue's `console_log` import when the per-export wrapper is called. A bare
    //    top-level `main()` program does NOT surface console through the import in
    //    a browser, so the exported-function form is the reliable entry point.
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("app.ts");
    fs::write(
        &source,
        "// kali-tree-shake: smoke\n\
export async function smoke(left, right) {\n\
  console.log(1 + 2);\n\
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

    // 3. Write a real HTML harness page next to the bundle dir. It imports the
    //    emitted glue's per-export `smoke` wrapper (which instantiates the wasm and
    //    calls `exports.smoke`, whose `console.log(1 + 2)` routes through the glue's
    //    `console_log` import), then signals completion with a single string arg
    //    (Chromium `addBinding` functions require exactly one string argument).
    let harness_path = dir.path().join("cdp-harness.html");
    let harness = "<!doctype html>\n\
<meta charset=\"utf-8\">\n\
<script type=\"module\">\n\
try {\n\
  const mod = await import('./app/app.js');\n\
  await mod.smoke(1n, 2n);\n\
} catch (err) {\n\
  console.error('harness error: ' + (err && err.stack || err));\n\
}\n\
if (globalThis.__kaliHarnessDone) { globalThis.__kaliHarnessDone(''); }\n\
</script>\n";
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

    // 6. Assert the real browser produced the program's console output.
    assert!(outcome.completed, "harness did not signal completion");
    let log_lines: Vec<&CdpConsoleLine> = outcome
        .console
        .iter()
        .filter(|line| line.kind == "log")
        .collect();
    assert!(
        log_lines.iter().any(|line| line.text.contains('3')),
        "expected a '3' log line, got: {:?}",
        outcome.console
    );
    assert!(
        outcome.stdout().contains("3\n"),
        "unexpected stdout: {:?}",
        outcome.stdout()
    );
}
