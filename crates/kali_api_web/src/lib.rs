//! Web API compatibility surface for Kali runtime.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use kali_common::bytewise_shared_memory_is_lock_free;
use url::{form_urlencoded, Url};

mod base64;
pub use base64::*;

mod crypto;
pub use crypto::*;

mod file;
pub use file::*;

mod storage;
pub use storage::*;

mod streams;
pub use streams::*;

mod util;
pub use util::*;

static NAVIGATOR: OnceLock<Navigator> = OnceLock::new();

/// Errors returned when mutating a deterministic URL baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlMutationError {
    InvalidProtocol,
    InvalidHost,
    InvalidPort,
}

impl fmt::Display for UrlMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidProtocol => "invalid URL protocol",
            Self::InvalidHost => "invalid URL host",
            Self::InvalidPort => "invalid URL port",
        };
        f.write_str(message)
    }
}

impl std::error::Error for UrlMutationError {}

/// Return the shared in-memory `navigator` baseline.
pub fn navigator() -> Navigator {
    NAVIGATOR.get_or_init(Navigator::default).clone()
}

/// Parse a URL string using the shared support library's URL parser.
pub fn parse_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}

/// Resolve a URL against a base URL string.
pub fn resolve_url(base: &str, input: &str) -> Result<Url, url::ParseError> {
    Url::parse(base)?.join(input)
}

/// A deterministic in-memory Web `navigator` baseline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigator {
    user_agent: String,
    language: String,
    languages: Vec<String>,
    online: bool,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            user_agent: "Kali/1.0 (Web)".to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string()],
            online: true,
        }
    }
}

impl Navigator {
    /// Return the user-agent string.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Return the preferred primary language.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Return the preferred language list.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Return whether the browser baseline considers the host online.
    pub fn on_line(&self) -> bool {
        self.online
    }

    /// Return a deterministic snapshot of the browser navigator baseline.
    pub fn snapshot(&self) -> BTreeMap<String, Value> {
        self.snapshot_object_value()
    }

    /// Alias for the deterministic navigator snapshot helper.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, Value> {
        BTreeMap::from([
            (
                "userAgent".to_string(),
                Value::String(self.user_agent.clone()),
            ),
            ("language".to_string(), Value::String(self.language.clone())),
            (
                "languages".to_string(),
                Value::Array(self.languages.iter().cloned().map(Value::String).collect()),
            ),
            ("online".to_string(), Value::Bool(self.online)),
        ])
    }

    /// Return the navigator snapshot as a JSON object value.
    pub fn snapshot_value(&self) -> Value {
        Value::Object(self.snapshot().into_iter().collect())
    }

    /// Alias for the JSON-ready navigator snapshot helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }
}

/// A deterministic in-memory Web `URL` baseline.
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct URL {
    url: Url,
}

impl PartialEq for URL {
    fn eq(&self, other: &Self) -> bool {
        self.url.as_str() == other.url.as_str()
    }
}

impl Eq for URL {}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl URL {
    /// Create a new URL from an absolute or base-resolved URL string.
    pub fn new(input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::parse(input)
    }

    /// Parse a URL string into the deterministic baseline wrapper.
    pub fn parse(input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Url::parse(input.as_ref()).map(Self::from_url)
    }

    /// Resolve a relative URL against a base URL string.
    pub fn resolve(base: impl AsRef<str>, input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Url::parse(base.as_ref())?
            .join(input.as_ref())
            .map(Self::from_url)
    }

    /// Wrap an existing parsed URL value.
    pub fn from_url(url: Url) -> Self {
        Self { url }
    }

    /// Unwrap the inner parsed URL value.
    pub fn into_inner(self) -> Url {
        self.url
    }

    /// Return the underlying parsed URL.
    pub fn as_url(&self) -> &Url {
        &self.url
    }

    /// Return the serialized URL string.
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    /// Return the canonical URL href string.
    pub fn href(&self) -> &str {
        self.as_str()
    }

    /// Return the current protocol with the trailing `:` suffix.
    pub fn protocol(&self) -> String {
        format!("{}:", self.url.scheme())
    }

    /// Update the protocol/scheme component.
    pub fn set_protocol(&mut self, protocol: impl AsRef<str>) -> Result<(), UrlMutationError> {
        let protocol = protocol.as_ref().trim_end_matches(':');
        self.url
            .set_scheme(protocol)
            .map_err(|_| UrlMutationError::InvalidProtocol)
    }

