use super::*;

#[test]
fn supported_for_of_array_iteration_accepts_static_predicate_filter_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2, 3].filter((value) => value > 1)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_array_callback_iteration_lowering_reports_feature_unavailable() {
    for source in [
        "const values = [1, 2]; for (const item of values.find((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findIndex((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findLast((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findLastIndex((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.some((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.every((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.reduce((acc, value) => acc + value, 0)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.reduceRight((acc, value) => acc + value, 0)) { console.log(item); }",
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
                    && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && diagnostic
                        .message
                        .contains("for-of array iteration lowering is unavailable")
            }),
            "expected an unavailable array-callback diagnostic: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_const_alias_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; const values = ([1, (value)]); for (const item of (values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for ((item) of [1, 2]) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] as const)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_array_from_wrappers() {
    let program =
        parse_and_lower_lir("for (const item of Array.from([1, 2])) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_wrapped_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((null ?? Array.from))([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_and_wrapped_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((true && globalThis[\"Array\"].from))([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_logical_or_wrapped_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((false || Array.from))([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_wrapped_fully_bracketed_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((null ?? globalThis[\"Array\"][\"from\"]))([1, 2])) { console.log(item); }\nfor (const item of Object.freeze((null ?? globalThis['Array']['from']))([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_direct_array_from_calls() {
    let program =
        parse_and_lower_lir("for (const item of Array.from([1, 2])) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis.Array.from([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis[\"Array\"].from([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis['Array'].from([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_bracketed_global_this_array_bracket_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis[\"Array\"][\"from\"]([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_bracketed_global_this_array_bracket_from_calls(
) {
    let program = parse_and_lower_lir(
        "for (const item of globalThis['Array']['from']([1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_array_from_calls() {
    let program =
        parse_and_lower_lir("for (const item of Array['from']([1, 2])) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(Array.from)(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis[\"Array\"].from)(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_bracketed_and_single_quoted_frozen_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])['from'])(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_mixed_quoted_bracket_root_frozen_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])['from'])(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis.Array.from)(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_and_logical_wrapped_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((null ?? globalThis.Array.from))(values)) { console.log(item); } for (const item of Object.freeze((true && globalThis.Array.from))(values)) { console.log(item); } for (const item of Object.freeze((false || globalThis.Array.from))(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_parenthesized_global_this_array_bracket_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis.Array))[\"from\"](values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_bracketed_global_this_array_receiver_freeze_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"]))[\"from\"](values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis['Array']['from'])(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(Array['from'])(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_global_this_array_receiver_freeze_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis['Array']).from)(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array']))[\"from\"](values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_parenthesized_global_this_array_single_quoted_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis.Array))['from'](values)) { console.log(item); } for (const item of Object.freeze((globalThis[\"Array\"]))['from'](values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array']))[\"from\"](values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_frozen_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((Array.from))(values)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_identity_map_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of values.map((value) => value)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_truthy_identity_filter_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2].filter((value) => value)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_identity_flat_map_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2].flatMap((value) => [value])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] as const)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_array_from_wrappers() {
    let program =
        parse_and_lower_lir("for await (const item of Array.from([1, 2])) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for await ((item) of [1, 2]) { console.log(item); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_identity_map_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of values.map((value) => value)) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_await_wrappers() {
    let program =
        parse_and_lower_lir("for await (const value of await [1, 2, 3]) { console.log(value); }");
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...values]) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays()
{
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }",
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

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
