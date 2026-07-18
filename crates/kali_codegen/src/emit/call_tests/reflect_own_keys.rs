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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
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

    // Deny lane (PR #16 merge readiness, family object-enum; Reflect.ownKeys
    // rides the same choke): for-of / spread over an enumeration result is
    // fail-closed E5506 (kali has no runtime materialization of
    // enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}
