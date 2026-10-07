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
fn an_f64_growable_join_is_refused_until_a_float_join_body_exists() {
    // Task 9 carried item 7: `__join_growable_i64` would print the raw f64
    // bits; Task 10 adds `__join_growable_f64` and replaces this refusal.
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); console.log(a.join(\",\")); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        result.diagnostics.iter().any(|d| d.code == Some(5506)
            && d.message
                .contains("join on a growable array of floating-point numbers")),
        "{:?}",
        result.diagnostics
    );
}
