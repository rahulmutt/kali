# Stage B — Array-callback nested-function silent miscompile: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Root-cause and fix the silent wrong-answer produced when a named `FunctionDeclaration` nested inside a real function body contains 2+ array-callback `for-of` loops — so the case computes correctly or fails closed (E5506), never silently.

**Architecture:** Investigation-first (systematic-debugging, then fix). The arena/loop machinery is keyed by `(function_name, loop_ordinal)`; loop ordinals are computed **per-function-body** in two independent places — codegen's `loop_preorder_ordinals` (`kali_codegen`) and `kali_mir`'s `arena_gate` `next_loop_ordinal` stream. The lead hypothesis is that these two streams disagree on how a **nested** function's loops are numbered, so the `(function_name, ordinal)` keys misalign (needs ≥2 loops to collide). Confirm or refute that, then align the two streams (correct lowering) or, if a construct genuinely cannot be lowered under the region-only model, fail closed.

**Tech Stack:** Rust (kali_codegen / kali_mir / kali_common / kali_cli), wasm, node (differential oracle).

**Spec:** `docs/superpowers/specs/2026-07-15-blockarrow-prereqs-design.md` §3 Stage B.

## Global Constraints

- **Branch:** `soundness-batch1-pra` · **Baseline commit:** `192984c39` · **Frozen failure baseline: 731.**
- **Develop on the CLEAN branch** with plain `function(){}` fixtures — the bug reproduces without the un-flatten patch. Do NOT apply `task5-block-arrows-WIP.patch`.
- **Reject-don't-miscompile:** the fix makes the case correct OR fails closed E5506 — a silent wrong answer is never an acceptable end state.
- **GC-less:** region/escape reclamation only; never propose a tracing/copying GC.
- **PRIMARY GATE — zero newly-red:** `comm -13 <baseline> <post>` over a full `cargo test --workspace --no-fail-fast` enumeration must print nothing. Cross-check against a `main` worktree (`/workspace/.worktrees/kali-main`).
- **Enumeration uses `sort -u`, never plain `sort`** — 18 test names live in two harness binaries each; raw `sort` fabricates newly-red.
- A full workspace run exceeds one command timeout — run it **detached with a `.done` marker and poll a bounded blocking loop**; do not rely on background-task notifications.
- **Re-run every claim on a freshly built binary** (`cargo build -p kali_cli`). Fix/status reports are unreliable in this repo.
- **Fixture footguns:** never `String(<bigint>)` (folds to `0`); never bind a call's result to a `const` (evaluates `uses + 1` times — use `let`).
- **No `_ =>` arm** in any census/walk you add; every no-op arm cites `file:line`.
- `$SCRATCH` = `/tmp/claude-1000/-workspace/d221ba43-684d-49e2-9cd1-4b5dba1ed267/scratchpad`. Canonical baseline: `$SCRATCH/stage6-pre.txt` (731 lines, from Stage 6 Task 1). If absent, regenerate it per Task 1 Step 1 and STOP if it is not 731.

---

## Task 1: Pin the minimal repro + characterize the failure boundary

**Files:**
- Create: `docs/superpowers/followups/stageB-array-callback-triage.md`

**Interfaces:**
- Produces: `$SCRATCH/stageB-pre.txt` — the canonical sorted baseline (**731**), consumed by every later gate. (Copy from `$SCRATCH/stage6-pre.txt` if present and verified 731; otherwise regenerate.)
- Produces: the minimal repro JS, recorded in the triage doc, consumed by Tasks 2–4.

- [ ] **Step 1: Confirm/regenerate the 731 baseline on a fresh binary**

