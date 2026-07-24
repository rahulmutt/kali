# R-06 read-only var/let object-literal materialization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a read-only `var`/`let` object literal read back its real field values instead of silently reading `0`, by materializing mutable object-literal bindings on field read.

**Architecture:** One types-side change in `crates/kali_types/src/repr_infer.rs`. Record non-`const` object-literal bindings in a new set, then in the object-resolution pass mark such a binding materialized when it is field-read — the exact treatment a write already gives it. Everything downstream (shape interning, `emit_object_allocation`, the fail-closed conflict checks) is existing machinery, reused unchanged. `const` bindings are never in the set, so they keep their byte-identical compile-time fold lowering.

**Tech Stack:** Rust. `kali_types` (repr inference), `kali_codegen` (unchanged this stage), `kali_cli` integration tests. Oracle is `node v26.5.0`. Build the CLI with `cargo build -p kali_cli --bin kali`.

## Global Constraints

- **Objects only.** Arrays are explicitly out of scope (R-06-R3, a separate later stage). Do not touch array/element lowering.
- **`const` must stay fold-first.** The new materialization trigger fires only for `var`/`let` bindings. Folding a mutable binding is the R-07 miscompile — never route `var`/`let` through the fold lane; never route `const` through the new read-materialization.
- **Never silent-0.** A shape the materialized lane cannot store must fail closed with `E5506`, not produce a wrong value. This is inherited from the existing conflict checks — do not add a new deny list.
- **No new crash / no new nonzero-miscompile at escape.** A newly-materialized object that escapes (returned/stored) must behave identically to today's escaping materialized objects (silent-`0` or fail-closed — never a crash or a new wrong nonzero value).
- **Gate:** `cargo test --workspace` (the CI command) diffed against a `main` worktree must be **0 newly-red**. `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` clean. 6/6 CLBG goldens + web-baseline byte-for-byte unchanged.
- **Standing discipline:** re-run every reproducer on a freshly built binary; full-workspace enumeration is the only gate; finish with an adversarial whole-stage review.

---

## File Structure

- `crates/kali_types/src/repr_infer.rs` — **modify.** Add one `BTreeSet<ObjSlot>` field, one insertion site in `visit_declarator_init`, one read-materialization block in `resolve_objects`.
- `crates/kali_cli/tests/soundness_r06_object_init.rs` — **create.** Green pins (read-only var/let objects), fail-closed pins (nested/unknown field), residual guard pins (escape/reassignment).
- `docs/superpowers/followups/kali-silent-miscompile-register.md` — **modify.** Mark R-06 objects-half closed; record R-06-R1/R2/R3 residuals.
- Possible fixture/census re-pins surfaced by the gate (Task 3) — files TBD by the enumeration diff, re-pinned to the now-correct node value or `E5506`.

---

### Task 1: Core fix — materialize read-only mutable object literals (TDD)

**Files:**
- Create: `crates/kali_cli/tests/soundness_r06_object_init.rs`
- Modify: `crates/kali_types/src/repr_infer.rs` (struct field ~after line 476; `visit_declarator_init` object arm at lines 2619–2632; `resolve_objects` after the write-materialization block at lines 4330–4338)

**Interfaces:**
- Consumes: existing `ObjSlot::Binding(String, String)` (`repr_infer.rs:627`), `obj_materialized: BTreeSet<ObjSlot>` (`repr_infer.rs:397`), `ObjAccess { base, field, other, is_write }` (`repr_infer.rs:640`), the local `fields_of` in `resolve_objects`, `emit_object_allocation` / `Repr::Object` codegen at `control_flow.rs:1421` (unchanged).
- Produces: new private field `mutable_object_literal_bindings: BTreeSet<ObjSlot>` on `ReprInfer`, populated in `visit_declarator_init`, consumed only inside `resolve_objects`. No public/`ReprTable` surface change.

- [ ] **Step 1: Write the failing test file**

Create `crates/kali_cli/tests/soundness_r06_object_init.rs`:

```rust
// R-06 — read-only var/let object-literal materialization soundness pins.
// A read-only mutable object literal must read back its real field values
// (materialized allocation), not the silent-0 fold fallback. Shapes the
// materialized lane cannot store fail closed with E5506, never silent-0.
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

fn kali_bin() -> String {
    env!("CARGO_BIN_EXE_kali").to_string()
}

fn run(source: &str) -> Output {
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

/// Compile+run, assert success, return trimmed stdout.
fn run_ok(source: &str) -> String {
    let out = run(source);
    assert!(
        out.status.success(),
        "expected success\nsource: {source}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compile+run, assert fail-closed with E5506.
fn run_e5506(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed E5506\nsource: {source}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    stderr
}

// ---- Green pins: read-only mutable object literals materialize correctly ----

#[test]
fn var_object_numeric_field_reads_value() {
    assert_eq!(run_ok("var o = { f: 7 }; console.log(o.f);"), "7");
}

#[test]
fn let_object_numeric_field_reads_value() {
    assert_eq!(run_ok("let o = { f: 7 }; console.log(o.f);"), "7");
}

#[test]
fn var_object_string_field_reads_value() {
    assert_eq!(run_ok("var o = { f: \"hi\" }; console.log(o.f);"), "hi");
}

#[test]
fn var_object_bool_field_reads_value() {
    assert_eq!(run_ok("var o = { f: true }; console.log(o.f);"), "true");
}

#[test]
fn var_object_multi_field_reads_all() {
    assert_eq!(
        run_ok("var o = { a: 1, b: 2, c: 3 }; console.log(o.a + o.b + o.c);"),
        "6"
    );
}

#[test]
fn var_object_mixed_fields_read() {
    assert_eq!(
        run_ok("var o = { n: 7, s: \"hi\" }; console.log(o.n, o.s);"),
        "7 hi"
    );
}

#[test]
fn var_object_function_scope_reads_value() {
    assert_eq!(
        run_ok("function h(){ var o = { f: 7 }; return o.f; } console.log(h());"),
        "7"
    );
}

#[test]
fn const_object_still_folds() {
    // Regression guard: const stays fold-first, unchanged.
    assert_eq!(run_ok("const o = { f: 7 }; console.log(o.f);"), "7");
}

// ---- Fail-closed pins: shapes the materialized lane cannot store ----

#[test]
fn var_object_nested_field_fails_closed() {
    // Nested-object field: E5506, not silent-0.
    run_e5506("var o = { inner: { x: 1 } }; console.log(o.inner.x);");
}

#[test]
fn var_object_unknown_field_fails_closed() {
    // Unknown field on a materialized read-only object: E5506 (kali has no
    // `undefined`; honest over-deny beats today's silent-0).
    run_e5506("var o = { f: 7 }; console.log(o.zzz);");
}
```

- [ ] **Step 2: Build the binary and run the test to verify it fails**

Run:
```bash
cargo build -p kali_cli --bin kali
cargo test -p kali_cli --test soundness_r06_object_init
```
Expected: the green pins (`var_object_*`, `let_object_*`) FAIL — kali prints `0`/`0 0` instead of the value. The two fail-closed pins also FAIL today (`var o={f:7}; o.zzz` currently exits 0 with `0`). `const_object_still_folds` PASSES. This is the RED state — do not commit yet.

- [ ] **Step 3: Add the `mutable_object_literal_bindings` field**

In `crates/kali_types/src/repr_infer.rs`, immediately after the `object_initialized_bindings` field (ends at line 476), add:

```rust
    /// `(func, binding)` object-literal bindings declared with a MUTABLE kind
    /// (`var`/`let`, never `const`), stored as their `ObjSlot::Binding`. A
    /// read-only mutable object literal cannot use the const-only compile-time
    /// fold lane (folding a mutable binding is the R-07 miscompile), so a field
    /// READ must materialize it into a real `Repr::Object` allocation — else the
    /// read falls to the silent-0 fold fallback (R-06). Populated in
    /// `visit_declarator_init`; consumed by the read-materialization block in
    /// `resolve_objects`. `const` bindings are absent, so they keep their
    /// byte-identical fold-first lowering.
    mutable_object_literal_bindings: BTreeSet<ObjSlot>,
```

(The struct derives `Default`, so a new `BTreeSet` field needs no other change.)

- [ ] **Step 4: Record mutable object-literal bindings in `visit_declarator_init`**

In `repr_infer.rs`, the object-literal arm of `visit_declarator_init` currently reads (lines 2619–2632):

