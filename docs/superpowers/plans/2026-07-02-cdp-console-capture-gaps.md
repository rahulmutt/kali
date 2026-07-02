# CDP Driver Console-Capture Gaps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the test-only CDP smoke driver so page console output is never silently dropped: buffer events that arrive during command calls, filter by session, capture uncaught exceptions, and make `stdout()`/`stderr()` match node's kind split.

**Architecture:** All changes live in the integration-test crate `crates/kali_cli/tests/` (compiled per-file as test binaries; `browser_cdp_smoke.rs` declares `mod cdp_driver;` so the driver's own `#[cfg(test)]` tests run in that binary). The fix has three parts: (1) a pure `route_event` helper that decides what one CDP event means for a page run (unit-testable without Chromium), (2) `stdout()`/`stderr()` node-parity helpers on `CdpPageOutcome`, (3) a `pending_events` buffer on `CdpBrowser` so events seen while `call()` awaits a response are queued instead of discarded, and a rewired `run_page()` loop that drains the buffer through `route_event`.

**Tech Stack:** Rust, serde_json, tungstenite (blocking WebSocket), headless Chromium via CDP. No new dependencies.

## Global Constraints

- Test-only scope: modify ONLY `crates/kali_cli/tests/cdp_driver/driver.rs`. No production code changes. Do not touch `protocol.rs` or `browser_cdp_smoke.rs`.
- Console kind strings are exact: `"log"`, `"info"`, `"debug"`, `"warn"`, `"error"`, `"exception"`.
- `stdout()` = kinds `log`, `info`, `debug` (node's stdout set). `stderr()` = kinds `warn`, `error`. `"exception"` lines appear in `outcome.console` only, never in `stdout()`/`stderr()`.
- Completion binding name is the existing constant `CDP_DONE_BINDING` (`"__kaliHarnessDone"`). Chromium `addBinding` functions require exactly one string argument.
- Unit lane (no browser needed): `cargo test -p kali_cli --test browser_cdp_smoke`
- Gated browser lane (needs Chromium): `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` (also exposed as `mise run browser-smoke`). If no Chromium binary is available locally, the gated tests self-skip with an eprintln — run the unit lane and note the skip; do not treat it as failure.
- Repo hygiene before every commit: `cargo fmt` must leave no diff.

---

### Task 1: Pure event-routing helper (`route_event` + `PageEvent`)

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (helper after `console_arg_text`, tests in the existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: existing `console_arg_text(arg: &Value) -> String`, `CDP_DONE_BINDING: &str`, `CdpConsoleLine { kind: String, text: String }`.
- Produces (Task 3 relies on these exact signatures):
  - `pub(crate) enum PageEvent { Console(CdpConsoleLine), Completed, Ignore }`
  - `pub(crate) fn route_event(method: &str, params: &Value, event_session: Option<&str>, page_session: &str) -> PageEvent`
  - `CdpConsoleLine` additionally derives `PartialEq`.

- [ ] **Step 1: Write the failing unit tests**

Add to the bottom of the existing `mod tests` in `crates/kali_cli/tests/cdp_driver/driver.rs` (these are NOT `#[ignore]`d — they need no browser):

```rust
    #[test]
    fn route_event_ignores_other_sessions() {
        let params = serde_json::json!({ "type": "log", "args": [{ "value": "hi" }] });
        assert_eq!(
            route_event("Runtime.consoleAPICalled", &params, Some("OTHER"), "S1"),
            PageEvent::Ignore
        );
        assert_eq!(
            route_event("Runtime.consoleAPICalled", &params, None, "S1"),
            PageEvent::Ignore
        );
        let done = serde_json::json!({ "name": CDP_DONE_BINDING, "payload": "" });
        assert_eq!(
            route_event("Runtime.bindingCalled", &done, Some("OTHER"), "S1"),
            PageEvent::Ignore
        );
    }

    #[test]
    fn route_event_captures_console_kinds_and_joins_args() {
        let params = serde_json::json!({
            "type": "info",
            "args": [{ "value": "a" }, { "value": 3 }]
        });
        assert_eq!(
            route_event("Runtime.consoleAPICalled", &params, Some("S1"), "S1"),
            PageEvent::Console(CdpConsoleLine {
                kind: "info".to_owned(),
                text: "a 3".to_owned(),
            })
        );
    }

    #[test]
    fn route_event_recognizes_completion_binding_only_by_name() {
        let done = serde_json::json!({ "name": CDP_DONE_BINDING, "payload": "" });
        assert_eq!(
            route_event("Runtime.bindingCalled", &done, Some("S1"), "S1"),
            PageEvent::Completed
        );
        let other = serde_json::json!({ "name": "someOtherBinding", "payload": "" });
        assert_eq!(
            route_event("Runtime.bindingCalled", &other, Some("S1"), "S1"),
            PageEvent::Ignore
        );
    }

    #[test]
    fn route_event_captures_exceptions_with_text_and_description() {
        let params = serde_json::json!({
            "exceptionDetails": {
                "text": "Uncaught",
                "exception": { "description": "Error: boom\n    at <anonymous>:1:1" }
            }
        });
        assert_eq!(
            route_event("Runtime.exceptionThrown", &params, Some("S1"), "S1"),
            PageEvent::Console(CdpConsoleLine {
                kind: "exception".to_owned(),
                text: "Uncaught Error: boom\n    at <anonymous>:1:1".to_owned(),
            })
        );
        // Missing description: only the text survives, no trailing separator.
        let bare = serde_json::json!({ "exceptionDetails": { "text": "Uncaught" } });
        assert_eq!(
            route_event("Runtime.exceptionThrown", &bare, Some("S1"), "S1"),
            PageEvent::Console(CdpConsoleLine {
                kind: "exception".to_owned(),
                text: "Uncaught".to_owned(),
            })
        );
    }

    #[test]
    fn route_event_ignores_unrelated_methods() {
        let params = serde_json::json!({});
        assert_eq!(
            route_event("Page.frameNavigated", &params, Some("S1"), "S1"),
            PageEvent::Ignore
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: FAIL to compile with errors like ``cannot find function `route_event` in this scope`` / ``cannot find type `PageEvent` ``. (In Rust TDD, a compile failure of the test is the failing state.)

- [ ] **Step 3: Implement `route_event` and `PageEvent`**

In `crates/kali_cli/tests/cdp_driver/driver.rs`:

3a. Change the `CdpConsoleLine` derive (currently `#[derive(Clone, Debug)]`) to:

```rust
/// One captured console call from the page.
#[derive(Clone, Debug, PartialEq)]
pub struct CdpConsoleLine {
```

3b. Insert immediately after the `console_arg_text` function:

```rust
/// What a page run should do with one incoming CDP event.
#[derive(Debug, PartialEq)]
pub(crate) enum PageEvent {
    /// Record this console (or exception) line.
    Console(CdpConsoleLine),
    /// The page called the completion binding.
    Completed,
    /// Not for this page run.
    Ignore,
}

/// Route one CDP event during a page run. Only events from `page_session`
/// count; other sessions and unrelated methods are ignored, so output from
/// a different target can neither bleed into the console nor fake completion.
pub(crate) fn route_event(
    method: &str,
    params: &Value,
    event_session: Option<&str>,
    page_session: &str,
) -> PageEvent {
    if event_session != Some(page_session) {
        return PageEvent::Ignore;
    }
    match method {
        "Runtime.consoleAPICalled" => {
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
            PageEvent::Console(CdpConsoleLine { kind, text })
        }
        "Runtime.bindingCalled" if params["name"].as_str() == Some(CDP_DONE_BINDING) => {
            PageEvent::Completed
        }
        "Runtime.exceptionThrown" => {
            let details = &params["exceptionDetails"];
            let mut parts = Vec::new();
            for candidate in [
                details.get("text"),
                details.get("exception").and_then(|e| e.get("description")),
            ] {
                if let Some(part) = candidate.and_then(Value::as_str) {
                    if !part.is_empty() {
                        parts.push(part);
                    }
                }
            }
            PageEvent::Console(CdpConsoleLine {
                kind: "exception".to_owned(),
                text: parts.join(" "),
            })
        }
        _ => PageEvent::Ignore,
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — the 5 new `route_event_*` tests plus the pre-existing `parses_result_error_and_event_messages` all green; the 3 `#[ignore]`d browser tests stay ignored.

Note: until Task 3 wires `route_event` into `run_page`, the compiler may warn that `route_event`/`PageEvent` are unused outside tests — that is acceptable within this plan's lifetime (Task 3 removes it); do NOT add `#[allow(dead_code)]`. If the warning is promoted to an error in this repo, proceed to Task 3 before committing Task 1 and squash the two commits' contents into their own commits as written.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): pure route_event helper for CDP page-run events (session-filtered, exception-aware)"
```

---

### Task 2: `stdout()` node parity and `stderr()`

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (the `impl CdpPageOutcome` block, tests in `mod tests`)

**Interfaces:**
- Consumes: `CdpPageOutcome { console: Vec<CdpConsoleLine>, completed: bool }` (fields are pub; tests construct it literally).
- Produces (Task 3's gated test relies on these):
  - `pub fn stdout(&self) -> String` — `log` + `info` + `debug` lines, emission order, `\n`-terminated.
  - `pub fn stderr(&self) -> String` — `warn` + `error` lines, emission order, `\n`-terminated.

- [ ] **Step 1: Write the failing unit tests**

Add to `mod tests` in `driver.rs`:

```rust
    #[test]
    fn stdout_and_stderr_split_kinds_like_node() {
        let outcome = CdpPageOutcome {
            console: vec![
                CdpConsoleLine { kind: "log".to_owned(), text: "l".to_owned() },
                CdpConsoleLine { kind: "info".to_owned(), text: "i".to_owned() },
                CdpConsoleLine { kind: "warn".to_owned(), text: "w".to_owned() },
                CdpConsoleLine { kind: "debug".to_owned(), text: "d".to_owned() },
                CdpConsoleLine { kind: "error".to_owned(), text: "e".to_owned() },
                CdpConsoleLine { kind: "exception".to_owned(), text: "x".to_owned() },
            ],
            completed: true,
        };
        assert_eq!(outcome.stdout(), "l\ni\nd\n");
        assert_eq!(outcome.stderr(), "w\ne\n");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_cli --test browser_cdp_smoke stdout_and_stderr_split_kinds_like_node`
Expected: FAIL — compile error ``no method named `stderr` `` (and once that exists, `stdout()` returning `"l\n"` would fail the assertion).

- [ ] **Step 3: Implement the kind split**

Replace the entire existing `impl CdpPageOutcome { ... }` block (the one containing the current `stdout`) with:

```rust
impl CdpPageOutcome {
    /// Reproduce node-style stdout: `log`, `info`, and `debug` lines in order.
    pub fn stdout(&self) -> String {
        self.lines_of(&["log", "info", "debug"])
    }

    /// Reproduce node-style stderr: `warn` and `error` lines in order.
    pub fn stderr(&self) -> String {
        self.lines_of(&["warn", "error"])
    }

    fn lines_of(&self, kinds: &[&str]) -> String {
        let mut out = String::new();
        for line in &self.console {
            if kinds.contains(&line.kind.as_str()) {
                out.push_str(&line.text);
                out.push('\n');
            }
        }
        out
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — all non-ignored tests green (route_event tests from Task 1, the new stdout/stderr test, protocol parse test).

- [ ] **Step 5: Format and commit**

```bash
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): node-parity stdout (log+info+debug) and stderr (warn+error) on CdpPageOutcome"
```

---

### Task 3: Pending-event buffer, `run_page` rewiring, gated race regression test

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (`CdpBrowser` struct, `launch`, `call`, `run_page`, one new gated test in `mod tests`)

**Interfaces:**
- Consumes: `route_event` / `PageEvent` from Task 1 (exact signature `route_event(method: &str, params: &Value, event_session: Option<&str>, page_session: &str) -> PageEvent`), `stdout()`/`stderr()` from Task 2, existing `CdpIncoming::{Result, Error, Event}` from `protocol.rs`, `chromium_available`, `CdpBrowser::launch/close`.
- Produces: `CdpBrowser::call` and `CdpBrowser::run_page` keep their public signatures unchanged; behavior change only (no caller updates needed — `browser_cdp_smoke.rs` compiles as-is).

- [ ] **Step 1: Write the gated race-regression test**

Add to `mod tests` in `driver.rs`. A classic (non-module) inline script executes during document parse — the earliest console output a page can produce, and exactly the window in which the old `call("Page.navigate", ...)` discarded events:

```rust
    #[test]
    #[ignore = "launches a real Chromium; run with `-- --ignored`"]
    fn captures_console_logged_synchronously_at_document_start() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let mut browser =
            CdpBrowser::launch("chromium", std::time::Duration::from_secs(20)).expect("launch");
        // Classic script: runs during parse, before Page.navigate's response is
        // necessarily read — the window where the driver used to drop events.
        let html = "<!doctype html><meta charset=utf-8><script>\
console.log('early-log');console.info('early-info');console.debug('early-debug');\
console.warn('early-warn');console.error('early-error');\
globalThis.__kaliHarnessDone && globalThis.__kaliHarnessDone('');\
</script>";
        let url = format!("data:text/html,{}", html);
        let outcome = browser
            .run_page(&url, std::time::Duration::from_secs(30))
            .expect("run page");
        browser.close().expect("close");

        assert!(outcome.completed, "page should have signaled completion");
        assert_eq!(
            outcome.stdout(),
            "early-log\nearly-info\nearly-debug\n",
            "console: {:?}",
            outcome.console
        );
        assert_eq!(
            outcome.stderr(),
            "early-warn\nearly-error\n",
            "console: {:?}",
            outcome.console
        );
    }
```

- [ ] **Step 2: Run the gated lane to observe pre-fix behavior**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: the new test FAILS (missing early lines and/or `completed: false` after the 30s timeout). Because this is a race, it MAY pass on a given run — if it passes, re-run once; either way record the outcome and proceed. (Skip this step's browser runs entirely if Chromium is unavailable; the test self-skips.)

- [ ] **Step 3: Implement the pending-event buffer and rewire `run_page`**

All in `driver.rs`:

3a. Add the import at the top (alongside the existing `use std::io::...`):

```rust
use std::collections::VecDeque;
```

3b. Add the buffer field to `CdpBrowser`:

```rust
/// A launched Chromium plus its blocking CDP connection.
pub struct CdpBrowser {
    child: Child,
    conn: CdpConnection,
    /// Events received while a command call was awaiting its response,
    /// preserved for the next page-run loop instead of being discarded.
    pending_events: VecDeque<CdpIncoming>,
    _user_data_dir: TempDir,
}
```

3c. Initialize it in `launch` (the `Ok(Self { ... })` at the end):

```rust
        Ok(Self {
            child,
            conn,
            pending_events: VecDeque::new(),
            _user_data_dir: user_data_dir,
        })
```

3d. In `call`, replace the response-wait loop so events are buffered, not dropped. The whole method becomes:

```rust
    /// Send a method and read frames until its matching response, returning `result`.
    /// Events that arrive first are buffered for the page-run loop, never dropped.
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
                event @ CdpIncoming::Event { .. } => self.pending_events.push_back(event),
                _ => continue,
            }
        }
    }
```

3e. In `run_page`, replace everything from `let deadline = Instant::now() + timeout;` down to (and including) the `loop { ... }` with:

```rust
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
                self.conn.set_read_timeout(remaining)?;
                self.conn.read()
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
```

Note: `session_id` here is the existing `String` local already bound earlier in `run_page` (from `attachToTarget`); the `session` local (`Option<&str>`) borrows it, so pattern-bind the event's session as `event_session` exactly as shown to avoid shadowing. The old inline `Runtime.consoleAPICalled` parsing and `Runtime.bindingCalled` match arms are deleted — `route_event` replaces them.

- [ ] **Step 4: Run the unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — all non-ignored tests green, and no `dead_code` warning remains (route_event is now used by `run_page`).

- [ ] **Step 5: Run the gated browser lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: PASS — the new `captures_console_logged_synchronously_at_document_start`, the three pre-existing gated driver tests, and `real_chromium_runs_a_browser_bundle_and_captures_console` all green. Run it twice to gain confidence against the race. (If Chromium is unavailable, the tests self-skip; note that in the commit/PR description instead of claiming a green browser lane.)

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p kali_cli --tests -- -D warnings
git add crates/kali_cli/tests/cdp_driver/driver.rs
git commit -m "test(cli): buffer CDP events during call() so early console.log and completion are never dropped"
```

Expected: fmt produces no diff; clippy clean.

---

## Verification (whole plan)

- `cargo test -p kali_cli --test browser_cdp_smoke` — unit lane green.
- `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` (or `mise run browser-smoke`) — gated lane green, twice.
- `git status` clean; three commits, each formatted.