    /// Return the current pathname component.
    pub fn pathname(&self) -> &str {
        self.url.path()
    }

    /// Update the pathname component.
    pub fn set_pathname(&mut self, pathname: impl AsRef<str>) {
        self.url.set_path(pathname.as_ref());
    }

    /// Return the current query string with the leading `?`, if present.
    pub fn search(&self) -> String {
        self.url
            .query()
            .map(|query| format!("?{}", query))
            .unwrap_or_default()
    }

    /// Update the query string.
    pub fn set_search(&mut self, search: impl AsRef<str>) {
        let search = search.as_ref().strip_prefix('?').unwrap_or(search.as_ref());
        self.url.set_query((!search.is_empty()).then_some(search));
    }

    /// Return the current fragment with the leading `#`, if present.
    pub fn hash(&self) -> String {
        self.url
            .fragment()
            .map(|fragment| format!("#{}", fragment))
            .unwrap_or_default()
    }

    /// Update the fragment component.
    pub fn set_hash(&mut self, hash: impl AsRef<str>) {
        let hash = hash.as_ref().strip_prefix('#').unwrap_or(hash.as_ref());
        self.url.set_fragment((!hash.is_empty()).then_some(hash));
    }

    /// Return the current host component, if any.
    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Update the host component.
    pub fn set_host(&mut self, host: impl AsRef<str>) -> Result<(), UrlMutationError> {
        self.url
            .set_host(Some(host.as_ref()))
            .map_err(|_| UrlMutationError::InvalidHost)
    }

    /// Return the current port component, if any.
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    /// Update the port component.
    pub fn set_port(&mut self, port: Option<u16>) -> Result<(), UrlMutationError> {
        self.url
            .set_port(port)
            .map_err(|_| UrlMutationError::InvalidPort)
    }
}

fn normalize_header_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// A deterministic in-memory Web `Headers` baseline.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    entries: Arc<Mutex<Vec<(String, String)>>>,
}

impl Headers {
    /// Create an empty header bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a header while preserving insertion order.
    pub fn append(&self, name: impl Into<String>, value: impl Into<String>) {
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .push((normalize_header_name(&name.into()), value.into()));
    }

    /// Replace all matching headers with a single value.
    pub fn set(&self, name: impl Into<String>, value: impl Into<String>) {
        let name = normalize_header_name(&name.into());
        self.delete(&name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .push((name, value.into()));
    }

    /// Return whether a matching header exists.
    pub fn has(&self, name: &str) -> bool {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .iter()
            .any(|(entry_name, _)| entry_name == &name)
    }

    /// Return the first matching header value, if present.
    pub fn get(&self, name: &str) -> Option<String> {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .iter()
            .find(|(entry_name, _)| entry_name == &name)
            .map(|(_, value)| value.clone())
    }

    /// Remove all matching headers.
    pub fn delete(&self, name: &str) {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .retain(|(entry_name, _)| entry_name != &name);
    }

    /// Return a deterministic snapshot of the current entries.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.entries.lock().expect("headers mutex poisoned").clone()
    }
}

/// A deterministic in-memory Web `Request` baseline.
#[derive(Clone, Debug)]
pub struct Request {
    url: Url,
    method: String,
    headers: Headers,
    body: Arc<[u8]>,
}

impl Request {
    /// Create a GET request with no body or headers.
    pub fn new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::with_parts(url, "GET", Headers::new(), [])
    }

    /// Create a request with explicit method, headers, and body.
    pub fn with_parts(
        url: impl AsRef<str>,
        method: impl Into<String>,
        headers: Headers,
        body: impl AsRef<[u8]>,
    ) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
            method: method.into().to_ascii_uppercase(),
            headers,
            body: Arc::from(body.as_ref().to_vec()),
        })
    }

    /// Return the request URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the HTTP method.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Return the request headers.
    pub fn headers(&self) -> Headers {
        self.headers.clone()
    }

    /// Return the request body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Decode the request body as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }
}

/// A deterministic in-memory Web `Response` baseline.
#[derive(Clone, Debug)]
pub struct Response {
    url: Url,
    status: u16,
    status_text: String,
    headers: Headers,
    body: Arc<[u8]>,
}

impl Response {
    /// Create a basic 200 OK response with no associated URL.
    pub fn new(body: impl AsRef<[u8]>) -> Result<Self, url::ParseError> {
        Self::with_parts("https://example.invalid/", 200, "OK", Headers::new(), body)
    }

