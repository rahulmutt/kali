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
    // Both the subscripted param and the caller's binding are array bindings.
    assert!(t.is_array_binding("store", "v"));
    assert!(t.is_array_binding("main", "w"));
}

#[test]
fn array_bindings_cover_int_params_pass_through_and_exclude_scalars() {
    let src = "\
function store(v) { v[0] = 7; }\n\
function get(v) { return v[0]; }\n\
function passthrough(a, b) { store(a); store(b); }\n\
function scalar(i) { return i + 1; }\n\
const u = new Array(3); store(u); get(u); passthrough(u, u); scalar(1);\n";
    let t = reprs(src);
    // Subscripted i64 array params are array bindings (element repr stays I64).
    assert!(t.is_array_binding("store", "v"));
    assert_eq!(t.array_element("store", "v"), Repr::I64);
    assert!(t.is_array_binding("get", "v"));
    // Pass-through params never subscripted directly are array bindings via the
    // transitive array-param fixpoint.
    assert!(t.is_array_binding("passthrough", "a"));
    assert!(t.is_array_binding("passthrough", "b"));
    // The top-level array binding.
    assert!(t.is_array_binding("_start", "u"));
    // A scalar param is NOT an array binding.
    assert!(!t.is_array_binding("scalar", "i"));
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

#[test]
fn spectral_norm_indices_stay_int_and_floats_propagate() {
    let src = r#"
function A(i, j) { return 1 / ((i + j) * (i + j + 1) / 2 + i + 1); }
function Au(u, v) { for (let i = 0; i < u.length; i = i + 1) { let t = 0; for (let j = 0; j < u.length; j = j + 1) { t = t + A(i, j) * u[j]; } v[i] = t; } }
function Atu(u, v) { for (let i = 0; i < u.length; i = i + 1) { let t = 0; for (let j = 0; j < u.length; j = j + 1) { t = t + A(j, i) * u[j]; } v[i] = t; } }
function AtAu(u, v, w) { Au(u, w); Atu(w, v); }
function spectralnorm(n) {
  const u = new Array(n).fill(1); const v = new Array(n); const w = new Array(n);
  for (let i = 0; i < 10; i = i + 1) { AtAu(u, v, w); AtAu(v, u, w); }
  let vBv = 0; let vv = 0;
  for (let i = 0; i < n; i = i + 1) { vBv = vBv + u[i] * v[i]; vv = vv + v[i] * v[i]; }
  return Math.sqrt(vBv / vv);
}
console.log(spectralnorm(100).toFixed(9));
"#;
    let t = infer_reprs(&crate::test_support::parse_statements(src));
    use kali_common::Repr;
    // Indices/counters MUST stay integer (float indices are unlowerable).
    for (f, b) in [
        ("A", "i"),
        ("A", "j"),
        ("Au", "i"),
        ("Au", "j"),
        ("Atu", "i"),
        ("Atu", "j"),
        ("spectralnorm", "i"),
        ("spectralnorm", "n"),
    ] {
        assert_eq!(t.scalar(f, b), Repr::I64, "scalar {f}.{b} must be I64");
    }
    // Accumulators are float.
    for (f, b) in [
        ("Au", "t"),
        ("Atu", "t"),
        ("spectralnorm", "vBv"),
        ("spectralnorm", "vv"),
    ] {
        assert_eq!(t.scalar(f, b), Repr::F64, "scalar {f}.{b} must be F64");
    }
    // Array elements are float — including the pass-through vector w.
    for (f, b) in [
        ("Au", "u"),
        ("Au", "v"),
        ("Atu", "u"),
        ("Atu", "v"),
        ("AtAu", "u"),
        ("AtAu", "v"),
        ("AtAu", "w"),
        ("spectralnorm", "u"),
        ("spectralnorm", "v"),
        ("spectralnorm", "w"),
    ] {
        assert_eq!(
            t.array_element(f, b),
            Repr::F64,
            "array elem {f}.{b} must be F64"
        );
    }
    // Returns.
    assert_eq!(t.return_repr("A"), Repr::F64);
    assert_eq!(t.return_repr("spectralnorm"), Repr::F64);
}
