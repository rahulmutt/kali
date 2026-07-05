//! Engine-level pins for the interprocedural escape round: plain-ident
//! stores to enclosing bindings must mark `escapes` (the round-5 blindness —
//! `is_heap_store_target` covers only member/chain LHS, so `sink = p` left
//! `p.escapes == false` and callers passed heap values un-flagged).

use super::*;

#[test]
fn test_param_stored_to_module_binding_via_plain_ident_escapes() {
    let mir = analyze("let sink; function retain(p) { sink = p; }");
    let retain = mir.function("retain").expect("retain function");
    let binding = retain.binding("p").expect("p binding");

    assert_eq!(binding.kind, MirBindingKind::Parameter);
    assert!(binding.escapes);
    // Parameters keep their Borrowed ownership (finalise_binding's
    // returned/escaped arm leaves Parameter ownership untouched).
    assert_eq!(binding.ownership, OwnershipClass::Borrowed);
}

#[test]
fn test_local_stored_outward_through_alias_chain_escapes() {
    let mir = analyze(
        "let cache;
         function f() {
           const a = { v: 1 };
           const b = a;
           cache = b;
           return 0;
         }",
    );
    let f = mir.function("f").expect("f function");
    let a = f.binding("a").expect("a binding");
    let b = f.binding("b").expect("b binding");

    assert!(a.escapes);
    assert!(b.escapes);
    assert_eq!(a.ownership, OwnershipClass::OwnedHeap);
    assert_eq!(b.ownership, OwnershipClass::OwnedHeap);
}

#[test]
fn test_purely_local_binding_still_does_not_escape() {
    let mir = analyze("function f() { const o = { v: 1 }; let s = o.v; return s; }");
    let f = mir.function("f").expect("f function");
    let o = f.binding("o").expect("o binding");

    assert!(!o.escapes);
    assert_eq!(o.ownership, OwnershipClass::Stack);
}

#[test]
fn test_module_scope_bindings_are_not_blanket_flipped() {
    // Module bindings are storage roots, not escapees: the post-pass must
    // skip the module scope.
    let mir = analyze("const answer = 40 + 2;");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert!(!binding.escapes);
    assert_eq!(binding.ownership, OwnershipClass::Stack);
}
