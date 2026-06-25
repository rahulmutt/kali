//! Web Storage API (`localStorage`, `sessionStorage`).

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

static LOCAL_STORAGE: OnceLock<Storage> = OnceLock::new();
static SESSION_STORAGE: OnceLock<Storage> = OnceLock::new();

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

    /// Alias for the deterministic storage snapshot helper.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.snapshot()
    }

    /// Return the storage snapshot as a JSON object value.
    pub fn snapshot_value(&self) -> Value {
        Value::Object(
            self.snapshot()
                .into_iter()
                .map(|(key, value)| (key, Value::String(value)))
                .collect(),
        )
    }

    /// Alias for the deterministic JSON-ready storage snapshot helper.
    pub fn snapshot_json_value(&self) -> Value {
        self.snapshot_value()
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

#[cfg(test)]
#[path = "storage_tests.rs"]
mod storage_tests;
