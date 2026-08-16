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
fn canonical_property_key_text_reads_the_two_slots_opposite_ways() {
    use super::super::canonical_property_key_text;
    use super::super::KeyTextSlot::{Expression, ObjectLiteralKey};

    // Object-literal KEY slot: quoted means "this was a number, stringified by
    // HIR with Rust's `Display`"; unquoted means "this was a string or an
    // identifier". Both spellings of the same key must land on one text.
    assert_eq!(
        canonical_property_key_text("\"1000000000000000000000\"", ObjectLiteralKey),
        "1e+21"
    );
    assert_eq!(
        canonical_property_key_text("\"0.0000001\"", ObjectLiteralKey),
        "1e-7"
    );
    assert_eq!(canonical_property_key_text("\"5\"", ObjectLiteralKey), "5");
    assert_eq!(canonical_property_key_text("a", ObjectLiteralKey), "a");
    // A STRING key keeps its own text: `{"1000000000000000000000": 1}` is not
    // the same property as `{1e21: 1}`, and renumbering it would fix one wrong
    // answer by creating another.
    assert_eq!(
        canonical_property_key_text("1000000000000000000000", ObjectLiteralKey),
        "1000000000000000000000"
    );

    // EXPRESSION slot: the quoting convention is inverted.
    assert_eq!(
        canonical_property_key_text("1000000000000000000000", Expression),
        "1e+21"
    );
    assert_eq!(canonical_property_key_text("0.0000001", Expression), "1e-7");
    assert_eq!(
        canonical_property_key_text("\"1000000000000000000000\"", Expression),
        "1000000000000000000000"
    );
    assert_eq!(canonical_property_key_text("\"a\"", Expression), "a");
    assert_eq!(canonical_property_key_text("'b'", Expression), "b");
    // `String(42n)` is "42", and the digits are taken textually so a BigInt
    // past `f64`'s exact range keeps every one of them.
    assert_eq!(canonical_property_key_text("42n", Expression), "42");
    assert_eq!(
        canonical_property_key_text("123456789012345678901234567890n", Expression),
        "123456789012345678901234567890"
    );
}

