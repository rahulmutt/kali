# fannkuch-redux End-to-End Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `kali run` execute the `fannkuch-redux` Computer Language Benchmarks Game program and print its exact canonical output (`228` then `Pfannkuchen(7) = 16` for n=7) by building the imperative-core execution backend: relational operators, real `return`, mutable locals, real loops, linear-memory integer arrays, and runtime integer→string + string concatenation.

**Architecture:** Kali lowers TS/JS through AST → HIR → MIR → LIR → wasm (`crates/kali_{ast,hir,mir,lir,codegen}`) and runs the wasm on a wasmtime host with custom `kali:rt` imports (`crates/kali_runtime`). HIR→MIR and MIR→LIR are pure structural tree copies that preserve each node's `text` discriminator and child order; all real lowering lives in `crates/kali_codegen/src/emit/*.rs`. Today only compile-time-constant programs execute correctly; this plan replaces the "constant-fold-or-emit-0" behavior with genuine, type-directed lowering for the integer imperative core, using fannkuch-redux as the acceptance test.

**Tech Stack:** Rust; `wasm_encoder` (module emission) and `wasmparser` (validation/inspection); `wasmtime` (execution engine); existing `kali` CLI integration tests via `Command::new(CARGO_BIN_EXE_kali)`.

**Design spec:** `docs/superpowers/specs/2026-06-30-fannkuch-redux-end-to-end-design.md`.

## Global Constraints

Every task's requirements implicitly include all of these (copied from the spec):

- **i64-only value model.** Every JS value is one `i64`. Integers are raw two's-complement in the low 63 bits (bit 63 = 0). String handles set bit 63: `STRING_HANDLE_TAG = 0x8000_0000_0000_0000`, packed `TAG | (offset << 32) | len`. Array handles are raw byte offsets into linear memory (bit 63 = 0), disambiguated by static position — never tagged.
- **No floating point.** fannkuch is integer-only; do not introduce f64.
- **No tracing/background GC.** Allocation is bump-only (no free); allocations live for the whole program. This is permitted; tracing GC is not.
- **Pure-Rust toolchain.** Add no dependency that vendors/compiles C/C++/asm (`-sys`, `cc`, `build.rs` translation units). Only existing crates.
- **AOT-only.** No language-level JIT; all lowering is ahead-of-time.
- **Engine = wasmtime.** Output runs via the existing `crates/kali_runtime` wasmtime embedding; the module exports `memory`.
- **Preserve observable outputs.** Existing constant-folded programs must print exactly what they print today. `cargo test --workspace` must stay green at the end of every task. Optimizer fixtures that assert on wasm *size/instruction counts* (in `crates/kali_cli/tests/runtime_smoke.rs`) may shift; update those evidence numbers if and only if a real lowering change moves them, and never change an expected **stdout** string.
- **Additive maturity claims only.** When you update `specs/19-feature-maturity.md`, describe exactly the supported integer slice; do not over-claim. **Do not touch `proofs/BOUNDARY.md`.**
- **Hygiene.** `cargo clippy --workspace` clean and `cargo fmt` applied before each commit.
- **Type annotations are out of scope.** TS `: number` annotations currently fail name resolution (`E3100 undefined identifier 'number'`); that is a separate gating issue. The fannkuch fixture is written annotation-free, matching the upstream JS submission.

---

## Pipeline Orientation (read before Task 1)

Facts established by reading the codebase; every task relies on them.

**LIR node shape** (`crates/kali_lir/src/node.rs`): `LirNode { kind: LirNodeKind, text: Option<String>, children: Vec<LirNodeId>, function_flavor }`. Kinds: `Program, Block, Instruction, Value, Branch, Call, Literal, Unknown`.

**Control constructs collapse to `Branch`.** HIR `WhileStmt | DoWhileStmt | ForStmt | ForInStmt | IfStmt | ReturnStmt | Break | Continue` all map to `MirNodeKind::ControlFlow` → `LirNodeKind::Branch` (`crates/kali_mir/src/lower.rs:83-97`, `crates/kali_lir/src/lower.rs:64`). `text`/child order are copied verbatim from HIR. Today **all of these have `text = None`** except `for-of`/`for-await-of` (`"for-of"`/`"for-await-of"`) and break/continue (`"break"`/`"continue"`), so `while`/`for`/`do-while`/`return`/`if` are indistinguishable and all fall into the `_ =>` arm of the `Branch` match in `emit_node` → `emit_branch` (one-shot `if`). Child layouts set in `crates/kali_hir/src/lowering/statement.rs`:
- `while` (`statement.rs:227-236`): children `[test, body]`.
- `do-while` (`statement.rs:237-246`): children `[body, test]`.
- `for` (`statement.rs:159-188`): children `[init?, test?, update?, body]` — pushed only if present; `body` is always last.
- `return` (`statement.rs:47-53`): children `[arg?]`.
- `if`: children `[test, then, else?]`.

**The `Branch` match arm** is `crates/kali_codegen/src/emit/control_flow.rs:163-174`. New construct routing is added here.

**Frame machinery** (`crates/kali_codegen/src/emitter.rs:32-133`): `ControlFlowLabelKind` enum; `LoopFrame { break_index, continue_index }`; `self.control_frames: Vec<ControlFlowLabelKind>`; `self.loop_frames: Vec<LoopFrame>`; `push_control_frame(kind) -> usize` (returns the frame index), `pop_control_frame(kind)`, `control_frame_depth(target_index) -> u32` (= `control_frames.len() - 1 - target_index`, i.e. the relative `Br` depth). `emit_break_or_continue` (`control_flow.rs:4-60`) already turns unlabeled `break`/`continue` into `Br(control_frame_depth(loop_frame.break_index|continue_index))`. The only existing real-loop example to mirror is `emit_for_of_array_iteration` (`crates/kali_codegen/src/intrinsics/array.rs:1116`).

**Locals & bindings** (`crates/kali_codegen/src/emitter.rs:65-66`): `self.locals: BTreeMap<String,u32>` (name → wasm local index), `self.bindings: BTreeMap<String,LirNodeId>` (const name → init node). Params occupy local indices `0..n`; collected `let`/`var` names occupy `n..` (`emitter.rs:90-96`). Each function declares one extra scratch i64 local (`lower.rs:269`: `Function::new(vec![(locals.len()+1, ValType::I64)])`). `const` is NOT a local — it re-emits its init expression on each read (`control_flow.rs:134-136, 214-216`). `_start` (the program entry) collects top-level `let`/`var` as its locals (`lower.rs:122-132`).

**Operators** (`crates/kali_codegen/src/emit/operators.rs:351-505`, `emit_binary`): supported binary ops emit i64 instructions: `+ - * / %` → `I64Add/Sub/Mul/DivS/RemS`; `== === != !==` → `I64Eq (+I32Eqz)` then `I64ExtendI32U`; `&& ||` → `I64And/Or`. Relational `< <= > >=` fall through to the `_ =>` default which emits `E8001` + a bogus `I64Add` (operators.rs:493-503). Before emitting operands, line 356 calls `emit_assignment(...)` (defined in `crates/kali_codegen/src/emit/literal.rs:173`), which handles `=`/compound ops for **identifier** targets via `self.locals` but returns `false` for array-element targets like `a[i] = v` (→ falls through to the `E8001 '='` default).