```bash
cd /workspace && cargo build -p kali_cli
if [ -f "$SCRATCH/stage6-pre.txt" ] && [ "$(wc -l < "$SCRATCH/stage6-pre.txt")" = "731" ]; then
  cp "$SCRATCH/stage6-pre.txt" "$SCRATCH/stageB-pre.txt"
else
  nohup bash -c 'cd /workspace && cargo test --workspace --no-fail-fast > "$SCRATCH/b1.log" 2>&1; echo DONE > "$SCRATCH/b1.done"' >/dev/null 2>&1 &
  # then, in a SEPARATE call (timeout 600000): for i in $(seq 1 590); do [ -f "$SCRATCH/b1.done" ] && break; sleep 1; done
  grep -E '^test .* \.\.\. FAILED' "$SCRATCH/b1.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort -u > "$SCRATCH/stageB-pre.txt"
fi
wc -l "$SCRATCH/stageB-pre.txt"   # expect 731
```
STOP and reconcile if it is not 731.

- [ ] **Step 2: Reproduce the bug on the clean branch (no patch)**

Write `$SCRATCH/repro.js`:
```js
function outer() {
  function inner() {
    let out = [];
    for (const x of [1, 2].map(v => v)) out.push(x);
    for (const y of [3, 4].map(v => v)) out.push(y);
    console.log(out.join(","));
  }
  inner();
}
outer();
```
Run:
```bash
BIN=$(find target/debug -maxdepth 1 -name kali -type f)
$BIN run "$SCRATCH/repro.js"; echo "exit=$?"
node "$SCRATCH/repro.js"
```
Expected: kali prints a wrong/short value (`0` or partial), exit 0, no diagnostic; node prints `1,2,3,4`. Record BOTH verbatim in the triage doc. If kali already prints `1,2,3,4`, STOP — the bug does not reproduce on this binary and the plan's premise must be re-checked with the controller.

- [ ] **Step 3: Characterize the failure boundary (bisection matrix)**

Run each variant through `$BIN run` and node; record kali-vs-node for every cell in the triage doc. These pin exactly which structural features are required to trigger the bug:

| # | variant | expect |
|---|---|---|
| a | the repro above (nested named `inner`, **2** array-callback for-of loops) | kali wrong, node `1,2,3,4` |
| b | same but **1** loop only (`out.push(x)` for the first loop; drop the second; `console.log(out.join(","))`) | both `1,2` (works) |
| c | the 2 loops inlined directly in `outer` (no nested `inner`) | both `1,2,3,4` (works) |
| d | the 2 loops in `inner` but `inner` defined+called at **module scope** (no `outer`) | both `1,2,3,4` (works) |
| e | the 2 loops in `inner` nested inside a `function mid(){}` nested inside `outer` (double nesting, no anon fn) | record actual |
| f | the 2 loops as plain `for` (not array-callback `for-of` — `for (let i=0;i<2;i++) out.push(arr[i])`) inside nested `inner` | record actual — isolates whether the trigger is the array-callback source specifically |

Record which cells break. The known-good/known-bad split (a breaks; b, c, d work) localizes the bug to **per-nested-function loop-ordinal numbering with ≥2 array-callback loops**. Cells e/f sharpen it.

- [ ] **Step 4: Commit the triage**

```bash
git add docs/superpowers/followups/stageB-array-callback-triage.md
git commit -m "docs(soundness): stageB triage — minimal repro + failure boundary pinned [stageB]"
```

---

## Task 2: Root-cause to a specific mechanism

**Files:**
- Modify (temporarily, reverted before commit): `crates/kali_codegen/src/lower.rs`, `crates/kali_mir/src/analysis/arena_gate.rs` (debug instrumentation only)
- Modify: `docs/superpowers/followups/stageB-array-callback-triage.md` (record the root cause)

**Interfaces:**
- Consumes: Task 1's minimal repro.
- Produces: a written, evidence-backed root cause naming the exact mechanism and `file:line`, consumed by Task 3.

- [ ] **Step 1: Dump the emitted wasm for the repro and read what each `for-of` binds**

