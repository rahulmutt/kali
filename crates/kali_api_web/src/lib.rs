//! Web API compatibility surface for Kali runtime.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

mod base64;
pub use base64::*;

mod crypto;
pub use crypto::*;

mod events;
pub use events::*;

mod fetch;
pub use fetch::*;

mod file;
pub use file::*;

mod navigator;
pub use navigator::*;

mod storage;
pub use storage::*;

mod streams;
pub use streams::*;

mod threads;
pub use threads::*;

mod url;
pub use url::*;

mod util;
pub use util::*;

mod websocket;
pub use websocket::*;

mod worker;
pub use worker::*;

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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
