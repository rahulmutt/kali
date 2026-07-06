use crate::{Repr, ReprTable, UnionFind};

#[test]
fn union_find_propagates_float_through_union() {
    let mut uf = UnionFind::new();
    let a = uf.fresh();
    let b = uf.fresh();
    let c = uf.fresh();
    uf.seed_float(a);
    uf.union(a, b); // b joins a's float set
    assert!(uf.is_float(a));
    assert!(uf.is_float(b));
    assert!(!uf.is_float(c)); // untouched node stays int
}

#[test]
fn union_find_float_survives_union_order() {
    // Seeding one member then unioning must make the whole set float
    // regardless of which node is the resulting root.
    let mut uf = UnionFind::new();
    let x = uf.fresh();
    let y = uf.fresh();
    uf.union(x, y);
    uf.seed_float(y);
    assert!(uf.is_float(x));
    assert!(uf.is_float(y));
}

#[test]
fn repr_table_defaults_int_and_records_float() {
    let mut t = ReprTable::default();
    assert_eq!(t.scalar("f", "x"), Repr::I64);
    assert_eq!(t.array_element("f", "u"), Repr::I64);
    assert_eq!(t.return_repr("f"), Repr::I64);
    assert!(t.is_empty());
    t.set_scalar("f", "x", Repr::F64);
    t.set_array_element("f", "u", Repr::F64);
    t.set_return("f", Repr::F64);
    t.set_param("f", 0, Repr::F64);
    assert_eq!(t.scalar("f", "x"), Repr::F64);
    assert_eq!(t.array_element("f", "u"), Repr::F64);
    assert_eq!(t.return_repr("f"), Repr::F64);
    assert_eq!(t.param("f", 0), Repr::F64);
    assert!(!t.is_empty());
}

#[test]
fn repr_table_tracks_array_bindings() {
    let mut t = ReprTable::default();
    // Unset bindings default to false.
    assert!(!t.is_array_binding("f", "v"));
    // Recording a binding reports true; an unrelated one stays false.
    t.set_array_binding("f", "v");
    assert!(t.is_array_binding("f", "v"));
    assert!(!t.is_array_binding("f", "scalar"));
    assert!(!t.is_array_binding("g", "v"));
    // Additive: array bindings alone (no float) keep the table "empty".
    assert!(t.is_empty());
}

#[test]
fn shape_interning_dedupes_identical_field_lists() {
    let mut table = ReprTable::default();
    let a = table.intern_shape(vec![("x".into(), Repr::F64), ("m".into(), Repr::I64)]);
    let b = table.intern_shape(vec![("x".into(), Repr::F64), ("m".into(), Repr::I64)]);
    let c = table.intern_shape(vec![("x".into(), Repr::I64), ("m".into(), Repr::I64)]);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(table.shape_field(a, "m"), Some((1, Repr::I64)));
    assert_eq!(table.shape_field(a, "nope"), None);
    assert_eq!(table.shape_fields(a).len(), 2);
}

#[test]
fn object_entries_and_conflicts_make_the_table_non_empty() {
    let mut table = ReprTable::default();
    assert!(table.is_empty());
    let s = table.intern_shape(vec![("x".into(), Repr::I64)]);
    table.set_scalar("_start", "p", Repr::Object(s));
    assert!(!table.is_empty());
    assert_eq!(table.scalar("_start", "p"), Repr::Object(s));

    let mut conflicted = ReprTable::default();
    conflicted.add_shape_conflict("boom".into());
    assert!(!conflicted.is_empty());
    assert_eq!(conflicted.shape_conflicts(), ["boom".to_string()]);
}

#[test]
fn repr_table_records_string_and_is_non_empty() {
    let mut t = ReprTable::default();
    assert!(t.is_empty());
    t.set_scalar("_start", "s", Repr::String);
    assert_eq!(t.scalar("_start", "s"), Repr::String);
    assert!(!t.is_empty(), "a string decision makes the table non-empty");
    // A string decision must not spuriously mark the program as containing floats.
    let mut t2 = ReprTable::default();
    t2.set_return("f", Repr::String);
    assert_eq!(t2.return_repr("f"), Repr::String);
}

#[test]
fn repr_table_records_non_ascii_string_provenance() {
    let mut t = ReprTable::default();
    assert!(!t.is_string_non_ascii("_start", "s"));
    t.mark_string_non_ascii("_start", "s");
    assert!(t.is_string_non_ascii("_start", "s"));
    assert!(!t.is_string_non_ascii("_start", "other"));

    assert!(!t.is_string_non_ascii_return("f"));
    t.mark_string_non_ascii_return("f");
    assert!(t.is_string_non_ascii_return("f"));
}
