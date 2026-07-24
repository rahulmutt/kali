# R-11 Bitwise Compound Assignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `&= |= ^= <<= >>= >>>=` compute the correct value on integer targets and fail closed (`E5506`) on every other target, closing the R-11 silent-no-op class (currently 48/48 silent).

**Architecture:** Extract the JS-int32 result semantics from the existing plain-operator lowering (`emit_bitwise`) into one shared combiner, then admit the six bitwise compound ops at the single `emit_assignment` allowlist gate that currently short-circuits them to a silent bare-read, and give each assignment target arm a bitwise case that mirrors its arithmetic `+=` sibling (lower for integer targets, `E5506` otherwise). No target's `false` fall-through survives.

**Tech Stack:** Rust; `wasm-encoder` (`Function`, `Instruction`); the kali codegen crate `crates/kali_codegen`; integration tests in `crates/kali_cli/tests` driving the built `kali` binary against node-captured expectations.

## Global Constraints

- **Oracle:** `node v26.5.0`. Every expected stdout value is what node prints. Binary under test: the `kali` build (`CARGO_BIN_EXE_kali` in tests).
- **Reuse, do not re-derive, int32 semantics.** All bitwise result semantics live in ONE function shared by the plain and compound forms. Never inline `I32Shl`/`I32And`/extend logic at a second site.
- **Allowlist, never denylist.** Admit integer targets explicitly; everything else fails closed `E5506`. Never add a "shape to skip" list.
- **No silent fall-through.** After this change, no path through `emit_assignment` for a bitwise op may `return false` (caller turns that into a silent bare read) or emit a store of a wrong-width/wrong-type value. Non-integer targets and float RHS fail closed with `E5506` — never an internal `E4201`.
- **No new host imports, no new synthetic functions.** Pure linear-memory / opcode work. The 4-way `kali:rt` import-sync surface and the `count_tag_boxing_ops` synthetic census stay untouched (verify, don't assume).
- **Diagnostic code:** honest refusals use `e5::FEATURE_UNAVAILABLE` (`E5506`) via `Diagnostic::error`.
- **Gate:** `cargo test --workspace` diffed against a `main` worktree = **0 newly-red**; `cargo fmt --check` + `cargo clippy -- -D warnings` clean; 6/6 CLBG goldens + web-baseline byte-for-byte unchanged.

---

## File map

- `crates/kali_codegen/src/emit/operators.rs` — refactor `emit_bitwise` (`:1610`) to call a new shared helper `emit_bitwise_i32_op_extend`.
- `crates/kali_codegen/src/emit/literal.rs` — `emit_assignment` allowlist gate (`:222`); local-scalar bitwise arm + default-deny (`:808` match, `:1035` default); module-global bitwise arm (`emit_module_global_assignment`, `:1049`).
- `crates/kali_codegen/src/emit/closure_access.rs` — captured env-cell bitwise arm (`try_emit_captured_assign`, `:165`).
- `crates/kali_codegen/src/emit/object.rs` — heap-object integer-field bitwise arm + float-field rejection (`emit_object_field_compound_assign_dynamic`, `:340`).
- `crates/kali_cli/tests/soundness_bitwise_compound.rs` — **create**; correctness + fail-closed pins.
- `docs/superpowers/followups/kali-silent-miscompile-register.md` — mark R-11 closed (Task 7).

---

## Task 1: Shared int32 combiner (refactor `emit_bitwise`, behavior-neutral)

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:1610-1639`
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs` (create — reference pins only this task)

**Interfaces:**
- Produces: `fn emit_bitwise_i32_op_extend(&mut self, function: &mut Function, op: &str)` — consumes two `i32` values on the value stack (left pushed first, then right), applies the JS bitwise op, and pushes the `i64` result (sign-extended, or zero-extended for `>>>`). Accepts BOTH the plain op text (`"<<"`) and the compound op text (`"<<="`). This is the sole home of bitwise result semantics; Tasks 2–5 call it.

- [ ] **Step 1: Write the failing reference test (create the test file with its harness)**

Create `crates/kali_cli/tests/soundness_bitwise_compound.rs`:

```rust
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
    assert!(stderr.contains(needle), "expected {needle:?}, got: {stderr}");
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
```

- [ ] **Step 2: Run the test to verify it passes against the CURRENT binary (baseline capture)**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound plain_binary_bitwise_operators_unchanged`
Expected: PASS (these are plain operators, already correct — this pins the pre-refactor behavior so Step 4 can prove neutrality).

- [ ] **Step 3: Extract the shared combiner and route `emit_bitwise` through it**

In `crates/kali_codegen/src/emit/operators.rs`, replace the body of `emit_bitwise` (`:1616-1638`, the `self.emit_float_operand ... EmittedValue { ... }` block) and add the helper immediately after it:

```rust
    fn emit_bitwise(
        &mut self,
        function: &mut Function,
        op: &str,
        left: LirNodeId,
        right: LirNodeId,
    ) -> EmittedValue {
        self.emit_float_operand(function, left, false);
        function.instruction(&Instruction::I32WrapI64);
        self.emit_float_operand(function, right, false);
        function.instruction(&Instruction::I32WrapI64);
        self.emit_bitwise_i32_op_extend(function, op);
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Applies a JS bitwise op to two `i32` operands already on the value stack
    /// (left pushed first, then right) and extends the `i32` result back to
    /// `i64` — sign-extended for every op except `>>>`, which zero-extends
    /// (uint32). The SOLE home of bitwise result semantics: the plain operators
    /// (`emit_bitwise`) and every compound-assignment target arm route through
    /// here, so the two forms cannot desynchronize. Accepts both the plain op
    /// text (`"<<"`) and the compound op text (`"<<="`).
    pub(crate) fn emit_bitwise_i32_op_extend(&mut self, function: &mut Function, op: &str) {
        match op {
            "&" | "&=" => function.instruction(&Instruction::I32And),
            "|" | "|=" => function.instruction(&Instruction::I32Or),
            "^" | "^=" => function.instruction(&Instruction::I32Xor),
            "<<" | "<<=" => function.instruction(&Instruction::I32Shl),
            ">>" | ">>=" => function.instruction(&Instruction::I32ShrS),
            ">>>" | ">>>=" => function.instruction(&Instruction::I32ShrU),
            _ => unreachable!("emit_bitwise_i32_op_extend called with non-bitwise op"),
        };
        if matches!(op, ">>>" | ">>>=") {
            function.instruction(&Instruction::I64ExtendI32U);
        } else {
            function.instruction(&Instruction::I64ExtendI32S);
        }
    }
```

- [ ] **Step 4: Run the reference test to verify the refactor is neutral**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound plain_binary_bitwise_operators_unchanged`
Expected: PASS (identical output — proves the extract changed no behavior).

- [ ] **Step 5: Confirm the crate builds clean**

Run: `cargo build -p kali_codegen && cargo clippy -p kali_codegen -- -D warnings`
Expected: builds, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/soundness_bitwise_compound.rs
git commit -m "refactor(codegen): R-11 T1 — extract emit_bitwise_i32_op_extend shared combiner"
```

---

## Task 2: Admit at the gate + local-scalar arm + default-deny (the core case)

**Files:**
- Modify: `crates/kali_codegen/src/emit/literal.rs:222-227` (gate), `:808` match (add bitwise arm before `_ => false`), `:1035` (`_ => false` → default-deny)
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs`

**Interfaces:**
- Consumes: `emit_bitwise_i32_op_extend` (Task 1).
- Produces: bitwise compound assignment on `let`/`var`/function-local/parameter integer scalars lowers correctly; float/string target or float RHS → `E5506`.

- [ ] **Step 1: Write the failing tests**

Append to `soundness_bitwise_compound.rs`:

```rust
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
    assert_fails_closed("let x = 1.5; x <<= 1; console.log(x);\n", "unavailable");
    assert_fails_closed("let n = 6; n <<= 1.5; console.log(n);\n", "unavailable");
    assert_fails_closed("let s = \"a\"; s <<= 1; console.log(s);\n", "unavailable");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_let_scalar`
Expected: FAIL — kali prints `6` (the unmodified operand) instead of `2`.

- [ ] **Step 3: Admit the six ops at the `emit_assignment` gate**

In `crates/kali_codegen/src/emit/literal.rs:222-227`, extend the allowlist:

```rust
        if !matches!(
            op,
            "=" | "??="
                | "&&="
                | "||="
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "**="
                | "&="
                | "|="
                | "^="
                | "<<="
                | ">>="
                | ">>>="
        ) {
            return false;
        }
```

- [ ] **Step 4: Add the local-scalar bitwise arm and convert the default**

In the `match op` inside the local branch (`literal.rs:808`), add a new arm immediately BEFORE `_ => false` (`:1035`), then replace `_ => false` with a fail-closed default:

```rust
            "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => {
                // JS bitwise compound: a op= b ≡ a = ToInt32(a) <op> ToInt32(b).
                // Float/string targets and a float RHS have no integer meaning —
                // fail closed, mirroring emit_binary's bitwise float rejection
                // and the arithmetic arm's string rejection. (Emitting the
                // I32WrapI64 over an f64 would produce a malformed module, E4201
                // — the wrong error kind — so this guard is load-bearing.)
                if matches!(
                    self.scalar_repr(&name),
                    kali_common::Repr::F64 | kali_common::Repr::String
                ) || self.is_float_valued(right)
                {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "bitwise compound assignment '{op}' on a non-integer binding '{name}' is unavailable in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                }
                function.instruction(&Instruction::LocalGet(index));
                function.instruction(&Instruction::I32WrapI64);
                self.emit_float_operand(function, right, false);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_bitwise_i32_op_extend(function, op);
                function.instruction(&Instruction::LocalTee(index));
                true
            }
            _ => {
                // Default-deny. After the gate admits `= ??= &&= ||= += -= *= /=
                // %= **=` and the six bitwise ops (all with explicit arms above),
                // nothing reaches here. Fail closed rather than returning `false`
                // — the caller turns `false` into a silent bare read of the
                // target, which was the R-11 fail-open.
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "compound assignment '{op}' on binding '{name}' is unavailable in the current phase"
                    ),
                ));
                function.instruction(&Instruction::I64Const(0));
                true
            }
