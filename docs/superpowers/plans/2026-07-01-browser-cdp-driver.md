# Browser CDP Driver (real-browser smoke execution) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an opt-in, gated smoke test that runs a real Kali browser bundle inside a single shared headless Chromium via the Chrome DevTools Protocol (CDP), capturing the page's `console.log` output and asserting on it — the fidelity coverage node cannot provide.

**Architecture:** The CDP driver is **test-only infrastructure**, not production code — no production path drives a real browser (the CLI `run`/`test` commands go through `execute.rs`'s `Command::output()` harness). So the driver lives in `kali_cli`'s integration-test tree (`tests/cdp_driver/`), and `tungstenite` is a **dev-dependency** of `kali_cli` — nothing leaks into any production build or into other crates' test builds. It is a small synchronous CDP client: launch one Chromium with `--remote-debugging-port=0`, connect over a localhost `ws://` WebSocket, and per case open a fresh target/tab, navigate to the harness HTML, capture `Runtime.consoleAPICalled` events, and detect completion via a `Runtime.addBinding` callback the harness invokes. A single browser instance is shared across cases; every CDP read is timeout-bounded so it can never hang.

**Tech Stack:** Rust (sync, integration-test code), `tungstenite` (WebSocket, ws:// only, no TLS), `serde_json` (CDP messages), Chromium headless, CDP domains `Target`/`Page`/`Runtime`/`Browser`.

## Global Constraints

- **Test-only, no production footprint.** All new code is Rust integration-test code under `crates/kali_cli/tests/`. Do NOT add anything to `kali_runtime`, `kali_test_support`, or any production crate. `kali_test_support` is reserved for genuinely cross-crate helpers; this driver has a single consumer (`kali_cli`'s smoke test), so it stays local per that crate's convention.
- **WebSocket dependency (dev only):** add `tungstenite = { version = "0.24", default-features = false }` to `[workspace.dependencies]` in `/workspace/Cargo.toml`, and reference it as `tungstenite = { workspace = true }` under `[dev-dependencies]` in `crates/kali_cli/Cargo.toml`. Default features are disabled so no TLS stack is pulled in; plain `ws://` needs none.
- **Synchronous only.** No tokio / async runtime; use blocking sockets with read timeouts.
- **Chromium launch flags (exact):** `--headless --no-sandbox --disable-gpu --remote-debugging-port=0 --user-data-dir=<fresh temp dir> about:blank`. `--no-sandbox` is mandatory in this container (`kernel.unprivileged_userns_clone=0`, no setuid `chrome-sandbox`).
- **Every CDP read is timeout-bounded** via `TcpStream::set_read_timeout`; per-operation timeout 20s, overall page-run timeout 30s. A timeout is an error/early-break, never an infinite wait.
- **Gating:** the end-to-end smoke test is annotated `#[ignore]` so `cargo test` never runs it by default; run it explicitly with `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`. When run, it first probes Chromium availability and returns early (pass) if absent. The driver's own pure unit tests (message parsing) are NOT ignored and run by default (no browser needed).
- **Single shared browser instance:** launch one `CdpBrowser`, run every case as a new target on it, close it once at the end. Do NOT launch a browser per case.
- **Do NOT change the default harness resolution** (`browser_harness_default_command_parts`) — it stays node-preferred (spec Part A). This path is additive and fully isolated in the test tree.
- Match existing test style in `crates/kali_cli/tests`: `snake_case`, `tempfile::tempdir()` for fixtures, `CARGO_BIN_EXE_kali` for the binary, manual `Display` on error enums.

---

## File Structure

Cargo compiles top-level `tests/*.rs` as separate test binaries, but files in **subdirectories** of `tests/` are only compiled when a test binary `mod`-includes them. So the driver lives in a subdirectory and is pulled in by the one smoke-test binary.

- Create `crates/kali_cli/tests/browser_cdp_smoke.rs` — the test binary: `mod cdp_driver;` plus the `#[ignore]`d end-to-end smoke test.
- Create `crates/kali_cli/tests/cdp_driver/mod.rs` — module root; `mod protocol; mod driver;` and `pub use` of the driver types.
- Create `crates/kali_cli/tests/cdp_driver/protocol.rs` — CDP message framing: request-id allocation, `send`, `read`, `parse_incoming`, timeout wiring. One responsibility: bytes ↔ CDP JSON over the socket.
- Create `crates/kali_cli/tests/cdp_driver/driver.rs` — high-level `CdpBrowser` (launch, `run_page`, `close`) built on `protocol.rs`. One responsibility: browser lifecycle + per-page session orchestration.
- Modify `/workspace/Cargo.toml` — add `tungstenite` to `[workspace.dependencies]`.
- Modify `crates/kali_cli/Cargo.toml` — add `tungstenite` to `[dev-dependencies]`.

Because this is integration-test code (not a `#[cfg(test)]` module inside a lib), the module's functions are plain `pub`/`pub(crate)` items and its tests are plain `#[test]` fns; everything is compiled in test mode only.

---

### Task 1: Add the `tungstenite` dev-dependency

**Files:**
- Modify: `/workspace/Cargo.toml` (`[workspace.dependencies]`)
- Modify: `crates/kali_cli/Cargo.toml` (`[dev-dependencies]`)

**Interfaces:**
- Produces: `tungstenite` available to `kali_cli`'s tests as `tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream}`.

- [ ] **Step 1: Add to workspace dependencies**

In `/workspace/Cargo.toml`, under `[workspace.dependencies]` (in the `# External dependencies` group), add:

```toml
tungstenite = { version = "0.24", default-features = false }
```

- [ ] **Step 2: Reference it as a kali_cli dev-dependency**

In `crates/kali_cli/Cargo.toml`, under `[dev-dependencies]` (create the section if absent), add:

```toml
tungstenite = { workspace = true }
```

- [ ] **Step 3: Verify it resolves**

Run: `cargo build -p kali_cli --tests`
Expected: builds (no code uses it yet; this locks the dependency for the test target only).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock crates/kali_cli/Cargo.toml
git commit -m "build(cli): add tungstenite dev-dependency for the CDP smoke driver"
```

---

### Task 2: CDP protocol framing (with the test-binary shell)

**Files:**
- Create: `crates/kali_cli/tests/browser_cdp_smoke.rs`
- Create: `crates/kali_cli/tests/cdp_driver/mod.rs`
- Create: `crates/kali_cli/tests/cdp_driver/protocol.rs`

**Interfaces:**
- Produces:
  - `pub(crate) enum CdpError { Timeout, Protocol(String), Transport(String), Launch(String) }` with `Display`.
  - `pub(crate) enum CdpIncoming { Result { id: u64, result: Value }, Error { id: u64, message: String }, Event { method: String, params: Value, session_id: Option<String> } }`
  - `pub(crate) fn parse_incoming(text: &str) -> Result<CdpIncoming, CdpError>` (pure; the unit-test target)
  - `pub(crate) struct CdpConnection` with `from_socket`, `send(method, params, session_id) -> u64`, `read() -> CdpIncoming`, `set_read_timeout(Duration)`.

- [ ] **Step 1: Create the test-binary shell so the subdir module compiles**

Create `crates/kali_cli/tests/browser_cdp_smoke.rs`:

```rust
mod cdp_driver;
```

Create `crates/kali_cli/tests/cdp_driver/mod.rs`:

```rust
//! Test-only Chrome DevTools Protocol driver for real-browser smoke coverage.
mod driver;
mod protocol;

pub use driver::{CdpBrowser, CdpConsoleLine, CdpPageOutcome};
```

Create an empty `crates/kali_cli/tests/cdp_driver/driver.rs` so `mod driver;` resolves:

Run: `touch crates/kali_cli/tests/cdp_driver/driver.rs`

- [ ] **Step 2: Write the failing test for message parsing**

Create `crates/kali_cli/tests/cdp_driver/protocol.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_result_error_and_event_messages() {
        match parse_incoming(r#"{"id":7,"result":{"ok":true}}"#).unwrap() {
            CdpIncoming::Result { id, result } => {
                assert_eq!(id, 7);
                assert_eq!(result["ok"], true);
            }
            other => panic!("expected result, got {other:?}"),
        }

        match parse_incoming(r#"{"id":8,"error":{"code":-32000,"message":"boom"}}"#).unwrap() {
            CdpIncoming::Error { id, message } => {
                assert_eq!(id, 8);
                assert_eq!(message, "boom");
            }
            other => panic!("expected error, got {other:?}"),
        }

        match parse_incoming(
            r#"{"method":"Runtime.consoleAPICalled","params":{"type":"log"},"sessionId":"S1"}"#,
        )
        .unwrap()
        {
            CdpIncoming::Event { method, session_id, params } => {
                assert_eq!(method, "Runtime.consoleAPICalled");
                assert_eq!(session_id.as_deref(), Some("S1"));
                assert_eq!(params["type"], "log");
            }
            other => panic!("expected event, got {other:?}"),
        }
    }
}
```

- [ ] **Step 3: Run it to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke parses_result 2>&1 | tail -5`
Expected: FAIL to compile — `parse_incoming` / `CdpIncoming` not defined.

- [ ] **Step 4: Implement the framing above the test module**

Prepend to `protocol.rs`:

```rust
//! Minimal Chrome DevTools Protocol message framing over a blocking WebSocket.
use std::net::TcpStream;

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

/// Errors surfaced by the CDP transport and driver.
#[derive(Debug)]
pub(crate) enum CdpError {
    /// A bounded read elapsed without the expected message.
    Timeout,
    /// The peer sent a malformed or unexpected message.
    Protocol(String),
    /// The underlying socket failed.
    Transport(String),
    /// The browser process could not be launched or its endpoint not found.
    Launch(String),
}

impl std::fmt::Display for CdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "CDP operation timed out"),
            Self::Protocol(m) => write!(f, "CDP protocol error: {m}"),
            Self::Transport(m) => write!(f, "CDP transport error: {m}"),
            Self::Launch(m) => write!(f, "CDP browser launch error: {m}"),
        }
    }
}

impl std::error::Error for CdpError {}

/// A decoded inbound CDP message.
#[derive(Debug)]
pub(crate) enum CdpIncoming {
    Result { id: u64, result: Value },
    Error { id: u64, message: String },
    Event { method: String, params: Value, session_id: Option<String> },
}

/// Decode one CDP frame. Pure over the JSON text so it is unit-testable.
pub(crate) fn parse_incoming(text: &str) -> Result<CdpIncoming, CdpError> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| CdpError::Protocol(e.to_string()))?;
    let session_id = value
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(method) = value.get("method").and_then(Value::as_str) {
        return Ok(CdpIncoming::Event {
            method: method.to_owned(),
            params: value.get("params").cloned().unwrap_or(Value::Null),
            session_id,
        });
    }
    let id = value
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| CdpError::Protocol(format!("message missing id: {text}")))?;
    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown CDP error")
            .to_owned();
        return Ok(CdpIncoming::Error { id, message });
    }
    Ok(CdpIncoming::Result {
        id,
        result: value.get("result").cloned().unwrap_or(Value::Null),
    })
}

/// A blocking CDP connection with monotonic request ids.
pub(crate) struct CdpConnection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpConnection {
    pub(crate) fn from_socket(socket: WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self { socket, next_id: 1 }
    }

    /// Send a CDP method call, optionally scoped to a flat session. Returns its id.
    pub(crate) fn send(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
    ) -> Result<u64, CdpError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut message = json!({ "id": id, "method": method, "params": params });
        if let Some(session_id) = session_id {
            message["sessionId"] = json!(session_id);
        }
        self.socket
            .send(Message::Text(message.to_string()))
            .map_err(|e| CdpError::Transport(e.to_string()))?;
        Ok(id)
    }

    /// Read the next non-ping frame, mapping a socket read timeout to `CdpError::Timeout`.
    pub(crate) fn read(&mut self) -> Result<CdpIncoming, CdpError> {
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => return parse_incoming(&text),
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(Message::Close(_)) => {
                    return Err(CdpError::Transport("socket closed".to_owned()))
                }
                Ok(_) => continue,
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return Err(CdpError::Timeout)
                }
                Err(e) => return Err(CdpError::Transport(e.to_string())),
            }
        }
    }

    /// Set the read timeout on the underlying TCP stream.
    pub(crate) fn set_read_timeout(&mut self, timeout: std::time::Duration) -> Result<(), CdpError> {
        let stream = match self.socket.get_ref() {
            MaybeTlsStream::Plain(stream) => stream,
            _ => return Err(CdpError::Transport("unexpected TLS stream for ws://".to_owned())),
        };
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| CdpError::Transport(e.to_string()))
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test -p kali_cli --test browser_cdp_smoke parses_result 2>&1 | tail -5`
Expected: PASS (1 test).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/browser_cdp_smoke.rs crates/kali_cli/tests/cdp_driver
git commit -m "test(cli): CDP protocol framing (send/read/parse) over a blocking WebSocket"
```

---

### Task 3: Launch Chromium and discover its WebSocket endpoint

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs`

**Interfaces:**
- Consumes: `CdpError` from `protocol.rs` (via `use super::protocol::...`).
- Produces:
  - `pub(crate) fn chromium_available(executable: &str) -> bool`
  - `pub(crate) fn spawn_chromium(executable: &str, timeout: Duration) -> Result<(Child, String, TempDir), CdpError>` — returns the process, the discovered `ws://` browser endpoint, and the owning temp user-data dir (keep it alive for the process lifetime).

- [ ] **Step 1: Write the failing test (gated on Chromium)**

Put in `crates/kali_cli/tests/cdp_driver/driver.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_chromium_and_reports_ws_endpoint() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let (mut child, ws_url, _dir) =
            spawn_chromium("chromium", std::time::Duration::from_secs(20))
                .expect("spawn chromium");
        assert!(ws_url.starts_with("ws://"), "unexpected ws url: {ws_url}");
        let _ = child.kill();
        let _ = child.wait();
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke spawns_chromium 2>&1 | tail -5`
Expected: FAIL to compile — `chromium_available` / `spawn_chromium` not defined.

- [ ] **Step 3: Implement the launcher above the test module**

Prepend to `driver.rs`:

```rust
//! High-level CDP browser lifecycle: launch, per-page run, close.
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tempfile::TempDir;

use super::protocol::{CdpConnection, CdpError, CdpIncoming};

/// Whether the given browser executable can be invoked (`--version` succeeds).
pub(crate) fn chromium_available(executable: &str) -> bool {
    Command::new(executable).arg("--version").output().is_ok()
}

/// Launch headless Chromium with remote debugging and read its `ws://` browser
/// endpoint from stderr ("DevTools listening on ws://..."), bounded by `timeout`.
pub(crate) fn spawn_chromium(
    executable: &str,
    timeout: Duration,
) -> Result<(Child, String, TempDir), CdpError> {
    let user_data_dir =
        TempDir::new().map_err(|e| CdpError::Launch(format!("temp user-data dir: {e}")))?;
    let mut child = Command::new(executable)
        .arg("--headless")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .arg("--remote-debugging-port=0")
        .arg(format!("--user-data-dir={}", user_data_dir.path().display()))
        .arg("about:blank")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CdpError::Launch(format!("spawn {executable}: {e}")))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CdpError::Launch("chromium stderr not captured".to_owned()))?;
    let deadline = Instant::now() + timeout;
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        if Instant::now() >= deadline {
            let _ = child.kill();
            return Err(CdpError::Launch(
                "timed out waiting for DevTools endpoint".to_owned(),
            ));
        }
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|e| CdpError::Launch(format!("reading chromium stderr: {e}")))?;
        if read == 0 {
            let _ = child.kill();
            return Err(CdpError::Launch(
                "chromium exited before announcing DevTools endpoint".to_owned(),
            ));
        }
        if let Some(idx) = line.find("ws://") {
            let ws_url = line[idx..].trim().to_owned();
            return Ok((child, ws_url, user_data_dir));
        }
    }
}
```

> Note: `read_line` blocks; the deadline is checked between lines. Chromium emits the DevTools line within ~1s of a successful start, so a 20s budget is ample. If hardening is ever needed, move the read onto a thread with a channel + `recv_timeout`; not required for the smoke suite.

- [ ] **Step 4: Run the gated test**

Run: `cargo test -p kali_cli --test browser_cdp_smoke spawns_chromium -- --nocapture 2>&1 | tail -8`
Expected: PASS (skips silently if Chromium absent; on this container it launches and reports a `ws://` URL).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): launch headless Chromium and discover its CDP ws endpoint"
```

---

### Task 4: Connect and complete a request/response round-trip

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs`

