use super::*;

#[test]
fn test_span_creation() {
    let file_id = FileId::new(0);
    let span = Span::new(file_id, 10, 20);

    assert_eq!(span.start(), 10);
    assert_eq!(span.end(), 20);
    assert_eq!(span.len(), 10);
    assert!(!span.is_empty());
}

#[test]
fn test_span_position() {
    let span = Span::from_position(FileId::new(0), 42);

    assert_eq!(span.start(), 42);
    assert_eq!(span.end(), 42);
    assert!(span.is_empty());
}

#[test]
fn test_span_contains() {
    let span = Span::new(FileId::new(0), 10, 20);

    assert!(span.contains(10));
    assert!(span.contains(15));
    assert!(span.contains(19));
    assert!(!span.contains(9));
    assert!(!span.contains(20));
}

#[test]
fn test_empty_span() {
    let span = Span::new(FileId::new(0), 42, 42);
    assert!(span.is_empty());
    assert_eq!(span.len(), 0);
}
