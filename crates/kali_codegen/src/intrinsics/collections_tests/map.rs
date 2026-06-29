use super::*;

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
