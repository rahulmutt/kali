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
