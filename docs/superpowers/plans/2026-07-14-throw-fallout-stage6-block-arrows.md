# throw-fallout Stage 6 — Block-Arrow Un-flattening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop the parser from destroying block-bodied arrows and executing their bodies inline — make a callback passed to a function a real, compiled, *actually-invoked* function.

**Architecture:** Three moves, in a forced order. (1) Name every anonymous function-shaped node in an **AST pre-pass** before the resolver, so `kali_types` and HIR key on the *same* name. (2) Make arrow / function-expression / class-method bodies **repr-tracked** by pushing them onto `current_function`, which is what makes repr proofs inside callback bodies stop failing closed. (3) Apply the existing parse patch **and wire the callback consumers**, so an un-flattened callback is invoked rather than silently dropped.

**Tech Stack:** Rust (kali_parser / kali_ast / kali_types / kali_hir / kali_codegen / kali_cli), wasm, node (differential oracle).

**Spec:** `docs/superpowers/specs/2026-07-14-throw-fallout-stage6-block-arrows-design.md`
**Branch:** `soundness-batch1-pra` · **Stage base:** `2e1d0efe3`

---

## Global Constraints

- **Stage-entry denominator: 731** (measured, two enumerations, `sort -u`).
- **NO DRAIN IS CLAIMED.** This stage is soundness + unblocking. If a drain appears, measure and report it; do not forecast one. (Spec §6.1 records why the previous stage's forecast was wrong.)
- **PRIMARY GATE: zero newly-red.** `comm -13 pre post` on full `cargo test --workspace --no-fail-fast` enumerations must print **nothing**. Cross-check against a **`main` worktree**.
- **Enumeration MUST use `sort -u`, never `sort`** — 18 test names exist in two harness binaries each; raw `sort` fabricates newly-red.
- **The task order in this plan is forced.** Landing the parse patch (Task 5) before repr-tracking (Task 3) is the one thing that must not happen — it costs the 16 named regressions.
- **A callback consumer that is not wired must fail closed (`E5506`).** Compiling a callback to a function nobody calls is a *new* silent miscompile, not a fix.
- **No hand-mirrored oracles.** `kali_types` must not re-derive HIR's `__kali_fn_N` counter. One name, assigned in the AST, read by both halves.
- **No `_ =>` arm** in any census/deny walk; every no-op arm cites `kali_ast`/`kali_parser` `file:line`.
- **Fixture-authoring:** never `String(<bigint>)` (folds to `0`); never bind a call to a `const` (evaluates `uses + 1` times — use `let`).

---

## File Structure

| file | responsibility |
|---|---|
| `crates/kali_ast/src/expression.rs:145` | **modify.** Add `id: Option<String>` to `ArrowFunctionExpression` (matches `FunctionExpression.id` at `:129`). |
| `crates/kali_cli/src/build/name_anon_functions.rs` | **new.** AST pre-pass naming every anonymous function-shaped node `__kali_fn_{N}`. |
| `crates/kali_cli/src/build/compile.rs` | **modify.** Wire the pre-pass before the resolver (same slot as `module_link` / `monomorphize`). |
| `crates/kali_hir/src/lowering/function.rs:47` | **modify.** Use the AST-assigned `id` when present instead of minting a fresh name. |
| `crates/kali_types/src/resolve/function.rs:6,24,43` | **modify.** Push `current_function` / `current_function_scopes` in `resolve_function_expression`, `resolve_arrow_function`, and class methods. |
| `crates/kali_parser/src/expression/primary.rs` | **modify.** The parse patch (block arrows in all expression positions). |
| `crates/kali_types/src/resolve/call.rs` | **modify.** `reject_anonymous_function_argument` (from the patch), with its exemption list == the set of *actually wired* consumers. |
| `crates/kali_codegen/src/{intrinsics/host.rs, emit/call.rs, lower.rs, lib.rs}` | **modify.** `queueMicrotask` / `setTimeout` recognizers + conditional imports. |
| `crates/kali_cli/tests/soundness_block_arrows.rs` | **new.** Does NOT exist despite the deferred doc's claim — write from scratch. |
| `docs/superpowers/followups/throw-fallout-stage6-triage.md` | **new.** Snapshot, adjudications, probes, follow-ups. |

---

## Task 1: Stage-entry triage + baseline probes

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage6-triage.md`

**Interfaces:**
- Produces: `$SCRATCH/stage6-pre.txt` — the canonical sorted entry set (**731**), consumed by every later gate.

- [ ] **Step 1: Capture two enumerations on a fresh binary**

```bash
cd /workspace && cargo build -p kali_cli
for i in 1 2; do
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stage6-pre-run$i.txt"
done
diff "$SCRATCH/stage6-pre-run1.txt" "$SCRATCH/stage6-pre-run2.txt"
sort -u "$SCRATCH/stage6-pre-run1.txt" "$SCRATCH/stage6-pre-run2.txt" > "$SCRATCH/stage6-pre.txt"
wc -l "$SCRATCH/stage6-pre.txt"      # expect 731
```

STOP and reconcile if it is not 731.

- [ ] **Step 2: Pin the parse defect** (this is the bug, in one command)

Add a temporary test to `crates/kali_parser/tests/parser_integration.rs` (the file already has a `parse(source) -> ParserOutput` helper at `:14`) dumping the AST of `queueMicrotask(() => { ran = 1; });`. Confirm it yields **two** statements — a `CallExpression` whose arg is `ParenthesizedExpression(Identifier("unknown"))`, plus a **separate top-level `BlockStatement`** holding the body. Record the dump in the triage doc, then **remove the temporary test** and confirm `git diff --stat` on that file is empty.

- [ ] **Step 3: Record the four baseline miscompiles** (run each; do not copy from the spec)

| probe | node | kali |
|---|---|---|
| `queueMicrotask(() => { ran = 1; }); console.log(ran);` | `0` (undefined→`0` after init) | body runs inline ⇒ `1` |
| `class C { run(){ return 42; } } console.log(new C().run());` | `42` | `0` |
| a block-arrow callback body using `+=` / `??=` / `typeof` / `new` | runs | (record: today it is *masked* by the flatten) |
| `Kali.test('x', () => { throw new Error('boom'); })` | — | records whether the failure is attributed as `FAILED 1` or surfaces as a `_start` trap |

- [ ] **Step 4: Pin the 16 predicted regressions and confirm they are GREEN today**

```bash
for n in compound_assignment_wrapped_local_binding logical_assignment_wrapped_local_binding \
         nullish_assignment_wrapped_local_binding object_type_and_constructor_semantics; do
  grep -c "$n" "$SCRATCH/stage6-pre.txt"    # expect 0 = currently green
done
```

These are the deferred doc's named fallout (12 + 4). They are the **prediction this stage's task order exists to falsify**. Record the exact test names in the triage doc — Task 5's gate checks them by name.

- [ ] **Step 5: Verify the patch still applies**

```bash
git apply --check docs/superpowers/followups/task8-block-arrows-deferred.patch && echo APPLIES
```

Expected: `APPLIES`. Note in the triage doc that the patch contains **only two source files** — the `soundness_block_arrows.rs` tests the deferred doc claims exist **do not exist** and are written fresh in Task 5.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage6-triage.md
git commit -m "docs(soundness): stage6 triage — entry 731, parse defect pinned, 16 regressions predicted [stage6]"
```

---

## Task 2: Name anonymous functions in the AST

The structural precondition for everything else: one name, assigned before the resolver, read by both halves. (Spec §3.3 — re-deriving HIR's counter in `kali_types` would be a hand-mirrored oracle, and mirrored oracles in this repo fail **open**.)

**Files:**
- Modify: `crates/kali_ast/src/expression.rs:145` (add `id` to `ArrowFunctionExpression`)
- Create: `crates/kali_cli/src/build/name_anon_functions.rs`
- Modify: `crates/kali_cli/src/build/{mod.rs, compile.rs}`
- Modify: `crates/kali_hir/src/lowering/function.rs:47`
- Test: `crates/kali_parser/tests/parser_integration.rs` (AST shape) + `crates/kali_cli/tests/soundness_block_arrows.rs` (end-to-end)

**Interfaces:**
- Produces:
  ```rust
  /// Assign `__kali_fn_{N}` to every anonymous function-shaped node, in a
  /// deterministic pre-order walk. Idempotent: a node that already has an `id`
  /// keeps it.
  pub fn name_anonymous_functions(statements: &mut Vec<Statement>);
  ```
  Consumed by Task 3 (`binding_repr_function_key`) and by HIR lowering.

- [ ] **Step 1: Write the failing test**

```rust
// crates/kali_ast/src/expression.rs — ArrowFunctionExpression gains `id`
// (mirrors FunctionExpression.id at expression.rs:129)

// crates/kali_cli/tests/soundness_block_arrows.rs (new file)
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// A named function expression keeps its own name; two anonymous arrows in the
/// same module get DISTINCT names. If the pre-pass reused one name, the second
/// body would overwrite the first and this prints the wrong value.
#[test]
fn anonymous_functions_get_distinct_stable_names() {
    let out = run_kali(
        r#"const a = () => 1;
const b = () => 2;
console.log(a() + b());
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: FAIL to COMPILE first (the test file is new and `ArrowFunctionExpression.id` does not exist yet). Add the field, then re-run; it should compile and pass or fail on behavior — record which.

- [ ] **Step 3: Implement the pre-pass**

`crates/kali_cli/src/build/name_anon_functions.rs` — a deterministic pre-order walk over every
`Statement`/`Expression`, assigning `__kali_fn_{N}` to any `FunctionExpression` with `id: None` and any
`ArrowFunctionExpression` with `id: None`. **Exhaustive match, no `_ =>` arm** — mirror
`deny_import_positions_expression` (`crates/kali_cli/src/build/module_link.rs:2601`), whose arms
already enumerate every variant.

Wire it in `compile.rs` in the same slot as the other AST passes, **before** the resolver:

```rust
// Name every anonymous function-shaped node BEFORE the resolver, so kali_types
// (binding_repr_function_key) and kali_hir (synthetic function names) key on the
// SAME name. Re-deriving HIR's counter inside kali_types would be a
// hand-mirrored oracle — this repo has shipped two of those, and both failed OPEN.
crate::build::name_anon_functions::name_anonymous_functions(&mut parsed.statements);
```

Then `crates/kali_hir/src/lowering/function.rs:47` — use the id when present:

```rust
let name = expr
    .id
    .clone()
    .unwrap_or_else(|| self.next_synthetic_function_name());
```

(Do the same in `lower_function_expression`.) Keep `next_synthetic_function_name()` as the fallback so
any node the pre-pass did not reach still lowers — but Task 4's test proves the pre-pass reaches them.

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: PASS — prints `3`.

- [ ] **Step 5: Full-workspace gate**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6-t2.txt"
comm -13 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-t2.txt"   # MUST print nothing
```

This task should be **behavior-neutral** (it only assigns names that were previously minted later). A
newly-red name here means a name collision with a source function — check the collision guard.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_ast crates/kali_cli crates/kali_hir
git commit -m "feat(build): name anonymous functions in the AST so types and HIR share one key [stage6]"
```

---

## Task 3: Repr-track function-shaped scopes

The prerequisite the whole stage was deferred on.

**Files:**
- Modify: `crates/kali_types/src/resolve/function.rs:6` (`resolve_function_expression`), `:24` (`resolve_arrow_function`), `:43` (`resolve_class_body` → methods)
- Test: `crates/kali_cli/tests/soundness_block_arrows.rs`

**Interfaces:**
- Consumes: Task 2's AST-assigned `id`.
- Produces: inside an arrow / function-expression / class-method body, `binding_repr_function_key` returns the body's own name instead of `None`, so repr proofs resolve.

- [ ] **Step 1: Write the failing test**

```rust
/// The exact shape of the 16 predicted regressions: a callback body doing
/// compound / logical / nullish assignment on a local, plus `typeof` and `new`.
/// Today these are MASKED because the flattened body sits in module scope.
/// Once bodies are real function scopes, they must still compile and run —
/// that is what repr-tracking buys.
#[test]
fn a_function_expression_body_supports_compound_and_typeof_and_new() {
    let out = run_kali(
        r#"function Box(v) { this.v = v; }
const f = function () {
  let n = 1;
  n += 2;
  let s = null;
  s ??= "x";
  let t = typeof n;
  let b = new Box(9);
  console.log(n + " " + s + " " + t);
};
f();
"#,
    );
    assert!(
        out.status.success(),
        "expected the body to compile and run, got:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3 x number\n");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: FAIL with `E5506` (or an `E4000` trap) — `binding_repr_function_key` returns `None` inside the
function-expression body (`crates/kali_types/src/resolve/expression.rs:306-308`), so the repr proof for
`n += 2` fails closed.

- [ ] **Step 3: Push the function scope onto `current_function`**

Mirror the `FunctionDeclaration` arm (`crates/kali_types/src/resolve/mod.rs:716-747`), which is the only
site that currently pushes. In `crates/kali_types/src/resolve/function.rs`:

```rust
pub(crate) fn resolve_function_expression(&mut self, expr: &FunctionExpression) {
    let function_scope_id = self.push_scope(ScopeType::Function);
    // Repr-tracking (Stage 6): a function-expression body is a REAL function
    // scope, and codegen already compiles it as a named function (Task 2's
    // AST-assigned `id`). Push it onto current_function so
    // binding_repr_function_key resolves inside the body instead of returning
    // None — which is what made compound/typeof/new fail closed in here.
    let name = expr.id.clone().expect("Task 2 names every anonymous function");
    self.current_function.push(name);
    self.current_function_scopes.push(function_scope_id);

    // ... existing generator/id/param/body handling, unchanged ...

    self.current_function_scopes.pop();
    self.current_function.pop();
    self.pop_scope();
}
```

Do the same in `resolve_arrow_function` (`:24`) and for each method in `resolve_class_body` (`:43`).
Also mark params mutable and mirror the runtime-array registry exactly as the `FunctionDeclaration` arm
does (`mod.rs:726-742`) — omitting that would leave a param compound-assign failing closed inside a
callback for a *different* reason, and the bug would look like this task's fault.

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: PASS — prints `3 x number`.

- [ ] **Step 5: Adversarial re-mask probe**

Comment out the two `current_function.push` lines. Rebuild. The test MUST go red with `E5506`. This
proves the test is actually measuring the repr-tracking and not passing for some incidental reason.
Restore; confirm an empty diff.

- [ ] **Step 6: Full-workspace gate + commit**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6-t3.txt"
comm -13 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-t3.txt"   # MUST print nothing
comm -23 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-t3.txt"   # any drain is a BONUS — report it
git add crates/kali_types && git commit -m "fix(types): repr-track arrow/fn-expression/class-method bodies [stage6]"
```

---

## Task 4: The class-method corollary — test the hypothesis

**Files:**
- Test: `crates/kali_cli/tests/soundness_block_arrows.rs`

- [ ] **Step 1: Write the test (the Stage-5 reproducer, verbatim)**

```rust
/// Stage 5 recorded `class C { run(){ return 42; } } new C().run()` → 0.
/// Class-method bodies are one of the untracked function-shaped scopes
/// (kali_types/src/context.rs:48-54), so Task 3 is the HYPOTHESISED root cause.
/// This test decides it either way.
#[test]
fn class_method_bodies_return_their_value() {
    let out = run_kali(
        r#"class C {
  run() {
    return 42;
  }
}
console.log(new C().run());
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}
```

- [ ] **Step 2: Run it and record the honest outcome**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`

- **If it PASSES:** Task 3 fixed a second top-tier silent miscompile. Record the before (`0`) and after
  (`42`) with the probe output in the triage doc.
- **If it FAILS:** the hypothesis is wrong. Say so plainly in the triage doc, mark the test
  `#[ignore = "class-method return lowering — separate root cause, see stage6 triage"]` with its
  assertions intact, and file the finding. **Do not** widen this stage to chase it, and **do not**
  quietly drop the test.

- [ ] **Step 3: Commit either outcome**

```bash
git add crates/kali_cli/tests/soundness_block_arrows.rs docs/superpowers/followups/throw-fallout-stage6-triage.md
git commit -m "test(cli): decide the class-method corollary against the stage-5 reproducer [stage6]"
```

---

## Task 5: Apply the parse patch + wire the callback consumers

The two halves must land **together**: un-flattening without wiring replaces "the body runs at the wrong time" with "the body never runs" (spec §4.1).

**Files:**
- Modify: `crates/kali_parser/src/expression/primary.rs` and `crates/kali_types/src/resolve/call.rs` (via the patch)
- Modify: `crates/kali_codegen/src/{intrinsics/host.rs, emit/call.rs, lower.rs, lib.rs}`
- Test: `crates/kali_cli/tests/soundness_block_arrows.rs`

**Interfaces:**
- Consumes: Tasks 2 + 3.
- Produces: block-bodied arrows parse as named `FunctionExpression` in all expression positions; `queueMicrotask(cb)` calls the host `queue_microtask` import with `cb`'s wasm index; an anonymous callback reaching an **unwired** consumer is `E5506`.

- [ ] **Step 1: Write the failing tests**

```rust
/// The bug, end to end: the arrow's body must NOT execute inline.
/// Pre-stage kali prints `1` (body flattened into module scope and run).
#[test]
fn a_block_arrow_callback_body_does_not_run_inline() {
    let out = run_kali(
        r#"let ran = 0;
queueMicrotask(() => {
  ran = 1;
});
console.log("sync ran=" + ran);
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    // The callback is deferred to the microtask FIFO, so `ran` is still 0 here.
    // (It runs during the post-_start drain; node agrees.)
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("sync ran=0"),
        "callback body ran inline: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// An anonymous callback handed to a consumer that CANNOT invoke it must fail
/// closed — never compile to a function nobody calls.
#[test]
fn an_anonymous_callback_to_an_uninvocable_callee_fails_closed() {
    let out = run_kali(
        r#"function takesCallback(cb) { return 1; }
takesCallback(() => {
  console.log("never");
});
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
```

- [ ] **Step 2: Run and watch them fail**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: FAIL — the first prints `sync ran=1` (body inlined); the second exits 0.

- [ ] **Step 3: Apply the patch**

```bash
git apply docs/superpowers/followups/task8-block-arrows-deferred.patch
cargo build -p kali_cli
```

It touches `crates/kali_parser/src/expression/primary.rs` (block arrows → unnamed `FunctionExpression`
in all expression positions, single-param `x => { … }` included) and
`crates/kali_types/src/resolve/call.rs` (`reject_anonymous_function_argument`). The
`FunctionExpression`s it creates carry `id: None` — **Task 2's pre-pass names them**, which is exactly
why Task 2 comes first.

- [ ] **Step 4: Wire `queueMicrotask` (and audit `setTimeout`)**

The host already provides `queue_microtask` (`crates/kali_runtime/src/host/imports_default.rs:878`) and
`setTimeout` (`:815`), but **codegen imports neither** — verified by `grep` returning empty over
`crates/kali_codegen/src`. Add the recognizer + emit site, mirroring `is_kali_test_call` /
`kali_test_callback_index` (`crates/kali_codegen/src/intrinsics/host.rs:748-765`), which is a plain
name lookup in `self.functions`:

```rust
// crates/kali_codegen/src/intrinsics/host.rs
pub(crate) fn queue_microtask_callback_index(&self, node: &LirNode) -> Option<u32> {
    let callback_node = node.children.get(1).copied()?;   // [callee, callback]
    let callback_name = self.node(callback_node).text.as_deref()?;
    self.functions.get(callback_name).copied()
}
```

```rust
// crates/kali_codegen/src/emit/call.rs — beside the is_kali_test_call arm (:61)
if callee_node.text.as_deref() == Some("queueMicrotask") {
    let (Some(index), Some(import_index)) =
        (self.queue_microtask_callback_index(node), self.queue_microtask_import_index())
    else {
        // Fail closed: NEVER drop the callback silently.
        self.error_e5506("queueMicrotask requires a callback lowered as a function");
        return;
    };
    function.instruction(&Instruction::I32Const(index as i32));
    function.instruction(&Instruction::Call(import_index));
    return;
}
```

Add the import **conditionally**, at the END of the conditional chain in `lower.rs` (after
`crypto_subtle_digest`), with a matching `+ if uses_queue_microtask { 1 } else { 0 }` term in
`function_index_offset` (`lower.rs:92`) — then **no existing import or function index shifts**
(contract documented at `crates/kali_codegen/src/lib.rs:63-71`).

**Audit `setTimeout`/`setInterval` the same way.** Either wire them or make them `E5506`. Whichever you
choose, the exemption list inside `reject_anonymous_function_argument` must contain **exactly** the
consumers that are actually wired — an exemption without a wire is a fail-open, and it is the precise
shape of the bug this stage exists to kill.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4`
Expected: PASS — `sync ran=0`, and the uninvocable-callee case rejects with `E5506`.

- [ ] **Step 6: Adversarial re-mask probes**

- Revert the parse patch only → the first test MUST go red with `sync ran=1` (body inlined).
- Make the `queueMicrotask` emit arm a no-op (drop the callback) → the first test still shows
  `sync ran=0`, so **add an assertion that the callback DID run** (`ran=1` on a later line, after the
  post-`_start` drain) or this probe cannot fail. **A test that cannot distinguish "deferred" from
  "dropped" is exactly the mirage this program keeps finding — fix the test, not the probe.**

Restore; confirm empty diffs.

- [ ] **Step 7: THE gate — the 16 predicted regressions**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6-t5.txt"
comm -13 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-t5.txt"   # PRIMARY GATE: MUST print nothing
grep -E "compound_assignment_wrapped_local_binding|logical_assignment_wrapped_local_binding|nullish_assignment_wrapped_local_binding|object_type_and_constructor_semantics" "$SCRATCH/stage6-t5.txt"
```

The `grep` must print **nothing**. Those 16 are the deferred doc's prediction for landing the patch
*without* repr-tracking; Task 3 is what makes them not happen. **If they appear, the repr-tracking is
incomplete — go back to Task 3. Do not re-pin them, and do not proceed.**

- [ ] **Step 8: Commit**

```bash
git add crates/kali_parser crates/kali_types crates/kali_codegen crates/kali_cli
git commit -m "fix(parser): block-bodied arrows parse as functions; wire the callback consumers [stage6]"
```

---

## Task 6: `Kali.test` attribution + stage gate + triage

**Files:**
- Modify: `crates/kali_cli/tests/soundness_block_arrows.rs`, `docs/superpowers/followups/throw-fallout-stage6-triage.md`

- [ ] **Step 1: Pin the `Kali.test` lane**

Un-flattening moves test bodies out of `_start` and into real `__kali_callback_<index>` exports (the
deferred doc reports this lane already works — **verify, don't trust it**):

```rust
#[test]
fn a_block_arrow_test_callback_failure_is_attributed() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(
        &path,
        r#"Kali.test('bad', () => {
  throw new Error('boom');
});
Kali.test('good', () => {});
"#,
    )
    .expect("write source");
    let out = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&path)
        .output()
        .expect("run kali test");
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("FAILED 1"),
        "failure must be attributed to the named test, not surface as a _start trap: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
```

- [ ] **Step 2: Two independent enumerations + the primary gate**

```bash
cargo build -p kali_cli
for i in 1 2; do
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stage6-post-run$i.txt"
done
diff "$SCRATCH/stage6-post-run1.txt" "$SCRATCH/stage6-post-run2.txt"    # zero drift
sort -u "$SCRATCH/stage6-post-run1.txt" "$SCRATCH/stage6-post-run2.txt" > "$SCRATCH/stage6-post.txt"
comm -13 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-post.txt"   # PRIMARY GATE: must print NOTHING
comm -23 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-post.txt"   # drain — measure, do not forecast
```

- [ ] **Step 3: Main-worktree cross-check**

```bash
# inside /workspace/.worktrees/kali-main
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/main-post.txt"
comm -13 "$SCRATCH/main-post.txt" "$SCRATCH/stage6-post.txt" | comm -13 "$SCRATCH/stage6-pre.txt" -
```

Expected: **empty**.

- [ ] **Step 4: Adversarial whole-stage sweep** (fresh binary; fix reports are not evidence)

- A block arrow in every expression position: call argument, array element, object property, return
  value, ternary branch, template substitution. Each must parse as a function, not flatten.
- A block arrow **capturing** an enclosing local — does it read the right value, or a stale copy?
- Nested arrows (an arrow inside an arrow). Two anonymous arrows in one module — **distinct names?**
- A class method calling another class method.
- `setTimeout(() => {...}, 0)` — does the callback run, or was it silently dropped? (§4.1 — this is the
  exact fail-open shape.)

Every divergence from node is fixed or recorded with node-vs-kali evidence. **A doc claiming
fail-closed while a fail-open is live is itself the defect.**

- [ ] **Step 5: Write the triage doc + commit**

Record: entry 731 → exit (measured); the class-method corollary outcome (§Task 4, either way); the
16-regression prediction and whether it held; any drain, **measured, not forecast**; the re-mask probe
results; and the follow-up inventory — at minimum the re-sequenced **async ordering core** (its spec and
plan are already written and still valid), the **`const`-bound-call double evaluation** miscompile
(`uses + 1` evaluations), and the parser's **`_ => None` silent statement drop** (I2).

```bash
git add docs/superpowers/followups/throw-fallout-stage6-triage.md crates/kali_cli
git commit -m "docs(soundness): stage6 checkpoint — gate, class-method corollary, adversarial sweep [stage6]"
```

---

## Self-Review

**Spec coverage:** §1 the bug → Task 1 Step 2 (pinned) + Task 5. §2 why deferred → Task 3. §3.1 forced order → Tasks 2→3→5, stated in Global Constraints. §3.2 class-method corollary → Task 4 (tests the hypothesis, records either outcome). §3.3 naming key → Task 2. §4 scope → Tasks 2/3/5. §4.1 wiring → Task 5 Step 4 + the Step 6 probe. §5 verification → every task's gate + Task 6. §5 "tests don't exist" → Task 1 Step 5 + Task 2 Step 1 (written fresh). §6 no drain claimed → Global Constraints + Task 6 Step 2. **No gaps.**

**Type consistency:** `run_kali(&str) -> Output` and `kali_bin()` are defined in Task 2 and reused in Tasks 3–6. `name_anonymous_functions(&mut Vec<Statement>)` is defined in Task 2 and consumed in Task 3. `ArrowFunctionExpression.id` is added in Task 2 and read in Tasks 2 (HIR) and 3 (types). `queue_microtask_callback_index` / `queue_microtask_import_index` are introduced together in Task 5.

**Known risk carried into execution:** Task 5 is the moment the parse behavior flips for every callback in the suite. If the 16 predicted regressions appear, the answer is to go back to Task 3 (repr-tracking is incomplete) — **not** to re-pin them. That distinction is the whole reason this stage exists in this order.
