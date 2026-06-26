//! WebSocket family — stub WebSocket implementation.

use ::url::{ParseError as UrlParseError, Url}; // external `url` crate; `url` name is shadowed by our local module
use std::sync::{Arc, Mutex};

/// A stub WebSocket implementation that keeps the browser baseline deterministic.
#[derive(Clone, Debug)]
pub struct WebSocket {
    url: Url,
    ready_state: WebSocketReadyState,
    sent_text_messages: Arc<Mutex<Vec<String>>>,
    sent_binary_messages: Arc<Mutex<Vec<Vec<u8>>>>,
}

/// Ready-state values for the stub WebSocket baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketReadyState {
    Connecting,
    Open,
    Closing,
    Closed,
}

impl WebSocket {
    /// Create a stub WebSocket bound to a parsed URL.
    pub fn new(url: impl AsRef<str>) -> Result<Self, UrlParseError> {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
            ready_state: WebSocketReadyState::Open,
            sent_text_messages: Arc::new(Mutex::new(Vec::new())),
            sent_binary_messages: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Return the socket URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the current ready state.
    pub fn ready_state(&self) -> WebSocketReadyState {
        self.ready_state
    }

    /// Record a text payload in the deterministic stub buffer.
    pub fn send_text(&self, payload: impl Into<String>) {
        self.sent_text_messages
            .lock()
            .expect("websocket mutex poisoned")
            .push(payload.into());
    }

    /// Record a binary payload in the deterministic stub buffer.
    pub fn send_bytes(&self, payload: impl AsRef<[u8]>) {
        self.sent_binary_messages
            .lock()
            .expect("websocket mutex poisoned")
            .push(payload.as_ref().to_vec());
    }

    /// Return the buffered text payloads.
    pub fn sent_text_messages(&self) -> Vec<String> {
        self.sent_text_messages
            .lock()
            .expect("websocket mutex poisoned")
            .clone()
    }

    /// Return the buffered binary payloads.
    pub fn sent_binary_messages(&self) -> Vec<Vec<u8>> {
        self.sent_binary_messages
            .lock()
            .expect("websocket mutex poisoned")
            .clone()
    }

    /// Transition the socket to the closed state.
    pub fn close(&mut self) {
        self.ready_state = WebSocketReadyState::Closed;
    }
}

#[cfg(test)]
#[path = "websocket_tests.rs"]
mod websocket_tests;
