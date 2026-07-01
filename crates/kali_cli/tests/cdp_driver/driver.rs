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
}
