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

#[test]
fn written_object_literal_binding_gets_a_shape() {
    let t = reprs("const p = { x: 1.5, y: 2 };\np.x = p.x + 1.0;\nconsole.log(p.y);\n");
    let Repr::Object(shape) = t.scalar("_start", "p") else {
        panic!("p should be an object binding");
    };
    assert_eq!(t.shape_field(shape, "x"), Some((0, Repr::F64)));
    assert_eq!(t.shape_field(shape, "y"), Some((1, Repr::I64)));
}

#[test]
fn read_only_local_object_literal_stays_on_the_fold_lane() {
    let t = reprs("const p = { x: 1.5 };\nconsole.log(p.x);\n");
    assert_eq!(t.scalar("_start", "p"), Repr::I64); // no entry == fold lane
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn field_float_flows_to_reader_binding() {
    let t = reprs("const p = { x: 1 };\np.x = 2.5;\nconst d = p.x;\n");
    assert_eq!(t.scalar("_start", "d"), Repr::F64);
}

#[test]
fn array_of_objects_shares_shape_across_factory_param_and_alias() {
    let src = "\
function mk(v) { return { x: v, m: 1.5 }; }\n\
function bump(arr) { const b = arr[0]; b.x = b.x + arr[1].m; }\n\
const bodies = [mk(1.0), mk(2.0)];\n\
bump(bodies);\n";
    let t = reprs(src);
    let Repr::Object(elem) = t.array_element("_start", "bodies") else {
        panic!("bodies elements should be objects");
    };
    assert_eq!(t.array_element("bump", "arr"), Repr::Object(elem));
    assert_eq!(t.return_repr("mk"), Repr::Object(elem));
    assert_eq!(t.scalar("bump", "b"), Repr::Object(elem));
    assert_eq!(t.param("bump", 0), Repr::Object(elem));
    assert_eq!(t.shape_field(elem, "x"), Some((0, Repr::F64)));
    assert!(t.is_array_binding("_start", "bodies"));
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn quoted_string_keys_materialize_the_same_shape_as_identifier_keys() {
    // F-Stage1-4: `{ "b": 1, "a": 2 }` previously recorded a deferred
    // "non-identifier property name" conflict and never materialized a
    // shape; the byte-identical program with unquoted keys worked. Quoted
    // and unquoted keys are the same object in JS.
    // Field order is ES enumeration order: array-index-like keys first,
    // ascending; then insertion order.
    // { "b": 1, "2": 2, "a": 3, "1": 4 } -> fields ["1", "2", "b", "a"]
    let t = reprs(
        "const o = { \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 };\nfor (var k in o) { console.log(k); }\n",
    );
    let Repr::Object(shape) = t.scalar("_start", "o") else {
        panic!("o should be an object binding with a materialized shape");
    };
    let names: Vec<&str> = t
        .shape_fields(shape)
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(names, vec!["1", "2", "b", "a"]);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn quoted_proto_key_object_literal_fails_closed_not_a_phantom_own_property() {
    // CRITICAL (Stage 2 Lane A review): an object-literal `__proto__` key
    // (identifier OR string-literal form, non-computed) is JS's PROTOTYPE
    // SETTER — it creates NO own property. node's `for..in` on
    // `{ "__proto__": 1, "a": 2 }` prints only `a`. kali has no prototype
    // model, so materializing `__proto__` as an ordinary field and
    // enumerating it would be a miscompile. Must fail closed (a deferred
    // conflict, promoted to a real conflict by the `for..in`) rather than
    // ever materialize a shape.
    let t =
        reprs("const o = { \"__proto__\": 1, \"a\": 2 };\nfor (var k in o) { console.log(k); }\n");
    assert!(
        !matches!(t.scalar("_start", "o"), Repr::Object(_)),
        "a __proto__-keyed literal must never materialize a shape"
    );
    assert!(
        !t.shape_conflicts().is_empty(),
        "a __proto__-keyed literal must record a conflict instead of enumerating a phantom own key"
    );
}

#[test]
fn identifier_proto_key_object_literal_fails_closed_not_a_phantom_own_property() {
    // Same as above but the identifier form `{ __proto__: 1, a: 2 }`. This
    // form was ALREADY a miscompile before Task 3 (pre-existing) — both
    // forms share the same key-admission choke point, so this fix closes it
    // too.
    let t = reprs("const o = { __proto__: 1, a: 2 };\nfor (var k in o) { console.log(k); }\n");
    assert!(
        !matches!(t.scalar("_start", "o"), Repr::Object(_)),
        "a __proto__-keyed literal must never materialize a shape"
    );
    assert!(
        !t.shape_conflicts().is_empty(),
        "a __proto__-keyed literal must record a conflict instead of enumerating a phantom own key"
    );
}

#[test]
fn shape_mismatch_reassignment_is_a_conflict() {
    let t = reprs("let p = { x: 1.0 };\np = { y: 2.0 };\np.y = 3.0;\n");
    assert!(!t.shape_conflicts().is_empty());
}

#[test]
fn unknown_field_access_is_a_conflict() {
    let t = reprs("const p = { x: 1.0 };\np.x = 2.0;\np.z = 1.0;\n");
    assert!(t.shape_conflicts().iter().any(|m| m.contains("'z'")));
}

#[test]
fn object_literal_as_direct_call_argument_is_a_conflict() {
    let t = reprs("function f(o) { return o.x; }\nf({ x: 1.0 });\n");
    assert!(!t.shape_conflicts().is_empty());
}

#[test]
fn float_and_array_programs_gain_no_shapes() {
    let t = reprs("function f(a) { a[0] = 1 / 2; }\nconst w = new Array(2);\nf(w);\n");
    assert!(t.shape_conflicts().is_empty());
    assert_eq!(t.array_element("_start", "w"), Repr::F64);
}

// ---- Review fix: promotion-hole regression tests -----------------------
//
// Task 3 deferred *structural* object-literal conflicts (non-identifier
// property key / getter-setter / nested object) into `obj_pending_conflicts`
// so read-only fold-lane literals (consumed only by `Object.keys`-style
// builtins) keep compiling. Promotion of a pending conflict was originally
// tied to the slot ACQUIRING A FIELD LIST — but a purely-structural literal
// never gets one, so several materialization paths (a local field write, a
// factory-return then write, an array-of-objects element write across a
// function boundary, and an object passed as a call argument then
// field-read in the callee) never promoted their pending conflict. The
// object silently fell through to the pre-existing "fold lane" object
// codegen, which is demonstrably wrong for these four shapes (confirmed
// against node): it either ignores writes and folds a `.field` read back to
// the literal's own declared value (or `0` if the field doesn't appear in
// the literal at all), or has no visibility into the original literal text
// once the read crosses a function boundary. Each test below is one of the
// four confirmed-escaping paths and must now be rejected (a real shape
// conflict) instead of silently compiling.

#[test]
fn local_field_write_on_structural_literal_is_a_conflict() {
    // node: `p.c = 2; console.log(p.c)` prints `2`. Before the fix, kali
    // silently compiled this (no conflict) and the buggy fold-lane codegen
    // printed `0`, ignoring the write.
    let t = reprs("const p = {\"a-b\": 1};\np.c = 2;\nconsole.log(p.c);\n");
    assert!(
        !t.shape_conflicts().is_empty(),
        "a written-then-read structurally-unsupported literal must be a shape conflict"
    );
}

#[test]
fn factory_return_then_write_on_structural_literal_is_a_conflict() {
    // node: `result.c` (after `result.c = 2`) is `2`. Before the fix, kali
    // never promoted the pending conflict on the factory's `Return` slot
    // because it never acquired a field list, and the buggy fold lane
    // printed `0`.
    let t = reprs(
        "function mk() { return {\"a-b\": 1}; }\nconst result = mk();\nresult.c = 2;\nconsole.log(result.c);\n",
    );
    assert!(
        !t.shape_conflicts().is_empty(),
        "a factory returning a structurally-unsupported literal, written and read by the caller, must be a shape conflict"
    );
}

#[test]
fn array_of_objects_element_write_across_boundary_on_structural_literal_is_a_conflict() {
    // node: `bodies[0].c` (after `bump` writes it) is `9`. Before the fix,
    // the pending conflict on the array-element slot was never promoted
    // (the write happens on a *different*, flow-connected `ArrayElem` slot
    // inside `bump`), and the buggy fold lane printed `0`.
    let t = reprs(
        "function bump(arr) { arr[0].c = 9; }\nconst bodies = [{\"a-b\": 1}];\nbump(bodies);\nconsole.log(bodies[0].c);\n",
    );
    assert!(
        !t.shape_conflicts().is_empty(),
        "an array-of-structurally-unsupported-objects element written across a function boundary must be a shape conflict"
    );
}

#[test]
fn structural_literal_as_call_argument_field_read_in_callee_is_a_conflict() {
    // node: `f(o)` is `undefined` (`o` has no `.c` field). Before the fix,
    // this purely-read-only-but-cross-function case never promoted the
    // pending conflict on the caller's binding, and the buggy fold lane
    // printed `0` instead of the (would-be) `undefined`.
    let t = reprs("function f(o) { return o.c; }\nconst o = {\"a-b\": 1};\nconsole.log(f(o));\n");
    assert!(
        !t.shape_conflicts().is_empty(),
        "a structurally-unsupported literal passed as a call argument (via a const binding) and field-read in the callee must be a shape conflict"
    );
}

#[test]
fn unknown_field_read_gate_is_fold_first_but_rejects_once_materialized() {
    // Read-only: `{x: 1.0}` is never written or aliased, so an unknown-field
    // read on it must stay on the fold lane (matches node's `undefined`)
    // rather than reject — this is the same fold-first contract as
    // `read_only_local_object_literal_stays_on_the_fold_lane`, extended to
    // an *unknown*-field read.
    let read_only = reprs("const p = { x: 1.0 };\nconsole.log(p.y);\n");
    assert!(
        read_only.shape_conflicts().is_empty(),
        "a read-only unknown-field access on a non-materialized literal must not conflict"
    );

    // Materialized: the same shape, but written first — now genuinely
    // escapes the fold lane, so the unknown-field read must still conflict.
    let materialized = reprs("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p.y);\n");
    assert!(
        !materialized.shape_conflicts().is_empty(),
        "an unknown-field access on a materialized (written) object must still conflict"
    );
}

#[test]
fn object_enumeration_delete_reinsert_style_literal_stays_on_the_fold_lane() {
    // Mirrors `object-enumeration-delete-reinsert-benchmark-v1`: a
    // structurally-unsupported literal that is only written-to (and
    // "deleted" from, which the parser today folds into a bare, unobserved
    // member-read statement) and consumed via `Object.keys`-style
    // enumeration builtins — never through a genuine `.field` read — must
    // stay on the fold lane (fold-first), exactly like it does today.
    // Regression guard for the promotion-hole fix: promoting on ANY write,
    // unconditionally, would wrongly reject this.
    let t = reprs(
        "function hot(seed) {\n  const literal = { 1: 4, 2: 2, b: 1 };\n  delete literal.b;\n  literal.b = 3;\n  return seed;\n}\nhot(0);\n",
    );
    assert!(
        t.shape_conflicts().is_empty(),
        "a write-only structural literal never observed through a genuine field read must not conflict: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn string_literal_binding_is_string_repr() {
    let t = reprs("let s = \"hi\";\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
}

#[test]
fn string_flows_through_concat_reassignment() {
    // a starts as a string literal and accumulates string concatenations.
    let t = reprs("let a = \"\";\na = a + \"y\";\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
}

#[test]
fn string_flows_through_param_and_return() {
    let src = "\
function f(s) { return s + \"!\"; }\n\
let out = f(\"hi\");\n";
    let t = reprs(src);
    assert_eq!(t.param("f", 0), Repr::String);
    assert_eq!(t.return_repr("f"), Repr::String);
    assert_eq!(t.scalar("_start", "out"), Repr::String);
}

#[test]
fn plain_integer_program_has_no_string_repr() {
    let t = reprs("let a = 1 + 2;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::I64);
    assert!(t.is_empty());
}

#[test]
fn unary_plus_over_a_string_typed_operand_solves_numeric_not_string() {
    // fasta Spec 5 Task 6: unary `+` coerces a runtime string to a number
    // (codegen's `emit_string_to_i64_parse`). `s` is a genuine string-typed
    // binding (a string literal), so this is the general shape the
    // `add_edge_float_only` fix in `repr_infer.rs`'s `UnaryExpression` arm
    // guards, not just the `process.argv` special case (which never seeded
    // the string axis in the first place). Before that fix, `n` would
    // incorrectly solve `Repr::String` (the full `add_edge` used for `-`
    // carries the string axis too), which would make codegen's
    // `is_string_valued` misclassify every later read of `n` as a live
    // string handle even though `n`'s local actually holds the coerced
    // integer — a miscompile once codegen accepts `+` on a string operand.
    let t = reprs("let s = \"5\";\nlet n = +s;\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
    assert_eq!(t.scalar("_start", "n"), Repr::I64);
}

#[test]
fn unary_plus_over_process_argv_element_solves_numeric() {
    // Sibling of the above, pinning the actual fasta shape: `process.argv[i]`
    // is a proven runtime string at codegen (`is_string_valued`), but was
    // never added to `repr_infer`'s string-seed set (only the taint-candidate
    // list) — so `n` was already `I64` here even before the `+` fix above.
    // Pinned together so a future change to the argv element's seeding can't
    // silently regress this without a failing test.
    let t = reprs("let n = +process.argv[2];\n");
    assert_eq!(t.scalar("_start", "n"), Repr::I64);
}

#[test]
fn unary_minus_over_a_string_typed_operand_still_solves_string_repr_axis() {
    // Fail-closed pin (does NOT change behavior): unary `-` keeps the FULL
    // `add_edge` (both axes), unlike `+` above. This is safe only because
    // codegen's OWN `is_string_valued` guard (operators.rs) unconditionally
    // rejects `-` over any string-valued operand at emission time,
    // independent of this repr solve — so this node's `Repr::String`
    // classification is never actually consumed to emit anything. This test
    // pins that `-` was NOT touched by the Task 6 narrowing: the general
    // string-flow edge for `-` behaves exactly as it did before this task.
    let t = reprs("let s = \"5\";\nlet n = -s;\n");
    assert_eq!(t.scalar("_start", "n"), Repr::String);
}

#[test]
fn mixed_literal_int_and_string_store_is_element_conflict() {
    // Spec 1 pinned s == I64 here via the element-read string-axis exclusion.
    // Spec 3 lifts that exclusion (stores are gated + mixed arrays conflict),
    // so this launder shape now fails closed instead of reading back an int.
    let t = reprs("let a = [1];\nlet s = a[0];\na[0] = \"x\";\n");
    assert!(t
        .shape_conflicts()
        .iter()
        .any(|m| m.contains("elements of `a`")));
}

#[test]
fn string_stores_prove_string_element_axis() {
    let t = reprs("function f(s) { const a = new Array(2); a[0] = s.substring(0, 1); a[1] = \"x\"; }\nf(\"hey\");\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn mixed_string_and_number_element_stores_conflict() {
    let t = reprs("const a = new Array(2);\na[0] = \"x\";\na[1] = 1;\n");
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("elements of `a`")),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn element_read_of_string_element_array_is_string() {
    let t = reprs("const a = new Array(1);\na[0] = \"x\";\nlet s = a[0];\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
}

#[test]
fn non_ascii_element_store_marks_element_non_ascii() {
    let t = reprs("const a = new Array(1);\na[0] = \"héllo\";\n");
    assert!(t.is_array_element_non_ascii("_start", "a"));
}

#[test]
fn concat_store_marks_element_tainted_but_literal_store_does_not() {
    let t = reprs("function f(s) { const a = new Array(1); a[0] = s + \"y\"; }\nf(\"x\");\nconst b = new Array(1);\nb[0] = \"z\";\n");
    assert!(t.is_array_element_concat_tainted("f", "a"));
    assert!(!t.is_array_element_concat_tainted("_start", "b"));
}

#[test]
fn array_alloc_reassignment_merges_element_axes() {
    let t = reprs("function f(n) { let a = new Array(60); if (n < 60) { a = new Array(n); } a[0] = \"x\"; }\nf(3);\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn string_element_array_flows_through_param() {
    let t = reprs("function g(q) { q[0] = \"x\"; }\nfunction f() { const a = new Array(1); g(a); let s = a[0]; }\nf();\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
}

#[test]
fn float_still_flows_through_element_read() {
    // The float axis KEEPS element-read edges: a scalar capturing an f64
    // element read is still `Repr::F64` (Finding 2 excludes only the STRING
    // axis). Companion to `mixed_literal_int_and_string_store_is_element_conflict`; the existing
    // `array_element_float_from_store_and_interprocedural_param` pins the
    // array-element side.
    let t = reprs("let a = [1.5];\nlet s = a[0];\n");
    assert_eq!(t.scalar("_start", "s"), Repr::F64);
}

#[test]
fn concat_derived_string_is_tainted_but_literal_is_not() {
    // Finding 1: a runtime-concat-derived string is tainted (its fresh handle
    // may not be identity-compared); a literal-rooted string is interned and
    // NOT tainted.
    let t = reprs("let s = \"hi\";\nlet a = \"x\";\nlet b = a + \"y\";\n");
    assert_eq!(t.scalar("_start", "b"), Repr::String);
    assert!(
        t.is_string_concat_tainted("_start", "b"),
        "a concat result must be tainted"
    );
    assert_eq!(t.scalar("_start", "s"), Repr::String);
    assert!(
        !t.is_string_concat_tainted("_start", "s"),
        "an interned literal string must NOT be tainted"
    );
}

#[test]
fn interpolated_template_result_is_tainted() {
    // An interpolated template lowers to runtime concatenation (a fresh handle).
    let t = reprs("let n = 5;\nlet x = `a${n}`;\n");
    assert_eq!(t.scalar("_start", "x"), Repr::String);
    assert!(t.is_string_concat_tainted("_start", "x"));
}

#[test]
fn module_scope_string_number_conflict_message_reads_at_module_scope() {
    // Minor: a top-level binding conflict renders "at module scope", not the
    // synthetic `_start`.
    let t = reprs("let x = \"a\";\nx = 5;\n");
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("at module scope") && !m.contains("_start")),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn call_result_argument_seeds_callee_param_object_shape() {
    // No bound-identifier call site anywhere: `check`'s param `t` must get
    // its object shape from the call-result argument `mk()` itself.
    let t = reprs(
        r#"function mk() {
  return { left: null, right: null };
}
function check(t) {
  if (t.left === null) { return 1; }
  return 2;
}
function main() {
  console.log(check(mk()));
}
main();
"#,
    );
    assert!(
        matches!(t.param("check", 0), Repr::Object(_)),
        "param must receive the object shape from the call-result argument"
    );
}

#[test]
fn resolution_result_carries_string_reprs() {
    // End-to-end through the resolver (not infer_reprs directly): the reordered
    // table must reach ResolutionResult unchanged.
    let parsed = crate::test_support::parse_statements("let s = \"hi\";\n");
    let mut ctx = crate::context::TypeContext::default();
    let result = ctx.resolve_statements_at_path(None::<&std::path::Path>, &parsed);
    assert_eq!(result.repr_table.scalar("_start", "s"), Repr::String);
}
#[test]
fn uncalled_string_concat_return_is_string_without_conflict() {
    // `string-concatenation-benchmark-v1.ts` shape: the return expression is a
    // `+` rooted in string literals, so the (single) return in-edge is itself
    // string-reachable — proven `Repr::String`, no conflict, even though the
    // function is never called (its param stays unproven I64).
    let t = reprs(
        "function dead0(value) { return (\"ka\" + \"li\") + value; }\nfunction hot(prefix, suffix) {\n  return prefix + ((\"a\" + \"head\") + (\"-\" + \"of\") + (\"-\" + \"time\")) + suffix;\n}\nhot(\"start-\", \"-end\");\n",
    );
    assert!(
        t.shape_conflicts().is_empty(),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
    assert_eq!(t.return_repr("dead0"), Repr::String);
}

#[test]
fn mixed_string_and_plain_returns_downgrade_to_i64_without_conflict() {
    // `template-literal-concatenation-benchmark-v1.ts` shape: one return is a
    // template literal (string seed), the other returns the unproven param
    // (plain). The repr axis cannot claim `Repr::String` (a call site could
    // receive a raw int), but it must NOT hard-reject either — the function
    // stays on the pre-string-flow I64 lane (codegen and the E3200 gate both
    // treat the call result as non-string, exactly the pre-existing behavior).
    let t = reprs(
        "function dead0(value) {\n  if (false) {\n    return `ka${\"li\"}${value}`;\n  }\n  return value;\n}\n",
    );
    assert!(
        t.shape_conflicts().is_empty(),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
    assert_eq!(t.return_repr("dead0"), Repr::I64);
}

#[test]
fn consumed_mixed_return_captured_by_scalar_is_a_conflict() {
    // Finding 1: `g` mixes a string return with a plain (unproven-int) return,
    // so it downgrades to I64 — but the call result is CAPTURED by `r`, and
    // string-reachability flows return -> call-result -> `r`, which would then
    // classify `Repr::String` over a runtime int. Codegen materialises the call
    // as a raw i64 (return_repr != String), so this MUST fail closed.
    let t =
        reprs("function g(v, k) { if (k > 0) { return \"yes\"; } return v; }\nlet r = g(99, 0);\n");
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("both a string and a number")),
        "consumed mixed return must conflict; conflicts: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn never_called_mixed_return_captured_nowhere_is_not_a_conflict() {
    // The other side of Finding 1: the SAME mixed-return shape, but the function
    // is never called (no call-result node exists), so no scalar is string-
    // tainted and nothing miscompiles. The return simply downgrades to I64 with
    // NO conflict — the `kali check`-only benchmark fixtures depend on this.
    let t = reprs("function g(v, k) { if (k > 0) { return \"yes\"; } return v; }\n");
    assert!(
        t.shape_conflicts().is_empty(),
        "never-called mixed return must not conflict; conflicts: {:?}",
        t.shape_conflicts()
    );
    assert_eq!(t.return_repr("g"), Repr::I64);
}

#[test]
fn string_then_plain_reassignment_is_a_conflict() {
    // A BINDING that is string-reachable and also directly written with a
    // plain (non-string, non-float) value cannot get one runtime repr: with
    // codegen now trusting `Repr::String` for identifier reads, claiming
    // String would read the raw integer as a string handle. Unlike the
    // mixed-RETURN case above (downgraded, never previously miscompiled at
    // call sites), a scalar downgrade would silently print through the old
    // int lane, so this fails closed as a shape conflict instead.
    let t = reprs("let x = \"a\";\nx = 5;\n");
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("both a string and a number")),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn substring_result_binding_is_string_and_tainted() {
    let t = reprs("let a = \"GGCC\";\nlet s = a.substring(1, 3);\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
    assert!(
        t.is_string_concat_tainted("_start", "s"),
        "a runtime substring result is a non-interned string and must be concat-tainted"
    );
    assert!(!t.is_string_non_ascii("_start", "s"));
}

#[test]
fn bound_join_result_is_string_and_tainted() {
    let t = reprs("const a = new Array(1);\na[0] = \"x\";\nconst j = a.join(\"\");\n");
    assert_eq!(t.scalar("_start", "j"), Repr::String);
    assert!(t.is_string_concat_tainted("_start", "j"));
}

#[test]
fn param_that_is_both_length_and_substring_receiver_is_string() {
    // Task 10 (commit 9efba347d) `resolve_calls` independence pin, at the unit
    // level: the fastaRepeat shape passes a bare string identifier (`ALU`) to a
    // param (`seq`) that is BOTH a `.length` receiver and a `.substring`
    // receiver. Repr inference must prove `seq: Repr::String` from the string
    // argument flowing in — independent of resolution order.
    let src = "\
function f(seq) { if (seq.length > 0) { return seq.substring(0, 1); } return seq; }\n\
const ALU = \"GGCC\";\n\
let out = f(ALU);\n";
    let t = reprs(src);
    assert_eq!(t.scalar("f", "seq"), Repr::String);
    assert_eq!(t.return_repr("f"), Repr::String);
    assert_eq!(t.scalar("_start", "out"), Repr::String);
}

#[test]
fn substring_flows_through_param_and_return() {
    let src = "\
function f(seq) { return seq.substring(0, 2); }\n\
let out = f(\"GGCC\");\n";
    let t = reprs(src);
    assert_eq!(t.return_repr("f"), Repr::String);
    assert_eq!(t.scalar("_start", "out"), Repr::String);
}

#[test]
fn non_ascii_literal_marks_non_ascii_through_flow() {
    let t = reprs("let a = \"héllo\";\nlet b = a + \"!\";\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
    assert!(t.is_string_non_ascii("_start", "a"));
    assert!(
        t.is_string_non_ascii("_start", "b"),
        "non-ASCII propagates through +"
    );
}

#[test]
fn ascii_only_flow_is_not_marked_non_ascii() {
    let t = reprs("let a = \"GG\" + \"CC\";\nlet b = a + 5;\n");
    assert!(!t.is_string_non_ascii("_start", "a"));
    assert!(!t.is_string_non_ascii("_start", "b"));
}

#[test]
fn non_ascii_interpolated_string_propagates_through_template() {
    // The parser desugars `x${s}y` into a `+` chain BEFORE repr_infer runs
    // (kali_parser's desugar_template_literal), so the non-ASCII mark on `s`
    // flows through real `+` value-flow edges into `a`.
    let t = reprs("let s = \"héllo\";\nlet a = `x${s}y`;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
    assert!(t.is_string_non_ascii("_start", "a"));
}

#[test]
fn non_ascii_template_quasi_marks_non_ascii() {
    // After desugaring, the non-ASCII quasi chunk is a plain non-ASCII
    // string Literal seed feeding the `+` chain.
    let t = reprs("let n = 3;\nlet a = `héllo${n}`;\n");
    assert!(t.is_string_non_ascii("_start", "a"));
}

#[test]
fn numeric_interpolation_is_ascii_precise() {
    // Decision (controller-approved deviation from the task brief): the
    // parser desugars interpolations into real `+` value-flow edges, so
    // interpolated contents ARE modeled by repr_infer — the brief's premise
    // that they are unprovable does not hold for real parsed source. Every
    // non-string primitive (number/bool/null) stringifies to ASCII, and a
    // string interpoland propagates its own non-ASCII mark through the `+`
    // edge (see the two tests above) — precise, not fail-open.
    let t = reprs("let n = 3;\nlet a = `x${n}y`;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
    assert!(!t.is_string_non_ascii("_start", "a"));
}

#[test]
fn for_in_key_is_seeded_and_not_a_string_repr_by_default() {
    // The key binding exists after inference and defaults to a scalar repr
    // (I64 ordinal) until a string-use lifts it. This pins that seeding the
    // key node did not accidentally make it F64 or a shape.
    let t =
        reprs("function m(table) { for (var c in table) { let z = c; } }\nm({ a: 1, c: 2 });\n");
    assert_eq!(t.scalar("m", "c"), Repr::I64);
}

// ---- throw-fallout Stage 4: growable-array promotion gate ----

#[test]
fn growable_promotion_fires_for_safe_numeric_push_bindings() {
    let t = reprs(
        "function main() { const o = []; o.push(1); o.push(2); \
         console.log(o.length); console.log(o[0]); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "o"));
}

#[test]
fn growable_promotion_accepts_identifier_and_arithmetic_pushes() {
    let t = reprs(
        "function main() { const o = []; \
         for (let i = 0; i < 10; i++) { o.push(i * 2); } \
         for (const item of [1, 2]) { o.push(item); } \
         console.log(o.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "o"));
}

#[test]
fn growable_promotion_blocks_non_i64_pushes() {
    // Float push.
    let t =
        reprs("function main() { const o = []; o.push(1.5); console.log(o.length); }\nmain();\n");
    assert!(!t.is_growable_array_binding("main", "o"));
    // Float-solved identifier push.
    let t = reprs(
        "function main() { const f = 1 / 2; const o = []; o.push(f); console.log(o.length); }\nmain();\n",
    );
    assert!(!t.is_growable_array_binding("main", "o"));
    // Undeclared identifier push (`undefined` has no i64 value).
    let t = reprs(
        "function main() { const o = []; o.push(undefined); console.log(o.length); }\nmain();\n",
    );
    assert!(!t.is_growable_array_binding("main", "o"));
}

#[test]
fn growable_promotion_promotes_uniform_string_pushes() {
    // Task 3: a uniform-String push set promotes, with the element axis
    // solving `Repr::String` (deliberate flip of the pre-Task-3 pin above,
    // which used to assert a string push blocks promotion — see the Task 3
    // report for the recorded intent).
    let t = reprs(
        "function main() { const o = []; o.push(\"a\"); o.push(\"b\"); \
         console.log(o[0]); console.log(o.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "o"));
    assert_eq!(t.array_element("main", "o"), Repr::String);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn growable_promotion_accepts_string_identifier_pushes() {
    // A declared (non-function/array/object/for-in-key) string-valued
    // identifier push is allowed — the Task 2 identifier guard is
    // repr-agnostic and stays intact for the String lane too.
    let t = reprs(
        "function main() { const s = \"x\"; const o = []; o.push(s); \
         console.log(o.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "o"));
    assert_eq!(t.array_element("main", "o"), Repr::String);
}

#[test]
fn growable_promotion_rejects_mixed_i64_and_string_pushes() {
    // Task 3 fail-closed requirement: a MIXED i64+String push set on the
    // SAME growable candidate must not silently fall back to the
    // pre-promotion no-op lane — it is a shape conflict (E5506), mirroring
    // the pre-existing mixed-store rejection idiom for ordinary array
    // element stores.
    let t = reprs(
        "function main() { const o = []; o.push(1); o.push(\"a\"); \
         console.log(o.length); }\nmain();\n",
    );
    assert!(
        !t.shape_conflicts().is_empty(),
        "expected a shape conflict for a mixed i64/String push set"
    );
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("used as both strings and numbers")),
        "shape_conflicts: {:?}",
        t.shape_conflicts()
    );
    assert!(!t.is_growable_array_binding("main", "o"));
}

#[test]
fn growable_promotion_blocks_escaping_and_module_scope_bindings() {
    // Escaping (call argument) — not a candidate.
    let t = reprs(
        "function f(x) { return x; }\nfunction main() { const o = []; o.push(1); f(o); }\nmain();\n",
    );
    assert!(!t.is_growable_array_binding("main", "o"));
    // Module-scope push receiver — deliberately not analyzed.
    let t = reprs("const o = [];\no.push(1);\nconsole.log(o.length);\n");
    assert!(!t.is_growable_array_binding("_start", "o"));
}

// ---- F-AB-2 lockstep tripwire ----------------------------------------------
//
// These pin the two `__kali_fn_N` sets the shared Phase-A descent (walks 1-3)
// and Phase B (walk 4) build. The product code carries a hard
// `debug_assert!(seeded ⊆ registered)` in `assert_nested_fn_lockstep`; these
// tests pin BOTH directions — that common callback positions are in exact
// lockstep (seeded == registered) and that the KNOWN-exotic positions form the
// documented, allowed reverse gap `registered − seeded`. See
// `docs/superpowers/followups/stageAB-followups.md` §F-AB-2. (A named function
// expression, e.g. `function cb(){…}`, stands in for the synthetic
// `__kali_fn_N` id that `name_anon_functions` assigns in the real pipeline; the
// lockstep logic keys purely on the fn-expr/arrow id, so the source of the name
// is immaterial.)

use super::repr_infer::nested_fn_lockstep_sets;

#[test]
fn nested_fn_lockstep_common_positions_are_equal() {
    // A fn-expr in a common position (declarator init) is reached by BOTH the
    // shared Phase-A descent AND Phase B's own walk-4 fn-expr arm → the sets
    // are EQUAL. This is the invariant the debug_assert protects for the vast
    // majority of real programs (no exotic positions).
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "let f = function cb(){ let x = 1; };\n",
    ));
    assert!(
        registered.contains("cb"),
        "walks 1-3 must register the fn-expr id; registered={registered:?}"
    );
    assert_eq!(
        registered, seeded,
        "common-position fn-expr must be in exact lockstep (seeded == registered)"
    );
}

#[test]
fn nested_fn_lockstep_ternary_and_arg_positions_are_equal() {
    // Ternary branch + bare call-argument callback — both common positions
    // that walk 4 seeds. Still exact lockstep.
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "function run(cb){ return cb; }\n\
             let g = true ? function a(){ let x = 1; } : function b(){ let y = 2; };\n\
             run(function c(){ let z = 3; });\n",
    ));
    assert_eq!(
        registered, seeded,
        "ternary + bare-arg fn-exprs must be in exact lockstep; \
         registered={registered:?} seeded={seeded:?}"
    );
    assert!(seeded.contains("a") && seeded.contains("b") && seeded.contains("c"));
}

#[test]
fn nested_fn_lockstep_exotic_object_literal_arg_is_the_allowed_gap() {
    // F-AB-2 exotic position: a fn-expr inside an object literal passed
    // DIRECTLY as a call argument (`sink({ f: function(){…} })`). The shared
    // Phase-A descent (walks 1-3) descends the object-property value and
    // REGISTERS it, but Phase B's walk-4 `_` arm has no `ObjectExpression`
    // recursion, so it is NOT seeded. This is the documented, allowed reverse
    // gap `registered − seeded` — pinned here rather than by a hard
    // equal-assert (which would fire on day one for any such program).
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "function sink(o){ return o; }\n\
             sink({ f: function exotic(){ let x = 1; } });\n",
    ));
    assert!(
        registered.contains("exotic"),
        "walks 1-3 must register the exotic-position fn-expr; registered={registered:?}"
    );
    assert!(
        !seeded.contains("exotic"),
        "walk 4 must NOT seed the object-literal-as-direct-call-arg position \
         (F-AB-2 known gap); seeded={seeded:?}"
    );
    // The SAFE-direction invariant the debug_assert enforces still holds:
    // everything walk 4 seeds was registered by walks 1-3.
    assert!(
        seeded.is_subset(&registered),
        "F-AB-2 safe-direction invariant (seeded ⊆ registered) must hold; \
         seeded={seeded:?} registered={registered:?}"
    );
    // The reverse gap is EXACTLY the one exotic fn (the allowlist/count of
    // known-unseeded shapes for this program).
    let gap: Vec<_> = registered.difference(&seeded).cloned().collect();
    assert_eq!(
        gap,
        vec!["exotic".to_string()],
        "unexpected unseeded gap: {gap:?}"
    );
}

// ---- F-AB-2 remaining exotic positions (coverage follow-up) ----------------
//
// The Stage C C4 review (F-AB-2 resolution) landed the lockstep assertion and
// pinned exactly ONE of the documented exotic unseeded positions (the
// object-literal-as-direct-call-arg case above). The other documented shapes
// — spread arg, tagged-template operand, yield operand, optional-chain
// operand, bare/doubly-nested array literal element — had no pin, so a future
// change that silently seeded or unregistered one of them would trip
// nothing. Each was probed against kali's real lexer/parser pipeline (not
// hand-built ASTs) before writing an assertion, per "run first, pin reality":
//
//   - Two of the five are NOT expressible through kali's parser today and so
//     cannot be pinned as the documented shape at all — see the block comment
//     further down titled "parser-inexpressible exotic positions" for the
//     evidence.
//   - The other three (yield operand, optional-chain operand, and the
//     bare/doubly-nested array literal family, split into two concrete
//     shapes) parse cleanly with zero diagnostics and are confirmed-live
//     `registered − seeded` gaps today; each gets its own pin below.

#[test]
fn nested_fn_lockstep_yield_operand_is_an_unseeded_gap() {
    // F-AB-2 exotic position: a fn-expr as a `yield` operand inside a
    // generator (`function* gen(){ yield function(){…}; }`). `YieldExpression`
    // has an explicit arm in the shared Phase-A descent (`descend_expr_fns`,
    // repr_infer.rs) that registers the operand, but Phase B's walk-4
    // `visit_expr` has no `YieldExpression` arm at all, so it falls to the `_`
    // catch-all and is NOT seeded.
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "function* gen(){ yield function yieldfn(){ let x = 1; }; }\n",
    ));
    assert!(
        registered.contains("yieldfn"),
        "walks 1-3 must register the yield-operand fn-expr; registered={registered:?}"
    );
    assert!(
        !seeded.contains("yieldfn"),
        "walk 4 must NOT seed the yield-operand position (F-AB-2 known gap); \
         seeded={seeded:?}"
    );
    assert!(
        seeded.is_subset(&registered),
        "F-AB-2 safe-direction invariant (seeded ⊆ registered) must hold; \
         seeded={seeded:?} registered={registered:?}"
    );
    let gap: Vec<_> = registered.difference(&seeded).cloned().collect();
    assert_eq!(
        gap,
        vec!["yieldfn".to_string()],
        "unexpected unseeded gap: {gap:?}"
    );
}

#[test]
fn nested_fn_lockstep_optional_chain_operand_is_an_unseeded_gap() {
    // F-AB-2 exotic position: a fn-expr as the OPERAND of an optional chain —
    // the base being null-guarded, not a call argument reached through it
    // (`(function(){…})?.length`, not `o?.f(function(){…})`; the latter is
    // the already-seeded "call argument" common position). `visit_member`'s
    // fallback arm visits `member.object` generically
    // (`self.visit_expr(func, &member.object)`), and walk-4's `visit_expr` has
    // no `OptionalChainExpression` arm, so it falls to the `_` catch-all and
    // does not recurse into `chain.inner.object` — NOT seeded. The shared
    // Phase-A descent (`descend_expr_fns`) has an explicit
    // `OptionalChainExpression` arm and DOES register it.
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "let r = (function optfn(){ let x = 1; })?.length;\n",
    ));
    assert!(
        registered.contains("optfn"),
        "walks 1-3 must register the optional-chain-operand fn-expr; \
         registered={registered:?}"
    );
    assert!(
        !seeded.contains("optfn"),
        "walk 4 must NOT seed the optional-chain-operand position (F-AB-2 \
         known gap); seeded={seeded:?}"
    );
    assert!(
        seeded.is_subset(&registered),
        "F-AB-2 safe-direction invariant (seeded ⊆ registered) must hold; \
         seeded={seeded:?} registered={registered:?}"
    );
    let gap: Vec<_> = registered.difference(&seeded).cloned().collect();
    assert_eq!(
        gap,
        vec!["optfn".to_string()],
        "unexpected unseeded gap: {gap:?}"
    );
}

#[test]
fn nested_fn_lockstep_bare_array_literal_call_arg_is_an_unseeded_gap() {
    // F-AB-2 exotic position: a "bare" array literal — one reached through
    // NEITHER of the two special-cased array-literal positions
    // (declarator-init via `note_array_init`, or assignment RHS via
    // `visit_assignment`) — passed directly as a call argument
    // (`sink([function(){…}])`). The plain-identifier-callee call path visits
    // each argument via the generic `self.visit_expr(func, arg)`, and walk-4's
    // `visit_expr` has no `ArrayExpression` arm, so it falls to the `_`
    // catch-all and does not recurse into the literal's elements — NOT
    // seeded. The shared Phase-A descent (`descend_expr_fns`) has an explicit
    // `ArrayExpression` arm and DOES register it.
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "function sink(a){ return a; }\n\
             sink([function barecallarg(){ let x = 1; }]);\n",
    ));
    assert!(
        registered.contains("barecallarg"),
        "walks 1-3 must register the bare-array-literal-call-arg fn-expr; \
         registered={registered:?}"
    );
    assert!(
        !seeded.contains("barecallarg"),
        "walk 4 must NOT seed the bare-array-literal-call-arg position \
         (F-AB-2 known gap); seeded={seeded:?}"
    );
    assert!(
        seeded.is_subset(&registered),
        "F-AB-2 safe-direction invariant (seeded ⊆ registered) must hold; \
         seeded={seeded:?} registered={registered:?}"
    );
    let gap: Vec<_> = registered.difference(&seeded).cloned().collect();
    assert_eq!(
        gap,
        vec!["barecallarg".to_string()],
        "unexpected unseeded gap: {gap:?}"
    );
}

#[test]
fn nested_fn_lockstep_doubly_nested_array_literal_is_an_unseeded_gap() {
    // F-AB-2 exotic position: a fn-expr inside a DOUBLY-nested array literal
    // in declarator-init position (`let arr = [[function(){…}]];`).
    // `note_array_init` only recurses one level (it calls `visit_expr` on
    // each element of the OUTER array literal); a single-level array literal
    // element (`let arr = [function(){…}];`) IS seeded this way (verified: it
    // is NOT a gap, unlike this doubly-nested shape), but the inner array
    // literal's own elements are never descended into by that one level of
    // recursion, nor by any walk-4 arm (no `ArrayExpression` arm at all), so
    // the innermost fn-expr is NOT seeded. The shared Phase-A descent
    // (`descend_expr_fns`) recurses into `ArrayExpression` elements
    // unconditionally (arbitrary depth) and DOES register it.
    let (registered, seeded) = nested_fn_lockstep_sets(&crate::test_support::parse_statements(
        "let arr = [[function nestedfn(){ let x = 1; }]];\n",
    ));
    assert!(
        registered.contains("nestedfn"),
        "walks 1-3 must register the doubly-nested-array-literal fn-expr; \
         registered={registered:?}"
    );
    assert!(
        !seeded.contains("nestedfn"),
        "walk 4 must NOT seed the doubly-nested-array-literal position \
         (F-AB-2 known gap); seeded={seeded:?}"
    );
    assert!(
        seeded.is_subset(&registered),
        "F-AB-2 safe-direction invariant (seeded ⊆ registered) must hold; \
         seeded={seeded:?} registered={registered:?}"
    );
    let gap: Vec<_> = registered.difference(&seeded).cloned().collect();
    assert_eq!(
        gap,
        vec!["nestedfn".to_string()],
        "unexpected unseeded gap: {gap:?}"
    );
}

// ---- parser-inexpressible exotic positions (documented, not pinned) -------
//
// The remaining two documented exotic positions — a SPREAD arg
// (`foo(...[() => {}])`) and a TAGGED-TEMPLATE operand
// (`` tag`hello ${() => {}}` ``) — cannot be pinned against kali's real
// lexer/parser pipeline: neither produces the intended AST shape, and NEITHER
// raises a diagnostic, so there is nothing to assert against without hand-
// building an AST that no real kali program can produce (which would pin an
// assumption, not reality).
//
//   - Spread (`...`): the lexer tokenizes `...` as `TokenType::DotDotDot`
//     (kali_lexer/src/token.rs), but `DotDotDot` is never referenced anywhere
//     in `kali_parser/src` — no call-argument spread, no rest parameter, no
//     `SpreadElement` construction exists in the grammar (confirmed by
//     `grep -rn DotDotDot crates/kali_parser/src` returning nothing). Probed
//     directly: `function sink(...args){ return args; }` parses `args` as a
//     dropped identifier — the param list becomes the single literal string
//     `"..."`, not `"args"` — and `sink(...arr)` parses as `sink(unknown)`
//     followed by a spurious extra `arr;` expression statement (the trailing
//     identifier is split off as its own statement). `repr_infer.rs` does
//     carry a `SpreadElement` arm in `descend_expr_fns` (used by walks 1-3),
//     but — mirroring the `TemplateLiteral` arm's own note a few lines below
//     it in `visit_expr` — real source parsed through kali's own front end
//     never reaches it; only a hand-built AST could.
//   - Tagged template (`` tag`...` ``): probed directly:
//     `` tag`hello ${function taggedfn(){ let x = 1; }}` `` parses as TWO
//     unrelated statements — a bare `tag` identifier expression statement,
//     then a separate `BinaryExpression` `"`hello `" + taggedfn` (the
//     backtick characters survive literally inside the string token, and the
//     `${...}` interpolation desugars to string concatenation, exactly like
//     an ordinary un-tagged template literal) — never a
//     `TaggedTemplateExpression` with `tag` as its tag callee. Neither
//     construct is reachable from real source, so — same as the untagged
//     `TemplateLiteral` arm's note in `visit_expr` — the
//     `TaggedTemplateExpression` arm in `descend_expr_fns` is dead code from
//     kali's own parser's perspective today.
//
// Both probes ran with zero parser diagnostics (no rejection to point at);
// the "rejection evidence" here is the AST shape itself failing to match what
// the source asked for. If kali's parser ever grows real spread/rest or
// tagged-template support, these two gaps must be pinned then (or closed).
