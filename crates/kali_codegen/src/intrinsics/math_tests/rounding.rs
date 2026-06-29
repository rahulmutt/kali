use super::*;

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
