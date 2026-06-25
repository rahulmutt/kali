//! Web API `navigator` baseline.

use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};

static NAVIGATOR: OnceLock<Navigator> = OnceLock::new();

/// Return the shared in-memory `navigator` baseline.
pub fn navigator() -> Navigator {
    NAVIGATOR.get_or_init(Navigator::default).clone()
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

#[cfg(test)]
#[path = "navigator_tests.rs"]
mod navigator_tests;
