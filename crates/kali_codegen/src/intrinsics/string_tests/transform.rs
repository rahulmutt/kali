use super::*;

#[test]
fn supported_static_string_repeat_lowers_ascii_literals() {
    let program = parse_and_lower_lir("console.log('ha'.repeat(3)); console.log('x'.repeat(0));");
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("hahaha"), "{printed}");
}

#[test]
fn supported_static_string_concat_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('he'.concat('llo')); console.log('he'.concat('l', 'lo')); console.log('hello'.concat());",
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("hello"), "{printed}");
    // Match `call 1` at end-of-line so the count is exactly the index-1
    // (console.log) calls and is NOT polluted by the `call 17` substring —
    // `int_to_string` is now called by the always-present growable-int-join
    // synthetic `__join_growable_i64` (Task 5) in every module.
    assert_eq!(printed.matches("call 1\n").count(), 3, "{printed}");
}

#[test]
fn unsupported_static_string_concat_dynamic_operand_is_gated() {
    let program =
        parse_and_lower_lir("const suffix = Deno.args[0]; console.log('he'.concat(suffix));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(5506)
                && diagnostic
                    .message
                    .contains("String.prototype.concat is unavailable")),
        "expected concat gate diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn supported_static_string_trim_family_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('  hello  '.trim()); console.log('  hello  '.trimStart()); console.log('  hello  '.trimEnd()); console.log('  hello  '.trimLeft()); console.log('  hello  '.trimRight());",
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("hello"), "{printed}");
    assert!(printed.contains("hello  "), "{printed}");
    assert!(printed.contains("  hello"), "{printed}");
}

#[test]
fn supported_static_string_case_family_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('HeLLo'.toLowerCase()); console.log('HeLLo'.toUpperCase()); console.log('HeLLo'.toLocaleLowerCase()); console.log('HeLLo'.toLocaleUpperCase());",
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("hello"), "{printed}");
    assert!(printed.contains("HELLO"), "{printed}");
}

#[test]
fn supported_static_string_replace_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello hello'.replace('hello', 'hi')); console.log('abc'.replace('', 'X')); console.log('hello hello'.replaceAll('hello', 'hi')); console.log('abc'.replaceAll('', 'X'));",
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

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("hi hello"), "{printed}");
    assert!(printed.contains("Xabc"), "{printed}");
    assert!(printed.contains("hi hi"), "{printed}");
    assert!(printed.contains("XaXbXcX"), "{printed}");
}

#[test]
fn unsupported_static_string_replace_all_substitution_marker_is_gated() {
    let program = parse_and_lower_lir("console.log('hello'.replaceAll('h', '$&')); ");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(5506)
                && diagnostic
                    .message
                    .contains("String.prototype.replaceAll is unavailable")),
        "expected replaceAll gate diagnostic: {:?}",
        result.diagnostics
    );
}
