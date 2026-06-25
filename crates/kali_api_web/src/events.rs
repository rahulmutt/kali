//! Event system types: AbortSignal, AbortController, Event, CustomEvent, EventTarget.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

/// A minimal abort signal used by the Web baseline support library.
#[derive(Clone, Default)]
pub struct AbortSignal {
    aborted: Arc<AtomicBool>,
    event_target: EventTarget,
}

impl AbortSignal {
    pub fn aborted(&self) -> bool {
        self.aborted.load(Ordering::SeqCst)
    }

    pub fn add_event_listener<F>(&self, event_type: impl Into<String>, listener: F) -> usize
    where
        F: FnMut(&Event) + Send + 'static,
    {
        self.event_target.add_event_listener(event_type, listener)
    }

    pub fn remove_event_listener(&self, event_type: &str, listener_id: usize) -> bool {
        self.event_target
            .remove_event_listener(event_type, listener_id)
    }

    pub fn dispatch_event(&self, event: &Event) -> usize {
        self.event_target.dispatch_event(event)
    }
}

/// A minimal abort controller used by the Web baseline support library.
#[derive(Clone, Default)]
pub struct AbortController {
    signal: AbortSignal,
}

impl AbortController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn signal(&self) -> AbortSignal {
        self.signal.clone()
    }

    pub fn abort(&self) {
        if self.signal.aborted.swap(true, Ordering::SeqCst) {
            return;
        }

        let event = Event::new("abort");
        let _ = self.signal.dispatch_event(&event);
    }
}

/// A lightweight event object used by the support library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    default_prevented: bool,
}

impl Event {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            bubbles: false,
            cancelable: false,
            default_prevented: false,
        }
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn bubbles(&self) -> bool {
        self.bubbles
    }

    pub fn cancelable(&self) -> bool {
        self.cancelable
    }

    pub fn default_prevented(&self) -> bool {
        self.default_prevented
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }
}

/// A lightweight custom event object used by the support library.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomEvent {
    event: Event,
    detail: Value,
}

impl CustomEvent {
    pub fn new(event_type: impl Into<String>, detail: Value) -> Self {
        Self {
            event: Event::new(event_type),
            detail,
        }
    }

    pub fn event(&self) -> &Event {
        &self.event
    }

    pub fn detail(&self) -> &Value {
        &self.detail
    }
}

type EventListenerId = usize;
type EventListener = Box<dyn FnMut(&Event) + Send + 'static>;
type SharedEventListener = Arc<Mutex<EventListener>>;

struct RegisteredEventListener {
    id: EventListenerId,
    active: Arc<AtomicBool>,
    listener: SharedEventListener,
}

type ListenerMap = BTreeMap<String, Vec<RegisteredEventListener>>;

/// A minimal event target used by the support library.
#[derive(Default, Clone)]
pub struct EventTarget {
    listeners: Arc<Mutex<ListenerMap>>,
    next_listener_id: Arc<AtomicUsize>,
}

impl EventTarget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_listener<F>(&self, event_type: impl Into<String>, listener: F) -> usize
    where
        F: FnMut(&Event) + Send + 'static,
    {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        let id = self.next_listener_id.fetch_add(1, Ordering::SeqCst);
        listeners
            .entry(event_type.into())
            .or_default()
            .push(RegisteredEventListener {
                id,
                active: Arc::new(AtomicBool::new(true)),
                listener: Arc::new(Mutex::new(Box::new(listener))),
            });
        id
    }

    pub fn remove_event_listener(&self, event_type: &str, listener_id: usize) -> bool {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        let Some(event_listeners) = listeners.get_mut(event_type) else {
            return false;
        };
        let mut removed = false;
        event_listeners.retain(|listener| {
            if listener.id == listener_id {
                listener.active.store(false, Ordering::SeqCst);
                removed = true;
                return false;
            }

            listener.active.load(Ordering::SeqCst)
        });
        removed
    }

    pub fn dispatch_event(&self, event: &Event) -> usize {
        let snapshot = {
            let listeners = self
                .listeners
                .lock()
                .expect("event listener mutex poisoned");
            listeners
                .get(event.event_type())
                .map(|event_listeners| {
                    event_listeners
                        .iter()
                        .map(|listener| {
                            (Arc::clone(&listener.active), Arc::clone(&listener.listener))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let mut invoked = 0;
        for (active, listener) in snapshot {
            if !active.load(Ordering::SeqCst) {
                continue;
            }

            invoked += 1;
            let mut callback = listener
                .lock()
                .expect("event listener callback mutex poisoned");
            (callback)(event);
        }

        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        if let Some(event_listeners) = listeners.get_mut(event.event_type()) {
            event_listeners.retain(|listener| listener.active.load(Ordering::SeqCst));
        }

        invoked
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod events_tests;
