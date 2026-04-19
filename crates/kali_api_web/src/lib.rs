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
static LOCAL_STORAGE: OnceLock<Storage> = OnceLock::new();
static SESSION_STORAGE: OnceLock<Storage> = OnceLock::new();

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

/// An in-memory Web `Blob`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    bytes: Arc<[u8]>,
    mime_type: Option<String>,
}

impl Blob {
    /// Create a blob from byte chunks and an optional MIME type.
    pub fn new<I, B>(parts: I, mime_type: Option<String>) -> Self
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let mut bytes = Vec::new();
        for part in parts {
            bytes.extend_from_slice(part.as_ref());
        }

        Self {
            bytes: Arc::from(bytes),
            mime_type,
        }
    }

    /// Return the blob's bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Return the blob's size in bytes.
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Return the blob's MIME type, if one was supplied.
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    /// Decode the blob as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.bytes.to_vec())
    }
}

/// An in-memory Web `File`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    blob: Blob,
    name: String,
    last_modified: u64,
}

impl File {
    /// Create a file from byte chunks, a file name, and an optional MIME type.
    pub fn new<I, B>(
        name: impl Into<String>,
        parts: I,
        mime_type: Option<String>,
        last_modified: u64,
    ) -> Self
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        Self {
            blob: Blob::new(parts, mime_type),
            name: name.into(),
            last_modified,
        }
    }

    /// Return the file name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the file's last-modified timestamp.
    pub fn last_modified(&self) -> u64 {
        self.last_modified
    }

    /// Return the file's bytes.
    pub fn bytes(&self) -> &[u8] {
        self.blob.bytes()
    }

    /// Return the file's size in bytes.
    pub fn size(&self) -> usize {
        self.blob.size()
    }

    /// Return the embedded blob view.
    pub fn blob(&self) -> &Blob {
        &self.blob
    }

    /// Decode the file as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        self.blob.text()
    }
}

/// Readable state for the in-memory Web `FileReader`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileReaderState {
    Empty,
    Loading,
    Done,
}

/// An in-memory Web `FileReader` baseline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReader {
    ready_state: FileReaderState,
    result: Option<Vec<u8>>,
}

impl Default for FileReader {
    fn default() -> Self {
        Self {
            ready_state: FileReaderState::Empty,
            result: None,
        }
    }
}

impl FileReader {
    /// Create a new file reader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the reader's current ready state.
    pub fn ready_state(&self) -> FileReaderState {
        self.ready_state
    }

    /// Return the last read bytes, if any.
    pub fn result_bytes(&self) -> Option<&[u8]> {
        self.result.as_deref()
    }

    /// Reset the reader back to the empty state.
    pub fn clear(&mut self) {
        self.ready_state = FileReaderState::Empty;
        self.result = None;
    }

    /// Read a blob as raw bytes.
    pub fn read_as_bytes(&mut self, blob: &Blob) -> Vec<u8> {
        self.ready_state = FileReaderState::Loading;
        let bytes = blob.bytes().to_vec();
        self.result = Some(bytes.clone());
        self.ready_state = FileReaderState::Done;
        bytes
    }

    /// Read a blob as UTF-8 text.
    pub fn read_as_text(&mut self, blob: &Blob) -> Result<String, std::string::FromUtf8Error> {
        self.ready_state = FileReaderState::Loading;
        let bytes = blob.bytes().to_vec();
        self.result = Some(bytes.clone());
        self.ready_state = FileReaderState::Done;
        String::from_utf8(bytes)
    }

    /// Read a file as raw bytes.
    pub fn read_file_as_bytes(&mut self, file: &File) -> Vec<u8> {
        self.read_as_bytes(file.blob())
    }

    /// Read a file as UTF-8 text.
    pub fn read_file_as_text(&mut self, file: &File) -> Result<String, std::string::FromUtf8Error> {
        self.read_as_text(file.blob())
    }
}

/// A lightweight in-memory Web Storage implementation.
#[derive(Clone, Debug, Default)]
pub struct Storage {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

impl Storage {
    /// Create an empty storage bucket.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of stored entries.
    pub fn length(&self) -> usize {
        self.values.lock().expect("storage mutex poisoned").len()
    }

