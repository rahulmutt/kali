use crate::*;

#[test]
fn test_hir_builder() {
    let mut builder = HirBuilder::new();
    let root = builder.alloc(HirNodeKind::Program, None);
    assert_eq!(root.0, 0);
    assert_eq!(builder.next_id.0, 1);
}