**Interfaces:**
- Consumes: `CdpConnection`, `CdpIncoming`, `CdpError`; `spawn_chromium` (Task 3).
- Produces:
  - `pub struct CdpBrowser { child: Child, conn: CdpConnection, _user_data_dir: TempDir }`
  - `CdpBrowser::launch(executable: &str, timeout: Duration) -> Result<Self, CdpError>`
  - `CdpBrowser::call(&mut self, method: &str, params: Value, session_id: Option<&str>, timeout: Duration) -> Result<Value, CdpError>` (send + read until the matching response id; discard unrelated events).
  - `CdpBrowser::close(mut self) -> Result<(), CdpError>` (best-effort `Browser.close`, then kill + reap).

- [ ] **Step 1: Write the failing gated test**

Add to the `tests` module in `driver.rs`:

```rust
    #[test]
    fn round_trips_browser_get_version() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let mut browser =
            CdpBrowser::launch("chromium", std::time::Duration::from_secs(20)).expect("launch");
        let result = browser
            .call(
                "Browser.getVersion",
                serde_json::json!({}),
                None,
                std::time::Duration::from_secs(20),
            )
            .expect("getVersion");
        assert!(
            result["product"].as_str().unwrap_or_default().contains("Chrom"),
            "unexpected product: {result}"
        );
        browser.close().expect("close");
    }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke round_trips 2>&1 | tail -5`