    /// Create a response with explicit status, headers, and body.
    pub fn with_parts(
        url: impl AsRef<str>,
        status: u16,
        status_text: impl Into<String>,
        headers: Headers,
        body: impl AsRef<[u8]>,
    ) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
            status,
            status_text: status_text.into(),
            headers,
            body: Arc::from(body.as_ref().to_vec()),
        })
    }

    /// Return the response URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the HTTP status.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Return the HTTP status text.
    pub fn status_text(&self) -> &str {
        &self.status_text
    }

    /// Return whether the status is in the 2xx range.
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Return the response headers.
    pub fn headers(&self) -> Headers {
        self.headers.clone()
    }

    /// Return the response body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Decode the response body as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }

    /// Create a response that deterministically echoes the request payload.
    pub fn from_request(request: &Request) -> Self {
        Self {
            url: request.url.clone(),
            status: 200,
            status_text: "OK".to_string(),
            headers: request.headers.clone(),
            body: Arc::from(request.body.as_ref().to_vec()),
        }
    }
}

/// Return a deterministic fetch result for the provided request.
pub fn fetch(request: &Request) -> Response {
    Response::from_request(request)
}

/// A deterministic in-memory Web `URLSearchParams` baseline.
#[derive(Clone, Debug, Default)]
pub struct URLSearchParams {
    entries: Arc<Mutex<Vec<(String, String)>>>,
}

impl URLSearchParams {
    /// Create an empty parameter bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a parameter bag from a query string.
    pub fn from_query(query: impl AsRef<str>) -> Self {
        let params = Self::new();
        for (name, value) in form_urlencoded::parse(query.as_ref().as_bytes()) {
            params.append(name.into_owned(), value.into_owned());
        }
        params
    }

    /// Append a parameter while preserving insertion order.
    pub fn append(&self, name: impl Into<String>, value: impl Into<String>) {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .push((name.into(), value.into()));
    }

    /// Replace all matching parameters with a single value.
    pub fn set(&self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        self.delete(&name);
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .push((name, value.into()));
    }

    /// Return whether a matching parameter exists.
    pub fn has(&self, name: &str) -> bool {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .any(|(entry_name, _)| entry_name == name)
    }

    /// Return the first matching value, if present.
    pub fn get(&self, name: &str) -> Option<String> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .find(|(entry_name, _)| entry_name == name)
            .map(|(_, value)| value.clone())
    }

    /// Return all matching values in insertion order.
    pub fn get_all(&self, name: &str) -> Vec<String> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .filter(|(entry_name, _)| entry_name == name)
            .map(|(_, value)| value.clone())
            .collect()
    }

    /// Remove all matching parameters.
    pub fn delete(&self, name: &str) {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .retain(|(entry_name, _)| entry_name != name);
    }

    /// Return a deterministic snapshot of the current entries.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .clone()
    }

    fn serialize(&self) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (name, value) in self
            .entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
        {
            serializer.append_pair(name, value);
        }
        serializer.finish()
    }
}

impl fmt::Display for URLSearchParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.serialize())
    }
}


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
    pub fn new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
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

#[derive(Clone, Debug, PartialEq)]
enum PostedItem {
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
    pub fn new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
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

