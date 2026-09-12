//! Task 8's `allocation_ctor_unshadowed` guard (`crates/kali_codegen/src/emit/call.rs`)
//! decides when a text-matched `Array`/`Uint8Array` callee may route to the
//! shared `__alloc` helper. The rule these four tests pin, in both
//! directions:
//!
//! - a BARE callee routes only while the ctor name is unbound -- a user
//!   `function Uint8Array` / `const Uint8Array` must be called, not allocated;
//! - a QUALIFIED callee routes only in exactly one shape, a bare identifier
//!   named `globalThis` whose own name is unbound. A bare-name `Uint8Array`
//!   shadow must NOT block it (qualifying through the real global names the
//!   real property), but a user binding named `globalThis` must, and so must
//!   a nested/member-expression object (`a.globalThis.Uint8Array(n)`), whose
//!   node carries its PROPERTY name as its text while the deciding binding is
//!   the base identifier.
//!
//! Every one of those facts was a regression found by code review after this
//! task's first landing -- round 1: the bare-name case was unguarded at all;
//! round 2: the guard then over-blocked the qualified spelling; round 4: the
//! wholesale qualified exemption that fixed round 2 routed a user object
//! bound to the name `globalThis`; round 5: reading the object's text alone
//! still routed the nested spelling. Each was silent (exit 0, wrong number)
//! where the near-miss spellings refuse, so these tests pin the corrected
//! behaviour structurally. Deliberately structural (wasm-shape) rather than
//! an oracle `.toml` pin: Task 2 owns
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

#[test]
fn globalthis_qualified_uint8array_allocation_does_not_route_under_a_shadowed_object() {
    // Round 3's regression: `is_array_like_constructor` matches the
    // qualifying object by TEXT only, so a user object bound to the name
    // `globalThis` is indistinguishable from the real global at that
    // recognizer. Round 2's guard exempted every qualified callee wholesale
    // (arity alone), so nothing checked the object either, and this program
    // routed a user object's method through the allocator -- measured kali
    // `4104` at exit 0, no diagnostic, where node prints `6` and where every
    // near-miss spelling (two arguments, a renamed property, a renamed
    // object) refuses with `E5506`. The guard now checks the OBJECT name for
    // a qualified callee, so this must NOT reach `__alloc`; the program then
    // falls through to the same first-class-function-value refusal as its
    // controls.
    let src = "const globalThis = { Uint8Array: (n) => n + 1 }; \
               function f(x) { return x; } \
               f(globalThis.Uint8Array(5));";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.arena_table.set_arena_eligible("_start");
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    let alloc_index = exported_function_index(&text, "__alloc");
    let start_index = exported_function_index(&text, "_start");
    let start_body = function_body(&text, start_index);
    let call_needle = format!("call {alloc_index}");
    assert!(
        !start_body.lines().any(|line| line.trim() == call_needle),
        "expected `globalThis.Uint8Array(5)` under a user binding named \
         `globalThis` to NOT route through the __alloc helper (index \
         {alloc_index}): the qualifying object is the user's own object, not \
         the real global, so the allocation arm must decline:\n{start_body}"
    );
}

#[test]
fn nested_globalthis_qualified_uint8array_allocation_does_not_route() {
    // Round 5's regression: reading the qualifying object's own `text`
    // checks the wrong name when that object is itself a member expression.
    // `a.globalThis.Uint8Array(5)`'s object node carries the PROPERTY name
    // `globalThis` as its text, while the binding that decides the meaning
    // is the base identifier `a` -- so the round-4 guard saw an unbound
    // `globalThis` and routed a user object's method through the allocator
    // (measured kali `4104` at exit 0 where node prints `6`, while the
    // renamed-property, renamed-object and two-argument controls all refuse
    // with `E5506`). The guard now admits exactly one qualified shape, a
    // BARE identifier named `globalThis`, so this must NOT reach `__alloc`.
    let src = "const a = { globalThis: { Uint8Array: (n) => n + 1 } }; \
               function f(x) { return x; } \
               f(a.globalThis.Uint8Array(5));";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.arena_table.set_arena_eligible("_start");
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    let alloc_index = exported_function_index(&text, "__alloc");
    let start_index = exported_function_index(&text, "_start");
    let start_body = function_body(&text, start_index);
    let call_needle = format!("call {alloc_index}");
    assert!(
        !start_body.lines().any(|line| line.trim() == call_needle),
        "expected the nested `a.globalThis.Uint8Array(5)` spelling to NOT \
         route through the __alloc helper (index {alloc_index}): the \
         qualifying object is a member expression, not the real global, so \
         the allocation arm must decline:\n{start_body}"
    );
}
