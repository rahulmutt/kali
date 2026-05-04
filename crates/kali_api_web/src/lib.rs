//! Web API compatibility surface for Kali runtime.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::{self, Write as _},
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::Instant,
};

use kali_common::bytewise_shared_memory_is_lock_free;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use url::{form_urlencoded, Url};

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();
static LOCAL_STORAGE: OnceLock<Storage> = OnceLock::new();
static SESSION_STORAGE: OnceLock<Storage> = OnceLock::new();
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

    /// Produce a deterministic readable stream over the blob payload.
    pub fn stream(&self) -> ReadableStream {
        ReadableStream::from_chunks([self.bytes()])
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

    /// Produce a deterministic readable stream over the file payload.
    pub fn stream(&self) -> ReadableStream {
        self.blob.stream()
    }
}

#[derive(Debug, Default)]
struct DeterministicStreamState {
    chunks: Mutex<Vec<Vec<u8>>>,
    closed: AtomicBool,
}

impl DeterministicStreamState {
    fn push_chunk(&self, chunk: impl AsRef<[u8]>) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        self.chunks
            .lock()
            .expect("stream mutex poisoned")
            .push(chunk.as_ref().to_vec());
    }

    fn snapshot(&self) -> Vec<Vec<u8>> {
        self.chunks.lock().expect("stream mutex poisoned").clone()
    }

    fn bytes(&self) -> Vec<u8> {
        self.snapshot().into_iter().flatten().collect()
    }

    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

/// A deterministic readable Web stream baseline.
#[derive(Clone, Debug, Default)]
pub struct ReadableStream {
    state: Arc<DeterministicStreamState>,
}

impl ReadableStream {
    /// Create an empty readable stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a readable stream from deterministic byte chunks.
    pub fn from_chunks<I, B>(chunks: I) -> Self
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let stream = Self::new();
        for chunk in chunks {
            stream.append_chunk(chunk);
        }
        stream
    }

    /// Append a chunk to the readable stream.
    pub fn append_chunk(&self, chunk: impl AsRef<[u8]>) {
        self.state.push_chunk(chunk);
    }

    /// Return the buffered chunks in deterministic order.
    pub fn chunks(&self) -> Vec<Vec<u8>> {
        self.state.snapshot()
    }

    /// Return the buffered bytes as a single flattened payload.
    pub fn bytes(&self) -> Vec<u8> {
        self.state.bytes()
    }

    /// Decode the readable stream as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.bytes())
    }

    /// Transition the readable stream to a closed state.
    pub fn close(&self) {
        self.state.close();
    }

    /// Return whether the stream has been closed.
    pub fn is_closed(&self) -> bool {
        self.state.is_closed()
    }
}

impl PartialEq for ReadableStream {
    fn eq(&self, other: &Self) -> bool {
        self.is_closed() == other.is_closed() && self.chunks() == other.chunks()
    }
}

impl Eq for ReadableStream {}

/// A deterministic writable Web stream baseline.
#[derive(Clone, Debug, Default)]
pub struct WritableStream {
    state: Arc<DeterministicStreamState>,
}

impl WritableStream {
    /// Create an empty writable stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a chunk to the writable stream.
    pub fn write(&self, chunk: impl AsRef<[u8]>) {
        self.state.push_chunk(chunk);
    }

    /// Append UTF-8 text to the writable stream.
    pub fn write_text(&self, text: impl Into<String>) {
        self.write(text.into().into_bytes());
    }

    /// Return the buffered chunks in deterministic order.
    pub fn chunks(&self) -> Vec<Vec<u8>> {
        self.state.snapshot()
    }

    /// Return the buffered bytes as a single flattened payload.
    pub fn bytes(&self) -> Vec<u8> {
        self.state.bytes()
    }

    /// Decode the writable stream as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.bytes())
    }

    /// Transition the writable stream to a closed state.
    pub fn close(&self) {
        self.state.close();
    }

    /// Return whether the stream has been closed.
    pub fn is_closed(&self) -> bool {
        self.state.is_closed()
    }
}

