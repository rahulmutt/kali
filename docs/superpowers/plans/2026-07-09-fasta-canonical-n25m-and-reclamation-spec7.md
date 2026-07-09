# fasta Spec 7 — Canonical N=25M + Reclamation + Soundness Closures Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run the verbatim fasta fixture byte-for-byte at its canonical input N=25,000,000 by reclaiming per-line runtime-string temporaries into the existing per-iteration arena, and close three adjacent soundness gaps (existential-laundering param proof, module var-object `o+=1` miscompile, `??=` nullish-vs-falsy miscompile).

**Architecture:** Five tasks. Tasks 1–3 are self-contained resolve/codegen soundness fixes (reject-don't-miscompile), each additive-reject with its own reproducer pins. Task 4 is the headline: `__join` and host `string_concat` currently allocate into the never-reset global arena (`__alloc_global`); Task 4 adds arena-variant twins (`__join_arena`, `string_concat_arena`) that allocate into the current arena (`__alloc`), plus a new per-call-site escape analysis (a new `ArenaTable` string-site channel populated by `classify_value`/`escape_flow`) that proves a string temporary is iteration-local before routing it to the resettable arena — fail-closed to global otherwise. Task 5 replaces the interim N=2M SHA-256 tier with the canonical N=25M pin and adds a bounded-peak regression proof.

**Tech Stack:** Rust workspace (`kali_lexer`, `kali_common`, `kali_types`, `kali_mir`, `kali_codegen`, `kali_cli`); hand-emitted wasm via the `wasm-encoder` crate; wasmtime-backed `kali run`; `sha2` (dev-dep); node v26.4.0 as the reference oracle.

## Global Constraints

- **Gate command** (run in the FOREGROUND before every commit that touches compiler crates):
  `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` and `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` — all clean. The CLI package is `kali_cli` (binary `kali`; tests use `CARGO_BIN_EXE_kali`). Clippy is CI-enforced (`ci.yml`) and the per-task gate historically missed lints — do not skip it.
- **Controller discipline (5+ prior validations):** re-run every reproducer on a **freshly-built** binary. Trust observed behavior, not impl/fix reports or static reads — the Spec 7 design itself corrected two static-read errors this way. Reviewers must execute a fix's claimed mechanism example, not just its reject pins.
- **Fail-closed, never fail-open:** unprovable forms reject (E5506), never miscompile. Both-sides oracle mirroring: any new expression shape recognized in codegen must have a matching kali_types/analysis predicate, or it fails open.
- **Additive to existing fixtures:** all five prior CLBG fixtures (nbody, fannkuch, spectral-norm, mandelbrot n=200, binary-trees N=21) stay **byte-identical**. Task 4 must not change routing for any allocation site it cannot positively prove iteration-local.
- **Node oracle:** v26.4.0. Reference constants: fixture at `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts`; N=25,000,000 output SHA-256 = `6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee` (re-derive locally against node before pinning if in doubt).
- **Integration:** push a PR + self-merge per the `kali-integration-convention` memory (`gh` authed as rahulmutt; run `gh auth setup-git` if git can't read credentials). Branch: `fasta-canonical-n25m-spec7` (already created; the spec doc is committed there).

---

## Task 1: Existential-laundering closure (∃ ⇒ ∃ ∧ ∀)

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs:1698-1770` — the Step 1b `scalar_inflow_params` loop.
- Test: `crates/kali_cli/tests/param_compound_assign.rs` (append reproducer pins; file exists from Spec 6).

**Interfaces:**
- Consumes: `CallEdge` fields `arg_scalar_syntactic: Vec<bool>`, `arg_array_literal: Vec<bool>`, `arg_array_names: Vec<Option<(String,String)>>`, `arg_obj_slots: Vec<Option<ObjSlot>>` (repr_infer.rs:89-116); `self.scalar_inflow_params: BTreeSet<(String,String)>`.
- Produces: no new public surface — tightens the population of `scalar_inflow_params`, which is emitted (negated) as `ReprTable::params_lacking_scalar_inflow` and read by `target_repr_is_one_of` (resolve/expression.rs:1927). Behavior change: a param compound/update gate rejects when **any** call edge passes it a non-scalar-or-unproven argument, even if another edge is scalar.

### Background

Today Step 1b is purely existential: a param enters `scalar_inflow_params` when **some** edge supplies a syntactically-scalar arg. The hole (Spec 6 final review, deferred/masked): `f(5); f(g())` — the `f(5)` edge proves `f`'s param scalar, so the gate admits `p += 1` in `f` even though `f(g())` could deliver a heap handle; a self-recursive `h(p+1)` seeds its own chain. Inert today only because indirect array *delivery* is non-functional (param receives 0). We close it now because it is the same positive-proof family and Task 4 must not rely on that masking.

The fix pairs the existential proof with an **∀-no-unproven-edge** veto: a param is proven-scalar iff (some edge is syntactically scalar) AND (no edge passes it a non-scalar or unproven argument). An argument is *proven-scalar-evidence* iff `arg_scalar_syntactic[k]` is true (literal/arith/unary/update/template). An argument is a *veto* iff it is an array (`arg_array_literal[k]` or an identifier in `array_bindings`), an object (`arg_obj_slots[k].is_some()`), or **neither proven-scalar nor a veto-array/object** — i.e. an unproven indirect form (call-result `g()`, member `o.a`) that is not syntactically scalar. A self-recursive `h(p+1)` passes `p+1` (syntactically scalar) → not a veto → still self-proves.

- [ ] **Step 1: Write the failing reproducer pins**

Append to `crates/kali_cli/tests/param_compound_assign.rs` (reuse the file's existing `run_source` helper — a unique-slug temp-file `kali run` returning `std::process::Output`):

```rust
// Existential-laundering closure (Spec 7 Task 1): a scalar edge at one call
// site must NOT admit a compound assign when ANOTHER edge passes an unproven
// (indirect, non-syntactically-scalar) argument. `g()` is a call-result → the
// `f(g())` edge is a veto → `f`'s `p += 1` must reject fail-closed (E5506),
// even though `f(5)` is a scalar edge. Masked today (indirect delivery gives 0)
// but must be structurally closed.
#[test]
fn mixed_scalar_and_indirect_edges_reject_param_compound() {
    let out = run_source(
        "function g() { return {x:1}; }\n\
         function f(p) { p += 1; return p; }\n\
         f(5);\n\
         console.log(f(g()));\n",
    );
    assert!(!out.status.success(), "expected E5506 reject, got success: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506") || stderr.contains("not a provably scalar"),
        "expected fail-closed compound-assign diagnostic, stderr: {stderr}");
}

// A purely-scalar call site (fasta's shape: `2*n`, `k+0`) still ADMITS — the
// ∀-condition must not over-reject genuine scalar flow.
#[test]
fn all_scalar_edges_still_admit_param_compound() {
    let out = run_source(
        "function f(p) { p += 1; return p; }\n\
         console.log(f(2 * 3));\n\
         console.log(f(10 + 0));\n",
    );
    assert!(out.status.success(), "expected admit, stderr: {}",
        String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n11\n");
}
```

- [ ] **Step 2: Run the tests to confirm the reject one fails (currently admits/miscompiles)**

Run: `cargo test -p kali_cli --test param_compound_assign mixed_scalar_and_indirect_edges_reject_param_compound -- --exact`
Expected: FAIL (the program currently compiles and runs — exit 0 — instead of rejecting).

- [ ] **Step 3: Add the ∀-veto to Step 1b**

In `crates/kali_types/src/repr_infer.rs`, replace the existential-only loop body (around 1746-1770) so a param is inserted into `scalar_inflow_params` only after a two-pass check: (a) at least one edge is `arg_scalar_syntactic`, and (b) **no** edge for that param is a veto. Structure it as: first compute, per `(callee, param)` key, `has_scalar_evidence` and `has_veto_edge` across all edges; then insert the key iff `has_scalar_evidence && !has_veto_edge`. Veto for edge `k`:

```rust
let ident = edge.arg_array_names.get(k).cloned().flatten();
let is_array = edge.arg_array_literal.get(k).copied().unwrap_or(false)
    || ident.as_ref().is_some_and(|(caller, name)| {
        array_bindings.contains(&(caller.clone(), name.clone()))
    });
let is_object = matches!(edge.arg_obj_slots.get(k), Some(Some(_)));
let is_scalar = edge.arg_scalar_syntactic.get(k).copied().unwrap_or(false);
// Veto: an array/object argument, OR an argument that is neither proven
// scalar nor a known array/object — an unproven indirect form (call-result,
// member read) that could deliver a heap handle. Only a syntactically-scalar
// argument is non-veto evidence.
let is_veto = is_array || is_object || !is_scalar;
```

Because `arg_obj_slots` returns `Some` for **every** bare identifier (documented at repr_infer.rs:1728), a bare-identifier argument is already a veto under `is_object` — consistent with the existing "bare identifiers are never scalar evidence" rule. A self-edge `h(p+1)` has `is_scalar==true` (arithmetic) → not a veto. Convert the single-pass `for edge in &self.calls` into a two-pass accumulation (first pass fills two `BTreeSet`/`BTreeMap` keyed by `(callee, param)`: one of keys-with-scalar-evidence, one of keys-with-veto; then insert `scalar_inflow_params = scalar_evidence \ veto`). Update the large doc comment (1698-1745) to describe the ∀-condition and delete the "MUST be closed before making array delivery functional" tripwire note (now closed).

- [ ] **Step 4: Run the reproducer pins + the full param_compound_assign suite**

Run: `cargo test -p kali_cli --test param_compound_assign`
Expected: PASS — both new tests plus the existing 16 (16/16 → 18/18).

- [ ] **Step 5: Regression gate**

Run: `cargo test -p kali_types -p kali_cli --test clbg_fasta_runtime` then the full gate (`cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check && cargo clippy --workspace -- -D warnings`).
Expected: PASS — fasta 2/2 (N=8 golden + N=2M SHA still green; Task 5 changes the large tier later), all crates clean. Re-run `kali run` on the two reproducers against the freshly-built binary to confirm behavior, not just the test harness.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/param_compound_assign.rs
git commit -m "fix(types): pair scalar-inflow existential proof with an all-edges veto (fasta Spec 7 Task 1)"
```

---

## Task 2: Module var-object `o+=1` reject (diagnose-then-fix)

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` around `target_repr_is_one_of` (1927) — extend the scalar allowlist to reject object/array-initialized bindings. (Exact locus confirmed in Step 2.)
- Possibly modify: `crates/kali_types/src/repr_infer.rs` — if the fix is to give an object-initialized module binding a non-I64 (heap) repr rather than an initializer-taint set.
- Test: `crates/kali_cli/tests/module_var_object_compound.rs` (create).

**Interfaces:**
- Consumes: `target_repr_is_one_of(&self, name, allowed)` (resolve/expression.rs:1927) — the single choke point both the compound and update gates route through; `ReprTable::scalar(func, name)`, `ReprTable::is_array_binding`, `ReprTable::is_non_scalar_param`.
- Produces: the compound/update gate rejects a binding whose declarator initializer is an object/array literal (heap) whose repr defaulted to I64. No new public method unless Step 2 chooses the taint-set approach (then a `ReprTable::mark_object_initialized_binding` / `object_initialized_binding` accessor pair mirroring `non_scalar_params`).

### Background (verified 2026-07-09 on a fresh binary)

`var o = {x:1}; o += 1; console.log(o)` prints `1` (node: `[object Object]1`); `var o = {x:1}; console.log(o.x)` prints `0`. `o` is **not** promoted to a module global (`collect_module_scalar_globals` only promotes names referenced inside a function; this `o` is module-only), so it is a `_start` local with default repr `I64` (object shape never proven). `o += 1` takes the generic **local** compound path and does `0 + 1`. The gate `compound_update_target_is_scalar` (resolve/expression.rs:1697) admits because `scalar("_start","o") == I64`. Var locals are exempt from the param positive-proof lane, so an object-initialized var slips through. This is the same fail-open family as Task 1, now for a var local. (Fixing `o.x` reading 0 is out of scope — only the compound miscompile.)

- [ ] **Step 1: Write the failing reject pin + a positive control**

Create `crates/kali_cli/tests/module_var_object_compound.rs`:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String { std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path") }

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-mod-obj-{}-{}-{}",
        std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed), src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin()).arg("run").arg(&path).output().expect("run kali")
}

// A compound assign on an object-initialized binding has no scalar lowering
// (node string-coerces the object; kali cannot) — must reject fail-closed,
// never miscompile `0 + 1 = 1`.
#[test]
fn object_initialized_binding_compound_rejects() {
    let out = run_source("var o = {x:1};\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("E5506")
        || String::from_utf8_lossy(&out.stderr).contains("not a provably scalar"));
}

// A genuine numeric var local still compiles and runs — the fix must not
// over-reject scalars.
#[test]
fn numeric_var_local_compound_still_runs() {
    let out = run_source("var k = 0;\nk += 1;\nk += 41;\nconsole.log(k);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}
```

- [ ] **Step 2: Diagnose the exact repr/locus (short investigation)**

Run these and record findings inline in the commit message:
```bash
cargo build -p kali_cli --bin kali
# Confirm o stays a _start local (not promoted) and repr is I64:
printf 'var o={x:1};\no+=1;\nconsole.log(o);\n' > /tmp/o.ts; ./target/debug/kali run /tmp/o.ts
```
Decide the fix locus: **(A)** give an object/array-literal-initialized binding a non-I64 (heap/Object) `scalar` repr in `repr_infer.rs` so `target_repr_is_one_of` rejects it via the existing `allowed.contains(...)` check (preferred if inference already sees the initializer); **or (B)** add an `object_initialized_binding` taint set in `ReprTable` (mirroring `non_scalar_params`), populated in inference when a declarator RHS is an ObjectExpr/ArrayExpr, and add `if self.repr_table.object_initialized_binding(&func, name) { return false; }` to `target_repr_is_one_of` alongside the existing `is_array_binding`/`is_non_scalar_param` guards. Prefer (A) if it does not disturb other repr consumers; else (B), which is contained to the gate.

- [ ] **Step 3: Implement the chosen fix**

Route through `target_repr_is_one_of` (resolve/expression.rs:1927) so it also gates codegen (resolve runs first; a reject stops compilation). If (B): add the guard line and the `ReprTable` accessor pair + inference population. Keep the fix minimal and fail-closed — an object/array-initialized binding rejects; a numeric binding is untouched.

- [ ] **Step 4: Run the pins + regression**

Run: `cargo test -p kali_cli --test module_var_object_compound` (expect 2/2 PASS), then the full gate. Re-run both reproducers on the fresh binary.
Expected: `o += 1` → E5506 exit 1; `k += 1` → 42; all five CLBG fixtures byte-identical; fasta 2/2.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src crates/kali_cli/tests/module_var_object_compound.rs
git commit -m "fix(types): reject compound/update on an object-initialized binding (fasta Spec 7 Task 2)"
```

---

## Task 3: Reject scalar `??=` fail-closed

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs:1683-1715` — the `NullishAssign` branch of the compound gate, to reject scalar `??=` (except the for-in-key-alias sentinel path).
- Modify: `crates/kali_codegen/src/emit/literal.rs:571-588` — the now-unreachable-for-scalars `"??="` arm may stay (defensive) but its scalar behavior is dead once resolve rejects; keep the for-in-key-alias path working.
- Modify: `crates/kali_codegen/src/emit/literal_tests.rs:100-101` and `crates/kali_codegen/src/test_support.rs:47` — update/relocate `assert_nullish_assignment_lowers` to a for-in-key-alias case (the only surviving `??=` lowering) or remove the scalar case.
- Test: `crates/kali_cli/tests/nullish_assign_reject.rs` (create).

**Interfaces:**
- Consumes: `binding_is_mutable`, the `AssignmentOperator::NullishAssign` arm, `for_in_key_aliases` recognition (codegen).
- Produces: `??=` on a scalar local/param → E5506 in the resolve phase. Only a for-in-key alias (`-1` null sentinel) continues to lower.

### Background (verified 2026-07-09 on a fresh binary)

`let x = 0; x ??= 1` prints `1` (node: `0`); numeric param `f(0)` with `p ??= 1` prints `1` (node: `0`). `??=` lowers with `I64Eqz` (literal.rs:571-588) — a falsy test — and `null`/`undefined` both lower to i64 `0` for a scalar (control_flow.rs:1131-1137). kali cannot distinguish `null` from `0` for a scalar; a correct nullish test is unrepresentable without a nullable-scalar type (out of scope). Decision: reject scalar `??=` fail-closed. This changes the accidentally-"working" `let value = null; value ??= 1` (→ 1) to a clean reject.

- [ ] **Step 1: Write the failing reject pins**

Create `crates/kali_cli/tests/nullish_assign_reject.rs` (same `run_source` helper pattern as Task 2). Pins:
```rust
#[test]
fn scalar_local_nullish_assign_rejects() {
    let out = run_source("let x = 0;\nx ??= 1;\nconsole.log(x);\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
}
#[test]
fn numeric_param_nullish_assign_rejects() {
    let out = run_source("function f(p) { p ??= 1; return p; }\nconsole.log(f(0));\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
}
```

- [ ] **Step 2: Run to confirm they fail (currently miscompile to 1)**

Run: `cargo test -p kali_cli --test nullish_assign_reject`
Expected: FAIL (programs currently exit 0 printing `1`).

- [ ] **Step 3: Reject scalar `??=` in resolve**

In `resolve/expression.rs`, in the `AssignmentExpression` arm, when `expr.operator == AssignmentOperator::NullishAssign` and the target is a scalar binding that is **not** a for-in-key alias, push an E5506 diagnostic (message: "nullish assignment on binding '{name}' is unavailable: null and 0 are indistinguishable for a scalar value; only a for-in-key alias with a null sentinel supports `??=`"). Gate the exception on the same for-in-key-alias recognition codegen uses (`for_in_key_aliases`); if resolve lacks that set, gate narrowly on the binding being a for-in-key alias binding — confirm the recognizer during Step 3 and mirror it. Everything else scalar → reject.

- [ ] **Step 4: Update the codegen lowering tests**

`assert_nullish_assignment_lowers("let value = null; ((value)) ??= 1; ...")` (literal_tests.rs:101) now rejects at resolve, so it can no longer be a "lowers cleanly" assertion. Replace it with a for-in-key-alias `??=` source that still lowers (the surviving path), or remove the scalar case and keep only a for-in-key case. Keep `assert_nullish_assignment_lowers` itself (test_support.rs:47) — just change its caller's source.

- [ ] **Step 5: Run pins + full gate**

Run: `cargo test -p kali_cli --test nullish_assign_reject` (2/2 PASS), then `cargo test -p kali_codegen` (the updated lowering test), then the full gate + fmt + clippy. Re-run both reproducers on the fresh binary.
Expected: `x ??= 1` and `p ??= 1` → E5506; for-in-key `??=` still lowers; five CLBG fixtures byte-identical.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/resolve/expression.rs crates/kali_codegen/src/emit/literal_tests.rs crates/kali_codegen/src/test_support.rs crates/kali_cli/tests/nullish_assign_reject.rs
git commit -m "fix(types): reject scalar ??= as unrepresentable-nullish, keep for-in-key sentinel (fasta Spec 7 Task 3)"
```

---

## Task 4: Per-line reclamation — arena-variant string producers + per-call-site escape analysis

This is the headline and the largest task. It is split into five sub-tasks (4a–4e) so each has an independently reviewable, testable deliverable. Sub-tasks 4a–4b add the analysis machinery (behavior-neutral until wired); 4c–4d wire codegen; 4e proves bounded peak.

**Design invariants (apply to every sub-task):**
- **Fail-closed to global.** Any string site not *positively* proven iteration-local keeps `__alloc_global`/`STRING_CONCAT_IMPORT_INDEX`. No existing fixture may change routing.
- **Both-sides oracle.** The analysis predicate that records a site as arena-routable and the codegen recognizer that selects the twin must key off the same site identity.
- **Name-collision safety.** The new per-site channel flows through the name-keyed, collision-poisoned `FunctionArenaFacts` pipeline (arena_gate.rs:143-147, 250-256) — a poisoned function must not retain arena-routable string sites.

### Task 4a: New `ArenaTable` string-site channel (kali_common) + plumb, behavior-neutral

**Files:**
- Modify: `crates/kali_common/src/arena_table.rs` — add a per-site set + setter/getter.
- Test: `crates/kali_common/src/arena_table.rs` (inline `#[cfg(test)]`).

**Interfaces:**
- Produces:
  ```rust
  // ArenaTable
  pub fn set_arena_string_site(&mut self, func: &str, site_ordinal: u32);
  pub fn arena_string_site(&self, func: &str, site_ordinal: u32) -> bool; // miss => false (global)
  ```
  Backing field `arena_string_site: BTreeSet<(String, u32)>`, keyed `(function_name, string_site_preorder_ordinal)` — same shape/discipline as the existing `loop_arena` field. `site_ordinal` is the pre-order index of string-producing call/binary nodes within the function body (a new ordinal stream, analogous to `loop_ordinals`).

- [ ] **Step 1: Write the failing unit test**

In `arena_table.rs` `#[cfg(test)]`:
```rust
#[test]
fn arena_string_site_defaults_closed_and_records() {
    let mut t = ArenaTable::default();
    assert!(!t.arena_string_site("f", 0), "miss must fail closed (global)");
    t.set_arena_string_site("f", 2);
    assert!(t.arena_string_site("f", 2));
    assert!(!t.arena_string_site("f", 1));
    assert!(!t.arena_string_site("g", 2));
}
```

- [ ] **Step 2: Run to confirm it fails to compile (methods absent)**

Run: `cargo test -p kali_common arena_string_site_defaults_closed_and_records`
Expected: FAIL — `no method named set_arena_string_site`.

- [ ] **Step 3: Add the field + setter/getter**

Mirror the `loop_arena` triplet exactly (field, `set_*`, `*` getter with `contains`), documenting the fail-closed contract in the module doc.

- [ ] **Step 4: Run the unit test + kali_common gate**

Run: `cargo test -p kali_common`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/arena_table.rs
git commit -m "feat(common): ArenaTable per-string-site channel, fail-closed (fasta Spec 7 Task 4a)"
```

### Task 4b: `classify_value`/`escape_flow` string-site locality → populate the channel

**Files:**
- Modify: `crates/kali_mir/src/analysis/escape_flow.rs` — string-site locality classification.
- Modify: `crates/kali_mir/src/analysis/arena_gate.rs` — collect string sites during the walk, record ordinals, and set `arena_string_site` in `compute_arena_table` from the resolved facts (respecting poisoning).
- Test: `crates/kali_mir/src/analysis/arena_gate_tests.rs` (append).

**Interfaces:**
- Consumes: `FlowSolution::class_may_heap`, the `console.log` recognizer `is_whitelisted_host_method` (arena_gate.rs:30-41), `ValueClass`.
- Produces: after `compute_arena_table`, `arena_string_site(func, ord)` is true for a `.join(...)` call site or string `+` (concat) site whose result is proven iteration-local. A string site is **iteration-local** iff its result flows only into consumers that drop or copy it: a `console.log` argument (whitelisted, dropped) or a `string_concat`/`+` operand (copied). Any other flow (bound to a name outliving the iteration, stored into field/element/global, returned) → not recorded (fail-closed).

### Background

Runtime strings are invisible to the arena analysis today: `classify_value` (escape_flow.rs:380-585) returns `Scalar` for `.join()` and `+`, so they seed no site. This sub-task adds a **parallel** string-site stream: it does **not** reclassify join/`+` as `Heap` (that would perturb `arena_eligible` for object allocations), but records, per string-producing node, whether its result is provably iteration-local. Model the "only consumer is a dropping/copying sink" the way binary-trees modeled outflow: enumerate the escaping shapes and veto each; default-deny.

- [ ] **Step 1: Write the failing analysis pins**

Append to `arena_gate_tests.rs` two cases against the MIR→`compute_arena_table` pipeline (follow the file's existing harness for building a `MirProgram` and calling `compute_arena_table`):
```rust
// A join whose result is dropped into console.log inside a loop is
// iteration-local → recorded as an arena string site.
#[test]
fn join_into_console_log_is_arena_string_site() {
    let table = arena_table_for(
        "function r(a){ while (a.length > 0) { console.log(a.join(\"\")); a = new Array(0); } }"
    );
    // ordinal of the single join site in `r` is 0 (first string-producing node)
    assert!(table.arena_string_site("r", 0));
}
// A join whose result is RETURNED escapes → NOT recorded (fail-closed global).
#[test]
fn returned_join_is_not_arena_string_site() {
    let table = arena_table_for("function r(a){ return a.join(\"\"); }");
    assert!(!table.arena_string_site("r", 0));
}
```
(Use/introduce an `arena_table_for(src) -> ArenaTable` helper if the file lacks one — mirror the existing test setup that lowers source to MIR.)

- [ ] **Step 2: Run to confirm failure**

Run: `cargo test -p kali_mir join_into_console_log_is_arena_string_site returned_join_is_not_arena_string_site`
Expected: FAIL — sites not recorded (both return false).

- [ ] **Step 3: Implement string-site collection + locality**

In `arena_gate.rs`, during the ownership walk, assign a pre-order ordinal to each string-producing node (`CallExpr` with `.join` member callee; `BinaryExpr` `+` with a string operand — reuse the `is_string_valued`/join recognizers). For each, push a deferred `StringSiteIf { function, ordinal, class }` (mirror `ReturnedSiteIf`/`push_returned_site`, escape_flow.rs:257-270) carrying the result's outflow classification. In `escape_flow.rs`, classify the site's locality: local iff every consumer is a dropping sink (console.log arg — via `is_whitelisted_host_method`) or a copying operand (`+`/concat). In `into_facts`/`compute_arena_table`, set `arena_string_site(func, ord)` only when the deferred class resolves iteration-local AND the function is not poisoned. Apply the veto-every-shape discipline: returned, stored (member/element/global assign target), or bound-beyond-iteration → not recorded.

- [ ] **Step 4: Run the analysis pins + kali_mir suite**

Run: `cargo test -p kali_mir`
Expected: PASS — new pins green, existing arena_gate tests unchanged (this sub-task is behavior-neutral for codegen — nothing reads `arena_string_site` yet).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir/src/analysis/escape_flow.rs crates/kali_mir/src/analysis/arena_gate.rs crates/kali_mir/src/analysis/arena_gate_tests.rs
git commit -m "feat(mir): per-string-site iteration-locality analysis, fail-closed (fasta Spec 7 Task 4b)"
```

### Task 4c: `__join_arena` twin + per-site selection at `emit_runtime_join`

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` — refactor `emit_join_body` to take an `alloc_index: u32`; register `__join_arena` synthetic (name in `SYNTHETIC_FUNCTIONS`, placeholder push, signature dedup, local-count, dispatch arm passing the `__alloc` index).
- Modify: `crates/kali_codegen/src/emit/call.rs:2615-2646` (`emit_runtime_join`) — select `join_arena_fn_index()` vs `join_fn_index()` per site.
- Modify: `crates/kali_codegen/src/emitter.rs` — add `join_arena_fn_index()`; expose the current function's string-site ordinal stream (mirror `loop_ordinals`).
- Test: `crates/kali_cli/tests/clbg_fasta_runtime.rs` (bounded-peak proof lands in 4e; a codegen validation test here).

**Interfaces:**
- Consumes: `ArenaTable::arena_string_site` (4a), the per-function string-site ordinals (new, analogous to `loop_ordinals`), `self.functions["__alloc"]`.
- Produces: `emit_runtime_join` emits `Call(__join_arena)` iff `arena_string_site(&self.function_name, site_ord)` for this join node; else `Call(__join)` (unchanged). `__join_arena` is identical to `__join` but allocates via `__alloc`.

### Background

`emit_join_body(func, alloc_global_index)` (lower.rs:3238) hard-wires the allocator at its single `Call(alloc_index)` (lower.rs:3324). Refactor is trivial: rename the param to `alloc_index` and pass `function_name_to_index["__alloc_global"]` for `__join` and `["__alloc"]` for `__join_arena` in the dispatch (lower.rs:694-708). `__join_arena` reuses the exact `(i64,i64)->i64` signature (dedup) and 6-i64 local count. Emit it unconditionally alongside `__join` (both are tiny; DCE is not required — but if module-size matters, gate its push on any function having an `arena_string_site` join — defer that optimization).

- [ ] **Step 1: Write the failing test — a join-in-loop routes to the arena twin**

Add a codegen-level assertion (in a new `crates/kali_codegen/src/emit/reclamation_tests.rs` or an existing emit test module) that a `while`-loop `console.log(a.join(""))` in an arena-eligible function emits a `Call` to the `__join_arena` index (assert the emitted wasm contains a call to the function exported as `__join_arena`, or use the existing printed-LIR/wasm inspection harness). If a direct index assertion is impractical, assert via 4e's bounded-peak runtime test and make this step a wasm-validates check. Prefer a real routing assertion.

- [ ] **Step 2: Run to confirm failure (twin absent)**

Run: `cargo test -p kali_codegen reclamation`
Expected: FAIL — `__join_arena` not a known function.

- [ ] **Step 3: Add the twin + refactor + per-site selection**

Refactor `emit_join_body` signature to `(func, alloc_index)`. Add `"__join_arena"` to `SYNTHETIC_FUNCTIONS`; push its `FunctionPlan` after `__join` (params `["arr","sep"]`, `result: true`); add signature dedup (`(i64,i64)->i64`), local-count (`local_decls.push((6, ValType::I64))`), and dispatch `"__join_arena" => emit_join_body(&mut body, function_name_to_index["__alloc"])`. Add `join_arena_fn_index(&self)` to emitter.rs. In `emit_runtime_join`, compute the join node's string-site ordinal and select the twin iff `self.arena_table.arena_string_site(&self.function_name, ord)`.

- [ ] **Step 4: Run codegen tests + regression**

Run: `cargo test -p kali_codegen` then full gate. Confirm the five CLBG fixtures byte-identical (none use `.join` in an arena-routable spot except fasta) and fasta 2/2.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src
git commit -m "feat(codegen): __join_arena twin + per-site arena routing for join (fasta Spec 7 Task 4c)"
```

### Task 4d: `string_concat_arena` host import + per-site selection

**Files:**
- Modify: `crates/kali_runtime/src/host/memory.rs:137-179` — add `alloc_guest_string_current` (looks up `caller.get_export("__alloc")` instead of `"__alloc_global"`; identical fallback/write).
- Modify: `crates/kali_runtime/src/host/imports_default.rs:647-657` — register a sibling `string_concat_arena` import calling the current-arena helper.
- Modify: the four `kali:rt` JS import lists — `crates/kali_runtime/src/browser/harness.rs:205` & `:545`, `crates/kali_cli/src/bin/cmd_build.rs:1558` (ESM) & `:1823` (CJS) — add `string_concat_arena` + an `allocGuestStringCurrent` variant that reads `instance.exports.__alloc` (bind `wasmAllocCurrent` beside each `wasmAllocGlobal = …__alloc_global` at harness.rs:306/:646, cmd_build.rs:1675/:1785/:1940/:2050).
- Modify: `crates/kali_codegen/src/lib.rs` + `crates/kali_codegen/src/emit/operators.rs:1214-1227` and `emit/literal.rs:637` — add a `STRING_CONCAT_ARENA_IMPORT_INDEX`, register the import, and select it per site.
- Test: `crates/kali_cli/tests/clbg_fasta_runtime.rs` (bounded-peak covers this in 4e) + a browser import-sync smoke.

**Interfaces:**
- Consumes: `ArenaTable::arena_string_site` for the `+`/concat node's ordinal; `caller.get_export("__alloc")` (already exported — verified at lower.rs:516-521).
- Produces: codegen emits `Call(STRING_CONCAT_ARENA_IMPORT_INDEX)` iff the concat node is an `arena_string_site`; else `Call(STRING_CONCAT_IMPORT_INDEX)` (unchanged). The arena import allocates the concat result into the current arena.

### Background

`string_concat` (imports_default.rs:647) decodes both handles and calls `alloc_guest_string` (memory.rs:137), which looks up `__alloc_global`. The arena variant is a copy that looks up `__alloc`. Adding a host import shifts **no** existing wasm import index if appended after current imports — but the four JS lists must all gain the entry (LinkError otherwise; `kali-browser-harness-import-sync` memory). `__alloc` is already exported, and the g1/g2/g3 trio is live during the host call mid-iteration, so the exported `__alloc` bumps the current iteration arena.

- [ ] **Step 1: Write the failing test**

Add a `string_concat_arena`-routing assertion analogous to 4c (a `while`-loop `console.log(x + y)` on string operands in an arena function selects the arena import), plus keep the existing browser import-sync test green (it will fail to link if a JS list is missed). If a routing assertion is impractical, rely on 4e + a wasm-validates check, but prefer a real assertion.

- [ ] **Step 2: Run to confirm failure**

Run: `cargo test -p kali_codegen reclamation` and `cargo test -p kali_cli` (import-sync)
Expected: FAIL — arena import absent.

- [ ] **Step 3: Implement the host helper + import + JS lists + codegen selection**

Add `alloc_guest_string_current` (memory.rs) as a copy of `alloc_guest_string` with `"__alloc"`. Register `string_concat_arena` (imports_default.rs). Add the entry + `allocGuestStringCurrent` + `wasmAllocCurrent` binding to all four JS lists. Add `STRING_CONCAT_ARENA_IMPORT_INDEX` (lib.rs) and register the import at the matching index; select it in operators.rs:1222 and literal.rs:637 per the concat node's `arena_string_site`.

- [ ] **Step 4: Run codegen + runtime + browser gate**

Run: `cargo test -p kali_codegen -p kali_cli`, then the browser smoke if available (`mise run browser-smoke` per the `browser-cdp-smoke-driver` memory — infra flakes possible; re-run a flaked chromium launch). Full gate + fmt + clippy.
Expected: PASS — five CLBG fixtures byte-identical; fasta 2/2; browser links (all four JS lists synced).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_runtime/src crates/kali_cli/src/bin/cmd_build.rs crates/kali_codegen/src
git commit -m "feat(runtime,codegen): string_concat_arena current-arena concat + per-site routing (fasta Spec 7 Task 4d)"
```

### Task 4e: Bounded-peak unit test (the reclamation proof)

**Files:**
- Test: `crates/kali_cli/tests/reclamation_bounded_peak.rs` (create).
- Create: `crates/kali_cli/tests/fixtures/benchmarks/join-loop-peak.ts` (+ a small sandbox policy) — a minimal `join`/`+`-in-`while` program.

**Interfaces:**
- Consumes: `kali run --sandbox <small-policy>` behavior; the arena routing from 4c/4d.

### Background

The canonical N=25M pin (Task 5) is slow; this is the fast, deterministic proof that peak is O(1) in N. A minimal program joins/concats a fixed-size array per iteration and loops `argv[2]` times, printing each line. Under a **small fixed memory policy**, it must pass at two very different iteration counts — proving reclamation, since without it the larger count exceeds the budget (E4000).

- [ ] **Step 1: Write the fixture + the failing test**

`join-loop-peak.ts`:
```js
var line = new Array(60);
for (var i = 0; i < 60; i++) line[i] = "x";
var n = +process.argv[2];
while (n > 0) { console.log(line.join("") + "!"); n -= 1; }
```
Test asserts `kali run --api node --sandbox <small-policy>` succeeds (exit 0) at both a small and a large iteration count with the **same** small memory budget:
```rust
#[test]
fn join_concat_loop_has_bounded_peak() {
    for n in ["1000", "500000"] {
        let out = run_fixture_with_policy("join-loop-peak.ts", "join-loop-peak.policy.json", n);
        assert!(out.status.success(),
            "N={n} should fit the fixed small budget under reclamation; stderr: {}",
            String::from_utf8_lossy(&out.stderr));
    }
}
```
Size the policy's memory budget so the large N would exceed it *without* reclamation (compute from ~64B/line × N vs the budget) but comfortably fits *with* it (a few KB peak). Document the arithmetic in a comment.

- [ ] **Step 2: Run — before 4c/4d wiring this would fail at large N; after, it passes**

Run: `cargo test -p kali_cli --test reclamation_bounded_peak`
Expected: PASS (4c/4d already merged). Sanity: temporarily raise N or shrink the budget to confirm the test is discriminating (would fail without reclamation), then restore.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/reclamation_bounded_peak.rs crates/kali_cli/tests/fixtures/benchmarks/join-loop-peak.ts crates/kali_cli/tests/fixtures/benchmarks/join-loop-peak.policy.json
git commit -m "test(cli): bounded-peak reclamation proof at two N under one fixed budget (fasta Spec 7 Task 4e)"
```

---

## Task 5: Canonical N=25M pin (replace the N=2M tier)

**Files:**
- Modify: `crates/kali_cli/tests/clbg_fasta_runtime.rs` — replace the N=2,000,000 SHA-256 tier with N=25,000,000; keep the N=8 golden tier.
- Modify: `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json` (or a new N=25M policy) — raise fuel + memory budgets to cover bounded peak + working set.

**Interfaces:**
- Consumes: reclamation from Task 4; the fixture + policy machinery from Spec 6.

### Background

With Task 4 bounding peak, N=25M fits a modest policy. The node reference SHA-256 is `6a26f1c8…`. **Wall-clock risk:** if kali runs N=25M in more than ~30 s, downgrade this tier to `#[ignore]`-by-default (a dedicated CI job runs it) while Task 4e remains the in-gate reclamation proof. Measure in Step 2 and decide.

- [ ] **Step 1: Re-derive the node reference locally**

```bash
node --version   # expect v26.4.0
node crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts 25000000 | sha256sum
```
Expected: `6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee`. If node differs, record the actual hash and use it (note the discrepancy in the commit).

- [ ] **Step 2: Measure kali wall-clock + peak at N=25M**

```bash
cargo build -p kali_cli --bin kali --release
time ./target/release/kali run --api node --sandbox crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts -- 25000000 | sha256sum
```
Confirm the SHA matches node and note wall-clock. If it traps E4000, raise the policy's memory budget until it fits (peak should be small — a large required budget signals a reclamation gap, not a budgeting problem: investigate before inflating). If wall-clock > ~30 s, mark the test `#[ignore]` and add the dedicated-job note.

- [ ] **Step 3: Update the test tier**

Replace the `fasta_large_n_matches_node_sha256` body's `const N: &str = "2000000"` and `NODE_SHA256` with `"25000000"` and `6a26f1c8…`, and point at the (possibly raised) policy. Update the doc comment to state the canonical pin and the bounded-peak dependency on Task 4. Keep `fasta_small_n_matches_node_golden` unchanged.

- [ ] **Step 4: Run the fasta tier + full gate**

Run: `cargo test -p kali_cli --test clbg_fasta_runtime` (or with `-- --ignored` if downgraded), then the full 5-crate gate + fmt + clippy.
Expected: N=8 golden + N=25M SHA-256 green; five prior CLBG fixtures byte-identical.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/clbg_fasta_runtime.rs crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json
git commit -m "test(cli): pin canonical fasta N=25,000,000 SHA-256 vs node, replacing the N=2M tier (fasta Spec 7 Task 5)"
```

---

## Integration

After all tasks: final whole-branch review from the merge-base (`main` at branch creation) per the SDD process; fix any Important finding before merge (repo precedent); then push a PR and self-merge per the `kali-integration-convention` memory. CI watch: browser-cdp-smoke chromium-launch and ubuntu "No space left" are known infra flakes — re-run a flaked job before concluding a real failure.

## Self-Review (completed at authoring)

**Spec coverage:** §1 acceptance → Tasks 1–5; §3.1 reclamation → Task 4 (join 4c, concat 4d, analysis 4a/4b, proof 4e); §3.2 laundering → Task 1; §3.3 `o+=1` → Task 2; §3.4 `??=` → Task 3; §4.1 canonical pin → Task 5; §4.2 bounded-peak → Task 4e; §4.3 soundness pins → Tasks 1/2/3; §4.4 regression → every task's gate step; §5 risks (wall-clock, host boundary, fail-open, DCE) → Task 5 Step 2, Task 4d background, Task 4 invariants, Task 4c Step 3. No spec requirement is unmapped.

**Placeholder scan:** the two genuinely-investigation-gated steps (Task 2 Step 2 locus, Task 4b/4c/4d exact analysis wiring) are structured as diagnose-then-implement with the interface, the failing test, and the fail-closed contract fixed — not "TBD". Exact hashes, file:line, gate commands, and full test code are inline.

**Type consistency:** `arena_string_site`/`set_arena_string_site` (4a) are used verbatim in 4b/4c/4d; `join_arena_fn_index`/`__join_arena` (4c) and `STRING_CONCAT_ARENA_IMPORT_INDEX`/`alloc_guest_string_current`/`string_concat_arena` (4d) are named consistently across their tasks; `target_repr_is_one_of` (Tasks 2/3) matches the verified source name.