```bash
BIN=$(find target/debug -maxdepth 1 -name kali -type f)
# find the build subcommand that emits .wat/.wasm (check `$BIN --help`); e.g.:
$BIN build "$SCRATCH/repro.js" -o "$SCRATCH/repro.wasm" 2>&1 | tee "$SCRATCH/repro-build.log"
# disassemble (wasm-tools is a dev dep in this repo; if absent, use the project's existing wat-dump path):
wasm-tools print "$SCRATCH/repro.wasm" > "$SCRATCH/repro.wat" 2>/dev/null || echo "use the repo's wat dump"
```
In `repro.wat`, locate the wasm function compiled from `inner` and read how its two loops address their arena / accumulation buffer (`out`). Record in the triage doc whether the two loops write to the **same** buffer slot/arena offset (the collision symptom) or different ones. This is the codegen-side evidence.

- [ ] **Step 2: Compare the two loop-ordinal streams for `inner`**

The hypothesis: codegen's `loop_preorder_ordinals` (`crates/kali_codegen/src/lower.rs:2241`, also `crates/kali_codegen/src/emitter.rs:234`) and `kali_mir`'s `arena_gate` `next_loop_ordinal` stream (`crates/kali_mir/src/analysis/arena_gate.rs:787-789`, with per-function save/restore at `:197,:329,:375`) disagree on the loop-ordinal numbering for a **nested** function — so the `(function_name, ordinal)` keys used by `arena_table.loop_arena(function_name, ordinal)` / `opens_arena(function_name)` (`crates/kali_codegen/src/lower.rs:2252-2266`) misalign.

Add temporary `eprintln!` instrumentation at both ordinal-assignment sites, printing `(function_name, node_id, ordinal)` for every loop. Rebuild, run the repro, and capture both streams:
```bash
cargo build -p kali_cli 2>/dev/null
$BIN run "$SCRATCH/repro.js" 2>"$SCRATCH/ordinals.txt"; cat "$SCRATCH/ordinals.txt"
```
Record in the triage doc: for `inner`'s two loops, does codegen assign ordinals `{0,1}` while `arena_gate` assigned them under a different `function_name` key or a shifted `{n,n+1}` (because one stream descended into / counted the nested function and the other did not)? Confirm or REFUTE the hypothesis with the actual numbers. If refuted, follow the evidence to the real divergence (e.g. `collect_function_locals` at `lower.rs:1754` keying the accumulation buffer by a name that collides across nesting) and record THAT.

- [ ] **Step 3: Write the root cause and remove all instrumentation**

Write the confirmed mechanism (exact `file:line`, why ≥2 loops + nesting is required, what the two disagreeing sides are) into the triage doc. Then remove every temporary `eprintln!`:
```bash
git diff --stat crates/kali_codegen crates/kali_mir   # MUST be empty (all instrumentation reverted)
```

- [ ] **Step 4: Commit the root cause**

```bash
git add docs/superpowers/followups/stageB-array-callback-triage.md
git commit -m "docs(soundness): stageB root cause — nested-function loop-ordinal keying divergence [stageB]"
```

---

## Task 3: Fix — align the ordinal streams (correct) or fail closed

The exact diff depends on Task 2's confirmed root cause; this task fixes the acceptance criteria and the decision rule, and points at the files the confirmed hypothesis implicates. Do NOT proceed until Task 2's root cause is written.