#[test]
fn canonical_property_key_text_never_renames_a_string_key() {
    use super::super::canonical_property_key_text;
    use super::super::KeyTextSlot::{Expression, ObjectLiteralKey};

    // `lower_property_name` quotes NUMERIC keys and nothing else, so in the key
    // slot a quoted-looking text whose inner is not a number is a string key
    // whose quote characters are part of the NAME, and whitespace is never
    // padding. Trimming or unquoting those renames the property -- the first
    // version of this helper did both, turning `{" a ": 1}`'s key into `a` and
    // `{"'q'": 1}`'s into `q`, which made `Object.hasOwn(o, "a")` fold to a
    // silent, diagnostic-free `true`.
    assert_eq!(canonical_property_key_text(" a ", ObjectLiteralKey), " a ");
    assert_eq!(canonical_property_key_text("'q'", ObjectLiteralKey), "'q'");
    assert_eq!(
        canonical_property_key_text("\"d\"", ObjectLiteralKey),
        "\"d\""
    );
    assert_eq!(canonical_property_key_text("`t`", ObjectLiteralKey), "`t`");
    assert_eq!(canonical_property_key_text("", ObjectLiteralKey), "");
    assert_eq!(canonical_property_key_text(" ", ObjectLiteralKey), " ");
    // Unquoted in the key slot means "string key", so a numeric-LOOKING one is
    // still never renumbered.
    assert_eq!(canonical_property_key_text(" 5 ", ObjectLiteralKey), " 5 ");
    assert_eq!(canonical_property_key_text("5", ObjectLiteralKey), "5");
    assert_eq!(
        canonical_property_key_text("1000000000000000000000", ObjectLiteralKey),
        "1000000000000000000000"
    );
    // ... while the QUOTED twin in the same slot is the genuine numeric key and
    // does get renumbered. The two must not collide: this pair is exactly
    // `{1e21: 1, "1000000000000000000000": 2}`, two distinct JS properties.
    assert_eq!(
        canonical_property_key_text("\"1000000000000000000000\"", ObjectLiteralKey),
        "1e+21"
    );
    assert_ne!(
        canonical_property_key_text("\"1000000000000000000000\"", ObjectLiteralKey),
        canonical_property_key_text("1000000000000000000000", ObjectLiteralKey)
    );

    // Expression slot: a string literal's content is the key however it is
    // spelled, quotes and spaces included, and nothing unquoted is trimmed.
    assert_eq!(canonical_property_key_text("\" a \"", Expression), " a ");
    assert_eq!(canonical_property_key_text("\"'q'\"", Expression), "'q'");
    assert_eq!(canonical_property_key_text("'\"d\"'", Expression), "\"d\"");
    assert_eq!(canonical_property_key_text("\"`t`\"", Expression), "`t`");
    assert_eq!(canonical_property_key_text("\"\"", Expression), "");
    assert_eq!(canonical_property_key_text("\" 5 \"", Expression), " 5 ");
    assert_eq!(canonical_property_key_text(" 5 ", Expression), " 5 ");

    // NEGATIVE probes: a false positive must fail this test, not just a false
    // negative. `Object.hasOwn({" a ": 1}, "a")` is `false` in JS.
    assert_ne!(
        canonical_property_key_text(" a ", ObjectLiteralKey),
        canonical_property_key_text("\"a\"", Expression)
    );
    assert_ne!(
        canonical_property_key_text("'q'", ObjectLiteralKey),
        canonical_property_key_text("\"q\"", Expression)
    );
    assert_ne!(
        canonical_property_key_text(" 5 ", ObjectLiteralKey),
        canonical_property_key_text("5", Expression)
    );
    assert_ne!(
        canonical_property_key_text("1000000000000000000000", ObjectLiteralKey),
        canonical_property_key_text("1000000000000000000000", Expression)
    );

    // POSITIVE pairs that must still meet: the probe spelling and the stored
    // spelling of one key land on one text.
    assert_eq!(
        canonical_property_key_text(" a ", ObjectLiteralKey),
        canonical_property_key_text("\" a \"", Expression)
    );
    assert_eq!(
        canonical_property_key_text("'q'", ObjectLiteralKey),
        canonical_property_key_text("\"'q'\"", Expression)
    );
    assert_eq!(
        canonical_property_key_text("\"1000000000000000000000\"", ObjectLiteralKey),
        canonical_property_key_text("1000000000000000000000", Expression)
    );
    assert_eq!(
        canonical_property_key_text("1000000000000000000000", ObjectLiteralKey),
        canonical_property_key_text("\"1000000000000000000000\"", Expression)
    );
}