Expected: FAIL to compile — `CdpBrowser` not defined.

- [ ] **Step 3: Implement `CdpBrowser::launch/call/close`**

Add to `driver.rs` (after the free functions, before the `tests` module):

```rust
/// A launched Chromium plus its blocking CDP connection.
pub struct CdpBrowser {
    child: Child,
    conn: CdpConnection,
    _user_data_dir: TempDir,
}

impl CdpBrowser {
    /// Launch Chromium and open a CDP connection to its browser endpoint.
    pub fn launch(executable: &str, timeout: Duration) -> Result<Self, CdpError> {
        let (child, ws_url, user_data_dir) = spawn_chromium(executable, timeout)?;
        let (socket, _response) = tungstenite::connect(&ws_url)
            .map_err(|e| CdpError::Transport(format!("connect {ws_url}: {e}")))?;
        let mut conn = CdpConnection::from_socket(socket);
        conn.set_read_timeout(timeout)?;
        Ok(Self { child, conn, _user_data_dir: user_data_dir })
    }

    /// Send a method and read frames until its matching response, returning `result`.
    /// Unrelated events are discarded here (page runs collect them in Task 5).
    pub fn call(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        self.conn.set_read_timeout(timeout)?;
        let id = self.conn.send(method, params, session_id)?;
        loop {
            match self.conn.read()? {
                CdpIncoming::Result { id: got, result } if got == id => return Ok(result),
                CdpIncoming::Error { id: got, message } if got == id => {
                    return Err(CdpError::Protocol(format!("{method}: {message}")))
                }
                _ => continue,
            }
        }
    }

    /// Best-effort clean shutdown: ask the browser to close, then kill and reap.
    pub fn close(mut self) -> Result<(), CdpError> {
        let _ = self.conn.send("Browser.close", json!({}), None);
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }
}
```

- [ ] **Step 4: Run the gated test**

Run: `cargo test -p kali_cli --test browser_cdp_smoke round_trips -- --nocapture 2>&1 | tail -8`
Expected: PASS — `product` contains "Chrom".

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): connect to Chromium CDP and round-trip Browser.getVersion"
```

---

### Task 5: Run a page — capture console output and detect completion

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs`

**Interfaces:**
- Consumes: `CdpBrowser::call`, `CdpConnection::read`.
- Produces:
  - `pub struct CdpConsoleLine { pub kind: String, pub text: String }`
  - `pub struct CdpPageOutcome { pub console: Vec<CdpConsoleLine>, pub completed: bool }` with `pub fn stdout(&self) -> String` (the `log`-kind lines joined with `\n` plus a trailing `\n`, reproducing node's line-oriented stdout).
  - `CdpBrowser::run_page(&mut self, url: &str, timeout: Duration) -> Result<CdpPageOutcome, CdpError>`
  - `pub(crate) const CDP_DONE_BINDING: &str = "__kaliHarnessDone";`

**Completion protocol:** the driver registers a CDP binding `__kaliHarnessDone` (`Runtime.addBinding`). A page signals completion by calling `globalThis.__kaliHarnessDone?.()`, which raises a `Runtime.bindingCalled` event. Pages that never call it end on `timeout` (returned with `completed: false`).

- [ ] **Step 1: Write the failing gated test (self-contained data: URL)**

Add to the `tests` module:

```rust
    #[test]
    fn runs_page_captures_console_and_completes() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let mut browser =
            CdpBrowser::launch("chromium", std::time::Duration::from_secs(20)).expect("launch");
        let html = "<!doctype html><meta charset=utf-8><script type=\"module\">\
console.log('3'); console.log('3'); globalThis.__kaliHarnessDone && globalThis.__kaliHarnessDone();\
</script>";
        let url = format!("data:text/html,{}", html);
        let outcome = browser
            .run_page(&url, std::time::Duration::from_secs(30))
            .expect("run page");
        browser.close().expect("close");

        assert!(outcome.completed, "page should have signaled completion");
        let stdout = outcome.stdout();
        assert!(stdout.contains("3\n"), "stdout: {stdout:?}");
        assert!(stdout.matches("3\n").count() >= 2, "stdout: {stdout:?}");
    }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke runs_page 2>&1 | tail -5`
Expected: FAIL to compile — `run_page` / `CdpPageOutcome` not defined.

- [ ] **Step 3: Implement `run_page` and the outcome types**

Add to `driver.rs`:

```rust
/// The completion binding a harness page calls to signal it finished.
pub(crate) const CDP_DONE_BINDING: &str = "__kaliHarnessDone";

/// One captured console call from the page.
#[derive(Clone, Debug)]
pub struct CdpConsoleLine {
    /// Console method: "log", "error", "warn", "info", "debug".
    pub kind: String,
    /// The joined, stringified arguments.
    pub text: String,
}

/// The result of running one page in the browser.
#[derive(Clone, Debug)]
pub struct CdpPageOutcome {
    /// Console calls in emission order.
    pub console: Vec<CdpConsoleLine>,
    /// Whether the page invoked the completion binding before the timeout.
    pub completed: bool,
}

impl CdpPageOutcome {
    /// Reproduce node-style stdout: every `log` line joined with newlines.
    pub fn stdout(&self) -> String {
        let mut out = String::new();
        for line in &self.console {
            if line.kind == "log" {
                out.push_str(&line.text);
                out.push('\n');
            }
        }
        out
    }
}

/// Extract the text of a `Runtime.consoleAPICalled` arg (RemoteObject).
fn console_arg_text(arg: &Value) -> String {
    if let Some(value) = arg.get("value") {
        return match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
    }
    arg.get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

impl CdpBrowser {
    /// Open a fresh target, navigate to `url`, capture console output, and return
    /// when the page calls the completion binding or `timeout` elapses.
    pub fn run_page(
        &mut self,
        url: &str,
        timeout: Duration,
    ) -> Result<CdpPageOutcome, CdpError> {
        let target = self.call(
            "Target.createTarget",
            json!({ "url": "about:blank" }),
            None,
            timeout,
        )?;
        let target_id = target["targetId"]
            .as_str()
            .ok_or_else(|| CdpError::Protocol("createTarget: no targetId".to_owned()))?
            .to_owned();
        let attach = self.call(
            "Target.attachToTarget",
            json!({ "targetId": target_id, "flatten": true }),
            None,
            timeout,
        )?;
        let session_id = attach["sessionId"]
            .as_str()
            .ok_or_else(|| CdpError::Protocol("attachToTarget: no sessionId".to_owned()))?
            .to_owned();
        let session = Some(session_id.as_str());

        self.call("Runtime.enable", json!({}), session, timeout)?;
        self.call("Page.enable", json!({}), session, timeout)?;
        self.call(
            "Runtime.addBinding",
            json!({ "name": CDP_DONE_BINDING }),
            session,
            timeout,
        )?;
        self.call("Page.navigate", json!({ "url": url }), session, timeout)?;

        let deadline = Instant::now() + timeout;
        let mut console = Vec::new();
        let mut completed = false;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            self.conn.set_read_timeout(remaining)?;
            match self.conn.read() {
                Ok(CdpIncoming::Event { method, params, .. })
                    if method == "Runtime.consoleAPICalled" =>
                {
                    let kind = params["type"].as_str().unwrap_or("log").to_owned();
                    let text = params["args"]
                        .as_array()
                        .map(|args| {
                            args.iter()
                                .map(console_arg_text)
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .unwrap_or_default();
                    console.push(CdpConsoleLine { kind, text });
                }
                Ok(CdpIncoming::Event { method, params, .. })
                    if method == "Runtime.bindingCalled"
                        && params["name"].as_str() == Some(CDP_DONE_BINDING) =>
                {
                    completed = true;
                    break;
                }
                Ok(_) => continue,
                Err(CdpError::Timeout) => break,
                Err(other) => return Err(other),
            }
        }

        let _ = self.call(
            "Target.closeTarget",
            json!({ "targetId": target_id }),
            None,
            Duration::from_secs(5),
        );
        Ok(CdpPageOutcome { console, completed })
    }
}
```

- [ ] **Step 4: Run the gated test**

Run: `cargo test -p kali_cli --test browser_cdp_smoke runs_page -- --nocapture 2>&1 | tail -8`
Expected: PASS — `completed == true`, stdout contains "3\n" twice.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): CDP run_page — capture console output, detect completion via binding"
```

---

### Task 6: Gated end-to-end smoke test against a real Kali bundle

**Files:**
- Modify: `crates/kali_cli/tests/browser_cdp_smoke.rs`

**Interfaces:**
- Consumes: `cdp_driver::CdpBrowser`; `kali_runtime::browser_bundle_harness_script` (production API); the `kali` binary via `CARGO_BIN_EXE_kali`.

This test builds a real browser bundle with `kali build --bundle --api browser`, writes a bundle harness whose body imports the bundle, runs it, logs a value, and calls the completion binding, then drives it through one shared `CdpBrowser`. It is `#[ignore]`-gated and skips cleanly if Chromium is unavailable.

- [ ] **Step 1: Write the ignored smoke test**

Replace `crates/kali_cli/tests/browser_cdp_smoke.rs` with:

```rust
mod cdp_driver;

use std::fs;
use std::process::Command;
use std::time::Duration;

use tempfile::tempdir;

use cdp_driver::CdpBrowser;

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

#[test]
#[ignore = "requires a real Chromium; run with `-- --ignored`"]
fn real_chromium_runs_a_browser_bundle_and_captures_console() {
    let Some(chromium_exe) = chromium() else {
        eprintln!("skipping: no Chromium available");
        return;
    };

    // 1. Build a browser bundle from a program that logs a known value (1 + 2).
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("main.ts");
    fs::write(&source, "export function main(): void { console.log(1 + 2); }\nmain();\n")
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
    let bundle_dir = dir.path().join("main");
    assert!(bundle_dir.join("main.js").exists(), "bundle glue missing");

    // 3. Write a harness whose body imports the bundle, runs it, then signals done.
    let harness_path = bundle_dir
        .parent()
        .expect("bundle parent")
        .join("cdp-harness.html");
    let body = "const mod = await import(bundleJs.href);\n\
const instance = await mod.load();\n\
if (typeof instance.exports.main === 'function') { instance.exports.main(); }\n\
globalThis.__kaliHarnessDone && globalThis.__kaliHarnessDone();\n";
    let harness = kali_runtime::browser_bundle_harness_script("main", false, body);
    fs::write(&harness_path, harness).expect("write harness");

    // 4. Drive it through a single shared Chromium via CDP.
    let mut browser = CdpBrowser::launch(&chromium_exe, Duration::from_secs(20))
        .expect("launch chromium");
    let url = format!("file://{}", harness_path.display());
    let outcome = browser
        .run_page(&url, Duration::from_secs(30))
        .expect("run page");
    browser.close().expect("close");

    // 5. Assert the real browser produced the program's console output.
    assert!(outcome.completed, "harness did not signal completion");
    assert!(
        outcome.stdout().contains("3\n"),
        "unexpected stdout: {:?}",
        outcome.stdout()
    );
}
```

> If `mod.load()` is not the correct entry for a `main`-style export, inspect the emitted `bundle_dir/main.js` (Part A's glue exposes `load`/`loadWithImports` plus per-export async wrappers) and adjust step 3's body. The assertion target `"3\n"` is `1 + 2`.

- [ ] **Step 2: Verify it is ignored by default**

Run: `cargo test -p kali_cli --test browser_cdp_smoke 2>&1 | grep "test result:"`
Expected: the pure `parses_result…` unit test passes and the smoke test is counted under `ignored` (e.g. `... 1 passed; 0 failed; N ignored` — the browser-gated `#[cfg(test)]` driver tests that probe Chromium also run/skip here).

- [ ] **Step 3: Run it explicitly and verify it passes against real Chromium**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored --nocapture 2>&1 | tail -12`
Expected: PASS — real Chromium loads the bundle, `stdout` contains `3\n`. (Without Chromium it prints the skip line and passes.)

- [ ] **Step 4: Confirm a single browser instance and no leaks**

While step 3 runs, in another shell: `pgrep -c chromium` peaks at one browser's process group (not one-per-case) and returns to 0 after the test. If non-zero after completion, `close()` is not reaping — revisit Task 4 step 3.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/browser_cdp_smoke.rs
git commit -m "test(cli): gated CDP smoke — run a real browser bundle in shared Chromium"
```

---

### Task 7: Final gate — format, lint, document

**Files:**
- Modify: `docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md`

- [ ] **Step 1: Format and lint**

Run: `cargo fmt --check && cargo clippy -p kali_cli --tests 2>&1 | grep -E "^(warning|error)" | head`
Expected: `fmt` exits 0; clippy prints no warnings/errors for the new test code.

If `fmt` reports diffs, run `cargo fmt` and re-check. If clippy flags the new code, fix inline and re-run.

- [ ] **Step 2: Confirm the default suite is unaffected**

Run: `cargo test -p kali_cli --test browser_cdp_smoke 2>&1 | grep "test result:"`
Expected: the end-to-end smoke test is `ignored`; only the pure unit test runs by default. No browser is spawned in the default run (Chromium-gated driver tests skip unless Chromium is present, and even then are fast/clean).

Run: `pgrep -c chromium` after the default run.
Expected: `0`.

- [ ] **Step 3: Mark Part B implemented in the spec**

In `docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md`, under "Part B — future work", update the status line and append:

```markdown
**Implemented as a gated, test-only driver.** Run the real-browser smoke test with:
`cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
(requires Chromium; launched with `--no-sandbox` for containers). The driver is
test infrastructure, not production code: it lives in
`crates/kali_cli/tests/cdp_driver/` with `tungstenite` as a `kali_cli`
dev-dependency, so nothing enters production builds.
```

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md
git commit -m "docs(spec): mark CDP smoke driver implemented (test-only); record run command"
```

---

## Self-Review

**Spec coverage (Part B of `2026-07-01-browser-harness-node-preference-design.md`):**
- "sync CDP driver" → Tasks 2–5 (blocking `tungstenite`, no async). ✓
- "launch `chromium --headless --no-sandbox --remote-debugging-port=0`, read WS endpoint from stderr" → Task 3. ✓
- "capture `Runtime.consoleAPICalled`" → Task 5. ✓
- "detect completion, `Browser.close`" → Task 5 (binding) + Task 4 (`close`). ✓
- "replay captured console to the driver's own stdout to match the node contract" → `CdpPageOutcome::stdout()` (Task 5). ✓
- "one shared browser instance, new target/page per test" → `run_page` per case (Task 5) + one `CdpBrowser` (Task 6). ✓
- "bounded launch timeout so it can never hang" → per-op `set_read_timeout` (Tasks 2/5) + launch deadline (Task 3). ✓
- "small, explicitly gated smoke suite, not the default path" → `#[ignore]` + availability skip (Task 6); default resolution untouched; **driver is test-only, in `kali_cli`'s test tree, `tungstenite` a dev-dependency** (Global Constraints, Task 1). ✓
- "hand-rolled or `tungstenite`" → chose `tungstenite` (Task 1). ✓

**Placeholder scan:** No TBD/TODO; every code step shows full code; the two adjustment notes (Task 3 read-thread, Task 6 entry adjustment) are explicit and optional, not placeholders.

**Type consistency:** `CdpError`/`CdpIncoming`/`CdpConnection` (protocol.rs) are consumed verbatim in driver.rs; `CdpBrowser`/`CdpPageOutcome`/`CdpConsoleLine` are defined in Tasks 4–5, re-exported by `cdp_driver/mod.rs`, and consumed as `cdp_driver::CdpBrowser` in Task 6; `CDP_DONE_BINDING` value `__kaliHarnessDone` matches the page-side `globalThis.__kaliHarnessDone` call in Tasks 5 and 6.

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-01-browser-cdp-driver.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?** (Or, since Part B is explicitly deferred future work, we can leave the plan on disk and not execute now.)
