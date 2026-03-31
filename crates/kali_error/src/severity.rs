//! Diagnostic severity levels.

/// Severity level for compiler diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Error-level diagnostics prevent compilation.
    Error,
    /// Warning-level diagnostics allow compilation.
    Warning,
    /// Info-level diagnostics are informational only.
    Info,
}

impl Severity {
    /// Parse a severity from a string.
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

impl serde::Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl serde::de::DeserializeOwned for Severity {}

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
