//! Deterministic environment view for the Deno compatibility layer.

use serde_json::Value;
use std::collections::BTreeMap;

/// Deterministic environment view for the Deno compatibility layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenoEnv {
    values: BTreeMap<String, String>,
}

impl DenoEnv {
    /// Create an environment view from host-provided values.
    pub fn new(values: BTreeMap<String, String>) -> Self {
        Self { values }
    }

    /// Read an environment variable from the sandbox-filtered view.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Check whether an environment variable is present in the captured view.
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Set or replace an environment variable in the captured view.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(key.into(), value.into())
    }

    /// Remove an environment variable from the captured view.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }

    /// Alias for the mutable environment removal helper.
    pub fn delete(&mut self, key: &str) -> Option<String> {
        self.remove(key)
    }

    /// Return a deterministic snapshot of the visible environment.
    pub fn to_object(&self) -> BTreeMap<String, String> {
        self.values.clone()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Alias for the deterministic environment snapshot helper with a generic object-value name.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Return the visible environment as a JSON object value.
    pub fn to_json_value(&self) -> Value {
        Value::Object(
            self.values
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        )
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_to_json_value(&self) -> Value {
        self.to_json_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_value(&self) -> Value {
        self.to_json_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_json_value(&self) -> Value {
        self.to_json_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper with a generic value name.
    pub fn snapshot_json_value(&self) -> Value {
        self.to_json_value()
    }

    /// Alias for the deterministic environment snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> Value {
        self.to_json_value()
    }

    /// Iterate over the captured key/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

#[cfg(test)]
#[path = "env_tests.rs"]
mod env_tests;
