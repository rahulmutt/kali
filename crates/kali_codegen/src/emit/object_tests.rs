use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

/// Spec 4a Task 3: a computed for-in-key access `t[c]` over a uniform-float
/// fixed shape lowers to a HEADERLESS field slot — address `base + c*8`, load
/// AND store at `offset: 0`. Objects carry no length header (arrays put their
/// elements at `offset: 8` behind an 8-byte length word); reusing the array
/// element-address path here would be off by one slot. A single-field shape
/// keeps the whole module free of any `offset=8` memory access, so the
/// headerless invariant is directly observable: the dynamic access scales the
/// index by 8 (`i32.mul`) yet every f64 load/store sits at offset 0.
#[test]
fn computed_forin_key_access_uses_headerless_offset_zero() {
    // `kali_codegen`'s own test pipeline (`parse_and_lower_lir`) does not run
    // the `kali_types` shape inference that populates `ReprTable` in the real
    // compiler driver. Drive the lane the way the real compiler does:
    // construct the `ReprTable` entries the inference would produce for
    // `const t = { a: 1.0 };` — a one-field uniform-F64 shape and `t`'s scalar
    // repr pointing at it — before lowering. `for_in_key_shapes` is populated
    // by `emit_for_in` itself during codegen, so no manual key registration is
    // needed.
    let src = "const t = { a: 1.0 };\nfor (var c in t) {\n  t[c] = t[c];\n}\n";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let shape = ctx
        .repr_table
        .intern_shape(vec![("a".to_string(), kali_common::Repr::F64)]);
    ctx.repr_table
        .set_scalar("_start", "t", kali_common::Repr::Object(shape));
    ctx.arena_table.set_arena_eligible("_start");

    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");

    // The dynamic slot address scales the ordinal by the 8-byte field stride —
    // this is the computed-key lane, not a static field read (which would emit
    // no `i32.mul`).
    assert!(printed.contains("i32.mul"), "expected index*8 scaling:\n{printed}");
    // Both directions of the access are emitted as f64 (uniform-float shape).
    assert!(printed.contains("f64.store"), "expected f64.store:\n{printed}");
    assert!(printed.contains("f64.load"), "expected f64.load:\n{printed}");
    // Headerless: the f64 field access sits at offset 0, NOT the array's
    // 8-byte length-header offset. The only `offset=8` in the module is the
    // i64/byte runtime helpers (`__join` etc.) — no FLOAT access rides the
    // array element-address path. A single f64.load/f64.store at offset=8
    // would mean the object access wrongly reused the array header offset.
    assert!(
        !printed.contains("f64.load offset=8") && !printed.contains("f64.store offset=8"),
        "computed object f64 access must be headerless (offset 0), found an f64 offset=8:\n{printed}"
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
