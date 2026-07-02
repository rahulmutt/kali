# CDP Driver Follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the recorded test-side follow-ups on the CDP smoke driver: make its command/page-run logic testable without a browser (transport trait + scripted fake), replay the event-drop race deterministically, bound `call()` with an aggregate deadline, and fix two probe/server hygiene nits.

**Architecture:** Extract a `CdpTransport` trait in `protocol.rs` (implemented by the real `CdpConnection`) and move `call()`/`run_page()` verbatim into a new transport-generic `CdpClient<T>` in `driver.rs`; `CdpBrowser` keeps its exact public API and delegates. Tests then drive `CdpClient` with scripted fakes: a replayed frame sequence proves events arriving before a command response are buffered (the race, deterministic and browser-free), and a "chatty" fake whose response never arrives proves `call()` gives up at an aggregate deadline instead of looping while events stream.

**Tech Stack:** Rust, serde_json, tungstenite (blocking WebSocket), headless Chromium via CDP. No new dependencies.

## Global Constraints

- Test-only scope: modify ONLY files under `crates/kali_cli/tests/` (`cdp_driver/protocol.rs`, `cdp_driver/driver.rs`, `cdp_driver/mod.rs`, `browser_cdp_smoke.rs`). Zero production-code changes.
- `CdpBrowser`'s public API is frozen: `launch(executable: &str, timeout: Duration) -> Result<Self, CdpError>`, `call(&mut self, method: &str, params: Value, session_id: Option<&str>, timeout: Duration) -> Result<Value, CdpError>`, `run_page(&mut self, url: &str, timeout: Duration) -> Result<CdpPageOutcome, CdpError>`, `close(self) -> Result<(), CdpError>`. `mod.rs` keeps `pub use driver::{CdpBrowser, CdpConsoleLine, CdpPageOutcome};` intact.
- Behavior of the merged console-capture fixes is frozen: pending-event buffering, `route_event` session filtering, `"warning"`→`"warn"` normalization, `stdout()` = log+info+debug, `stderr()` = warn+error.
- Unit lane (no browser needed): `cargo test -p kali_cli --test browser_cdp_smoke`
- Gated browser lane (needs Chromium): `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` (also `mise run browser-smoke`). If no Chromium is available the gated tests self-skip — run the unit lane and note the skip; do not treat it as failure.
- Repo hygiene before every commit: `cargo fmt` leaves no diff; at each task's commit `cargo clippy -p kali_cli --tests -- -D warnings` is clean.

---

### Task 1: `CdpTransport` trait + `CdpClient` extraction (pure refactor)

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/protocol.rs` (add trait + impl after the `impl CdpConnection` block)
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (new `CdpClient<T>`; `CdpBrowser` delegates)

**Interfaces:**
- Consumes: existing `CdpConnection::{send, read, set_read_timeout}`, `CdpIncoming`, `CdpError`.
- Produces (Tasks 2 and 3 rely on these exact shapes):
  - `pub(crate) trait CdpTransport` in `protocol.rs` with methods `send(&mut self, method: &str, params: Value, session_id: Option<&str>) -> Result<u64, CdpError>`, `read(&mut self) -> Result<CdpIncoming, CdpError>`, `set_read_timeout(&mut self, timeout: Duration) -> Result<(), CdpError>`; implemented by `CdpConnection`.
  - `struct CdpClient<T: CdpTransport> { transport: T, pending_events: VecDeque<CdpIncoming> }` in `driver.rs` (module-private; the `mod tests` child can see it) with `fn new(transport: T) -> Self`, and `call`/`run_page` methods with the same parameter/return types as today's `CdpBrowser` methods.

This task is code motion with zero behavior change. There is deliberately no new test; the gate is that both existing lanes stay green and clippy stays clean.

- [ ] **Step 1: Add the trait to `protocol.rs`**

Insert immediately after the closing brace of `impl CdpConnection { ... }` (before `#[cfg(test)] mod tests`):

