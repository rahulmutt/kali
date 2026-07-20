# `throw`-fallout — un-mask and genuinely fix all 922 workspace tests (zero flips)

**Date:** 2026-07-11
**Branch:** `soundness-batch1-pra` (PR #16, currently draft)
**Status:** Design approved — ready for `writing-plans`
**Supersedes as the driving doc:** `docs/superpowers/followups/throw-fallout-project.md` (that file remains the raw triage; this is the program design)

## Problem

Soundness Batch 1 Task 1 made `throw` sound (print-then-trap instead of a silent
no-op). That is correct, but it **un-masks 922 workspace tests** that were
fake-green on `main`: their self-check `throw`s were silent no-ops, so a program
with a real underlying bug still exited 0.

- `cargo test --workspace`: **main = 0 failures, branch = 922 failures.**
- PR #16 is held **draft** until the branch is green vs main.

This program fixes all 922 — **genuinely, with zero flips**. Every failing test
gets a real implementation matching node's observable behavior. No construct is
rejected or trapped to make a test pass.

## Ambition (locked)

Maintainer decision 2026-07-11: **one mega effort, fix everything, zero flips.**
Every one of the 922 is treated as a real feature to implement — including the
full async/Promise value lane and real short-circuit semantics. Nothing is
rejected/trapped to go green. This is recorded as a program **invariant** so no
individual stage can quietly downgrade a fix to a flip.

## Program shape

A **decomposed program** on the existing `soundness-batch1-pra` branch:

- This umbrella spec fixes the invariants, the class inventory, and the sequence.
- Each bug-class is its own `writing-plans` cycle, lands as its own commit + gate
  checkpoint.
- The async lane, being spec-scale, gets its own **nested sub-spec** brainstormed
  when the cheaper stages are green.
- PR #16 flips draft → ready only after the **last** stage is green.

"One mega effort" is about *completeness* (all 922, zero flips), not about one
monolithic document — decomposition keeps each subsystem independently
understandable and independently gated.

## The one hard gate (every stage, every commit)

- Run `cargo test --workspace` — the **exact CI command**, whole workspace, never
  a subset.
- Diff the failing set against a **`main` worktree** (built at merge-base), never
  a mid-branch red baseline. Stand up one persistent `main` worktree (e.g.
  `../kali-main`) at program start; every checkpoint diffs against it.
- A stage is **done** only when its target tests are green **and** the global
  failing set has strictly shrunk with **zero** green-on-main tests turned red.
- Node parity is byte-for-byte on the same fixture, matching prior CLBG
  discipline.

See memory `ci-gate-vs-poisoned-baseline` for why the gate must be whole-workspace
and diffed against main, not a self-defined post-change red baseline.

## Invariants (non-negotiable across the whole program)

1. **Fix, never flip.** Every one of the 922 gets a real implementation that
   matches node. No construct is rejected or trapped to pass a test.
2. **Honest-red is allowed mid-stage, never at a checkpoint.** Stage 0 will
   *increase* red as it exposes true failures; a stage does not close until
   net-green vs its start.
3. **No re-masking.** A fix that makes a self-check `throw` silently no-op again
   is a defect even if the test goes green. Each stage's review checks this.
4. **Parity is defined by node**, same fixture, byte-for-byte.

## Stages (each its own `writing-plans` cycle + gate checkpoint)

Ordered so the true denominator comes first and the spec-scale lanes come last.
Test-count estimates are from the Task-1b triage and are approximate; each stage
enumerates its exact target set at its own start.

### Stage 0 — Harness trap-swallow (#9) — FOUNDATION
- **Broken:** browser `Kali.test` envelope reports `passed:1, success:true,
  exitCode 0` while stderr shows `RuntimeError: unreachable`.
- **Root:** once a callback body traps, the harness catches/ignores it instead of
  propagating.
- **Fix:** any trap in a test callback is a failure — non-zero exit,
  `success:false`.
- **Why first:** this turns *more* browser tests honestly red. We need the true
  denominator before counting green, or every later stage measures against a lie.
  This is the one stage expected to increase red before anything decreases it.

### Stage 1 — Runtime string equality (#2 + #3) — BIGGEST GREEN DELTA
- **Broken:** `Object.keys(o)[0] !== '1'` is true though it prints `1`;
  `Deno.env.get('X') !== 'y'` likewise.
- **Root:** runtime `==`/`!=`/`===` compares string **handles** (fresh buffer vs
  interned literal), not **contents**.
- **Fix:** real content comparison on the `Repr::String` axis — compare length +
  bytes, not handle identity. Greens a large fraction of the enumeration +
  deno-env + web-baseline corpus at once.

### Stage 2 — Enumeration: delete + reinsert (#4)
- **Broken:** `delete o.b; o.b = 4` yields stale key order/values
  (reflect_own_keys, frozen_object_enumeration).
- **Root:** object-model doesn't update the shape/key-table on delete-then-reinsert.
- **Fix:** delete removes and reinsert re-appends in insertion order, matching node.

### Stage 3 — Host wiring (#5 + #6) — own sub-spec
- **Broken:** `performance.now()` → 0; web crypto (`getRandomValues`,
  `subtle.digest`, `randomUUID`) unimplemented.
- **Root:** host imports not wired into the wasm lane.
- **Fix:** wire host imports through **all four hand-mirrored import lists**
  (harness.rs ×2 + cmd_build.rs ×2 — sync hazard from memory
  `kali-browser-harness-import-sync`) plus host-side implementations. Grouped as
  one host-wiring family; scoped as its own sub-spec.

### Stage 4 — Array/for-of push lane (#10)
- **Broken:** pushing into a const array via for-of over `.map`/`.filter`
  observes empty (`array_callback_identity`, ~8).
- **Fix:** the for-of-over-callback-result lane materializes and iterates the
  produced array so pushes land.

### Stage 5 — Dynamic import member typeof (#7, ~32)
- **Broken:** `typeof mod.f !== 'function'` outside the provable lane.
- **Fix:** `typeof` on a dynamic-import member resolves to the real member kind.
  Under zero-flips this is a genuine fix, not the rejection the raw inventory
  hedged as "fix or flip".

### Stage 6 — Short-circuit semantics (#8)
- **Broken:** `&&`/`||` evaluate a side-effecting RHS unconditionally.
- **Fix:** real short-circuit lowering — RHS evaluated only when LHS requires it.
- **Note:** the raw inventory tagged this "PR-B scope", but no PR-B branch exists
  and the ambition is zero-flips, so it is **in** this program. The parser has
  **no LogicalExpression node** (memory `kali-runtime-join-spec3`) — this stage
  must first confirm how `&&`/`||` are represented before lowering. Codegen and
  kali_types predicates are hand-mirrored — new expression handling needs arms on
  **both** sides or it fails open.

### Stage 7 — Async / Promise value lane (#1, ~200) — SPEC-SCALE, own sub-spec
- **Broken:** `await Promise.resolve(7)` → 0; microtasks run eagerly.
- **Fix:** a real await/Promise value lane + a microtask queue with node-correct
  draining order.
- **Sequenced last:** all cheaper green is banked and the branch is otherwise
  clean before taking on the hardest lane. Gets its own nested design (below).

## Async sub-spec boundary (Stage 7)

Not fully designed here; the umbrella fixes its boundary so its later brainstorm
has a frame:

- **Scope:** `await`, `Promise.resolve/reject`, `.then` value propagation,
  `Promise.all/any/race/allSettled`, `queueMicrotask` ordering — the exact set is
  **enumerated from the failing tests at the start of that sub-spec**, not assumed.
- **Model:** a microtask queue with node-correct draining (microtasks after the
  current synchronous run-to-completion, before macrotasks). Replaces today's
  eager-microtask + placeholder-0 behavior.
- **Constraint:** stays **GC-less** (memory `kali-gc-less-invariant`) — no tracing
  GC for promise state; reclamation by region/escape as everywhere else.
- **Deliverable:** its own `docs/superpowers/specs/…-async-value-lane-design.md`,
  brainstormed when Stages 0–6 are green.

## Testing / gate mechanics

- Persistent `main` worktree stood up once at program start (e.g. `../kali-main`),
  built at merge-base; its failing set is 0.
- Per checkpoint: `cargo test --workspace` on branch → capture failing set → diff
  vs the `main` worktree. Pass = branch failing set strictly shrank **and**
  contains no test green on main.
- Snapshot each stage's failing-set delta into this doc (or an adjacent progress
  file) so the 922 visibly drains stage by stage.
