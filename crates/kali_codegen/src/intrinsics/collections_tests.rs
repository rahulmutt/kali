use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn map_constructor_iteration_lowers_without_diagnostics() {
    let program = parse_and_lower_lir("for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }");
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
fn map_constructor_iteration_lowers_with_frozen_input_without_diagnostics() {
    let program = parse_and_lower_lir("for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry); }");
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
fn map_constructor_iteration_lowers_via_builtin_alias_without_diagnostics() {
    let program = parse_and_lower_lir("const mapAlias = Map; const wrappedMapAlias = (mapAlias); const values = [[1, 2], [1, 3], [4, 5]]; for (const entry of new mapAlias(values)) { console.log(entry[0], entry[1]); } for await (const entry of new (wrappedMapAlias)(values)) { console.log(entry[0], entry[1]); }");
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
fn map_constructor_iteration_lowers_via_frozen_constructor_alias_without_diagnostics() {
    let program = parse_and_lower_lir("const frozenMap = Object.freeze(Map); const values = [[1, 2], [1, 3], [4, 5]]; for (const entry of new (frozenMap)(values)) { console.log(entry[0], entry[1]); }");
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

#[test]
fn map_constructor_iteration_lowers_through_frozen_constructor_result_without_diagnostics() {
    let program = parse_and_lower_lir(
        "const values = [[1, 2], [1, 3], [4, 5]]; for (const entry of Object.freeze(new Map(values))) { console.log(entry[0], entry[1]); }",
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
fn map_constructor_iteration_lowers_through_parenthesized_frozen_constructor_result_without_diagnostics(
) {
    let program = parse_and_lower_lir(
        "const values = [[1, 2], [1, 3], [4, 5]]; for (const entry of Object.freeze((new Map(values)))) { console.log(entry[0], entry[1]); }",
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
fn set_and_map_constructor_iteration_lowers_through_nullish_and_logical_wrapped_frozen_constructor_result_without_diagnostics(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; for (const value of Object.freeze((null ?? new Set(values)))) { console.log(value); } for (const value of Object.freeze((true && new Set(values)))) { console.log(value); } for (const value of Object.freeze((false || new Set(values)))) { console.log(value); } for await (const entry of Object.freeze((null ?? new Map(mapValues)))) { console.log(entry[0], entry[1]); } for await (const entry of Object.freeze((true && new Map(mapValues)))) { console.log(entry[0], entry[1]); } for await (const entry of Object.freeze((false || new Map(mapValues)))) { console.log(entry[0], entry[1]); }",
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
fn set_and_map_constructor_iteration_lowers_via_global_this_roots_without_diagnostics() {
    let program = parse_and_lower_lir("for (const value of new globalThis.Set([1, 2, 1])) { console.log(value); } for await (const entry of new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }");
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
fn set_and_map_constructor_iteration_lowers_via_bracketed_global_this_roots_without_diagnostics() {
    let program = parse_and_lower_lir("for (const value of new globalThis[\"Set\"]([1, 2, 1])) { console.log(value); } for await (const entry of new globalThis['Map']([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }");
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
