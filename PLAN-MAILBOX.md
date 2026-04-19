## 2026-04-19 — Stage 3.3 browser web-baseline corpus widening sync

I widened the Stage 3.3 browser web-baseline interop corpus with `@emotion/react`, so the representative package set now carries one more scoped browser package name without changing the support-rung story.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the browser web-baseline corpus notes name `@emotion/react` explicitly
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 browser web-baseline corpus widening sync

I widened the Stage 3.3 browser web-baseline interop corpus with `@radix-ui/react-dialog`, so the representative package set now carries one more scoped browser package name without changing the support-rung story.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the browser web-baseline corpus notes name `@radix-ui/react-dialog` explicitly
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 CustomEvent corpus/progress sync

I found a small Stage 3.3 documentation drift: the browser/web-baseline corpus and TODO note already treat `CustomEvent` as part of the widening slice, but the Stage 3.3 progress note in `plan/phase-3/03-ecosystem-breadth.md` still omits it from the representative browser-interop line.

Planned update:
- sync `plan/phase-3/03-ecosystem-breadth.md` so the browser package-corpus progress note names `CustomEvent` explicitly alongside `AbortController`, `EventTarget`, `structuredClone`, and `FileReader`
- keep the claim narrow: this is a progress-note / corpus-coverage wording sync, not a support-rung change

## 2026-04-19 — Stage 3.3 browser router corpus widening sync

I synced the Stage 3.3 progress notes so the browser corpus breadth tracker now names the router representatives (`vue-router` and `react-router`) explicitly as one browser-router widening slice.

Planned update:
- keep `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` aligned if the browser router corpus widens again
- keep the claim narrow: this is still corpus / interoperability breadth work, not a support-rung change

## 2026-04-19 — Stage 3.3 browser router corpus widening

I widened the Stage 3.3 browser corpus with `vue-router` in the exports-map and pattern-exports slices, so the representative package set now carries one more browser-oriented package shape without changing the support-rung story.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the corpus notes name `vue-router` explicitly
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 react-router corpus widening

I widened the Stage 3.3 browser corpus with `react-router` across the web-baseline interop, exports-map, and pattern-exports slices, so the representative package set now carries one more browser-router package shape without changing the support-rung story.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the corpus notes name `react-router` explicitly
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 representative package corpus widening

I widened the Stage 3.3 package-corpus follow-up with `zod` as one more representative package name across the browser and utility web-baseline interop slices, so the ongoing breadth note keeps reflecting incremental corpus growth without changing the documented support rungs.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the corpus notes name `zod` explicitly
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change
## 2026-04-19 — Stage 3.1 nested MIR-specialization depth regression

I widened the Stage 3.1 specialization follow-up with a nested-call regression: `release_recursively_specializes_nested_mir_call_sites` now proves a specialized MIR clone can expose a second specializable call site inside its own body. That keeps the deeper monomorphisation path regression-tested while preserving the deterministic specialization budget story.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md` and `TODO.md` so the Stage 3.1 progress notes call out the nested specialization depth regression explicitly
- keep the claim narrow: this is a specialization-depth widening inside the existing optimizer model, not a new support-rung claim

## 2026-04-19 — Stage 3.3 scoped browser typed-export representative widening

I widened the Stage 3.3 browser typed-export-branch corpus with `@tanstack/react-query`, then synced `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the representative package corpus now names one more scoped browser package without changing the support-rung story.

Suggested follow-up:
- keep the Stage 3.3 progress note and TODO tracker aligned if the representative browser corpus widens again; this remains a corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 4.2 decrement target positive-count iff bridge

I widened the RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`, a helper theorem that makes the decrement target's positive-count status after `releaseAndDecrement` explicit as an iff bridge against the original count.

Planned update:
- sync the proof-boundary manifest, status tracker, and verification summary docs so the new decrement-path iff theorem is named explicitly alongside the current RC helper inventory
- keep the claim narrow: this is a helper-level proof widening on top of the existing RC snapshot slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-19 — Stage 3.3 browser storage baseline widening

I widened the browser support library with an in-memory `localStorage` / `sessionStorage` simulation and reexported the storage helpers through the Deno compatibility surface. That gives the Stage 3.3 browser-interoperability follow-up one more concrete baseline primitive without changing the support-rung story.

Planned update:
- sync `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the Stage 3.3 progress notes name the new in-memory storage baseline explicitly
- keep the claim narrow: this is a browser-interop helper widening, not a new browser-runtime availability claim

