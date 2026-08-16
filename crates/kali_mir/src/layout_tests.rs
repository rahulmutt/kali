use crate::test_support::*;
use crate::*;

#[test]
fn test_layout_fingerprints_are_deterministic_and_reusable() {
    let closure = LayoutDescriptor::Closure {
        captures: vec!["z".to_string(), "a".to_string()],
    };
    assert_eq!(closure.fingerprint(), "Closure(captures=a|z)");

    let structure = LayoutDescriptor::Struct {
        fields: vec![
            (
                "beta".to_string(),
                Box::new(LayoutDescriptor::scalar("number")),
            ),
            ("alpha".to_string(), Box::new(LayoutDescriptor::TaggedVal)),
        ],
    };
    assert_eq!(
        structure.fingerprint(),
        "Struct(beta:Scalar(number),alpha:TaggedVal)"
    );

    let binding = MirBinding {
        name: "value".to_string(),
        kind: MirBindingKind::Local,
        ownership: OwnershipClass::Stack,
        layout: closure,
        escapes: false,
        captured_by: Vec::new(),
    };
    assert_eq!(binding.layout_fingerprint(), "Closure(captures=a|z)");
    assert_eq!(
        binding.representation_fingerprint(),
        "ownership=stack;layout=Closure(captures=a|z)"
    );
}

#[test]
fn test_object_layout_orders_integer_like_property_keys_before_string_keys() {
    let mir = analyze("const bag = { b: 1, 2: 2, a: 3, 1: 4 };");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("bag").expect("bag binding");

    let LayoutDescriptor::Struct { fields } = &binding.layout else {
        panic!("expected struct layout, got {:?}", binding.layout);
    };

    // RE-PINNED, was right now spelled differently, FOR THE ORDERING CLAIM
    // ONLY -- which is the only claim this test makes and the only one
    // verified here. Integer-like keys still come first in ascending numeric
    // order, then string keys in insertion order. Only the field-name TEXT
    // moved: `lower_property_name` now stores a numeric key's JavaScript
    // property name (`1`) instead of marking it as "was a number" with a
    // leading double quote (`"1"`). `object_property_order_key` strips those
    // quotes before parsing, so it reads `1` exactly as it read `"1"`.
    //
    // A layout field name is NOT only an ordering key -- an interned shape's
    // field names are compared against a canonicalised probe in
    // `kali_codegen/src/intrinsics/object.rs` (`repr_table.shape_field`), so
    // this spelling could in principle move a `hasOwn` answer. Measured
    // separately, and it does not: a numeric property name never reaches the
    // shape lane at all (`kali_types/src/repr_infer.rs` refuses it with E5506
    // before a shape is interned), and for the string keys that DO intern,
    // `let o = {"1": 1, "1e+21": 2, a: 3}` mutated through a function
    // parameter answers `hasOwn` `true/true/true/true/false` for
    // `"1"`, `1`, `"1e+21"`, `1e21`, `"b"` -- identical to node.
    let field_names: Vec<_> = fields.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(field_names, vec!["1", "2", "b", "a"]);
}
