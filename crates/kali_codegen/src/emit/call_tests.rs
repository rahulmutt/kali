use crate::test_support::*;
use crate::*;
use kali_test_support::fixtures::{tempdir, write_file};
use wasmparser::Validator;
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

#[test]
fn object_enumeration_helper_iteration_lowers_via_frozen_object_entries_call_without_diagnostics() {
    let program = parse_and_lower_lir("for await (const entry of (Object.freeze(Object.entries))({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }");
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
fn unresolved_identifier_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_value.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_value")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-identifier diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unresolved_call_target_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[10].text = Some("missing_fn".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_fn.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_fn")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-call diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn duplicate_unresolved_identifier_lowering_reports_one_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());
    program.nodes[8].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let matching_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic.message.contains("missing_value")
        })
        .count();
    assert_eq!(
        matching_diagnostics, 1,
        "expected the repeated unresolved identifier to be reported once: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn source_path_in_temp_dir_attaches_to_unresolved_identifier_diagnostics() {
    // Use kali_test_support::fixtures to create a real temp directory with a
    // source file alongside a package.json; verify the source_path is threaded
    // through to diagnostics so downstream tooling knows where the file lives.
    let dir = tempdir();
    let src = write_file(dir.path(), "index.ts", "missing_var;");
    let _pkg = write_file(
        dir.path(),
        "package.json",
        r#"{"name":"smoke","version":"1.2.3"}"#,
    );

    let program = parse_and_lower_lir("missing_var;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(src.clone());
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // The unresolved identifier should produce a warning that embeds the
    // source path from the temp directory.
    let src_str = src.to_string_lossy();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| { d.notes.iter().any(|note| note.contains(src_str.as_ref())) }),
        "expected a diagnostic containing the tempdir source path {:?}; got: {:?}",
        src,
        result.diagnostics
    );
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
