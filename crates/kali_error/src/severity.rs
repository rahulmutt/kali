//! Diagnostic severity levels.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Severity level for compiler diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error-level diagnostics prevent compilation.
    Error,
    /// Warning-level diagnostics allow compilation.
    Warning,
    /// Info-level diagnostics are informational only.
    Info,
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Semantically: Error > Warning > Info
        match (self, other) {
            (Severity::Error, Severity::Warning) => std::cmp::Ordering::Greater,
            (Severity::Error, Severity::Info) => std::cmp::Ordering::Greater,
            (Severity::Warning, Severity::Error) => std::cmp::Ordering::Less,
            (Severity::Info, Severity::Error) => std::cmp::Ordering::Less,
            (Severity::Warning, Severity::Info) => std::cmp::Ordering::Greater,
            (Severity::Info, Severity::Warning) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeverityVisitor;

        impl<'de> serde::de::Visitor<'de> for SeverityVisitor {
            type Value = Severity;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string: error, warning, or info")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Severity::from_str(value).ok_or_else(|| E::custom(format!("invalid severity: {}", value)))
            }
        }

        deserializer.deserialize_str(SeverityVisitor)
    }
}

impl Severity {
    /// Parse a severity from a string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "info" | "information" => Some(Severity::Info),
            _ => None,
        }
    }

    /// Convert to a string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str("error"), Some(Severity::Error));
        assert_eq!(Severity::from_str("warning"), Some(Severity::Warning));
        assert_eq!(Severity::from_str("info"), Some(Severity::Info));
        assert_eq!(Severity::from_str("information"), Some(Severity::Info));
        assert_eq!(Severity::from_str("invalid"), None);
    }

    #[test]
    fn test_severity_ordering() {
        // Higher severity should be greater
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Error > Severity::Info);
    }
}