    /// Look up a stored value by key.
    pub fn get_item(&self, key: &str) -> Option<String> {
        self.values
            .lock()
            .expect("storage mutex poisoned")
            .get(key)
            .cloned()
    }

    /// Insert or replace a stored value.
    pub fn set_item(&self, key: impl Into<String>, value: impl Into<String>) {
        self.values
            .lock()
            .expect("storage mutex poisoned")
            .insert(key.into(), value.into());
    }

    /// Remove a stored value and return it if present.
    pub fn remove_item(&self, key: &str) -> Option<String> {
        self.values
            .lock()
            .expect("storage mutex poisoned")
            .remove(key)
    }

    /// Remove all entries from the storage bucket.
    pub fn clear(&self) {
        self.values.lock().expect("storage mutex poisoned").clear();
    }

    /// Return the key at the requested insertion index.
    pub fn key(&self, index: usize) -> Option<String> {
        self.values
            .lock()
            .expect("storage mutex poisoned")
            .keys()
            .nth(index)
            .cloned()
    }

    /// Return a deterministic snapshot of the current entries.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.values.lock().expect("storage mutex poisoned").clone()
    }
}

/// Return the shared in-memory `localStorage` bucket.
pub fn local_storage() -> Storage {
    LOCAL_STORAGE.get_or_init(Storage::new).clone()
}