**Module assembly** (`crates/kali_codegen/src/lower.rs:21-329`): sections built in order type → import → function → memory → export → code → (data). Imports are `"kali:rt"` functions; import-index constants in `crates/kali_codegen/src/lib.rs:42-62`, `FUNCTION_INDEX_OFFSET = 17`. Memory: `MemorySection` with hard-coded `minimum: 1` page, exported as `"memory"` (`lower.rs:244-254`). String pool data segments start at `ENV_GET_BUFFER_RESERVED = 4096` (`lib.rs:61`); `string_pool.next_offset` is the first free byte after interned strings. **There is no `GlobalSection`** anywhere.

**Test harness pattern** (canonical example `crates/kali_cli/tests/array_callback_number_predicates_runtime.rs`): write source to a `tempdir`, `Command::new(kali_bin()).arg("run").arg(path).output()`, assert `output.status.success()` and on `String::from_utf8_lossy(&output.stdout)`. All micro-acceptance tests in this plan go in a new file `crates/kali_cli/tests/imperative_core_runtime.rs` using this pattern.

**Shared test helper** — create this once at the top of the new test file and reuse it in every task:

```rust
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Compile+run `source` as a standalone `.js` program and return its stdout.
fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "kali run failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}
```

---

## Task 1: Relational operators (`<`, `<=`, `>`, `>=`)

