use crate::program_classes::{assigned_property_names, ProgramClasses};
use crate::test_support::parse_statements;

#[test]
fn a_class_extending_a_program_class_is_not_host_derived() {
    let classes = ProgramClasses::collect(&parse_statements(
        "class A { f(){} } class B extends A { g(){} }",
    ));
    assert!(classes.host_derived().is_empty());
    let names: Vec<_> = classes.member_names("B").unwrap().into_iter().collect();
    assert_eq!(names, ["f", "g"]);
}

#[test]
fn a_class_whose_chain_leaves_the_program_is_host_derived() {
    let classes = ProgramClasses::collect(&parse_statements(
        "class X extends EventTarget {} class Y extends X {} class Z extends mixin(A) {} class W extends ns.B {}",
    ));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["W", "X", "Y", "Z"]);
    assert_eq!(classes.member_names("Y"), None);
}

#[test]
fn nested_and_expression_classes_are_collected() {
    let classes = ProgramClasses::collect(&parse_statements(
        "function f(){ class Inner extends EventTarget {} } const K = class extends HTMLElement {};",
    ));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["Inner", "K"]);
}

#[test]
fn field_names_join_the_member_set() {
    let classes = ProgramClasses::collect(&parse_statements("class S { n = 0; f(){} }"));
    let names: Vec<_> = classes.member_names("S").unwrap().into_iter().collect();
    assert_eq!(names, ["f", "n"]);
}

#[test]
fn a_cyclic_chain_terminates_and_counts_as_host() {
    // Not valid at run time, but the walk must stop.
    let classes = ProgramClasses::collect(&parse_statements(
        "class A extends B {} class B extends A {}",
    ));
    assert!(classes.host_derived().contains("A"));
}

#[test]
fn assigned_property_names_cover_every_receiver_and_this() {
    let names = assigned_property_names(&parse_statements(
        "const o={k:1}; o.f = 1; class S { constructor(){ this.cb = 2; } } function g(x){ x.h += 1; }",
    ));
    let names: Vec<_> = names.into_iter().collect();
    assert_eq!(names, ["cb", "f", "h"]);
}

#[test]
fn a_duplicated_class_name_is_ambiguous_not_host() {
    let classes = ProgramClasses::collect(&parse_statements(
        "function f(){ class A extends EventTarget {} } class A {} class B extends A {}",
    ));
    assert!(classes.is_program_class("A"));
    assert_eq!(classes.member_names("A"), None);
    assert_eq!(classes.member_names("B"), None);
    assert!(!classes.host_derived().contains("A"));
    assert!(!classes.host_derived().contains("B"));
}

#[test]
fn a_named_class_expression_is_recorded_under_its_binding_and_its_id() {
    let classes = ProgramClasses::collect(&parse_statements(
        "const K = class Foo extends EventTarget {}; const L = class Bar { f(){} };",
    ));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["Foo", "K"]);
    // The same class under two names is not ambiguous (ruling R6).
    let names: Vec<_> = classes.member_names("L").unwrap().into_iter().collect();
    assert_eq!(names, ["f"]);
    let names: Vec<_> = classes.member_names("Bar").unwrap().into_iter().collect();
    assert_eq!(names, ["f"]);
}

#[test]
fn a_class_expression_whose_id_matches_its_binding_is_recorded_once() {
    let classes = ProgramClasses::collect(&parse_statements("const K = class K { f(){} };"));
    let names: Vec<_> = classes.member_names("K").unwrap().into_iter().collect();
    assert_eq!(names, ["f"]);
}

#[test]
fn a_member_expression_base_leaves_the_program_even_from_a_program_class() {
    // `class B extends A.Inner` used to record `A` (spec A-6).
    let classes =
        ProgramClasses::collect(&parse_statements("class A {} class B extends A.Inner {}"));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["B"]);
    assert_eq!(classes.member_names("B"), None);
}

#[test]
fn a_type_parameter_constraint_is_not_a_base() {
    let classes = ProgramClasses::collect(&parse_statements(
        "class B<T extends EventTarget> { g(){} }",
    ));
    assert!(classes.host_derived().is_empty());
    let names: Vec<_> = classes.member_names("B").unwrap().into_iter().collect();
    assert_eq!(names, ["g"]);
}

#[test]
fn a_computed_member_makes_the_member_set_unknown() {
    let classes = ProgramClasses::collect(&parse_statements(
        "class C { [\"foo\"](){ return 1; } } class D extends C { g(){} } class E { static [k] = 1; h(){} }",
    ));
    assert_eq!(classes.member_names("C"), None);
    assert_eq!(classes.member_names("D"), None);
    assert_eq!(classes.member_names("E"), None);
    assert!(classes.host_derived().is_empty());
}
