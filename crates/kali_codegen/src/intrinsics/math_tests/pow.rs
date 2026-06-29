use super::*;

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
