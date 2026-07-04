//! Structural coverage for the synthetic `__alloc` bump allocator (Task 2 of
//! the reclaiming-allocator plan): object and array allocation sites must
//! call the one shared `__alloc` function rather than inlining their own
//! `__heap` bump.
use super::*;

/// Locates the wasm function index wasmprinter assigns to the export named
/// `export_name` by scanning the printed text for its
/// `(export "name" (func N))` line. There is no wasm "name" custom section in
/// this codegen (function identity is only visible via exports), so this is
/// the honest way to recover a function's real index from the emitted module.
/// Panics with the full text on failure so a genuine miscompile is easy to
/// diagnose from the test's own output.
fn exported_function_index(text: &str, export_name: &str) -> u32 {
    let needle = format!("(export \"{export_name}\" (func ");
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with(&needle))
        .unwrap_or_else(|| panic!("missing export \"{export_name}\":\n{text}"));
    line.trim_start()
        .trim_start_matches(&needle)
        .split(')')
        .next()
        .and_then(|digits| digits.trim().parse::<u32>().ok())
        .unwrap_or_else(|| panic!("could not parse function index from: {line}"))
}

#[test]
fn alloc_function_is_emitted_with_the_i32_to_i32_bump_signature() {
    // Even a program with no allocation at all must still emit `__alloc`: it
    // is a fixed synthetic slot registered unconditionally in `all_functions`,
    // not lowered on demand from a call site.
    let program = parse_and_lower_lir("console.log(1);");
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

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    // Just confirms the export exists (panics otherwise) and pins the
    // `(param i32) (result i32)` signature next to it, so a regression back
    // to the repr-directed default (i64) signature fails loudly here instead
    // of only showing up as a validation error downstream.
    let index = exported_function_index(&text, "__alloc");
    let decl_needle = format!("(func (;{index};) (type ");
    let decl_line = text
        .lines()
        .find(|line| line.trim_start().starts_with(&decl_needle))
        .unwrap_or_else(|| panic!("missing __alloc function declaration (index {index}):\n{text}"));
    assert!(
        decl_line.contains("(param i32)") && decl_line.contains("(result i32)"),
        "__alloc should be `(i32) -> i32`, got: {decl_line}"
    );
}

#[test]
fn array_allocation_calls_shared_alloc_helper() {
    let src = "const a = new Array(3); a[0] = 1;";
    let program = parse_and_lower_lir(src);
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

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    let alloc_index = exported_function_index(&text, "__alloc");

    let call_needle = format!("call {alloc_index}");
    assert!(
        text.lines().any(|line| line.trim() == call_needle),
        "expected array allocation to call the shared __alloc helper (index {alloc_index}):\n{text}"
    );
}

#[test]
fn object_allocation_calls_shared_alloc_helper() {
    // `kali_codegen`'s own test pipeline (`parse_and_lower_lir`) does not run
    // the `kali_types` shape inference that populates `ReprTable` in the real
    // `kali_cli` compiler driver (that crate is not even a dev-dependency
    // here). Drive `emit_object_allocation` the same way the real compiler
    // does: construct the `ReprTable` entries the inference would have
    // produced for `const o = { a: 1, b: 2 };` — an interned two-field shape,
    // and `o`'s own scalar repr pointing at it — before lowering.
    let src = "const o = { a: 1, b: 2 }; console.log(o.a);";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let shape = ctx.repr_table.intern_shape(vec![
        ("a".to_string(), kali_common::Repr::I64),
        ("b".to_string(), kali_common::Repr::I64),
    ]);
    ctx.repr_table
        .set_scalar("_start", "o", kali_common::Repr::Object(shape));

    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    let alloc_index = exported_function_index(&text, "__alloc");

    let call_needle = format!("call {alloc_index}");
    assert!(
        text.lines().any(|line| line.trim() == call_needle),
        "expected object allocation to call the shared __alloc helper (index {alloc_index}):\n{text}"
    );
}
