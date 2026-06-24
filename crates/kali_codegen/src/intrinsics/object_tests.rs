use crate::*;
use crate::test_support::*;
use wasmparser::Validator;

#[test]
fn object_is_lowers_for_static_primitive_literals() {
    let program = parse_and_lower_lir(
        "const flag = true; const text = \"hello\"; const big = 1n; console.log(Object.is(flag, true)); console.log(Object.is(text, \"hello\")); console.log(Object.is(big, 1n)); console.log(Object.is(-1n, -1n)); console.log(Object.is(-0, +0)); console.log(Object.is(-0, -0)); console.log(Object.is(+0, +0)); console.log(Object.is(Infinity, Infinity)); console.log(Object.is(NaN, NaN)); console.log(Object.is(null, null)); console.log(Object.is(void 0, void 0)); console.log(Object.is(1, 0));",
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn object_is_lowers_for_same_static_reference() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; console.log(Object.is(alias, object)); console.log(Object.is(object, object));",
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
fn object_is_lowers_for_static_member_callable_alias() {
    let program = parse_and_lower_lir(
        "const same = Object.is; const object = { a: 1 }; console.log(same(object, object));",
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
fn object_is_lowers_for_fresh_object_and_array_literals() {
    let program = parse_and_lower_lir(
        "console.log(Object.is({}, {})); console.log(Object.is([], [])); console.log(Object.is({ a: 1 }, { a: 1 })); console.log(Object.is([1], [1]));",
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
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn object_is_lowers_for_same_static_reference_through_object_freeze() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const frozen = Object.freeze(object); console.log(Object.is(frozen, object)); console.log(Object.is(Object.freeze(object), object)); console.log(globalThis[\"Object\"][\"is\"](frozen, object)); console.log(globalThis.Object[\"is\"](frozen, object)); console.log(globalThis[\"Object\"].is(frozen, object));",
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
fn object_is_lowers_for_same_static_member_roots() {
    let program = parse_and_lower_lir(
        "async function main() { console.log(Object.is(globalThis.Object, globalThis.Object)); console.log(Object.is(globalThis[\"Object\"], globalThis[\"Object\"])); console.log(Object.is(globalThis['Object'], globalThis['Object'])); console.log(Object.is(await globalThis.Object, await globalThis.Object)); console.log(Object.is(await globalThis[\"Object\"], await globalThis[\"Object\"])); } main();",
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
fn object_is_lowers_for_bracketed_global_this_object_spellings() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; const frozen = Object.freeze(object); console.log(globalThis[\"Object\"][\"is\"](alias, object)); console.log(globalThis.Object[\"is\"](frozen, object)); console.log(globalThis[\"Object\"].is(alias, object)); console.log(globalThis.Object.is(frozen, object));",
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
fn object_is_lowers_through_parenthesized_same_reference_wrappers() {
    let program = parse_and_lower_lir(
        "const object = { a: 1 }; const alias = object; console.log(Object.is((alias), (((object)))));",
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
fn object_is_lowers_for_unary_plus_wrapped_numeric_literals() {
    let program = parse_and_lower_lir("console.log(Object.is(+1, 1));");
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
