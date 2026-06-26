//! Worker and BroadcastChannel stub implementations.

use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use crate::SharedArrayBuffer;
use ::url::{ParseError as UrlParseError, Url}; // external `url` crate; `url` name is shadowed by our local module

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PostedItem {
    Message(Value),
    SharedBuffer(SharedArrayBuffer),
}

#[derive(Clone, Debug, Default)]
struct DeterministicPostQueue {
    items: Arc<Mutex<Vec<PostedItem>>>,
}

impl DeterministicPostQueue {
    fn push_message(&self, message: Value) {
        self.items
            .lock()
            .expect("post queue mutex poisoned")
            .push(PostedItem::Message(message));
    }

    fn push_shared_buffer(&self, buffer: SharedArrayBuffer) {
        self.items
            .lock()
            .expect("post queue mutex poisoned")
            .push(PostedItem::SharedBuffer(buffer));
    }

    fn snapshot(&self) -> Vec<PostedItem> {
        self.items
            .lock()
            .expect("post queue mutex poisoned")
            .clone()
    }

    fn messages(&self) -> Vec<Value> {
        self.snapshot()
            .into_iter()
            .filter_map(|item| match item {
                PostedItem::Message(message) => Some(message),
                PostedItem::SharedBuffer(_) => None,
            })
            .collect()
    }

    fn shared_buffers(&self) -> Vec<SharedArrayBuffer> {
        self.snapshot()
            .into_iter()
            .filter_map(|item| match item {
                PostedItem::Message(_) => None,
                PostedItem::SharedBuffer(buffer) => Some(buffer),
            })
            .collect()
    }
}

/// A deterministic worker stub used by the browser baseline.
#[derive(Clone, Debug)]
pub struct Worker {
    script_url: Url,
    posted_items: DeterministicPostQueue,
    terminated: Arc<AtomicBool>,
}

impl Worker {
    /// Create a new worker stub from a parsed script URL.
    pub fn new(url: impl AsRef<str>) -> Result<Self, UrlParseError> {
        Ok(Self {
            script_url: Url::parse(url.as_ref().trim())?,
            posted_items: DeterministicPostQueue::default(),
            terminated: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Return the worker script URL.
    pub fn script_url(&self) -> &Url {
        &self.script_url
    }

    /// Record a posted message in the deterministic stub buffer.
    pub fn post_message(&self, message: Value) {
        if self.terminated.load(Ordering::SeqCst) {
            return;
        }
        self.posted_items.push_message(message);
    }

    /// Record a shared buffer in the deterministic worker-message queue.
    pub fn post_shared_buffer(&self, buffer: SharedArrayBuffer) {
        if self.terminated.load(Ordering::SeqCst) {
            return;
        }
        self.posted_items.push_shared_buffer(buffer);
    }

    /// Return the buffered messages.
    pub fn posted_messages(&self) -> Vec<Value> {
        self.posted_items.messages()
    }

    /// Return the buffered shared buffers.
    pub fn posted_shared_buffers(&self) -> Vec<SharedArrayBuffer> {
        self.posted_items.shared_buffers()
    }

    pub(crate) fn posted_items(&self) -> Vec<PostedItem> {
        self.posted_items.snapshot()
    }

    /// Transition the stub worker into the terminated state.
    pub fn terminate(&self) {
        self.terminated.store(true, Ordering::SeqCst);
    }

    /// Return whether the stub worker has been terminated.
    pub fn is_terminated(&self) -> bool {
        self.terminated.load(Ordering::SeqCst)
    }
}

/// A deterministic broadcast channel stub used by the browser baseline.
#[derive(Clone, Debug)]
pub struct BroadcastChannel {
    name: String,
    posted_items: DeterministicPostQueue,
    closed: Arc<AtomicBool>,
}

impl BroadcastChannel {
    /// Create a new broadcast channel stub with a stable name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            posted_items: DeterministicPostQueue::default(),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Return the channel name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a posted message in the deterministic stub buffer.
    pub fn post_message(&self, message: Value) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        self.posted_items.push_message(message);
    }

    /// Record a shared buffer in the deterministic broadcast queue.
    pub fn post_shared_buffer(&self, buffer: SharedArrayBuffer) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        self.posted_items.push_shared_buffer(buffer);
    }

    /// Return the buffered messages.
    pub fn posted_messages(&self) -> Vec<Value> {
        self.posted_items.messages()
    }

    /// Return the buffered shared buffers.
    pub fn posted_shared_buffers(&self) -> Vec<SharedArrayBuffer> {
        self.posted_items.shared_buffers()
    }

    #[cfg(test)]
    fn posted_items(&self) -> Vec<PostedItem> {
        self.posted_items.snapshot()
    }

    /// Transition the stub channel into the closed state.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    /// Return whether the stub channel has been closed.
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;