## 2026-04-19 — Stage 4.2 linear-memory payload follow-up sync

I updated the top-level follow-up lane and the Stage 4.2 status tracker so they now call out the explicit linear-memory payload preservation corollaries as already included in the published proof-backed boundary. That keeps the remaining Stage 4.2 widening language focused on the work that still sits beyond the payload bridge.

Planned update:
- keep `PLAN.md` and `PLAN-4.2-STATUS.md` aligned with the proof-backed boundary wording whenever the RC snapshot slice widens again
- keep the claim narrow: this is a summary-doc sync for the existing published boundary, not a boundary widening

## 2026-04-19 — Stage 1.5 type-checker diagnostics sync

I updated the lightweight type-checker facade so `TypeChecker::typecheck` drains any pending annotation-resolution diagnostics from the shared context before returning. That keeps the Stage 1.5 error story explicit at the facade boundary instead of leaving the method as a pure no-op clone.

Planned update:
- sync `plan/phase-1/05-type-checker.md` and `TODO.md` so the Stage 1.5 progress notes name the pending-diagnostics drain explicitly
- keep the claim narrow: this is a diagnostics-plumbing sync for the existing Stage 1.5 contract, not a phase-level availability change

## 2026-04-18 — Stage 3.1 tagged-parameter specialization widening

I widened the MIR-aware specialization path so `kali_optimize` can now specialize tagged parameters when the actual call arguments have a concrete, stable layout or literal shape. That lets the optimizer revisit a deeper monomorphisation slice instead of stopping at the existing non-tagged-layout path.

Planned update:
- sync `TODO.md` so the Stage 3.1 progress notes name the tagged-parameter specialization widening explicitly
- keep the claim narrow: this is a specialization-depth widening, not a new support-rung claim

## 2026-04-18 — Stage 3.3 browser corpus representative widening

I widened the Stage 3.3 browser corpus with an additional `solid-js` representative in the exports-map slice, so the package corpus now carries one more browser-oriented package name without changing the support-rung story.

Planned update:
- sync `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the Stage 3.3 progress notes name the additional browser representative explicitly
- keep the claim narrow: this is a corpus widening, not a new support rung

## 2026-04-18 — Stage 4.2 pure release-origin helper widening

I found a small proof-summary gap in the pure release helper slice: the RC snapshot boundary already names `KaliCore.Safety.releaseRefHeapCharacterisation` and `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, but it does not yet call out the plain origin theorem `KaliCore.Safety.releaseRefHeapCellOrigin` explicitly.

Planned update:
- add `KaliCore.Safety.releaseRefHeapCellOrigin` to `proofs/KaliCore/Safety.lean`, then sync the Stage 4.2 plan/progress notes (`plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md`) so the pure release-helper provenance story is explicit at the same granularity as the decrement and collection helper families
- keep the claim narrow: this is a helper-theorem / progress-tracker sync for the published boundary, not a boundary widening beyond the current RC snapshot model

## 2026-04-18 — Stage 4.2 heap-characterisation anti-drift guard

I tightened the proof-boundary anti-drift test so `crates/kali_cli/tests/schema_docs.rs` now explicitly pins the heap-characterisation theorem names `releaseAndDecrementHeapCharacterisation` and `releaseAndCollectHeapCharacterisation` alongside the existing RC summary guards.

Suggested follow-up:
- keep `PLAN-4.2-STATUS.md` and `TODO.md` aligned with the stronger proof-summary guard if the published boundary widens again
- continue treating this as a wording / drift-guard sync, not a boundary widening

## 2026-04-17 — Stage 3.3 browser replacement-map coverage

I widened the browser-targeted package-resolution path so `package.json#browser` replacement maps are honored after entry selection, including exact-path rewrites and `false` blocks, and I plan to keep the Stage 3.3 corpus/status notes aligned with that browser-resolution coverage.

## 2026-04-17 — Stage 4.2 heap-characterisation follow-up

I widened the current RC snapshot proof slice with exact heap-membership characterisations for `releaseAndDecrementHeapCharacterisation` and `releaseAndCollectHeapCharacterisation`, so the helper-level RC story now states the decrement/collection membership shape directly instead of only through the origin and filter corollaries.

Suggested follow-up:
- keep the Stage 4.2 progress notes aligned if the helper-level RC slice widens again
- continue treating this as a helper-level RC slice; the broader ownership/freeing target is still wider than the current proof-backed boundary

