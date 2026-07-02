//! High-level CDP browser lifecycle: launch, per-page run, close.
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tempfile::TempDir;

use super::protocol::{CdpConnection, CdpError, CdpIncoming, CdpTransport};

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
        .arg(format!(
            "--user-data-dir={}",
            user_data_dir.path().display()
        ))
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

/// The completion binding a harness page calls to signal it finished.
pub(crate) const CDP_DONE_BINDING: &str = "__kaliHarnessDone";

/// One captured console call from the page.
#[derive(Clone, Debug, PartialEq)]
pub struct CdpConsoleLine {
    /// Console method: "log", "error", "warn", "info", "debug", "exception" (uncaught errors), or any other CDP console type passed through.
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
            let mut kind = params["type"].as_str().unwrap_or("log").to_owned();
            // Normalize Chrome CDP's "warning" to "warn" for node-style stderr compatibility.
            if kind == "warning" {
                kind = "warn".to_owned();
            }
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

/// Kills the browser even when a test panics or errors before `close()` runs.
impl Drop for CdpBrowser {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "launches a real Chromium; run with `-- --ignored`"]
    fn spawns_chromium_and_reports_ws_endpoint() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let (mut child, ws_url, _dir) =
            spawn_chromium("chromium", std::time::Duration::from_secs(20)).expect("spawn chromium");
        assert!(ws_url.starts_with("ws://"), "unexpected ws url: {ws_url}");
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    #[ignore = "launches a real Chromium; run with `-- --ignored`"]
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
            result["product"]
                .as_str()
                .unwrap_or_default()
                .contains("Chrom"),
            "unexpected product: {result}"
        );
        browser.close().expect("close");
    }

    #[test]
    #[ignore = "launches a real Chromium; run with `-- --ignored`"]
    fn runs_page_captures_console_and_completes() {
        if !chromium_available("chromium") {
            eprintln!("skipping: chromium not available");
            return;
        }
        let mut browser =
            CdpBrowser::launch("chromium", std::time::Duration::from_secs(20)).expect("launch");
        let html = "<!doctype html><meta charset=utf-8><script type=\"module\">\
console.log('3'); console.log('3'); globalThis.__kaliHarnessDone && globalThis.__kaliHarnessDone('');\
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
        // Chromium reports console.warn as raw type "warning"; the driver
        // normalizes it to "warn" so stderr()'s node-style kind split matches.
        let warn_params = serde_json::json!({
            "type": "warning",
            "args": [{ "value": "w" }]
        });
        assert_eq!(
            route_event("Runtime.consoleAPICalled", &warn_params, Some("S1"), "S1"),
            PageEvent::Console(CdpConsoleLine {
                kind: "warn".to_owned(),
                text: "w".to_owned(),
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

    #[test]
    fn stdout_and_stderr_split_kinds_like_node() {
        let outcome = CdpPageOutcome {
            console: vec![
                CdpConsoleLine {
                    kind: "log".to_owned(),
                    text: "l".to_owned(),
                },
                CdpConsoleLine {
                    kind: "info".to_owned(),
                    text: "i".to_owned(),
                },
                CdpConsoleLine {
                    kind: "warn".to_owned(),
                    text: "w".to_owned(),
                },
                CdpConsoleLine {
                    kind: "debug".to_owned(),
                    text: "d".to_owned(),
                },
                CdpConsoleLine {
                    kind: "error".to_owned(),
                    text: "e".to_owned(),
                },
                CdpConsoleLine {
                    kind: "exception".to_owned(),
                    text: "x".to_owned(),
                },
            ],
            completed: true,
        };
        assert_eq!(outcome.stdout(), "l\ni\nd\n");
        assert_eq!(outcome.stderr(), "w\ne\n");
    }

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
            result_frame(3, serde_json::json!({})),                   // Runtime.enable
            result_frame(4, serde_json::json!({})),                   // Page.enable
            result_frame(5, serde_json::json!({})),                   // Runtime.addBinding
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
}
