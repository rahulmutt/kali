//! Node-style event emitter and event types.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// A minimal Node-style event object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeEvent {
    event_type: String,
    detail: Option<String>,
}

impl NodeEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            detail: None,
        }
    }

    pub fn with_detail(event_type: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            detail: Some(detail.into()),
        }
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

type Listener = Box<dyn FnMut(&NodeEvent) + Send + 'static>;
type ListenerMap = BTreeMap<String, Vec<Listener>>;

/// Minimal Node-style `EventEmitter`.
#[derive(Clone, Default)]
pub struct EventEmitter {
    listeners: Arc<Mutex<ListenerMap>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on<F>(&self, event_type: impl Into<String>, listener: F)
    where
        F: FnMut(&NodeEvent) + Send + 'static,
    {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        listeners
            .entry(event_type.into())
            .or_default()
            .push(Box::new(listener));
    }

    pub fn emit(&self, event: &NodeEvent) -> usize {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        let Some(event_listeners) = listeners.get_mut(event.event_type()) else {
            return 0;
        };

        for listener in event_listeners.iter_mut() {
            listener(event);
        }

        event_listeners.len()
    }

    pub fn listener_count(&self, event_type: &str) -> usize {
        self.listeners
            .lock()
            .expect("event listener mutex poisoned")
            .get(event_type)
            .map(|listeners| listeners.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod events_tests;