#[test]
fn canonical_property_key_text_renumbers_only_hir_double_quoted_keys() {
    use super::super::canonical_property_key_text;
    use super::super::KeyTextSlot::{Expression, ObjectLiteralKey};

    // HIR's numeric marker is specifically a DOUBLE quote
    // (`lower_property_name`'s `Number` arm is `format!("\"{}\"", ...)`), so a
    // string key whose own content is a SINGLE- or BACKTICK-quoted number is
    // still just a string key. Accepting any quote character here renumbered
    // them: `{"'5'": 1}` answered `Object.hasOwn(o, "'5'")` with `false` and
    // `Object.hasOwn(o, 5)` with `true`, while member access still found the
    // key -- one program contradicting itself in a single output.
    assert_eq!(canonical_property_key_text("'5'", ObjectLiteralKey), "'5'");
    assert_eq!(canonical_property_key_text("`7`", ObjectLiteralKey), "`7`");
    assert_eq!(
        canonical_property_key_text("'0.0000001'", ObjectLiteralKey),
        "'0.0000001'"
    );
    assert_eq!(
        canonical_property_key_text("'1e21'", ObjectLiteralKey),
        "'1e21'"
    );
    // `parse_numeric_literal_value` accepts spellings JS never produces
    // (`inf`, `nan`, `+5`, `5.`) and strips an `n` suffix. None of them may
    // reach a key-slot decision: a `PropertyName::Number` text is always
    // `f64::to_string()` output, so anything else quoted here is a string key.
    assert_eq!(
        canonical_property_key_text("'42n'", ObjectLiteralKey),
        "'42n'"
    );
    assert_eq!(
        canonical_property_key_text("'inf'", ObjectLiteralKey),
        "'inf'"
    );
    assert_eq!(
        canonical_property_key_text("`nan`", ObjectLiteralKey),
        "`nan`"
    );

    // NEGATIVE probes: the false POSITIVE must fail this test too, not only the
    // false negative. `Object.hasOwn({"'5'": 1}, 5)` is `false` in JS.
    assert_ne!(
        canonical_property_key_text("'5'", ObjectLiteralKey),
        canonical_property_key_text("5", Expression)
    );
    assert_ne!(
        canonical_property_key_text("'5'", ObjectLiteralKey),
        canonical_property_key_text("\"5\"", Expression)
    );
    assert_ne!(
        canonical_property_key_text("`7`", ObjectLiteralKey),
        canonical_property_key_text("7", Expression)
    );
    assert_ne!(
        canonical_property_key_text("'42n'", ObjectLiteralKey),
        canonical_property_key_text("42n", Expression)
    );
    assert_ne!(
        canonical_property_key_text("'inf'", ObjectLiteralKey),
        canonical_property_key_text("\"Infinity\"", Expression)
    );
    // ... and the matching POSITIVE pairs still meet.
    assert_eq!(
        canonical_property_key_text("'5'", ObjectLiteralKey),
        canonical_property_key_text("\"'5'\"", Expression)
    );
    assert_eq!(
        canonical_property_key_text("`7`", ObjectLiteralKey),
        canonical_property_key_text("\"`7`\"", Expression)
    );

    // A double-quoted key whose inner IS a number is the genuine numeric key
    // and still renumbers -- including `{1e999: 1}`, which HIR stringifies to
    // `inf` and JS names `Infinity`. This shape is live; the double-quote gate
    // must not be "simplified" into breaking it.
    assert_eq!(canonical_property_key_text("\"5\"", ObjectLiteralKey), "5");
    assert_eq!(
        canonical_property_key_text("\"inf\"", ObjectLiteralKey),
        "Infinity"
    );
    assert_eq!(
        canonical_property_key_text("\"inf\"", ObjectLiteralKey),
        canonical_property_key_text("\"Infinity\"", Expression)
    );

    // One-character multi-byte keys are two BYTES but one CHAR, and must never
    // be read as quoted (the byte length guard paired with char delimiter
    // tests is what guarantees it).
    assert_eq!(canonical_property_key_text("é", ObjectLiteralKey), "é");
    assert_eq!(canonical_property_key_text("é", Expression), "é");
    assert_eq!(canonical_property_key_text("«x»", ObjectLiteralKey), "«x»");
}

