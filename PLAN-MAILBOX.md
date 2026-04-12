# PLAN Mailbox

## 2026-04-12 — Stage 1.14 configless-install wording aligned to SPEC

`plan/phase-1/14-evidence-hardening.md` previously described `kali install` on a project with no `kali.json` as a clear error, but `specs/14-packages.md` defines the configless-install split as a clean no-op success that must not create a placeholder manifest. The stage note has been updated to match the spec-correct behavior.

## 2026-04-12 — Stage 1.14 raw-URL install idempotence coverage added

Added a regression test in `crates/kali_npm` that installs the same raw-URL graph twice and asserts the resulting `kali.lock` bytes remain identical across both runs. This closes the most straightforward remaining install-workflow determinism gap in Stage 1.14 without changing the underlying install semantics.

## 2026-04-12 — Stage 2.1 alias-chain precision completed

Stage 2.1's remaining escape-analysis gap was closed by teaching `kali_mir` to resolve function-expression aliases through alias chains, including anonymous function expressions lowered to synthetic function names. That keeps direct-call precision intact for `const alias = identity; const alias2 = alias; alias2(...)` style call targets.

## 2026-04-12 — Stage 2.4 provisional Lean model update

`PLAN.md`'s Stage 2.4 row was updated to reflect the checked-in Lean workspace and the fact that the current progress/preservation work is still represented by theorem statements with documented-`sorry` placeholders.

No change was made to the Phase 2 completion gate yet; it should continue to read as an open gate until the later proof-backed work closes the remaining obligations.

## 2026-04-12 — Stage 2.1 escape-analysis follow-up

While extending MIR ownership analysis, I confirmed that call-argument escape marking is working, but the current HIR lowering for object-literal / heap-store-shaped values still flattens them into placeholder nodes instead of a stable composite shape. That means precise "stored into heap object" escape tracking is still blocked by frontend/HIR normalization, not by MIR alone.

Plan follow-up: schedule the HIR normalization work before claiming full heap-store classification coverage in Stage 2.1, or explicitly narrow the stage note to call/return/capture escapes until the frontend shape is stabilized.

## 2026-04-12 — Stage 2.1 HIR normalization resolved

The frontend now lowers object-literal properties into a dedicated `ObjectProperty` HIR node and lowers property keys as literals instead of identifiers. MIR ownership analysis now sees object literal values as escape flows without treating property names as bindings.

Suggested plan/status follow-up: update the Stage 2.1 status note to reflect the stabilized composite shape and keep the remaining escape-analysis work focused on other coverage gaps.

## 2026-04-12 — Stage 2.1 escape-analysis broadened

Added targeted MIR ownership tests for array-element and member-assignment heap-store flows so Stage 2.1 coverage now extends beyond call/return/object-literal cases.

Suggested follow-up: keep extending the analyzer with any remaining nested store or closure-shape edge cases before marking the stage complete.

## 2026-04-12 — Stage 3.2 package-host-fit preparation

`kali_npm` now keys install-time host-fit validation off the project `compilerOptions.apiSurface`, so `node`-targeted projects can accept Node-only builtins while the default standalone context still rejects them with `E6005`.

Suggested follow-up:
- keep the Stage 3.2 plan/status notes aligned with this package-host-fit split
- continue wiring the remaining CLI/runtime `--api node` paths so this install-time allowance becomes part of a real Node compatibility command context
- no spec change was needed for this increment
