# PLAN Mailbox

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
