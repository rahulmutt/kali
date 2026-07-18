use super::*;

#[test]
fn for_of_object_enumeration_lowers_for_single_quoted_bracketed_aliases_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis['Object']['values'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for (const key of [...globalThis['Object']['keys'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(key); } for (const entry of [...globalThis['Object']['entries'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_quote_bracketed_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(globalThis[\"Object\"]['fromEntries']([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(globalThis['Object'][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_await_spread_of_object_enumeration_lowers_for_remaining_global_this_object_spellings_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "for await (const value of [...globalThis[\"Object\"].values(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for await (const value of [...globalThis.Object.values(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for await (const key of [...globalThis.Object[\"keys\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(key); } for await (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_bracketed_object_spelling_variants() {
    let program = parse_and_lower_lir(
        "for (const key of Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); } for (const entry of globalThis.Object[\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_object_freeze_wrapped_object_roots() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze(Object).keys({ b: 1, a: 2 })) { console.log(key); } for (const value of Object.freeze(globalThis[\"Object\"]).values({ b: 1, a: 2 })) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_bracketed_global_this_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(globalThis[\"Object\"][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); } for (const entry of Object.entries(globalThis[\"Object\"][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_quote_bracket_root_aliases() {
    let program = parse_and_lower_lir(
        r#"const object = Object.freeze({ "b": 1, "2": 2, "a": 3, "1": 4 }); for (const key of globalThis["Object"]['keys'](object)) { console.log(key); } for (const value of globalThis['Object']["values"](object)) { console.log(value); } for (const entry of globalThis["Object"]['entries'](object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis["Reflect"]['ownKeys'](object)) { console.log(key); } for (const key of globalThis['Reflect']["ownKeys"](object)) { console.log(key); }"#,
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_dot_root_bracket_aliases() {
    let program = parse_and_lower_lir(
        "const object = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of globalThis[\"Object\"].keys(object)) { console.log(key); } for (const value of globalThis[\"Object\"].values(object)) { console.log(value); } for (const entry of globalThis[\"Object\"].entries(object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis['Object'].keys(object)) { console.log(key); } for (const value of globalThis['Object'].values(object)) { console.log(value); } for (const entry of globalThis['Object'].entries(object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); } for (const key of globalThis['Reflect'][\"ownKeys\"](object)) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_values_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of Object.values(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_entries_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_bracket_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis.Object[\"values\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(value); } for (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const key of [...globalThis[\"Object\"].keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_frozen_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_object_enumeration_lowers_for_frozen_static_object_literal_values() {
    let program = parse_and_lower_lir(
        "for (const value of Object.values(Object.freeze({ \"b\": 1, \"a\": 2 }))) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_of_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of [...Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn for_await_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for await (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_parenthesized_frozen_bracket_root_wrappers()
{
    let program = parse_and_lower_lir(
        "for await (const entry of Object.freeze((globalThis[\"Object\"]))[\"entries\"]({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_of_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_of_object_keys_iteration_accepts_parenthesized_frozen_callable_wrappers() {
    let program = parse_and_lower_lir(
        "for (const key of (Object.freeze(Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_of_object_keys_iteration_accepts_parenthesized_global_this_frozen_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "for (const key of (Object.freeze((globalThis.Object.keys)))({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_keys_iteration_accepts_logical_and_or_and_single_quoted_bracket_root_wrappers(
) {
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((true && Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); } for await (const key of Object.freeze((false || Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); } for await (const key of Object.freeze((globalThis['Object'])['keys'])({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_values_iteration_accepts_parenthesized_frozen_callable_wrappers() {
    let program = parse_and_lower_lir(
        "for await (const value of (Object.freeze(Object.values))({ \"b\": 1, \"a\": 2 })) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_parenthesized_global_this_frozen_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "for await (const entry of (Object.freeze((globalThis.Object.entries)))({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_of_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys('ab')) { console.log(key); } for (const value of Object.values('ab')) { console.log(value); } for (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_of_object_entries_string_literals_accept_bracketed_global_this_alias() {
    let program = parse_and_lower_lir(
        "for (const entry of globalThis[\"Object\"][\"entries\"]('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for await (const key of Object.keys('ab')) { console.log(key); } for await (const value of Object.values('ab')) { console.log(value); } for await (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_enumeration_accepts_nullish_and_logical_wrapped_callable_aliases() {
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((null ?? Object.keys))('ab')) { console.log(key); } for await (const value of Object.freeze((true && Object.values))('ab')) { console.log(value); } for await (const entry of Object.freeze((false || Object.entries))('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_object_enumeration_accepts_parenthesized_receiver_wrapped_bracketed_aliases()
{
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((globalThis.Object)[\"keys\"])('ab')) { console.log(key); } for await (const value of Object.freeze((globalThis[\"Object\"])[\"values\"])('ab')) { console.log(value); } for await (const entry of Object.freeze((globalThis[\"Object\"])[\"entries\"])('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): for-of / spread
    // over an enumeration result is fail-closed E5506 (kali has no runtime
    // materialization of enumeration-result arrays). Flip-back:
    // pr16-honest-repin-inventory.md#object-enum.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_for_await_string_concatenation_iteration_accepts_static_string_operands() {
    let program = parse_and_lower_lir(
        "const prefix = 'he'; const suffix = 'llo'; for await (const ch of prefix + suffix) { console.log(ch); } for await (const ch of (`${prefix}${suffix}`)) { console.log(ch); }",
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
fn object_enumeration_helper_iteration_lowers_via_frozen_object_entries_call_without_diagnostics() {
    let program = parse_and_lower_lir("for await (const entry of (Object.freeze(Object.entries))({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // Deny lane (PR #16 merge readiness, family object-enum): fail-closed E5506.
    assert!(
        result.diagnostics.iter().any(|diag| diag.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected fail-closed E5506, got: {:?}",
        result.diagnostics
    );
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
