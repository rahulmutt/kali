use crate::test_support::parse_and_lower_lir;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};

/// `kali_codegen`'s own test pipeline (`parse_and_lower_lir`) does not run the
/// `kali_types` shape inference that populates `ReprTable` in the real
/// compiler driver — see `object_tests`'s
/// `computed_forin_key_access_uses_headerless_offset_zero` for the same
/// pattern. Construct the `ReprTable` entries inference would produce for
/// `{ a: <int> }` before lowering.
fn ctx_with_object_shape() -> (CodegenCtx, kali_common::ShapeId) {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let shape = ctx
        .repr_table
        .intern_shape(vec![("a".to_string(), kali_common::Repr::I64)]);
    ctx.repr_table
        .set_scalar("_start", "o", kali_common::Repr::Object(shape));
    // `shape_field_is_proven_numeric` is a SEPARATE proof from `Repr::I64`
    // (real `repr_infer` populates it via `set_numeric_shape_fields`); the
    // bitwise compound-assign arm gates on both, so both must be set up here
    // for the arm to reach the BigInt-taint check this test targets, exactly
    // as `computed_forin_key_access_uses_headerless_offset_zero` primes only
    // what its own lane needs.
    ctx.repr_table
        .set_numeric_shape_fields([(shape, "a".to_string())].into_iter().collect());
    (ctx, shape)
}

/// computed-member-static-name Task 4 follow-up, route 3 of
/// `collect_bigint_tainted_shape_fields`'s write-route inventory: a
/// computed-key write `o["a"] = 7n` must taint the `(shape, "a")` field
/// exactly like the dot-field write `o.a = 7n` already does (route 2), since
/// Task 4's store choke point now lowers both through the same
/// `try_emit_shaped_field_store`. Before the fix this route was unwalked, so
/// the field was never tainted and a later `o.a &= 3` silently computed on
/// the raw BigInt bit pattern instead of refusing — exactly the gap
/// `bitwise_compound_tripwire_computed_key_write_not_covered_by_bigint_taint_scan`
/// exists to catch, now that Task 4 makes the write land for real.
#[test]
fn a_computed_key_bigint_write_taints_the_field_like_the_dot_write_does() {
    let (mut ctx, _shape) = ctx_with_object_shape();
    let program = parse_and_lower_lir("let o = {a: 6}; o[\"a\"] = 7n; o.a &= 3; console.log(o.a);");
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("a BigInt value was observed")),
        "a computed-key BigInt write must taint the field so the later bitwise \
         compound-assign fails closed instead of computing on the raw bits: {:?}",
        result.diagnostics
    );
}

/// Negative control: a computed-key write of a PLAIN integer (no BigInt
/// anywhere in the program) must NOT taint the field — the scan is additive
/// and must not regress the working, already-covered lane merely because a
/// bracket-form target is now also walked.
#[test]
fn a_computed_key_plain_integer_write_does_not_taint_the_field() {
    let (mut ctx, _shape) = ctx_with_object_shape();
    let program = parse_and_lower_lir("let o = {a: 6}; o[\"a\"] = 7; o.a &= 3; console.log(o.a);");
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("a BigInt value was observed")),
        "a plain-integer computed-key write must not taint the field: {:?}",
        result.diagnostics
    );
}
