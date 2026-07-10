use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn mutable_local_reassignment_keeps_runtime_reads() {
    let program = parse_and_lower_lir("let value = 1; value = 3; console.log(value);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn mutable_local_compound_assignment_accepts_wrapper_targets() {
    let program = parse_and_lower_lir("let value = 1; ((value)) += 2; console.log(value);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn compound_assignment_on_non_local_targets_reports_feature_unavailable() {
    let program = parse_and_lower_lir("let target = { value: 1 }; target.value += 2;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("compound assignment lowering is unavailable")
        }),
        "expected a compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn compound_assignment_on_immutable_bindings_reports_feature_unavailable() {
    let program = parse_and_lower_lir("const value = 1; value += 2;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("compound assignment lowering is unavailable for binding 'value'")
        }),
        "expected an immutable-binding compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn nullish_assignment_lowers_for_wrapped_for_in_key_alias_targets() {
    // fasta Spec 7 Task 3: scalar `??=` now rejects in resolve (null and 0 are
    // indistinguishable for a scalar), so the surviving `??=` lowering — and
    // the only valid "lowers cleanly" pin — is a for-in-key ALIAS target,
    // whose `-1` null sentinel keeps null distinct from every key ordinal.
    // The wrapped parens keep this test's transparent-target coverage.
    assert_nullish_assignment_lowers(
        "var table = { a: 1, b: 2 }; var last = null; for (var c in table) { last = c; } ((last)) ??= null; if (last) { console.log(1); }",
        |ctx| {
            let shape = ctx.repr_table.intern_shape(vec![
                ("a".to_string(), kali_common::Repr::I64),
                ("b".to_string(), kali_common::Repr::I64),
            ]);
            ctx.repr_table
                .set_scalar("_start", "table", kali_common::Repr::Object(shape));
            ctx.arena_table.set_arena_eligible("_start");
        },
    );
}

#[test]
fn logical_assignment_lowers_for_wrapped_mutable_local_binding_targets() {
    assert_logical_assignment_lowers("let left = 0; ((left)) ||= 1; console.log(left); let right = 1; ((right)) &&= 2; console.log(right);");
}
