//! Web API compatibility surface for Kali runtime.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use kali_common::bytewise_shared_memory_is_lock_free;
use ::url::ParseError as UrlParseError; // external `url` crate; `url` name is shadowed by our local module

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

mod url;
pub use url::*;

mod util;
pub use util::*;

mod websocket;
pub use websocket::*;

mod worker;
pub use worker::*;

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
    pub fn spawn_worker(&mut self, url: impl AsRef<str>) -> Result<usize, UrlParseError> {
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
