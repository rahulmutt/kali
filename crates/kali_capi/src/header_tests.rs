use crate::*;

#[test]
fn header_generation_produces_c_compatible_prototypes() {
    let header = generate_header("lib", &[Export::new("add", 2), Export::new("1bad-name", 0)]);

    assert!(header.contains("#include <stdint.h>"));
    assert!(header.contains("extern int32_t add(int32_t arg0, int32_t arg1);"));
    assert!(header.contains("extern int32_t _1bad_name(void);"));
}

#[test]
fn identifier_sanitization_is_deterministic() {
    assert_eq!(sanitize_identifier("foo-bar"), "foo_bar");
    assert_eq!(sanitize_identifier("1foo"), "_1foo");
    assert_eq!(sanitize_identifier(""), "_");
}
