use crate::*;

#[test]
fn test_node_id() {
    let id = NodeId::new(42);
    assert_eq!(id.as_u32(), 42);
    assert_eq!(id.to_string(), "n42");
}