impl PartialEq for WritableStream {
    fn eq(&self, other: &Self) -> bool {
        self.is_closed() == other.is_closed() && self.chunks() == other.chunks()
    }
}

impl Eq for WritableStream {}

/// A deterministic transform stream baseline.
#[derive(Clone, Debug)]
pub struct TransformStream {
    readable: ReadableStream,
    writable: WritableStream,
}

impl TransformStream {
    /// Create a new transform stream whose readable and writable sides share one backing state.
    pub fn new() -> Self {
        let state = Arc::new(DeterministicStreamState::default());
        Self {
            readable: ReadableStream {
                state: Arc::clone(&state),
            },
            writable: WritableStream { state },
        }
    }

    /// Return the readable side of the transform stream.
    pub fn readable(&self) -> &ReadableStream {
        &self.readable
    }

    /// Return the writable side of the transform stream.
    pub fn writable(&self) -> &WritableStream {
        &self.writable
    }
}

impl Default for TransformStream {
    fn default() -> Self {
        Self::new()
    }
}

/// A deterministic `TextEncoderStream` baseline layered on the shared transform-stream state.
#[derive(Clone, Debug)]
pub struct TextEncoderStream {
    inner: TransformStream,
}

impl TextEncoderStream {
    /// Create a new text encoder stream baseline.
    pub fn new() -> Self {
        Self {
            inner: TransformStream::new(),
        }
    }

    /// Return the readable side of the text encoder stream.
    pub fn readable(&self) -> &ReadableStream {
        self.inner.readable()
    }

    /// Return the writable side of the text encoder stream.
    pub fn writable(&self) -> &WritableStream {
        self.inner.writable()
    }

    /// Append UTF-8 text to the encoder stream.
    pub fn write_text(&self, text: impl Into<String>) {
        self.writable().write_text(text);
    }
}

impl Default for TextEncoderStream {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for TextEncoderStream {
    fn eq(&self, other: &Self) -> bool {
        self.readable() == other.readable() && self.writable() == other.writable()
    }
}

impl Eq for TextEncoderStream {}

/// A deterministic `TextDecoderStream` baseline layered on the shared transform-stream state.
#[derive(Clone, Debug)]
pub struct TextDecoderStream {
    inner: TransformStream,
}

impl TextDecoderStream {
    /// Create a new text decoder stream baseline.
    pub fn new() -> Self {
        Self {
            inner: TransformStream::new(),
        }
    }

    /// Return the readable side of the text decoder stream.
    pub fn readable(&self) -> &ReadableStream {
        self.inner.readable()
    }

    /// Return the writable side of the text decoder stream.
    pub fn writable(&self) -> &WritableStream {
        self.inner.writable()
    }

    /// Append raw bytes to the decoder stream.
    pub fn write(&self, chunk: impl AsRef<[u8]>) {
        self.writable().write(chunk);
    }
}

impl Default for TextDecoderStream {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for TextDecoderStream {
    fn eq(&self, other: &Self) -> bool {
        self.readable() == other.readable() && self.writable() == other.writable()
    }
}

impl Eq for TextDecoderStream {}

/// Value stored inside the in-memory Web `FormData` baseline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormDataValue {
    Text(String),
    Blob(Blob),
    File(File),
}

impl From<&str> for FormDataValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for FormDataValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Blob> for FormDataValue {
    fn from(value: Blob) -> Self {
        Self::Blob(value)
    }
}

impl From<File> for FormDataValue {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}

/// A single deterministic `FormData` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormDataEntry {
    name: String,
    value: FormDataValue,
}

