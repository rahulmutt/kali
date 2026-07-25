//! Soundness pins for R-11: bitwise compound assignment (`&= |= ^= <<= >>= >>>=`).
//!
//! All six were silent no-ops on every assignment target (48/48 in the
//! 2026-07-24 register re-derivation): `let n=6; n<<=2` returned the unmodified
//! `6` at exit 0. The fix reuses the plain-operator int32 lowering
//! (`emit_bitwise`) at every assignment target arm, lowering integer targets and
//! failing closed (E5506) on float/string/unadmitted targets.
//!
//! Every expected value here was captured from node v26.5.0.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-bitwise-compound-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

fn assert_stdout(src: &str, expected: &str) {
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{out:?}");
}

fn assert_fails_closed(src: &str, needle: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success: {out:?}"
    );
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    assert!(
        stderr.contains(needle),
        "expected {needle:?}, got: {stderr}"
    );
}

// --- Task 1: plain binary bitwise operators stay correct (refactor is neutral) ---

#[test]
fn plain_binary_bitwise_operators_unchanged() {
    assert_stdout("console.log(6 & 3);\n", "2\n");
    assert_stdout("console.log(6 | 8);\n", "14\n");
    assert_stdout("console.log(6 ^ 1);\n", "7\n");
    assert_stdout("console.log(6 << 2);\n", "24\n");
    assert_stdout("console.log(6 >> 1);\n", "3\n");
    assert_stdout("console.log(6 >>> 1);\n", "3\n");
    assert_stdout("console.log(-1 >>> 0);\n", "4294967295\n");
    assert_stdout("console.log(1 << 31);\n", "-2147483648\n");
    assert_stdout("console.log(1 << 32);\n", "1\n");
}

// --- Task 1.5: the front end no longer silently mis-parses the six ops ---

#[test]
fn bitwise_compound_ops_are_not_silently_misparsed() {
    // Before Task 1.5 these lexed as two unrelated tokens and the statement
    // decayed into no-ops at exit 0 with ZERO diagnostics — the true R-11 root
    // cause. After Task 1.5 the op reaches codegen; Task 2 makes it compute the
    // right value. Here we pin only that the silent-garbage parse is gone: the
    // program must NOT exit 0 while printing the unmodified operand.
    for src in [
        "let n = 6; n &= 3; console.log(n);\n",
        "let n = 6; n |= 8; console.log(n);\n",
        "let n = 6; n ^= 1; console.log(n);\n",
        "let n = 6; n <<= 2; console.log(n);\n",
        "let n = 6; n >>= 1; console.log(n);\n",
        "let n = 6; n >>>= 1; console.log(n);\n",
    ] {
        let out = run_source(src);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !(out.status.success() && stdout.trim() == "6"),
            "silent no-op survived for {src:?}: {out:?}"
        );
    }
}

// --- Task 1.5 review follow-up: pin the resolve-stage gate's BOUNDARY ---
//
// `bitwise_compound_ops_are_not_silently_misparsed` above only asserts
// `!(exit 0 && stdout == "6")`, which passes for a great many wrong
// behaviors (a crash, an unrelated diagnostic, a wrong-but-nonzero value,
// ...). It does not pin that the six ops specifically produce the
// `kali_types::resolve::expression` fail-closed gate
// (`bitwise_compound_assign_op_text`, `crates/kali_types/src/resolve/expression.rs`),
// nor does it exercise any target shape other than a plain mutable scalar
// local. The two tests below close that gap: they pin the EXACT E5506
// diagnostic text the gate emits, across all six operators on the
// local-scalar case and across one representative operator on every other
// target shape codegen has not validated.
//
// These are BOUNDARY pins, not permanent truth: as Task 2 admits a shape
// (starting with the local-scalar arm, per its own task description), the
// corresponding row here is expected to flip from `assert_fails_closed` to
// an `assert_stdout` value assertion — that is progress, not a regression.
// What must NOT happen is a row silently going from "denied by this specific
// gate" to "denied by nothing" (i.e. admitted with no codegen support) or
// "denied by some unrelated diagnostic" without a deliberate, reviewed
// change to this file. If Task 2 deletes the whole `if let Some(op_text) =
// bitwise_compound_assign_op_text(...)` block in one shot instead of
// narrowing it shape-by-shape, every row below whose shape codegen still
// does not support will fail here (not silently pass), because the message
// text pinned is specific to this gate and no other diagnostic in this
// codebase reuses it.
//
// R-11 T2 update: the local-scalar row below has now flipped, exactly as
// anticipated — the resolve-stage gate (`bitwise_compound_target_is_admitted_local_scalar`)
// narrowly admits a bare-identifier mutable scalar `let`/`var`/parameter
// owned by the current function's own scope, and codegen's new local-branch
// arm (`literal.rs`) computes the real value. Renamed from
// `bitwise_compound_fails_closed_on_plain_scalar_all_six_ops` to reflect
// that it now pins ADMISSION, not denial, for this one shape.

#[test]
fn bitwise_compound_admitted_on_plain_scalar_all_six_ops() {
    for (src, expected) in [
        ("let n = 6; n &= 3; console.log(n);\n", "2\n"),
        ("let n = 6; n |= 8; console.log(n);\n", "14\n"),
        ("let n = 6; n ^= 1; console.log(n);\n", "7\n"),
        ("let n = 6; n <<= 2; console.log(n);\n", "24\n"),
        ("let n = 6; n >>= 1; console.log(n);\n", "3\n"),
        ("let n = 6; n >>>= 1; console.log(n);\n", "3\n"),
    ] {
        assert_stdout(src, expected);
    }
}

#[test]
fn bitwise_compound_fails_closed_on_every_target_shape() {
    // One representative op (`&=`) is enough per shape: the gate this pins
    // decides purely on `AssignmentOperator`, before it ever looks at the
    // LHS shape (`crates/kali_types/src/resolve/expression.rs:1794-1820` runs
    // ahead of every shape-specific admit path), so all six operators take
    // the same route through every shape below. `bitwise_compound_admitted_on_plain_scalar_all_six_ops`
    // above already covers the cross-operator axis on the one shape Task 2
    // admits; this test covers the cross-shape axis (every OTHER shape,
    // still denied).
    //
    // R-11 T3: the "module global written from a function" row that used to
    // live here (`let g = 6; function f(){ g &= 3; } f(); console.log(g);`)
    // is now an ADMITTED shape (T3's codegen change lowers it) and was moved
    // to `bitwise_compound_on_module_global_from_function` below as a value
    // assertion — leaving it here asserting `assert_fails_closed` would pin
    // the exact silent-no-op-turned-diagnostic regression this project
    // exists to fix, the wrong direction now that the shape is supported.
    //
    // R-11 T4: the "Closure-captured variable" row that used to live here
    // (`function outer(){ let x = 6; function g(){ x &= 3; } g();
    // console.log(x); } outer();`) is likewise now an ADMITTED shape —
    // T4's resolve-gate widening (`is_captured_ancestor`) plus its
    // codegen change (`try_emit_captured_assign`'s bitwise branch) lower it
    // — and was moved to `bitwise_compound_on_captured_scalar` below as a
    // value assertion, for the same reason the module-global row moved.
    //
    // R-11 T5: the "Member target" row that used to live here (`let o = {
    // a: 6 }; o.a &= 3; console.log(o.a);`) is likewise now an ADMITTED
    // shape — a static dot-field target on a proven fixed-shape integer
    // field (`bitwise_compound_dot_field_target_is_admitted` at resolve,
    // `emit_object_field_bitwise_compound_assign` at codegen) — and was
    // moved to `bitwise_compound_on_object_field` below as a value
    // assertion. Unlike the local/module/captured shapes, this admission is
    // NOT restricted to `const`: field mutation is legal on a `let`-bound
    // object too, and the proof chain (shape existence, `Repr::I64`,
    // `shape_field_is_proven_numeric`, the whole-program BigInt-taint set)
    // is entirely shape/field-keyed, not binding-keyed, so it holds for
    // `let`/`var` exactly as it does for `const`.
    let needle = "&=";
    assert_fails_closed(
        // Array element (`a[0]`).
        "let a = [6, 1, 2]; a[0] &= 3; console.log(a[0]);\n",
        needle,
    );
    assert_fails_closed(
        // For-in-key computed target (`o[k]`).
        "let o = { a: 6, b: 6 }; for (const k in o) { o[k] &= 3; } console.log(o.a);\n",
        needle,
    );
    assert_fails_closed(
        // Float target.
        "let f = 6.5; f &= 3; console.log(f);\n",
        needle,
    );
    assert_fails_closed(
        // String target.
        "let s = \"6\"; s &= 3; console.log(s);\n",
        needle,
    );
    assert_fails_closed(
        // `const` target.
        "const c = 6; c &= 3; console.log(c);\n",
        needle,
    );
}

// --- Task 2: local / parameter scalar targets ---

#[test]
fn bitwise_compound_on_let_scalar() {
    assert_stdout("let n = 6; n &= 3; console.log(n);\n", "2\n");
    assert_stdout("let n = 6; n |= 8; console.log(n);\n", "14\n");
    assert_stdout("let n = 6; n ^= 1; console.log(n);\n", "7\n");
    assert_stdout("let n = 6; n <<= 2; console.log(n);\n", "24\n");
    assert_stdout("let n = 6; n >>= 1; console.log(n);\n", "3\n");
    assert_stdout("let n = 6; n >>>= 1; console.log(n);\n", "3\n");
}

#[test]
fn bitwise_compound_on_var_scalar() {
    assert_stdout("var n = 6; n <<= 2; console.log(n);\n", "24\n");
}

#[test]
fn bitwise_compound_int32_edges() {
    // shift-count masking, sign, and uint32 round-trip through the slot.
    assert_stdout("let x = 1; x <<= 31; console.log(x);\n", "-2147483648\n");
    assert_stdout("let x = 1; x <<= 32; console.log(x);\n", "1\n");
    assert_stdout("let x = -8; x >>= 1; console.log(x);\n", "-4\n");
    assert_stdout("let x = -1; x >>>= 0; console.log(x);\n", "4294967295\n");
    assert_stdout("let x = 6; x <<= 2; x |= 1; console.log(x);\n", "25\n");
}

#[test]
fn bitwise_compound_in_function_scope_and_param() {
    assert_stdout(
        "function f(p) { p <<= 2; return p; } console.log(f(6));\n",
        "24\n",
    );
    assert_stdout(
        "function g() { let n = 5; n |= 2; return n; } console.log(g());\n",
        "7\n",
    );
}

