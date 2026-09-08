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

/// Task 4 review round 2, Critical 1: the `const`-fold (and plain-variable)
/// spelling of a computed-key write (`o[k] = <expr>`, `k` a bare identifier
/// the parser cannot read statically) lowers to `LirNodeKind::ComputedMember`
/// — a DIFFERENT LIR shape than the parser-named `o["a"] = <expr>` the
/// `Value`-kind branch above matches, and one that branch's structural guard
/// (`lhs_node.kind == LirNodeKind::Value`) does not match at all.
/// `parse_and_lower_lir` runs no `kali_types` checker pass, so this LIR
/// shape is reachable in this harness regardless of what the checker would
/// say about the source program end to end today — the point is pinning the
/// SHAPE this scan must walk (Task 5 is what will make the checker admit a
/// `const`-folded key end to end; this scan must already cover the shape
/// before that lands, per the coordinator's ruling on this Critical). Since
/// this scan has no binding table and cannot resolve which field a fold
/// names, it must taint EVERY field of the shape conservatively — pinned
/// here by targeting a bitwise compound assign on `b`, a field the write
/// never even names, and asserting it STILL fails closed.
#[test]
fn a_computed_member_bigint_write_taints_every_field_of_the_shape() {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let shape = ctx.repr_table.intern_shape(vec![
        ("a".to_string(), kali_common::Repr::I64),
        ("b".to_string(), kali_common::Repr::I64),
    ]);
    ctx.repr_table
        .set_scalar("_start", "o", kali_common::Repr::Object(shape));
    ctx.repr_table.set_numeric_shape_fields(
        [(shape, "a".to_string()), (shape, "b".to_string())]
            .into_iter()
            .collect(),
    );
    let program = parse_and_lower_lir(
        "let o = {a: 6, b: 9}; const k = \"a\"; o[k] = 7n; o.b &= 3; console.log(o.b);",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("a BigInt value was observed")),
        "a computed-member (fold/variable-key) BigInt write must taint EVERY \
         field of the shape, since this scan cannot resolve which field a \
         fold names -- a bitwise compound assign on 'b', a field the write \
         never even names, must still fail closed: {:?}",
        result.diagnostics
    );
}

/// Negative control for the `ComputedMember` branch: a plain-integer
/// computed-member write must not taint anything, mirroring the `Value`
/// branch's own negative control.
#[test]
fn a_computed_member_plain_integer_write_does_not_taint_any_field() {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let shape = ctx.repr_table.intern_shape(vec![
        ("a".to_string(), kali_common::Repr::I64),
        ("b".to_string(), kali_common::Repr::I64),
    ]);
    ctx.repr_table
        .set_scalar("_start", "o", kali_common::Repr::Object(shape));
    ctx.repr_table.set_numeric_shape_fields(
        [(shape, "a".to_string()), (shape, "b".to_string())]
            .into_iter()
            .collect(),
    );
    let program = parse_and_lower_lir(
        "let o = {a: 6, b: 9}; const k = \"a\"; o[k] = 7; o.b &= 3; console.log(o.b);",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("a BigInt value was observed")),
        "a plain-integer computed-member write must not taint the shape: {:?}",
        result.diagnostics
    );
}