    fn posted_items(&self) -> Vec<PostedItem> {
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

/// A deterministic shared-memory baseline used by the browser/runtime compatibility layer.
#[derive(Clone, Default)]
pub struct SharedArrayBuffer {
    bytes: Arc<Vec<AtomicU8>>,
}

impl SharedArrayBuffer {
    /// Create a zero-initialized shared buffer with a fixed byte length.
    pub fn new(byte_length: usize) -> Self {
        Self::from_bytes(vec![0u8; byte_length])
    }

    /// Create a shared buffer from deterministic initial bytes.
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self {
            bytes: Arc::new(bytes.as_ref().iter().copied().map(AtomicU8::new).collect()),
        }
    }

    /// Return the buffer length in bytes.
    pub fn byte_length(&self) -> usize {
        self.bytes.len()
    }

    /// Return whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Read a byte at the provided offset.
    pub fn load(&self, index: usize) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.load(Ordering::SeqCst))
    }

    /// Overwrite a byte at the provided offset, returning the previous value.
    pub fn store(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.swap(value, Ordering::SeqCst))
    }

    /// Add to a byte at the provided offset with wrapping arithmetic.
    pub fn add(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.fetch_add(value, Ordering::SeqCst))
    }

    /// Bitwise-and a byte at the provided offset with wrapping semantics.
    pub fn and(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.fetch_and(value, Ordering::SeqCst))
    }

    /// Bitwise-or a byte at the provided offset with wrapping semantics.
    pub fn or(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.fetch_or(value, Ordering::SeqCst))
    }

    /// Bitwise-xor a byte at the provided offset with wrapping semantics.
    pub fn xor(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.fetch_xor(value, Ordering::SeqCst))
    }

    /// Subtract from a byte at the provided offset with wrapping arithmetic.
    pub fn sub(&self, index: usize, value: u8) -> Option<u8> {
        self.bytes
            .get(index)
            .map(|cell| cell.fetch_sub(value, Ordering::SeqCst))
    }

    /// Compare-and-exchange a byte at the provided offset.
    pub fn compare_exchange(&self, index: usize, current: u8, new: u8) -> Option<Result<u8, u8>> {
        self.bytes
            .get(index)
            .map(|cell| cell.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst))
    }

    /// Return a deterministic snapshot of the current bytes.
    pub fn snapshot(&self) -> Vec<u8> {
        self.bytes
            .iter()
            .map(|cell| cell.load(Ordering::SeqCst))
            .collect()
    }
}

impl PartialEq for SharedArrayBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}

impl Eq for SharedArrayBuffer {}

impl std::fmt::Debug for SharedArrayBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedArrayBuffer")
            .field("byte_length", &self.byte_length())
            .field("bytes", &self.snapshot())
            .finish()
    }
}

/// Deterministic atomics helpers for the shared-memory baseline.
#[derive(Clone, Copy, Debug, Default)]
pub struct Atomics;

impl Atomics {
    /// Report whether the bytewise shared-memory helpers are lock-free on this target.
    pub fn is_lock_free() -> bool {
        bytewise_shared_memory_is_lock_free()
    }

    /// Load a byte from the provided shared buffer.
    pub fn load(buffer: &SharedArrayBuffer, index: usize) -> Option<u8> {
        buffer.load(index)
    }

    /// Store a byte into the provided shared buffer.
    pub fn store(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.store(index, value)
    }

    /// Exchange a byte in the provided shared buffer.
    pub fn exchange(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer
            .bytes
            .get(index)
            .map(|cell| cell.swap(value, Ordering::SeqCst))
    }

    /// Add to a byte in the provided shared buffer.
    pub fn add(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.add(index, value)
    }

    /// Bitwise-and a byte in the provided shared buffer.
    pub fn and(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.and(index, value)
    }

    /// Bitwise-or a byte in the provided shared buffer.
    pub fn or(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.or(index, value)
    }

    /// Bitwise-xor a byte in the provided shared buffer.
    pub fn xor(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.xor(index, value)
    }

    /// Subtract from a byte in the provided shared buffer.
    pub fn sub(buffer: &SharedArrayBuffer, index: usize, value: u8) -> Option<u8> {
        buffer.sub(index, value)
    }

    /// Compare-and-exchange a byte in the provided shared buffer.
    pub fn compare_exchange(
        buffer: &SharedArrayBuffer,
        index: usize,
        current: u8,
        new: u8,
    ) -> Option<Result<u8, u8>> {
        buffer.compare_exchange(index, current, new)
    }

