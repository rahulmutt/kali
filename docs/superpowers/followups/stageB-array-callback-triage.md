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

## 5. Root cause — CONFIRMED (Task 2)

Full evidence + wat/instrumentation dumps: `.superpowers/sdd/task-2-report.md`.

**Mechanism (exact site):** codegen picks the growable-array lane per binding
from `repr_table.is_growable_array_binding(function_name, name)`
(`crates/kali_codegen/src/emitter.rs:277-282`). That table is filled by the
choke-point predicate `growable_array_candidates`
(`crates/kali_types/src/growable.rs:122`), which the repr-inference walker runs
**only** on `Statement::FunctionDeclaration` nodes:
`collect_growable_candidates_in_stmt`
(**`crates/kali_types/src/repr_infer.rs:733-772`**) has a lone
`FunctionDeclaration` arm (line 735) and a terminal `_ => {}` arm (line 771) that
swallows `Statement::VariableDeclaration` **without descending into its
initializer expression**. Arrows / function-expressions are *expressions* (the
init of `const f = …`), so the predicate never runs on their bodies — yet those
bodies ARE emitted as standalone wasm functions named `__kali_fn_{N}`
(`name_anon_functions`, `crates/kali_cli/src/build/compile.rs:674`, before the
repr pass). Codegen then queries `is_growable_array_binding("__kali_fn_0","o")`
→ **false** → the array literal + `.push` lower to the poison lane
(`i64.const 0`; the whole growable sequence is absent from the wat), so
`.length`/`o[i]` read back `0`. Two sibling walkers share the gap:
`collect_functions_in_stmt` (`repr_infer.rs:~690-717`) and
`collect_local_names_in_stmt` (`repr_infer.rs:788-796`).

**Instrumentation proof:** working `function f` → `DBGSCAN … 'f' -> {"o"}` +
`DBGEMIT function 'f' … growable={"o"}`; broken arrow → **no** `DBGSCAN` line
(predicate never ran) + `DBGEMIT function '__kali_fn_0' locals=["o"] growable={}`.

**(a)** Defect site: `repr_infer.rs:733-772` (the `_ => {}` arm at 771 never
descends into fn-expr/arrow inits; the predicate is invoked only in the
`FunctionDeclaration` arm at 735). Consumer that reads `false`:
`emitter.rs:277-282`. **(b)** `function` declarations hit the explicit
`FunctionDeclaration` arm keyed on `func.name`, which matches the emitted name —
so promotion reaches codegen; scalars never consult the growable table (plain
wasm locals), so they are scope-independent. **(c)** **Contained — no missing
region capability.** The growable runtime lane already works (the fn-decl path
proves it); the array local does not escape (no closure capture needed). The only
gap is analysis *traversal*, and the synthetic `__kali_fn_{N}` name is already
assigned at the AST level before repr-inference, so keying the predicate on it
matches codegen by construction.

**Note — bug is PRE-EXISTING, not introduced by the patch.** On the clean branch
(patch reverted), a plain function-expression `const f = function(){ let o=[];
o.push(1); o.push(2); console.log(o.length) }; f()` already silently miscompiles
(kali `0` vs node `2`). The un-flatten only *widens* the trigger to block-arrows
(which previously mis-parsed).

**Recommendation: correct-lowering** — extend the three
`FunctionDeclaration`-only repr-inference walkers to descend into
function-expression / arrow bodies (keyed on their synthetic `__kali_fn_{N}`
name) and run the existing predicate. The fn-expr shape gives a clean-branch,
non-`#[ignore]` regression test to gate the fix now; the block-arrow fixture
stays `#[ignore]`'d until Stage D. Fail-closed E5506 is the sound fallback only
if Task 3 finds the repr-axis (Phase B) intersection needs more than a traversal
extension — it would convert the silent wrong answer to a loud diagnostic without
regressing any currently-working program (the affected shapes miscompile today).
