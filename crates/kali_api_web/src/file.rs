//! File, Blob, FormData, and FileReader Web API baselines.

use std::sync::{Arc, Mutex};

use crate::ReadableStream;

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

#[cfg(test)]
#[path = "file_tests.rs"]
mod file_tests;