    /// Return a deterministic snapshot of the shared buffer.
    pub fn snapshot(buffer: &SharedArrayBuffer) -> Vec<u8> {
        buffer.snapshot()
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

/// A deterministic runtime-topology model that assigns one worker/runtime instance per spawned
/// thread and produces a stable shutdown/leak report.
#[derive(Clone, Debug, Default)]
pub struct ThreadRuntimeTopology {
    next_instance_id: usize,
    instances: BTreeMap<usize, Worker>,
}

/// A snapshot of one runtime instance at shutdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadRuntimeInstanceSnapshot {
    /// Stable runtime-instance identifier.
    pub instance_id: usize,
    /// Script URL for the worker/runtime instance.
    pub script_url: String,
    /// Buffered messages observed for this instance.
    pub posted_messages: Vec<Value>,
    /// Buffered shared-buffer snapshots observed for this instance.
    pub posted_shared_buffers: Vec<Vec<u8>>,
    /// Whether the instance had already been terminated before shutdown.
    pub was_terminated: bool,
}

impl ThreadRuntimeInstanceSnapshot {
    /// Return the instance snapshot as a JSON-ready value.
    pub fn snapshot_value(&self) -> Value {
        Value::Object(
            [
                (
                    "instanceId".to_string(),
                    Value::from(self.instance_id as u64),
                ),
                (
                    "scriptUrl".to_string(),
                    Value::String(self.script_url.clone()),
                ),
                (
                    "postedMessages".to_string(),
                    Value::Array(self.posted_messages.clone()),
                ),
                (
                    "postedSharedBuffers".to_string(),
                    Value::Array(
                        self.posted_shared_buffers
                            .iter()
                            .map(|buffer| {
                                Value::Array(buffer.iter().map(|byte| Value::from(*byte)).collect())
                            })
                            .collect(),
                    ),
                ),
                (
                    "wasTerminated".to_string(),
                    Value::Bool(self.was_terminated),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }

    /// Alias for the JSON-ready instance snapshot helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready instance snapshot helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready instance snapshot helper with an explicit thread-topology name.
    pub fn thread_topology_snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the threaded-topology snapshot helper.
    pub fn thread_topology_snapshot(&self) -> Self {
        self.clone()
    }

    /// Alias for the threaded-topology snapshot helper.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

/// Deterministic shutdown/leak accounting for the runtime-topology model.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThreadRuntimeShutdownReport {
    /// Number of runtime instances created by the topology.
    pub total_instances: usize,
    /// Number of instances that were already terminated before shutdown.
    pub terminated_instances: usize,
    /// Instances that were still live when shutdown began.
    pub live_instances: Vec<ThreadRuntimeInstanceSnapshot>,
}

impl ThreadRuntimeShutdownReport {
    /// Return the shutdown/leak report as a JSON-ready value.
    pub fn snapshot_value(&self) -> Value {
        Value::Object(
            [
                (
                    "totalInstances".to_string(),
                    Value::from(self.total_instances as u64),
                ),
                (
                    "terminatedInstances".to_string(),
                    Value::from(self.terminated_instances as u64),
                ),
                (
                    "liveInstances".to_string(),
                    Value::Array(
                        self.live_instances
                            .iter()
                            .map(ThreadRuntimeInstanceSnapshot::snapshot_value)
                            .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }

    /// Alias for the JSON-ready shutdown/leak report helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready shutdown/leak report helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready shutdown/leak report helper with an explicit thread-topology name.
    pub fn thread_topology_snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the threaded-topology snapshot helper.
    pub fn thread_topology_snapshot(&self) -> Self {
        self.clone()
    }

    /// Alias for the threaded-topology snapshot helper.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

impl ThreadRuntimeTopology {
    /// Create an empty runtime-topology model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new worker/runtime instance with a deterministic identifier.
    pub fn spawn_worker(&mut self, url: impl AsRef<str>) -> Result<usize, url::ParseError> {
        let instance_id = self.next_instance_id;
        self.next_instance_id = self.next_instance_id.saturating_add(1);
        self.instances.insert(instance_id, Worker::new(url)?);
        Ok(instance_id)
    }

    /// Return the number of tracked runtime instances.
    pub fn total_instances(&self) -> usize {
        self.instances.len()
    }

    /// Return the identifiers of all tracked instances in deterministic order.
    pub fn instance_ids(&self) -> Vec<usize> {
        self.instances.keys().copied().collect()
    }

    /// Return whether the selected runtime instance is still live.
    pub fn is_live(&self, instance_id: usize) -> bool {
        self.instances
            .get(&instance_id)
            .map(|worker| !worker.is_terminated())
            .unwrap_or(false)
    }

    /// Forward a posted message to the selected runtime instance.
    pub fn post_message(&self, instance_id: usize, message: Value) -> bool {
        let Some(worker) = self.instances.get(&instance_id) else {
            return false;
        };
        worker.post_message(message);
        true
    }

    /// Forward a shared buffer to the selected runtime instance.
    pub fn post_shared_buffer(&self, instance_id: usize, buffer: SharedArrayBuffer) -> bool {
        let Some(worker) = self.instances.get(&instance_id) else {
            return false;
        };
        worker.post_shared_buffer(buffer);
        true
    }

    /// Terminate the selected runtime instance.
    pub fn terminate(&self, instance_id: usize) -> bool {
        let Some(worker) = self.instances.get(&instance_id) else {
            return false;
        };
        worker.terminate();
        true
    }

    fn snapshot_instance(
        &self,
        instance_id: usize,
        worker: &Worker,
    ) -> ThreadRuntimeInstanceSnapshot {
        let posted_items = worker.posted_items();
        let posted_messages = posted_items
            .iter()
            .filter_map(|item| match item {
                PostedItem::Message(message) => Some(message.clone()),
                PostedItem::SharedBuffer(_) => None,
            })
            .collect();
        let posted_shared_buffers = posted_items
            .into_iter()
            .filter_map(|item| match item {
                PostedItem::Message(_) => None,
                PostedItem::SharedBuffer(buffer) => Some(buffer.snapshot()),
            })
            .collect();

        ThreadRuntimeInstanceSnapshot {
            instance_id,
            script_url: worker.script_url().as_str().to_string(),
            posted_messages,
            posted_shared_buffers,
            was_terminated: worker.is_terminated(),
        }
    }

    /// Produce a stable snapshot of the current topology state.
    pub fn snapshot(&self) -> ThreadRuntimeShutdownReport {
        let total_instances = self.instances.len();
        let terminated_instances = self
            .instances
            .values()
            .filter(|worker| worker.is_terminated())
            .count();
        let live_instances = self
            .instances
            .iter()
            .filter(|(_, worker)| !worker.is_terminated())
            .map(|(instance_id, worker)| self.snapshot_instance(*instance_id, worker))
            .collect::<Vec<_>>();

        ThreadRuntimeShutdownReport {
            total_instances,
            terminated_instances,
            live_instances,
        }
    }

    /// Alias for the topology snapshot helper with an explicit thread-topology name.
    pub fn thread_topology_snapshot(&self) -> ThreadRuntimeShutdownReport {
        self.snapshot()
    }

    /// Produce a stable JSON-ready snapshot of the current topology state.
    pub fn snapshot_value(&self) -> Value {
        self.snapshot().snapshot_value()
    }

    /// Alias for the JSON-ready topology snapshot helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready topology snapshot helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready topology snapshot helper with an explicit thread-topology name.
    pub fn thread_topology_snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Produce a stable shutdown/leak report and mark every tracked instance terminated.
    pub fn shutdown(self) -> ThreadRuntimeShutdownReport {
        let report = self.snapshot();

        for worker in self.instances.values() {
            worker.terminate();
        }

        report
    }
}

/// A deterministic in-memory IndexedDB stub.
#[derive(Clone, Debug, Default)]
pub struct IndexedDb {
    name: String,
    stores: Arc<Mutex<BTreeMap<String, BTreeMap<String, Value>>>>,
}

/// Browser-aligned alias for the deterministic IndexedDB stub.
pub type IndexedDB = IndexedDb;

impl IndexedDb {
    /// Create a database stub with a stable name.
    pub fn open(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stores: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Return the database name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Store a JSON value in an object store.
    pub fn put(&self, store: impl Into<String>, key: impl Into<String>, value: Value) {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .entry(store.into())
            .or_default()
            .insert(key.into(), value);
    }

    /// Retrieve a JSON value from an object store.
    pub fn get(&self, store: &str, key: &str) -> Option<Value> {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .get(store)
            .and_then(|entries| entries.get(key))
            .cloned()
    }

    /// Delete a key from an object store.
    pub fn delete(&self, store: &str, key: &str) -> Option<Value> {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .get_mut(store)
            .and_then(|entries| entries.remove(key))
    }

    /// Remove all entries from one object store.
    pub fn clear_store(&self, store: &str) {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .remove(store);
    }

    /// Return the current object-store names.
    pub fn store_names(&self) -> Vec<String> {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Return a deterministic snapshot of all object stores and their entries.
    pub fn snapshot(&self) -> BTreeMap<String, BTreeMap<String, Value>> {
        self.stores
            .lock()
            .expect("indexeddb mutex poisoned")
            .clone()
    }

    /// Return the snapshot as a JSON object value.
    pub fn snapshot_value(&self) -> Value {
        Value::Object(
            self.snapshot()
                .into_iter()
                .map(|(store, entries)| {
                    (
                        store,
                        Value::Object(entries.into_iter().collect::<serde_json::Map<_, _>>()),
                    )
                })
                .collect(),
        )
    }

    /// Alias for the JSON-ready snapshot helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
    }

    /// Alias for the JSON-ready snapshot helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> Value {
        self.snapshot_value()
    }
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
#[path = "tests.rs"]
mod tests;