```

- [ ] **Step 5: Run the Task-2 tests to verify they pass**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_`
Expected: PASS for `on_let_scalar`, `on_var_scalar`, `int32_edges`, `in_function_scope_and_param`, `on_non_integer_fails_closed`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/literal.rs crates/kali_cli/tests/soundness_bitwise_compound.rs
git commit -m "fix(codegen): R-11 T2 — lower bitwise compound on scalar locals; gate admission + default-deny"
```

---

## Task 3: Module-global integer target

**Files:**
- Modify: `crates/kali_codegen/src/emit/literal.rs` — `emit_module_global_assignment` (`:1049`), add a bitwise arm before the `_ =>` E5506 default (`:1119`)
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs`

**Interfaces:**
- Consumes: `emit_bitwise_i32_op_extend` (Task 1).
- Produces: bitwise compound on a promoted i64 module global lowers; f64 global → `E5506`.

- [ ] **Step 1: Write the failing test**

Append:

```rust
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_module_global`
Expected: FAIL — kali prints `6`.

- [ ] **Step 3: Add the module-global bitwise arm**

In `emit_module_global_assignment` (`literal.rs:1049`), add before the `_ =>` default (`:1119`):

```rust
            "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => {
                if is_f64 || self.is_float_valued(right) {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "bitwise compound assignment '{op}' on a floating-point module global is unavailable in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::F64Const(0.0.into()));
                    return true;
                }
                function.instruction(&Instruction::GlobalGet(global_index));
                function.instruction(&Instruction::I32WrapI64);
                self.emit_float_operand(function, right, false);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_bitwise_i32_op_extend(function, op);
                function.instruction(&Instruction::GlobalSet(global_index));
                function.instruction(&Instruction::GlobalGet(global_index));
                true
            }
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_module_global`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/emit/literal.rs crates/kali_cli/tests/soundness_bitwise_compound.rs
git commit -m "fix(codegen): R-11 T3 — lower bitwise compound on module-scope integer globals"
```

---

## Task 4: Captured env-cell integer target

**Files:**
- Modify: `crates/kali_codegen/src/emit/closure_access.rs` — `try_emit_captured_assign` (`:165`): widen the guard (`:172`) and add a bitwise branch
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs`

