use crate::test_support::parse;
use crate::*;
use kali_ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyName};

#[test]
fn test_object_literal_lowers_to_stable_property_shape() {
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(&Expression::ObjectExpression(ObjectExpression {
        properties: vec![ObjectProperty {
            key: PropertyName::Identifier("answer".to_string()),
            value: Expression::Identifier("value".to_string()),
            kind: ObjectPropertyKind::Init,
        }],
    }));

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);
    assert_eq!(property.text.as_deref(), Some("init"));
    assert_eq!(property.children.len(), 2);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("answer"));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}

/// The lowered program and the key node of its first object property.
///
/// Lowers `source` as a whole program so the key travels the same parser path a
/// real program's key does -- the plain-key grammar for `({3: 1})` and the
/// computed-key fold for `({[-0]: 1})`.
fn lower_first_property_key(source: &str) -> (LoweringResult, HirNodeId) {
    let statements = parse(source);
    let mut lowerer = HirLowerer::new();
    let program = lowerer.lower_statements(&statements);
    let property = program
        .nodes
        .iter()
        .position(|node| node.kind == HirNodeKind::ObjectProperty)
        .unwrap_or_else(|| panic!("no object property lowered from {source}"));
    let key = program.nodes[property].children[0];
    (program, key)
}

impl LoweringResult {
    fn node(&self, id: HirNodeId) -> &HirNode {
        &self.nodes[id.0 as usize]
    }
}

#[test]
fn numeric_object_property_names_lower_to_their_javascript_property_name() {
    // The invariant: a key-slot node's text IS `String(key)`. No quoting
    // marker, because a marker is what made `{'"5"': 1}` and `{5: 1}` the same
    // text (register R-56), and Rust's `Display` is what made `{1e-7: 1}`'s key
    // `0.0000001` instead of `1e-7`.
    for (source, expected) in [
        ("({3: 1})", "3"),
        ("({5: 1})", "5"),
        ("({1e-7: 1})", "1e-7"),
        ("({1e21: 1})", "1e+21"),
        ("({[-0]: 1})", "0"),
        ("({[-1e999]: 1})", "-Infinity"),
        // BigInt keys are digits, not doubles: `String(42n)` is `"42"`, and a
        // value with no exact `f64` keeps every digit.
        ("({42n: 1})", "42"),
        (
            "({123456789012345678901234567890n: 1})",
            "123456789012345678901234567890",
        ),
        // R-56 ITSELF. The STRING key `'"5"'` names the three characters
        // `"5"`, quotes included, and must reach codegen as exactly those --
        // never collapsed to `5`, which is what `{5: 1}` above lowers to. This
        // row and the `("({5: 1})", "5")` row are the whole entry: they are
        // distinct texts now, where the marker made them one. If anyone ever
        // re-adds marking to the `String` arm, this is the row that goes red.
        ("({'\"5\"': 1})", "\"5\""),
    ] {
        let (program, key) = lower_first_property_key(source);
        let node = program.node(key);
        // A key slot holds a Literal, not an arithmetic node: `{3: 1}`'s key is
        // the property name `3`, not the number 3 to be computed with.
        assert_eq!(node.kind, HirNodeKind::Literal, "source {source}");
        assert_eq!(node.text.as_deref(), Some(expected), "source {source}");
    }
}