#[test]
fn object_literal_key_renumbers_exactly_the_spellings_hir_can_write() {
    use super::super::canonical_property_key_text;
    use super::super::KeyTextSlot::{Expression, ObjectLiteralKey};
    use kali_common::js_number::format_js_number;

    // THE INVARIANT, stated directly: in the key slot a double-quoted inner is
    // renumbered IF AND ONLY IF it is exactly what `lower_property_name` could
    // have written for a numeric property name. The oracle below is generated
    // the way HIR generates it -- from VALUES through
    // `if value == 0.0 { "0" } else { value.to_string() }` -- so this pins the
    // property rather than restating the implementation's spelling checks.
    let hir_writes = |value: f64| {
        if value == 0.0 {
            "0".to_string()
        } else {
            value.to_string()
        }
    };

    // (1) EVERY spelling in the image renumbers, and renumbers to String(value).
    for value in [
        0.0,
        -0.0, // `{[-0]: 1}`: the parser hands HIR a signed zero, HIR writes "0".
        1.0,
        5.0,
        0.5,
        42.0,
        5000.0,
        1234.5,
        1e20,
        1e21,
        1e-6,
        1e-7,
        9007199254740992.0,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::INFINITY, // `{1e999: 1}`: HIR writes "inf", JS names it Infinity.
        // NEGATIVES ARE REACHABLE, through the parser's computed-key folding of
        // a unary minus (`{[-1]: 1}`). Leaving them out of this generated set is
        // what let an unsound `is_sign_negative` guard survive a review round:
        // the counter-corpus below hand-asserted the guard as if it were the
        // invariant, and nothing generated from VALUES contradicted it.
        -1.0,
        -1.5,
        -5.0,
        -0.5,
        -1234.5,
        -1e21,
        -1e-7,
        f64::MIN,
        f64::NEG_INFINITY, // `{[-1e999]: 1}` -> "-inf" -> JS "-Infinity".
    ] {
        let text = format!("\"{}\"", hir_writes(value));
        assert_eq!(
            canonical_property_key_text(&text, ObjectLiteralKey),
            format_js_number(value),
            "{text}"
        );
    }

    // (2) NOTHING outside the image renumbers -- each is a string key whose name
    // includes its own quotes. These are the six ways `parse_numeric_literal_value`'s
    // language is wider than `Display for f64`'s image, plus the unreachable
    // values: exponent, sign prefix, leading zero, trailing/leading dot,
    // `n` suffix, and the non-literal values NaN / negative.
    for inner in [
        "1e21", "1e-7", "5e3", "5E3", // exponent: Display never emits one
        "+5",  // sign PREFIX on a positive value: Display never writes one.
        // (`-5` is absent on purpose -- it IS in the image, see above.)
        "05", "00", // leading zero
        "5.", ".5", "0.0", // trailing / leading dot, redundant fraction
        "42n", // `n` suffix
        "NaN", "nan", // no numeric literal denotes NaN
        // `-0` is genuinely unreachable AS A SPELLING even though negatives are
        // reachable as values: the parser folds `{[-0]: 1}` to a signed zero and
        // HIR collapses both zeros to "0". `-5` and `-inf` are NOT here -- they
        // are in the generated image set above, because `{[-5]: 1}` and
        // `{[-1e999]: 1}` write exactly those.
        "-0", "infinity", // Display writes "inf", never this
        " 5", "5 ", "1_000", "0x10", "", // not numbers at all
    ] {
        let text = format!("\"{inner}\"");
        assert_eq!(
            canonical_property_key_text(&text, ObjectLiteralKey),
            text,
            "inner {inner:?} must be left verbatim"
        );
    }

    // (3) The false-POSITIVE direction, cross-slot: a string key spelled like a
    // number must not answer a numeric probe. `Object.hasOwn({'"1e21"': 1}, 1e21)`
    // is `false` in JS.
    for (inner, probe) in [
        ("1e21", "1000000000000000000000"),
        ("1e-7", "0.0000001"),
        ("+5", "5"),
        ("05", "5"),
        ("5.", "5"),
        ("42n", "42n"),
        ("NaN", "NaN"),
        // No ("-5", "-5") pair: `{'"-5"': 1}` and `{[-5]: 1}` lower to the
        // IDENTICAL text `"-5"`, so that one is undecidable here, exactly like
        // `"3"`, `"5"` and `"0"` -- not a collision this helper may rule on.
    ] {
        assert_ne!(
            canonical_property_key_text(&format!("\"{inner}\""), ObjectLiteralKey),
            canonical_property_key_text(probe, Expression),
            "string key {inner:?} must not collide with the number {probe:?}"
        );
    }
}
