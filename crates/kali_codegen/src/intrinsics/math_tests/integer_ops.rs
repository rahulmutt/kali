use super::*;

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
