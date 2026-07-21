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
    Result {
        id: u64,
        result: Value,
    },
    Error {
        id: u64,
        message: String,
    },
    Event {
        method: String,
        params: Value,
        session_id: Option<String>,
    },
}

/// Decode one CDP frame. Pure over the JSON text so it is unit-testable.
pub(crate) fn parse_incoming(text: &str) -> Result<CdpIncoming, CdpError> {
    let value: Value = serde_json::from_str(text).map_err(|e| CdpError::Protocol(e.to_string()))?;
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
            .send(Message::text(message.to_string()))
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
    pub(crate) fn set_read_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<(), CdpError> {
        let stream = match self.socket.get_ref() {
            MaybeTlsStream::Plain(stream) => stream,
            _ => {
                return Err(CdpError::Transport(
                    "unexpected TLS stream for ws://".to_owned(),
                ))
            }
        };
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| CdpError::Transport(e.to_string()))
    }
}

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
            CdpIncoming::Event {
                method,
                session_id,
                params,
            } => {
                assert_eq!(method, "Runtime.consoleAPICalled");
                assert_eq!(session_id.as_deref(), Some("S1"));
                assert_eq!(params["type"], "log");
            }
            other => panic!("expected event, got {other:?}"),
        }
    }
}