Suggested follow-up:
- sync `PLAN-3.3-STATUS.md` and `TODO.md` so the Stage 3.3 progress tracker explicitly names browser replacement-map coverage alongside the existing exports-map and browser-condition package corpus cases
- keep the broader Stage 3.3 corpus widening incremental; this is still a browser-resolution coverage slice, not a new package-support rung

## 2026-04-17 — Stage 4.2 decrement positive-count plan sync

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells` and synced the Stage 4.2 progress trackers in `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md` so the plan now names the decrement-path positive-count preservation theorem alongside the existing RC helper slice.

Suggested follow-up:
- keep the Stage 4.2 progress trackers aligned if the decrement helper widens again
- continue treating this as a helper-level RC slice; the broader ownership/freeing target is still wider than the current proof-backed boundary

## 2026-04-17 — Stage 4.2 origin/positivity conjunction helper

I widened the current RC snapshot proof slice with a small helper theorem that bundles the surviving-cell origin and positive-count facts for `releaseAndCollect`, so the local collection story now has a single reusable conjunction lemma on top of the existing origin and positivity theorems.

Suggested follow-up:
- keep the Stage 4.2 progress trackers aligned if the local collection helper widens again
- continue treating this as a helper-level RC slice; the broader ownership/freeing target is still wider than the current proof-backed boundary

## 2026-04-17 — Stage 3.3 package-corpus expansion

I expanded the Stage 3.3 package corpus tests with exports-map / subpath coverage for the browser, utility, and Node-runner cases, and then added dual-package / mixed-format coverage so the corpus now exercises conditional exports plus mixed CJS/ESM entrypoints instead of only single-entrypoint stubs.

Suggested follow-up:
- keep the stage progress notes aligned with the corpus tests whenever another representative shape is added
- continue broadening the corpus as new package shapes are triaged

## 2026-04-17 — Stage 4.2 target-cell retention follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, which makes the local collection helper's released-target retention behavior explicit when the decremented count remains positive.

Suggested follow-up:
- sync `PLAN-4.2-STATUS.md` and the proof-boundary / verification summary docs so the progress tracker names the new target-cell retention theorem
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level retention theorem, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 plan-progress sync

I synced the Stage 4.2 plan progress note so it now names the full local RC helper slice, including the zero-count removal / origin / positive-count heap theorems and the release-and-decrement bookkeeping corollaries.

Suggested follow-up:
- keep the Stage 4.2 progress note aligned with the published boundary inventory whenever the proof slice widens again

## 2026-04-17 — Stage 4.2 status-summary sync

I synced `PLAN-4.2-STATUS.md` and the TODO summary bullet so the RC snapshot inventory now explicitly names `releaseAndCollectRemovesZeroCountCells` alongside the other zero-count collection theorems.

Suggested follow-up:
- keep the Stage 4.2 status summary and the published proof boundary inventory aligned if the zero-count collection slice widens again

## 2026-04-17 — Stage 4.2 DoD checklist sync

I marked the Stage 4.2 formal-verification depth checklist complete in `plan/phase-4/02-formal-verification-depth.md` so the plan document now reflects the proof-backed boundary state described in `proofs/BOUNDARY.md`.

Suggested follow-up:
- keep the Stage 4.2 status tracker and proof-boundary inventory synchronized if the mechanized slice widens again

## 2026-04-17 — Stage 4.2 original zero-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, which makes the local release-and-collect helper's original zero-count filtering behavior explicit.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `PLAN-4.2-STATUS.md` so the published boundary inventory names the new original-zero-count helper theorem
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level no-leak slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 heap-characterisation sync

I added `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, which characterises the local collection helper's heap as exactly the positive-count filter of the decrement pass.

Suggested follow-up:
- keep the broader ownership/freeing widening incremental; this remains a helper-level local collection fact, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count-removal sync

I synced the Stage 4.2 progress summary so the current RC snapshot proof slice now names `releaseAndCollectDropsZeroCountCells` explicitly alongside the other local collection-helper theorems.

Suggested follow-up:
- keep the later ownership/freeing widening incremental; this is still a local helper-level slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count freeing follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, which makes the local collection helper's zero-count removal behavior explicit in the theorem inventory.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new zero-count-removal lemma
- keep the story incremental: this is still a local freeing slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count collection follow-up

The RC snapshot proof slice now includes a local freeing step: `releaseAndCollect` filters zero-count cells after the decrement pass, and the proof boundary should now mention that collection helper alongside the existing release/decrement bookkeeping.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker matches the new theorem inventory
- keep the story incremental: this is still a local zero-count collection slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect disjointness follow-up

