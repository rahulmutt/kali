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