- Node parity byte-for-byte on the same fixture.

## Risks & coordination

1. **Stage 0 raises red before lowering it** — expected per Invariant 2; not a
   regression.
2. **Import-list desync** (Stage 3) — the four hand-mirrored lists are a known
   footgun; the stage plan must touch all four or browser tests LinkError
   (`kali-browser-harness-import-sync`).
3. **Codegen / kali_types hand-mirror** — oracles and type predicates are
   hand-mirrored; new expression kinds (short-circuit, async) need arms on **both**
   sides or they fail open. Standing check for Stages 6–7.
4. **Block-arrows deferred item stays OUT.** `task8-block-arrows-DEFERRED.md` is
   blocked on a separate repr-tracking prerequisite (untracked function scopes).
   It shares the "un-masking exposes real gaps" theme but is **not** in this
   program's 922 and stays deferred. Linkage noted; not absorbed.
5. **Re-masking defects** — a fix that re-silences a self-check `throw` goes green
   while re-hiding a bug (Invariant 3). Each stage's review checks it didn't
   re-mask.

## Definition of done (whole program)

`cargo test --workspace` on `soundness-batch1-pra` = **0 failures** vs the `main`
worktree; all fixes real (zero flips); no re-masking; PR #16 flipped draft →
ready.

## Recommended execution

Brainstorm this umbrella (done) → `writing-plans` per stage, in order Stage 0 → 7
→ subagent-driven execution, each stage gated on `cargo test --workspace` vs the
`main` worktree. Stages 3 and 7 each spawn a nested sub-spec before their plan.