```rust
/// The transport operations the driver needs from a CDP connection.
/// Extracted as a trait so the driver's command/page-run logic can be
/// exercised against a scripted fake without a browser or a socket.
pub(crate) trait CdpTransport {
    /// Send a CDP method call, optionally scoped to a flat session. Returns its id.
    fn send(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
    ) -> Result<u64, CdpError>;
    /// Read the next decoded message.
    fn read(&mut self) -> Result<CdpIncoming, CdpError>;
    /// Bound subsequent reads.
    fn set_read_timeout(&mut self, timeout: std::time::Duration) -> Result<(), CdpError>;
}

impl CdpTransport for CdpConnection {
    fn send(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
    ) -> Result<u64, CdpError> {
        CdpConnection::send(self, method, params, session_id)
    }

    fn read(&mut self) -> Result<CdpIncoming, CdpError> {
        CdpConnection::read(self)
    }

    fn set_read_timeout(&mut self, timeout: std::time::Duration) -> Result<(), CdpError> {
        CdpConnection::set_read_timeout(self, timeout)
    }
}
```

- [ ] **Step 2: Move `call`/`run_page` into `CdpClient` in `driver.rs`**

2a. Change the protocol import at the top of `driver.rs` from:

```rust
use super::protocol::{CdpConnection, CdpError, CdpIncoming};
```

to:

```rust
use super::protocol::{CdpConnection, CdpError, CdpIncoming, CdpTransport};
```

