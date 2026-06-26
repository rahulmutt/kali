//! Utility helpers: crate init, formatting, inspection, and promisify bridge.

/// Initialize the Node API compatibility surface.
pub fn node_api_init() {}

/// A tiny `util.format`-style helper for deterministic test output.
pub fn util_format<T: AsRef<str>>(parts: &[T]) -> String {
    parts
        .iter()
        .map(|part| part.as_ref())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A deterministic `inspect` helper for debug-style summaries.
pub fn util_inspect<T: std::fmt::Debug>(value: &T) -> String {
    format!("{:?}", value)
}

/// Namespace-style wrapper for util helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeUtil;

impl NodeUtil {
    pub fn format<T: AsRef<str>>(parts: &[T]) -> String {
        util_format(parts)
    }

    pub fn inspect<T: std::fmt::Debug>(value: &T) -> String {
        util_inspect(value)
    }

    pub fn promisify<T: 'static, E: 'static, F>(operation: F) -> Result<T, E>
    where
        F: FnOnce(Box<dyn FnOnce(Result<T, E>)>),
    {
        util_promisify(operation)
    }
}

/// Minimal `util.promisify`-style helper for synchronous callback bridges.
///
/// The callback is invoked exactly once and its result is returned to the caller.
pub fn util_promisify<T: 'static, E: 'static, F>(operation: F) -> Result<T, E>
where
    F: FnOnce(Box<dyn FnOnce(Result<T, E>)>),
{
    let outcome = std::sync::Arc::new(std::sync::Mutex::new(None));
    let slot = std::sync::Arc::clone(&outcome);
    operation(Box::new(move |result| {
        *slot.lock().expect("promisify result mutex poisoned") = Some(result);
    }));

    let result = outcome
        .lock()
        .expect("promisify result mutex poisoned")
        .take()
        .expect("promisify callback was not invoked");
    result
}
