Subject: Stage 3.3 reselect utility exports-map/pattern-exports widening

## 2026-04-19 — Stage 3.3 reselect utility exports-map/pattern-exports widening

I widened the Stage 3.3 package corpus one more step by adding `reselect` to the utility exports-map and pattern-exports slices, so the representative state-management package breadth now has a shape-focused standalone corpus case too.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `reselect` in the utility exports-map and pattern-exports slices.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name the `reselect` utility exports-map/pattern-exports widening explicitly.
- Kept the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

Subject: Stage 3.3 chart.js standalone corpus widening

## 2026-04-19 — Stage 3.3 chart.js standalone corpus widening

I’m widening the Stage 3.3 package corpus one more step by adding `chart.js` to the utility plain-package and web-baseline interop slices, so the browser charting package breadth is now explicit on the default standalone surface too.

Planned update:
- add `chart.js` to `crates/kali_cli/tests/package_corpus.rs` in the utility plain-package and web-baseline interop corpus slices
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new standalone `chart.js` coverage is named explicitly
- keep the claim narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 @playwright/test browser corpus widening

I’m widening the Stage 3.3 package corpus one step further by adding `@playwright/test` to the browser web-baseline interop slice, so the representative browser test-runner breadth keeps growing without changing any support-rung claims.

Planned update:
- add `@playwright/test` to `crates/kali_cli/tests/package_corpus.rs`
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`
- keep the claim narrow: this is a corpus/evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 msw utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `msw` to the utility plain-package corpus on the default standalone surface, so the representative browser/networking breadth is now explicit on both the browser and standalone paths.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly name the utility `msw` coverage
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the utility plain-package corpus now also exercises `msw` on the default standalone surface, and the Stage 3.3 progress notes now call that coverage out explicitly.

## 2026-04-19 — Stage 3.3 react/preact utility corpus widening

I widened the Stage 3.3 package-corpus follow-up a little further by adding `react` and `preact` to the utility plain-package corpus on the default standalone surface, so the React/Preact package breadth is now explicit on both the browser and standalone paths.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly name the utility React/Preact coverage
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the utility plain-package corpus now also exercises `react` and `preact` on the default standalone surface, and the Stage 3.3 progress notes now call that coverage out explicitly.

## 2026-04-19 — Stage 4.2 collection-helper provenance/linear-memory wording sync

I tightened the Stage 4.2 verification-depth follow-up wording so the collection helper's owned-payload bridge is spelled out more explicitly in the top-level plan note.

Planned update:
- sync `PLAN.md` so the Stage 4.2 follow-up note explicitly names the decrement and collection ownership/linear-memory companions alongside the release-only bridge
- keep the claim narrow: this is a progress-note wording sync for the published boundary, not a boundary widening

Completed:
- the top-level Stage 4.2 follow-up note now names the decrement and collection ownership/linear-memory companions explicitly, and the stale remaining-work note in `TODO.md` has been cleaned up to match the already-published boundary wording


## 2026-04-19 — Stage 3.3 @tanstack/router browser/utility corpus widening

I widened the Stage 3.3 package corpus a little further by adding `@tanstack/router` to the browser and utility web-baseline interop slices, keeping the representative scoped routing-package breadth concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `@tanstack/router` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser and utility web-baseline interop corpus now also names `@tanstack/router` explicitly, and the related Stage 3.3 progress notes stay aligned with that narrower slice coverage.

## 2026-04-19 — Stage 4.2 decrement target origin/positive-count linear-memory companion sync

Completed: the decrement-path target theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` now has an explicit matching linear-memory companion in the proof-backed boundary, and the stage/plan summaries now name it alongside the existing RC snapshot inventory.

## 2026-04-19 — Stage 3.1 special-number literal-signature sync

I synced the Stage 3.1 specialization-depth summary wording so `Infinity`, `-Infinity`, and `NaN` are called out explicitly alongside the other literal-signature cases in the top-level follow-up notes.

Planned update:
- sync `PLAN.md` and `TODO.md` so the Stage 3.1 top-level progress summary names the special-number literal signatures explicitly alongside the existing signed-zero / numeric-literal wording
- keep the claim narrow: this is a summary-doc wording sync for the existing specialization model, not a new support-rung claim

Completed:
- the top-level Stage 3.1 follow-up notes now name the special-number literal-signature cases explicitly, keeping the summary aligned with the stage-level progress note.

## 2026-04-19 — Stage 3.3 @chakra-ui/react exports-map/browser-condition widening

