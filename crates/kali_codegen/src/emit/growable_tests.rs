//! Growable-runtime-arrays spec A-15: the hand-emitted growable synthetics
//! validate, and the growable emitters produce valid wasm for an f64 array.
//! (Codegen unit tests run without repr inference, so the table entries
//! inference would write are set by hand.)

use crate::test_support::{compile_and_measure, parse_and_lower_lir, sample_program};
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};

pub(crate) fn exported_function_names(bytes: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        if let Ok(wasmparser::Payload::ExportSection(reader)) = payload {
            for export in reader {
                let export = export.expect("export entry");
                if export.kind == wasmparser::ExternalKind::Func {
                    names.push(export.name.to_string());
                }
            }
        }
    }
    names
}

pub(crate) fn ctx_with_growable(func: &str, name: &str, elem: kali_common::Repr) -> CodegenCtx {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.repr_table.set_growable_array_binding(func, name);
    ctx.repr_table.set_array_binding(func, name);
    if elem != kali_common::Repr::I64 {
        ctx.repr_table.set_array_element(func, name, elem);
    }
    ctx
}

#[test]
fn every_module_carries_the_growable_bounds_guard_and_validates() {
    let (bytes, _) = compile_and_measure(&sample_program());
    let names = exported_function_names(&bytes);
    assert!(
        names.iter().any(|n| n == "__growable_elem_addr"),
        "{names:?}"
    );
}

#[test]
fn an_f64_growable_array_push_read_and_write_lower_to_valid_wasm() {
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); a.push(2); let i = 1; a[i] = 0.25; console.log(a[0], a[i], a.length); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn an_f64_growable_join_lowers_through_the_float_join_body() {
    // Task 10 replaces Task 9's interim refusal (carried item 7): an f64
    // growable `join` calls `__join_growable_f64`, which renders each slot
    // through `float_to_string` (spec §3.5, A-12).
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); console.log(a.join(\",\")); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn a_float_index_on_a_growable_write_is_refused_not_lowered_to_invalid_wasm() {
    // Task 9 fix round 1, I1: the float-index refusal covers writes as well
    // as reads (it lived only on the read lane; a write emitted an f64 under
    // the index slot, and the module failed to load).
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::I64);
    // Inference would make `f` an f64 scalar; set by hand (no inference here).
    ctx.repr_table
        .set_scalar("main", "f", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1); let f = 0.5; a[f] = 9; console.log(a.length); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        result.diagnostics.iter().any(|d| d.code == Some(5506)
            && d.message
                .contains("indexing a growable array with a floating-point value")),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn a_growable_index_write_stores_through_the_store_time_bounds_guard() {
    // Task 9 fix round 1, C1: the value is evaluated BEFORE the bounds check
    // and the slot address (a right-hand side that grows the array moves the
    // data block), so the write calls `__growable_store`, never
    // `__growable_elem_addr`.
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::I64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1); let i = 0; a[i] = 7; console.log(a.length); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    let names = exported_function_names(&result.wasm_bytes);
    assert!(names.iter().any(|n| n == "__growable_store"), "{names:?}");
}

#[test]
fn every_module_carries_the_growable_method_synthetics_and_validates() {
    let (bytes, _) = compile_and_measure(&sample_program());
    let names = exported_function_names(&bytes);
    for name in [
        "__growable_pop",
        "__growable_find",
        "__growable_slice",
        "__join_growable_f64",
    ] {
        assert!(
            names.iter().any(|n| n == name),
            "{name} missing from {names:?}"
        );
    }
}

#[test]
fn f64_pop_search_slice_and_join_lower_to_valid_wasm() {
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    ctx.repr_table.set_growable_array_binding("main", "t");
    ctx.repr_table.set_array_binding("main", "t");
    ctx.repr_table
        .set_array_element("main", "t", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); a.push(0.25); const t = a.slice(0, 1); console.log(a.indexOf(0.25), a.includes(1.5), a.join(\",\"), t.length, a.pop()); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn an_f64_growable_index_write_used_as_a_value_lowers_to_valid_wasm() {
    // Task 9 re-review carried item: the assignment node is the value of an
    // outer store (`f[1] = (f[0] = 0.25)`) or a declaration (`const r = (f[0]
    // = …)`); `is_float_valued` must see it as f64, or the slot encoder
    // converts an f64 as if it were an i64 and the module fails to load.
    let mut ctx = ctx_with_growable("main", "f", kali_common::Repr::F64);
    ctx.repr_table
        .set_scalar("main", "r", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const f = []; f.push(0.5); f.push(1.5); f[1] = (f[0] = 0.25); const r = (f[0] = f[1] * 2); console.log(r, f[0], f[1]); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn an_extra_pop_or_slice_argument_is_refused_not_dropped() {
    // Task 10 review I1, the codegen belt: the lanes evaluate no `pop`
    // argument and only two `slice` bounds, so more is refused (inference
    // refuses first under `check` and `run`).
    for (src, needle) in [
        (
            "function main() { const a = []; a.push(1); a.pop(1); console.log(a.length); } main();",
            "`.pop()` with an argument on the growable array `a` in `main`",
        ),
        (
            "function main() { const a = []; a.push(1); console.log(a.slice(0, 1, 2).length); } main();",
            "`.slice()` with more than two arguments on the growable array `a` in `main`",
        ),
    ] {
        let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::I64);
        let result = lower_lir_to_wasm(&mut ctx, &parse_and_lower_lir(src));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == Some(5506) && d.message.contains(needle)),
            "{src}: {:?}",
            result.diagnostics
        );
    }
}
