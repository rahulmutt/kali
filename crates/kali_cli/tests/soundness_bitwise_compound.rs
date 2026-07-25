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
    let needle = "&=";
    assert_fails_closed(
        // Member target (`o.a`).
        "let o = { a: 6 }; o.a &= 3; console.log(o.a);\n",
        needle,
    );
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
        "shadowed by a same-named module global",
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
