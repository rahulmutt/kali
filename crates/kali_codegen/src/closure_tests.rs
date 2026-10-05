use super::*;
use kali_common::{Repr, ReprTable};

/// `f.k` with the default `I64` repr, proven numeric when `proven`.
fn table(proven: bool) -> ReprTable {
    let mut t = ReprTable::default();
    if proven {
        t.set_numeric_bindings([("f".to_string(), "k".to_string())].into());
    }
    t
}

const TAGGED: Widening = Widening::CapturedBindings { is_tagged: true };

#[test]
fn a_tagged_cell_with_a_numeric_proof_promotes() {
    assert!(cell_is_promotable(&table(true), "f", "k", false, TAGGED));
}

#[test]
fn a_tagged_cell_without_a_proof_does_not_promote() {
    assert!(!cell_is_promotable(&table(false), "f", "k", false, TAGGED));
}

#[test]
fn baseline_widening_keeps_the_baseline_verdict() {
    assert!(!cell_is_promotable(
        &table(true),
        "f",
        "k",
        false,
        Widening::Baseline
    ));
}

#[test]
fn a_non_tagged_heap_cell_does_not_promote_even_with_a_proof() {
    // An `Array` / `Closure` cell: non-scalar, not `TaggedVal`, repr `I64`.
    let widening = Widening::CapturedBindings { is_tagged: false };
    assert!(!cell_is_promotable(&table(true), "f", "k", false, widening));
}

#[test]
fn a_tagged_f64_cell_does_not_promote() {
    let mut t = table(true);
    t.set_scalar("f", "k", Repr::F64);
    assert!(!cell_is_promotable(&t, "f", "k", false, TAGGED));
}

#[test]
fn the_scalar_and_untagged_object_verdicts_ignore_the_widening() {
    let mut t = table(false);
    t.set_scalar("f", "o", Repr::Object(kali_common::ShapeId(0)));
    for widening in [Widening::Baseline, TAGGED] {
        assert!(cell_is_promotable(&t, "f", "k", true, widening));
    }
    let untagged = Widening::CapturedBindings { is_tagged: false };
    for widening in [Widening::Baseline, untagged] {
        assert!(cell_is_promotable(&t, "f", "o", false, widening));
    }
}

#[test]
fn a_tagged_object_cell_does_not_promote() {
    // Ruling R9: a rewritten Object parameter (`let o = o{p}`) is TaggedVal.
    let mut t = table(true);
    t.set_scalar("f", "o", Repr::Object(kali_common::ShapeId(0)));
    assert!(!cell_is_promotable(&t, "f", "o", false, TAGGED));
}