Independent of all other tasks; unblocks loop and recursion conditions. Today `console.log(3 < 5)` prints `8` (the bogus `I64Add` default).

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (the `match op` in `emit_binary`, around `operators.rs:397-504`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `emit_binary`'s existing operand emission (line 392-395 emits left then right onto the stack as two i64 values before the `match`).
- Produces: relational ops yield an i64 `1`/`0` with `EmittedValue { produced: true, shape: ValueShape::Boolean }`, matching the existing `==`/`!=` convention.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_cli/tests/imperative_core_runtime.rs` (include the shared helper above):

```rust
#[test]
fn relational_operators_compute_booleans() {
    assert_eq!(run_js("console.log(3 < 5);\n"), "1\n");
    assert_eq!(run_js("console.log(5 < 3);\n"), "0\n");
    assert_eq!(run_js("console.log(5 > 3);\n"), "1\n");
    assert_eq!(run_js("console.log(3 >= 3);\n"), "1\n");
    assert_eq!(run_js("console.log(2 <= 1);\n"), "0\n");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime relational_operators_compute_booleans -- --nocapture`
Expected: FAIL — first assert gets `"8\n"` (bogus `I64Add`), not `"1\n"`.

- [ ] **Step 3: Add the relational arms**

In `emit_binary`'s `match op` (`crates/kali_codegen/src/emit/operators.rs`), add these arms alongside the existing `"=="` arm (operands are already on the stack as two i64 values, left then right):

```rust
            "<" => {
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue { produced: true, shape: ValueShape::Boolean }
            }
            "<=" => {
                function.instruction(&Instruction::I64LeS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue { produced: true, shape: ValueShape::Boolean }
            }
            ">" => {
                function.instruction(&Instruction::I64GtS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue { produced: true, shape: ValueShape::Boolean }
            }
            ">=" => {
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue { produced: true, shape: ValueShape::Boolean }
            }
```

Note: the pre-existing static-string relational fold (`operators.rs:382-390`) runs before operand emission and already returns for static ASCII string operands, so these arms only see numeric operands.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime relational_operators_compute_booleans`
Expected: PASS.

- [ ] **Step 5: Guard against regressions, format, lint, commit**

Run: `cargo fmt && cargo clippy -p kali_codegen && cargo test -p kali_codegen`
Then:

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): lower relational operators to i64 comparisons [feat]"
```

---

## Task 2: `return` value lowering

Today a function's `return expr` is mis-lowered: the `ReturnStmt` (a `text=None` Branch) flows into `emit_branch`, which consumes `expr` as an `if` condition and yields `0`. So every function returns `0`. Fix by tagging `ReturnStmt` with a `text` discriminator and adding an `emit_return`.

**Files:**
- Modify: `crates/kali_hir/src/lowering/statement.rs` (the `ReturnStatement` arm, `statement.rs:47-53`)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (the `Branch` arm at `control_flow.rs:163-174`; add `emit_return`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Produces: a `Branch` LIR node whose `text == Some("return")` with children `[arg?]`. `emit_return(function, node)` emits the argument (or `0`) then `Instruction::Return`, returning `EmittedValue { produced: false, shape: ValueShape::Unknown }` (control does not fall through; `emit_function_body` already appends a trailing `I64Const(0)` when the body did not "produce", keeping the wasm stack valid).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn functions_return_computed_values() {
    assert_eq!(run_js("function id(x) { return x; }\nconsole.log(id(42));\n"), "42\n");
    assert_eq!(
        run_js("function add(a, b) { return a + b; }\nconsole.log(add(40, 2));\n"),
        "42\n"
    );
    assert_eq!(
        run_js("function dbl(x) { return x * 2; }\nconsole.log(dbl(21));\n"),
        "42\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime functions_return_computed_values -- --nocapture`
Expected: FAIL — each prints `"0\n"`.

- [ ] **Step 3: Tag `ReturnStmt` at HIR**

In `crates/kali_hir/src/lowering/statement.rs`, the `ReturnStatement` arm currently calls `self.builder.alloc(HirNodeKind::ReturnStmt, None)`. Change the text from `None` to `Some("return")`:

```rust
Statement::ReturnStatement(ReturnStatement { argument }) => {
    let id = self.builder.alloc(HirNodeKind::ReturnStmt, Some("return".to_string()));
    if let Some(arg) = argument {
        push_child!(self, id, self.lower_expression(arg));
    }
    id
}
```

(Confirm the exact `alloc`/`alloc_text` signature in this file and match it; the text propagates verbatim through MIR/LIR to codegen.)

- [ ] **Step 4: Route `"return"` and add `emit_return`**

In `crates/kali_codegen/src/emit/control_flow.rs`, extend the `Branch` match arm (`control_flow.rs:163-174`) to handle the new discriminator:

```rust
            LirNodeKind::Branch => match node.text.as_deref() {
                Some(text) if text.starts_with("break") => {
                    self.emit_break_or_continue(function, false, &node)
                }
                Some(text) if text.starts_with("continue") => {
                    self.emit_break_or_continue(function, true, &node)
                }
                Some("for-of") | Some("for-await-of") => {
                    self.emit_for_of_array_iteration(function, &node)
                }
                Some("return") => self.emit_return(function, &node),
                _ => self.emit_branch(function, &node, want_value),
            },
```

Add the method (same `impl<'a> FunctionEmitter<'a>` block):

```rust
    pub(crate) fn emit_return(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        if let Some(arg) = node.children.first().copied() {
            let produced = self.emit_node(function, arg, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Return);
        EmittedValue { produced: false, shape: ValueShape::Unknown }
    }
```

Note: user functions are all typed to return one `i64` (`lower.rs:217-237`, `result: true`), so pushing a value before `Return` is always correct. `_start` (`result: false`) contains no `return` statement in supported programs, so this path is not exercised there.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime functions_return_computed_values`
Expected: PASS.

- [ ] **Step 6: Full suite, format, lint, commit**

Run: `cargo fmt && cargo clippy -p kali_hir -p kali_codegen && cargo test --workspace`
Expected: green (tagging is additive; `emit_branch` still handles `if`/loops as before this task). If any optimizer wasm-size assertion in `runtime_smoke.rs` shifts, update the evidence number (never an stdout string) and note it in the commit body.

```bash
git add crates/kali_hir/src/lowering/statement.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): lower return statements to real wasm returns [feat]"
```

---

## Task 3: Mutable local read/write correctness

Today `let x = 5; console.log(x)` prints `0` and `let x=5; x=x+1; console.log(x)` prints `0`, even though the locals plumbing (decl `LocalSet`, read `LocalGet`, identifier `emit_assignment`) appears present. This task localizes and fixes the defect so scalar mutable locals round-trip at both top level and inside functions.

**Files:**
- Investigate then modify the offending site among: `crates/kali_codegen/src/emit/control_flow.rs` (the `let`/`var` decl path `control_flow.rs:120-149`; identifier load `control_flow.rs:204-212`), `crates/kali_codegen/src/emit/literal.rs` (`emit_assignment`), `crates/kali_codegen/src/emit/call.rs` (the `console.log` argument path, `render_console_call` / `render_console_arguments`, and the user-call arg path), and/or `crates/kali_codegen/src/lower.rs:122-132` (`_start` local collection).
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `self.locals` (name → index), `Instruction::LocalGet/LocalSet/LocalTee`. Task 2's `return` must be merged first so the function-scope assertions are observable.
- Produces: reading a `let`/`var` binding emits `LocalGet(index)`; `name = expr` emits `expr` then `LocalTee(index)` (leaves the assigned value, so assignment-as-expression and assignment-as-statement both behave). No new public surface.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn mutable_locals_round_trip() {
    assert_eq!(run_js("let x = 5;\nconsole.log(x);\n"), "5\n");
    assert_eq!(run_js("let x = 5;\nx = x + 1;\nconsole.log(x);\n"), "6\n");
    assert_eq!(run_js("let x = 0;\nx = 9;\nconsole.log(x);\n"), "9\n");
    assert_eq!(
        run_js("function f() { let x = 5; x = x + 1; return x; }\nconsole.log(f());\n"),
        "6\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime mutable_locals_round_trip -- --nocapture`
Expected: FAIL — values print as `0`.

- [ ] **Step 3: Localize the defect (concrete investigation)**

Determine whether the value is never *stored* or never *read* by inspecting the emitted wasm. Build the `.wasm` and dump it:

```bash
TMP=$(mktemp -d); printf 'let x = 5;\nconsole.log(x);\n' > "$TMP/main.js"
cargo run -q -p kali_cli --bin kali -- build "$TMP/main.js" --out-dir "$TMP/out"
# Inspect with the wasmparser-based dumper used in tests, or add a throwaway #[test]
# in crates/kali_codegen that builds this source and prints the _start body instructions.
```

Diagnose against these specific suspects, in order:
1. **`_start` locals registration** — confirm `collect_function_locals(&lir.nodes, lir.root)` (`lower.rs:130`) returns `["x"]` and that `self.locals["x"]` exists for the entry emitter (`emitter.rs:90-96`). If top-level `let` is absent from `self.locals`, the decl path takes the `else { Drop }` branch (`control_flow.rs:140`) and the read falls to the `0` placeholder (`control_flow.rs:251-256`).
2. **`console.log` argument path** — in `crates/kali_codegen/src/emit/call.rs`, confirm that for `console.log(x)` with a non-static arg, `render_console_call(node)` returns `None` (so it does not statically render `x` to a constant) and the code emits `emit_node(first_arg)` → `LocalGet`. If `render_console_call` resolves `x` to a static `0`, restrict it to genuinely static (literal/`const`-bound) arguments.
3. **Index agreement** — confirm the `LocalSet` index in the decl path equals the `LocalGet` index in the read path for the same name (both go through `self.locals.get(&name)`).

- [ ] **Step 4: Apply the fix at the localized site**

Implement the minimal fix the investigation points to. The expected shape, by suspect:
- If (1): ensure top-level `let`/`var` names reach `self.locals` for `_start` (they should via `collect_function_locals(lir.root)`; if a wrapper node hides the declarator name, fix `collect_function_locals_from_node` in `lower.rs:696-730` to read the declarator `text`).
- If (2): in `call.rs`, make `render_console_call`/`render_console_arguments` return `None` for any argument that is a `let`/`var` identifier (i.e. present in `self.locals`), so the dynamic `emit_node` → `LocalGet` path runs.
- Additionally make `name = expr` assignment leave the value available: in `emit_assignment` (`literal.rs:173`) for the identifier-target `=` case, emit the RHS then `Instruction::LocalTee(index)` (not bare `LocalSet`), and return `true` with the value produced — so `x = 9` as a statement stores 9 and `console.log(x)` later reads 9.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime mutable_locals_round_trip`
Expected: PASS (all four cases).

- [ ] **Step 6: Full suite, format, lint, commit**

Run: `cargo fmt && cargo clippy --workspace && cargo test --workspace`
Expected: green; no existing stdout expectation changes (constant programs still fold). 

```bash
git add crates/kali_codegen/src crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "fix(codegen): make mutable local read/write round-trip at runtime [fix]"
```

---

## Task 4: Real `while` / `for` / `do-while` loops

Today these are one-shot `if`s (no wasm back-edge). Tag them at HIR and add `emit_loop` that emits a real `block { loop { ... br } }` with break/continue frames. This is the foundational fix; it also makes recursion's base cases reachable (combined with Task 1's relational ops) since the inner loops finally terminate.

**Files:**
- Modify: `crates/kali_hir/src/lowering/statement.rs` (the `WhileStatement` `statement.rs:227-236`, `DoWhileStatement` `statement.rs:237-246`, `ForStatement` `statement.rs:159-188` arms)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (`Branch` arm; add `emit_loop`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Produces: `Branch` nodes with `text == Some("while")` (children `[test, body]`), `Some("do-while")` (children `[body, test]`), `Some("for")` (children `[init?, test?, update?, body]`). `emit_loop` consumes Task 1 relational ops (conditions), Task 3 locals (counters), and the existing `emit_break_or_continue` + `LoopFrame` machinery.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn loops_iterate() {
    // while with a counter and accumulator
    assert_eq!(
        run_js("let s = 0;\nlet i = 0;\nwhile (i < 5) { s = s + i; i = i + 1; }\nconsole.log(s);\n"),
        "10\n"
    );
    // for loop
    assert_eq!(
        run_js("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\nconsole.log(s);\n"),
        "10\n"
    );
    // break out of while(true)
    assert_eq!(
        run_js("let i = 0;\nwhile (true) { if (i >= 3) { break; } i = i + 1; }\nconsole.log(i);\n"),
        "3\n"
    );
    // do-while runs body first
    assert_eq!(
        run_js("let i = 0;\nlet n = 0;\ndo { n = n + 1; i = i + 1; } while (i < 4);\nconsole.log(n);\n"),
        "4\n"
    );
    // recursion now terminates (relational base case + real calls)
    assert_eq!(
        run_js("function s(n) { if (n < 1) { return 0; } return n + s(n - 1); }\nconsole.log(s(5));\n"),
        "15\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime loops_iterate -- --nocapture`
Expected: FAIL — loops run their body at most once (e.g. first case prints `0`); the `break` case errors `E5506` (no loop frame) or traps.

- [ ] **Step 3: Tag the loop statements at HIR**

In `crates/kali_hir/src/lowering/statement.rs`, give each loop node a discriminator text (mirror the existing `for-of` tagging which uses `alloc_text`). Match the file's existing `alloc`/`alloc_text` signatures; set:
- `WhileStmt` → text `"while"` (keep children `[test, body]`).
- `DoWhileStmt` → text `"do-while"` (keep children `[body, test]`).
- `ForStmt` → text `"for"` (keep children `[init?, test?, update?, body]`).

Leave `IfStmt` untagged (`text=None`) so it still routes to `emit_branch`.

- [ ] **Step 4: Route the loop discriminators and add `emit_loop`**

In `crates/kali_codegen/src/emit/control_flow.rs`, add arms to the `Branch` match (after the `"return"` arm from Task 2):

```rust
                Some("while") | Some("do-while") | Some("for") => self.emit_loop(function, &node),
```

Add `emit_loop`, mirroring the frame discipline of `emit_for_of_array_iteration` (`crates/kali_codegen/src/intrinsics/array.rs:1116`). It builds `block` (break target) → `loop` (continue target), tests the condition each iteration, exits via `BrIf` to the block on a falsy condition, emits the body, then `Br` back to the loop:

```rust
    pub(crate) fn emit_loop(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let kind = node.text.as_deref().unwrap_or_default();

        // Resolve clauses by loop kind.
        let (init, test, update, body) = match kind {
            "while" => (None, node.children.first().copied(), None, node.children.get(1).copied()),
            "do-while" => (None, node.children.get(1).copied(), None, node.children.first().copied()),
            _ /* "for" */ => {
                // [init?, test?, update?, body] — body is always last; classify by count.
                let n = node.children.len();
                let body = node.children.last().copied();
                let (init, test, update) = match n {
                    1 => (None, None, None),
                    2 => (None, node.children.first().copied(), None),
                    3 => (node.children.first().copied(), node.children.get(1).copied(), None),
                    _ => (node.children.first().copied(), node.children.get(1).copied(), node.children.get(2).copied()),
                };
                (init, test, update, body)
            }
        };

        // for-init runs once, before the loop.
        if let Some(init) = init {
            let produced = self.emit_node(function, init, false);
            if produced.produced { function.instruction(&Instruction::Drop); }
        }

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame { break_index, continue_index });

        let emit_body_and_update = |emitter: &mut Self, function: &mut Function| {
            if let Some(body) = body {
                let produced = emitter.emit_node(function, body, false);
                if produced.produced { function.instruction(&Instruction::Drop); }
            }
            if let Some(update) = update {
                let produced = emitter.emit_node(function, update, false);
                if produced.produced { function.instruction(&Instruction::Drop); }
            }
        };

        if kind == "do-while" {
            // body first, then test at the bottom.
            emit_body_and_update(self, function);
            if let Some(test) = test {
                let cond = self.emit_node(function, test, true);
                if !cond.produced { function.instruction(&Instruction::I64Const(0)); }
            } else {
                function.instruction(&Instruction::I64Const(1));
            }
            function.instruction(&Instruction::I64Eqz);   // 1 if falsy
            function.instruction(&Instruction::I32Eqz);    // invert: 1 if truthy
            function.instruction(&Instruction::BrIf(0));    // continue if truthy
        } else {
            // test at the top; exit (break) when falsy.
            if let Some(test) = test {
                let cond = self.emit_node(function, test, true);
                if !cond.produced { function.instruction(&Instruction::I64Const(0)); }
                function.instruction(&Instruction::I64Eqz);   // 1 if falsy
                function.instruction(&Instruction::BrIf(1));    // break out of `block` when falsy
            }
            emit_body_and_update(self, function);
            function.instruction(&Instruction::Br(0));          // back to loop top
        }

        function.instruction(&Instruction::End); // end loop
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

        EmittedValue { produced: false, shape: ValueShape::Unknown }
    }
```

Notes for the implementer:
- Confirm the exact `ControlFlowLabelKind` variant names in `emitter.rs:32` (use the same ones `emit_for_of_array_iteration` uses for break vs continue).
- The condition's truthiness uses the same normalization as `emit_branch` (`I64Eqz` → "is falsy"); the existing `condition.shape == Boolean` path in `emit_branch` is a micro-optimization you may replicate but is not required for correctness (a boolean is already 0/1).
- `continue` jumps to `continue_index` (the `loop` frame). For `while`/`do-while` that re-tests correctly. For `for`, a `continue` re-enters the loop *without* running `update` (update is emitted at the body tail), which would be incorrect — fannkuch uses no `continue` in any `for`, so this is acceptable for this slice; record the limitation in the Task 8 maturity note.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime loops_iterate`
Expected: PASS (all five cases, including recursion).

- [ ] **Step 6: Full suite, format, lint, commit**

Run: `cargo fmt && cargo clippy --workspace && cargo test --workspace`
Expected: green. The `for-of` unrolled path is untouched (different discriminators). Update any shifted optimizer size-evidence numbers in `runtime_smoke.rs` if needed (never stdout strings).

```bash
git add crates/kali_hir/src/lowering/statement.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): lower while/for/do-while to real wasm loops [feat]"
```

---

## Task 5: Linear-memory integer arrays

Add `new Array(n)`, `a[i]` read, and `a[i] = v` write, backed by a bump-allocated region of the exported linear memory. fannkuch uses `new Array(n)` plus indexed read/write (and never `.length` or `push`, so those stay out of scope).

**Files:**
- Modify: `crates/kali_codegen/src/lib.rs` (import `GlobalSection`, `GlobalType`, `ConstExpr` if not already; the section list already imports `MemorySection` etc.)
- Modify: `crates/kali_codegen/src/lower.rs` (add a `GlobalSection` with the `__heap` bump pointer; raise memory `minimum` pages; compute the heap base from `string_pool.next_offset`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (recognize `new Array(n)` — today it reaches the `E3100 undefined call target 'Array'` fallback — and emit allocation; extend the dynamic `a[i]` read path)
- Modify: `crates/kali_codegen/src/emit/literal.rs` (`emit_assignment`: handle array-element target `a[i] = v`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Produces:
  - `__heap`: a mutable wasm global (i32), global index `0`, initialized to `heap_base` (= `string_pool.next_offset` rounded up to 8). Exported as `"__heap"` (so Task 6's host helpers can share the same bump cursor).
  - An array handle is the i32 base offset (zero-extended to i64) of a block laid out as `[ length: i64 @ +0 ][ elem0: i64 @ +8 ][ elem1 @ +16 ]…`. `new Array(n)` allocates `(n + 1) * 8` bytes, stores `n` at `+0`, returns the base.
  - Element access address = `base_i32 + index_i32 * 8`, with the wasm memory `offset` immediate set to `8` to skip the length header.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn integer_arrays_read_write() {
    assert_eq!(
        run_js("const a = new Array(3);\na[0] = 10;\na[1] = 20;\na[2] = a[0] + a[1];\nconsole.log(a[2]);\n"),
        "30\n"
    );
    // dynamic index from a loop variable
    assert_eq!(
        run_js("const a = new Array(5);\nfor (let i = 0; i < 5; i = i + 1) { a[i] = i * i; }\nlet s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + a[i]; }\nconsole.log(s);\n"),
        "30\n"
    );
    // swap via a temp (the fannkuch inner idiom)
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 7;\na[1] = 9;\nconst t = a[0];\na[0] = a[1];\na[1] = t;\nconsole.log(a[0]);\nconsole.log(a[1]);\n"),
        "9\n7\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime integer_arrays_read_write -- --nocapture`
Expected: FAIL — `new Array` warns `E3100` and yields `0`; reads/writes warn `E8001`.

- [ ] **Step 3: Add the `__heap` global and grow memory**

In `crates/kali_codegen/src/lower.rs`, after the string pool is finalized (so `string_pool.next_offset` is known) and where sections are assembled (`lower.rs:244-321`):

```rust
// Heap base: first 8-aligned byte after interned string data.
let heap_base = (string_pool.next_offset + 7) & !7;

let mut global_section = GlobalSection::new();
global_section.global(
    GlobalType { val_type: ValType::I32, mutable: true, shared: false },
    &ConstExpr::i32_const(heap_base as i32),
);
```

Raise the memory floor so a real workload fits (n=7 needs only a few hundred bytes after the 4 KiB reserved prefix + string data, but give headroom):

```rust
memory_section.memory(MemoryType { minimum: 16, maximum: None, memory64: false, shared: false, page_size_log2: None });
```

Add the global section to the module in the standard order (globals come after memory, before exports per the wasm section order) and export the global:

```rust
module.section(&global_section);            // after memory_section, before export_section
export_section.export("__heap", ExportKind::Global, 0);
```

Import `GlobalSection`, `GlobalType`, `ConstExpr`, `ExportKind` in `crates/kali_codegen/src/lib.rs` if not already present.

- [ ] **Step 4: Emit `new Array(n)` allocation**

In `crates/kali_codegen/src/emit/call.rs`, before the generic "undefined call target" fallback, recognize a call whose callee identifier is `Array` constructed via `new` (confirm how `new X()` is shaped in LIR — a `Call` node whose callee text is `Array`; check `is_supported_callable_reference`/the new-expression lowering). Emit (using the per-function scratch local at index `params.len() + locals.len()` — the `+1` slot from `lower.rs:269` — to hold the base):

```text
;; stack: emit the size argument (i64 n)
<emit node.children[1] as i64 n>
i64.const 1
i64.add
i64.const 8
i64.mul
i32.wrap_i64            ;; nbytes (i32)
;; base = __heap ; __heap += nbytes
global.get 0            ;; __heap (base)
local.tee  <scratch>    ;; save base in scratch i32? note locals are i64 — see below
...
```

Implementation note: the function locals are all `i64`. Keep the base as an i64 (a small positive offset) and convert to i32 only at memory-access time with `I32WrapI64`. Concretely:

```rust
// nbytes_i32 on stack from (n+1)*8 as above, then:
function.instruction(&Instruction::GlobalGet(0));          // __heap (i32)
function.instruction(&Instruction::I64ExtendI32U);          // base as i64
// store length n at base+0: need base again as i32; recompute from saved base.
```

Recommended concrete sequence (store length, advance heap, leave base as the i64 result), using one i64 scratch local `base_local` (allocate it like `??` does at `operators.rs:476`, or reserve a dedicated slot):

```rust
// 1. compute nbytes (i32) on stack as (n+1)*8  -> I32WrapI64 at the end
// 2. base:
function.instruction(&Instruction::GlobalGet(0));   // i32 base
function.instruction(&Instruction::LocalSet(base_local_i32_or_via_wrap)); // save
// 3. advance: __heap = base + nbytes
function.instruction(&Instruction::GlobalGet(0));
//   (nbytes already consumed? recompute or dup) -> simplest: compute nbytes into a local first
function.instruction(&Instruction::GlobalSet(0));
// 4. store length: addr=base (i32), value=n (i64)
//    I64Store { offset: 0, align: 3, memory_index: 0 }
// 5. push base as the array handle (i64)
```

Because juggling i32/i64 and scratch locals inline is error-prone, the cleaner option (recommended) is to **synthesize a `kali_alloc(nbytes: i64) -> i64` helper function** as an extra `FunctionPlan` in `all_functions` (`lower.rs:122-132`) with body emitted directly as wasm: `__heap` base → extend → advance global → return. Then `new Array(n)` becomes: emit `(n+1)*8`, `Call(kali_alloc_index)`, then store the length via the array-write primitive from Step 6. Pick whichever keeps the code clearest; the test in Step 1 is the contract.

- [ ] **Step 5: Emit `a[i] = v` (array-element assignment)**

In `crates/kali_codegen/src/emit/literal.rs`, in `emit_assignment`, before the `assignment_target_name` path that currently returns `false` for non-identifier `=`, detect an index-member target (`a[i]`). Reuse the same node-shape recognition that `resolve_static_index_member` (`crates/kali_codegen/src/emit/call.rs:2164`) uses to pull the base expression and the index expression from the member node, but take the **dynamic** path. Emit:

```text
<emit base as i64>      ;; array handle
i32.wrap_i64            ;; base addr (i32)
<emit index as i64>
i32.wrap_i64
i32.const 8
i32.mul
i32.add                 ;; addr = base + index*8
<emit value as i64>
i64.store offset=8 align=3 memory=0   ;; skip 8-byte length header
```

Return `true` (handled). As a statement the assignment produces no value; if you want assignment-as-expression parity, also push the value via a scratch local + `LocalTee` before the store, but fannkuch only uses element assignment as a statement.

- [ ] **Step 6: Emit `a[i]` (array-element read)**

In `crates/kali_codegen/src/emit/call.rs` (or wherever the dynamic member read currently falls through to `emit_unary`'s `E8001`), add the dynamic index-read path mirroring `resolve_static_index_member`'s base/index extraction but emitting a load:

```text
<emit base as i64>
i32.wrap_i64
<emit index as i64>
i32.wrap_i64
i32.const 8
i32.mul
i32.add
i64.load offset=8 align=3 memory=0
```

Produce `EmittedValue { produced: true, shape: ValueShape::Scalar }`.

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime integer_arrays_read_write`
Expected: PASS (all three cases).

- [ ] **Step 8: Full suite, format, lint, commit**

Run: `cargo fmt && cargo clippy --workspace && cargo test --workspace`
Expected: green. The static array-fold paths (literal arrays in constant programs) are untouched; only the previously-erroring dynamic `new Array`/index paths change.

```bash
git add crates/kali_codegen/src crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): linear-memory integer arrays (new Array, indexed read/write) [feat]"
```

---

## Task 6: Runtime integer→string and string concatenation

The fannkuch line `"Pfannkuchen(" + n + ") = " + maxFlipsCount` needs an integer coerced to a decimal string and string concatenation at runtime. Implement these as two `kali:rt` host imports that allocate in the exported linear memory via the shared `__heap` global, and wire string-typed `+` in codegen to call them.

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (register two new imports)
- Modify: `crates/kali_runtime/src/host/memory.rs` (helpers to read the `__heap` global, write bytes, advance it; return a tagged handle)
- Modify: `crates/kali_codegen/src/lib.rs` and `crates/kali_codegen/src/lower.rs` (add the two import indices; keep `FUNCTION_INDEX_OFFSET` and downstream user-function indices consistent)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (string-typed `+` → concat; numeric operand of a string `+` → `int_to_string`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Produces two `"kali:rt"` imports:
  - `int_to_string(value: i64) -> i64` (string handle): host formats `value` as decimal ASCII, allocates `len` bytes at the current `__heap`, writes the bytes, advances `__heap`, returns `STRING_HANDLE_TAG | (offset << 32) | len`.
  - `string_concat(a: i64, b: i64) -> i64` (string handle): host decodes both handles (offset/len from each, reading from guest memory; a non-tagged i64 is treated as already-decimal via `int_to_string` semantics — but codegen guarantees both args are string handles, see below), allocates `lenA + lenB`, copies both, returns the new handle.
- Consumes: the exported `memory` and the exported `__heap` global (Task 5). Host accesses them via `caller.get_export("memory")` and `caller.get_export("__heap")` (mirror `crates/kali_runtime/src/host/memory.rs:92-94`).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn runtime_string_building() {
    assert_eq!(run_js("let n = 7;\nconsole.log(\"n=\" + n);\n"), "n=7\n");
    assert_eq!(
        run_js("let n = 7;\nlet m = 16;\nconsole.log(\"Pfannkuchen(\" + n + \") = \" + m);\n"),
        "Pfannkuchen(7) = 16\n"
    );
    // concatenation of a computed integer
    assert_eq!(run_js("let x = 20;\nconsole.log(\"v=\" + (x + 1));\n"), "v=21\n");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime runtime_string_building -- --nocapture`
Expected: FAIL — `+` with a string operand currently emits `I64Add` on a string handle and an int, producing garbage.

- [ ] **Step 3: Add the host imports**

In `crates/kali_runtime/src/host/memory.rs`, add helpers (follow `read_guest_bytes`/`guest_memory` at `memory.rs:43-94`):

```rust
pub(crate) fn heap_global(caller: &mut Caller<'_, KaliHostState>) -> wasmtime::Result<wasmtime::Global> {
    match caller.get_export("__heap") {
        Some(Extern::Global(g)) => Ok(g),
        _ => wasmtime::bail!("missing __heap global export"),
    }
}

/// Allocate `bytes.len()` at the current __heap, write them, advance __heap,
/// return a tagged string handle.
pub(crate) fn alloc_guest_string(caller: &mut Caller<'_, KaliHostState>, bytes: &[u8]) -> wasmtime::Result<i64> {
    let g = heap_global(caller)?;
    let base = g.get(&mut *caller).i32().ok_or_else(|| wasmtime::Error::msg("__heap not i32"))?;
    let mem = guest_memory(caller)?;
    mem.write(&mut *caller, base as usize, bytes)?;
    g.set(&mut *caller, wasmtime::Val::I32(base + bytes.len() as i32))?;
    Ok((STRING_HANDLE_TAG | ((base as u64) << 32) | (bytes.len() as u64)) as i64)
}
```

In `crates/kali_runtime/src/host/imports_default.rs`, register (mirror the existing `console_*`/`math_*` `func_wrap` registrations):

```rust
linker.func_wrap("kali:rt", "int_to_string", |mut caller: Caller<'_, KaliHostState>, value: i64| -> i64 {
    let text = value.to_string();
    alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
})?;
linker.func_wrap("kali:rt", "string_concat", |mut caller: Caller<'_, KaliHostState>, a: i64, b: i64| -> i64 {
    let mut bytes = decode_handle_bytes(&mut caller, a).unwrap_or_default();
    bytes.extend(decode_handle_bytes(&mut caller, b).unwrap_or_default());
    alloc_guest_string(&mut caller, &bytes).unwrap_or(0)
})?;
```

where `decode_handle_bytes` reads a tagged string handle's bytes from guest memory (reuse the offset/len unpacking from `format_console_value` in `crates/kali_runtime/src/host/io.rs:22-35`, factored into a shared helper).

- [ ] **Step 4: Allocate the import indices in codegen**

In `crates/kali_codegen/src/lib.rs`, add index constants for the two new imports and bump `FUNCTION_INDEX_OFFSET` accordingly (it currently `= 17`; with two more imports it becomes `19`). In `crates/kali_codegen/src/lower.rs:158-216`, register the two imports in the import section at their indices, and store their resolved indices on the emitter (like `env_set_import_index`) so the operator code can `Call` them. Re-run the full suite after this step alone to confirm index bookkeeping didn't shift user-function calls.

- [ ] **Step 5: Wire string `+` in codegen**

In `crates/kali_codegen/src/emit/operators.rs` `emit_binary`, before the numeric `match`, add a string-concat path. Add a helper `fn is_string_valued(&self, id: LirNodeId) -> bool` that returns true for: a string literal node, a template-literal node, or a `+` binary whose either operand `is_string_valued`. Then:

```rust
        if op == "+" && (self.is_string_valued(left) || self.is_string_valued(right)) {
            self.emit_as_string(function, left);
            self.emit_as_string(function, right);
            function.instruction(&Instruction::Call(self.string_concat_index));
            return EmittedValue { produced: true, shape: ValueShape::String };
        }
```

where `emit_as_string(id)` emits the node, and if it is not `is_string_valued`, follows it with `Call(int_to_string)` to coerce the i64 to a decimal-string handle:

```rust
    fn emit_as_string(&mut self, function: &mut Function, id: LirNodeId) {
        let produced = self.emit_node(function, id, true);
        if !produced.produced { function.instruction(&Instruction::I64Const(0)); }
        if !self.is_string_valued(id) {
            function.instruction(&Instruction::Call(self.int_to_string_index));
        }
    }
```

Add a `ValueShape::String` variant (or reuse `Scalar` if adding a variant is invasive — but a distinct `String` shape lets `console.log` and nested `+` treat handles correctly; check `ValueShape` usages first). `console.log` already prints a string handle correctly via the host (`format_console_value`).

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p kali_cli --test imperative_core_runtime runtime_string_building`
Expected: PASS (all three cases).

- [ ] **Step 7: Full suite, format, lint, commit**

Run: `cargo fmt && cargo clippy --workspace && cargo test --workspace`
Expected: green. Static string folds (constant concatenations) are unaffected — `is_string_valued` + runtime concat only fires when operands reach codegen unfolded.

```bash
git add crates/kali_codegen/src crates/kali_runtime/src crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat: runtime int-to-string and string concatenation via kali:rt host helpers [feat]"
```

---

## Task 7: Vendor fannkuch-redux and assert end-to-end

Add the benchmark program and a dedicated end-to-end test that runs it and asserts the exact canonical output. The optimizer-footprint harness (`assert_optimization_benchmark_fixture`) asserts size *reductions* that do not hold for a real loop program, so this task uses a dedicated run-test, plus a metadata/sha validation test.

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.json`
- Create: `crates/kali_cli/tests/clbg_fannkuch_runtime.rs`

**Interfaces:**
- Consumes: every prior task (loops, locals, returns, relational ops, arrays, string building).
- Produces: a runnable CLBG fixture; expected stdout for n=7 is exactly `"228\nPfannkuchen(7) = 16\n"` (verified against Node.js: checksum 228, max flips 16 — the official CLBG values for n=7).

- [ ] **Step 1: Write the fixture program**

Create `crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts` (annotation-free; verified to produce the canonical output and to use only in-scope features — integer arithmetic, `< > === !==`, `if/else`, `while`/`for`, `break`, `new Array`, indexed read/write, function call, `console.log`, string `+`):

```ts
// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// fannkuch-redux — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (integer-only, no intrinsic tuning).
// Retains upstream attribution per the CLBG license terms.
function fannkuch(n) {
  const perm = new Array(n);
  const perm1 = new Array(n);
  const count = new Array(n);
  for (let i = 0; i < n; i = i + 1) {
    perm1[i] = i;
  }
  let maxFlipsCount = 0;
  let permCount = 0;
  let checksum = 0;
  let r = n;
  while (true) {
    while (r !== 1) {
      count[r - 1] = r;
      r = r - 1;
    }
    for (let i = 0; i < n; i = i + 1) {
      perm[i] = perm1[i];
    }
    let flipsCount = 0;
    let k = perm[0];
    while (k !== 0) {
      let i = 0;
      let j = k;
      while (i < j) {
        const temp = perm[i];
        perm[i] = perm[j];
        perm[j] = temp;
        i = i + 1;
        j = j - 1;
      }
      flipsCount = flipsCount + 1;
      k = perm[0];
    }
    if (flipsCount > maxFlipsCount) {
      maxFlipsCount = flipsCount;
    }
    if (permCount % 2 === 0) {
      checksum = checksum + flipsCount;
    } else {
      checksum = checksum - flipsCount;
    }
    let done = false;
    while (true) {
      if (r === n) {
        done = true;
        break;
      }
      const perm0 = perm1[0];
      let i = 0;
      while (i < r) {
        perm1[i] = perm1[i + 1];
        i = i + 1;
      }
      perm1[r] = perm0;
      count[r] = count[r] - 1;
      if (count[r] > 0) {
        break;
      }
      r = r + 1;
    }
    if (done) {
      break;
    }
    permCount = permCount + 1;
  }
  console.log(checksum);
  console.log("Pfannkuchen(" + n + ") = " + maxFlipsCount);
}
fannkuch(7);
```

- [ ] **Step 2: Compute the sha256 and write the metadata**

Run:

```bash
shasum -a 256 crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts
```

Create `crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.json` (schema `schemas/benchmark/v1.json`), substituting the hash:

```json
{
  "benchmark": "fannkuch-redux",
  "version": 1,
  "sourceFile": "fannkuch-redux-benchmark-v1.ts",
  "sourceSha256": "sha256-<paste the 64-hex-char digest here>",
  "buildModes": ["--fast", "--release", "--release-advanced"]
}
```

- [ ] **Step 3: Write the end-to-end + metadata test**

Create `crates/kali_cli/tests/clbg_fannkuch_runtime.rs`:

```rust
use std::{fs, path::PathBuf, process::Command};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/benchmarks").join(name)
}

#[test]
fn fannkuch_redux_runs_and_matches_canonical_output() {
    let source = fixture("fannkuch-redux-benchmark-v1.ts");
    let output = Command::new(kali_bin()).arg("run").arg(&source).output().expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "228\nPfannkuchen(7) = 16\n");
}

#[test]
fn fannkuch_redux_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fannkuch-redux-benchmark-v1.json")).expect("read metadata"),
    ).expect("parse metadata");
    assert_eq!(meta["benchmark"], "fannkuch-redux");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "fannkuch-redux-benchmark-v1.ts");
    assert_eq!(meta["buildModes"], serde_json::json!(["--fast", "--release", "--release-advanced"]));
    let src = fs::read(fixture("fannkuch-redux-benchmark-v1.ts")).expect("read source");
    let digest = format!("sha256-{:x}", Sha256::digest(&src));
    assert_eq!(meta["sourceSha256"], digest, "metadata sha256 must match the source file");
}
```

(Confirm `sha2` and `serde_json` are available as dev-dependencies of `kali_cli`; they are already used by `crates/kali_cli/tests/runtime_smoke.rs`.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test clbg_fannkuch_runtime`
Expected: PASS — both tests. If `fannkuch_redux_runs_and_matches_canonical_output` fails, the stderr in the panic message names the first unsupported construct; fix the responsible earlier task before proceeding.

- [ ] **Step 5: Sanity-check all three build modes compile**

Run:

```bash
for m in --fast --release --release-advanced; do
  cargo run -q -p kali_cli --bin kali -- build $m crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts --out-dir "$(mktemp -d)" || echo "FAILED: $m";
done
```

Expected: all three build without error.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.json crates/kali_cli/tests/clbg_fannkuch_runtime.rs
git commit -m "test(clbg): vendor fannkuch-redux fixture and assert canonical output [test]"
```

---

## Task 8: Honest maturity-doc updates and final verification

Record the newly-supported integer imperative slice in the availability matrix, narrowly and without over-claiming, and verify the whole workspace.

**Files:**
- Modify: `specs/19-feature-maturity.md` (add rows; do NOT touch `proofs/BOUNDARY.md`)

- [ ] **Step 1: Add narrow, accurate maturity rows**

In `specs/19-feature-maturity.md`'s Canonical Matrix, add rows describing exactly what now executes, each scoped to the integer slice. Use this wording (adjust to match the table's column format):

- `Runtime iterative loops (while / for / do-while over i64 conditions, with unlabeled break/continue; for-continue does not re-run the update clause yet)` — Phase 1 MVP — "Real wasm `loop`/back-edge lowering; integer conditions and counters; demonstrated by the fannkuch-redux end-to-end fixture."
- `Runtime mutable local variables (let/var read, assignment, compound assignment) over i64` — Phase 1 MVP.
- `User function calls returning computed i64 values, including self-recursion` — Phase 1 MVP.
- `Relational operators < <= > >= on i64 operands` — Phase 1 MVP.
- `Linear-memory integer arrays (new Array(n), indexed read/write; bump-allocated, no free, no .length/push yet)` — Phase 1 MVP — "Backed by the exported `__heap` bump global; consistent with the no-tracing-GC model."
- `Runtime integer→decimal string and string concatenation (kali:rt int_to_string / string_concat)` — Phase 1 MVP.
- Under the existing "adapted Computer Language Benchmarks Game workloads" lane (`19-feature-maturity.md:215`), note that `fannkuch-redux` is the first vendored, end-to-end-executing CLBG fixture; keep all throughput/performance wording out (per `plan/phase-24/README.md` §24.4).

Keep each row's claim limited to integers; explicitly do not imply f64, growable arrays, general objects, stdin, or regex.

- [ ] **Step 2: Run the dangling-link / consistency check**

If the repo's link checker is available, run it; otherwise verify any new cross-references resolve. Confirm no `R##` audit IDs or placeholders were introduced.

- [ ] **Step 3: Final full-workspace verification**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets && cargo test --workspace`
Expected: all green. In particular `imperative_core_runtime`, `clbg_fannkuch_runtime`, and the full existing suite pass.

- [ ] **Step 4: Commit**

```bash
git add specs/19-feature-maturity.md
git commit -m "docs(spec): record integer imperative-core execution slice + first CLBG fixture [spec]"
```

---

## Self-Review Notes (for the executor)

- **Spec coverage:** Task 1 = relational ops; Tasks 2+3 = working functions/locals (design pieces B/C); Task 4 = loops (piece A); Task 5 = memory arrays (piece D); Task 6 = int→string/concat (piece E); Task 7 = vendored port + exact-output acceptance; Task 8 = honest maturity updates + risk-section verification. All design acceptance criteria map to a task.
- **Dependency order:** 1 (relational) and 2 (return) are independent; 3 (locals) depends on 2 for its function-scope assertion; 4 (loops) depends on 1+3; 5 (arrays) depends on 3+4; 6 (strings) depends on 5's `__heap` global; 7 depends on all; 8 last. Execute in numeric order.
- **Type/name consistency:** the `__heap` global is index `0`, exported as `"__heap"`, shared by Task 5 (guest allocation) and Task 6 (host `alloc_guest_string`). Array layout `[len@+0][elems@+8…]` is identical in Task 5 Steps 4/5/6. `ValueShape::String` (Task 6) must be added to the enum if used. Import-index bumping (Task 6 Step 4) must keep `FUNCTION_INDEX_OFFSET` and the user-function index map (`lower.rs:122-136`) consistent — verify with a full-suite run after that step alone.
- **Investigative step:** Task 3 Step 3 is a genuine localization (the reads-as-0 cause was not statically decidable); its three concrete suspects and the wasm-dump method are specified so it is debuggable, not hand-wavy.

---

## Deferred Follow-ups (post-implementation, recorded 2026-07-01)

This slice shipped and merged to local `main`. During execution + adversarial review, a number of
follow-ups were surfaced. The ones below were **consciously deferred** — none is reachable-and-harmful
within the supported integer slice (each is unreachable in-slice, a documented limitation, an
intentional design choice, or a large pre-existing concern out of this slice's scope). They are logged
here as actionable backlog; if picked up, do them as their own TDD + review tasks.

For contrast, these related follow-ups were **already fixed** and are on `main`: the `cargo fmt`
workspace normalization (commit `8a0a18d3`); the silent `+` string-typed-variable miscompile, now a
checker `E3200` rejection (commits `db1463d5`, `2e28aaa3`, `0e9c430e`); extra do-while/for
constant-condition loop tests and while continue/break coverage; and the operator-named-key member
disambiguation doc note (all in commit `f3f53b47`). The `for`-`continue`-skips-update limitation and
the `E3200` string-`+` behavior are recorded in `specs/19-feature-maturity.md`.

| ID | Item | Reachability / kind | Why deferred | If picked up |
|---|---|---|---|---|
| F-1 | `a = new Array(n)` **reassignment** is not recognized as an allocation (only the `let`/`const` declarator-init position is). | Feature gap; no current consumer. | Not needed by fannkuch or claimed anywhere; declarator-init covers the supported slice. | Recognize `new Array(n)` in the assignment-RHS path (`emit_assignment`, `crates/kali_codegen/src/emit/literal.rs`) the same way the declarator-init path does. |
| F-2 | `array_bindings` are **flat, function-scoped by name** (no block scoping); a same-named non-array in a nested block could misroute. | Pre-existing systemic emitter pattern (locals/bindings are already flat maps); not exercised by current tests. | Large pre-existing refactor unrelated to this slice; fannkuch's block-local `const`s re-init per iteration and round-trip correctly. | Give the emitter block-scoped binding maps (affects `self.locals`/`self.bindings`/`array_bindings` together), not just arrays. |
| F-3 | Array-alloc keeps `base` in the **generic shared scratch local** (`self.locals.len()`); a size arg that itself claims that scratch (e.g. `new Array(x++)`) could clobber the saved base. | Unreachable today — no side-effecting size expression that also uses the scratch is expressible for `new Array(...)`. | Systemic single-shared-scratch hazard; not triggerable in the current feature set. | Reserve a dedicated (non-shared) scratch local for the array base, or a small scratch allocator, if size expressions gain side effects. |
| F-4 | `is_binary_operator_text` **misroutes an operator-named string-literal member key** (`obj["+"]`, `obj["in"]`) to `emit_binary`. | Unreachable in the i64 slice (no general object with operator-named string keys is expressible/evaluable). | Documented at `crates/kali_codegen/src/lower.rs` (doc comment on `is_binary_operator_text`). | Disambiguate computed-member vs binary on **node kind** rather than on `text`, once a richer object model exists. |
| F-5 | `ValueShape::String` is currently **inert** (treated identically to `Scalar`/`Unknown` at consumption sites). | Not a bug — intentional forward-looking variant added in Task 6. | Correct for the slice (`console.log` decodes the handle tag; truthiness routes fine); nothing to fix. | Use the distinct shape when nested `+`/`console.log`/future string ops need to branch on string-ness. |
| F-6 | Runtime **string heap allocations are unaligned** (`__heap` advances by exact byte length, no padding). | Wasm permits unaligned loads; correctness-neutral. Arrays stay 8-aligned (allocated before any string in fannkuch). | Perf-only; no correctness impact. | Round string allocations up to 8-byte alignment in `alloc_guest_string` (`crates/kali_runtime/src/host/memory.rs`) and set the load/store align hints. |
| F-7 | The one **known pre-existing clippy warning**: unused import `profile_data_hash` in `crates/kali_cli/src/build/mod.rs`. | Pre-existing (present on `main` before this slice); the sole workspace clippy warning. | Out of this slice's scope; removing a `pub(crate)` re-export could drop an intended API surface — needs an owner's call. | Confirm no intended consumer, then drop the re-export (or `#[allow]` with a reason). |

Residual acknowledged elsewhere (not in this table): an **untyped string function parameter** in a
`+` (`function f(s){ return s + "x" }`) is undecidable and stays out of scope — the typed-parameter
form is already blocked at resolution (`E3100`). See the `E3200` string-`+` row in
`specs/19-feature-maturity.md`.
