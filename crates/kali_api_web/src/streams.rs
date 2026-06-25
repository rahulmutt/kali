//! Deterministic Web Streams API baseline (`ReadableStream`, `WritableStream`, `TransformStream`, `TextEncoderStream`, `TextDecoderStream`).

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

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

#[cfg(test)]
#[path = "streams_tests.rs"]
mod streams_tests;