**Interfaces:**
- Consumes: `emit_bitwise_i32_op_extend` (Task 1); `crate::closure::emit_cell_load` / `emit_cell_store` (existing).
- Produces: bitwise compound on a C1-promoted captured scalar lowers via its env cell.

- [ ] **Step 1: Write the failing test**

Append:

```rust
// --- Task 4: captured scalar (env-cell) target ---

#[test]
fn bitwise_compound_on_captured_scalar() {
    // `flags` is captured and compound-assigned by a sibling closure — the
    // Stage C env-cell write path (try_emit_captured_assign).
    assert_stdout(
        "function outer() {\n  let flags = 6;\n  function set() { flags |= 8; }\n  set();\n  console.log(flags);\n}\nouter();\n",
        "14\n",
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_captured_scalar`
Expected: FAIL. (Today the op is not in the `try_emit_captured_assign` guard, so it returns `None`; the caller then hits the local lookup and — because the name is a captured cell, not a local — the compound-not-a-local `E5506` at `literal.rs:802` may fire, or a silent path. Either way it is not `14`.)

- [ ] **Step 3: Add the captured-cell bitwise branch**

In `try_emit_captured_assign` (`closure_access.rs:165`), widen the guard at `:172` and add a branch in the `match op`:

```rust
        if !matches!(
            op,
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>="
        ) {
            return None;
        }
```