I added the explicit `releaseAndCollectReleasedNotLiveRef` theorem to the RC snapshot slice, then synced `PLAN-4.2-STATUS.md` and `TODO.md` so the Stage 4.2 progress tracker reflects the new theorem inventory.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing work incremental; the local collection helper is still a slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 releaseAndCollect recording follow-up

I added `releaseAndCollectRecorded` to the RC snapshot proof slice, so the Stage 4.2 progress tracker now includes explicit release-recording for the local collection helper alongside the existing zero-count collection and disjointness results.

Suggested follow-up:
- keep widening the memory-safety slice incrementally; the current helper-level claim is still narrower than the full ownership/freeing target

## 2026-04-17 — Stage 4.2 RC freeing follow-up

I plan to widen the Stage 4.2 memory-safety slice with a helper-level lemma showing that `releaseAndCollect` preserves positive-count cells from the decrement pass, so the current local collection story has an explicit "only zero-count cells are dropped" theorem alongside the existing target-cell removal result.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` and `TODO.md` so the progress tracker names the new positive-count preservation lemma
- if the boundary wording changes, sync the proof-boundary / verification summary docs in the same pass

## 2026-04-17 — Stage 4.2 releaseAndCollect positive-count follow-up

The proof-backed RC snapshot slice now includes `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, so the progress tracker should name the positive-count-preservation/no-leak helper explicitly alongside the existing zero-count collection and disjointness theorems.

Suggested follow-up:
- update the Stage 4.2 status / progress docs and TODO tracker to mention the new helper theorem
- keep the story incremental; this is still a local collection-helper slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect other-live preservation follow-up

I plan to widen the Stage 4.2 RC snapshot slice with a helper-level theorem that `releaseAndCollect` preserves other live references, so the progress tracker can explicitly name that local collection-helper invariant alongside the existing zero-count collection and disjointness results.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` and `TODO.md` so the tracker names the new local-helper preservation theorem
- if the boundary wording changes, sync the proof-boundary / verification summary docs in the same pass

## 2026-04-17 — Stage 4.2 live-reference ownership/allocation follow-up

I added helper corollaries that both the decrement path and the local collection helper preserve the ownership/allocation story for surviving live references (`releaseAndDecrementLiveRefsAreOwnedAndAllocated` and `releaseAndCollectLiveRefsAreOwnedAndAllocated`).

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress tracker explicitly names the helper-level ownership/allocation preservation corollaries
- keep the broader Stage 4.2 ownership/freeing target incremental; these are helper corollaries on top of the current proof-backed slice

## 2026-04-17 — Stage 4.2 helper corollary sync follow-up

I synced the Stage 4.2 progress tracker and TODO notes so the current proof-backed slice now names the helper-level ownership/allocation corollaries (`releaseAndDecrementLiveRefsAreOwnedAndAllocated` and `releaseAndCollectLiveRefsAreOwnedAndAllocated`) alongside the existing zero-count collection and disjointness bookkeeping.

Suggested follow-up:
- keep the remaining Stage 4.2 memory-safety widening incremental; this is still a helper-level slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 positive-count final-heap follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, so the proof-backed boundary now names the local collection helper's positive-count-only final heap property alongside the existing heap-characterisation theorem.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing story incremental; this remains a helper-level slice, not the full RC target

## 2026-04-17 — Stage 4.2 pure release-helper follow-up

I widened the current RC snapshot proof slice with pure release-helper corollaries (`releaseRefLiveRefsAreOwnedAndAllocated` and `releaseRefReleasedNotLiveRef`) so the proof-backed boundary now covers the release-only helper in addition to the decrement and local collection paths.

Suggested follow-up:
- sync `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new pure release-helper corollaries
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 original zero-count follow-up

I synced the Stage 4.2 progress note in `plan/phase-4/02-formal-verification-depth.md` so it now explicitly names `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells` alongside the existing local collection-helper theorems.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level no-leak slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 ownership-envelope preservation follow-up

