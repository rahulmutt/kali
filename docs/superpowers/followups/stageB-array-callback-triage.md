# Stage B triage — array-callback nested-function silent miscompile

> Investigation-first stage. This doc records the reproducer, the failure
> boundary, the (reframed) root-cause direction, the fix, and the stage gate.
> Plan: `docs/superpowers/plans/2026-07-15-blockarrow-prereq-stageB-array-callback.md`.
> Spec: `docs/superpowers/specs/2026-07-15-blockarrow-prereqs-design.md` §3 Stage B.

## 0. PLAN AMENDMENT (2026-07-15) — premise corrected

The plan/spec premise ("the bug reproduces on the CLEAN branch with plain
`function(){}` fixtures; do NOT apply the un-flatten patch") is **FALSIFIED**.

- Verified on a fresh binary at `c3d71bbbd`: the plan's minimal repro
  (`function outer(){ function inner(){ 2× array-callback for-of } inner() } outer()`)
  prints `1,2,3,4` from **both** kali and node — no bug on the clean branch.
- The branch's own evidence (`throw-fallout-stage6-triage.md:494`) records the
  precondition the spec's bisection dropped: an **un-flattened anon callback
  scope**, a shape produced **only** by `task5-block-arrows-WIP.patch`.

**User decision (2026-07-15):** amend Stage B to reproduce/fix against the
un-flattened shape, applying `task5-block-arrows-WIP.patch` as an *uncommitted
investigation scaffold*. The eventual fix is codegen-level and patch-independent;
it lands with the patch reverted (clean-branch gate stays zero-newly-red vs 731),
and the end-to-end regression fixture is `#[ignore]`'d until Stage D lands the
un-flatten and activates the trigger shape.

## 1. Baseline

- Frozen failure baseline: **731** (`$SCRATCH/stageB-pre.txt`, copied from the
  verified `stage6-pre.txt`). Clean branch, no patch.
- Scaffold: `git apply docs/superpowers/followups/task5-block-arrows-WIP.patch`
  (8 files: parser/codegen/types + 2 test files). Applies clean onto HEAD; left
  applied+uncommitted during investigation.

## 2. Confirmed reproducer (under the patch scaffold)

True minimal trigger — **no** for-of, `.map`, `queueMicrotask`, nested function,
or ≥2 loops required (all red herrings from the original characterization):

```js
const f = () => { let out = [7, 8]; console.log(out.length); };
f();
```
- kali: `0`  (exit 0, no diagnostic — **silent wrong answer**)
- node: `2`

A bare array **literal** with no `push` already breaks. Element reads are also
lost: `let out=[]; out.push(1); out.push(2)` → `len=0 e0=0 e1=0` (kali) vs
`len=2 e0=1 e1=2` (node).

## 3. Failure boundary (all cells run under the patch, kali vs node)

| # | shape | kali | node | verdict |
|---|---|---|---|---|
| a | array local in **arrow** body (`()=>{...}`) | `0` | correct | **BUG** |
| b | array local in **function-expression** (`const f=function(){...}`) | `0` | correct | **BUG** |
| c | array local at **module scope** (under patch) | `0` | correct | **BUG** |
| d | array local in **function declaration** (`function f(){...}`) | correct | correct | works |
| e | arrow **nested inside** a working function declaration | `0` | correct | **BUG** |
| f | **scalar** accumulation in arrow (`let n=0; n+=1; n+=2`) | `3` | `3` | works |
| g | array **literal** in arrow, no push (`let out=[7,8]`) | `len=0` | `len=2` | **BUG** |

**Reading:** the defect is scoped to **heap-object (array) locals in
non-`FunctionDeclaration` function scopes** created (or re-shaped) by the
un-flatten. Scalar locals are fine (cell f); `function`-declaration scopes are
fine (cell d); the breakage follows the arrow/fn-expr scope itself, not its
enclosing context (cell e).

## 4. Ground-truth tests (under the patch)

`crates/kali_cli/tests/array_callback_identity_browser_harness.rs` (fixture
`browserArrayCallbackIdentitySlices`) — 8 tests newly-fail:
`{json_,}test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_{js,jsx,ts,tsx}_input`.
The `test` command wraps the body in `Kali.test('...', () => {...})` — an
un-flattened callback. Because that fixture has a self-check
`if (observed.join(",") !== "…") throw`, the corruption surfaces **loud**
(`error[E4000]: runtime trap in callback '__kali_callback_…'`, exit 1). The
parallel unwrapped `run_*` variants pass. **Without a self-check guard the same
corruption is SILENT** (wrong value, exit 0) — that is the real user-facing
miscompile this stage must close.

## 5. Root-cause direction (to be pinned to a line in Task 2)

Reframed from the plan's refuted "loop-ordinal keying divergence": array/heap-object
locals are not materialized/tracked when their declaring scope is an
arrow/function-expression body under the un-flatten — the growable-array header
(`[len][cap][data_ptr]`) / heap-object base slot is never allocated in that
frame, so length and elements read back as `0`. Category-identical to gaps A/C:
the un-flatten introduces a scope kind codegen does not fully handle. Task 2
pins the exact `file:line`; Task 3 fixes correct-lowering (materialize the frame)
or, if intractable in-stage, fails closed E5506.