I widened the Stage 3.3 package corpus a little further by adding `@chakra-ui/react` to the scoped browser exports-map and browser-condition slices, keeping the representative UI-package breadth concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `@chakra-ui/react` alongside the existing browser package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the scoped browser exports-map and browser-condition corpus now also name `@chakra-ui/react` explicitly, and the related Stage 3.3 progress notes stay aligned with that narrower slice coverage.

## 2026-04-19 — Stage 3.3 @apollo/client browser corpus widening

I widened the Stage 3.3 package corpus a little further by adding `@apollo/client` to the browser web-baseline interop, typed-export-branch, exports-map, and browser-condition slices, keeping the representative scoped browser breadth concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `@apollo/client` alongside the existing browser package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser corpus and the related Stage 3.3 progress notes now also name `@apollo/client` explicitly, and the corpus-breadth wording stays aligned with that narrower slice coverage.

## 2026-04-19 — Stage 3.3 classnames corpus widening

I widened the Stage 3.3 package corpus a little further by adding `classnames` to the browser web-baseline interop slice and the utility corpus, keeping the representative lightweight package breadth concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `classnames` alongside the existing browser/utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser web-baseline interop corpus and utility corpus now also name `classnames` explicitly, and the related Stage 3.3 progress notes stay aligned with that narrower slice coverage.

## 2026-04-19 — Stage 3.1 concrete-argument specialization fallback

I finished the in-progress generic/function specialization fallback so literal-shaped call sites can clone deterministic helpers even when MIR layout metadata is unavailable.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the current Stage 3.1 progress note names the pure-LIR fallback explicitly
- keep the claim narrow: this is still a specialization-depth widening within the existing optimizer model, not a new support-rung claim

Completed:
- the pure-LIR release path now clones deterministic generic/function helpers from literal-shaped call sites without MIR layout metadata, and the Stage 3.1 progress notes now call out that fallback explicitly.

## 2026-04-19 — Stage 3.3 hono exports-map/pattern widening

I found a small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add `hono` to the browser exports-map and pattern-exports corpora so the representative browser package-shape coverage keeps widening without changing any support-rung claims.

Planned update:
- add `hono` to the browser exports-map and pattern-exports coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the corpus breadth note names the new package-shape coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser exports-map and pattern-exports corpus now also names `hono` explicitly, and the related Stage 3.3 progress notes stay aligned with that narrower slice coverage.

I found a small proof-boundary follow-up for the RC snapshot tracker: the decrement helper can expose a target-specific origin/positive-count theorem alongside the existing target-allocation and heap-characterisation lemmas.

Planned update:
- add `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` to the current published theorem inventory, then sync `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and the verification summaries so the new theorem is named explicitly
- keep the claim narrow: this is still helper-level RC widening, not the broader Stage 4.2 ownership/freeing target

## 2026-04-19 — Stage 4.2 final-heap positive-count wording sync

I tightened the Stage 4.2 progress wording so the local collection helper's final-heap positivity theorem stays named explicitly alongside its origin/positive-count companion.

Planned update:
- sync `PLAN.md` and `plan/phase-4/02-formal-verification-depth.md` so `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` is named explicitly in the current boundary progress wording
- keep the claim narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening

Completed:
- the Stage 4.2 top-level and stage-specific plan notes now name `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly alongside the current RC snapshot companion inventory.
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

Completed:
- the browser and utility web-baseline interop corpus now also names `xstate` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`, and the related Stage 3.3 progress notes stay aligned with that narrower slice coverage.

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

Completed:
- the utility plain-package corpus now also exercises `axios` on the default standalone surface, and the Stage 3.3 progress notes now call that narrower coverage out explicitly.

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

Completed:
- the current proof-backed boundary summaries and Stage 4.2 progress notes now name `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly, so the final-heap positivity story stays direct rather than implied.

## 2026-04-19 — Stage 3.3 react-dom browser corpus widening

I found a small but concrete Stage 3.3 follow-up in the package-corpus breadth lane: `react-dom` is already covered in the browser web-baseline interop slice, but it still has room to exercise another representative browser package shape in the exports-map / browser-condition corpus.

Planned update:
- add `react-dom` to the browser exports-map and browser-condition corpus slices in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the browser representative-package breadth note names the new slice coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 4.2 no-dangling-reference summary sync

I found a small Stage 4.2 proof-summary drift gap: the published boundary already names `KaliCore.Safety.noDanglingReference`, but the current anti-drift guard and several summary docs did not pin it explicitly yet.