/// Return the shared in-memory `sessionStorage` bucket.
pub fn session_storage() -> Storage {
    SESSION_STORAGE.get_or_init(Storage::new).clone()
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

/// A stub WebSocket implementation that keeps the browser baseline deterministic.
#[derive(Clone, Debug)]
pub struct WebSocket {
    url: Url,
    ready_state: WebSocketReadyState,
    sent_text_messages: Arc<Mutex<Vec<String>>>,
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

    /// Return the buffered text payloads.
    pub fn sent_text_messages(&self) -> Vec<String> {
        self.sent_text_messages
            .lock()
            .expect("websocket mutex poisoned")
            .clone()
    }

    /// Transition the socket to the closed state.
    pub fn close(&mut self) {
        self.ready_state = WebSocketReadyState::Closed;
    }
}

/// A deterministic worker stub used by the browser baseline.
#[derive(Clone, Debug)]
pub struct Worker {
    script_url: Url,
    posted_messages: Arc<Mutex<Vec<Value>>>,
    terminated: Arc<AtomicBool>,
}

impl Worker {
    /// Create a new worker stub from a parsed script URL.
    pub fn new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(Self {
            script_url: Url::parse(url.as_ref())?,
            posted_messages: Arc::new(Mutex::new(Vec::new())),
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
        self.posted_messages
            .lock()
            .expect("worker mutex poisoned")
            .push(message);
    }

    /// Return the buffered messages.
    pub fn posted_messages(&self) -> Vec<Value> {
        self.posted_messages
            .lock()
            .expect("worker mutex poisoned")
            .clone()
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

/// A deterministic in-memory IndexedDB stub.
#[derive(Clone, Debug, Default)]
pub struct IndexedDb {
    name: String,
    stores: Arc<Mutex<BTreeMap<String, BTreeMap<String, Value>>>>,
}

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
    fn blob_collects_bytes_and_text() {
        let blob = Blob::new(
            ["hello ".as_bytes(), "world".as_bytes()],
            Some("text/plain".to_string()),
        );
        assert_eq!(blob.size(), 11);
        assert_eq!(blob.mime_type(), Some("text/plain"));
        assert_eq!(blob.bytes(), b"hello world");
        assert_eq!(blob.text().expect("blob text"), "hello world");
    }

    #[test]
    fn file_wraps_blob_metadata() {
        let file = File::new(
            "report.txt",
            ["hello ".as_bytes(), "world".as_bytes()],
            Some("text/plain".to_string()),
            42,
        );
        assert_eq!(file.name(), "report.txt");
        assert_eq!(file.last_modified(), 42);
        assert_eq!(file.size(), 11);
        assert_eq!(file.bytes(), b"hello world");
        assert_eq!(file.blob().mime_type(), Some("text/plain"));
        assert_eq!(file.text().expect("file text"), "hello world");
    }

    #[test]
    fn file_reader_reads_blob_and_file_payloads() {
        let blob = Blob::new(
            ["reader payload".as_bytes()],
            Some("text/plain".to_string()),
        );
        let file = File::new("reader.txt", ["reader payload".as_bytes()], None, 7);

        let mut reader = FileReader::new();
        assert_eq!(reader.ready_state(), FileReaderState::Empty);

        assert_eq!(
            reader.read_as_text(&blob).expect("blob text"),
            "reader payload"
        );
        assert_eq!(reader.ready_state(), FileReaderState::Done);
        assert_eq!(reader.result_bytes(), Some(b"reader payload".as_slice()));

        reader.clear();
        assert_eq!(reader.ready_state(), FileReaderState::Empty);
        assert!(reader.result_bytes().is_none());

        assert_eq!(reader.read_file_as_bytes(&file), b"reader payload");
        assert_eq!(reader.ready_state(), FileReaderState::Done);
        assert_eq!(
            reader.read_file_as_text(&file).expect("file text"),
            "reader payload"
        );
    }

    #[test]
    fn storage_round_trips_values_and_stays_ordered() {
        let storage = Storage::new();
        storage.set_item("alpha", "1");
        storage.set_item("beta", "2");

        assert_eq!(storage.length(), 2);
        assert_eq!(storage.get_item("alpha").as_deref(), Some("1"));
        assert_eq!(storage.key(0).as_deref(), Some("alpha"));
        assert_eq!(storage.key(1).as_deref(), Some("beta"));
        assert_eq!(storage.remove_item("alpha").as_deref(), Some("1"));
        assert_eq!(storage.length(), 1);
        storage.clear();
        assert_eq!(storage.length(), 0);
        assert!(storage.snapshot().is_empty());
    }

    #[test]
    fn shared_browser_storage_buckets_remain_isolated() {
        let local = local_storage();
        let session = session_storage();
        local.clear();
        session.clear();

        local.set_item("mode", "local");
        session.set_item("mode", "session");

        assert_eq!(local.get_item("mode").as_deref(), Some("local"));
        assert_eq!(session.get_item("mode").as_deref(), Some("session"));
        assert_ne!(local.snapshot(), session.snapshot());

        local.clear();
        session.clear();
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

    #[test]
    fn custom_event_carries_detail_payload() {
        let event = CustomEvent::new("payload", Value::String("detail".to_string()));
        assert_eq!(event.event().event_type(), "payload");
        assert_eq!(event.detail(), &Value::String("detail".to_string()));
    }

    #[test]
    fn websocket_stub_tracks_sent_messages() {
        let mut socket = WebSocket::new("https://example.com/socket").expect("websocket url");
        assert_eq!(socket.ready_state(), WebSocketReadyState::Open);
        assert_eq!(socket.url().as_str(), "https://example.com/socket");

        socket.send_text("hello");
        socket.send_text("world");
        assert_eq!(socket.sent_text_messages(), vec!["hello", "world"]);

        socket.close();
        assert_eq!(socket.ready_state(), WebSocketReadyState::Closed);
    }

    #[test]
    fn worker_stub_records_posted_messages() {
        let worker = Worker::new("https://example.com/worker.js").expect("worker url");
        assert_eq!(
            worker.script_url().as_str(),
            "https://example.com/worker.js"
        );
        assert!(!worker.is_terminated());

        worker.post_message(Value::String("ping".to_string()));
        assert_eq!(
            worker.posted_messages(),
            vec![Value::String("ping".to_string())]
        );

        worker.terminate();
        assert!(worker.is_terminated());
        worker.post_message(Value::String("ignored".to_string()));
        assert_eq!(
            worker.posted_messages(),
            vec![Value::String("ping".to_string())]
        );
    }

    #[test]
    fn indexed_db_stub_persists_values() {
        let db = IndexedDb::open("browser-cache");
        assert_eq!(db.name(), "browser-cache");

        db.put("sessions", "alpha", Value::String("1".to_string()));
        db.put("sessions", "beta", Value::String("2".to_string()));
        assert_eq!(db.store_names(), vec!["sessions".to_string()]);
        assert_eq!(
            db.get("sessions", "alpha"),
            Some(Value::String("1".to_string()))
        );
        assert_eq!(
            db.delete("sessions", "alpha"),
            Some(Value::String("1".to_string()))
        );
        assert_eq!(db.get("sessions", "alpha"), None);

        db.clear_store("sessions");
        assert!(db.store_names().is_empty());
    }
}
