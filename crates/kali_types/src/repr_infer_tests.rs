use super::repr_infer::infer_reprs;
use kali_common::Repr;

fn reprs(src: &str) -> kali_common::ReprTable {
    // Use the same parse helper the other kali_types tests use to get Vec<Statement>.
    let parsed = crate::test_support::parse_statements(src);
    infer_reprs(&parsed)
}

#[test]
fn division_is_float_addition_of_ints_is_int() {
    let t = reprs("let a = 1 + 2;\nlet b = 1 / 2;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::I64);
    assert_eq!(t.scalar("_start", "b"), Repr::F64);
}

#[test]
fn float_flows_through_accumulator_reassignment() {
    // t starts int-literal 0 but accumulates a float => float throughout.
    let t = reprs("let t = 0;\nt = t + 1 / 2;\n");
    assert_eq!(t.scalar("_start", "t"), Repr::F64);
}

#[test]
fn array_element_float_from_store_and_interprocedural_param() {
    let src = "\
function store(v) { v[0] = 1 / 2; }\n\
function main() { const w = new Array(2); store(w); }\n";
    let t = reprs(src);
    // store's param v has float elements (v[0] = float).
    assert_eq!(t.array_element("store", "v"), Repr::F64);
    // w flows into store's v => w has float elements too, even though main
    // never touches w with a float op.
    assert_eq!(t.array_element("main", "w"), Repr::F64);
}

#[test]
fn function_return_repr_propagates_to_call_site() {
    let src = "\
function half(x) { return 1 / x; }\n\
function main() { let y = half(4); }\n";
    let t = reprs(src);
    assert_eq!(t.return_repr("half"), Repr::F64);
    assert_eq!(t.scalar("main", "y"), Repr::F64);
}

#[test]
fn pure_integer_program_has_empty_table() {
    let t = reprs("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\n");
    assert!(t.is_empty(), "integer-only program must record no floats");
}
