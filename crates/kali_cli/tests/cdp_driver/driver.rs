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
        Ok(Self {
            child,
            conn,
            _user_data_dir: user_data_dir,
        })
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

impl CdpBrowser {
    /// Open a fresh target, navigate to `url`, capture console output, and return
    /// when the page calls the completion binding or `timeout` elapses.
    pub fn run_page(&mut self, url: &str, timeout: Duration) -> Result<CdpPageOutcome, CdpError> {
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
            spawn_chromium("chromium", std::time::Duration::from_secs(20)).expect("spawn chromium");
        assert!(ws_url.starts_with("ws://"), "unexpected ws url: {ws_url}");
        let _ = child.kill();
        let _ = child.wait();
    }

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
            result["product"]
                .as_str()
                .unwrap_or_default()
                .contains("Chrom"),
            "unexpected product: {result}"
        );
        browser.close().expect("close");
    }

    #[test]
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
}
