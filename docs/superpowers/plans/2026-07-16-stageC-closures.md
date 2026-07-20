# Stage C — Environment-Pointer Closures Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give kali a real closure model — a callback that reads or mutates an enclosing *function-scope* binding compiles and runs correctly (scalar and heap captures, synchronous and deferred callbacks, arbitrary nesting depth), instead of hard-failing E5506 (writes) or silently reading `0` (reads).

**Architecture:** A closure is the pair `(function_index, env_ptr)`. Captured bindings are promoted from WASM locals to cells in a per-activation **environment record** in the never-reset `__alloc_global` region; each record carries a `parent_env_ptr` for the env chain. A new `current_env` WASM global names the active record; for deferred callbacks the host sets it before the nullary `__kali_callback_<idx>` call, so `env_ptr` rides through the four scheduling imports. MIR already computes the capture set (`captured_by` / `LayoutDescriptor::Closure`); this stage threads it to codegen and materializes the records.

**Tech Stack:** Rust (`kali_mir`, `kali_codegen`, `kali_runtime`, `kali_cli`), WebAssembly (`wasm-encoder`), wasmtime host, node (differential oracle).

**Spec:** `docs/superpowers/specs/2026-07-16-stageC-closures-design.md`

## Global Constraints

- **Branch:** `soundness-batch1-pra`. **Baseline commit:** `192984c39`. **Frozen failure baseline: 731** tests.
- **Develop patch-free on the clean branch.** Do NOT apply `task5-block-arrows-WIP.patch` — that is Stage D. All fixtures use plain `function(){}` / arrow expressions (which reproduce the gap identically per spec §1.1).
- **Primary gate: zero newly-red.** `comm -13 <baseline> <post>` over a full `cargo test --workspace --no-fail-fast` enumeration must print **nothing**. Cross-check against a `main` worktree.
- **Enumeration uses `sort -u`, never plain `sort`** — 18 test names exist in two harness binaries each; raw `sort` fabricates newly-red.
- A full workspace run exceeds one command timeout — run it **detached with a `.done` marker** and poll a bounded blocking loop; do not rely on background notifications.
- **No drain is claimed.** A closed over-rejection may turn some red tests green — measure it, do not forecast it.
- **Reject-don't-miscompile.** Any capture the analysis cannot lower soundly emits **E5506** (`e5::FEATURE_UNAVAILABLE`), never a silent wrong answer. GC-less by design (region/escape only; never a tracing GC).
- **No hand-mirrored oracles.** Everything keys on the Task-2 synthetic name `__kali_fn_N` already assigned before the resolver and read by `kali_types` / `kali_hir` / `repr_infer`.
- **No `_ =>` arm** in any capture/census walk; every no-op arm cites `kali_ast`/`kali_parser` `file:line`.
- **Fixture-authoring:** never `String(<bigint>)` (folds to `0`); never bind a call to a `const` (evaluates `uses + 1` times — use `let`).

---

## File Structure

| file | responsibility |
|---|---|
| `crates/kali_mir/src/env_plan.rs` | **new.** Derive a per-function `EnvPlan` from `captured_by` / `LayoutDescriptor::Closure`: promoted cells (name → slot offset), captured-outer references (name → `(depth, offset)`), and whether the function owns an env record. Pure function over the existing MIR analysis output. |
| `crates/kali_mir/src/lib.rs` | **modify.** Export `env_plan` + the `EnvPlan` / `EnvCell` / `CapturedRef` types. |
| `crates/kali_codegen/src/closure.rs` | **new.** Codegen-side closure surface: the `current_env` global index constant, env-record allocation (bump into `__alloc_global`), cell load/store by offset, and parent-chain walk (`depth` `parent` loads). |
| `crates/kali_codegen/src/emitter.rs` | **modify (`:99`, `:200`, `:389-411`).** Carry the active function's `EnvPlan` on the `FunctionEmitter`; add `current_env` global accessor beside the arena-trio accessors. |
| `crates/kali_codegen/src/lower.rs` | **modify (`:1006-1071`, `:39-42`, `:947-953`, `:774`).** Reserve the `current_env` global; set/restore it in every function prologue/epilogue that owns an env; keep the `__kali_callback_<idx>` export. |
| `crates/kali_codegen/src/emit/literal.rs` | **modify (`:491-516`).** The write path: resolve a captured name to a cell store instead of E5506. |
| `crates/kali_codegen/src/emit/control_flow.rs` | **modify (`:1304`, `:247-281`, `:565-593`).** The read path: resolve a captured name to a cell load instead of the `I64Const(0)` placeholder; save/restore `current_env` alongside the arena trio. |
| `crates/kali_codegen/src/emit/operators.rs` | **modify (`:22`).** The update-expression (`count++`) twin of the write path. |
| `crates/kali_types/src/resolve/function.rs` | **modify (`:6`, `:24`, `:43`).** Already repr-tracks bodies (Stage AB); confirm the closure fail-closed diagnostics surface here for unlowerable captures. |
| `crates/kali_runtime/src/host/imports_default.rs` | **modify (`:250`, `:815`, `:878`).** Add `i64 env_ptr` to `test_register`, `setTimeout`/`setInterval`, `queueMicrotask`; store it beside `callback_id`. |
| `crates/kali_runtime/src/host/imports_node.rs` | **modify (`:563`).** Same for `addEventListener` / `register_event_listener`. |
| `crates/kali_runtime/src/host/enforce.rs` | **modify (`:87-131`).** `invoke_callback` sets the `current_env` global to the stored `env_ptr` before the nullary call and restores it after. |
| `crates/kali_runtime/src/host/state.rs` (or wherever `ScheduledTimer` / `pending_microtasks` live) | **modify.** Carry `env_ptr` on the scheduled entries. |
| `crates/kali_cli/tests/soundness_closures.rs` | **new.** End-to-end fixtures vs node (headline capture shapes + fail-closed cases + re-mask probes). |
| `docs/superpowers/followups/stageC-closures-triage.md` | **new.** Baseline, adjudications, probes, follow-ups. |