#[test]
fn bitwise_compound_on_non_integer_fails_closed() {
    // float target, float RHS, string target — all E5506, never a wrong value
    // and never an internal E4201.
    assert_fails_closed("let x = 1.5; x <<= 1; console.log(x);\n", "<<=");
    assert_fails_closed("let n = 6; n <<= 1.5; console.log(n);\n", "<<=");
    assert_fails_closed("let s = \"a\"; s <<= 1; console.log(s);\n", "<<=");
    // Review Critical 1: a STRING RHS (as opposed to a string TARGET, the row
    // above) was the actual miscompile the original guard missed —
    // `is_float_valued(right)` answers "is the RHS a float", not "is the RHS
    // safe", so a string RHS fell through to `I32WrapI64`, which truncates
    // the tagged string HANDLE to its low 32 bits and silently computes a
    // wrong-but-plausible integer at exit 0. Four reproductions from the
    // review, node v26.5.0 values noted in each comment (all wrong under the
    // pre-fix guard: `1`, `12`, `0`, `2` respectively):
    assert_fails_closed("let n = 0; n |= \"5\"; console.log(n);\n", "|="); // node: 5
    assert_fails_closed(
        "let s = \"3\"; let n = 6; n <<= s; console.log(n);\n",
        "<<=",
    ); // node: 48
    assert_fails_closed("let k = 3; let n = 6; n &= `${k}`; console.log(n);\n", "&="); // node: 2
    assert_fails_closed(
        "let a = \"1\"; let b = \"2\"; let n = 6; n &= a + b; console.log(n);\n",
        "&=",
    ); // node: 4
       // Review round 2 doc-fix: `parse_number_literal` strips a trailing `n`
       // and parses the remainder, so a naive integer-literal check would admit
       // a BigInt literal RHS unchanged (measured: `n &= 3n` printed `2` at exit
       // 0; node throws `TypeError: Cannot mix BigInt`). BigInt lowering stays
       // deferred — the plain bitwise operators have the identical gap — but
       // this predicate must fail closed for it, not silently truncate.
    assert_fails_closed("let n = 6; n &= 3n; console.log(n);\n", "&=");
}

// --- Task 3: module-scope global written across functions (promotes to a WASM global) ---

#[test]
fn bitwise_compound_on_module_global() {
    // `flags` is mutated inside a function AND read at module scope → promoted
    // to a persistent WASM global, exercising emit_module_global_assignment.
    assert_stdout(
        "let flags = 6;\nfunction set() { flags |= 8; }\nset();\nconsole.log(flags);\n",
        "14\n",
    );
    assert_stdout(
        "let h = 6;\nfunction sh() { h <<= 2; }\nsh();\nconsole.log(h);\n",
        "24\n",
    );
    assert_stdout(
        "let u = -1;\nfunction z() { u >>>= 0; }\nz();\nconsole.log(u);\n",
        "4294967295\n",
    );
}

#[test]
fn bitwise_compound_on_module_global_from_function() {
    // Moved from `bitwise_compound_fails_closed_on_every_target_shape`'s
    // "Module global written from a function" row: that shape is now
    // ADMITTED (T3's codegen change lowers it), so it belongs here as a
    // value assertion, not a fail-closed pin. node: 6 & 3 = 2.
    assert_stdout(
        "let g = 6; function f(){ g &= 3; } f(); console.log(g);\n",
        "2\n",
    );
}

#[test]
fn bitwise_compound_on_module_global_read_via_closure() {
    // Renamed from `bitwise_compound_fails_closed_on_module_global_read_via_closure`
    // (Task 2 review Important 3): `x` here is a module-scope `let` that is
    // also READ by a closure (`g`) — `collect_module_scalar_globals`
    // (`lower.rs`) promotes it to the SAME persistent WASM global lane a
    // plain "written from a function" module global uses (reading it from a
    // closure needs no special seeding — `emit_module_global_assignment`
    // does not care who else reads the global), so it is admitted by the
    // exact same T3 codegen change, not a distinct shape. node: 6 & 3 = 2.
    assert_stdout(
        "let x = 6; const g = () => x; x &= 3; console.log(x);\n",
        "2\n",
    );
}

// --- Task 3 review round 2, Important 2: a function-local `let` that a
// NESTED closure captures is promoted OUT of `self.locals` (Stage C env-cell
// storage), so the `!self.locals.contains_key` check guarding the
// `module_global_slots` lookup does not exclude it. A same-named MODULE
// global then silently absorbed the write meant for the function's own
// shadowing local. The read must happen AFTER the call (not folded into the
// same expression as the call, which would read the module `n` before `f`
// runs and hide the bug behind evaluation order) to actually observe the
// corruption: measured with the guard removed, `f(); console.log(n);`
// printed `1` where node prints `6` (the module `n`, never touched by a
// correct program, was silently written by `f`'s own unrelated local `n`).
// Pinned as a diagnostic (fail-closed), not that — or any — value.

#[test]
fn bitwise_compound_fails_closed_on_module_global_shadowed_by_captured_local() {
    assert_fails_closed(
        "let n = 6; function f(){ let n = 9; const g = () => n; n &= 3; return n + g(); } f(); console.log(n);\n",
        "shadowed by a same-named module-scope binding",
    );
}

// --- Task 3 review Critical 1 / Important 3: the module-global TARGET axis
// needs its own provenance pins, mirroring the local-scalar ones below
// (`bitwise_compound_fails_closed_on_target_from_string_object_field`,
// `..._target_from_array_element`). Round 1 of this arm trusted `is_f64`
// alone for the target proof, reasoning that `collect_module_scalar_globals`
// / `scan_numeric_assignments`'s `is_numeric_expr` already proved the
// binding numeric — but that helper's bare-identifier branch is
// `repr_table.scalar(func, t)`, `ReprTable::scalar`'s `unwrap_or_default()`
// accessor whose default is `Repr::I64`. A TWO-HOP indirection through a
// bare-identifier copy (`let s = o.a; let n = s;` — `o.a` alone is directly
// rejected by `is_numeric_expr`, but the copy `n = s` is not, since `s`
// itself silently defaults to `Repr::I64`) routes a lost-`Repr::String`
// binding (the pre-existing R-06-R4 residual) straight through promotion.
// Measured on the round-1 build: `n &= 3` printed `1` at exit 0 where node
// prints `3` — the exact string-handle-truncation miscompile class this
// project exists to close, one lane over from the local arm's already-fixed
// leak. Fixed by threading `name` into `emit_module_global_assignment` and
// reusing `binding_is_proven_numeric` there too. These two pins exist so
// that fix cannot silently regress.

