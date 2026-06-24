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