Planned update:
- add `KaliCore.Safety.noDanglingReference` to the RC snapshot theorem inventory in `crates/kali_cli/tests/schema_docs.rs`
- sync the summary/progress docs (`README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md`) so the base no-dangling theorem is named alongside the helper-level corollaries
- keep the claim narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening

Completed:
- The task is closed because `crates/kali_cli/tests/schema_docs.rs` already pins `KaliCore.Safety.noDanglingReference` alongside the rest of the RC snapshot theorem inventory, and the summary/progress docs already name it explicitly as well, so the base no-dangling theorem stays aligned with the helper-level corollaries.

## 2026-04-19 — Stage 3.3 framer-motion browser corpus widening

I found one more small Stage 3.3 follow-up in the representative browser package-corpus lane: add `framer-motion` to the browser web-baseline interop slice so the evidence set keeps widening without changing any support-rung claims.

Planned update:
- add `framer-motion` to `crates/kali_cli/tests/package_corpus.rs`
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention the new browser UI package
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 @storybook/react scoped-browser shape widening

I found a small follow-up in the scoped-browser package-shape lane: `@storybook/react` is already covered in the browser web-baseline interop slice, and we can keep the corpus breadth growing by exercising its exports-map and browser-condition shapes too.

Planned update:
- add `@storybook/react` to the scoped browser exports-map and browser-condition corpus slices in `crates/kali_cli/tests/package_corpus.rs`
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the progress notes call out the new scoped-browser slice coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change
## 2026-04-19 — Stage 4.2 target-origin theorem inventory sync

I found a small Stage 4.2 follow-up for the current proof-backed boundary summary: the RC snapshot progress notes should keep the newly explicit target-cell origin/positive-count theorem bullets aligned with the canonical theorem inventory.

Planned update:
- sync `PLAN-4.2-STATUS.md` and `TODO.md` so the target-cell origin/positive-count theorem is called out explicitly in the current Stage 4.2 progress wording, matching the theorem inventory added to `proofs/BOUNDARY.md`
- keep the claim narrow: this is a documentation / anti-drift sync for the published boundary, not a boundary widening

Completed:
- `PLAN-4.2-STATUS.md` and `TODO.md` now call out `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` explicitly in the current Stage 4.2 progress wording, keeping the progress notes aligned with the theorem inventory in `proofs/BOUNDARY.md`.

## 2026-04-19 — Stage 3.3 vue browser web-baseline corpus widening

I widened the Stage 3.3 browser web-baseline interop corpus one step further by adding `vue` to the browser package coverage, keeping the representative app-framework breadth concrete without changing support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `vue` alongside the existing browser web-baseline package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser web-baseline interop corpus now also names `vue` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md`.

## 2026-04-19 — Stage 3.3 @jridgewell/sourcemap-codec corpus widening

I found another small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add `@jridgewell/sourcemap-codec` to the browser web-baseline interop corpus so the representative utility/package-shape coverage keeps widening without changing any support-rung claims.

Planned update:
- add `@jridgewell/sourcemap-codec` to the browser web-baseline interop coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the corpus breadth note names the new package-shape coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser web-baseline interop corpus now also names `@jridgewell/sourcemap-codec` explicitly in `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.
## 2026-04-19 — Stage 4.2 wellformedness / ownership / linear-memory corollary widening

I widened the RC snapshot proof slice with combined wellformedness/ownership/linear-memory corollaries for the release-only, decrement, and collection helpers: `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`.

Planned update:
- sync the proof-boundary manifest and verification summaries so the new combined corollaries are named explicitly alongside the current RC helper inventory
- keep the claim narrow: this is still helper-level RC proof widening, not the broader Stage 4.2 ownership/freeing target

Completed:
- the proof-boundary manifest, verification summaries, Stage 4.2 tracker, TODO notes, `PLAN.md`, and proof-summary guard now name the combined wellformedness/ownership/linear-memory corollaries explicitly alongside the current RC helper inventory.

## 2026-04-19 — Stage 3.3 lodash corpus widening

I widened the Stage 3.3 package corpus one more step by adding `lodash` to the utility plain-package, exports-map, string-exports, pattern-exports, and web-baseline slices, keeping the representative common CJS utility breadth concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention `lodash` alongside the existing utility package breadth notes
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the utility corpus and Stage 3.3 progress notes now also name `lodash` explicitly in the relevant package-breadth slices.

## 2026-04-19 — Stage 4.2 collection target origin/positivity wording sync

