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

/// The outcome of resolving one dotted-path segment against one JSON node.
///
/// Kept as its own type (rather than folding straight into `Option`) so
/// `lookup` and `describe_absence` can never drift apart: both walk the path
/// through this single `resolve` function, so a failure mode `lookup` treats
/// as absent is guaranteed to be one `describe_absence` knows how to name.
///
/// Named `Segment` (and its resolver `resolve`) rather than `Step`/`step`:
/// this module is compiled alongside `model::Step` -- a case file's step --
/// and a `steps` module that runs them. Two unrelated `Step` types and a
/// `step` function shadowing the `steps` module is a reader trap, not a
/// namespacing win.
enum Segment<'a> {
    Found(&'a serde_json::Value),
    /// An object (or a scalar) has no such key.
    Absent,
    /// An array node hit a segment that is not a valid index -- non-numeric,
    /// negative (`usize` cannot represent it), or too large to fit a
    /// `usize`. All three collapse to one case: none of them is "a
    /// non-negative integer that fits", which is all a valid index is.
    NotAnIndex,
    /// An array node hit a syntactically valid index past the end.
    OutOfRange {
        len: usize,
    },
}

/// Resolve one segment against one node.
///
/// Dispatch is by the *actual JSON type* of `current`, never by whether the
/// segment looks numeric: a numeric segment against an array is an index: a
/// numeric-looking segment against an object (`{"0": "x"}`) is an ordinary
/// object key. This is the less surprising rule -- indexing is a property of
/// what you're indexing into, not of how the path was spelled -- and it is
/// what makes an object with a numeric-looking key behave exactly like any
/// other object, addressable the same way as `{"zero": "x"}` would be.
fn resolve<'a>(current: &'a serde_json::Value, segment: &str) -> Segment<'a> {
    match current {
        serde_json::Value::Array(items) => match segment.parse::<usize>() {
            Ok(index) => match items.get(index) {
                Some(next) => Segment::Found(next),
                None => Segment::OutOfRange { len: items.len() },
            },
            Err(_) => Segment::NotAnIndex,
        },
        _ => match current.get(segment) {
            Some(next) => Segment::Found(next),
            None => Segment::Absent,
        },
    }
}

/// `path == ""` addresses the whole document -- this is what `flatten_expected`
/// emits for a top-level `json = {}` (an empty table is a leaf at prefix
/// `""`, per its doc comment). `"".split('.')` would otherwise yield a single
/// empty-string segment and look for a JSON key literally named `""`, which
/// is never what an empty-table expectation means.
///
/// A numeric segment indexes into a JSON array (`errors.0.code`); against
/// anything else (an object, or a scalar with more path left to walk) a
/// segment is always a plain key/field lookup, numeric-looking or not -- see
/// `resolve`'s doc comment. This is closed dotted-path indexing, not an
/// expression language: no slices, no wildcards, no negative-from-end
/// indexing, no filters.
pub fn lookup<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    for segment in path.split('.') {
        match resolve(current, segment) {
            Segment::Found(next) => current = next,
            Segment::Absent | Segment::NotAnIndex | Segment::OutOfRange { .. } => return None,
        }
    }
    Some(current)
}

/// Describe *why* `lookup(value, path)` returned `None`, for a diagnosable
/// failure message. Walks the same `resolve` transitions a second time to find
/// which segment broke and how -- only meaningful to call after `lookup`
/// itself has already returned `None`.
pub(crate) fn describe_absence(value: &serde_json::Value, path: &str) -> String {
    let mut current = value;
    for segment in path.split('.') {
        match resolve(current, segment) {
            Segment::Found(next) => current = next,
            Segment::Absent => return "is absent".to_string(),
            Segment::NotAnIndex => {
                return format!(
                    "is absent (`.{segment}` is not a valid array index -- array elements are \
                     addressed by a non-negative integer segment that fits a `usize`, e.g. `.0`)"
                );
            }
            Segment::OutOfRange { len } => {
                return format!(
                    "is absent (index {segment} is out of range for an array of length {len})"
                );
            }
        }
    }
    "is absent".to_string()
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
