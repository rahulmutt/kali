## 2026-04-19 — Stage 4.2 release-only linear-memory companion widening

I found a small Stage 4.2 follow-up for the RC snapshot proof slice: the release-only helper could use the same combined origin / ownership / positive-count + linear-memory companion wording already used by the decrement and collection helpers.

Planned update:
- add `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` to the proof-backed RC snapshot inventory, then sync `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and the related summary docs so the companion theorem is named explicitly
- keep the claim narrow: this is a helper-level RC widening, not the broader Stage 4.2 ownership/freeing target

Completed:
- the companion theorem is now named explicitly in `PLAN.md`, `PLAN-4.2-STATUS.md`, and the current boundary summaries, so the pure-release provenance slice now matches the decrement and collection helper wording.

## 2026-04-19 — Stage 4.2 decrement linear-memory companion widening

I widened the RC snapshot proof slice so the decrement helper now names `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly alongside the release-only and collection companions.

Planned update:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and the proof-summary anti-drift test so the new theorem is named explicitly
- keep the claim narrow: this is another helper-level RC widening on top of the current published boundary, not the full Stage 4.2 ownership/freeing target

Completed:
- the companion theorem is now explicitly named in `PLAN.md`, `PLAN-4.2-STATUS.md`, `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and the schema-docs anti-drift guard.

## 2026-04-19 — Stage 3.1 object-literal property-order canonicalization

I widened the Stage 3.1 specialization path one step further so object-literal property order is now canonicalized during MIR-aware specialization, which lets semantically identical object shapes with reordered fields reuse the same clone instead of splitting on insertion order.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the object-literal property-order canonicalization alongside the existing MIR-backed layout and array-literal widening coverage
- keep the claim narrow: this is another specialization-depth widening within the existing optimizer model, not a new support-rung claim

## 2026-04-19 — Stage 3.1 direct array-literal specialization widening

I widened the Stage 3.1 specialization path one step further so direct array-literal call-site arguments now carry explicit array-shape signatures (`Value:array:len=...`) during MIR-aware specialization, which lets the optimizer split inline arrays with different lengths even when the callee only sees a tagged parameter.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the direct array-literal shape widening alongside the existing MIR-backed array-binding coverage
- keep the claim narrow: this is another specialization-depth widening within the existing optimizer model, not a new support-rung claim

## 2026-04-19 — Stage 3.1 array-layout specialization widening

I widened the Stage 3.1 specialization path so MIR-backed array bindings now preserve their layout fingerprints during call-site specialization. That lets the optimizer split otherwise identical hot paths when callers supply arrays with different element/length layouts, which is a concrete follow-up slice on top of the existing struct/closure/object-layout coverage.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the array-layout widening
- keep the claim narrow: this is still a specialization-depth widening within the existing optimizer model, not a new support-rung claim

## 2026-04-19 — Stage 3.3 scoped browser package widening

I widened the Stage 3.3 browser web-baseline interop corpus one step further by adding `@emotion/styled` alongside the existing scoped browser representative packages.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `@emotion/styled` alongside the existing browser web-baseline package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 heroicons/react-dom browser-corpus widening

I widened the Stage 3.3 browser web-baseline interop corpus one step further by adding `react-dom` alongside `@heroicons/react` in the package corpus, keeping the browser breadth notes concrete without changing the support-rung story.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `react-dom` alongside the existing browser web-baseline package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 4.2 ownership provenance wording sync

I synchronized the RC snapshot provenance wording so `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is spelled out directly alongside `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` in the summary docs and Stage 4.2 progress trackers.

Planned update:
- keep the companion theorem named directly in `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, and `plan/phase-4/02-formal-verification-depth.md` whenever the RC snapshot wording changes again
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 4.2 decrement linear-memory companion widening

I plan to widen the RC snapshot proof slice with a decrement-path linear-memory companion theorem, `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, so the decrement helper's origin/ownership/positivity story is paired with the same explicit linear-memory payload wording already used by the collection helper.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and the proof-summary anti-drift test so the new theorem is named explicitly
- keep the claim narrow: this is another helper-level RC widening on top of the current published boundary, not the full Stage 4.2 ownership/freeing target