I added the new proof-backed RC helper theorem `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount` and synced the Stage 4.2 progress notes / plan summaries so the collection-target provenance and positive-count bridge is named explicitly alongside the existing target-allocation and origin/ownership helpers.

Planned update:
- keep the Stage 4.2 proof-summary inventory and plan tracker wording aligned if the collection-target helper widens again
- treat this as another proof-summary / anti-drift sync for the published boundary, not a broader ownership/freeing widening

Completed:
- `README.md` now names `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount` explicitly alongside the existing target-allocation and origin/ownership helpers, keeping the verification summary aligned with the published RC snapshot boundary.

## 2026-04-19 — Stage 3.3 mobx corpus wording sync

I noticed the Stage 3.3 package corpus already exercises `mobx`, but the top-level follow-up summary doesn't call it out yet.

Planned update:
- sync `PLAN.md` and `TODO.md` so the Stage 3.3 follow-up wording names `mobx` explicitly alongside the existing browser web-baseline package breadth notes
- keep the claim narrow: this is a progress-summary wording sync for already-covered corpus evidence, not a new support-rung claim

Completed:
- the Stage 3.3 top-level follow-up notes now name `mobx` explicitly, keeping the summary aligned with the existing corpus evidence.

## 2026-04-19
- Requested follow-up: widen the Stage 3.3 package-corpus evidence with `vite` and sync the progress notes accordingly.
- Suggested concrete change: add `vite` to the utility corpus plain-package and web-baseline coverage in `crates/kali_cli/tests/package_corpus.rs`, then update the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and the matching tracker entries in `TODO.md` so the docs reflect the new evidence slice.

## 2026-04-19 — Stage 3.3 @tanstack/table-core corpus widening

I found one more small Stage 3.3 package-corpus widening that fits the current breadth follow-up lane: add `@tanstack/table-core` to the browser corpus so the representative scoped table-package coverage keeps widening without changing any support-rung claims.

Planned update:
- add `@tanstack/table-core` to the browser web-baseline interop slice and the scoped browser exports-map / browser-condition slices in `crates/kali_cli/tests/package_corpus.rs`
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the corpus-breadth note names the new package explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed: `crates/kali_cli/tests/package_corpus.rs` now exercises `@tanstack/table-core` in the browser web-baseline interop slice and the scoped browser exports-map / browser-condition slices, and the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name it explicitly alongside the existing package-corpus widening notes.

## 2026-04-19 — Stage 3.3 @mantine/core scoped-browser shape widening

I found a small Stage 3.3 follow-up in the browser package-shape lane: `@mantine/core` is already covered in the browser web-baseline interop slice and the utility plain-package corpus, and we can keep the corpus breadth growing by exercising its exports-map and browser-condition shapes too.

Planned update:
- add `@mantine/core` to the scoped browser exports-map and browser-condition corpus slices in `crates/kali_cli/tests/package_corpus.rs`
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the progress notes call out the new scoped-browser slice coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change
Completed:
- the scoped browser exports-map and browser-condition corpus now also names `@mantine/core` explicitly in `crates/kali_cli/tests/package_corpus.rs`, and the related Stage 3.3 progress notes now call out that narrower slice coverage alongside the existing browser package breadth notes.

## 2026-04-19 — Stage 4.2 collection target ownership/linear-memory companion widening

I widened the Stage 4.2 proof-backed RC snapshot slice one step further by adding the collection-target ownership theorem's linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, keeping the collection-path provenance story explicit without changing the broader ownership/freeing target.

Planned update:
- sync `proofs/KaliCore/Safety.lean`, `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and the proof-summary drift guard so the new collection-target companion theorem is named explicitly
- keep the claim narrow: this is still a companion-theorem widening on the published RC snapshot slice, not the full Stage 4.2 ownership/freeing target

Completed:
- the collection-target ownership theorem now has an explicit linear-memory companion in the proof-backed boundary, and the summary docs and anti-drift guard now name it explicitly.

## 2026-04-19 — Stage 3.3 utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `superjson` and `@jridgewell/sourcemap-codec` to the utility plain-package corpus on the default standalone surface, so the representative pure-JS utility breadth keeps growing without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `plan/phase-1/sum.md` so the Stage 3.3 progress notes explicitly mention the new utility corpus coverage
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the utility plain-package corpus now also exercises `superjson` and `@jridgewell/sourcemap-codec` on the default standalone surface, and the Stage 3.3 progress notes now call that coverage out explicitly.
