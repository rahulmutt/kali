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
