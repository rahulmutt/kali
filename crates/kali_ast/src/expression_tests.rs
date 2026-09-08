use crate::*;

fn dot(name: &str) -> MemberExpression {
    MemberExpression {
        object: Expression::Identifier("o".to_string()),
        property: Some(name.to_string()),
        computed_index: None,
    }
}

fn computed(name: Option<&str>) -> MemberExpression {
    MemberExpression {
        object: Expression::Identifier("o".to_string()),
        property: name.map(str::to_string),
        computed_index: Some(Box::new(Expression::Identifier("i".to_string()))),
    }
}

#[test]
fn dot_access_always_has_both_names() {
    let member = dot("b");
    assert_eq!(member.dot_name(), Some("b"));
    assert_eq!(member.static_name(), Some("b"));
}

#[test]
fn a_readable_computed_index_has_a_static_name_but_no_dot_name() {
    let member = computed(Some("b"));
    assert_eq!(member.dot_name(), None);
    assert_eq!(member.static_name(), Some("b"));
}

#[test]
fn an_unreadable_computed_index_has_no_name_at_all() {
    let member = computed(None);
    assert_eq!(member.dot_name(), None);
    assert_eq!(member.static_name(), None);
}

#[test]
fn an_absent_property_field_deserializes_as_none() {
    let json = r#"{"object":{"Identifier":"o"},"computed_index":{"Identifier":"i"}}"#;
    let member: MemberExpression = serde_json::from_str(json).expect("absent property is None");
    assert_eq!(member.property, None);
}
