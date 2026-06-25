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

    let field_names: Vec<_> = fields.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(field_names, vec!["\"1\"", "\"2\"", "b", "a"]);
}