#[test]
fn bitwise_compound_fails_closed_on_module_global_target_from_string_object_field() {
    // node: 3. Module-global twin of
    // `bitwise_compound_fails_closed_on_target_from_string_object_field`:
    // `n` is written from inside a function (`f`), so it promotes to a WASM
    // global instead of staying a `_start` local — exercising
    // `emit_module_global_assignment`'s bitwise arm instead of the local
    // one. The `let s = o.a;` hop is required: a direct `let n = o.a;`
    // is rejected by `is_numeric_expr` at promotion time and never reaches
    // this arm at all (a less interesting, differently-denied shape).
    assert_fails_closed(
        "let o = {a: \"3\"}; let s = o.a; let n = s; function f(){ n &= 3; } f(); console.log(n);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_module_global_target_from_array_element() {
    // node: 3. Module-global twin of
    // `bitwise_compound_fails_closed_on_target_from_array_element`, same
    // two-hop shape (a direct `let n = a[1];` is rejected by
    // `is_numeric_expr` at promotion time and never reaches this arm).
    assert_fails_closed(
        "let a = [1, 2, 3]; let idx = a[1]; let n = idx; function f(){ n |= 1; } f(); console.log(n);\n",
        "|=",
    );
}

// --- Task 3 review Important 1: a BigInt-literal-initialized module global
// is a NEW wrong-value-at-exit-0 regression, distinct from Critical 1 above
// and NOT closed by the same `binding_is_proven_numeric` fix — `n`'s write
// (`6n`) genuinely satisfies that proof's own definition (a numeric/BigInt
// literal is exactly what `write_value_is_numeric` admits). `is_numeric_expr`
// strips the trailing `n` before parsing, so the declarator promotes with
// `is_f64 == false` regardless. node throws `TypeError: Cannot mix BigInt
// and other types`; before this task the six bitwise ops were denied on
// EVERY module global uniformly (T2's resolve gate), so this specific
// combination read as `E5506`; measured on the T3-Critical-1-round build:
// it printed `2` at exit 0. Denied via a separate, narrow, ADDITIVE
// provenance set (`module_global_bigint_targets`) that does not touch
// promotion or the ten pre-existing operators — general BigInt semantics on
// a module global stay exactly as deferred as before this task.

#[test]
fn bitwise_compound_fails_closed_on_bigint_initialized_module_global() {
    assert_fails_closed(
        "let n = 6n; function f(){ n &= 3; } f(); console.log(n);\n",
        "non-integer module global",
    );
}

// --- Task 3 review round 2, Important 1: round 1's BigInt guard
// (`expr_is_bigint_literal`) recognized only a bare BigInt literal or unary
// `-` over one — a DENYLIST of shapes, the exact pattern this project has
// now been bitten by repeatedly. `write_value_is_numeric` (the proof
// `binding_is_proven_numeric` is built from) admits a whole closure of
// write shapes as "numeric" — unary `- + ~`, binary
// `+ - * % & | ^ << >> >>>`, and a parameter — and a BigInt literal
// anywhere in that closure survives to a bitwise op untouched by round 1's
// narrow check. Fixed with an ALLOWLIST instead
// (`expr_is_provably_not_bigint`): taint unless the write is STRUCTURALLY
// proven not BigInt. All six of the reviewer's reproduction shapes are
// pinned below asserting the specific "non-integer module global"
// diagnostic (not merely the op text), so a shape that reaches some OTHER,
// unrelated denial cannot pass this pin by accident.

#[test]
fn bitwise_compound_fails_closed_on_bigint_arithmetic_declarator() {
    // node: throws. `6n + 1n` is a binary `+` over two BigInt literals —
    // outside round 1's literal-or-unary-minus check.
    assert_fails_closed(
        "let n = 6n + 1n; function f(){ n &= 3; } f(); console.log(n);\n",
        "non-integer module global",
    );
    // `6n * 2n` — same shape, `*`.
    assert_fails_closed(
        "let n = 6n * 2n; function f(){ n |= 1; } f(); console.log(n);\n",
        "non-integer module global",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_bigint_bitwise_not_declarator() {
    // node: throws. Unary `~` over a BigInt literal — round 1 only handled
    // unary `-`.
    assert_fails_closed(
        "let n = ~6n; function f(){ n &= 3; } f(); console.log(n);\n",
        "non-integer module global",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_bigint_reassignment_from_another_function() {
    // node: throws. The BigInt-tainting write is a REASSIGNMENT
    // (`n = 6n + 1n;`) inside a DIFFERENT function than the one performing
    // the bitwise op — round 1's scan only inspected declarator inits and
    // reassignment RHS for the LITERAL/unary-minus shape, not this
    // arithmetic-over-literals shape reached from a sibling function.
    assert_fails_closed(
        "let n = 0; function g(){ n = 6n + 1n; } g(); function f(){ n &= 3; } f(); console.log(n);\n",
        "non-integer module global",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_bigint_via_parameter_argument_inflow() {
    // node: throws (both). A BigInt literal reaches the module global
    // through a PARAMETER — interprocedural call-site inflow, the axis
    // round 1 did not model at all (`write_value_is_numeric`'s own
    // definition of "numeric" already covers a parameter; the taint must
    // follow every call site's argument at that position).
    assert_fails_closed(
        "let n = 0; function f(p){ n = p; n &= 3; } f(6n); console.log(n);\n",
        "non-integer module global",
    );
    // Two-hop: the parameter belongs to a DIFFERENT function than the one
    // doing the bitwise op.
    assert_fails_closed(
        "let n = 0; function g(p){ n = p; } g(9n); function f(){ n &= 3; } f(); console.log(n);\n",
        "non-integer module global",
    );
}

// --- Task 2 review round 2: Critical 1 was still open on the RHS-IDENTIFIER
// axis — `bitwise_compound_rhs_is_provably_i64`'s identifier branch admitted
// via `scalar_repr(name) == Repr::I64`, which is `ReprTable::scalar`'s
// `#[default]`, indistinguishable from "repr_infer recorded nothing at all."
// Fixed by requiring an EXPLICIT `ReprTable::scalar_entry` record instead.
// Five of the six round-2 reproductions are RHS-axis and were closed in that
// round; the sixth is TARGET-axis and was closed in round 3 by a different
// mechanism (`binding_is_proven_numeric`) — see
// `bitwise_compound_fails_closed_on_target_from_string_object_field` below.
// (An earlier revision of this comment named a
// `..._is_a_tracked_residual` test that recorded the then-open leak with no
// assertions; round 3 replaced it with that real pin.)

#[test]
fn bitwise_compound_fails_closed_on_rhs_from_string_object_field() {
    // node: 2. `s` reads a STRING object field (`o.a`); `repr_infer` does not
    // propagate `Repr::String` onto `s` through this provenance chain (the
    // pre-existing R-06-R4 residual), so `s` has NO explicit repr entry at
    // all — `scalar_entry` correctly reports "unproven" rather than
    // defaulting to "looks like I64".
    assert_fails_closed(
        "let o = {a: \"3\"}; let s = o.a; let n = 6; n &= s; console.log(n);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_rhs_from_string_object_field_or_assign() {
    // node: 5.
    assert_fails_closed(
        "let o = {a: \"5\"}; let s = o.a; let n = 0; n |= s; console.log(n);\n",
        "|=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_rhs_from_computed_string_object_field() {
    // node: 2. Computed-key variant of the field-read leak above.
    assert_fails_closed(
        "let o = {a: \"3\"}; let k = \"a\"; let s = o[k]; let n = 6; n &= s; console.log(n);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_unary_minus_over_string_field_rhs() {
    // node: 4. Unary `-` over the same tainted identifier — the recursive
    // arm must not unwrap past the identifier check.
    assert_fails_closed(
        "let o = {a: \"3\"}; let s = o.a; let n = 6; n &= -s; console.log(n);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_growable_array_rhs() {
    // node: 1. `a` is a growable-array HANDLE, not an integer — its own
    // `scalar_entry` is also unset (never explicitly I64), so the same fix
    // closes this leak too, incidentally.
    assert_fails_closed(
        "let a = []; a.push(1); let n = -1; n &= a; console.log(n);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_target_from_string_object_field() {
    // Review round 2 found this OPEN; round 3 closed it. `n` itself (the
    // bitwise compound-assign TARGET, not the RHS) is assigned from a string
    // object field (`o.a`) and therefore had no positive repr evidence in
    // either direction under the round-2 fix — `repr_infer` never records an
    // explicit `Repr::I64` `ReprTable::scalar` entry for ANY binding anywhere
    // in this codebase (confirmed by inspection: no `set_scalar` call site
    // ever passes `Repr::I64`, since it is `Repr`'s `#[default]`), so an
    // explicit-entry requirement on the TARGET (mirroring the RHS fix
    // verbatim) was measured to deny 100% of the admitted lane instead of
    // just the leak. Round 3 closed it with a DIFFERENT positive-evidence
    // signal instead: `ReprTable::numeric_bindings` / `binding_is_proven_numeric`,
    // a pre-existing allowlist `repr_infer` writes affirmatively (not
    // defaulted), now covering the six bitwise ops too
    // (`crates/kali_types/src/repr_infer.rs`'s `visit_assignment`). node: 3.
    assert_fails_closed(
        "let o = {a: \"3\"}; let n = o.a; n |= 1; console.log(n);\n",
        "|=",
    );
}

#[test]
fn bitwise_compound_over_denies_write_values_outside_the_numeric_proof() {
    // DELIBERATE-COST PINS, not correctness pins. Read this before "fixing"
    // any row below.
    //
    // Review round 3 added `binding_is_proven_numeric` to codegen's target
    // guard (`crates/kali_codegen/src/emit/literal.rs`), which closed a
    // family of silent miscompiles. It is not free. The proof that guard
    // consults is built by `write_value_is_numeric`
    // (`crates/kali_types/src/repr_infer.rs:1010-1041`), whose allowlist
    // admits ONLY: a numeric/BigInt literal, a self-reference, a PARAMETER of
    // the current function, and unary `- + ~` / binary
    // `+ - * % & | ^ << >> >>>` recursively over those. Every other write
    // value leaves the target unproven, so the guard denies it — including
    // when node computes the program correctly.
    //
    // Round 3 reported this cost as ONE shape ("a numeric object field read
    // into a local"). An A/B measurement against the round-2 parent binary
    // (`820e3dd91`, where every row below printed node's value at exit 0)
    // found SIX. All six are pinned here so a future change cannot move them
    // silently in either direction. Each is fail-CLOSED (E5506, nonzero
    // exit), never a wrong value.
    //
    // These rows are EXPECTED to flip back to `assert_stdout` value
    // assertions one day. The follow-up that does it is widening
    // `write_value_is_numeric` to model member/call/local-identifier inflow —
    // NOT loosening the codegen guard or the resolve-stage admit predicate,
    // which is what the guard is there to prevent. A row flipping to a VALUE
    // is progress; a row starting to print a wrong value, or being denied by
    // some unrelated diagnostic, is a regression.

    // 1. Arithmetic whose leaves are non-parameter LOCALS. node: 9.
    assert_fails_closed(
        "let a = 3; let b = 3; let n = a * b; n |= 0; console.log(n);\n",
        "|=",
    );
    // 2. Initialized from a CALL return. node: 24.
    assert_fails_closed(
        "function f() { return 6; }\nlet n = f(); n <<= 2; console.log(n);\n",
        "<<=",
    );
    // 3. Initialized from a numeric object FIELD (the one case round 3
    //    named). node: 3.
    assert_fails_closed(
        "let o = {a: 3}; let n = o.a; n |= 1; console.log(n);\n",
        "|=",
    );
    // 4. Initialized from a `const` binding — an identifier, not a
    //    parameter. node: 24.
    assert_fails_closed("const c = 6; let n = c; n <<= 2; console.log(n);\n", "<<=");
    // 5. Initialized from another LOCAL. node: 24.
    assert_fails_closed("let m = 6; let n = m; n <<= 2; console.log(n);\n", "<<=");
    // 6. Reassigned from a CALL after a provable initializer — one
    //    unprovable write is enough to unprove the binding. node: 28.
    assert_fails_closed(
        "function f() { return 7; }\nlet n = 0; n = f(); n <<= 2; console.log(n);\n",
        "<<=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_target_from_array_element() {
    // NOT a deliberate-cost pin — this one is a CLOSED MISCOMPILE, and it
    // was closed incidentally (and unreported) by round 3's
    // `binding_is_proven_numeric` tightening. Measured on the round-2 parent
    // binary (`820e3dd91`): this program printed `1` at exit 0. node prints
    // `3`. At HEAD it fails closed with E5506.
    //
    // It shares the round-4 over-denial's mechanism — an index read is
    // outside `write_value_is_numeric`'s allowlist
    // (`crates/kali_types/src/repr_infer.rs:1010-1041`), so `n` gets no
    // positive numeric evidence — but it is pinned separately because the
    // fact it protects is different: the six rows in
    // `bitwise_compound_over_denies_write_values_outside_the_numeric_proof`
    // may legitimately flip to value assertions when that proof is widened,
    // whereas this row must NEVER go back to printing `1`. If a future
    // widening admits array-element inflow, this row flips to
    // `assert_stdout(.., "3\n")` — node's value — and to nothing else.
    assert_fails_closed(
        "let a = [1, 2, 3]; let n = a[1]; n |= 1; console.log(n);\n",
        "|=",
    );
}

// --- Task 2 review Important 3: the admit predicate's actual boundary on a
// variable that is captured by SOME closure but referenced from the function
// that OWNS it (as opposed to referenced from the CAPTURING closure, now
// covered by `bitwise_compound_on_captured_scalar` below).
// `bitwise_compound_target_is_admitted_local_scalar` admits both of these at
// resolve — `x`/`g` below are structurally owned by the function doing the
// write. Before R-11 T4, the FUNCTION-scope row (`outer`'s own `x`, captured
// by a nested `g`) failed closed at codegen (Stage C env-cell promotion had
// no bitwise lowering yet). T4's codegen change
// (`try_emit_captured_assign`'s bitwise branch, own-cell path — `x` is
// `outer`'s OWN cell, written from `outer`'s own body, so this shape needed
// no resolve-gate widening, only the codegen guard) now lowers it. The
// MODULE-scope row (module-level `x`, read by a closure `g`) was already
// admitted by R-11 T3's codegen change via the same module-global lane a
// plain "written from a function" module global uses — see
// `bitwise_compound_on_module_global_read_via_closure` above.
//
// node: `x = 6 & 3 = 2`; `return x + g()` reads the already-updated `x`
// (`2`) then calls `g()`, which reads the SAME cell (`2`) — `2 + 2 = 4`.

#[test]
fn bitwise_compound_on_owning_function_captured_variable() {
    assert_stdout(
        "function outer(){ let x = 6; const g = () => x; x &= 3; return x + g(); } console.log(outer());\n",
        "4\n",
    );
}

// --- Task 4: captured scalar (env-cell) target ---

#[test]
fn bitwise_compound_on_captured_scalar() {
    // `flags` is captured and compound-assigned by a sibling closure — the
    // Stage C env-cell write path (`try_emit_captured_assign`).
    assert_stdout(
        "function outer() {\n  let flags = 6;\n  function set() { flags |= 8; }\n  set();\n  console.log(flags);\n}\nouter();\n",
        "14\n",
    );
    // Moved from `bitwise_compound_fails_closed_on_every_target_shape`'s
    // "Closure-captured variable" row (R-11 T4: now an ADMITTED shape — see
    // that test's updated comment). node: `6 & 3 = 2`.
    assert_stdout(
        "function outer(){ let x = 6; function g(){ x &= 3; } g(); console.log(x); } outer();\n",
        "2\n",
    );
    // Both directions of the parent chain: TWO sibling closures share the
    // same owner's cell, one writing and one reading, confirming the write
    // is visible through the SAME cell regardless of which closure reads it
    // (not a copy). node: `6 | 8 = 14`.
    assert_stdout(
        "function outer(){ let flags = 6; function set(){ flags |= 8; } function get(){ return flags; } set(); console.log(get()); } outer();\n",
        "14\n",
    );
}

// --- Task 4 review: target/RHS provenance and the C1 promotion boundary,
// mirroring the local (Task 2) and module-global (Task 3) provenance pins
// over the THIRD storage location the bitwise ops now reach — a captured env
// cell. Every row asserts `assert_fails_closed` (never a wrong value).

#[test]
fn bitwise_compound_fails_closed_on_captured_target_non_integer_and_rhs() {
    // String RHS on a captured int target — the exact string-handle-
    // truncation miscompile class the RHS oracle exists to close (node: `7`,
    // `6 | "5"` coerces `"5"` to `5`).
    assert_fails_closed(
        "function outer(){ let n = 6; function set(){ n |= \"5\"; } set(); console.log(n); } outer();\n",
        "|=",
    );
    // Float RHS on a captured int target (node: `6 << 1.5` truncates `1.5`
    // to `1` — `12`).
    assert_fails_closed(
        "function outer(){ let n = 6; function set(){ n <<= 1.5; } set(); console.log(n); } outer();\n",
        "<<=",
    );
    // A captured target whose only initializer is a string-object-field copy
    // through a bare-identifier hop (the R-06-R4-adjacent target-axis
    // over-denial the local/module arms document — `s`/`n` are not
    // parameters, so `write_value_is_numeric` proves nothing about them;
    // fails closed rather than truncating the string handle). node: `1`
    // (`"3" & 1` coerces to `3 & 1`).
    assert_fails_closed(
        "function outer(){ let o = {a: \"3\"}; let s = o.a; let n = s; function set(){ n &= 1; } set(); console.log(n); } outer();\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_bigint_initialized_captured_target() {
    // node: throws (`TypeError: Cannot mix BigInt and other types`). `n`'s
    // declarator init is a raw BigInt literal — `write_value_is_numeric`
    // admits a BigInt literal exactly like a plain number, so
    // `binding_is_proven_numeric` alone cannot refuse this; closed by the
    // separate, additive `captured_cell_bigint_targets` scan
    // (`collect_bigint_tainted_captured_cells`, mirroring Task 3's
    // `module_global_bigint_targets`).
    assert_fails_closed(
        "function outer(){ let n = 6n; function set(){ n &= 3; } set(); console.log(n); } outer();\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_captured_target_from_object_or_array_element() {
    // A captured MEMBER target (`o.a`) is not a bare identifier —
    // `resolve_update_binding_name` returns `None` regardless of capture —
    // still Task 5's territory, untouched by this task. node: `2`.
    assert_fails_closed(
        "function outer(){ let o = { a: 6 }; function set(){ o.a &= 3; } set(); console.log(o.a); } outer();\n",
        "&=",
    );
    // A captured ARRAY ELEMENT target — same reasoning, plus the pre-existing
    // literal-array-mutation reject. node: `2`.
    assert_fails_closed(
        "function outer(){ let a = [6, 1, 2]; function set(){ a[0] &= 3; } set(); console.log(a[0]); } outer();\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_two_hop_captured_chain() {
    // `inner` captures `x` through TWO env-owning hops: `mid` owns its own
    // record (its nested `h` captures `mid`'s own `y`), so `mid` is a
    // genuine, non-transparent intermediate between `inner` and `outer` —
    // `mir_depth == 2` for `inner`'s reference to `x`. `env_walk_depth_for`
    // only proves `mir_depth == 1` (see its own doc on why a deeper chain is
    // not provable against the runtime env-record chain); this shape falls
    // through to the pre-existing local-miss fallback exactly as it did
    // before this task, unaffected by the resolve-gate widening (which is
    // purely structural and does not itself distinguish hop count) or the
    // codegen change (`resolve_scalar_capture_access` returns `None` for
    // it). node: `2` (`6 & 3`) — a real value this compiler does not yet
    // reach, not a program that should error.
    assert_fails_closed(
        "function outer(){\n  let x = 6;\n  function mid(){\n    let y = 1;\n    function h(){ return y; }\n    function inner(){ x &= 3; }\n    inner();\n    return h();\n  }\n  mid();\n  console.log(x);\n}\nouter();\n",
        "unless it is a mutable local binding",
    );
}

// --- Task 4 review Critical 1: a module-scope binding of ANY kind (`const`,
// `let`, `var`) shadowing a captured cell must refuse the bitwise WRITE, not
// just the pre-existing (and already-guarded) `module_global_slots` case.
//
// A module-scope `const` is never in `module_global_slots` — it is compile-
// time INLINED at each read site from `module_const_inits`, a wholly separate
// table `emit_identifier` (`control_flow.rs:2238`) consults BEFORE
// `try_emit_captured_read` (`:2298`). Before this fix, denying the bitwise
// WRITE only when `module_global_slots.contains_key(name)` let the write to
// the captured cell through (correctly targeting the owner's cell) while a
// LATER read of the same name from the owning function's own body silently
// inlined the UNRELATED module `const` instead — a wrong value at exit 0,
// not a diagnostic. Confirmed non-vacuous: every row below reproduces the
// wrong value shown in its comment against the pre-fix build (commit
// `135bc0904`, this task's own first round) and now fails closed E5506 on
// this build.

#[test]
fn bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell() {
    let needle = "shadowed by a same-named module-scope binding";
    // node: 4. Pre-fix (135bc0904): printed `6` (the untouched module const).
    assert_fails_closed(
        "const n = 6;\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\n",
        needle,
    );
    // All six ops through the same shadow shape. node: 28 / 9 / 48 / 6 / 6.
    // Pre-fix: printed `6` (the const) for every one of them.
    for op in ["|= 16", "^= 5", "<<= 2", ">>= 1", ">>>= 1"] {
        let src = format!(
            "const n = 6;\nfunction f() {{ let n = 12; function s() {{ n {op}; }} s(); console.log(n); }}\nf();\n"
        );
        assert_fails_closed(&src, needle);
    }
    // A different const value/op pair. node: 1. Pre-fix: printed `100`.
    assert_fails_closed(
        "const n = 100;\nfunction f() { let n = 9; function s() { n &= 3; } s(); console.log(n); }\nf();\n",
        needle,
    );
    // The module const is a STRING — the captured write still must not let a
    // rendered string handle leak through. node: 4. Pre-fix: printed `abc`.
    assert_fails_closed(
        "const n = \"abc\";\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\n",
        needle,
    );
    // The module const is a FLOAT. node: 4. Pre-fix: printed `1.5`.
    assert_fails_closed(
        "const n = 1.5;\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\n",
        needle,
    );
    // Own-cell write (T2/T4's `is_own_scope` admitted shape), read TWICE —
    // directly and through a second closure `g`. Confirms the shadow is not
    // merely a stale constant fold: a second, independent closure's read
    // also loses to the const. node: 1 / 1. Pre-fix: printed `100` / `100`.
    assert_fails_closed(
        "const n = 100;\nfunction f() { let n = 9; const g = () => n; n &= 3; console.log(n); console.log(g()); }\nf();\n",
        needle,
    );
    // Arrow-function capturer instead of a `function` declaration. node: 4.
    // Pre-fix: printed `6`.
    assert_fails_closed(
        "const n = 6;\nfunction f() { let n = 12; const s = () => { n &= 7; }; s(); console.log(n); }\nf();\n",
        needle,
    );
    // Read in a fold-sensitive `===` comparison position, matching the
    // `invalidate_static_binding` fold-sensitivity check used elsewhere in
    // this file's provenance suite. node: true. Pre-fix: printed `0`
    // (`false`) — reading the module const `6`, not `4`.
    assert_fails_closed(
        "const n = 6;\nfunction f() { let n = 12; function s() { n &= 7; } s(); return n === 4; }\nconsole.log(f());\n",
        needle,
    );
}

// Round 3 review Minor 2: this is a MESSAGE pin, by design, not a
// wrong-value-prevention pin — all three rows below already failed closed
// (exit 1, E5506) on the PARENT too, before this task's shadow guard existed
// at all: the numeric `let`/`var` rows were always caught by the
// pre-existing `module_global_slots`-only guard (a numeric module scalar
// whose name recurs inside a function is ALWAYS promoted by
// `collect_module_scalar_globals`'s name-based, scope-blind reference scan —
// a necessary condition for a shadow to exist at all, so this sub-case never
// depended on the widening), and the STRING `let` row was independently
// caught by the pre-existing READ-side `module_binding_names` gate
// (`control_flow.rs:2243`, "reading module binding ... is only available for
// compile-time-constant `const`") even with the WRITE-side guard's original,
// narrower `module_global_slots`-only condition — measured directly: with
// only that original condition, the STRING row still exits 1, just via TWO
// diagnostics (the read-gate's own, plus nothing from the write guard) rather
// than one. So no row here demonstrates a value the fix prevented; each row
// only confirms the CHOKE POINT that catches the shadow (this task's own
// write-side guard, via its `identifier_read_resolves_only_through_captured_cell`
// needle) rather than some OTHER, coincidentally-safe pre-existing gate.
#[test]
fn bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell() {
    let needle = "shadowed by a same-named module-scope binding";
    // Module `let`, numeric — promotes to `module_global_slots` (the
    // pre-existing T3 guard's own case), still covered after the widening.
    // node: 4 / 6.
    assert_fails_closed(
        "let n = 6;\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\nconsole.log(n);\n",
        needle,
    );
    // Module `var`, numeric — same shape, `var` instead of `let`.
    assert_fails_closed(
        "var n = 6;\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\nconsole.log(n);\n",
        needle,
    );
    // Module `let` holding a STRING — never promoted to `module_global_slots`
    // (only `I64`/`F64` scalars promote), so only the write-side guard's
    // `module_binding_names`-derived coverage (now folded into
    // `identifier_read_resolves_only_through_captured_cell`) intercepts it
    // at the WRITE. node: 4 / "hi".
    assert_fails_closed(
        "let n = \"hi\";\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\nconsole.log(n);\n",
        needle,
    );
}

#[test]
fn bitwise_compound_on_module_function_name_does_not_shadow_captured_cell() {
    // A module-scope named `function` declaration is NOT in
    // `module_global_slots`, `module_const_inits`, or `module_binding_names`
    // (those tables hold only `const`/`let`/`var` names) — it does not
    // intercept the captured read at all, so this shape is NOT part of the
    // shadow class the two tests above cover and must keep computing the
    // real value. node: 4.
    assert_stdout(
        "function n() { return 99; }\nfunction f() { let n = 12; function s() { n &= 7; } s(); console.log(n); }\nf();\n",
        "4\n",
    );
}

// --- Task 4 review Important 1: a FLOAT write reaching a promoted captured
// cell from a THIRD function (neither the cell's owner nor the function
// performing the bitwise op) must refuse the bitwise op, not reach codegen's
// raw `I32WrapI64` combiner. `repr_infer`'s scalar-repr union-find resolves
// an off-scope write's node key via `binding_scope`, which cannot name the
// true owner for a write reached from such a third function — the write is
// filed under a disconnected union-find node
// `crate::closure::cell_is_promotable`'s owner-scoped query never sees, so
// the cell promotes as if it were safely `I64` and the target check passes.
// Confirmed non-vacuous: every row below reproduced `E4201: failed to load
// WASM module` against the pre-fix build (commit `135bc0904`) — an internal
// error the plan's Global Constraints forbid — and now fails closed E5506 on
// this build.

#[test]
fn bitwise_compound_fails_closed_on_float_write_reaching_captured_target_from_sibling_function() {
    let needle = "on a captured binding";
    // node: 2. Pre-fix: E4201.
    assert_fails_closed(
        "function o(){ let n=6; function w(){ n=6.5; } function s(){ n&=3; } w(); s(); console.log(n); } o();\n",
        needle,
    );
    // Deep-nested: the float write is TWO function boundaries away from the
    // owner (`mid` -> `w`), not just one. node: 2. Pre-fix: E4201.
    assert_fails_closed(
        "function o(){ let n=6; function mid(){ function w(){ n=6.5; } w(); } function s(){ n&=3; } mid(); s(); console.log(n); } o();\n",
        needle,
    );
    // The float write is an ARITHMETIC expression, not a bare literal. node:
    // 2. Pre-fix: E4201.
    assert_fails_closed(
        "function o(){ let n=6; function w(){ n=6.5+0; } function s(){ n&=3; } w(); s(); console.log(n); } o();\n",
        needle,
    );
    // A negative float literal. node: 0. Pre-fix: E4201.
    assert_fails_closed(
        "function o(){ let n=6; function w(){ n=-0.5; } function s(){ n&=3; } w(); s(); console.log(n); } o();\n",
        needle,
    );
}

// --- Task 4 review round 3, Minor 1: `collect_float_tainted_captured_cells`
// is keyed by NAME ONLY (globally, across the whole program — see that
// function's own doc for why, mirroring `captured_cell_bigint_targets`'s
// identical policy). A float LOCAL in a completely unrelated function that
// happens to share a name with a captured int cell elsewhere disables the
// bitwise lane for that unrelated cell too. This is fail-closed (never a
// wrong value) and was already documented, but not previously pinned —
// pinned here as a deliberate, accepted over-denial cost so it cannot move
// silently. node: 2 (a real, correctly-computable value this compiler does
// not reach because of the name collision, not because the shape itself is
// unsupported).

#[test]
fn bitwise_compound_fails_closed_on_unrelated_float_local_sharing_a_captured_cell_name() {
    assert_fails_closed(
        "function q(){ let n=6.5; return n; } function o(){ let n=6; function s(){ n&=3; } s(); console.log(n); } o();\n",
        "on a captured binding",
    );
}

// --- Task 4 review rounds 3-4: the shadow guard
// (`FunctionEmitter::identifier_read_resolves_only_through_captured_cell`,
// `closure_access.rs`) now defers to the SHARED classifier
// (`control_flow.rs::resolve_identifier_kind`, returning
// `IdentifierResolution`) both it and `emit_identifier` match — see that
// classifier's own doc for why round 3's hand-mirrored `!(A || B || …)`
// version of this same idea was proven, not just suspected, to drift the
// moment a new arm was added to `emit_identifier` (the round-3 review's
// `"Reflect"` experiment).
//
// IMPORTANT — what THIS TEST does and does not prove: it enumerates arms
// that exist TODAY and checks each one denies TODAY. It does NOT, by itself,
// guarantee a FUTURE arm added to `emit_identifier` is caught — that
// guarantee now comes from the exhaustive `match` on `IdentifierResolution`
// at BOTH `emit_identifier`'s dispatch and (via equality against the one
// admitted variant) this guard, which the Rust compiler enforces
// independently of any test ever running. This test is a regression pin for
// the arms enumerated below, not the mechanism that makes new arms safe.
//
// Two arms in `emit_value`'s dispatch (`is_process_kill`,
// `is_supported_callable_reference`) are NOT included below: both require a
// member-expression or single-child-call node shape and structurally cannot
// match a bare 0-children identifier at all, so there is no repro to pin.
//
// Every row was measured against BOTH the round-2 build (commit
// `5e7dbb622`, before round 3's choke-point inversion) and the current
// build. ONLY the four `Set`/`Map`/`Infinity`/`NaN` rows are non-vacuous —
// they printed a WRONG VALUE at exit 0 on round 2 and now fail closed. Every
// OTHER row below (EventTarget, AbortController, URL, URLSearchParams,
// TextEncoder, Event, `undefined`) is caught by a completely UNRELATED,
// pre-existing diagnostic (byte-identical stderr confirmed against the
// parent) and would still pass with this task's guard deleted entirely —
// they are included as a REGRESSION PIN against those upstream gates ever
// loosening, not as evidence this guard is doing anything for them.

#[test]
fn bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm() {
    // `Set` / `Map` — round-2 WRONG VALUE: printed `0` at exit 0 (both).
    // node: 4 (both).
    assert_fails_closed(
        "function f(){ let Set=12; function s(){ Set &= 7; } s(); console.log(Set); } f();\n",
        "shadowed by a same-named module-scope binding",
    );
    assert_fails_closed(
        "function f(){ let Map=12; function s(){ Map &= 7; } s(); console.log(Map); } f();\n",
        "shadowed by a same-named module-scope binding",
    );
    // `Infinity` / `NaN` — round-2 WRONG VALUE: printed `Infinity` / `NaN`
    // (the JS global text) at exit 0. node: 4 (both). Defensive-only: the
    // real interception mechanism is `resolve_static_object_identity_value`
    // (`intrinsics/object.rs`), a SEPARATE, pre-existing, out-of-scope bug
    // that also breaks a PLAIN (non-captured) local named `Infinity`/`NaN`
    // with zero relation to R-11 — see the fix report for the measured
    // `let Infinity = 12; console.log(Infinity);` → `E4201` on the pre-R-11
    // parent. This pin only proves the CAPTURED-bitwise combination no
    // longer computes a silent wrong value; it does not claim the general
    // bug is fixed.
    assert_fails_closed(
        "function f(){ let Infinity=12; function s(){ Infinity &= 7; } s(); console.log(Infinity); } f();\n",
        "shadowed by a same-named module-scope binding",
    );
    assert_fails_closed(
        "function f(){ let NaN=12; function s(){ NaN &= 7; } s(); console.log(NaN); } f();\n",
        "shadowed by a same-named module-scope binding",
    );
    // `EventTarget` — already fail-closed on round 2 (a module-scope object
    // handle is never an inlinable `const`, so the read-side module-binding
    // gate already denied). node: 4.
    assert_fails_closed(
        "const t = new EventTarget(); function f(){ let t=12; function s(){ t &= 7; } s(); console.log(t); } f();\n",
        "&=",
    );
    // `AbortController` — already fail-closed on round 2 at RESOLVE (an
    // AbortHandle-repr target is not a scalar `I64`/`F64`/`String`, so
    // `compound_update_target_is_scalar` never admits it). node: 4.
    assert_fails_closed(
        "const c = new AbortController(); function f(){ let c=12; function s(){ c &= 7; } s(); console.log(c); } f();\n",
        "&=",
    );
    // `URL` / `URLSearchParams` — same resolve-side reasoning as
    // AbortController. node: 4 (both).
    assert_fails_closed(
        "const u = new URL(\"https://a.b/\"); function f(){ let u=12; function s(){ u &= 7; } s(); console.log(u); } f();\n",
        "&=",
    );
    assert_fails_closed(
        "const p = new URLSearchParams(\"a=1\"); function f(){ let p=12; function s(){ p &= 7; } s(); console.log(p); } f();\n",
        "&=",
    );
    // `TextEncoder` — same reasoning as EventTarget (module-binding read
    // gate). node: 4.
    assert_fails_closed(
        "const e = new TextEncoder(); function f(){ let e=12; function s(){ e &= 7; } s(); console.log(e); } f();\n",
        "&=",
    );
    // `Event` — same resolve-side reasoning as AbortController. node: 4.
    assert_fails_closed(
        "const ev = new Event(\"tick\"); function f(){ let ev=12; function s(){ ev &= 7; } s(); console.log(ev); } f();\n",
        "&=",
    );
    // `undefined` — not a reserved word in JS but rejected by kali's own
    // parser/lexer as a binding name before this ever reaches codegen or
    // resolve; included to document that this arm's collision risk is
    // already closed at a completely different layer. node: 4.
    assert_fails_closed(
        "function f(){ let undefined=12; function s(){ undefined &= 7; } s(); console.log(undefined); } f();\n",
        "reserved word",
    );
}

// --- Task 5: integer object field ---

#[test]
fn bitwise_compound_on_object_field() {
    // A fixed-shape integer field compound-assigned. Reachability note (Step
    // 2 of the R-11 T5 brief): this does NOT reach
    // `emit_object_field_compound_assign_dynamic` — that function lowers the
    // COMPUTED for-in-key member form (`obj[c] op= v`), never a static dot
    // field. Before T5 there was no codegen lowering at all for a static
    // dot-field compound assign (arithmetic OR bitwise) — this is new
    // lowering (`emit_object_field_bitwise_compound_assign`, `object.rs`),
    // not a widened existing arm.
    assert_stdout("const o = { a: 6 }; o.a <<= 2; console.log(o.a);\n", "24\n");
    // All six ops.
    assert_stdout("const o = { a: 6 }; o.a &= 3; console.log(o.a);\n", "2\n");
    assert_stdout("const o = { a: 6 }; o.a |= 8; console.log(o.a);\n", "14\n");
    assert_stdout("const o = { a: 6 }; o.a ^= 1; console.log(o.a);\n", "7\n");
    assert_stdout("const o = { a: 6 }; o.a >>= 1; console.log(o.a);\n", "3\n");
    assert_stdout(
        "const o = { a: -1 }; o.a >>>= 0; console.log(o.a);\n",
        "4294967295\n",
    );
    // `let`/`var`-bound object: field mutation is legal on a non-const
    // binding too, and the admission proof (shape/field-keyed, not
    // binding-keyed) holds identically. Moved out of
    // `bitwise_compound_fails_closed_on_every_target_shape`'s "Member
    // target" row now that this shape is admitted.
    assert_stdout("let o = { a: 6 }; o.a &= 3; console.log(o.a);\n", "2\n");
    assert_stdout("var o = { a: 6 }; o.a <<= 2; console.log(o.a);\n", "24\n");
    // A field WRITE between construction and the compound-assign is visible
    // (real read-modify-write off the heap, not a fold of the original
    // literal): 101 & 3 = 1, distinct from both 0 (would indicate a bare
    // wrong-value bug) and 2 (6 & 3, would indicate a stale read of the
    // original literal instead of the current heap value).
    assert_stdout(
        "let o = { a: 6 }; o.a = 101; o.a &= 3; console.log(o.a);\n",
        "1\n",
    );
    // Multiple fields on the same shape; only the targeted field mutates.
    assert_stdout(
        "let o = { a: 6, b: 9 }; o.a <<= 2; console.log(o.a + \",\" + o.b);\n",
        "24,9\n",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_float() {
    // A float-repr'd field has no bitwise lowering — E5506, not a truncated
    // value.
    assert_fails_closed("const o = { a: 6.5 }; o.a &= 3; console.log(o.a);\n", "&=");
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_string() {
    // `repr_infer` interns a string field as `Repr::I64` too (see review C-5
    // in `emit/call.rs`) — `shape_field_is_proven_numeric` is what actually
    // excludes it. node computes `2` (ToNumber("6") coerces before the
    // bitwise op — it does NOT throw here); kali refuses instead of
    // truncating the tagged string HANDLE through `I32WrapI64`, a deliberate
    // over-denial matching the local-scalar precedent
    // (`bitwise_compound_on_non_integer_fails_closed`'s string-target row).
    assert_fails_closed(
        "const o = { a: \"6\" }; o.a &= 3; console.log(o.a);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_bigint_literal() {
    // Target-axis BigInt guard (`collect_bigint_tainted_shape_fields`): a
    // BigInt-literal field interns as plain `Repr::I64` and passes
    // `shape_field_is_proven_numeric` (which only excludes strings) — the
    // object-field analogue of the module-global/captured-cell BigInt taint
    // Task 3/4 needed. node throws `TypeError: Cannot mix BigInt`.
    assert_fails_closed("const o = { a: 6n }; o.a &= 3; console.log(o.a);\n", "&=");
    // The taint also closes a LATER write of a BigInt literal into an
    // originally-safe field, not just the declarator init.
    assert_fails_closed(
        "let o = { a: 6 }; o.a = 7n; o.a &= 3; console.log(o.a);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_unknown_field() {
    assert_fails_closed("const o = { a: 6 }; o.z &= 3; console.log(o.z);\n", "&=");
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_rhs_string() {
    // RHS-axis proof reused verbatim (`bitwise_compound_rhs_is_provably_i64`)
    // — a string RHS must not truncate through `I32WrapI64` the way the
    // local-scalar review Critical 1 measured. node: 5.
    assert_fails_closed(
        "const o = { a: 0 }; o.a |= \"5\"; console.log(o.a);\n",
        "|=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_array_element_target() {
    // Array element target (`a[0] &= 3`) must stay denied: it never reaches
    // `object_shape_of_node` as an object (it is an array binding), and the
    // resolve-side gate (`bitwise_compound_dot_field_target_is_admitted`)
    // requires `member.computed_index.is_none()`, which `a[0]` never
    // satisfies. Guards against T5's widening leaking into the array-element
    // shape Task 6 is scoped to audit.
    assert_fails_closed("let a = [6, 1, 2]; a[0] &= 3; console.log(a[0]);\n", "&=");
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_forin_key_computed_target() {
    // Computed for-in-key target (`o[k] &= 3`) must stay denied: it has
    // `computed_index.is_some()`, so `bitwise_compound_dot_field_target_is_admitted`
    // rejects it structurally — this task opens the STATIC dot-field shape
    // only, not the computed/for-in-key one (`emit_object_field_compound_assign_dynamic`
    // is untouched by T5).
    assert_fails_closed(
        "let o = { a: 6, b: 6 }; for (const k in o) { o[k] &= 3; } console.log(o.a);\n",
        "&=",
    );
}

// --- T5 review Critical 1: a shape carrying a field literally named
// `length` must be refused ENTIRELY, not just when `length` is the
// compound-assign's own target. `object_shape_of_node` treats ANY
// `<expr>.length` dot access as an ARRAY length read before it ever tries
// the object-field interpretation, so a LATER read of `o.length` silently
// returns `0` instead of the real field value — independent of which field
// this compound-assign targets. Measured pre-fix: `o.length &= 3` itself
// computed `0` (node: `2`); `o.a &= 3` on a shape ALSO carrying an unrelated
// `length` field computed `o.a` correctly but a later `o.length` read still
// printed `0` (node: `9`). Both closed by refusing the whole SHAPE.

#[test]
fn bitwise_compound_fails_closed_on_object_field_named_length() {
    // node: 2 (6 & 3). kali must refuse, not compute 0.
    assert_fails_closed(
        "let o = { length: 6 }; o.length &= 3; console.log(o.length);\n",
        "&=",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_shape_with_unrelated_length_field() {
    // The compound-assign TARGET here is `a`, not `length` — this pins that
    // the refusal is keyed on the SHAPE carrying a `length` field at all,
    // not just on `length` being the immediate target. node: "2,9".
    assert_fails_closed(
        "let o = { a: 6, length: 9 }; o.a &= 3; console.log(o.a + \",\" + o.length);\n",
        "&=",
    );
}

// --- T5 review Important 1: a pre-existing codegen bug ("float-into-i64-cell
// storage", standing deferred item) unmasked by this task's admission, not
// caused by it. ---

#[test]
fn bitwise_compound_object_field_unmasks_preexisting_float_into_i64_cell_e4201() {
    // This program hits `E4201` ("failed to load WASM module") — an
    // INTERNAL codegen defect, not a clean `E5506`. It is NOT introduced by
    // this task: the identical program with the bitwise line deleted
    // (`const w = (x) => { x.a = 1.5; }; let o = { a: 6 }; w(o);
    // console.log(o.a);`) already emits `E4201` on this build — it is the
    // deferred "float-into-i64-cell storage bug" (standing deferred list),
    // caused entirely by the arrow function `w`'s own pre-existing
    // `x.a = 1.5` write; nothing this task's new codegen touches.
    //
    // What T5 changes is REACHABILITY, not correctness: pre-T5, ANY bitwise
    // compound-assign on a member target was denied at resolve (`E5506`)
    // before codegen ever ran, so a program that also happens to contain
    // `o.a &= 3` never got far enough to hit this pre-existing codegen bug.
    // Post-T5 the admission lets compilation proceed into it.
    //
    // Closing this for real would need proving "this shape's field is never
    // written a float by ANY function that receives it as a parameter" —
    // the same interprocedural FLOAT-write taint infrastructure the
    // Important-2 finding explicitly says not to build this task (partial
    // coverage of only SOME write routes would be actively dangerous, a
    // false sense of soundness — see `collect_bigint_tainted_shape_fields`'s
    // doc). Recorded here as an explicit, PINNED, accepted unmasking rather
    // than a silent one. If a future task closes the underlying
    // float-into-i64-cell bug (or adds the missing parameter-write taint
    // route), this test should start failing with a clean `E5506` instead —
    // that is progress; update this assertion to `assert_fails_closed` at
    // that point, don't just delete the test.
    let out = run_source(
        "const w = (x) => { x.a = 1.5; }; let o = { a: 6 }; w(o); o.a &= 3; console.log(o.a);\n",
    );
    assert!(
        !out.status.success(),
        "expected a compile failure, got success: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E4201"),
        "expected the documented pre-existing E4201; got: {stderr}"
    );
}

// --- T5 review Important 2: `collect_bigint_tainted_shape_fields` sees only
// 2 of (at least) 5 source-language write routes into an admitted field.
// These three tripwires pin the CURRENT (write-silently-dropped) behavior
// for the uncovered routes — NOT as certified-correct output (all three
// diverge from node, which throws), but so that a future change which makes
// any of these writes real is caught immediately if it does not ALSO extend
// the taint scan first. See the scan's own doc in `lower.rs` for the full
// route inventory and why partial coverage is dangerous. ---

#[test]
fn bitwise_compound_tripwire_computed_key_write_not_covered_by_bigint_taint_scan() {
    // `o["a"] = 7n` is a computed-key write the taint scan does not walk.
    // It is currently silently dropped (pre-existing, reproduces with no
    // bitwise op in the program at all), so `o.a` is still `6` when
    // `o.a &= 3` runs: 6 & 3 = 2. node throws `TypeError: Cannot mix
    // BigInt`. If this write ever becomes real, this value must change —
    // if it silently doesn't (this test still passes) while the taint scan
    // is untouched, that is exactly the unsoundness this pin exists to
    // catch.
    assert_stdout(
        "let o = {a: 6}; o[\"a\"] = 7n; o.a &= 3; console.log(o.a);\n",
        "2\n",
    );
}

#[test]
fn bitwise_compound_tripwire_arrow_parameter_write_not_covered_by_bigint_taint_scan() {
    // A dot-field write off an arrow-function PARAMETER — params carry
    // their `Repr::Object(shape)` via `ReprTable::set_param`, a different
    // accessor than the `scalar()` lookup this scan's write-detection
    // consults, so this base is invisible to it. Currently silently
    // dropped (reproduces with no bitwise op at all), so `o.a` is still `6`
    // when `o.a &= 3` runs: 6 & 3 = 2. node throws `TypeError: Cannot mix
    // BigInt`.
    assert_stdout(
        "const w = (x) => { x.a = 7n; }; let o = { a: 6 }; w(o); o.a &= 3; console.log(o.a);\n",
        "2\n",
    );
}

#[test]
fn bitwise_compound_tripwire_forof_element_write_not_covered_by_bigint_taint_scan() {
    // The third uncovered route: a `for..of` element dot-field write.
    // Currently the WHOLE for-of loop over an array of objects is denied by
    // a separate, pre-existing structural gate (unrelated to the taint
    // scan) — this program fails closed today for that reason alone, not
    // because the taint scan caught anything. If a future task admits this
    // for-of shape without ALSO extending `collect_bigint_tainted_shape_fields`
    // to see element writes, a BigInt reaching the field through this route
    // would go undetected. node throws `TypeError: Cannot mix BigInt`.
    assert_fails_closed(
        "let os = [{a: 6}]; for (const o of os) { o.a = 7n; } os[0].a &= 3; console.log(os[0].a);\n",
        "for-of",
    );
}

// =====================================================================
// --- Task 6: the default-deny audit ---
//
// Task 6 is not a feature task. Its job is the standing R-11 invariant:
// for EVERY assignment target and EVERY one of the six operators, kali
// either computes node's value or fails closed with `E5506` at a nonzero
// exit — never the unmodified operand at exit 0 (the R-11 signature
// failure), and never an internal `E4201`.
//
// The audit corpus itself (37 target kinds x 6 ops = 222 cells, plus 1212
// float/string/BigInt laundering routes and 85 read routes) lives in the
// task report; what is pinned HERE is every row the audit MOVED and every
// invariant a future change could break silently.
// =====================================================================

#[test]
fn bitwise_compound_on_unsupported_targets_fails_closed() {
    // Array element on a const-literal array (R-06-R3 / R-12 lane): `+=` is
    // not a sound lowering here, so bitwise must fail closed — never a
    // silent no-op.
    //
    // Needle note: the brief's draft used `"unavailable"`, the wording every
    // per-LANE arm's own message uses. Both rows here are denied EARLIER, by
    // the Task 1.5 resolve gate's generic default-deny, whose wording is
    // different — so the needle pins the arm that actually fires. Getting
    // this wrong would have made the pin pass on an unrelated diagnostic,
    // which is the failure mode `assert_fails_closed`'s needle exists to
    // prevent.
    assert_fails_closed(
        "const a = [6]; a[0] <<= 2; console.log(a[0]);\n",
        "no codegen lowering yet",
    );
    // Computed variable key (R-13 lane).
    assert_fails_closed(
        "const o = { a: 6 }; const k = \"a\"; o[k] <<= 2; console.log(o[k]);\n",
        "no codegen lowering yet",
    );
}

#[test]
fn bitwise_compound_never_returns_unmodified_operand() {
    // The R-11 signature failure: exit 0 with the operand unchanged. For
    // every target this must be impossible — either the value changed
    // (lowered) or the program failed closed (nonzero exit). These two
    // scalar shapes MUST lower, and the asserted value is one the
    // unmodified operand can never be.
    assert_stdout("let n = 6; n &= 0; console.log(n);\n", "0\n"); // 0, not 6
    assert_stdout("let n = 6; n ^= 6; console.log(n);\n", "0\n"); // 0, not 6
}

// --- Task 6 work item 1: BigInt taint-scan write route 6 (a declarator
// init that is NOT an object literal) and route 7 (a whole-binding
// reassignment). Both were live silent miscompiles on `e2578aa7e`:
// `function m() { return {a: 7n}; } let o = m(); o.a &= 3;` printed `3`
// at exit 0 where node throws `TypeError: Cannot mix BigInt`. Closed by
// `taint_shape_fields_from_object_inflow` (`kali_codegen::lower`), which
// taints every field of the shape on ANY inflow it cannot fully parse as
// an object literal. ---

#[test]
fn bitwise_compound_fails_closed_on_object_field_from_non_literal_declarator_init() {
    // Route 6a: the object arrives from a function RETURN.
    assert_fails_closed(
        "function m() { return { a: 7n }; }\nlet o = m();\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
    // Route 6b: the BigInt is threaded through a PARAMETER into the literal
    // the callee returns.
    assert_fails_closed(
        "function m(v) { return { a: v }; }\nlet o = m(7n);\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
}

#[test]
fn bitwise_compound_fails_closed_on_object_field_from_whole_binding_reassignment() {
    // Route 7a: `o = { a: 7n }` — a reassignment of the whole binding, not
    // of one field. (The underlying object reassignment is itself a
    // PRE-EXISTING silently-dropped write — `let o={a:6}; o={a:9};
    // console.log(o.a)` prints `0` with no bitwise op anywhere — so what
    // this route buys is refusal, not a correct value.)
    //
    // SCOPE (T6 review Important 2): route 7 taints only when the inflow is
    // NOT a cleanly-parsed, provably-safe literal. A SAFE-literal
    // reassignment such as `o = { a: 22 }` is still ADMITTED and still reads
    // the dropped write's garbage — pinned, with its plain-operator sibling,
    // in `bitwise_compound_tripwire_object_reassignment_write_is_dropped`.
    assert_fails_closed(
        "let o = { a: 6 };\no = { a: 7n };\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
    // Route 7b: same, reached from a declarator with no initializer.
    assert_fails_closed(
        "let o;\no = { a: 7n };\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
}

#[test]
fn bitwise_compound_over_denies_object_fields_reached_by_a_non_literal_inflow() {
    // DELIBERATE-COST PINS for the work-item-1 closure, not correctness
    // pins — the sibling of
    // `bitwise_compound_over_denies_write_values_outside_the_numeric_proof`
    // on the object-field lane. The taint is keyed by SHAPE, so ANY binding
    // of the shape reached by an inflow the scan cannot parse denies the
    // bitwise compound assign on EVERY binding of that shape.
    //
    // Measured against `e2578aa7e`: this closure moved 17 rows of the 1212-row
    // laundering corpus — 2 from a WRONG VALUE to fail-closed (the two rows
    // pinned above) and 15 from a correct value to fail-closed (the rows this
    // test samples). Every move is ok -> DENY; none is ok -> wrong.
    //
    // These rows are EXPECTED to flip back to `assert_stdout` value
    // assertions one day. The follow-up that does it is teaching the scan to
    // FOLLOW a non-literal inflow (a callee's returned literal, an alias's
    // own declarator) — NOT deleting the default-deny.

    // The object arrives from a call; the field value is a plain integer and
    // node computes 2.
    assert_fails_closed(
        "function m() { return { a: 22 }; }\nlet o = m();\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
    // An ALIAS of a perfectly safe literal-initialised object.
    assert_fails_closed(
        "let src = { a: 22 };\nlet o = src;\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
    // A SECOND, unrelated binding of the same shape reached by a call: the
    // shape-level key denies the literal-initialised `o` too. node: 2.
    assert_fails_closed(
        "function m() { return { a: 5 }; }\nlet q = m();\nlet o = { a: 22 };\no.a &= 3;\nconsole.log(o.a);\n",
        "BigInt",
    );
}

// --- Task 6 work item 5: `expr_is_provably_not_bigint` (and its float-axis
// twin `expr_is_provably_i64_literal_or_arith`) had no self-reference arm,
// unlike the `write_value_is_numeric` they claim to mirror, so the
// commonest counter idiom in JS denied the whole lane. ---

#[test]
fn bitwise_compound_admits_a_self_referential_counter_write() {
    // Module-global lane: `n = n + 1` is a self-reference. node: 3.
    assert_stdout(
        "let n = 6;\nfunction g() { n = n + 1; }\ng();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "3\n",
    );
    // Captured env-cell lane: same write, same result. node: 3.
    assert_stdout(
        "function o() {\n  let n = 6;\n  function g() { n = n + 1; }\n  function s() { n &= 3; }\n  g(); s();\n  console.log(n);\n}\no();\n",
        "3\n",
    );
    // The arm must not be dead for a binding whose NAME ends in `n`: the
    // bare-`Value` BigInt-literal check is `text.ends_with('n')`, so a
    // self-check placed after it would never fire for `n` itself — which is
    // the name every program in this file uses. (This is why the two rows
    // above are the load-bearing ones; a differently-named binding would
    // have passed either way.)
    assert_stdout(
        "let count = 6;\nfunction g() { count = count + 1; }\ng();\nfunction f() { count &= 3; }\nf();\nconsole.log(count);\n",
        "3\n",
    );
}

#[test]
fn bitwise_compound_self_reference_arm_does_not_launder_bigint_or_float() {
    // The self-reference arm is sound ONLY because the taint set is a union
    // over every write: a BigInt/float introduced by any OTHER write is
    // still caught. Each row below has a self-referential write AND a
    // second, tainting write; each must still fail closed.

    // BigInt declarator + self-referential BigInt arithmetic.
    assert_fails_closed(
        "let n = 6n;\nfunction g() { n = n + 1n; }\ng();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "&=",
    );
    // Plain declarator, but the self-referential expression itself carries a
    // BigInt leaf — the leaf is refused, so the whole write is refused.
    assert_fails_closed(
        "let n = 6;\nfunction g() { n = n + 1n; }\ng();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "&=",
    );
    // A second, non-self write from a BigInt-holding global.
    assert_fails_closed(
        "let n = 6;\nlet q = 7n;\nfunction g() { n = n + 1; n = q; }\ng();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "&=",
    );
    // `/` is not in the arithmetic allowlist, so a self-referential division
    // is still unproven. node: 3.
    assert_fails_closed(
        "let n = 6;\nfunction g() { n = n / 2; }\ng();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "&=",
    );
    // Float axis, captured lane: a self-referential integer write plus a
    // float write elsewhere. node: 2.
    assert_fails_closed(
        "function o() {\n  let n = 6;\n  function g() { n = n + 1; }\n  function h() { n = 6.5; }\n  function s() { n &= 3; }\n  g(); h(); s();\n  console.log(n);\n}\no();\n",
        "captured binding",
    );
}

// --- Task 6 work item 3: KNOWN, OUT-OF-SCOPE RESIDUAL. Imported modules are
// never analyzed, so neither the R-11 resolve gate nor any codegen guard can
// fire inside imported code. This is the tracked pre-existing "static named
// imports never link" bug (an imported `f(x){return x+1}` also returns 0),
// NOT an R-11 hole — but it is the one place where an unsound bitwise
// compound assign provably raises no diagnostic, so it is pinned rather than
// left to be rediscovered. Confirmed still behaving this way at Task 6:
// `lib.ts` exporting `function bad(){ let s="hi"; let n=22; n &= s; return n; }`
// runs to exit 0 with zero diagnostics.
//
// There is no `assert_*` for it here because `run_source` writes a single
// file; the measurement is recorded in the Task 6 report with its transcript.
// If multi-file test support is ever added, this becomes a real pin. ---

// --- Task 6 work item 2 / register: routes whose wrong value is entirely
// PRE-EXISTING and reproduces with NO bitwise operator in the program.
// Pinned as tripwires (current behavior, NOT certified correct) so that
// fixing the underlying bug is noticed here rather than silently changing
// an R-11 lane's output. ---

#[test]
fn bitwise_compound_tripwire_object_reassignment_write_is_dropped() {
    // `o = { a: 22 }` after `let o = { a: 6 }` is silently dropped:
    // `console.log(o.a)` prints `0` with NO bitwise op in the program at all
    // (node prints 22). Pre-existing; identical on `e416b22a1`.
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\nconsole.log(o.a);\n",
        "0\n",
    );
    // T6 review Important 2 — CORRECTION. An earlier version of this comment
    // claimed the bitwise form of this shape is "now REFUSED (route 7)", so
    // R-11 could not surface the garbage. That was FALSE. Route 7 taints only
    // when the inflow is NOT a cleanly-parsed, provably-safe literal, so a
    // SAFE-literal reassignment is still admitted and the bitwise op does read
    // the dropped write's garbage:
    //
    //   let o = {a:6}; o = {a:22}; o.a |= 3;   -> kali 3, node 23
    //
    // This is the same defence as the local-scalar BigInt residual below, and
    // it is a real defence, not an excuse: kali's own PLAIN operator gives the
    // identical `3` on the pre-task binary, on `e2578aa7e` and on HEAD, so the
    // compound form agrees with the plain form and the wrongness is entirely
    // the deferred dropped-write bug. Pinned in BOTH forms so neither can move
    // without the other.
    //
    // What these rows DO and DO NOT discriminate (measured, not assumed). The
    // stale field reads back as `0`, so the expected `3` is `0 | 3`. That
    // catches three real regressions: the row failing closed (nonzero exit),
    // the row starting to print node's `23` (the dropped write became real —
    // at which point these flip to `23` and to nothing else), and the bitwise
    // arm becoming a no-op again (`o.a` would read `0`, not `3`). It does NOT
    // discriminate `|=` from `^=`: over a zero base the two coincide, verified
    // by forcing the shared combiner to `^=` and rebuilding — this row stayed
    // `3` while the ordinary admitted row moved `23` -> `21`. The
    // operator-discriminating coverage lives in
    // `bitwise_compound_on_object_field`, not here.
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a |= 3;\nconsole.log(o.a);\n",
        "3\n",
    );
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\nconsole.log(o.a | 3);\n",
        "3\n",
    );
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a ^= 3;\nconsole.log(o.a);\n",
        "3\n",
    );
    // The no-initializer form behaves identically.
    assert_stdout(
        "let o;\no = { a: 22 };\no.a |= 3;\nconsole.log(o.a);\n",
        "3\n",
    );
    assert_stdout("let o;\no = { a: 22 };\nconsole.log(o.a | 3);\n", "3\n");
    // The rest of the class. A denser A/B (10 object-inflow routes x 13 values
    // x 3 ops = 390 programs) reported the wrong-value rows here as
    // {`&=`, `|=`, `<<=`} x {reassign, no-init form} — but THREE was the
    // operator set that corpus happened to run, not a measured exhaustion.
    // Re-measured across all six: every one is wrong on this shape
    // (`&=` 0/2, `|=` 3/23, `^=` 3/21, `<<=` 0/176, `>>=` 0/2, `>>>=` 0/2,
    // kali/node), so `>>=` and `>>>=` were unpinned members of a pinned class.
    // All six are pinned now.
    //
    // The rows expecting `0` do NOT discriminate a no-op (over a zero base
    // `&`, `<<`, `>>` and `>>>` all give `0` whether the arm ran or not); they
    // are here so the class cannot move partially. The `3` rows do
    // discriminate — see the note above.
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a &= 3;\nconsole.log(o.a);\n",
        "0\n",
    );
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a <<= 3;\nconsole.log(o.a);\n",
        "0\n",
    );
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a >>= 3;\nconsole.log(o.a);\n",
        "0\n",
    );
    assert_stdout(
        "let o = { a: 6 };\no = { a: 22 };\no.a >>>= 3;\nconsole.log(o.a);\n",
        "0\n",
    );
}

#[test]
fn bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float() {
    // DELIBERATE-COST PIN, not a correctness pin — the third member of the
    // family `bitwise_compound_over_denies_write_values_outside_the_numeric_proof`
    // and `bitwise_compound_over_denies_object_fields_reached_by_a_non_literal_inflow`
    // belong to.
    //
    // EVERY name-keyed taint set in R-11 (`module_global_bigint_targets`,
    // `captured_cell_bigint_targets`, `captured_cell_float_targets` and T6
    // review round 1's `module_global_float_targets`) is keyed by NAME ONLY,
    // never by (owner, name). That is documented as a deliberate,
    // conservative over-denial on the captured lane and inherited verbatim by
    // the module-global lane, because the module-global float scan REUSES the
    // captured lane's walk rather than mirroring it. The consequence: an
    // UNRELATED local in some other function, sharing the target's name,
    // taints the module global.
    //
    // Below, `flags` the module global only ever holds `6`; the float lives in
    // a completely separate binding that happens to be spelled the same.
    // node 14, pre-task `e416b22a1` 14, parent `e2578aa7e` 14, HEAD E5506 —
    // so this IS a regression of a previously-correct program, in the
    // fail-closed direction. It was not visible until a SHADOWING axis was
    // added to the audit corpus; on that corpus 42 rows move MATCH -> E5506
    // (7 shadow forms x {float, NaN, Infinity} x 2 lanes) with zero
    // `ok -> wrong`. That 42 is a property of the corpus, not a bound.
    //
    // The follow-up that recovers these rows is keying the taint sets by
    // (owner, name) — for BOTH lanes at once, since they now share one
    // implementation — NOT deleting the scan, which is load-bearing against
    // the `E4201` it was added to close.
    assert_fails_closed(
        "let flags = 6;\nfunction other() { let flags = 6.5; return flags; }\nother();\nfunction f() { flags |= 8; }\nf();\nconsole.log(flags);\n",
        "module global",
    );
    // Same shape, `NaN` instead of a float literal. node: 14.
    assert_fails_closed(
        "let flags = 6;\nfunction other() { let flags = NaN; return flags; }\nother();\nfunction f() { flags |= 8; }\nf();\nconsole.log(flags);\n",
        "module global",
    );
    // The shadow does not even have to be reachable — `other` is never called.
    // node: 14.
    assert_fails_closed(
        "let flags = 6;\nfunction other() { let flags = 6.5; return flags; }\nfunction f() { flags |= 8; }\nf();\nconsole.log(flags);\n",
        "module global",
    );
    // The admitted lane is NOT lost when the shadow holds an integer — this
    // row proves the over-denial is keyed on the VALUE class, not merely on
    // the existence of a same-named binding. node: 14.
    assert_stdout(
        "let flags = 6;\nfunction other() { let flags = 9; return flags; }\nother();\nfunction f() { flags |= 8; }\nf();\nconsole.log(flags);\n",
        "14\n",
    );
}

#[test]
fn bitwise_compound_tripwire_local_scalar_bigint_target_matches_the_plain_operator() {
    // STANDING DEFERRED ITEM ("local-lane BigInt truncation" / "BigInt
    // through the ten pre-existing compound ops"), pinned so it cannot move
    // silently. The local-scalar lane has no BigInt taint scan (the
    // module-global, captured-cell and object-field lanes all do), so a
    // BigInt-literal target is treated as a plain i64.
    //
    // The pre-task binary (`e416b22a1`) printed the UNMODIFIED operand `7`
    // here — the R-11 signature failure. HEAD prints `3`, which is exactly
    // what kali's own PLAIN operator prints for `let m = n & 3` on BOTH the
    // pre-task binary and HEAD. node throws `TypeError: Cannot mix BigInt`
    // for all of them. So R-11 makes the compound form agree with the plain
    // form; it does not introduce the BigInt model's wrongness.
    assert_stdout("let n = 7n;\nn &= 3;\nconsole.log(n);\n", "3\n");
    assert_stdout("let n = 7n;\nlet m = n & 3;\nconsole.log(m);\n", "3\n");
    // The truncation row from the Task 2 review, same class.
    assert_stdout("let n = 4294967296n;\nn |= 0;\nconsole.log(n);\n", "0\n");
    assert_stdout(
        "let n = 4294967296n;\nlet m = n | 0;\nconsole.log(m);\n",
        "0\n",
    );
}

// --- Task 6 Global-Constraint audit: every remaining `E4201` reachable with a
// bitwise compound assign in the program. The constraint is that R-11 never
// turns a clean `E5506` into an internal invalid-module error; these rows are
// pinned because they are `E4201`, so a future change that makes R-11 the
// CAUSE of one would have to move a pin instead of landing silently.
//
// THE CLAIM THAT HOLDS — and read the next paragraph before strengthening it:
// **no `E4201` is CAUSED by an R-11 admission.** Every `E4201` shape found so
// far is `E4201` on the pre-task binary `e416b22a1` as well, with the bitwise
// line, without it, with `+=` in its place, and on a plain `n & 3` read. The
// root cause is the standing deferred "float-into-i64-cell storage bug" (and,
// for the shadow shape, the general shadowing bug). Whoever fixes those flips
// these pins to value assertions (node's values are in the comments), NOT
// deletes them.
//
// DO NOT restate this as a COUNT. Earlier revisions of this comment said
// "exactly two such shapes remain, both on the object-field lane". A five-line
// probe falsified it — a `const` shadow in an unrelated function is a third:
//
//   let n = 6; function other() { const n = 6.5; return n; } other();
//   function f() { n &= 3; } f(); console.log(n);     // E4201; node: 2
//
// That is the THIRD time in this task an absolute derived from a corpus was
// falsified by an axis the corpus lacked (object-inflow, then self-reference,
// then shadowing). The census is a lower bound on what exists, never an
// enumeration of it. State the DIRECTION, not the count, unless the axis is
// proven exhaustive. Latest census, 2028-row corpus, four binaries:
// pre-task 59, parent `e2578aa7e` 6, `d61821a46` 12, HEAD 4 — and all 4 of
// HEAD's are `E4201` on the pre-task binary too.
//
// Shapes that used to reach `E4201` and no longer do, each with a fail-closed
// pin instead of an `E4201` pin:
//   * a float write to a CAPTURED cell — closed by Task 4's
//     `collect_float_tainted_captured_cells`, pinned by
//     `bitwise_compound_fails_closed_on_float_write_reaching_captured_target_from_sibling_function`;
//   * a float write to a MODULE GLOBAL — closed by Task 6 review round 1's
//     `collect_float_tainted_module_scalars`, pinned immediately below. That
//     one E4201'd on `e416b22a1` AND on `e2578aa7e`. ---

fn assert_fails_with_e4201(src: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "expected a failure, got: {out:?}");
    assert!(stderr.contains("E4201"), "expected E4201, got: {stderr}");
}

#[test]
fn bitwise_compound_fails_closed_on_module_global_written_with_a_float() {
    // T6 review Important 1. The module-global lane's four other guards all
    // ACCEPT a float written from another function — `is_f64` reads the
    // promoted slot's own repr, and `binding_is_proven_numeric` rests on
    // `write_value_is_numeric`, whose literal arm accepts `6.5`. The lane was
    // safe only INCIDENTALLY: such a program almost always also carried a
    // write the BigInt scan could not prove. T6's self-reference arm removed
    // one such over-taint (`n = n + 1`) and the hole underneath became a
    // reachable `E4201` — a Global-Constraint violation. Closed by
    // `collect_float_tainted_module_scalars`, the module-global twin of the
    // captured lane's float scan.
    //
    // Row 1 is the shape the self-reference arm exposed; it fails closed on
    // the parent `e2578aa7e` too, but for the wrong reason (BigInt over-taint),
    // so it would have gone green either way — the mutation test in the task
    // report is the real evidence. node: 2.
    assert_fails_closed(
        "let n = 6;\nfunction g() { n = n + 1; }\nfunction h() { n = 6.5; }\nh();\nfunction f() { n &= 3; }\nf();\nconsole.log(n);\n",
        "module global",
    );
    // Rows 2 and 3 have NO self-reference at all: they produced `E4201` on
    // `e2578aa7e` AND on the pre-task `e416b22a1`, so these are the
    // non-vacuous ones — the lane is now better than it has ever been. node: 2.
    assert_fails_closed(
        "let n = 6;\nfunction ww() { n = 6.5; }\nww();\nn &= 3;\nconsole.log(n);\n",
        "module global",
    );
    assert_fails_closed(
        "let n = 6;\nfunction ww() { n = 6.5; }\nww();\nfunction f() { n |= 3; }\nf();\nconsole.log(n);\n",
        "module global",
    );
}

#[test]
fn bitwise_compound_object_field_computed_key_float_write_hits_preexisting_e4201() {
    // node: 2. Pre-existing: identical with the `o.a &= 3;` line deleted, and
    // identical on `e416b22a1`.
    assert_fails_with_e4201("let o = { a: 6 };\no[\"a\"] = 6.5;\no.a &= 3;\nconsole.log(o.a);\n");
}