2b. Insert immediately BEFORE the `/// A launched Chromium plus its blocking CDP connection.` doc comment of `CdpBrowser` (the method bodies below are today's `CdpBrowser::call` and `CdpBrowser::run_page` verbatim, with `self.conn` renamed to `self.transport` — no other body edits):

```rust
/// Transport-generic CDP client: command calls and the page-run event pump.
/// Generic over [`CdpTransport`] so tests can drive it with a scripted fake.
struct CdpClient<T: CdpTransport> {
    transport: T,
    /// Events received while a command call was awaiting its response,
    /// preserved for the next page-run loop instead of being discarded.
    /// The buffer is unbounded; fine for this test-only driver whose calls are short.
    pending_events: VecDeque<CdpIncoming>,
}

impl<T: CdpTransport> CdpClient<T> {
    fn new(transport: T) -> Self {
        Self {
            transport,
            pending_events: VecDeque::new(),
        }
    }

    /// Send a method and read frames until its matching response, returning `result`.
    /// Events that arrive first are buffered for the page-run loop, never dropped.
    fn call(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        self.transport.set_read_timeout(timeout)?;
        let id = self.transport.send(method, params, session_id)?;
        loop {
            match self.transport.read()? {
                CdpIncoming::Result { id: got, result } if got == id => return Ok(result),
                CdpIncoming::Error { id: got, message } if got == id => {
                    return Err(CdpError::Protocol(format!("{method}: {message}")))
                }
                event @ CdpIncoming::Event { .. } => self.pending_events.push_back(event),
                _ => continue,
            }
        }
    }

    /// Open a fresh target, navigate to `url`, capture console output, and return
    /// when the page calls the completion binding or `timeout` elapses.
    fn run_page(&mut self, url: &str, timeout: Duration) -> Result<CdpPageOutcome, CdpError> {
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
            // Drain events buffered during call() first — they arrived before
            // anything still on the socket, and they are already ours to read
            // even if the deadline has passed.
            let incoming = if let Some(event) = self.pending_events.pop_front() {
                Ok(event)
            } else {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                self.transport.set_read_timeout(remaining)?;
                self.transport.read()
            };
            match incoming {
                Ok(CdpIncoming::Event {
                    method,
                    params,
                    session_id: event_session,
                }) => match route_event(&method, &params, event_session.as_deref(), &session_id) {
                    PageEvent::Console(line) => console.push(line),
                    PageEvent::Completed => {
                        completed = true;
                        break;
                    }
                    PageEvent::Ignore => {}
                },
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

2c. Replace the `CdpBrowser` struct definition and BOTH of its `impl CdpBrowser` blocks (the one containing `launch`/`call`/`close` and the later one containing `run_page`) with the following. The `impl Drop for CdpBrowser` block between them stays exactly as it is:

```rust
/// A launched Chromium plus its blocking CDP connection.
pub struct CdpBrowser {
    child: Child,
    client: CdpClient<CdpConnection>,
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
        Ok(Self {
            child,
            client: CdpClient::new(conn),
            _user_data_dir: user_data_dir,
        })
    }

    /// Send a method and read frames until its matching response, returning `result`.
    /// Events that arrive first are buffered for the page-run loop, never dropped.
    pub fn call(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        self.client.call(method, params, session_id, timeout)
    }

    /// Open a fresh target, navigate to `url`, capture console output, and return
    /// when the page calls the completion binding or `timeout` elapses.
    pub fn run_page(&mut self, url: &str, timeout: Duration) -> Result<CdpPageOutcome, CdpError> {
        self.client.run_page(url, timeout)
    }

    /// Best-effort clean shutdown: ask the browser to close, then kill and reap.
    pub fn close(mut self) -> Result<(), CdpError> {
        let _ = self.client.transport.send("Browser.close", json!({}), None);
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }
}
```

Notes: the old `pending_events` field moves off `CdpBrowser` (it lives on `CdpClient` now). `close()` reaches the transport through the private `client.transport` field — legal because `CdpClient` is defined in the same module. Delete the now-empty second `impl CdpBrowser` block entirely.

- [ ] **Step 3: Run the unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 7 passed / 5 ignored, no warnings (everything moved is still used).

- [ ] **Step 4: Run the gated browser lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: PASS — 5/5 against real Chromium (refactor changed nothing observable). Skip only if Chromium is unavailable (tests self-skip; note it).

- [ ] **Step 5: Lint, format, commit**

```bash
cargo clippy -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/cdp_driver/protocol.rs crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): extract CdpTransport trait and transport-generic CdpClient (pure refactor)"
```

Expected: clippy clean; fmt no diff.

---

### Task 2: `FakeTransport` + deterministic browser-free race regression test

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (test-support fake + one test, all inside the existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `CdpClient::{new, run_page}` and the `CdpTransport` trait from Task 1; `CdpIncoming`, `CdpError`, `CDP_DONE_BINDING`.
- Produces (Task 3 relies on this): `struct FakeTransport` in `mod tests` implementing `CdpTransport`, with a `sent: Vec<String>` field recording sent method names, `fn new(frames: Vec<CdpIncoming>) -> Self`, ids handed out 1,2,3…, and `read()` returning `Err(CdpError::Timeout)` once the scripted frames run out.

- [ ] **Step 1: Add the fake and the test**

Add to the bottom of `mod tests` in `driver.rs`:

```rust
    /// A scripted transport: `send` records method names and hands out ids
    /// 1,2,3…; `read` replays a fixed frame sequence, then times out.
    struct FakeTransport {
        frames: VecDeque<CdpIncoming>,
        sent: Vec<String>,
        next_id: u64,
    }

    impl FakeTransport {
        fn new(frames: Vec<CdpIncoming>) -> Self {
            Self {
                frames: frames.into(),
                sent: Vec::new(),
                next_id: 1,
            }
        }
    }

    impl CdpTransport for FakeTransport {
        fn send(
            &mut self,
            method: &str,
            _params: Value,
            _session_id: Option<&str>,
        ) -> Result<u64, CdpError> {
            self.sent.push(method.to_owned());
            let id = self.next_id;
            self.next_id += 1;
            Ok(id)
        }

        fn read(&mut self) -> Result<CdpIncoming, CdpError> {
            self.frames.pop_front().ok_or(CdpError::Timeout)
        }

        fn set_read_timeout(&mut self, _timeout: Duration) -> Result<(), CdpError> {
            Ok(())
        }
    }

    fn result_frame(id: u64, result: Value) -> CdpIncoming {
        CdpIncoming::Result { id, result }
    }

    fn event_frame(method: &str, params: Value, session: &str) -> CdpIncoming {
        CdpIncoming::Event {
            method: method.to_owned(),
            params,
            session_id: Some(session.to_owned()),
        }
    }

    #[test]
    fn run_page_keeps_events_that_arrive_before_the_navigate_response() {
        // The document-start race, replayed deterministically: the page's whole
        // console output AND its completion binding arrive while Page.navigate's
        // response is still pending. A driver that drops events during call()
        // loses all of them and times out incomplete.
        let mut client = CdpClient::new(FakeTransport::new(vec![
            result_frame(1, serde_json::json!({ "targetId": "T1" })), // Target.createTarget
            result_frame(2, serde_json::json!({ "sessionId": "S1" })), // Target.attachToTarget
            result_frame(3, serde_json::json!({})),                  // Runtime.enable
            result_frame(4, serde_json::json!({})),                  // Page.enable
            result_frame(5, serde_json::json!({})),                  // Runtime.addBinding
            event_frame(
                "Runtime.consoleAPICalled",
                serde_json::json!({ "type": "log", "args": [{ "value": "early" }] }),
                "S1",
            ),
            event_frame(
                "Runtime.consoleAPICalled",
                serde_json::json!({ "type": "warning", "args": [{ "value": "careful" }] }),
                "S1",
            ),
            event_frame(
                "Runtime.bindingCalled",
                serde_json::json!({ "name": CDP_DONE_BINDING, "payload": "" }),
                "S1",
            ),
            result_frame(6, serde_json::json!({})), // Page.navigate — AFTER the events
        ]));
        let outcome = client
            .run_page("http://unused.invalid/", Duration::from_secs(5))
            .expect("run page");

        assert!(
            outcome.completed,
            "completion arrived before the navigate response and must not be dropped; console: {:?}",
            outcome.console
        );
        assert_eq!(outcome.stdout(), "early\n");
        assert_eq!(outcome.stderr(), "careful\n");
        assert_eq!(
            client.transport.sent,
            [
                "Target.createTarget",
                "Target.attachToTarget",
                "Runtime.enable",
                "Page.enable",
                "Runtime.addBinding",
                "Page.navigate",
                "Target.closeTarget",
            ]
        );
    }
```

- [ ] **Step 2: Run the new test — it must pass against the current (buffering) driver**

Run: `cargo test -p kali_cli --test browser_cdp_smoke run_page_keeps_events_that_arrive_before_the_navigate_response`
Expected: PASS (this test pins already-merged behavior; the RED demonstration is Step 3).

- [ ] **Step 3: Prove the test detects the race (temporary local revert — do NOT commit this state)**

In `CdpClient::call`, temporarily change the buffering arm

```rust
                event @ CdpIncoming::Event { .. } => self.pending_events.push_back(event),
```

to the old event-dropping behavior:

```rust
                CdpIncoming::Event { .. } => continue,
```

Run the Step 2 command again.
Expected: FAIL, deterministically and instantly — `completed` is false (the fake's exhausted script returns `Timeout`, so the page-run loop breaks with everything dropped). This is the RED the gated browser test could not reproduce in this environment.

Then restore the buffering line exactly as it was (`git diff` must show only the Task 2 test additions) and re-run: PASS.

- [ ] **Step 4: Run the full unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 8 passed / 5 ignored, no warnings.

- [ ] **Step 5: Lint, format, commit**

```bash
cargo clippy -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): deterministic browser-free regression for the CDP event-drop race via FakeTransport"
```

Expected: clippy clean; fmt no diff. Record in the report which outputs Step 3 produced (the FAIL excerpt and the restored PASS).

---

### Task 3: Aggregate deadline for `call()`

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (`CdpClient::call` body; one new fake + test in `mod tests`)

**Interfaces:**
- Consumes: `CdpClient::{new, call}` and `CdpTransport` from Task 1.
- Produces: `call()` returns `Err(CdpError::Timeout)` once `timeout` has elapsed in total, even if every individual read succeeds. Signatures unchanged.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `mod tests` in `driver.rs`:

```rust
    /// A transport whose `read` always yields another event after a short
    /// delay and whose command response never arrives — bounded so a driver
    /// without an aggregate deadline still terminates the test (with the
    /// wrong error, after much longer).
    struct ChattyTransport {
        events_left: u32,
    }

    impl CdpTransport for ChattyTransport {
        fn send(
            &mut self,
            _method: &str,
            _params: Value,
            _session_id: Option<&str>,
        ) -> Result<u64, CdpError> {
            Ok(1)
        }

        fn read(&mut self) -> Result<CdpIncoming, CdpError> {
            if self.events_left == 0 {
                return Err(CdpError::Transport("chatty script exhausted".to_owned()));
            }
            self.events_left -= 1;
            std::thread::sleep(Duration::from_millis(5));
            Ok(CdpIncoming::Event {
                method: "Page.frameNavigated".to_owned(),
                params: serde_json::json!({}),
                session_id: None,
            })
        }

        fn set_read_timeout(&mut self, _timeout: Duration) -> Result<(), CdpError> {
            Ok(())
        }
    }

    #[test]
    fn call_gives_up_at_its_deadline_even_while_events_keep_streaming() {
        // 400 events x 5ms = ~2s of chatter; the response never arrives. A call
        // bounded only per-read never times out (every read succeeds) and ends
        // up consuming the whole script; an aggregate deadline must abandon the
        // call at ~50ms with CdpError::Timeout.
        let mut client = CdpClient::new(ChattyTransport { events_left: 400 });
        let started = Instant::now();
        let result = client.call(
            "Browser.getVersion",
            serde_json::json!({}),
            None,
            Duration::from_millis(50),
        );
        let elapsed = started.elapsed();
        assert!(
            matches!(result, Err(CdpError::Timeout)),
            "expected Timeout, got {result:?}"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "call ran far past its deadline: {elapsed:?}"
        );
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke call_gives_up_at_its_deadline_even_while_events_keep_streaming`
Expected: FAIL after roughly 2 seconds with `expected Timeout, got Err(Transport("chatty script exhausted"))` — the current per-read-only `call()` happily consumes all 400 events.

- [ ] **Step 3: Implement the aggregate deadline**

Replace the entire `fn call` in `impl<T: CdpTransport> CdpClient<T>` with:

```rust
    /// Send a method and read frames until its matching response, returning `result`.
    /// Events that arrive first are buffered for the page-run loop, never dropped.
    /// The whole call is bounded by `timeout`, even while events keep arriving.
    fn call(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let deadline = Instant::now() + timeout;
        let id = self.transport.send(method, params, session_id)?;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(CdpError::Timeout);
            }
            self.transport.set_read_timeout(remaining)?;
            match self.transport.read()? {
                CdpIncoming::Result { id: got, result } if got == id => return Ok(result),
                CdpIncoming::Error { id: got, message } if got == id => {
                    return Err(CdpError::Protocol(format!("{method}: {message}")))
                }
                event @ CdpIncoming::Event { .. } => self.pending_events.push_back(event),
                _ => continue,
            }
        }
    }
