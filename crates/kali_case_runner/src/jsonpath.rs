//! Dotted-path lookup into a JSON document, and equality between a TOML
//! expectation and a JSON actual.

/// Flatten a nested TOML table into `(dotted.path, leaf)` pairs. Arrays are
/// leaves, not tables -- `json.errors = []` asserts the whole array. An
/// *empty* table is a leaf for the same reason: `payload.diagnostics = {}`
/// (or a top-level `json = {}`) asserts "this is an empty object," and
/// recursing into it would contribute zero paths, leaving `check_json`
/// nothing to iterate and the case passing having asserted nothing.
pub fn flatten_expected(table: &toml::Value) -> Vec<(String, toml::Value)> {
    fn walk(prefix: &str, value: &toml::Value, out: &mut Vec<(String, toml::Value)>) {
        match value {
            toml::Value::Table(map) if !map.is_empty() => {
                for (key, child) in map {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&path, child, out);
                }
            }
            leaf => out.push((prefix.to_string(), leaf.clone())),
        }
    }
    let mut out = Vec::new();
    walk("", table, &mut out);
    out
}

/// `path == ""` addresses the whole document -- this is what `flatten_expected`
/// emits for a top-level `json = {}` (an empty table is a leaf at prefix
/// `""`, per its doc comment). `"".split('.')` would otherwise yield a single
/// empty-string segment and look for a JSON key literally named `""`, which
/// is never what an empty-table expectation means.
pub fn lookup<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

pub fn values_equal(expected: &toml::Value, actual: &serde_json::Value) -> bool {
    match (expected, actual) {
        (toml::Value::String(e), serde_json::Value::String(a)) => e == a,
        (toml::Value::Integer(e), serde_json::Value::Number(a)) => a.as_i64() == Some(*e),
        (toml::Value::Float(e), serde_json::Value::Number(a)) => a.as_f64() == Some(*e),
        (toml::Value::Boolean(e), serde_json::Value::Bool(a)) => e == a,
        (toml::Value::Array(e), serde_json::Value::Array(a)) => {
            e.len() == a.len() && e.iter().zip(a.iter()).all(|(e, a)| values_equal(e, a))
        }
        (toml::Value::Table(e), serde_json::Value::Object(a)) => {
            e.len() == a.len()
                && e.iter()
                    .all(|(k, e)| a.get(k).is_some_and(|a| values_equal(e, a)))
        }
        // TOML cannot express null, so nothing matches a JSON null.
        _ => false,
    }
}

#[cfg(test)]
#[path = "jsonpath_tests.rs"]
mod jsonpath_tests;