```rust
        if let Expression::ObjectExpression(obj) = init {
            // Syntactic taint, independent of materialization — see the field
            // doc on `object_initialized_bindings`. A compound/update on `id`
            // must reject even when the object literal is never field-read and
            // so never gets promoted to `Repr::Object` below.
            self.object_initialized_bindings
                .insert((func.to_string(), id.to_string()));
            self.record_object_literal(
                func,
                ObjSlot::Binding(func.to_string(), id.to_string()),
                obj,
            );
            return;
        }
```

Insert the mutable-binding record between the `object_initialized_bindings` insert and `record_object_literal`:

```rust
        if let Expression::ObjectExpression(obj) = init {
            // Syntactic taint, independent of materialization — see the field
            // doc on `object_initialized_bindings`. A compound/update on `id`
            // must reject even when the object literal is never field-read and
            // so never gets promoted to `Repr::Object` below.
            self.object_initialized_bindings
                .insert((func.to_string(), id.to_string()));
            // R-06: a var/let (mutable) object-literal binding must materialize
            // on a field READ (const stays fold-first). Recorded here; the
            // read-materialization block in `resolve_objects` consumes it.
            if kind != "const" {
                self.mutable_object_literal_bindings
                    .insert(ObjSlot::Binding(func.to_string(), id.to_string()));
            }
            self.record_object_literal(
                func,
                ObjSlot::Binding(func.to_string(), id.to_string()),
                obj,
            );
            return;
        }
```

- [ ] **Step 5: Add the read-materialization block in `resolve_objects`**

In `repr_infer.rs`, immediately after the write-materialization block that ends at line 4338 (the block that begins `// 2.6. Pre-mark materialization for every WRITE access…`) and before the `// 3. Wire deferred member accesses…` comment at line 4340, insert:

```rust
        // 2.7. Read-materialize a MUTABLE object-literal binding (R-06). A
        //      var/let object literal is absent from the const-only fold table,
        //      so a field READ has no compile-time value to fold and must get a
        //      real Repr::Object allocation. Marking it materialized here means a
        //      known-field read lowers through the allocation (step 3's read arm
        //      + shape intern), and an UNKNOWN-field read promotes to a
        //      fail-closed conflict (step 3, ~line 4352) instead of the silent-0
        //      fold fallback. `const` bindings are absent from the set and keep
        //      their byte-identical fold-first lowering.
        for access in &self.obj_accesses {
            if !access.is_write
                && self.mutable_object_literal_bindings.contains(&access.base)
                && fields_of.contains_key(&access.base)
            {
                self.obj_materialized.insert(access.base.clone());
            }
        }
```

(This mirrors the write block's disjoint-field-borrow shape exactly — iterating `&self.obj_accesses` while inserting into `self.obj_materialized` compiles, as the existing block at 4330 proves.)

- [ ] **Step 6: Rebuild and run the soundness test to verify GREEN**

Run:
```bash
cargo build -p kali_cli --bin kali
cargo test -p kali_cli --test soundness_r06_object_init
```
Expected: ALL pins PASS. If `var_object_string_field_reads_value` FAILS (the read-only string-field materialization differs from the already-working write case), that is the one anticipated risk: change that pin from `run_ok` to `run_e5506` (string fields fail closed rather than silent-0 — still correct per the design's fail-closed fallback) and record it as residual R-06-R4 in Task 4. Do NOT force a silent value.

- [ ] **Step 7: fmt + clippy**

Run:
```bash
cargo fmt
cargo clippy -p kali_types --all-targets -- -D warnings
```
Expected: no diff from fmt beyond the new code; clippy clean.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/soundness_r06_object_init.rs
git commit -m "fix(types): R-06 — materialize read-only var/let object literals on field read"
```

---

### Task 2: Escape and reassignment residual guards

**Files:**
- Modify: `crates/kali_cli/tests/soundness_r06_object_init.rs`

**Interfaces:**
- Consumes: the `run` helper from Task 1.
- Produces: guard tests asserting the out-of-scope shapes are no-worse (never a crash, never a new nonzero-wrong value).

- [ ] **Step 1: Write the residual guard tests**

Append to `crates/kali_cli/tests/soundness_r06_object_init.rs`:

```rust
// ---- Residual guards (out of scope): must be NO WORSE than main. Each may
//      stay silent-0 or fail closed, but must never crash and never produce a
//      NEW nonzero-wrong value. ----

/// A newly-materialized object that ESCAPES via return then a member-on-call
/// read (R-06-R1 / R-14). Today: silent-0. Guard: exit 0 with "0", OR a
/// fail-closed diagnostic — never a crash, never a nonzero-wrong value.
#[test]
fn returned_object_member_read_no_worse() {
    let out = run("function h(){ var o = { f: 7 }; return o; } console.log(h().f);");
    if out.status.success() {
        // May not print node's "7" yet (R-14 escape is a later stage), but it
        // must not print a WRONG NONZERO value. Silent-0 is the tolerated state.
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert!(
            stdout == "0" || stdout == "7",
            "returned-object read produced a new nonzero-wrong value: {stdout:?}"
        );
    }
    // A non-success exit (fail-closed) is also acceptable — the only forbidden
    // outcome is a silent NONZERO-wrong value, guarded above.
}

/// Whole-object reassignment to an object literal (R-06-R2), a distinct store
/// mechanism from the declarator init. Today: the reassigned read is silent-0.
/// Guard: no crash, no new nonzero-wrong value.
#[test]
fn object_literal_reassignment_no_worse() {
    let out = run("var o = { f: 1 }; console.log(o.f); o = { f: 2 }; console.log(o.f);");
    if out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        // First read is correct (1); the reassigned read is the residual.
        assert!(
            stdout == "1\n0" || stdout == "1\n2",
            "reassignment read produced a new nonzero-wrong value: {stdout:?}"
        );
    }
}
```

- [ ] **Step 2: Run the guard tests**

Run:
```bash
cargo test -p kali_cli --test soundness_r06_object_init
```
Expected: ALL pass. If `returned_object_member_read_no_worse` or `object_literal_reassignment_no_worse` FAILS with a crash (`E4201`/`unreachable`) or a nonzero-wrong value, STOP: the fix introduced an escape regression. Route the escaping/reassigned shape to a fail-closed `E5506` (add the base to the conflict path when it is a `mutable_object_literal_bindings` slot reached by an escaping flow) rather than admitting it, and re-run. Record the outcome in Task 4.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/soundness_r06_object_init.rs
git commit -m "test(cli): R-06 — escape + reassignment residual no-worse guards"
```

---

### Task 3: Full-workspace gate vs main + fixture re-pins

**Files:**
- Modify: any fixture/census/soundness test files the enumeration diff flags (paths determined by the run; each re-pinned to the now-correct node value or `E5506`).

**Interfaces:**
- Consumes: a clean `main` worktree for the baseline.
- Produces: a 0-newly-red `cargo test --workspace` on the branch.

- [ ] **Step 1: Build a main baseline worktree and capture its failing set**

Run:
```bash
git worktree add /tmp/kali-main-baseline main
( cd /tmp/kali-main-baseline && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r06-main-baseline.log | grep -E '^test .* \.\.\. FAILED' | sort > /tmp/r06-main-fails.txt )
wc -l /tmp/r06-main-fails.txt
```
Expected: the `main` failing set (the honest baseline — non-empty is fine; what matters is the diff). Uses `--no-fail-fast` so enumeration is complete.

- [ ] **Step 2: Capture the branch failing set**

Run:
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r06-branch.log | grep -E '^test .* \.\.\. FAILED' | sort > /tmp/r06-branch-fails.txt
comm -13 /tmp/r06-main-fails.txt /tmp/r06-branch-fails.txt > /tmp/r06-newly-red.txt
echo "=== NEWLY RED (must be re-pins to correct behavior, not regressions) ==="; cat /tmp/r06-newly-red.txt
```
Expected: `/tmp/r06-newly-red.txt` lists only tests that assert the OLD silent-0 (or old `undefined`-as-0) behavior for a read-only var/let object — each is a stale pin now made correct, not a product regression.

- [ ] **Step 3: Triage and re-pin each newly-red test**

For each test in `/tmp/r06-newly-red.txt`: open it, confirm its expectation encoded the old silent-0/wrong value for a read-only mutable object, run the same source on `node v26.5.0` and on the freshly built `kali` to get the now-correct value, and update the assertion to the node-correct value (or to `E5506` where the shape fails closed). If any newly-red test is NOT explained by this change (a genuine regression), STOP and diagnose with superpowers:systematic-debugging before proceeding. Re-pin census tests (`count_tag_boxing_ops` allowlists, `int_to_string`/substring counts) additively per the established `string_tests/lookup.rs` procedure only if the diff shows a new synthetic — this change adds none, so expect none.

- [ ] **Step 4: Re-run the gate to confirm 0 newly-red**

Run:
```bash
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sort > /tmp/r06-branch-fails2.txt
comm -13 /tmp/r06-main-fails.txt /tmp/r06-branch-fails2.txt
```
Expected: empty output (0 newly-red).

- [ ] **Step 5: fmt + clippy + goldens**

Run:
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: clean. (CLBG goldens + web-baseline byte-for-byte are covered inside `cargo test --workspace`; confirm none appear in the newly-red diff.)

- [ ] **Step 6: Clean up the baseline worktree and commit any re-pins**

```bash
git worktree remove /tmp/kali-main-baseline
git add -A
git commit -m "test: R-06 — re-pin stale silent-0 object-read expectations to node-correct"
```
(If Step 3 produced no re-pins, skip the commit.)

---

### Task 4: Register + memory update, finalize

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`
- Create/Modify: memory files under `/home/dev/.claude/projects/-workspace/memory/`

- [ ] **Step 1: Update the register**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`, update the R-06 entry (line ~270) with a STATUS line: objects-half CLOSED 2026-07-24 via read-materialization of mutable object-literal bindings (`repr_infer.rs` `mutable_object_literal_bindings` + step 2.7), with nested/unknown fields inheriting the existing `E5506` conflict checks. Record residuals: **R-06-R1** returned/escaping objects (R-14), **R-06-R2** whole-object reassignment, **R-06-R3** arrays (own later stage), and **R-06-R4** string-field read-only objects IF Task 1 Step 6 routed them fail-closed. Note that this falsifies the register's §3 G7 inference ("R-06 falls out of the R-07 fix"): R-07 is fixed and R-06 still reproduced, so R-06 was independent.

- [ ] **Step 2: Write the stage memory**

Create `/home/dev/.claude/projects/-workspace/memory/kali-r06-object-init.md` (frontmatter `type: project`) summarizing: SHIPPED 2026-07-24, the fold-vs-materialize gap root cause, the one-change fix (record mutable object-literal bindings → materialize on read; const stays fold-first), the free fail-closed fallback (inherited conflict checks), the residuals R-06-R1..R4, and the headline lesson (the register was stale — most Tier-1 entries were already closed or fail-closed; measurement on a fresh binary falsified the premise before any design). Add a one-line pointer to `MEMORY.md`.

- [ ] **Step 3: Adversarial whole-stage review**

Dispatch a review (superpowers:requesting-code-review or a fresh subagent) over the whole branch diff, specifically hunting for: a store-site or value-sink that still silent-0s a newly-materialized mutable object; an escape shape that produces a new crash/nonzero-miscompile; a `const` binding accidentally routed through the new path (fold-first regression). Address findings, re-run the Task 3 gate, then commit.

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "docs(register): R-06 objects-half closed + residuals R-06-R1..R4"
```

- [ ] **Step 4: Push branch and open PR (per kali-integration-convention)**

```bash
git push -u origin r06-object-init-materialization
gh pr create --fill --title "R-06 — read-only var/let object-literal materialization"
```
Merge per the standing convention once the review and gate are green.

---

## Self-Review

**1. Spec coverage** — every spec section maps to a task:
- §1 problem / §2 root cause → Task 1 (the fix + green pins reproduce the exact §1 probe table).
- §3 the fix (record mutable set + read-materialize) → Task 1 Steps 3–5, exact code and sites.
- §4 fail-closed fallback is free → Task 1 fail-closed pins (nested/unknown).
- §5 scope boundary / residuals → Task 2 guards + Task 4 register residuals.
- §6 escape risk → Task 2 guards + Task 4 review, with the fail-closed escape hatch.
- §7 testing & gate → Task 3 (full-workspace vs main, re-pins, fmt/clippy/goldens).
- §8 interfaces → Task 1 Interfaces block; register/memory in Task 4.

**2. Placeholder scan** — no TBD/TODO in code steps; every code step shows complete code. The only deferred detail is which fixtures the gate flags (Task 3), which is inherently run-determined and handled by an explicit triage procedure, not a placeholder value.

**3. Type consistency** — `ObjSlot::Binding(String, String)`, `obj_materialized: BTreeSet<ObjSlot>`, `ObjAccess { base, field, other, is_write }`, and the new `mutable_object_literal_bindings: BTreeSet<ObjSlot>` are used identically across Steps 3–5. `run` / `run_ok` / `run_e5506` helper names match between Task 1 and Task 2.