impl FormDataEntry {
    /// Create a new entry.
    pub fn new(name: impl Into<String>, value: impl Into<FormDataValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// Return the entry name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the entry value.
    pub fn value(&self) -> &FormDataValue {
        &self.value
    }
}

/// A deterministic in-memory Web `FormData` baseline.
#[derive(Clone, Debug, Default)]
pub struct FormData {
    entries: Arc<Mutex<Vec<FormDataEntry>>>,
}

impl FormData {
    /// Create an empty form-data bucket.
    pub fn new() -> Self {
        Self::default()
    }

    fn remove_matching(&self, name: &str) {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .retain(|entry| entry.name != name);
    }

    /// Append a new entry while preserving insertion order.
    pub fn append(&self, name: impl Into<String>, value: impl Into<FormDataValue>) {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .push(FormDataEntry::new(name, value));
    }

    /// Replace all matching entries with a single value.
    pub fn set(&self, name: impl Into<String>, value: impl Into<FormDataValue>) {
        let name = name.into();
        self.remove_matching(&name);
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .push(FormDataEntry::new(name, value));
    }

    /// Return whether any entry with the requested name exists.
    pub fn has(&self, name: &str) -> bool {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .iter()
            .any(|entry| entry.name == name)
    }

    /// Return the first matching entry, if present.
    pub fn get(&self, name: &str) -> Option<FormDataEntry> {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .iter()
            .find(|entry| entry.name == name)
            .cloned()
    }

    /// Return all entries that match the requested name in insertion order.
    pub fn get_all(&self, name: &str) -> Vec<FormDataEntry> {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .iter()
            .filter(|entry| entry.name == name)
            .cloned()
            .collect()
    }

    /// Remove all entries that match the requested name.
    pub fn delete(&self, name: &str) {
        self.remove_matching(name);
    }

    /// Return a deterministic snapshot of the entries.
    pub fn entries(&self) -> Vec<FormDataEntry> {
        self.entries
            .lock()
            .expect("formdata mutex poisoned")
            .clone()
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

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Error returned by the deterministic base64 helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base64Error {
    message: String,
}

impl Base64Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Base64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Base64Error {}

/// Encode a binary string as base64 using the browser's `btoa` semantics.
pub fn btoa(input: &str) -> Result<String, Base64Error> {
    let mut bytes = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let code = ch as u32;
        if code > 0xFF {
            return Err(Base64Error::new(
                "The string to be encoded contains characters outside of the Latin1 range.",
            ));
        }
        bytes.push(code as u8);
    }

    Ok(encode_base64(&bytes))
}

/// Decode a base64 string using the browser's `atob` semantics.
pub fn atob(input: &str) -> Result<String, Base64Error> {
    let mut normalized: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    match normalized.len() % 4 {
        0 => {}
        1 => {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ))
        }
        2 => normalized.push_str("=="),
        3 => normalized.push('='),
        _ => {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ))
        }
    }

    let decoded = decode_base64(&normalized)?;
    Ok(decoded.into_iter().map(char::from).collect())
}

fn encode_base64(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);

        output.push(char::from(BASE64_ALPHABET[(first >> 2) as usize]));
        output.push(char::from(
            BASE64_ALPHABET[((first & 0b0000_0011) << 4 | (second >> 4)) as usize],
        ));
        if chunk.len() > 1 {
            output.push(char::from(
                BASE64_ALPHABET[((second & 0b0000_1111) << 2 | (third >> 6)) as usize],
            ));
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(char::from(BASE64_ALPHABET[(third & 0b0011_1111) as usize]));
        } else {
            output.push('=');
        }
    }
    output
}

fn decode_base64(input: &str) -> Result<Vec<u8>, Base64Error> {
    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(Base64Error::new(
            "The string to be decoded is not correctly encoded.",
        ));
    }

    let mut output = Vec::with_capacity((bytes.len() / 4) * 3);
    let chunk_count = bytes.len() / 4;
    for (chunk_index, chunk) in bytes.chunks(4).enumerate() {
        let mut values = [0u8; 4];
        let mut padding = 0usize;

        for (index, byte) in chunk.iter().copied().enumerate() {
            if byte == b'=' {
                padding += 1;
                values[index] = 0;
                continue;
            }

            if padding > 0 {
                return Err(Base64Error::new(
                    "The string to be decoded is not correctly encoded.",
                ));
            }

            values[index] = decode_base64_value(byte).ok_or_else(|| {
                Base64Error::new("The string to be decoded contains invalid base64 characters.")
            })?;
        }

        if padding > 2 {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ));
        }
        if padding > 0 && chunk_index + 1 != chunk_count {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ));
        }

        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding == 0 {
            output.push((values[2] << 6) | values[3]);
        }
    }

    Ok(output)
}