---

## Task 0: Stage-entry triage + baseline

**Files:**
- Create: `docs/superpowers/followups/stageC-closures-triage.md`

**Interfaces:**
- Produces: `$SCRATCH/stageC-pre.txt` — the canonical sorted **731** entry set, consumed by every later gate.

- [ ] **Step 1: Confirm the branch and baseline commit**

Run: `git -C /workspace rev-parse --abbrev-ref HEAD && git -C /workspace log --oneline -1`
Expected: on `soundness-batch1-pra`; HEAD is the Stage C design commit (`89e575918` or later). Confirm `192984c39` is an ancestor: `git merge-base --is-ancestor 192984c39 HEAD && echo OK`.

- [ ] **Step 2: Capture two enumerations on a fresh binary (detached)**

```bash
cd /workspace && cargo build -p kali_cli
run_enum() {  # $1 = output path
  ( cargo test --workspace --no-fail-fast 2>&1 \
      | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
      | sort -u > "$1"; touch "$1.done" ) &
}
run_enum "$SCRATCH/stageC-pre-run1.txt"
until [ -f "$SCRATCH/stageC-pre-run1.txt.done" ]; do sleep 5; done
run_enum "$SCRATCH/stageC-pre-run2.txt"
until [ -f "$SCRATCH/stageC-pre-run2.txt.done" ]; do sleep 5; done
diff "$SCRATCH/stageC-pre-run1.txt" "$SCRATCH/stageC-pre-run2.txt"      # expect no drift
sort -u "$SCRATCH/stageC-pre-run1.txt" "$SCRATCH/stageC-pre-run2.txt" > "$SCRATCH/stageC-pre.txt"
wc -l "$SCRATCH/stageC-pre.txt"                                          # expect 731
```

STOP and reconcile if it is not **731** or the two runs differ.

- [ ] **Step 3: Record the four baseline capture miscompiles** (run each on the fresh binary; do not copy expected values from the spec — record what kali actually prints)

| probe (`kali run`) | node | record kali |
|---|---|---|
| `function o(){ let c=0; function inc(){ c+=1; } inc(); inc(); console.log(c); } o();` | `2` | write path E5506? |
| `function o(){ let c=7; function rd(){ return c; } console.log(rd()); } o();` | `7` | read path → `0`? |
| `function o(){ let obj={n:1}; function rd(){ return obj.n; } console.log(rd()); } o();` | `1` | heap read? |
| `let count=0; queueMicrotask(function(){ count+=1; }); console.log("sync="+count);` (module-scope: should already work via module globals) | `sync=0` then drained | record |

Note in the triage doc: the first three are the exact shapes Stage C fixes; the last isolates the module-scope path (must stay working).

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/followups/stageC-closures-triage.md
git commit -m "docs(soundness): stageC triage — entry 731, four baseline capture miscompiles pinned [stageC]"
```

---

## Phase C1 — env plan + scalar synchronous capture (both fail sites)

Goal of C1: a nested function that reads and mutates an enclosing **scalar** local, invoked **synchronously** (direct call), compiles and runs. Scalar only, single owner env (no chain yet, no heap, no deferred). Closes both the write E5506 and the silent read-zero for this shape.

### Task 1: The `EnvPlan` MIR bridge

**Files:**
- Create: `crates/kali_mir/src/env_plan.rs`
- Modify: `crates/kali_mir/src/lib.rs`
- Test: `crates/kali_mir/src/env_plan.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: the MIR analysis output — `MirBinding { name, captured_by, layout, .. }` (`crates/kali_mir/src/binding.rs:16`), `LayoutDescriptor::Closure { captures }` (`crates/kali_mir/src/layout.rs:14`), and the per-function scope nesting already computed by `analysis/walk.rs`.
- Produces:
  ```rust
  /// One promoted binding: it lives in an env cell because a nested function
  /// captures it. `offset` is its byte offset within the owning env record,
  /// AFTER the 8-byte parent_env_ptr header.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct EnvCell { pub name: String, pub offset: u32, pub is_scalar: bool }

  /// A reference, from inside function F, to a binding owned by an ancestor
  /// env `depth` links up the parent chain (0 = F's own env).
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct CapturedRef { pub name: String, pub depth: u32, pub offset: u32, pub is_scalar: bool }

  /// The closure plan for a single function, keyed by its `__kali_fn_N` name
  /// (module root uses the reserved key "" — it never owns an env; its captured
  /// scalars are module globals, handled elsewhere).
  #[derive(Debug, Clone, Default, PartialEq, Eq)]
  pub struct EnvPlan {
      pub owns_env: bool,          // this function has >=1 promoted cell
      pub cells: Vec<EnvCell>,     // its own promoted bindings, in fixed order
      pub captured: Vec<CapturedRef>, // outer bindings it reads/writes
  }

  /// Derive an EnvPlan per function name from a completed MIR analysis.
  pub fn derive_env_plans(analysis: &MirAnalysis) -> std::collections::BTreeMap<String, EnvPlan>;
  ```
  (`MirAnalysis` is the existing completed-analysis handle produced at `crates/kali_mir/src/analysis/mod.rs`; use whatever the crate already exposes as the finalized per-function binding table. If it is not currently public, expose a read-only accessor rather than re-running analysis.)