**Files:**
- Modify: the divergent ordinal/keying site Task 2 named — most likely `crates/kali_codegen/src/lower.rs` (`loop_preorder_ordinals` / `collect_function_locals`) and/or `crates/kali_mir/src/analysis/arena_gate.rs` (the `next_loop_ordinal` walk's nested-function handling).
- Test: `crates/kali_cli/tests/soundness_stageB_array_callback.rs` (new).

**Interfaces:**
- Consumes: Task 2's root cause.
- Produces: the repro computes correctly, or fails closed E5506 — asserted by the Task 4 fixture.

- [ ] **Step 1: Write the failing end-to-end test**

Create `crates/kali_cli/tests/soundness_stageB_array_callback.rs`:
```rust
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

/// A named function nested in a real function body, with 2+ array-callback
/// for-of loops accumulating into one buffer, must not silently miscompile.
/// node prints "1,2,3,4"; pre-fix kali printed a wrong/short value with exit 0.
#[test]
fn nested_function_two_array_callback_loops_accumulate_correctly() {
    let out = run_kali(
        "function outer() {\n\
         \x20 function inner() {\n\
         \x20   let out = [];\n\
         \x20   for (const x of [1, 2].map(v => v)) out.push(x);\n\
         \x20   for (const y of [3, 4].map(v => v)) out.push(y);\n\
         \x20   console.log(out.join(\",\"));\n\
         \x20 }\n\
         \x20 inner();\n\
         }\n\
         outer();\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1,2,3,4\n");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_cli --test soundness_stageB_array_callback -- --test-threads=4`
Expected: FAIL — kali prints the wrong value (silent miscompile), not `1,2,3,4`.

- [ ] **Step 3: Apply the fix per Task 2's root cause**

**Decision rule (reject-don't-miscompile):**
- **If the root cause is a numbering/keying divergence** (the lead hypothesis — the two loop-ordinal streams disagree only because one descends into the nested function and the other does not): make them **agree** — apply the SAME nested-function boundary rule to both `loop_preorder_ordinals`/`collect_function_locals` (`kali_codegen`) and the `arena_gate` `next_loop_ordinal` walk (`kali_mir`), so `(function_name, ordinal)` keys line up. This is correct lowering: the loops get distinct arena/buffer slots and both accumulate. Prefer this — it computes the right answer.
- **If the correct lowering is genuinely intractable under the region-only model** (e.g. the construct needs a capability that does not exist and cannot be added within this stage): make the case **fail closed with E5506** and a clear diagnostic, rather than emit a wrong value. Record why in the triage doc. A fail-closed outcome still satisfies the reject-don't-miscompile invariant, but is second choice — the accumulation itself is a plain, common shape that should compile.

Whichever branch: no `_ =>` arm; every no-op arm cites `file:line`. Keep both ordinal streams' nested-function handling documented in a comment at each site so they cannot silently drift again (this is the exact class of hand-mirrored divergence that caused the bug).

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_cli --test soundness_stageB_array_callback -- --test-threads=4`
Expected: PASS (`1,2,3,4`) if you took the correct-lowering branch; if you took the fail-closed branch, change the test's assertion to expect `!success` + `E5506` and document the deviation in the triage doc (only permissible if Task 2 proved correct lowering intractable).

- [ ] **Step 5: Adversarial re-mask probe**

Revert only the fix hunk (keep the test), rebuild, and confirm the test goes red with the ORIGINAL silent symptom (wrong value, exit 0) — proving the test measures the fix. Restore; confirm an empty diff on the fix files.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen crates/kali_mir crates/kali_cli
git commit -m "fix(codegen): align nested-function loop-ordinal keying so array-callback loops accumulate [stageB]"
```

---

## Task 4: Regression fixture, independence, stage gate + triage

**Files:**
- Modify: `crates/kali_cli/tests/soundness_stageB_array_callback.rs`, `docs/superpowers/followups/stageB-array-callback-triage.md`

**Interfaces:**
- Consumes: Task 3's fix.

- [ ] **Step 1: Add the boundary-cell regressions**

Add tests pinning the cells from Task 1 Step 3 that must KEEP working (so a future change can't regress them) — at minimum: the single-loop case, the inlined-no-nesting case, and (if it broke pre-fix) the double-nested case. Each asserts the exact stdout, cross-checked against node's output recorded in the triage doc. Run:
`cargo test -p kali_cli --test soundness_stageB_array_callback -- --test-threads=4` → all green.

- [ ] **Step 2: Confirm independence from Stages A and C**

The fix must stand alone. Verify the repro's fix does not depend on repr_infer (Stage A) or closures (Stage C): the repro uses no `for..in`, no string-in-callback, and no enclosing-scope capture, so neither gap is in play. Record in the triage doc a one-paragraph argument that the fix touches only loop-ordinal/arena keying and is orthogonal to A/C, plus a note that Stage A/C repros still fail as before (run one A repro and one C repro from the umbrella spec; they must still E5506 — proving B's fix did not accidentally touch them).

- [ ] **Step 3: Two independent full-workspace enumerations + the primary gate**

```bash
cargo build -p kali_cli
for i in 1 2; do
  nohup bash -c "cd /workspace && cargo test --workspace --no-fail-fast > \"$SCRATCH/bpost$i.log\" 2>&1; echo DONE > \"$SCRATCH/bpost$i.done\"" >/dev/null 2>&1 &
  # poll each to completion in a separate bounded call before starting the next
  grep -E '^test .* \.\.\. FAILED' "$SCRATCH/bpost$i.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort -u > "$SCRATCH/stageB-post-run$i.txt"
done
diff "$SCRATCH/stageB-post-run1.txt" "$SCRATCH/stageB-post-run2.txt"        # zero drift
sort -u "$SCRATCH/stageB-post-run1.txt" "$SCRATCH/stageB-post-run2.txt" > "$SCRATCH/stageB-post.txt"
comm -13 "$SCRATCH/stageB-pre.txt" "$SCRATCH/stageB-post.txt"   # PRIMARY GATE: MUST print nothing
comm -23 "$SCRATCH/stageB-pre.txt" "$SCRATCH/stageB-post.txt"   # drain — measure, do not forecast
```

- [ ] **Step 4: Main-worktree cross-check**

```bash
cd /workspace/.worktrees/kali-main && cargo build -p kali_cli
nohup bash -c 'cd /workspace/.worktrees/kali-main && cargo test --workspace --no-fail-fast > "$SCRATCH/bmain.log" 2>&1; echo DONE > "$SCRATCH/bmain.done"' >/dev/null 2>&1 &
# poll to completion, then:
grep -E '^test .* \.\.\. FAILED' "$SCRATCH/bmain.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort -u > "$SCRATCH/bmain-post.txt"
comm -13 "$SCRATCH/bmain-post.txt" "$SCRATCH/stageB-post.txt" | comm -13 "$SCRATCH/stageB-pre.txt" -   # expect empty
```

- [ ] **Step 5: Write the triage close-out + commit**

Record in the triage doc: the minimal repro; the failure boundary matrix; the confirmed root cause; the fix (correct-lowering or fail-closed, with which and why); the re-mask probe result; the independence argument; entry 731 → exit (measured); any drain, measured not forecast.

```bash
git add crates/kali_cli/tests/soundness_stageB_array_callback.rs docs/superpowers/followups/stageB-array-callback-triage.md
git commit -m "docs(soundness): stageB checkpoint — fix verified, gate clean, independence confirmed [stageB]"
```

---

## Self-Review

**Spec coverage (§3 Stage B):** symptom pinned → Task 1; root cause → Task 2; fix correct-or-fail-closed → Task 3 (decision rule); regression fixture cross-checked vs node that reddens on revert → Task 3 Step 5 + Task 4 Step 1; independence from A/C → Task 4 Step 2; zero newly-red vs 731, sort -u, cross-check main → Task 4 Steps 3–4. **No gaps.**

**Placeholder scan:** the only contingent content is Task 3's exact fix diff, which is explicitly gated on Task 2's written root cause (an undiagnosed bug cannot have its exact diff pre-written; the acceptance test, decision rule, and implicated files ARE concrete). Investigation instrumentation is temporary and required-reverted (Task 2 Step 3). No "TBD"/"add error handling"/"similar to" placeholders.

**Type consistency:** `run_kali(&str) -> Output` / `kali_bin()` defined in Task 3 Step 1, reused in Task 4. Baseline artifact `$SCRATCH/stageB-pre.txt` produced in Task 1, consumed in Task 4. Test file name `soundness_stageB_array_callback.rs` consistent across Tasks 3–4.

**Known risk carried into execution:** if Task 2 refutes the loop-ordinal hypothesis, Task 3's correct-lowering branch changes shape — but its acceptance (correct-or-fail-closed, re-mask probe, zero newly-red) is invariant, so the stage's gate holds regardless of the diagnosis.
