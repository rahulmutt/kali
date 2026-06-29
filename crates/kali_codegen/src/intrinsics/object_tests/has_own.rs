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
