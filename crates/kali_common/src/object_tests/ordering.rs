use crate::object::{property_order_key, sort_properties_es_order};

#[test]
fn property_order_key_classifies_array_index_like_keys() {
    assert_eq!(property_order_key("0"), Some(0));
    assert_eq!(property_order_key("1"), Some(1));
    assert_eq!(property_order_key("\"2\""), Some(2)); // LIR text may keep quotes
    assert_eq!(property_order_key("01"), None); // leading zero: not an index
    assert_eq!(property_order_key(""), None);
    assert_eq!(property_order_key("b"), None);
    assert_eq!(property_order_key("4294967295"), None); // == 2^32-1: not an index
}

#[test]
fn sort_properties_es_order_matches_node_enumeration_order() {
    // node: Object.keys({ "b": 1, "2": 2, "a": 3, "1": 4 }) => ['1','2','b','a']
    let mut props = vec![
        ("b".to_string(), 1),
        ("2".to_string(), 2),
        ("a".to_string(), 3),
        ("1".to_string(), 4),
    ];
    sort_properties_es_order(&mut props);
    let keys: Vec<&str> = props.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["1", "2", "b", "a"]);
    let values: Vec<i32> = props.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, vec![4, 2, 1, 3]);
}

#[test]
fn sort_properties_es_order_is_stable_for_string_keys() {
    let mut props = vec![
        ("z".to_string(), 0),
        ("a".to_string(), 1),
        ("m".to_string(), 2),
    ];
    sort_properties_es_order(&mut props);
    let keys: Vec<&str> = props.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["z", "a", "m"]); // insertion order, NOT alphabetical
}