Then, inside the `match op` (after the `"+=" | "-=" | ...` arm, before the final `_ => unreachable!`), add:

```rust
            "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => {
                if self.is_float_valued(right) {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "bitwise compound assignment '{op}' on a captured binding is unavailable for a floating-point value in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return Some(true);
                }
                crate::closure::emit_cell_load(function, env_global, depth, offset);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_float_operand(function, right, false);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_bitwise_i32_op_extend(function, op);
                crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
            }
```

Note: the existing arms fall through to the `emit_cell_load` result-reload at the end of the function (`:212`), so this branch does the store and lets that shared reload produce the expression value. Confirm `e5` and `Diagnostic` are already imported in this file; if not, add `use` lines matching `literal.rs`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_captured_scalar`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/emit/closure_access.rs crates/kali_cli/tests/soundness_bitwise_compound.rs
git commit -m "fix(codegen): R-11 T4 — lower bitwise compound on captured env-cell scalars"
```

---

## Task 5: Heap-object integer field

**Files:**
- Modify: `crates/kali_codegen/src/emit/object.rs` — `emit_object_field_compound_assign_dynamic` (`:340`): add a bitwise branch in the integer (`_ =>`) arm and reject bitwise in the F64 arm
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs`

**Interfaces:**
- Consumes: `emit_bitwise_i32_op_extend` (Task 1).
- Produces: bitwise compound on a fixed-shape / for-in integer object field lowers; float field → `E5506`.

- [ ] **Step 1: Write the failing test**

Append (uses the supported heap-object field lane; if the exact object surface is not yet reachable for a direct dot-field compound, this test doubles as the reachability probe — see Task 6):

```rust
// --- Task 5: integer object field ---

#[test]
fn bitwise_compound_on_object_field() {
    // A fixed-shape integer field compound-assigned. Mirrors the arithmetic
    // sibling path in emit_object_field_compound_assign_dynamic.
    assert_stdout(
        "const o = { a: 6 }; o.a <<= 2; console.log(o.a);\n",
        "24\n",
    );
}
```

- [ ] **Step 2: Run to verify it fails (and record HOW it fails)**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_object_field`
Expected: FAIL. Record whether kali prints `6` (silent no-op) or emits a diagnostic — this tells you whether the dot-field compound reaches `emit_object_field_compound_assign_dynamic` or a different arm. If it reaches a DIFFERENT arm that has no bitwise case, note the file:line; the fix goes there instead, following the same RMW shape below.

- [ ] **Step 3: Add the integer-field bitwise branch and the F64 rejection**

In `emit_object_field_compound_assign_dynamic` (`object.rs:340`), inside the integer `_ =>` arm (`:410`), immediately after the existing `**=` rejection block (`:415-426`), add:

```rust
                if matches!(op, "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=") {
                    if self.is_float_valued(rhs) {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "bitwise compound assignment '{op}' with a floating-point value on an object field is unavailable in the current phase"
                            ),
                        ));
                        function.instruction(&Instruction::Drop); // discard store address
                        function.instruction(&Instruction::I64Const(0));
                        return EmittedValue { produced: true, shape: ValueShape::Scalar };
                    }
                    // RMW using the store address already on the stack (pushed at :365)
                    // and `scratch` (holds the same address) for the current-value load.
                    function.instruction(&Instruction::LocalGet(scratch));
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::I64Load(memarg));
                    function.instruction(&Instruction::I32WrapI64);
                    self.emit_float_operand(function, rhs, false);
                    function.instruction(&Instruction::I32WrapI64);
                    self.emit_bitwise_i32_op_extend(function, op);
                    function.instruction(&Instruction::I64Store(memarg));
                    function.instruction(&Instruction::LocalGet(scratch));
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::I64Load(memarg));
                    return EmittedValue { produced: true, shape: ValueShape::Scalar };
                }
```

And in the F64 arm's rejection guard (`object.rs:373`), widen it so bitwise on a float field also fails closed:

```rust
                if matches!(op, "%=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=") {
```

(and update that block's message to read "on a floating-point object field" — it already `Drop`s the store address and returns a fail-closed value).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_object_field`
Expected: PASS (if the dot-field form routes here). If Step 2 showed it routes elsewhere, apply the identical RMW at that arm and re-run.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/emit/object.rs crates/kali_cli/tests/soundness_bitwise_compound.rs
git commit -m "fix(codegen): R-11 T5 — lower bitwise compound on integer object fields; reject float fields"
```

---

## Task 6: Default-deny audit — no silent bitwise no-op survives on ANY target

**Files:**
- Modify (only if the audit finds a live silent path): the offending arm in `crates/kali_codegen/src/emit/literal.rs` or `object.rs`
- Test: `crates/kali_cli/tests/soundness_bitwise_compound.rs`

**Interfaces:**
- Consumes: all prior tasks.
- Produces: proof that every remaining bitwise-compound target either lowers or fails closed `E5506` — never silently returns the unmodified operand and never `E4201`.

- [ ] **Step 1: Write the audit tests (each asserts NOT-silent: correct value OR fail-closed)**

Append. For target kinds whose `+=` sibling is itself unsound/silent (aliased array element, computed variable key, const-array element — the R-12/R-13/R-06-R3 lanes), the correct R-11 outcome is **fail-closed**, not a value:

```rust
// --- Task 6: every remaining target is NOT a silent no-op ---

#[test]
fn bitwise_compound_on_unsupported_targets_fails_closed() {
    // Array element on a const-literal array (R-06-R3 / R-12 lane): += is not a
    // sound lowering here, so bitwise must fail closed — never a silent no-op.
    assert_fails_closed("const a = [6]; a[0] <<= 2; console.log(a[0]);\n", "unavailable");
    // Computed variable key (R-13 lane).
    assert_fails_closed(
        "const o = { a: 6 }; const k = \"a\"; o[k] <<= 2; console.log(o[k]);\n",
        "unavailable",
    );
}

#[test]
fn bitwise_compound_never_returns_unmodified_operand() {
    // The R-11 signature failure: exit 0 with the operand unchanged. For every
    // target this must be impossible — either the value changed (lowered) or the
    // program failed closed (nonzero exit). This test drives the two scalar
    // shapes that MUST lower and asserts the CHANGED value, guarding against a
    // regression to the silent path.
    assert_stdout("let n = 6; n &= 0; console.log(n);\n", "0\n"); // 0, not 6
    assert_stdout("let n = 6; n ^= 6; console.log(n);\n", "0\n"); // 0, not 6
}
```

- [ ] **Step 2: Run the audit tests**

Run: `cargo test -p kali_cli --test soundness_bitwise_compound bitwise_compound_on_unsupported_targets_fails_closed bitwise_compound_never_returns_unmodified_operand`
Expected: They should PASS given Tasks 2–5 (unsupported array/computed targets hit an arm that fails closed, or fall to the default-deny). **If `on_unsupported_targets_fails_closed` FAILS because kali prints `6` (silent no-op), you have found a live fail-open** — trace which arm handled it and convert that arm's bitwise (or missing) case to `E5506`, mirroring the default-deny in Task 2 Step 4.

- [ ] **Step 3: Manual sweep — reproduce the full 8-target matrix on the built binary**

Build once: `cargo build -p kali_cli`. Then for each target kind (let, var, function-local, parameter, module-global, captured, object-field, array-element, computed) run all six ops via a scratch `.ts` file and confirm each is EITHER the node value OR a nonzero `E5506`. Record the result table in the commit message. No cell may print the unmodified operand at exit 0.

- [ ] **Step 4: Fix any live fail-open found, then re-run**

If Step 2 or 3 surfaced a silent cell, apply the fail-closed/lowering fix at that arm and re-run the whole test file:
Run: `cargo test -p kali_cli --test soundness_bitwise_compound`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test(codegen): R-11 T6 — default-deny audit; no bitwise compound silently no-ops on any target"
```

---

## Task 7: Whole-stage review, full gate, register update

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (mark R-11 closed)
- Verify: whole workspace

**Interfaces:**
- Consumes: all prior tasks.

- [ ] **Step 1: Adversarial whole-stage review**

Re-read every arm touched (gate, local, module-global, captured, object-field) as a set. For EACH, confirm: (a) integer target lowers via the shared combiner; (b) float target/RHS and string target fail closed `E5506` (never `E4201`); (c) no path returns `false` or stores a wrong-width value for a bitwise op; (d) the `>>>` unsigned extend is used only for `>>>=`. This review has caught a store-site/value-sink fail-open on every prior stage — walk each RHS shape (literal, variable, float, call).

- [ ] **Step 2: Confirm no synthetic/import census drift**

Run: `grep -rn "count_tag_boxing_ops\|SYNTHETIC_FUNCTIONS" crates/kali_cli/tests | head`
Confirm this change added no synthetic function and no host import (it adds only opcodes to existing functions), so the census is untouched. Note the confirmation in the commit.

- [ ] **Step 3: Full workspace gate against a main worktree**

```bash
git worktree add /tmp/kali-main-baseline main
( cd /tmp/kali-main-baseline && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r11-main.txt )
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r11-branch.txt
```
Expected: the set of FAILED tests on the branch is a subset of main's (0 newly-red). Diff the two FAILED lists. Clean up: `git worktree remove /tmp/kali-main-baseline`.

- [ ] **Step 4: Format, lint, goldens**

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
```
Expected: clean. Then confirm the 6 CLBG goldens + web-baseline smoke are byte-for-byte unchanged (run the existing golden/smoke tests; none touch bitwise compound, so they must be identical).

- [ ] **Step 5: Update the register**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`, update the R-11 rows (the §0 status table entry and the §2 R-11 entry) to **CLOSED 2026-07-24** with a one-paragraph note: reused `emit_bitwise_i32_op_extend` at every assignment target arm (scalar local/param, module global, captured cell, integer object field); non-integer targets and float RHS fail closed `E5506`; the `emit_assignment` gate now admits the six ops and the local-scalar `_ => false` fail-open is now default-deny.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "docs(register): R-11 bitwise compound assignment CLOSED"
```

---

## Self-review notes (for the plan author / first reviewer)

- **Spec §3.3 coverage:** local/param (T2), module-global (T3), captured env-cell (T4), object field (T5), array-element/computed/other (T6 audit → fail-closed). All target rows of the 48-cell matrix are covered by a lowering task or the default-deny audit.
- **Spec §4.1 shared combiner:** T1.
- **Spec §5 fail-closed surface:** float/string target and float RHS pinned in T2/T3/T5; unsupported element/computed targets pinned in T6.
- **Spec §7 risks:** the multi-arm hazard is handled by the T6 audit + T7 review; `emit_bitwise` neutrality by T1's before/after reference test; `>>>=` uint32 round-trip pinned in T2 `int32_edges`; captured path admits-or-denies in T4.
- **Type consistency:** the helper is `emit_bitwise_i32_op_extend(&mut self, &mut Function, &str)` everywhere; op strings are the compound texts (`"<<="`) which the helper accepts alongside the plain texts.
