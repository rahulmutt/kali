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
fn an_f64_scalar_cell_promotes_and_a_tagged_f64_needs_a_proof() {
    // Captured-bindings A-4: an F64 cell stores the double's bits.
    let mut t = kali_common::ReprTable::default();
    t.set_scalar("f", "x", Repr::F64);
    let scalar = Widening::CapturedBindings { is_tagged: false };
    assert!(cell_is_promotable(&t, "f", "x", true, scalar));
    assert!(!cell_is_promotable(&t, "f", "x", false, TAGGED));
    t.set_numeric_bindings([("f".to_string(), "x".to_string())].into());
    assert!(cell_is_promotable(&t, "f", "x", false, TAGGED));
}

#[test]
fn baseline_widening_refuses_an_f64_cell() {
    // Followups §6 CB-1: iteration-record cells and the deferred lane keep refusing
    // F64 (spec §1.1), scalar or tagged, with or without a proof.
    let mut t = kali_common::ReprTable::default();
    t.set_scalar("f", "x", Repr::F64);
    t.set_numeric_bindings([("f".to_string(), "x".to_string())].into());
    assert!(!cell_is_promotable(&t, "f", "x", true, Widening::Baseline));
    assert!(!cell_is_promotable(&t, "f", "x", false, Widening::Baseline));
}

#[test]
fn a_non_tagged_heap_f64_cell_does_not_promote() {
    let mut t = table(false);
    t.set_scalar("f", "x", Repr::F64);
    t.set_numeric_bindings([("f".to_string(), "x".to_string())].into());
    let widening = Widening::CapturedBindings { is_tagged: false };
    assert!(!cell_is_promotable(&t, "f", "x", false, widening));
}

#[test]
fn the_scalar_and_object_verdicts_ignore_the_widening() {
    let mut t = table(false);
    t.set_scalar("f", "o", Repr::Object(kali_common::ShapeId(0)));
    for widening in [Widening::Baseline, TAGGED] {
        assert!(cell_is_promotable(&t, "f", "k", true, widening));
        assert!(cell_is_promotable(&t, "f", "o", false, widening));
    }
}

#[test]
fn a_tagged_object_cell_promotes_as_c2() {
    // Followups §6 CB-13: C2 is unchanged — MIR gives `TaggedVal` to call results,
    // `new`, member reads and identifier copies, and those promoted at
    // baseline. A member access no lane resolves is refused at the fallback.
    let mut t = table(true);
    t.set_scalar("f", "o", Repr::Object(kali_common::ShapeId(0)));
    assert!(cell_is_promotable(&t, "f", "o", false, TAGGED));
}
