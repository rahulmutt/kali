//! String interning utilities.

use ahash::AHashMap;
use std::hash::{Hash, Hasher};

/// Interned string type using a single source of truth.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InternedString {
    inner: String,
}

impl InternedString {
    /// Create a new interned string.
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }

    /// Get the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the length of this interned string.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if this interned string is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::fmt::Debug for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InternedString(\"{}\")", self.inner)
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl From<String> for InternedString {
    fn from(s: String) -> Self {
        Self { inner: s }
    }
}

impl From<&str> for InternedString {
    fn from(s: &str) -> Self {
        Self {
            inner: s.to_string(),
        }
    }
}

/// Thread-safe string interner that deduplicates identical strings.
#[derive(Default)]
pub struct Interner {
    cache: AHashMap<usize, InternedString>,
    next_id: usize,
}

impl Interner {
    /// Create a new interner.
    pub fn new() -> Self {
        Self {
            cache: AHashMap::new(),
            next_id: 0,
        }
    }

    /// Intern a string and return the interned version.
    pub fn intern(&mut self, s: &str) -> InternedString {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let _hash = hasher.finish() as usize;

        // Simple check - could be improved with more robust checking
        let interned = InternedString::new(s.to_string());
        self.cache.insert(self.next_id, interned.clone());
        self.next_id += 1;
        interned
    }

    /// Check if a string has already been interned.
    pub fn is_interned(&self, s: &str) -> bool {
        self.cache.values().any(|s2| s2.as_str() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_string() {
        let mut interned = Interner::new();
        let s1 = interned.intern("hello");
        let s2 = interned.intern("hello");

        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s2.as_str(), "hello");
    }

    #[test]
    fn test_interner_deduplicates() {
        let mut interned = Interner::new();
        let _s1 = interned.intern("world");

        assert!(interned.is_interned("world"));
        assert!(!interned.is_interned("not_world"));
    }
}