```

(The pre-loop `set_read_timeout(timeout)` is gone — each iteration now bounds the read by what remains of the deadline, so a real socket read can never outlive it either.)

- [ ] **Step 4: Run the unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 9 passed / 5 ignored (the new test finishes in ~50ms), no warnings.

- [ ] **Step 5: Run the gated browser lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: PASS — 5/5 against real Chromium (real calls respond in milliseconds against 20–30s timeouts; behavior unchanged). Skip only if Chromium is unavailable.

- [ ] **Step 6: Lint, format, commit**

```bash
cargo clippy -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): bound CdpClient::call by an aggregate deadline, not just per-read timeouts"
```

Expected: clippy clean; fmt no diff.

---

### Task 4: Probe exit-status checks + serve_dir no-op read cleanup

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (`chromium_available`; one new unit test)
- Modify: `crates/kali_cli/tests/cdp_driver/mod.rs` (re-export `chromium_available`)
- Modify: `crates/kali_cli/tests/browser_cdp_smoke.rs` (`chromium()` reuses the fixed probe; delete the no-op read in `serve_dir`)

**Interfaces:**
- Consumes: `chromium_available(executable: &str) -> bool` (driver.rs), `chromium() -> Option<String>` and `serve_dir` (browser_cdp_smoke.rs).
- Produces: `chromium_available` returns true only when `--version` exits 0; `mod.rs` additionally re-exports it (`pub(crate) use driver::chromium_available;`).

- [ ] **Step 1: Write the failing unit test**

Add to the bottom of `mod tests` in `driver.rs`:

```rust
    #[test]
    fn chromium_available_requires_a_zero_exit_status() {
        // Not on PATH at all:
        assert!(!chromium_available("kali-cdp-test-no-such-browser"));
        // Spawns fine but exits non-zero (`false --version` exits 1):
        assert!(!chromium_available("false"));
        // Spawns fine and exits 0 (`true --version` exits 0 under coreutils):
        assert!(chromium_available("true"));
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke chromium_available_requires_a_zero_exit_status`
Expected: FAIL on `assert!(!chromium_available("false"))` — the current probe only checks that spawning succeeded (`output().is_ok()`), not the exit status.

- [ ] **Step 3: Fix the probe, re-export it, reuse it in the smoke test**

3a. In `driver.rs`, replace the whole `chromium_available` function with:

```rust
/// Whether the given browser executable can be invoked (`--version` exits 0).
pub(crate) fn chromium_available(executable: &str) -> bool {
    Command::new(executable)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
```

3b. In `mod.rs`, add below the existing `pub use`:

```rust
pub(crate) use driver::chromium_available;
```

3c. In `browser_cdp_smoke.rs`, replace the whole `chromium()` function with:

```rust
fn chromium() -> Option<String> {
    ["chromium", "chromium-browser", "google-chrome", "chrome"]
        .into_iter()
        .find(|&exe| cdp_driver::chromium_available(exe))
        .map(str::to_owned)
}
```

- [ ] **Step 4: Delete the no-op read in `serve_dir`**

In `browser_cdp_smoke.rs`, delete these two lines at the end of the connection loop (the zero-length read does nothing — the response was already written and flushed, request headers were fully drained above, so a graceful close delivers all queued data):

```rust
            // Best-effort: let the client read before the socket drops.
            let _ = stream.read(&mut [0u8; 0]);
```

If `use std::io::{... Read ...}` now carries an unused `Read` import, remove `Read` from that import list.

- [ ] **Step 5: Run the unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 10 passed / 5 ignored, no warnings (including no unused-import warning).

- [ ] **Step 6: Run the gated browser lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: PASS — 5/5; in particular `real_chromium_runs_a_browser_bundle_and_captures_console` still passes with the trimmed `serve_dir` and the stricter probe. Skip only if Chromium is unavailable.

- [ ] **Step 7: Lint, format, commit**

```bash
cargo clippy -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs crates/kali_cli/tests/cdp_driver/mod.rs crates/kali_cli/tests/browser_cdp_smoke.rs
git commit -m "test(cli): require --version exit 0 in browser probes; drop serve_dir's no-op read"
```

Expected: clippy clean; fmt no diff.

---

## Verification (whole plan)

- `cargo test -p kali_cli --test browser_cdp_smoke` — unit lane green: 10 passed / 5 ignored.
- `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` (or `mise run browser-smoke`) — gated lane green: 5/5, twice.
- `cargo clippy -p kali_cli --tests -- -D warnings` clean; `cargo fmt` no diff; `git status` clean; four commits.
