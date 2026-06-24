use crate::*;
use crate::test_support::*;
use wasmparser::Validator;

#[test]
fn math_fround_zero_slice_lowers_for_static_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.fround(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn math_max_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("function max(value) { return Math.max(value, 2, 3); }");
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
    assert!(printed.contains("import \"kali:rt\" \"math_max\""));
}

#[test]
fn math_max_member_calls_lower_to_math_host_imports_through_global_this_math() {
    let program =
        parse_and_lower_lir("function max(value) { return globalThis.Math.max(value, 2, 3); }");
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
    assert!(printed.contains("import \"kali:rt\" \"math_max\""));
}

#[test]
fn math_floor_trunc_and_ceil_member_constant_folds_through_global_this_math() {
    let program = parse_and_lower_lir(
        "console.log(globalThis.Math.floor(1.6)); console.log(globalThis.Math.trunc(1.6)); console.log(globalThis.Math.ceil(1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_floor_trunc_and_ceil_member_constant_folds_through_bracketed_global_this_math() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Math\"][\"floor\"](1.6)); console.log(globalThis[\"Math\"][\"trunc\"](1.6)); console.log(globalThis[\"Math\"][\"ceil\"](1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_floor_member_constant_folding_is_available_through_object_freeze_callable_wrapper() {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze(globalThis.Math[\"floor\"])(1.6)); console.log(Object.freeze(globalThis[\"Math\"][\"floor\"])(1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn math_floor_trunc_and_ceil_member_constant_folding_is_available_through_direct_object_freeze_callable_wrapper(
) {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze(Math.floor)(1.6)); console.log(Object.freeze(Math.trunc)(1.6)); console.log(Object.freeze(Math.ceil)(1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_floor_trunc_and_ceil_member_constant_folding_is_available_through_parenthesized_single_quoted_receiver_wrapped_callable_wrapper(
) {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze((globalThis['Math'])['floor'])(1.6)); console.log(Object.freeze((globalThis['Math'])['trunc'])(1.6)); console.log(Object.freeze((globalThis['Math'])['ceil'])(1.6));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_operand() {
    let program = parse_and_lower_lir("console.log(Math.max(1, 2, 3));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_operand_through_global_this_math() {
    let program = parse_and_lower_lir("console.log(globalThis.Math.max(1, 2, 3));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_max_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 3; const alias = value; console.log(globalThis.Math.max(alias, 2, 1));",
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 7"), "{printed}");
}

#[test]
fn math_min_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("function min(value) { return Math.min(value, 3, 2); }");
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
    assert!(printed.contains("import \"kali:rt\" \"math_min\""));
}

#[test]
fn math_min_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 3; const alias = value; console.log(Math.min(alias, 2, 1));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 8"), "{printed}");
}

#[test]
fn math_abs_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.abs(3 - 6));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_abs\""));
}

#[test]
fn math_sign_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.sign(3 - 6));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_sign\""));
}

#[test]
fn math_abs_member_constant_folds_static_numeric_literal_alias_chains() {
    let program =
        parse_and_lower_lir("const value = -3; const alias = value; console.log(Math.abs(alias));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 9"), "{printed}");
}

#[test]
fn math_abs_member_constant_folds_static_numeric_literal_operand() {
    let program = parse_and_lower_lir("console.log(Math.abs(-3));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(!printed.contains("call 9"), "{printed}");
}

#[test]
fn math_sign_member_constant_folds_static_numeric_literal_operand() {
    let program = parse_and_lower_lir("console.log(Math.sign(1.6));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 10"), "{printed}");
}

#[test]
fn math_sign_member_constant_folds_static_numeric_literal_alias_chains() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.sign(alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 10"), "{printed}");
}

#[test]
fn math_abs_and_sign_member_calls_lower_through_bracketed_global_this_math_root() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Math\"][\"abs\"](3 - 6)); console.log(globalThis[\"Math\"][\"sign\"](3 - 6));",
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
    assert!(printed.contains(r#"import "kali:rt" "math_abs""#));
    assert!(printed.contains(r#"import "kali:rt" "math_sign""#));
}

#[test]
fn math_imul_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.imul(2147483647, 2));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_imul\""));
}

#[test]
fn math_imul_member_constant_folds_static_integer_literal_operands() {
    let program = parse_and_lower_lir("console.log(Math.imul(2147483647, 2));");
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
    assert!(printed.contains("i64.const -2"), "{printed}");
    assert!(!printed.contains("call 11"), "{printed}");
}

#[test]
fn math_imul_member_constant_folds_omitted_operands_to_zero() {
    let program = parse_and_lower_lir("console.log(Math.imul()); console.log(Math.imul(7));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 7"), "{printed}");
    assert!(!printed.contains("call 11"), "{printed}");
}

