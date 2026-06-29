use super::*;

#[test]
fn for_of_reflect_own_keys_lowers_for_frozen_static_object_literals() {
    let program = parse_and_lower_lir(
        "const object = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of Reflect.ownKeys(object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); }",
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
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_reflect_own_keys_lowers_for_frozen_static_object_literal_aliases() {
    let program = parse_and_lower_lir(
        "const frozen = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of Reflect.ownKeys(frozen)) { console.log(key); } for await (const key of globalThis['Reflect']['ownKeys'](frozen)) { console.log(key); } for await (const key of globalThis['Reflect'].ownKeys(frozen)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](frozen)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(frozen)) { console.log(key); }",
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
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for (const key of Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
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
fn supported_for_await_reflect_own_keys_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for await (const key of Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
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
fn supported_for_of_reflect_own_keys_iteration_accepts_sequence_expression_wrappers() {
    let program = parse_and_lower_lir(
        "for (const key of (0, Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }))) { console.log(key); }",
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
fn supported_for_of_reflect_own_keys_iteration_accepts_nullish_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((null ?? Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
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
fn supported_for_of_reflect_own_keys_iteration_accepts_logical_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((true && Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
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
fn supported_for_of_reflect_own_keys_iteration_accepts_logical_or_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((false || Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
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
fn supported_for_await_reflect_own_keys_iteration_accepts_sequence_expression_wrappers() {
    let program = parse_and_lower_lir(
        "for await (const key of (0, Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }))) { console.log(key); }",
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
