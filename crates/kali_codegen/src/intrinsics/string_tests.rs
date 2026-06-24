use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn supported_static_string_search_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log(\"hello\".includes(\"ell\")); console.log(\"hello\".includes(\"e\", -4)); console.log(\"hello\".indexOf(\"l\", 3)); console.log(\"hello\".indexOf(\"e\", -2)); console.log(\"hello\".lastIndexOf(\"l\")); console.log(\"hello\".lastIndexOf(\"l\", 2)); console.log(\"hello\".lastIndexOf(\"l\", -1));",
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
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const -1"), "{printed}");
}

#[test]
fn supported_static_string_prefix_suffix_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log(\"hello\".startsWith(\"he\")); console.log(\"hello\".startsWith(\"ll\", 2)); console.log(\"hello\".endsWith(\"lo\")); console.log(\"hello\".endsWith(\"ell\", 4)); console.log(\"hello\".endsWith(\"he\", 4));",
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
    assert_eq!(printed.matches("i64.const 1").count(), 4, "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_static_string_search_lowers_omitted_search_as_undefined() {
    let program = parse_and_lower_lir(
        "console.log('hello'.includes()); console.log('undefined value'.startsWith()); console.log('value undefined'.endsWith()); console.log('hello'.indexOf()); console.log('value undefined'.lastIndexOf());",
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
    assert_eq!(printed.matches("i64.const 1").count(), 2, "{printed}");
    assert!(printed.contains("i64.const 6"), "{printed}");
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_static_string_length_lowers_static_string_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.length); console.log('hé'.length); console.log(Object.freeze('world').length);",
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
    assert!(printed.contains("\"5\""), "{printed}");
    assert!(printed.contains("\"2\""), "{printed}");
}

#[test]
fn supported_static_string_slice_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.slice(1)); console.log('hello'.slice(1, 4)); console.log('hello'.slice(1.5, 4.9)); console.log('hello'.slice(-4, -1)); console.log('hello'.slice(4, 1));",
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
    assert!(printed.contains("ello"), "{printed}");
    assert!(printed.contains("ell"), "{printed}");
    assert!(printed.contains("el"), "{printed}");
}

#[test]
fn supported_static_string_substring_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.substring(1)); console.log('hello'.substring(1, 4)); console.log('hello'.substring(1.5, 4.9)); console.log('hello'.substring(4, 1)); console.log('hello'.substring(-2, 2));",
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
    assert!(printed.contains("ello"), "{printed}");
    assert!(printed.contains("ell"), "{printed}");
    assert!(printed.contains("he"), "{printed}");
}

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
    assert_eq!(printed.matches("call 1").count(), 3, "{printed}");
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
fn supported_static_string_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.at()); console.log('hello'.at(1)); console.log('hello'.at(-1)); console.log('hello'.at(99));",
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
    assert!(printed.contains("\"h\""), "{printed}");
    assert!(printed.contains("\"e\""), "{printed}");
    assert!(printed.contains("\"o\""), "{printed}");
    assert!(printed.contains("undefined"), "{printed}");
}

#[test]
fn supported_static_string_char_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.charAt()); console.log('hello'.charAt(1)); console.log('hello'.charAt(-1)); console.log('hello'.charAt(99));",
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
    assert!(printed.contains("\"h\""), "{printed}");
    assert!(printed.contains("\"e\""), "{printed}");
    assert!(printed.contains("\"\""), "{printed}");
}

#[test]
fn supported_static_string_char_code_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.charCodeAt()); console.log('hello'.charCodeAt(1)); console.log('hello'.charCodeAt(-1)); console.log('hello'.charCodeAt(99));",
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
    assert!(printed.contains("104"), "{printed}");
    assert!(printed.contains("101"), "{printed}");
    assert!(printed.contains("NaN"), "{printed}");
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