I widened the Stage 4.2 RC snapshot proof slice with explicit ownership-envelope preservation theorems for the release-only, decrement, and collection helpers (`KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, and `KaliCore.Safety.releaseAndCollectPreservesOwnership`).

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new ownership-envelope preservation lemmas
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level ownership-map slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 release-and-decrement heap-origin follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementHeapCellOrigin`, which makes the decrement helper's surviving heap provenance explicit.

Suggested follow-up:
- sync `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the Stage 4.2 progress notes name the release-and-decrement heap-origin theorem alongside the other RC helper invariants
- keep the broader Stage 4.2 ownership/freeing target incremental; this remains a helper-level provenance theorem, not the full RC target

## 2026-04-17 — Stage 4.2 release-set monotonicity sync

I synced the Stage 4.2 progress trackers after widening the RC snapshot proof slice with the release-set preservation corollaries (`releaseRefPreservesReleasedRefs`, `releaseAndDecrementPreservesReleasedRefs`, and `releaseAndCollectPreservesReleasedRefs`).

Suggested follow-up:
- keep the broader Stage 4.2 memory-safety widening incremental; this is still a helper-level release-set monotonicity slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 live-reference filtering sync

I synced the Stage 4.2 proof-progress notes after adding exact live-reference filtering theorems for the release-only, decrement, and collection helpers.

Suggested follow-up:
- keep the proof boundary / progress notes aligned if the live-reference slice widens again
- keep the broader Stage 4.2 ownership/freeing story incremental; these are helper-level shape theorems, not the full RC story

## 2026-04-17 — Stage 4.2 ownership-provenance follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, so the helper-level memory-safety story now names the surviving release-and-collect cells' original ownership tag explicitly alongside provenance and positive-count tracking.

Suggested follow-up:
- keep the Stage 4.2 progress notes aligned if the helper-level RC slice widens again
- continue treating this as a helper-level RC slice; the broader ownership/freeing target is still wider than the current proof-backed boundary

## 2026-04-17 — Stage 4.2 origin/ownership/positivity bundle follow-up

I plan to widen the current RC snapshot proof slice with a bundled theorem for surviving `releaseAndCollect` heap cells that packages the original-heap provenance, name/ownership preservation, and positive-count fact together, so the Stage 4.2 progress notes can name the combined helper fact explicitly if that slice lands.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` and `TODO.md` if the bundled helper theorem lands, and keep the proof-boundary summary docs aligned with the new theorem inventory
- keep the broader Stage 4.2 ownership/freeing target incremental; this would still be a helper-level conjunction theorem, not the full RC target

## 2026-04-17 — Stage 4.2 release-and-decrement ownership follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, so the helper-level memory-safety story now names the decrement helper's surviving heap provenance together with its original ownership tag.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress trackers name the new decrement-helper provenance/ownership theorem alongside the existing RC helper inventory
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level provenance-and-ownership theorem, not the full RC target

## 2026-04-17 — Stage 4.2 exact releasedRefs bookkeeping follow-up

I plan to widen the current RC snapshot proof slice with exact `releasedRefs` cons-shape theorems for the release-only, decrement, and collection helpers, so the published boundary can make the release bookkeeping shape explicit alongside the existing live-reference and no-dangling corollaries.

Suggested follow-up:
- update the proof-boundary manifest and the verification summaries so the new releasedRefs bookkeeping theorems are named explicitly
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level bookkeeping slice, not the full RC target

## 2026-04-17 — Stage 3.3 module-only corpus coverage

I widened the Stage 3.3 package corpus with module-only entrypoint packages, so the representative browser and utility sets now exercise `package.json#module` fallback resolution as a published shape instead of only through mixed-format packages.

Suggested follow-up:
- keep the Stage 3.3 corpus/status notes aligned if the module-only coverage widens again
- continue treating this as a package-corpus widening slice, not a new support rung

## 2026-04-18 — Stage 4.2 unrelated-heap / other-live wording sync

I found the Stage 4.2 summary tracker still described the unrelated-heap and other-live helper slices generically, even though the published boundary already mechanizes them.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` so the progress note names `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` explicitly alongside the rest of the RC snapshot inventory
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 collection target-cell iff helper surfacing

I promoted the local collection helper's target-cell survival/removal split to the public theorem `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` and synced the Stage 4.2 plan/status trackers plus the proof-boundary summaries so the next memory-safety widening step stays explicit.

Suggested follow-up:
- keep the claim narrow: this is a proof-boundary / tracker sync for the published RC snapshot slice, not a widening of the broader ownership/freeing target


## 2026-04-18 — Stage 1.5 type-checker diagnostics sync

I widened the Stage 1.5 type-checker foundation slightly: `TypeChecker` now drains and preserves annotation-resolution diagnostics instead of behaving like a pure no-op, and the Stage 1.5 plan note now reflects that the annotation-diagnostics plumbing is wired.

Suggested follow-up:
- keep the Stage 1.5 summary honest if the checker grows beyond annotation diagnostics into flow-sensitive or inference-heavy passes
- treat this as a small plan/status sync, not a phase-level availability change

## 2026-04-19 — Stage 4.2 RC predicate vocabulary sync

I made the proof-boundary / verification summaries explicitly name the RC snapshot predicate vocabulary (`hasOwnership`, `allocated`, and `liveAnnotated`) so the plan/status wording stays aligned with the current model shape.

Suggested follow-up:
- keep the plan and proof-boundary wording aligned if the RC snapshot predicate vocabulary widens again; this is a documentation sync, not a boundary widening

## 2026-04-19 — Stage 4.2 RC predicate vocabulary anti-drift guard

I tightened the Stage 4.2 schema-docs anti-drift guard so `crates/kali_cli/tests/schema_docs.rs` now also pins the explicit RC predicate vocabulary names (`hasOwnership`, `allocated`, and `liveAnnotated`) alongside the published theorem inventory.

Suggested follow-up:
- keep the proof-boundary and status summaries aligned if the RC predicate vocabulary widens again; this remains a wording / anti-drift sync, not a boundary widening

## 2026-04-19 — Stage 3.3 scoped UI package widening

I widened the Stage 3.3 package-corpus evidence slightly by adding `@mui/material` to the scoped browser exports-map / browser-condition slices, so the representative browser corpus now covers one more popular UI package shape without changing the documented support rungs.

Suggested follow-up:
- keep the Stage 3.3 progress notes and TODO tracker aligned if the representative browser package corpus widens again; this remains a corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 FileReader browser-baseline follow-up

I added an in-memory `FileReader` baseline to `kali_api_web`, reexported it through `kali_api_deno`, taught the browser corpus source to instantiate it, and updated the Stage 3.3 progress notes / TODO tracker so the browser interop widening now names the new primitive explicitly.

Suggested follow-up:
- keep the Stage 3.3 corpus and progress notes aligned if the browser baseline widens again; this remains a browser-interoperability slice, not a support-rung change

## 2026-04-19 — Stage 4.2 decrement target positive-count iff bridge

I updated the progress tracker notes so the Stage 4.2 proof-summary inventory now calls out `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` explicitly.

Planned update:
- keep `TODO.md` and the Stage 4.2 status/progress docs aligned with the published boundary wording whenever the RC snapshot slice widens again
- keep the claim narrow: this is a helper-level proof-summary sync, not a boundary widening

## 2026-04-19 — Stage 3.3 browser web-baseline package widening

I widened the Stage 3.3 browser web-baseline interop corpus with `date-fns` and `lodash-es`, so the browser command path now covers two more representative utility-package names in addition to the existing browser/runtime baseline.

Planned update:
- sync `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` so the browser-runtime interop progress notes name the new browser corpus packages explicitly
- keep the claim narrow: this is a corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 typed-export-branch browser corpus widening

I widened the Stage 3.3 browser typed-export-branch corpus with `@floating-ui/react`, so the representative scoped browser package set now carries one more modern UI package shape through the existing browser exports/type-branch checks without changing the documented support rungs.

Planned update:
- sync `crates/kali_cli/tests/package_corpus.rs`, `plan/phase-3/03-ecosystem-breadth.md`, and `TODO.md` so the browser corpus notes name `@floating-ui/react` explicitly
- keep the claim narrow: this is a corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 headlessui browser-corpus widening

I widened the Stage 3.3 browser corpus with `@headlessui/react` across the web-baseline interop,
exports-map, and browser-condition slices, so the scoped browser package set now carries one more
representative UI package without changing the documented support rungs.

Planned update:
- keep `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` aligned with the new scoped browser
  corpus representative if the browser package set widens again
- keep the claim narrow: this is another corpus-widening slice, not a support-rung change

## 2026-04-19 — Stage 3.3 FormData browser-baseline follow-up

I widened the shared Web/Deno browser support library with an in-memory `FormData` baseline and reexported it through the Deno compatibility surface, so the Stage 3.3 browser-interop follow-up now has one more deterministic browser-form primitive without changing the support-rung story.

Planned update:
- sync `plan/phase-3/03-ecosystem-breadth.md` so the browser-interoperability progress note names `FormData` explicitly alongside the existing Blob/File/FileReader/storage and stub-surface widening
- keep the claim narrow: this is support-library baseline widening, not a public support-rung change
