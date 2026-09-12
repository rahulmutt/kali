//! Task 8's `allocation_ctor_unshadowed` guard (`crates/kali_codegen/src/emit/call.rs`):
//! a bare `Array`/`Uint8Array` allocation callee must decline routing to the
//! shared `__alloc` helper when a user binding of the same name shadows it,
//! but a `globalThis`-qualified callee must ALWAYS route, because qualifying
//! through `globalThis` names the real builtin no matter what bare-name
//! binding exists. Both facts were regressions found by code review after
//! this task's first landing (round 1: the bare-name case was unguarded at
//! all; round 2: the guard, once added, over-blocked the qualified
//! spelling) -- these two tests pin the corrected behaviour so neither
//! regresses silently again. Deliberately structural (wasm-shape) rather
//! than an oracle `.toml` pin: Task 2 owns
//! `runtime/inline_allocation_value_position.toml` and its expected case set
//! is fixed.
use super::*;

/// Locates the wasm function index wasmprinter assigns to the export named
/// `export_name`, duplicated from `alloc_helper.rs` rather than shared,
/// because `super::*` re-exports items but not private free functions from a
/// sibling test module.
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

/// Extracts the printed body of the function declared `(func (;{index};)
/// ...)`, from its declaration line up to (excluding) the next top-level
/// `(func ...)` declaration. Needed because the module unconditionally emits
/// several other synthetic helpers (string/array runtime support) that
/// themselves call `__alloc` -- a whole-module substring search for "call
/// {alloc_index}" finds those too, which is a false positive for a test that
/// means to ask "did THIS program's own top-level code call the allocator".
fn function_body(text: &str, index: u32) -> String {
    let decl_needle = format!("(func (;{index};) (type ");
    let mut body = Vec::new();
    let mut started = false;
    for line in text.lines() {
        if line.trim_start().starts_with(&decl_needle) {
            started = true;
            body.push(line);
            continue;
        }
        if started {
            // wasmprinter indents every top-level module item (including
            // each function declaration) by exactly two spaces, so a line
            // starting a NEW `(func ...)` at that indentation marks the end
            // of this one.
            if line.starts_with("  (func ") {
                break;
            }
            body.push(line);
        }
    }
    assert!(
        started,
        "missing function declaration (index {index}):\n{text}"
    );
    body.join("\n")
}

#[test]
fn globalthis_qualified_uint8array_allocation_routes_under_a_shadowing_binding() {
    // A module-level `function Uint8Array` shadows the bare name in every
    // codegen namespace `allocation_ctor_unshadowed` consults. The argument
    // position (`f(new globalThis.Uint8Array(3))`) reaches Task 8's Site 1
    // Arm A -- the declarator/assignment/`.fill`-receiver lanes never see
    // this program at all, since the allocation is not bound by a
    // declarator here. Before the round-2 fix, this exact shape measured
    // kali `0` where node prints `3` (the qualified spelling was
    // over-blocked by the bare-name shadow it should be immune to).
    let src = "function Uint8Array(n) { return n + 1; } \
               function f(x) { return 1; } \
               f(new globalThis.Uint8Array(3));";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.arena_table.set_arena_eligible("_start");
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
    let start_index = exported_function_index(&text, "_start");
    let start_body = function_body(&text, start_index);
    let call_needle = format!("call {alloc_index}");
    assert!(
        start_body.lines().any(|line| line.trim() == call_needle),
        "expected `new globalThis.Uint8Array(3)` to call the shared __alloc \
         helper (index {alloc_index}) even under a same-named `function \
         Uint8Array` binding, but `_start`'s body does not call it:\n{start_body}"
    );
}

#[test]
fn bare_uint8array_call_does_not_route_under_a_shadowing_binding() {
    // The round-1 regression this guard closes: without
    // `allocation_ctor_unshadowed`, a bare `Uint8Array(3)` call in argument
    // position would route to `__alloc` even though `Uint8Array` here names
    // the user's own function, not the (nonexistent, in kali) bare builtin.
    // Pins the negative: the shadowed bare call must NOT call `__alloc` at
    // all -- it must be an ordinary call to the resolved user function.
    let src = "function Uint8Array(n) { return n + 1; } \
               function f(x) { return 1; } \
               f(Uint8Array(3));";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.arena_table.set_arena_eligible("_start");
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
    let start_index = exported_function_index(&text, "_start");
    let start_body = function_body(&text, start_index);
    let call_needle = format!("call {alloc_index}");
    assert!(
        !start_body.lines().any(|line| line.trim() == call_needle),
        "expected a shadowed bare `Uint8Array(3)` call to NOT route through \
         the __alloc helper (index {alloc_index}) from `_start` -- it should \
         call the user's own function instead:\n{start_body}"
    );
}