- [ ] **Step 1: Write the failing test**

```rust
// crates/kali_mir/src/env_plan.rs
#[cfg(test)]
mod tests {
    use super::*;

    /// outer() owns `c` (captured by inc); inc() captures `c` at depth 1.
    /// `c` is scalar → is_scalar true; offset 0 (first cell after header).
    #[test]
    fn scalar_capture_one_level_produces_owner_cell_and_ref() {
        let analysis = crate::analyze_source(
            "function outer(){ let c = 0; function inc(){ c += 1; } inc(); return c; }",
        );
        let plans = derive_env_plans(&analysis);

        let outer = plans.get("outer").expect("outer plan");
        assert!(outer.owns_env);
        assert_eq!(outer.cells, vec![EnvCell { name: "c".into(), offset: 0, is_scalar: true }]);

        let inc = plans.get("inc").expect("inc plan");
        assert!(!inc.owns_env);
        assert_eq!(
            inc.captured,
            vec![CapturedRef { name: "c".into(), depth: 1, offset: 0, is_scalar: true }]
        );
    }
}
```

(`analyze_source` stands for the crate's existing test entry point that runs the full MIR analysis on a source string — reuse the helper the MIR tests already use; grep `crates/kali_mir/src` for the analysis constructor used in `analysis/*_tests.rs` and call that. Do not invent a new pipeline.)

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_mir env_plan -- --nocapture`
Expected: FAIL to compile (`env_plan` module / `derive_env_plans` do not exist yet).

- [ ] **Step 3: Implement `derive_env_plans`**

Walk the finalized per-function bindings. For each function F:
- **cells:** the bindings owned by F whose `captured_by` is non-empty, sorted by name for determinism, assigned `offset = index * 8` (all cells are 8 bytes: i64/f64 scalar or an i64 heap pointer). Set `owns_env = !cells.is_empty()`.
- **captured:** for each name F *uses* that resolves to an ancestor scope's binding (the capture edges MIR already recorded in `captured_by` — invert them: a binding owned by ancestor A with `captured_by` containing F becomes a `CapturedRef` in F), compute `depth` = number of function-scope hops from F up to A (from the scope nesting) and `offset` = A's cell offset for that name.
- `is_scalar = matches!(binding.layout, LayoutDescriptor::Scalar(_))`.

Exhaustive `match` on `LayoutDescriptor` (no `_ =>` — cite `crates/kali_mir/src/layout.rs:5`): `Scalar` → scalar cell; `Struct | Array | Closure | TaggedVal` → heap cell (`is_scalar = false`). Module-root key: never `owns_env` (its scalars are module globals).

Wire the export in `crates/kali_mir/src/lib.rs`:
```rust
mod env_plan;
pub use env_plan::{derive_env_plans, CapturedRef, EnvCell, EnvPlan};
```

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_mir env_plan -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Add an env-chain unit test (defines the `depth` contract now, exercised in C2)**

```rust
#[test]
fn grandparent_capture_is_depth_two() {
    let analysis = crate::analyze_source(
        "function a(){ let g = 5; function b(){ function c(){ return g; } return c(); } return b(); }",
    );
    let plans = derive_env_plans(&analysis);
    let c = plans.get("c").expect("c plan");
    assert_eq!(c.captured, vec![CapturedRef { name: "g".into(), depth: 2, offset: 0, is_scalar: true }]);
}
```

Run: `cargo test -p kali_mir env_plan -- --nocapture` → PASS. (If `depth` computes as 1, the scope-hop counting is wrong — fix here, not in codegen.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_mir/src/env_plan.rs crates/kali_mir/src/lib.rs
git commit -m "feat(mir): derive per-function EnvPlan from the capture set [stageC]"
```

### Task 2: `current_env` global + env-record allocation surface

**Files:**
- Create: `crates/kali_codegen/src/closure.rs`
- Modify: `crates/kali_codegen/src/lib.rs` (module decl), `crates/kali_codegen/src/lower.rs:1006-1071` (global reservation), `crates/kali_codegen/src/emitter.rs:389-411` (accessor)
- Test: `crates/kali_cli/tests/soundness_closures.rs` (new; end-to-end, since the global is only observable through a running closure — the first real behavior lands in Task 3, so this task's gate is the full-workspace neutrality check)

**Interfaces:**
- Produces:
  ```rust
  // crates/kali_codegen/src/closure.rs
  /// Reserved WASM global holding the active environment record pointer (i64;
  /// 0 = no env). Allocated immediately after the arena trio; see the global
  /// map at lower.rs:1006-1032. RESERVED_GLOBAL_COUNT rises from 8 to 9.
  pub(crate) const CURRENT_ENV_GLOBAL: u32 = 8;

  /// Emit: allocate `header + cells*8` bytes in the GLOBAL (never-reset) region,
  /// store `parent_ptr` into the header, leave the new env ptr on the stack.
  /// `parent_ptr` is read from CURRENT_ENV_GLOBAL by the caller before calling this.
  pub(crate) fn emit_env_alloc(function: &mut Function, alloc_global_index: u32, cell_count: u32);
  ```
- Consumes: `emit_bump_body` / `alloc_global_fn_index` conventions (`crates/kali_codegen/src/lower.rs:951`, `emitter.rs:403`) and the global map (`lower.rs:1006-1032`, `RESERVED_GLOBAL_COUNT` at `lower.rs:2552`).

- [ ] **Step 1: Reserve the global**

In `crates/kali_codegen/src/lower.rs`: bump `RESERVED_GLOBAL_COUNT` from `8` to `9` (`:2552`) and append one mutable `I64` zero-initialized global after g7 in the `GlobalSection` build (`:1043-1071`), documenting it in the global-map comment (`:1006-1032`) as `g8 = current_env`. Because module scalar globals are assigned starting at `RESERVED_GLOBAL_COUNT` (`:2659`), raising the count keeps their indices contiguous **above** g8 — no existing index shifts.

- [ ] **Step 2: Add the accessor + module**

`crates/kali_codegen/src/lib.rs`: `mod closure;`. `crates/kali_codegen/src/emitter.rs` (beside `arena_reset_fn_index` at `:411`):
```rust
pub(crate) fn current_env_global(&self) -> u32 { crate::closure::CURRENT_ENV_GLOBAL }
```

- [ ] **Step 3: Implement `emit_env_alloc`**

Mirror the bump/store pattern used for heap object headers (grep `crates/kali_codegen/src/emit` for an existing `Call(alloc_global_index)` followed by an `I64Store` header write — reuse that exact idiom, don't invent addressing):
```rust
pub(crate) fn emit_env_alloc(function: &mut Function, alloc_global_index: u32, cell_count: u32) {
    let bytes = 8 + cell_count * 8;               // parent header + cells
    function.instruction(&Instruction::I64Const(bytes as i64));
    function.instruction(&Instruction::Call(alloc_global_index)); // -> base ptr (i64) on stack
    // header <- parent (CURRENT_ENV_GLOBAL); addressing matches the heap-object
    // header stores elsewhere in emit/. Derived precisely in Task 3 against a
    // running fixture — this task only reserves the global and is behavior-neutral.
}
```
(Leave the store to be finalized in Task 3 where a real fixture proves the addressing; this step's deliverable is the reserved global + the allocation entry point.)

- [ ] **Step 4: Full-workspace neutrality gate**

This task must be **behavior-neutral** (a reserved-but-unused global changes no output). Run the detached enumeration from Task 0 into `$SCRATCH/stageC-t2.txt`, then:
```bash
comm -13 "$SCRATCH/stageC-pre.txt" "$SCRATCH/stageC-t2.txt"   # MUST print nothing
```
A newly-red here means the global reservation shifted an index — re-check `RESERVED_GLOBAL_COUNT` and the module-scalar-global base.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/closure.rs crates/kali_codegen/src/lib.rs crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs
git commit -m "feat(codegen): reserve current_env global + env-record allocation entry point [stageC]"
```

### Task 3: Scalar promotion + synchronous capture — the write and read sites

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs:99,200` (carry the active `EnvPlan`), `crates/kali_codegen/src/lower.rs:947-953,774` (prologue/epilogue env set/restore), `crates/kali_codegen/src/emit/literal.rs:491-516` (write), `crates/kali_codegen/src/emit/control_flow.rs:1304` (read), `crates/kali_codegen/src/emit/operators.rs:22` (update expr)
- Modify: `crates/kali_codegen/src/closure.rs` (finalize `emit_env_alloc` header store; add `emit_cell_load` / `emit_cell_store`)
- Test: `crates/kali_cli/tests/soundness_closures.rs`

**Interfaces:**
- Consumes: `EnvPlan` (Task 1), `CURRENT_ENV_GLOBAL` / `emit_env_alloc` (Task 2).
- Produces:
  ```rust
  // crates/kali_codegen/src/closure.rs
  /// Load cell at `offset` from the env `depth` links up the parent chain.
  pub(crate) fn emit_cell_load(function: &mut Function, current_env_global: u32, depth: u32, offset: u32);
  /// Store the value already on the stack into that cell (consumes the value).
  pub(crate) fn emit_cell_store(function: &mut Function, current_env_global: u32, depth: u32, offset: u32);
  ```

- [ ] **Step 1: Write the failing end-to-end test**

```rust
// crates/kali_cli/tests/soundness_closures.rs (new file)
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String { std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path") }

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin()).current_dir(dir.path()).arg("run").arg(&path).output().expect("run kali")
}

/// Nested function mutates an enclosing scalar local; the enclosing scope reads
/// the mutation back. Pre-C1: `c += 1` hard-fails E5506. node prints 2.
#[test]
fn sync_scalar_capture_write_is_visible_to_owner() {
    let out = run_kali("function outer(){ let c = 0; function inc(){ c += 1; } inc(); inc(); console.log(c); } outer();\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

/// Nested function READS an enclosing scalar. Pre-C1: silently returns 0. node prints 7.
#[test]
fn sync_scalar_capture_read_returns_value_not_zero() {
    let out = run_kali("function outer(){ let c = 7; function rd(){ return c; } console.log(rd()); } outer();\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}
```

- [ ] **Step 2: Run and watch them fail**

Run: `cargo test -p kali_cli --test soundness_closures -- --test-threads=4`
Expected: `write` FAILs with E5506 in stderr; `read` FAILs printing `0`.

- [ ] **Step 3: Carry the `EnvPlan` on the emitter + set/restore in prologue**

- `crates/kali_codegen/src/emitter.rs`: add `pub(crate) env_plan: &'a EnvPlan` (or an owned clone) beside `locals` (`:99`) and `module_global_slots` (`:200`), populated per function from the map built by `derive_env_plans` (call it once during lowering setup in `lower.rs`, keyed by the function's `__kali_fn_N` name / declared name).
- In the function prologue (`lower.rs`, where per-function locals/body are emitted, near `:947-953`): if `env_plan.owns_env`, save `GlobalGet(CURRENT_ENV_GLOBAL)` into a fresh save local, call `emit_env_alloc(function, alloc_global_index, cells.len())`, `GlobalSet(CURRENT_ENV_GLOBAL)`. In every epilogue/return path, restore the save local via `GlobalSet(CURRENT_ENV_GLOBAL)` — mirror the arena-trio save/restore already emitted at `control_flow.rs:565-593`. **Finalize `emit_env_alloc`'s header store now** (parent from the save local) against the running fixture.

- [ ] **Step 4: Promotion at declaration + the two access sites**

Add a helper `EnvPlan::cell_for(name) -> Option<&EnvCell>` and `captured_for(name) -> Option<&CapturedRef>`.

- **Declaration of a promoted local:** where `let`/`var` initializers store into a WASM local, if `env_plan.cell_for(name)` is `Some`, emit `emit_cell_store(depth=0, offset)` instead of `LocalSet`. (The binding gets no WASM local index.)
- **Write path** — `crates/kali_codegen/src/emit/literal.rs`, at the `else` before `:496` (after the `module_global_slots` check, before the E5506 `else`): 
  ```rust
  if !self.locals.contains_key(&name) {
      if let Some(cell) = self.env_plan.cell_for(&name) {           // own promoted cell
          return self.emit_captured_compound_assign(function, op, 0, cell.offset, right);
      }
      if let Some(cap) = self.env_plan.captured_for(&name) {        // outer capture
          return self.emit_captured_compound_assign(function, op, cap.depth, cap.offset, right);
      }
  }
  ```
  where `emit_captured_compound_assign` does `emit_cell_load` → apply op with `right` → `emit_cell_store` (read-modify-write), returning `true`. Only the E5506 fall-through remains for genuinely unresolvable names.
- **Read path** — `crates/kali_codegen/src/emit/control_flow.rs:1304`, replace the placeholder fall-through: before `push_placeholder_fallback_diagnostic`, check `cell_for` / `captured_for` and `emit_cell_load(depth, offset)` returning an `EmittedValue { produced: true, shape }` (shape `Number` for scalar cells; `Unknown` otherwise for now).
- **Update expr** — `crates/kali_codegen/src/emit/operators.rs:22`: same two-check insertion before its E5506.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kali_cli --test soundness_closures -- --test-threads=4`
Expected: both PASS (`2`, `7`).

- [ ] **Step 6: Adversarial re-mask probe**

Comment out the two access-site checks in `literal.rs` (force the E5506 fall-through). Rebuild. `sync_scalar_capture_write...` MUST go red with E5506. Restore; confirm empty `git diff`. Then break the prologue restore (skip the epilogue `GlobalSet`) and add a fixture that calls `outer()` twice — the second call must still print `2`, not `4`; if it prints `4`, the env is leaking across activations. Record the probe in the triage doc; restore.

- [ ] **Step 7: Full-workspace gate + commit**

Detached enumeration → `$SCRATCH/stageC-t3.txt`:
```bash
comm -13 "$SCRATCH/stageC-pre.txt" "$SCRATCH/stageC-t3.txt"   # MUST print nothing
comm -23 "$SCRATCH/stageC-pre.txt" "$SCRATCH/stageC-t3.txt"   # any drain is a BONUS — report it
git add crates/kali_codegen crates/kali_cli/tests/soundness_closures.rs
git commit -m "fix(codegen): promote captured scalars to env cells; sync read+write (C1) [stageC]"
```

---

## Phase C2 — heap-cell captures + env chains

### Task 4: Heap captures

**Files:**
- Modify: `crates/kali_codegen/src/closure.rs` (heap cells store/load a pointer — same 8-byte cell, no repr change) and the access sites in `emit/control_flow.rs` to return the correct `ValueShape` for a heap cell (so downstream `.abort()` / member access lowers).
- Test: `crates/kali_cli/tests/soundness_closures.rs`

**Interfaces:**
- Consumes: `EnvCell.is_scalar` (Task 1). A heap cell holds the object/string/array pointer; the pointee is already SharedHeap-in-global-region (`arena_gate.rs:355`).

- [ ] **Step 1: Write the failing test**

```rust
/// A nested function captures an enclosing HEAP object and reads a field.
/// node prints 1. Pre-C2 the read path returns a scalar shape → member access
/// mis-lowers or fails closed.
#[test]
fn sync_heap_capture_reads_field() {
    let out = run_kali("function outer(){ let obj = { n: 1 }; function rd(){ return obj.n; } console.log(rd()); } outer();\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test -p kali_cli --test soundness_closures sync_heap_capture -- --test-threads=4`
Expected: FAIL (wrong value or E5506 on the member access).

- [ ] **Step 3: Implement**

In the read-path resolution (Task 3, Step 4), when `is_scalar == false` return `EmittedValue { produced: true, shape: ValueShape::HeapObject /* or the shape the emitter uses for object handles */ }` so member access / method calls resolve against the loaded pointer. Confirm the heap object is allocated in the global region for the capturing case (it already is, via `arena_gate`); if a fixture shows the object was arena-freed before the read, that is a `has_global_site` gap — record and fail closed, do not silently read freed memory.

- [ ] **Step 4: Run → PASS. Step 5: re-mask probe** (force `is_scalar=true` for the heap cell → member access breaks). **Step 6: gate + commit.**

```bash
git commit -m "fix(codegen): heap captures load object pointer with correct shape (C2) [stageC]"
```

### Task 5: Env chains (parent walk, depth > 1)

**Files:**
- Modify: `crates/kali_codegen/src/closure.rs` (`emit_cell_load`/`store` already take `depth`; verify the parent-walk loop emits `depth` chained `I64Load` of the header before the cell offset).
- Test: `crates/kali_cli/tests/soundness_closures.rs`

- [ ] **Step 1: Write the failing test**

```rust
/// Grandparent capture: c reads a's `g` through b. node prints 5.
#[test]
fn env_chain_grandparent_read() {
    let out = run_kali("function a(){ let g = 5; function b(){ function c(){ return g; } return c(); } console.log(b()); } a();\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}
```

- [ ] **Step 2: Run → FAIL** (depth-2 walk not emitted → reads b's env header as the value, or E5506).

- [ ] **Step 3: Implement the parent walk**

`emit_cell_load(depth, offset)`: `GlobalGet(current_env)`, then `depth` times `I64Load(offset=0)` (the parent header is at offset 0), then `I64Load(offset = 8 + offset)` for the cell. `emit_cell_store` the same addressing with the value threaded through a scratch local. **Requires** that when `a` calls `b` and `b` calls `c`, each env's `parent` header points at the lexical parent — verify the prologue sets `parent = current_env` at entry and that `b`/`c` that own no cells still leave `current_env` pointing at the correct ancestor (a no-cell function does not allocate and does not change `current_env`, per spec §3.4). If `c` owns no env, its `depth` to `g` counts function hops, not env-record hops — reconcile: **`depth` must count only ancestors that OWN an env record.** Fix `derive_env_plans` (Task 1) to count env-owning hops if this test exposes a mismatch, and add a MIR unit test pinning it.

- [ ] **Step 4: Run → PASS. Step 5: gate + commit.**

```bash
git commit -m "fix(codegen): env-chain parent walk for depth>1 captures (C2) [stageC]"
```

> **Note for the implementer:** Task 5 Step 3 surfaces the one subtlety in the whole design — `depth` counts **env-owning** ancestors, not lexical function hops, because a function that owns no cells allocates no record and is transparent to the chain. If the C1 MIR unit test (`grandparent_capture_is_depth_two`) assumed lexical hops, update it and `derive_env_plans` together here, with a comment citing spec §3.4.

---

## Phase C3 — deferred host threading

### Task 6: Thread `env_ptr` through the scheduling imports + `invoke_callback`

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs:250` (`test_register`), `:815` (`setTimeout`/`setInterval`), `:878` (`queueMicrotask`); `crates/kali_runtime/src/host/imports_node.rs:563` (`addEventListener` / `register_event_listener`); `crates/kali_runtime/src/host/enforce.rs:87-131` (`invoke_callback`); the state module holding `ScheduledTimer` / `pending_microtasks` / registered listeners.
- Modify: `crates/kali_codegen/src/emit/call.rs` + `crates/kali_codegen/src/intrinsics/host.rs` — pass `GlobalGet(current_env)` as the new trailing `env_ptr` argument at each scheduling call site.
- Test: `crates/kali_cli/tests/soundness_closures.rs`

**Interfaces:**
- Consumes: `CURRENT_ENV_GLOBAL` (Task 2). The host reads/writes the guest global via `instance.get_global(...).get/set` (wasmtime).
- Produces: each scheduling import signature gains a trailing `env_ptr: i64`; `invoke_callback(instance, store, callback_id, env_ptr)` sets `current_env` before the call and restores it after.

- [ ] **Step 1: Write the failing test**

```rust
/// Deferred callback captures an enclosing scalar and mutates it; asserted
/// AFTER the microtask drain (so the test distinguishes "captured" from
/// "coincidentally zero"). node prints "before=0" then "after=1".
#[test]
fn deferred_scalar_capture_runs_with_its_env() {
    let out = run_kali(
        "function outer(){ let c = 0; queueMicrotask(function(){ c += 1; }); console.log(\"before=\"+c); return function(){ return c; }; }\nlet read = outer();\nqueueMicrotask(function(){ console.log(\"after=\"+read()); });\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("before=0"), "stdout: {}", s);
    assert!(s.contains("after=1"), "stdout: {}", s);
}
```

(If returning a closure is itself unsupported at C3, replace the post-drain read with a heap flag the deferred callback sets and a second microtask that prints it — keep the post-drain assertion.)

- [ ] **Step 2: Run → FAIL** (the deferred callback runs with `current_env` = 0 → `c += 1` writes the wrong env or the read sees stale `0`).

- [ ] **Step 3: Add `env_ptr` to the scheduling state + imports**

- State (`ScheduledTimer` at `enforce.rs:74-79` and the microtask/listener queues): add `env_ptr: i64`.
- Each scheduling import: add the trailing `i64 env_ptr` param, store it with the `callback_id`. Update the codegen emit sites (`emit/call.rs`, `intrinsics/host.rs`) to push `GlobalGet(CURRENT_ENV_GLOBAL)` as the final argument, and update the import type signatures in the codegen import table AND the four hand-mirrored `kali:rt` JS import lists (harness.rs ×2 + cmd_build.rs ×2 — see `[[kali-browser-harness-import-sync]]`) or browser tests fail with a LinkError.

- [ ] **Step 4: Set `current_env` in `invoke_callback`**

```rust
pub(crate) fn invoke_callback(
    instance: &Instance, store: &mut Store<KaliHostState>, callback_id: i32, env_ptr: i64,
) -> Result<(), Diagnostic> {
    let env_global = instance.get_global(&mut *store, "__current_env"); // exported name for g8
    let saved = env_global.map(|g| g.get(&mut *store));
    if let Some(g) = env_global { let _ = g.set(&mut *store, Val::I64(env_ptr)); }
    // ... existing lookup + nullary call ...
    if let (Some(g), Some(prev)) = (env_global, saved) { let _ = g.set(&mut *store, prev); }
    Ok(())
}
```
Export g8 under a stable name (`__current_env`) from codegen (`lower.rs` export section) so the host can resolve it. Update both `drain_event_loop` call sites (`:34`, `:62`) to pass the stored `env_ptr`.

- [ ] **Step 5: Run → PASS. Step 6: re-mask probe** — make `invoke_callback` ignore `env_ptr` (always set 0). The deferred test MUST go red. Restore.

- [ ] **Step 7: Full-workspace gate (incl. a browser-harness run) + commit**

```bash
git commit -m "fix(runtime): thread env_ptr through scheduling imports; host sets current_env per deferred callback (C3) [stageC]"
```

---

## Phase C4 — fail-closed hardening + lockstep + stage gate

### Task 7: Fail-closed boundaries + F-AB-2 lockstep assertion

**Files:**
- Modify: `crates/kali_types/src/resolve/function.rs` and/or `crates/kali_codegen/src/closure.rs` — emit E5506 for captures the plan cannot lay out.
- Modify: `crates/kali_types/src/repr_infer.rs` (add the lockstep assertion at the walk-4 `_` arm tripwire).
- Test: `crates/kali_cli/tests/soundness_closures.rs`

- [ ] **Step 1: Write the fail-closed tests**

```rust
/// Array per-element callback with a capture — the array callback ABI is a
/// separate follow-up stage. Must fail closed E5506, never silent.
#[test]
fn array_callback_capture_fails_closed() {
    let out = run_kali("function outer(){ let base = 10; let r = [1,2,3].map(function(x){ return x + base; }); console.log(r.join(\",\")); } outer();\n");
    assert!(!out.status.success(), "expected E5506, got: {}", String::from_utf8_lossy(&out.stdout));
    assert!(String::from_utf8_lossy(&out.stderr).contains("E5506"), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

/// An F-AB-2 exotic position (fn-expr inside an object literal passed directly
/// as a call arg) that would capture — must fail closed, not silently lower.
#[test]
fn exotic_position_capture_fails_closed() {
    let out = run_kali("function outer(){ let c = 1; sink({ f: function(){ return c; } }); } function sink(o){ return o; } outer();\n");
    // Either E5506 or a clean no-op; MUST NOT print a wrong captured value silently.
    assert!(!String::from_utf8_lossy(&out.stdout).contains("garbage"), "stdout: {}", String::from_utf8_lossy(&out.stdout));
    // Prefer an explicit E5506 when the fn-expr is invocable; assert it if the callee invokes.
}
```

- [ ] **Step 2: Run → record actual behavior; implement the E5506 guards** so unlowerable captures (array per-element; unknown cell repr; the F-AB-2 exotic positions when invocable) emit `Diagnostic::error(e5::FEATURE_UNAVAILABLE, ...)` at closure creation rather than a silent value.

- [ ] **Step 3: Add the F-AB-2 lockstep assertion**

At the `repr_infer.rs` walk-4 `_` arm (tripwire from Stage AB, commit `a57cd09d5`): after both walk 1–3 registration and walk-4 seeding complete, assert the `__kali_fn_N` sets are equal (debug-assert + a unit test in `kali_types`). Cite `docs/superpowers/followups/stageAB-followups.md` §F-AB-2. This makes a future divergence trip a test, not a silent i64.

- [ ] **Step 4: Run → PASS. Commit.**

```bash
git commit -m "fix(codegen): fail closed on unlowerable captures + F-AB-2 lockstep assertion (C4) [stageC]"
```

### Task 8: Headline test flip, stage gate, triage, adversarial sweep

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke.rs:444-449` (+ the `:490-495` / build.rs browser variants), `crates/kali_cli/tests/soundness_closures.rs`, `docs/superpowers/followups/stageC-closures-triage.md`

- [ ] **Step 1: Flip the headline test to assert success**

`structured_clone_and_event_primitives_source` (`runtime_smoke.rs:424`) captures `count` (scalar) and `controller` (heap) in an `addEventListener` closure invoked synchronously via `dispatchEvent`. The current assertions require fail-closed (`run.rs:403-416`). Change them to assert the program **succeeds** and produces node's output for both test-mode (`:426`, capture inside a `Kali.test` arrow — function scope) and non-test-mode (`:472`, module scope). Verify vs node first: run node on both source variants, record the expected stdout, assert byte-for-byte.

- [ ] **Step 2: Add the recursion/distinct-env fixture**

```rust
/// Each activation of `make` creates a distinct closure over its own `n`.
/// node prints "10 20". Proves env-per-activation, not a shared global cell.
#[test]
fn recursion_distinct_envs() {
    let out = run_kali("function make(n){ return function(){ return n; }; } let a = make(10); let b = make(20); console.log(a() + \" \" + b());\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10 20\n");
}
```

- [ ] **Step 3: Adversarial whole-stage sweep** (fresh binary; fix reports are not evidence)

Run each vs node, fix or record every divergence with node-vs-kali evidence:
- capture read, capture write, capture read-modify-write; scalar and heap;
- one-level, two-level (chain), and a capture that skips a no-cell intermediate function;
- deferred (`queueMicrotask`, `setTimeout(cb,0)`, `addEventListener`+`dispatchEvent`), each asserted **post-drain**;
- `outer()` called twice → distinct envs (no cross-activation leak);
- a closure that captures nothing → still allocates no env, no `current_env` churn;
- the module-scope path (spec §3.3 case 4) still works unchanged.

- [ ] **Step 4: Two independent enumerations + primary gate + main cross-check**

```bash
# two detached enumerations → stageC-post-run1/2.txt, diff for drift, sort -u → stageC-post.txt
comm -13 "$SCRATCH/stageC-pre.txt" "$SCRATCH/stageC-post.txt"   # PRIMARY GATE: must print NOTHING
comm -23 "$SCRATCH/stageC-pre.txt" "$SCRATCH/stageC-post.txt"   # drain — measure, do not forecast
# main worktree cross-check (see umbrella §2): must be empty
```

- [ ] **Step 5: Write the triage doc + memory + commit**

Record: entry 731 → exit (measured); the four baseline miscompiles now fixed (before/after); any drain (measured); the re-mask probe results; the `depth`-counts-env-owning-ancestors reconciliation (Task 5); and the follow-up inventory — the **array per-element callback ABI stage**, the **reclaimable escaping-capture region**, and F-AB-1. Update `MEMORY.md` with a `kali-block-arrows-stageC` pointer.

```bash
git add crates/kali_cli docs/superpowers/followups/stageC-closures-triage.md
git commit -m "docs(soundness): stageC gate — capture fixtures green, headline flipped, 731 held [stageC]"
```

---

## Self-Review

**Spec coverage:** §1 two fail sites → Task 3 (both). §1.1 patch-free clean-branch → Global Constraints + every fixture uses `function(){}`. §2 scope table: full deferred+heap → C1(scalar)/C2(heap)/C3(deferred); host-invoked-only, array as follow-up → Task 7 fail-closed + Task 8 follow-up; full env chains → Task 5; controlled-leak reclamation → Task 2 (`__alloc_global`). §3.1 closure pair, no table → Tasks 3/6. §3.2 env layout + region → Task 2/3. §3.3 promotion + 4 resolution cases → Task 3 (cases 1–3) + Task 8 Step 3 (case 4 module scope). §3.4 prologue-managed current_env, direct call needs nothing → Task 3 Step 3 + Task 5 note. §4 fail-closed + §4.1 lockstep → Task 7. §5 MIR bridge, __kali_fn_N key → Task 1. §6 success criteria → Tasks 3/4/5/6/8 fixtures. §7 gating → Task 0 + every task gate + Task 8. §7.1 phasing C1–C4 → Phase headers. §8 follow-ups → Task 8 Step 5. **No gaps.**

**Placeholder scan:** the two "derive precisely in Task 3 / against a running fixture" notes (Task 2 Step 3, Task 3) are deliberate — the env-record byte addressing must be pinned against a real fixture rather than guessed, and Task 3 Step 3 finalizes it with a re-mask probe proving it. Every other step carries real code, real fixtures, exact commands.

**Type consistency:** `EnvPlan`/`EnvCell`/`CapturedRef` (Task 1) are consumed by name in Tasks 3–7. `CURRENT_ENV_GLOBAL` / `emit_env_alloc` (Task 2) used in Task 3. `emit_cell_load`/`emit_cell_store(function, current_env_global, depth, offset)` (Task 3) used in Tasks 4–5. `invoke_callback(..., env_ptr)` (Task 6) matches both `drain_event_loop` call sites. `run_kali`/`kali_bin` (Task 3) reused Tasks 4–8.

**Known risk carried into execution:** the `depth`-counts-env-owning-ancestors subtlety (Task 5 note) is the single place the design's abstraction can be gotten wrong. It has a MIR unit test (Task 1 Step 5) and an end-to-end test (Task 5) that jointly pin it; if they disagree, `derive_env_plans` is the thing to fix, not codegen.
