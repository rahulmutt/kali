//! Per-site arena routing for runtime string producers (fasta Spec 7 Task 4c+).
//!
//! These pin the codegen-side half of the string-site "both-sides oracle": a
//! `.join(...)` site the `kali_mir` escape gate proved iteration-local
//! (`ArenaTable::arena_string_site(fn, ordinal)`) must route to the resettable
//! `__join_arena` twin; every other site keeps the global `__join`. The
//! ordinal a join node is queried under is derived by
//! `crate::lower::string_site_preorder_ordinals`, which MUST enumerate the same
//! per-function pre-order string-site stream `kali_mir`'s
//! `OwnershipAnalyzer::arena_collect_string_sites` numbers (see that function's
//! "THE STRING-SITE ORDINAL RULE" doc comment). The mixed-ordinal test pins the
//! counter-advance for `+` nodes, which sit BEFORE their join operand in
//! pre-order.
use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

/// Locates the wasm function index wasmprinter assigns to `export_name` by
/// scanning the printed text for its `(export "name" (func N))` line (there is
/// no wasm "name" custom section in this codegen — see the identical helper in
/// `call_tests/alloc_helper.rs`). Returns `None` when the export is absent, so
/// the negative routing case can assert the arena twin is never CALLED even
/// though it is always EMITTED (and hence always exported).
fn exported_function_index(text: &str, export_name: &str) -> Option<u32> {
    let needle = format!("(export \"{export_name}\" (func ");
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with(&needle))?;
    line.trim_start()
        .trim_start_matches(&needle)
        .split(')')
        .next()
        .and_then(|digits| digits.trim().parse::<u32>().ok())
}

/// True iff the printed module contains a standalone `call <index>` line — a
/// real call instruction to that function index somewhere in the code section.
fn calls_function(text: &str, index: u32) -> bool {
    let needle = format!("call {index}");
    text.lines().any(|line| line.trim() == needle)
}

/// Build the wasm for a one-function program, priming the `ReprTable`/
/// `ArenaTable` the real driver's `kali_types`/`kali_mir` passes would produce
/// (this crate's `parse_and_lower_lir` runs neither), then print it.
fn compile_join_program(src: &str, grant_ordinals: &[u32]) -> String {
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    // `a` is a linear-memory array binding with proven String elements, so the
    // runtime-join recognizer (`runtime_join_call_parts`) fires instead of the
    // static-fold lane.
    ctx.repr_table.set_array_binding("r", "a");
    ctx.repr_table
        .set_array_element("r", "a", kali_common::Repr::String);
    ctx.arena_table.set_arena_eligible("r");
    for ord in grant_ordinals {
        ctx.arena_table.set_arena_string_site("r", *ord);
    }
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
    wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm")
}

/// A join in a loop whose result is dropped into `console.log` is the single
/// string site (ordinal 0). With the grant, it routes to the resettable
/// `__join_arena` twin.
#[test]
fn granted_join_in_loop_routes_to_arena_twin() {
    let src = "function r(a, n) { while (n > 0) { console.log(a.join(\"\")); n = n - 1; } }";
    let text = compile_join_program(src, &[0]);
    let arena_index =
        exported_function_index(&text, "__join_arena").expect("__join_arena must be exported");
    assert!(
        calls_function(&text, arena_index),
        "granted join site should call __join_arena (index {arena_index}):\n{text}"
    );
}

/// The SAME site with NO grant keeps the global `__join` — fail-closed. The
/// arena twin is still emitted/exported but never CALLED.
#[test]
fn ungranted_join_stays_on_global_join() {
    let src = "function r(a, n) { while (n > 0) { console.log(a.join(\"\")); n = n - 1; } }";
    let text = compile_join_program(src, &[]);
    let join_index = exported_function_index(&text, "__join").expect("__join must be exported");
    let arena_index =
        exported_function_index(&text, "__join_arena").expect("__join_arena must be exported");
    assert!(
        calls_function(&text, join_index),
        "ungranted join site should call the global __join (index {join_index}):\n{text}"
    );
    assert!(
        !calls_function(&text, arena_index),
        "ungranted join site must NOT call __join_arena (index {arena_index}):\n{text}"
    );
}

/// `console.log(a.join("") + "!")`: the `+` is numbered FIRST (ordinal 0,
/// parent-first pre-order), the join SECOND (ordinal 1). Granting ordinal 1
/// routes the join to the arena twin — pinning that the `+` node advanced the
/// string-site counter even though 4c never queries `+` ordinals.
#[test]
fn mixed_plus_join_numbers_join_at_ordinal_one() {
    let src =
        "function r(a, n) { while (n > 0) { console.log(a.join(\"\") + \"!\"); n = n - 1; } }";
    let text = compile_join_program(src, &[1]);
    let arena_index =
        exported_function_index(&text, "__join_arena").expect("__join_arena must be exported");
    assert!(
        calls_function(&text, arena_index),
        "join under a `+` is ordinal 1; granting 1 must route it to __join_arena (index {arena_index}):\n{text}"
    );
}

/// Discriminating control for the mixed case: granting ONLY ordinal 0 (the `+`)
/// leaves the join (ordinal 1) on the global `__join`. If the `+` did NOT
/// advance the counter, the join would be ordinal 0 and this grant would
/// wrongly route it — so this pins the counter-advance from the other side.
#[test]
fn mixed_plus_grant_does_not_leak_to_join() {
    let src =
        "function r(a, n) { while (n > 0) { console.log(a.join(\"\") + \"!\"); n = n - 1; } }";
    let text = compile_join_program(src, &[0]);
    let arena_index =
        exported_function_index(&text, "__join_arena").expect("__join_arena must be exported");
    assert!(
        !calls_function(&text, arena_index),
        "granting the `+` ordinal (0) must NOT route the join (ordinal 1) to __join_arena (index {arena_index}):\n{text}"
    );
}
