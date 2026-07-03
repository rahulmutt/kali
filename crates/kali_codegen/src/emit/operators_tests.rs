use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn bitwise_not_unary_lowers_integer_operands() {
    let program = parse_and_lower_lir("console.log(~1); console.log(~(-2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.sub"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn update_expression_lowering_keeps_prefix_and_postfix_local_reads() {
    let program = parse_and_lower_lir(
        "let value = 1; console.log(++value); console.log(value--); console.log(value);",
    );
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
    assert!(printed.contains("i64.add"), "{printed}");
    assert!(printed.contains("i64.sub"), "{printed}");
}

#[test]
fn supported_exponentiation_operator_lowering_is_available_for_integer_literals() {
    let program = parse_and_lower_lir("console.log(2 ** 3);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("call 16"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_remainder_operator_lowering_is_available_for_integer_literals() {
    let program = parse_and_lower_lir("console.log(7 % 3);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.rem_s"), "{printed}");
}

#[test]
fn unsupported_exponentiation_operator_rejects_negative_exponents() {
    let program = parse_and_lower_lir("console.log(2 ** -1);");
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
                && diagnostic.message.contains("negative numeric literals")
        }),
        "expected a negative-exponentiation diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn static_identity_strict_equality_uses_javascript_number_semantics() {
    assert!(
        StaticObjectIdentityValue::Number(0.0).strict_eq(&StaticObjectIdentityValue::Number(-0.0))
    );
    assert!(!StaticObjectIdentityValue::Number(f64::NAN)
        .strict_eq(&StaticObjectIdentityValue::Number(f64::NAN)));
}

#[test]
fn static_identity_same_value_zero_uses_array_includes_number_semantics() {
    assert!(StaticObjectIdentityValue::Number(0.0)
        .same_value_zero(&StaticObjectIdentityValue::Number(-0.0)));
    assert!(StaticObjectIdentityValue::Number(f64::NAN)
        .same_value_zero(&StaticObjectIdentityValue::Number(f64::NAN)));
}

#[test]
fn nullish_coalescing_lowers_for_supported_input_shapes() {
    assert_nullish_coalescing_lowers("console.log(null ?? 1);");
    assert_nullish_coalescing_lowers("console.log(undefined ?? 1);");
}

#[test]
fn bigint_literal_division_lowers_to_truncating_integer_division() {
    // node: (3n / 2n).toString() === "1" — BigInt `/` truncates toward zero.
    let program = parse_and_lower_lir("console.log(3n / 2n);");
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
    assert!(printed.contains("i64.div_s"), "{printed}");
    assert!(!printed.contains("f64.div"), "{printed}");
}

#[test]
fn unary_minus_bigint_literal_division_lowers_to_truncating_integer_division() {
    // node: (-7n / 2n).toString() === "-3" — BigInt `/` truncates toward zero,
    // and a unary-minus-wrapped BigInt literal is still a BigInt literal for the
    // purposes of picking the i64.div_s lane (not just the plain-literal shape).
    let program = parse_and_lower_lir("console.log(-7n / 2n);");
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
    assert!(printed.contains("i64.div_s"), "{printed}");
    assert!(!printed.contains("f64.div"), "{printed}");
}

#[test]
fn number_division_still_lowers_to_float_division() {
    let program = parse_and_lower_lir("console.log(3 / 2);");
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
    assert!(printed.contains("f64.div"), "{printed}");
}
