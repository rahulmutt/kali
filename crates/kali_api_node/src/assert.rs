//! Minimal Node.js `assert` module compatibility helpers.

/// Minimal assertion helpers used by Node compatibility tests.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeAssert;

impl NodeAssert {
    pub fn ok(condition: bool, message: impl Into<String>) -> Result<(), String> {
        condition.then_some(()).ok_or_else(|| message.into())
    }

    pub fn equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        (actual == expected).then_some(()).ok_or_else(|| {
            format!(
                "{}: expected {:?}, got {:?}",
                message.into(),
                expected,
                actual
            )
        })
    }

    pub fn not_equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        (actual != expected).then_some(()).ok_or_else(|| {
            format!(
                "{}: value unexpectedly matched {:?}",
                message.into(),
                expected
            )
        })
    }

    pub fn deep_equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::equal(actual, expected, message)
    }

    pub fn strict_equal<T>(
        actual: &T,
        expected: &T,
        message: impl Into<String>,
    ) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::equal(actual, expected, message)
    }

    pub fn not_strict_equal<T>(
        actual: &T,
        expected: &T,
        message: impl Into<String>,
    ) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::not_equal(actual, expected, message)
    }

    pub fn fail(message: impl Into<String>) -> Result<(), String> {
        Err(message.into())
    }
}

/// Backwards-compatible assertion helper used by existing tests.
pub fn assert_true(condition: bool, message: impl Into<String>) -> Result<(), String> {
    NodeAssert::ok(condition, message)
}

#[cfg(test)]
#[path = "assert_tests.rs"]
mod assert_tests;