fn decode_base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
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

/// Generate a v4 UUID string for `crypto.randomUUID()`-style calls.
pub fn random_uuid() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    fill_random_values(&mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut uuid = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            uuid.push('-');
        }
        write!(&mut uuid, "{:02x}", byte).expect("writing to a String cannot fail");
    }

    Ok(uuid)
}

/// Errors returned by the deterministic Web Crypto helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebCryptoError {
    UnsupportedDigestAlgorithm(String),
}

impl fmt::Display for WebCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDigestAlgorithm(algorithm) => {
                write!(f, "unsupported Web Crypto digest algorithm '{algorithm}'")
            }
        }
    }
}

impl std::error::Error for WebCryptoError {}

fn canonicalize_digest_algorithm(name: &str) -> String {
    name.chars()
        .filter(|character| {
            !character.is_ascii_whitespace() && *character != '-' && *character != '_'
        })
        .flat_map(char::to_uppercase)
        .collect()
}

/// Deterministic Web Crypto facade for the shared randomness and digest subset.
#[derive(Clone, Copy, Debug, Default)]
pub struct Crypto;

impl Crypto {
    /// Fill the provided buffer with randomness for `crypto.getRandomValues()`.
    pub fn get_random_values(&self, buffer: &mut [u8]) -> Result<(), getrandom::Error> {
        fill_random_values(buffer)
    }

    /// Generate a v4 UUID string for `crypto.randomUUID()`-style calls.
    pub fn random_uuid(&self) -> Result<String, getrandom::Error> {
        random_uuid()
    }

    /// Return the deterministic `subtle` helper namespace.
    pub fn subtle(&self) -> SubtleCrypto {
        SubtleCrypto
    }
}

/// Deterministic Web Crypto `subtle` facade for digest support.
#[derive(Clone, Copy, Debug, Default)]
pub struct SubtleCrypto;

impl SubtleCrypto {
    /// Compute a deterministic digest for the provided payload.
    pub fn digest(
        &self,
        algorithm: impl AsRef<str>,
        data: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, WebCryptoError> {
        let algorithm_name = algorithm.as_ref();
        let normalized = canonicalize_digest_algorithm(algorithm_name);

        match normalized.as_str() {
            "SHA1" => Ok(Sha1::digest(data.as_ref()).to_vec()),
            "SHA224" => Ok(Sha224::digest(data.as_ref()).to_vec()),
            "SHA256" => Ok(Sha256::digest(data.as_ref()).to_vec()),
            "SHA384" => Ok(Sha384::digest(data.as_ref()).to_vec()),
            "SHA512" => Ok(Sha512::digest(data.as_ref()).to_vec()),
            _ => Err(WebCryptoError::UnsupportedDigestAlgorithm(
                algorithm_name.trim().to_string(),
            )),
        }
    }
}

/// Return the shared deterministic Web Crypto facade.
pub fn crypto() -> Crypto {
    Crypto
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
            script_url: Url::parse(url.as_ref())?,
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

/// Deterministic shutdown/leak accounting for the runtime-topology model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadRuntimeShutdownReport {
    /// Number of runtime instances created by the topology.
    pub total_instances: usize,
    /// Number of instances that were already terminated before shutdown.
    pub terminated_instances: usize,
    /// Instances that were still live when shutdown began.
    pub live_instances: Vec<ThreadRuntimeInstanceSnapshot>,
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
