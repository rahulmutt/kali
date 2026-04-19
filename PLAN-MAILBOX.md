## 2026-04-19 — Stage 4.2 no-dangling-reference summary sync

I synced the Stage 4.2 proof-summary wording so `KaliCore.Safety.noDanglingReference` is named explicitly in the status tracker alongside the rest of the published RC snapshot inventory.

Planned update:
- sync `PLAN-4.2-STATUS.md` so the current Stage 4.2 status note names `KaliCore.Safety.noDanglingReference` explicitly alongside the existing RC snapshot theorem inventory
- keep the claim narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening

Completed:
- the Stage 4.2 status tracker now names `KaliCore.Safety.noDanglingReference` explicitly alongside the existing RC snapshot theorem inventory.

## 2026-04-19 — Stage 3.3 valtio corpus widening

I widened the Stage 3.3 package corpus one step further by adding `valtio` to the browser and utility web-baseline interop slices, keeping the representative state-management breadth concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `valtio` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `valtio` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 xstate corpus widening

I found a small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add one more representative state-management package name to the browser and utility web-baseline interop corpora so the evidence set keeps widening without changing any support-rung claims.

Planned update:
- add `xstate` to the browser and utility package-corpus coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the corpus breadth note names the new package explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change


## 2026-04-19 — Stage 3.3 pinia browser/utility corpus widening

I widened the Stage 3.3 package corpus one step further by adding `pinia` to the browser and utility web-baseline interop slices, keeping the representative Vue-oriented package breadth concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `pinia` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `pinia` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 formik browser/utility corpus widening

I widened the Stage 3.3 package corpus one step further by adding `formik` to the browser and utility web-baseline interop slices, keeping the representative package breadth concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `formik` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `formik` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 browser-baseline atob/btoa widening

I widened the shared browser/runtime baseline one step further by adding deterministic `atob` / `btoa` helpers to `kali_api_web` and reexporting them through the Deno compatibility surface, then mirrored that progress note in the Stage 3.3 ecosystem-breadth tracker.

Planned update:
- sync `crates/kali_api_web/src/lib.rs`, `crates/kali_api_deno/src/lib.rs`, and `plan/phase-3/03-ecosystem-breadth.md` so the browser-baseline note mentions the new helpers explicitly
- keep the claim narrow: this is still a browser-baseline / compatibility widening within the existing support model, not a support-rung change

Completed:
- deterministic `atob` / `btoa` helpers now exist in `kali_api_web`, are reexported through `kali_api_deno`, and are mentioned in the Stage 3.3 progress notes.

## 2026-04-19 — Stage 3.3 recoil browser/utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `recoil` to the browser and utility web-baseline interop slices, keeping the representative state-management corpus concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `recoil` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `recoil` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 swr package-corpus widening

I widened the Stage 3.3 package corpus a bit further by adding `swr` to the browser and utility web-baseline interop slices, keeping the package-corpus breadth note concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `swr` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `swr` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 redux utility-shape widening

I widened the Stage 3.3 utility corpus a bit further by adding `redux` to the utility exports-map, string-exports, and pattern-exports slices, keeping the package-corpus breadth note concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention the new utility-shape coverage
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change
## 2026-04-19 — Stage 3.3 mitt browser/utility corpus widening

I widened the Stage 3.3 package corpus one step further by adding `mitt` to the browser and utility web-baseline interop slices, keeping the package-corpus breadth note concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `mitt` alongside the existing browser web-baseline package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `mitt` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.

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

## 2026-04-19 — Stage 4.2 release-and-collect origin/positive-count linear-memory companion widening

I found a small Stage 4.2 follow-up for the RC snapshot proof slice: the local `releaseAndCollect` helper can use the same combined origin / positive-count + linear-memory companion wording already used by the release-only and decrement helpers.

Planned update:
- add `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` to the proof-backed RC snapshot slice, then sync `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and the related summary docs so the companion theorem is named explicitly
- keep the claim narrow: this is a helper-level RC widening, not the broader Stage 4.2 ownership/freeing target

Completed:
- the proof-backed RC slice now also names `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` explicitly in the plan/status trackers and summary docs.

## 2026-04-19 — Stage 3.3 utility-corpus breadth widening

I found a small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add one more representative utility/browser package name to the corpus so the evidence set keeps widening without changing any support-rung claims.

Planned update:
- add `redux` to the Stage 3.3 utility corpus coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress note in `plan/phase-3/03-ecosystem-breadth.md` and the matching `TODO.md` tracker so the corpus breadth note names the new package explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 4.2 heap-characterisation + linear-memory companion widening

I widened the proof-backed RC snapshot slice with explicit heap-characterisation companions for all three helper families: `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`.

Planned update:
- sync the proof-boundary manifest, verification summaries, Stage 4.2 tracker, TODO notes, and the proof-summary anti-drift guard so the new companion theorem names are spelled out everywhere the published boundary inventory is repeated
- keep the claim narrow: this is another helper-level RC proof widening, not the broader Stage 4.2 ownership/freeing target

Completed:
- the companion theorems now exist in `proofs/KaliCore/Safety.lean` and are named explicitly in the published boundary summaries and proof-summary guard.

## 2026-04-19 — Stage 4.2 final-heap positive-count wording sync

I found one remaining proof-summary drift point in the RC snapshot slice: the local collection helper's final-heap positivity theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` is mechanized already, but some of the summary/progress prose still describes it generically.

Planned update:
- sync `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and the Stage 4.2 plan/status notes so the theorem is named explicitly wherever the proof-backed boundary inventory is repeated
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 3.3 react-dom browser corpus widening

I found a small but concrete Stage 3.3 follow-up in the package-corpus breadth lane: `react-dom` is already covered in the browser web-baseline interop slice, but it still has room to exercise another representative browser package shape in the exports-map / browser-condition corpus.

Planned update:
- add `react-dom` to the browser exports-map and browser-condition corpus slices in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the browser representative-package breadth note names the new slice coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 4.2 no-dangling-reference summary sync

I found a small Stage 4.2 proof-summary drift gap: the published boundary already names `KaliCore.Safety.noDanglingReference`, but the current anti-drift guard and several summary docs do not pin it explicitly yet.

Planned update:
- add `KaliCore.Safety.noDanglingReference` to the RC snapshot theorem inventory in `crates/kali_cli/tests/schema_docs.rs`
- sync the summary/progress docs (`README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md`) so the base no-dangling theorem is named alongside the helper-level corollaries
- keep the claim narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening
