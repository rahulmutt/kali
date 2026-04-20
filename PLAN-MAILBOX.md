Subject: Stage 3.1 release-advanced cross-owner reuse widening

## 2026-04-20 — Stage 3.1 release-advanced cross-owner reuse widening

I widened the Stage 3.1 specialization evidence one step further by adding a `release-advanced` regression that proves identical generic specializations are still reused across layout-specialized owners, matching the existing release-mode cross-owner reuse shape while keeping the budget story deterministic.

Planned update:
- add a `release-advanced` counterpart to the existing cross-owner generic-reuse regression in `crates/kali_optimize/src/tests.rs`
- sync the Stage 3.1 progress notes in `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the new advanced-mode reuse coverage is named explicitly
- keep the claim narrow: this is still a specialization-depth widening within the existing optimizer model, not a new support-rung claim

Completed:
- `crates/kali_optimize/src/tests.rs` now also exercises the `release-advanced` cross-owner generic-reuse path across layout-specialized wrappers.
- The Stage 3.1 progress notes now name the new advanced-mode reuse coverage explicitly.
- Kept the update narrow: this widens specialization evidence within the existing optimizer model; it does not change the published support or benchmark claims.

Subject: Stage 3.3 @vueuse/core scoped-browser corpus widening

## 2026-04-20 — Stage 3.3 @vueuse/core scoped-browser corpus widening

I widened the Stage 3.3 package corpus one more step by adding `@vueuse/core` to the scoped browser exports-map and browser-condition slices, so the representative scoped browser utility breadth is now explicit on top of the existing web-baseline coverage.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now also exercises `@vueuse/core` in the scoped browser exports-map and browser-condition slices.
- `PLAN.md`, `plan/phase-3/03-ecosystem-breadth.md`, `plan/phase-1/sum.md`, and `TODO.md` now name the scoped-browser `@vueuse/core` widening explicitly.
- Kept the update narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change.

Subject: Stage 4.2 verification depth wording sync

## 2026-04-20 — Stage 4.2 verification depth wording sync

I tightened the top-level Stage 4.2 verification-depth summary so `PLAN.md`, `plan/phase-4/02-formal-verification-depth.md`, and the matching tracker note now name the collection helper's `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, and `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory` companions explicitly alongside the existing RC snapshot inventory.

Completed:
- `PLAN.md` and `plan/phase-4/02-formal-verification-depth.md` now call out the collection target-cell iff bridge, target-cell allocation corollary, and heap-filter + linear-memory companion explicitly in the Stage 4.2 verification-depth follow-up lane.
- Kept the update narrow: this is a plan-summary wording sync for the published boundary, not a boundary widening.

Subject: Stage 3.3 tailwindcss utility corpus widening

## 2026-04-20 — Stage 3.3 tailwindcss utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `tailwindcss` to the utility plain-package corpus on the default standalone surface, so the representative build-tool breadth now keeps growing without changing any support-rung claims.

Planned update:
- add `tailwindcss` to `crates/kali_cli/tests/package_corpus.rs` in the utility plain-package corpus slice
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the new standalone `tailwindcss` coverage is named explicitly
- keep the claim narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now also exercises `tailwindcss` in the utility plain-package corpus on the default standalone surface, and the tracker docs now call the new `tailwindcss` coverage out explicitly.
- Kept the update narrow: this widens the package corpus within the existing support-rung model; it does not change the documented support rungs.

Subject: Stage 3.3 @emotion/styled utility corpus widening

## 2026-04-20 — Stage 3.3 @emotion/styled utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `@emotion/styled` to the utility plain-package and web-baseline interop slices, so the representative scoped UI package breadth is now explicit on the default standalone surface too.

Planned update:
- add `@emotion/styled` to `crates/kali_cli/tests/package_corpus.rs` in the utility plain-package and web-baseline interop corpus slices
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new standalone `@emotion/styled` coverage is named explicitly
- keep the claim narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change

Completed:
- the utility plain-package corpus now also exercises `@emotion/styled` on the default standalone surface, and the Stage 3.3 progress notes now call that coverage out explicitly.

## 2026-04-20 — Stage 4.2 soundness-helper naming sync

I found one remaining proof-tree helper that the proof summary docs should name explicitly alongside `KaliCore.Soundness.subst_closed`: the literal-to-type helper `KaliCore.litTy` from `proofs/KaliCore/Types.lean`.

Completed:
- `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md` now name `KaliCore.litTy` alongside `KaliCore.Soundness.subst_closed` in the proof-summary / tracker prose.
- Kept the claim narrow: this is a proof-summary wording sync for the published boundary, not a widening of the proof-backed claim surface.

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

Completed:
- the utility plain-package corpus now also exercises `chart.js` on the default standalone surface, and the Stage 3.3 progress notes now call that coverage out explicitly.

## 2026-04-19 — Stage 3.3 @playwright/test browser corpus widening

I’m widening the Stage 3.3 package corpus one step further by adding `@playwright/test` to the browser web-baseline interop slice, so the representative browser test-runner breadth keeps growing without changing any support-rung claims.

Planned update:
- add `@playwright/test` to `crates/kali_cli/tests/package_corpus.rs`
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`
- keep the claim narrow: this is a corpus/evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `@playwright/test` in the browser web-baseline interop slice.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name the `@playwright/test` browser corpus widening explicitly.
- Kept the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

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

## 2026-04-19 — Stage 3.3 redux utility/browser corpus widening

I widened the Stage 3.3 package corpus a bit further by adding `redux` to the utility exports-map, string-exports, and pattern-exports slices, and now also to the browser web-baseline interop slice, keeping the package-corpus breadth note concrete without changing any support-rung claims.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the Stage 3.3 progress notes explicitly mention the new utility-shape coverage and the browser web-baseline interop slice
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser web-baseline interop corpus now also exercises `redux`, and the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now call that browser state-management coverage out explicitly alongside the existing utility `redux` shape coverage.
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

## 2026-04-19 — Stage 3.3 @radix-ui/react-dialog scoped-browser shape widening

I widened the Stage 3.3 package corpus one step further by adding `@radix-ui/react-dialog` to the scoped browser exports-map and browser-condition slices, so the representative dialog package breadth now stays concrete across both the browser command path and the scoped browser shape coverage.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new scoped-browser dialog coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 luxon corpus widening

I’m widening the Stage 3.3 package corpus one more step by adding `luxon` to the browser web-baseline interop, utility plain-package, utility web-baseline interop, and utility module-entry slices, so the representative date-time utility breadth stays concrete without changing any support-rung claims.

Planned update:
- add `luxon` to `crates/kali_cli/tests/package_corpus.rs` in the browser and utility corpus slices noted above
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new `luxon` coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-19 — Stage 3.3 jotai browser exports-map/browser-condition widening

I widened the Stage 3.3 browser package-shape corpus one step further by adding `jotai` to the browser exports-map and browser-condition slices, keeping the representative browser state-management breadth concrete without changing any support-rung claims.

Planned update:
- add `jotai` to `crates/kali_cli/tests/package_corpus.rs` in the browser exports-map and browser-condition slices
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new `jotai` shape coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser exports-map and browser-condition corpus now also exercises `jotai`, and the corresponding progress notes now call out that narrower shape coverage explicitly.

## 2026-04-20 — Stage 3.3 recharts browser web-baseline widening

I widened the Stage 3.3 package corpus one more step by adding `recharts` to the browser web-baseline interop slice, keeping the representative browser charting package breadth concrete without changing any support-rung claims.

Planned update:
- add `recharts` to `crates/kali_cli/tests/package_corpus.rs` in the browser web-baseline interop slice
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new `recharts` coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser web-baseline interop corpus now also exercises `recharts`, and the related Stage 3.3 progress notes now call out that narrower charting-package coverage explicitly.

## 2026-04-20 — Stage 3.3 rxjs browser/utility corpus widening

I widened the Stage 3.3 package corpus one more step by adding `rxjs` to the browser corpus and browser web-baseline interop slice, while the utility plain-package coverage already existed, keeping the representative observable/stream package breadth concrete without changing any support-rung claims.

Planned update:
- add `rxjs` to `crates/kali_cli/tests/package_corpus.rs` in the browser corpus and browser web-baseline interop slices
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new `rxjs` coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- the browser corpus now also exercises `rxjs`, the browser web-baseline interop corpus now also exercises `rxjs`, and the related Stage 3.3 progress notes now call out that narrower coverage explicitly.

## 2026-04-20 — Stage 3.3 @tanstack/router scoped-browser shape widening

I widened the Stage 3.3 package corpus one step further by adding `@tanstack/router` to the scoped browser exports-map and browser-condition slices, so the representative routing-package breadth stays concrete across both browser shape coverage paths without changing any support-rung claims.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `@tanstack/router` in the scoped browser exports-map and browser-condition slices.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name the scoped-browser `@tanstack/router` slices explicitly.
- Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

## 2026-04-20 — Stage 3.3 timing/microtask web-baseline widening

I widened the Stage 3.3 package corpus one more step by adding `performance.now()` and `queueMicrotask` to the shared web-baseline interop source, so the deterministic browser/runtime timing and microtask baseline is now explicitly covered in the package-corpus tests.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `performance.now()` and `queueMicrotask` in the shared web-baseline interop source.
- `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` now name the same helper widening explicitly.
- Kept the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

## 2026-04-20 — Stage 3.3 @emotion/react utility plain-package widening

I widened the Stage 3.3 package corpus one more step by adding `@emotion/react` to the utility plain-package corpus on the default standalone surface, so one more scoped UI package stays concrete through the standalone command path without changing any support-rung claims.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `@emotion/react` in the utility plain-package corpus on the default standalone surface.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `TODO.md`, and `plan/phase-1/sum.md` now name the new utility corpus coverage explicitly.
- Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model; it does not change the documented support rungs.

## 2026-04-20 — Stage 3.3 react-helmet-async browser web-baseline widening

I widened the Stage 3.3 package corpus one more step by adding `react-helmet-async` to the browser web-baseline interop slice, keeping the representative head-management package breadth concrete without changing any support-rung claims.

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `react-helmet-async` in the browser web-baseline interop slice.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name the widening explicitly.
- Kept the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

MIR specialization follow-up: allow generic specialization inside MIR-specialized clones so layout-specialized functions can still clone and fold large generic callees after the MIR pass narrows the arguments.

Completed: release-mode MIR-specialized clones now keep generic specialization enabled inside their bodies, so a layout-specialized wrapper can still clone and fold a large generic callee after the MIR pass narrows its arguments.

## 2026-04-20 — Stage 3.3 ajv browser/utility corpus widening

I found one more small Stage 3.3 package-corpus widening that fits the current breadth follow-up lane: add `ajv` to the browser web-baseline interop slice and the utility plain-package slice so the representative validation-package coverage stays concrete on both the browser and default standalone surfaces without changing any support-rung claims.

Planned update:
- add `ajv` to `crates/kali_cli/tests/package_corpus.rs` in the browser web-baseline interop and utility plain-package corpus slices
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the Stage 3.3 progress notes name the new `ajv` coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now also exercises `ajv` in the browser web-baseline interop slice and the utility plain-package slice.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` now name the new `ajv` coverage explicitly.
- Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

[2026-04-20] Follow-up widening note: add a new browser web-baseline interop package-corpus case for @stripe/react-stripe-js and mirror the evidence in the Stage 3.3 progress notes / TODO tracker. Support rungs stay unchanged; this is corpus breadth only.


Subject: Stage 3.3 @stripe/react-stripe-js scoped-browser widening

## 2026-04-20 — Stage 3.3 @stripe/react-stripe-js scoped-browser widening

I found a small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add `@stripe/react-stripe-js` to the scoped browser exports-map and browser-condition slices so the representative browser payment/UI package coverage keeps widening without changing any support-rung claims.

Planned update:
- add `@stripe/react-stripe-js` to the scoped browser exports-map and browser-condition coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the corpus-breadth note names the new package-shape coverage explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now exercises `@stripe/react-stripe-js` in the browser web-baseline interop slice and the scoped browser exports-map/browser-condition slices.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `TODO.md`, and `plan/phase-1/sum.md` now name the `@stripe/react-stripe-js` widening explicitly.
- Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

## 2026-04-20 — Stage 3.3 scoped utility corpus widening

I found a small Stage 3.3 follow-up in the scoped utility corpus breadth lane: add `@babel/runtime` and `@npmcli/package-json` to the default-standalone scoped-package slice so the representative scoped utility package coverage keeps widening without changing any support-rung claims.

Planned update:
- add `@babel/runtime` and `@npmcli/package-json` to the scoped utility corpus coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the new scoped utility corpus breadth is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` already carries `@babel/runtime` and `@npmcli/package-json` in the utility scoped-package slice on the default standalone surface.
- `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` now name that scoped utility corpus widening explicitly.
- Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

## 2026-04-20 — Stage 3.3 deepmerge utility corpus widening

I found a small Stage 3.3 follow-up in the utility plain-package breadth lane: add `deepmerge` to the default standalone package corpus so the representative pure-JS package coverage keeps widening without changing any support-rung claims.

Planned update:
- add `deepmerge` to the utility plain-package coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the new plain-package corpus breadth is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-20 — Stage 3.3 @tanstack/query-core corpus widening

I widened the Stage 3.3 package corpus one more step by adding `@tanstack/query-core` to the browser web-baseline interop and utility plain-package slices, so the representative scoped query-package breadth now stays concrete on both the browser and default standalone surfaces without changing any support-rung claims.

Planned update:
- add `@tanstack/query-core` to `crates/kali_cli/tests/package_corpus.rs` in the browser web-baseline interop and utility plain-package corpus slices
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the new `@tanstack/query-core` coverage is named explicitly
- keep the claim narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change

Completed:
- `crates/kali_cli/tests/package_corpus.rs` now also exercises `@tanstack/query-core` in the browser web-baseline interop and utility plain-package slices, and the tracker docs now call the new coverage out explicitly.
- Kept the update narrow: this widens the package corpus within the existing support-rung model; it does not change the documented support rungs.

## 2026-04-20 — Stage 4.2 no-dangling-reference plan sync

I completed the remaining no-dangling-reference wording sync on the plan side by naming `KaliCore.Safety.noDanglingReference` explicitly in the Stage 4.2 verification-depth follow-up lane in `PLAN.md`, keeping the top-level plan aligned with the published boundary inventory and the existing status tracker / TODO notes.

Completed:
- `PLAN.md` now names `KaliCore.Safety.noDanglingReference` explicitly in the Stage 4.2 verification-depth follow-up lane alongside the helper-level no-dangling-reference corollaries.
- `TODO.md` now records the plan-lane sync alongside the earlier status-tracker sync.
- No spec update was needed; this was a plan-summary anti-drift sync only.

Subject: Stage 3.3 react-router-dom browser exports-map widening

## 2026-04-20 — Stage 3.3 react-router-dom browser exports-map widening

I widened the Stage 3.3 package corpus one more step by adding `react-router-dom` to the browser exports-map slice, so the browser router corpus now has one more representative package shape without changing any support-rung claims.

Planned update:
- add `react-router-dom` to `crates/kali_cli/tests/package_corpus.rs` in the browser exports-map router slice
- sync the Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` so the new `react-router-dom` coverage is named explicitly
- keep the claim narrow: this is still a corpus/evidence widening within the existing package-support model, not a support-rung change

Subject: Stage 3.3 xstate exports-map/pattern-exports widening

## 2026-04-20 — Stage 3.3 xstate exports-map/pattern-exports widening

I found a small Stage 3.3 follow-up that fits the current package-corpus breadth lane: add `xstate` to the browser and utility exports-map and pattern-exports slices so the representative state-management package-shape coverage stays concrete in the shape-based corpora as well as the web-baseline interop slices.

Planned update:
- add `xstate` to the browser and utility exports-map / pattern-exports coverage in `crates/kali_cli/tests/package_corpus.rs`
- sync the corresponding Stage 3.3 progress notes in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the new xstate shape coverage is named explicitly
- keep the claim narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change

## 2026-04-20 — Stage 3.1 cross-module-style re-export specialization reuse

I found a small Stage 3.1 follow-up that fits the remaining specialization-depth lane: add a regression proving the release-mode generic specialization cache still reuses the same helper clone when the call chain runs through a longer re-export-style wrapper chain, so the cross-module-style reuse story stays concrete without changing any support-rung claims.

Planned update:
- add a release-mode optimizer regression in `crates/kali_optimize/src/tests.rs` that exercises a `public` / `bridge` / helper re-export chain and asserts the same generic helper specialization is reused once across the chain
- sync the Stage 3.1 progress notes in `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, `plan/phase-1/sum.md`, and `TODO.md` so the re-export-chain specialization coverage is named explicitly
- keep the claim narrow: this is still a specialization-depth evidence widening within the existing optimizer model, not a new support-rung or phase claim

## 2026-04-20 — Stage 3.1 explicit public/bridge/helper re-export-chain widening

I completed the pending Stage 3.1 cross-module-style re-export follow-up by turning the optimizer regression into an explicit `public` → `bridge` → helper chain, so the release-mode specialization cache now has a concrete bridge wrapper in the path it reuses.

Completed:
- `crates/kali_optimize/src/tests.rs` now exercises an explicit `public` / `bridge` / helper chain and asserts that `bridge`, `module_helper`, and `math_add` each specialize once while the two public wrappers still reuse the same underlying clones.
- The Stage 3.1 progress notes already call out the re-export-chain widening; I will keep them aligned with the new explicit bridge wrapper wording where needed.
- No spec update was needed; this is a specialization-depth evidence widening within the existing optimizer model.

### Stage 3.3 - @reduxjs/toolkit scoped utility corpus widening
- Noted that `crates/kali_cli/tests/package_corpus.rs` now also exercises `@reduxjs/toolkit` in the utility scoped-package corpus on the default standalone surface, so the representative scoped utility package breadth stays concrete without changing the documented support-rung story.
- Kept the progress wording aligned in `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md`.
