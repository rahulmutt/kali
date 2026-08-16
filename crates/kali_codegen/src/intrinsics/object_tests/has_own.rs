use super::*;

#[test]
fn object_has_own_lowers_for_single_quoted_bracketed_alias_over_frozen_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(globalThis['Object']['hasOwn'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_parenthesized_single_quoted_bracketed_alias_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze((globalThis['Object'])['hasOwn'])(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(Object.hasOwn(Object.fromEntries([[\"b\", 1], [\"a\", 2]]), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_frozen_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "console.log(Object.hasOwn(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_bracketed_global_this_object_spellings() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Object\"][\"hasOwn\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_bracketed_global_this_object_from_entries_spellings() {
    let program = parse_and_lower_lir(
        "console.log(globalThis[\"Object\"][\"hasOwn\"](globalThis[\"Object\"][\"fromEntries\"]([[\"b\", 1], [\"a\", 2]]), \"a\"));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_for_mixed_bracketed_global_this_object_spellings() {
    for program in [
        "console.log(globalThis.Object[\"hasOwn\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
        "console.log(globalThis[\"Object\"].hasOwn(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
    ] {
        let program = parse_and_lower_lir(program);
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
        assert!(printed.contains("i64.const 1"), "{printed}");
    }
}

#[test]
fn object_has_own_lowers_for_bracketed_callable_alias_over_static_object_literal() {
    let program =
        parse_and_lower_lir("console.log(Object[\"hasOwn\"]({\"a\": 1, \"b\": 2}, \"a\"));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn object_has_own_lowers_through_object_freeze_callable_wrapper() {
    let program = parse_and_lower_lir(
        "console.log(Object.freeze(globalThis.Object[\"hasOwn\"])(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\")); console.log(Object.freeze(globalThis[\"Object\"][\"hasOwn\"])(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2]])), \"a\"));",
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
fn object_has_own_folds_true_for_a_threshold_crossing_numeric_key() {
    // The probe key and the stored key are produced by ONE function. When they
    // were two -- `render_static_value` on the probe, raw HIR text on the
    // stored side -- this folded to a silent `false` the moment the renderer
    // started emitting JS notation, because the stored side was still Rust's
    // `Display` expansion (`1000000000000000000000`).
    let program = parse_and_lower_lir("console.log(Object.hasOwn({1e21: 1}, 1e21));");
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
    assert!(printed.contains("i64.const 1"), "{printed}");
}

#[test]
fn probe_key_text_is_the_property_name_the_expression_denotes() {
    use super::super::canonical_property_key_text;

    // Numbers render as JS renders them.
    assert_eq!(canonical_property_key_text("1000000000000000000000"), "1e+21");
    assert_eq!(canonical_property_key_text("0.0000001"), "1e-7");
    assert_eq!(canonical_property_key_text("5"), "5");

    // A quote means a string literal, whose content is the key however spelled.
    assert_eq!(
        canonical_property_key_text("\"1000000000000000000000\""),
        "1000000000000000000000"
    );
    assert_eq!(canonical_property_key_text("\"a\""), "a");
    assert_eq!(canonical_property_key_text("'b'"), "b");

    // BigInt digits survive exactly.
    assert_eq!(canonical_property_key_text("42n"), "42");
    assert_eq!(
        canonical_property_key_text("123456789012345678901234567890n"),
        "123456789012345678901234567890"
    );

    // A string literal's CONTENT is the key however it is spelled: whitespace
    // is never padding and is never trimmed, quote characters inside the
    // content are part of the name, and the empty string is a real key. The
    // first version of this helper trimmed and re-unquoted, which renamed
    // `{" a ": 1}`'s key to `a` and made `Object.hasOwn(o, "a")` fold to a
    // silent, diagnostic-free `true`.
    assert_eq!(canonical_property_key_text("\" a \""), " a ");
    assert_eq!(canonical_property_key_text("\"'q'\""), "'q'");
    assert_eq!(canonical_property_key_text("'\"d\"'"), "\"d\"");
    assert_eq!(canonical_property_key_text("\"`t`\""), "`t`");
    assert_eq!(canonical_property_key_text("\"\""), "");
    assert_eq!(canonical_property_key_text("\" 5 \""), " 5 ");
    // Unquoted and not a number: left verbatim, still untrimmed.
    assert_eq!(canonical_property_key_text(" 5 "), " 5 ");
    assert_eq!(canonical_property_key_text(""), "");
    assert_eq!(canonical_property_key_text(" "), " ");

    // A one-character multi-byte text is two BYTES but one CHAR, and must
    // never be read as quoted -- the byte-length guard paired with char
    // delimiter tests is what guarantees it, and slicing it as if quoted would
    // panic on a char boundary rather than merely answer wrongly.
    assert_eq!(canonical_property_key_text("é"), "é");
    assert_eq!(canonical_property_key_text("«x»"), "«x»");
}
