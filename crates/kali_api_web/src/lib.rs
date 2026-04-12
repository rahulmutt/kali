//! Web API compatibility surface for Kali runtime.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::Instant,
};
use url::Url;

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();

/// Initialize the Web API compatibility surface.
pub fn web_api_init() {}

/// Encode text as UTF-8 bytes for the Web baseline text encoder.
pub fn text_encode(input: &str) -> Vec<u8> {
    input.as_bytes().to_vec()
}

/// Decode UTF-8 bytes for the Web baseline text decoder.
pub fn text_decode(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}

/// Clone a support-library value using the host's ordinary `Clone` semantics.
pub fn structured_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

/// Parse a URL string using the shared support library's URL parser.
pub fn parse_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}

/// Resolve a URL against a base URL string.
pub fn resolve_url(base: &str, input: &str) -> Result<Url, url::ParseError> {
    Url::parse(base)?.join(input)
}

/// Return a monotonic millisecond timestamp for `performance.now()`-style calls.
pub fn performance_now() -> f64 {
    TIME_ORIGIN
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}

/// Fill the provided buffer with OS randomness for `crypto.getRandomValues()`.
pub fn fill_random_values(buffer: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(buffer)
}

/// A minimal abort signal used by the Web baseline support library.
#[derive(Clone, Default)]
pub struct AbortSignal {
    aborted: Arc<AtomicBool>,
}

impl AbortSignal {
    pub fn aborted(&self) -> bool {
        self.aborted.load(Ordering::SeqCst)
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
        self.signal.aborted.store(true, Ordering::SeqCst);
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

type EventListener = Box<dyn FnMut(&Event) + Send + 'static>;
type ListenerMap = BTreeMap<String, Vec<EventListener>>;

/// A minimal event target used by the support library.
#[derive(Default, Clone)]
pub struct EventTarget {
    listeners: Arc<Mutex<ListenerMap>>,
}

impl EventTarget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_listener<F>(&self, event_type: impl Into<String>, listener: F)
    where
        F: FnMut(&Event) + Send + 'static,
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

    pub fn dispatch_event(&self, event: &Event) -> usize {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_now_is_monotonic_and_non_negative() {
        let first = performance_now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let second = performance_now();

        assert!(first >= 0.0, "first timestamp: {first}");
        assert!(
            second >= first,
            "timestamps should not go backwards: {first} -> {second}"
        );
    }

    #[test]
    fn random_fill_populates_the_requested_buffer() {
        let mut buffer = [0u8; 16];
        fill_random_values(&mut buffer).expect("random fill");
        assert_eq!(buffer.len(), 16);
    }

    #[test]
    fn text_codec_round_trips_unicode() {
        let input = "héllo 🌍";
        let encoded = text_encode(input);
        assert_eq!(encoded, input.as_bytes());
        let decoded = text_decode(&encoded).expect("valid utf-8");
        assert_eq!(decoded, input);
    }

    #[test]
    fn structured_clone_copies_values() {
        let original = vec![1, 2, 3];
        let cloned = structured_clone(&original);
        assert_eq!(cloned, original);
    }

    #[test]
    fn url_parser_can_parse_and_resolve() {
        let parsed = parse_url("https://example.com/path").expect("url");
        assert_eq!(parsed.as_str(), "https://example.com/path");

        let resolved = resolve_url("https://example.com/base/", "../child").expect("resolved");
        assert_eq!(resolved.as_str(), "https://example.com/child");
    }

    #[test]
    fn abort_controller_flips_the_signal() {
        let controller = AbortController::new();
        let signal = controller.signal();
        assert!(!signal.aborted());
        controller.abort();
        assert!(signal.aborted());
    }

    #[test]
    fn event_target_dispatches_registered_listeners() {
        let target = EventTarget::new();
        let seen = Arc::new(AtomicBool::new(false));
        let seen_clone = Arc::clone(&seen);

        target.add_event_listener("hello", move |event| {
            seen_clone.store(event.event_type() == "hello", Ordering::SeqCst);
        });

        let event = Event::new("hello");
        assert_eq!(target.dispatch_event(&event), 1);
        assert!(seen.load(Ordering::SeqCst));
    }
}