#[test]
fn math_clz32_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.clz32(1));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_clz32\""));
}

#[test]
fn math_clz32_member_constant_folds_zero_arguments() {
    let program = parse_and_lower_lir("console.log(Math.clz32());");
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
    assert!(printed.contains("i64.const 32"), "{printed}");
    assert!(!printed.contains("call 15"), "{printed}");
}

#[test]
fn math_clz32_member_constant_folds_static_integer_literal_alias_chain() {
    let program = parse_and_lower_lir(
        "const value = 1; const alias = value; console.log(Math.clz32(alias));",
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
    assert!(printed.contains("i64.const 31"), "{printed}");
    assert!(!printed.contains("call 14"), "{printed}");
}

#[test]
fn math_clz32_member_constant_folds_static_non_integer_literal() {
    let program = parse_and_lower_lir("console.log(Math.clz32(1.6));");
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
    assert!(printed.contains("i64.const 31"), "{printed}");
    assert!(!printed.contains("call 14"), "{printed}");
}

#[test]
fn math_pow_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, 3));");
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
    assert!(printed.contains(r#"import "kali:rt" "math_pow""#));
}

#[test]
fn math_pow_member_constant_folds_zero_exponent_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, 0));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_zero_exponent_identity_for_non_integer_base_literals() {
    let program = parse_and_lower_lir("console.log(Math.pow(1.6, 0));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_zero_base_positive_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(0, alias));",
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
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_exponent_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(7, 1));");
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
    assert!(printed.contains("i64.const 7"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_identity() {
    let program = parse_and_lower_lir("console.log(Math.pow(1, 7));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_identity_through_bracketed_global_this_math() {
    let program = parse_and_lower_lir("console.log(globalThis[\"Math\"][\"pow\"](1, 7));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_uses_integer_exponent_alias_chain() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias));",
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_pow""#),
        "{printed}"
    );
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn math_pow_member_uses_integer_exponent_alias_chain_through_frozen_global_this_aliases() {
    let program = parse_and_lower_lir(
        r#"const exponent = 3; const alias = exponent; console.log(Object.freeze((globalThis.Math["pow"]))(2, alias)); console.log(Object.freeze((globalThis["Math"]["pow"]))(2, alias));"#,
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_pow""#),
        "{printed}"
    );
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn math_pow_member_uses_negative_integer_base_with_integer_exponent_alias_chain() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(-2, alias));",
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_pow""#),
        "{printed}"
    );
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn math_round_member_calls_lower_to_math_host_imports() {
    let program = parse_and_lower_lir("console.log(Math.round(1));");
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
    assert!(printed.contains("import \"kali:rt\" \"math_round\""));
}

#[test]
fn math_round_member_calls_constant_fold_floating_literal() {
    let program = parse_and_lower_lir("console.log(Math.round(1.6));");
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_round_member_constant_folding_is_available_through_object_freeze_callable_wrapper() {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze(globalThis.Math[\"round\"])(1.6)); console.log(Object.freeze(globalThis[\"Math\"][\"round\"])(1.6)); console.log(Object.freeze(globalThis.Math['round'])(1.6)); console.log(Object.freeze(globalThis[\"Math\"]['round'])(1.6)); console.log(Object.freeze(globalThis['Math'].round)(1.6)); console.log(Object.freeze(globalThis['Math']['round'])(1.6)); console.log(Object.freeze((globalThis['Math'])[\"round\"])(1.6));",
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_round_member_constant_folding_is_available_through_parenthesized_receiver_wrapped_callable_wrapper(
) {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze((globalThis[\"Math\"]).round)(1.6)); console.log(Object.freeze((globalThis['Math']).round)(1.6));",
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn math_round_member_calls_global_this_root_lower_to_math_host_imports() {
    let program =
        parse_and_lower_lir("function f(value) { console.log(globalThis.Math.round(value)); }");
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
    assert!(
        printed.contains(r#"import "kali:rt" "math_round""#),
        "{printed}"
    );
}

#[test]
fn math_trunc_member_lowers_without_runtime_host_import() {
    let program = parse_and_lower_lir("console.log(Math.trunc(1));");
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
    assert!(!printed.contains("import \"kali:rt\" \"math_trunc\""));
}

#[test]
fn math_ceil_member_lowers_without_runtime_host_import() {
    let program = parse_and_lower_lir("console.log(Math.ceil(1));");
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
    assert!(!printed.contains("import \"kali:rt\" \"math_ceil\""));
}

#[test]
fn supported_math_ceil_member_constant_folds_non_integer_numeric_literals() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias));",
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn supported_math_trunc_member_constant_folds_non_integer_numeric_literals() {
    let program = parse_and_lower_lir(
        "const value = 1.6; const alias = value; console.log(Math.trunc(alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_floor_member_lowering_is_available() {
    let program = parse_and_lower_lir("console.log(Math.floor(1));");
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
}

#[test]
fn supported_math_floor_member_constant_folding_is_available_for_non_integer_literal() {
    let program = parse_and_lower_lir("console.log(Math.floor(1.6));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_hypot_member_lowering_is_available_for_perfect_square_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.hypot(3, 4));");
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
    assert!(printed.contains("i64.const 5"), "{printed}");
}

#[test]
fn supported_math_hypot_member_lowering_is_available_for_zero_arguments() {
    let program = parse_and_lower_lir("console.log(Math.hypot());");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_sqrt_member_lowering_is_available_for_perfect_square_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.sqrt(4));");
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn supported_math_sqrt_member_constant_folding_is_available_through_object_freeze_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "const value = 4; console.log(Object.freeze(globalThis.Math.sqrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"sqrt\"])(value));",
    );
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
    assert!(printed.contains("i64.const 2"), "{printed}");
}

#[test]
fn supported_math_cbrt_member_constant_folding_is_available_through_object_freeze_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "const value = 27; console.log(Object.freeze(globalThis.Math.cbrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"cbrt\"])(value));",
    );
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
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_math_cbrt_member_lowering_is_available_for_perfect_cube_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.cbrt(27));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_math_log2_member_lowering_is_available_for_positive_power_of_two_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.log2(8));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_math_log10_member_lowering_is_available_for_positive_power_of_ten_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.log10(1000));");
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
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn unsupported_math_sqrt_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.sqrt(1.6));");
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
                    .contains("Math.sqrt is unavailable unless the argument is a statically-known perfect-square integer literal")
        }),
        "expected an unavailable Math-member diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_exp_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.exp(zero));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_exp2_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.exp2(zero));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_global_this_bracketed_math_exp2_member_lowering_is_available_for_exact_zero_literals()
{
    let program =
        parse_and_lower_lir("const zero = 0; console.log(globalThis[\"Math\"][\"exp2\"](zero));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn supported_math_log_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.log(one));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_exp_and_log_member_lowering_is_available_for_const_numeric_alias_chain_literals()
{
    let program = parse_and_lower_lir(
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.exp(zeroAlias)); console.log(Math.log(oneAlias));",
    );
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_exp_and_log_accept_object_freeze_callable_wrappers() {
    let program = parse_and_lower_lir(
        "const zero = 0; const one = 1; console.log(Object.freeze(Math.exp)(zero)); console.log(Object.freeze(globalThis[\"Math\"][\"log\"])(one)); console.log(Object.freeze(globalThis.Math.exp)(zero)); console.log(Object.freeze(globalThis.Math[\"log\"])(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one));",
    );
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_expm1_and_log1p_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir(
        "const zero = 0; console.log(Math.expm1(zero)); console.log(Math.log1p(zero));",
    );
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_expm1_and_log1p_member_lowering_is_available_for_const_numeric_alias_chain_literals(
) {
    let program = parse_and_lower_lir(
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));",
    );
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_tan_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.tan(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_sin_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.sin(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_cos_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.cos(zero));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn unsupported_math_tan_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.tan(1));");
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
                    .contains("Math.tan is unavailable unless the argument is a statically-known zero numeric literal")
        }),
        "expected an unavailable Math.tan diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_asin_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.asin(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_acos_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.acos(one));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.atan(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_asinh_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.asinh(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_acosh_member_lowering_is_available_for_exact_one_literals() {
    let program = parse_and_lower_lir("const one = 1; console.log(Math.acosh(one));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atanh_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir("const zero = 0; console.log(Math.atanh(zero));");
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_hyperbolic_zero_identity_member_lowering_is_available_for_exact_zero_literals() {
    let program = parse_and_lower_lir(
        "const zero = 0; console.log(Math.sinh(zero)); console.log(Math.cosh(zero)); console.log(Math.tanh(zero));",
    );
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
    assert!(printed.contains("i64.const 0"), "{printed}");
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn unsupported_math_hyperbolic_zero_identity_member_reports_feature_unavailable() {
    for (source, expected_method) in [
        ("console.log(Math.sinh(1));", "Math.sinh"),
        ("console.log(Math.cosh(1));", "Math.cosh"),
        ("console.log(Math.tanh(1));", "Math.tanh"),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains("zero numeric literal")
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_inverse_hyperbolic_member_reports_feature_unavailable() {
    for (source, expected_method, expected_literal) in [
        (
            "console.log(Math.asinh(1));",
            "Math.asinh",
            "zero numeric literal",
        ),
        (
            "console.log(Math.acosh(0));",
            "Math.acosh",
            "one numeric literal",
        ),
        (
            "console.log(Math.atanh(1));",
            "Math.atanh",
            "zero numeric literal",
        ),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains(expected_literal)
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_inverse_trig_member_reports_feature_unavailable() {
    for (source, expected_method, expected_literal) in [
        (
            "console.log(Math.asin(1));",
            "Math.asin",
            "zero numeric literal",
        ),
        (
            "console.log(Math.acos(0));",
            "Math.acos",
            "one numeric literal",
        ),
        (
            "console.log(Math.atan(1));",
            "Math.atan",
            "zero numeric literal",
        ),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains(expected_literal)
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_atan2_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.atan2(1, 1));");
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
                && diagnostic.message.contains("Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path")
        }),
        "expected an unavailable Math.atan2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_atan2_member_is_available_for_zero_numerator_and_non_negative_denominator_literals(
) {
    let program =
        parse_and_lower_lir("const zero = 0; const one = 1; console.log(Math.atan2(zero, one));");
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
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan2_member_is_available_for_const_numeric_alias_chain() {
    let program = parse_and_lower_lir(
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.atan2(zeroAlias, oneAlias));",
    );
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
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_math_atan2_member_is_available_for_bracketed_global_this_aliases() {
    let program = parse_and_lower_lir(
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one)); console.log(globalThis.Math[\"atan2\"](zero, one)); console.log(globalThis[\"Math\"].atan2(zero, one));",
    );
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
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn unsupported_math_exp_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.exp(2));");
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
                    .contains("Math.exp is unavailable unless the argument is a statically-known zero numeric literal")
        }),
        "expected an unavailable Math.exp diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_math_exp2_member_lowering_is_available_for_non_negative_integer_literals() {
    let program = parse_and_lower_lir("console.log(Math.exp2(1)); console.log(Math.exp2(3));");
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
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 8"), "{printed}");
}

#[test]
fn unsupported_math_exp2_non_integer_literals_report_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.exp2(1.5));");
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
                    .contains("Math.exp2 is unavailable unless the argument is a statically-known non-negative integer literal within the current integer-fold range")
        }),
        "expected an unavailable Math.exp2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_log_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log(2));");
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
                    .contains("Math.log is unavailable unless the argument is a statically-known one numeric literal")
        }),
        "expected an unavailable Math.log diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_expm1_log1p_and_fround_member_reports_feature_unavailable() {
    for (source, expected_method) in [
        ("console.log(Math.expm1(1));", "Math.expm1"),
        ("console.log(Math.log1p(1));", "Math.log1p"),
        ("console.log(Math.fround(1));", "Math.fround"),
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.is_error()
                    && diagnostic.code
                        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic.message.contains(expected_method)
                    && diagnostic.message.contains("zero numeric literal")
            }),
            "expected an unavailable {expected_method} diagnostic: {:?}",
            result.diagnostics
        );

        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn unsupported_math_log2_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log2(12));");
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
                && diagnostic.message.contains("positive power-of-two")
        }),
        "expected an unavailable Math.log2 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_log10_member_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.log10(12));");
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
                && diagnostic.message.contains("positive power-of-ten")
        }),
        "expected an unavailable Math.log10 diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_max_without_arguments_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.max());");
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
                    .contains("requires at least one argument")
        }),
        "expected an unavailable Math.max diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_with_single_argument_reports_feature_unavailable() {
    let program = parse_and_lower_lir("console.log(Math.pow(2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(5506)
                && diagnostic
                    .message
                    .contains("requires at least two operands")
        }),
        "expected an unavailable Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_member_rejects_non_integer_const_alias_exponents() {
    let program = parse_and_lower_lir(
        "const exponent = 1.6; const alias = exponent; console.log(Math.pow(2, alias));",
    );
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
                && diagnostic.message.contains("non-integer numeric literals")
        }),
        "expected a non-integer Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_math_pow_member_rejects_negative_exponents() {
    let program = parse_and_lower_lir("console.log(Math.pow(2, -1));");
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
                    .contains("Math.pow is unavailable for negative numeric literals")
        }),
        "expected a negative-exponent Math.pow diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn math_pow_member_constant_folds_negative_one_base_positive_integer_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = 3; const alias = exponent; console.log(Math.pow(-1, alias));",
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
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_negative_one_base_negative_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = -3; const alias = exponent; console.log(Math.pow(-1, alias));",
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
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}

#[test]
fn math_pow_member_constant_folds_one_base_negative_exponent() {
    let program = parse_and_lower_lir(
        "const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(!printed.contains("call 16"), "{printed}");
}
