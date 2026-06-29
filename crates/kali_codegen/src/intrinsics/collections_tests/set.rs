use super::*;

#[test]
fn set_constructor_iteration_lowers_with_frozen_input_without_diagnostics() {
    let program = parse_and_lower_lir(
        "for (const value of new Set(Object.freeze([1, 2, 1]))) { console.log(value); }",
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
}

#[test]
fn set_constructor_iteration_lowers_via_builtin_alias_without_diagnostics() {
    let program = parse_and_lower_lir("const setAlias = Set; const wrappedSetAlias = (setAlias); const values = [1, 2, 1]; for (const value of new setAlias(values)) { console.log(value); } for await (const value of new (wrappedSetAlias)(values)) { console.log(value); }");
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
}

#[test]
fn set_constructor_iteration_lowers_via_frozen_constructor_alias_without_diagnostics() {
    let program = parse_and_lower_lir("const frozenSet = Object.freeze(Set); const values = [1, 2, 1]; for (const value of new (frozenSet)(values)) { console.log(value); }");
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
}

#[test]
fn set_constructor_iteration_lowers_through_frozen_constructor_result_without_diagnostics() {
    let program = parse_and_lower_lir(
        "const values = [1, 2, 1]; for (const value of Object.freeze(new Set(values))) { console.log(value); }",
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
}

#[test]
fn set_constructor_iteration_lowers_through_parenthesized_frozen_constructor_result_without_diagnostics(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2, 1]; for (const value of Object.freeze((new Set(values)))) { console.log(value); }",
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
}
