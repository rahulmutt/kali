use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn number_is_finite_is_integer_and_is_nan_lowers_for_static_primitive_values() {
    let program = parse_and_lower_lir(
        "const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); console.log(isFinite(alias)); console.log(globalThis.isFinite(alias)); console.log(globalThis[\"isFinite\"](alias)); console.log(globalThis[\"isNaN\"](NaN)); console.log(Number.isFinite(alias)); console.log(Number.isInteger(alias)); console.log(Number.isSafeInteger(alias)); console.log(Number.isInteger(1.5)); console.log(Number.isFinite(\"hello\")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis[\"Number\"][\"isNaN\"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis[\"Number\"].isNaN(1)); console.log(globalThis[\"Number\"][\"isFinite\"](alias)); console.log(globalThis[\"Number\"][\"isInteger\"](alias)); console.log(globalThis[\"Number\"][\"isSafeInteger\"](alias)); console.log(globalThis.Number[\"isNaN\"](1)); console.log(globalThis[\"Number\"].isFinite(alias)); console.log(globalThis.Number[\"isInteger\"](alias)); console.log(globalThis[\"Number\"].isSafeInteger(alias)); console.log(Number[\"isFinite\"](alias)); console.log(Number[\"isInteger\"](alias)); console.log(Number[\"isSafeInteger\"](alias)); console.log(Number[\"isNaN\"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(finite(alias));",
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
fn supported_math_and_number_roots_accept_object_freeze_wrappers() {
    let program = parse_and_lower_lir(
        "const zero = 0; const one = 1; console.log(Object.freeze(Math).exp(zero)); console.log(Object.freeze(globalThis[\"Math\"]).log(one)); console.log(Object.freeze(Number).isFinite(zero)); console.log(Object.freeze(globalThis[\"Number\"]).isInteger(one));",
    );
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
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_static_parse_float_integer_slice_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log(parseFloat('42.0px')); console.log(globalThis.parseFloat('-1.2e1tail')); console.log(Number.parseFloat('7.000')); console.log(Object.freeze(globalThis[\"Number\"][\"parseFloat\"])(Object.freeze('6.02e2')));",
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
    assert!(printed.contains("42"), "{printed}");
    assert!(printed.contains("-12"), "{printed}");
    assert!(printed.contains("7"), "{printed}");
    assert!(printed.contains("602"), "{printed}");
}

#[test]
fn static_console_fold_renders_numeric_literals_like_js_and_bigints_as_digits() {
    // The fold's numeric `Literal` arm does NOT see the program's source text:
    // `kali_hir`'s `lower_literal_value` already rewrote it with Rust's
    // `Display for f64`, which never uses exponential notation. Re-parsing and
    // rendering through `format_js_number` is what keeps the fold agreeing with
    // the host and the dynamic lanes (`1e-7`, not `0.0000001`).
    //
    // BigInts must NOT round-trip through the f64 formatter: the 30-digit
    // literal below would come back as `1.2345678901234568e+29`.
    let program = parse_and_lower_lir(
        "console.log(1e-7); console.log(1e21); console.log(1e20); console.log(5); console.log(42n); console.log(123456789012345678901234567890n);",
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
    assert!(printed.contains("1e-7"), "{printed}");
    assert!(!printed.contains("0.0000001"), "{printed}");
    assert!(printed.contains("1e+21"), "{printed}");
    // Just inside the threshold: no exponent, and a plain integer stays plain.
    assert!(printed.contains("100000000000000000000"), "{printed}");
    assert!(printed.contains("42n"), "{printed}");
    assert!(
        printed.contains("123456789012345678901234567890n"),
        "{printed}"
    );
}

#[test]
fn bigint_literal_text_is_recognized_and_ordinary_numbers_are_not() {
    assert!(is_bigint_literal_text("42n"));
    assert!(is_bigint_literal_text("123456789012345678901234567890n"));
    assert!(!is_bigint_literal_text("42"));
    assert!(!is_bigint_literal_text("n"));
    assert!(!is_bigint_literal_text("0.0000001"));
    assert!(!is_bigint_literal_text("\"tenn\""));
}

#[test]
fn static_console_fold_renders_negated_literals_like_js_and_keeps_bigints_exact() {
    // The unary `+`/`-` arm is the `Literal` arm's twin and used Rust's
    // `Display for f64` too, so before this guard the SIGN of a literal decided
    // whether it rendered like JavaScript or like Rust. The BigInt cases negate
    // textually: the 30-digit literal below rendered as
    // `-123456789012345680000000000000` when it reached the f64 fall-through,
    // and `-42n` lost the `n` entirely on the i64 branch.
    let program = parse_and_lower_lir(
        "console.log(-1e-7); console.log(-1e21); console.log(-1e20); console.log(-5); console.log(-42n); console.log(-123456789012345678901234567890n); console.log(- -42n); console.log(-0n);",
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
    assert!(printed.contains("-1e-7"), "{printed}");
    assert!(!printed.contains("-0.0000001"), "{printed}");
    assert!(printed.contains("-1e+21"), "{printed}");
    // Just inside the threshold, and a plain negative integer, stay plain.
    assert!(printed.contains("-100000000000000000000"), "{printed}");
    assert!(printed.contains("-5"), "{printed}");
    assert!(printed.contains("-42n"), "{printed}");
    assert!(
        printed.contains("-123456789012345678901234567890n"),
        "{printed}"
    );
    // Double negation is closed under the same arm.
    assert!(printed.contains("42n"), "{printed}");
    // BigInt has no negative zero: JS prints `0n` for `-0n`, and static
    // `Map`/`Set` key texts must not treat `-0n` and `0n` as two keys.
    assert!(!printed.contains("-0n"), "{printed}");
}
