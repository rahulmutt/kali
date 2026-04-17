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

## 2026-04-17 — Stage 4.2 helper-level no-dangling plan sync

I widened the current RC snapshot proof slice with helper-level no-dangling-reference corollaries for `releaseRefNoDanglingReference`, `releaseAndDecrementNoDanglingReference`, and `releaseAndCollectNoDanglingReference`, and I planned to keep the Stage 4.2 status tracker aligned with those helper-level safety corollaries.

Completed follow-up:
- synced `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress trackers now name the helper-level no-dangling corollaries explicitly alongside the existing RC snapshot inventory

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level safety corollary slice, not the full RC target

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

## 2026-04-17 — Plan completion-gate sync

I reviewed the phase completion gates in `PLAN.md` after verifying the workspace tests and Lean proof build, and the plan text still had one stale effect-report schema reference.

Completed follow-up:
- updated `PLAN.md` so the Phase 2 completion gate now names the stable effect-report surface as schema-v1 JSON, matching `specs/09-sandboxing.md`, `specs/18-schemas.md`, and the stage-level docs
- marked the Phase 2, Phase 3, and Phase 4 completion gates complete in `PLAN.md` so the top-level plan now reflects the current stage status

Suggested follow-up:
- keep `PLAN.md`, the stage status docs, and the maturity/spec summaries aligned whenever another phase gate changes

## 2026-04-17 — Stage 3.3 browser-conditional exports follow-up

I expanded the Stage 3.3 package corpus tests with browser-conditional exports coverage for the browser corpus, so the browser package shapes now exercise the `browser` branch alongside the existing import/require and mixed-format cases.

Completed follow-up:
- updated `PLAN-3.3-STATUS.md` and `TODO.md` so the progress trackers now call out browser-conditional-export coverage explicitly

Suggested follow-up:
- keep the stage progress notes aligned with the corpus tests whenever another representative shape is added
- continue broadening the corpus as new package shapes are triaged

## 2026-04-17 — Stage 3.3 package-corpus expansion

I expanded the Stage 3.3 package corpus tests with exports-map / subpath coverage for the browser, utility, and Node-runner cases, and then added dual-package / mixed-format coverage so the corpus now exercises conditional exports plus mixed CJS/ESM entrypoints instead of only single-entrypoint stubs.

Suggested follow-up:
- keep the stage progress notes aligned with the corpus tests whenever another representative shape is added
- continue broadening the corpus as new package shapes are triaged

## 2026-04-17 — Stage 4.2 proof-summary anti-drift follow-up

I tightened the proof-boundary anti-drift coverage so `crates/kali_cli/tests/schema_docs.rs` now checks both the `proofs/BOUNDARY.md` covered-path inventory and the published theorem/lemma inventory against the concrete Lean source set, and now also verifies the canonical proof-summary docs keep the current RC theorem names plus the proof-backed summary string in sync.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the progress trackers now mention the theorem-name inventory check and the proof-summary doc sync guard alongside the path-level anti-drift guard

Suggested follow-up:
- keep the proof boundary manifest, theorem inventory, and summary docs aligned whenever the proof slice widens again

## 2026-04-17 — Stage 3.1 array-layout specialization follow-up

I widened the Stage 3.1 optimizer's layout prepass so const-bound array element reads now fold when the index is statically known or bound to a constant numeric value, extending the existing object-layout fast path.

Completed follow-up:
- updated `PLAN-3.1-STATUS.md` and `TODO.md` so the stage tracker names the new array-layout specialization behavior explicitly
- kept the long-term generic-instantiation and MIR-driven specialization work as the remaining Stage 3.1 follow-up; this is still a layout-folding pass, not the full planner

## 2026-04-17 — Stage 4.2 heap-positive summary sync

I synced the Stage 4.2 status tracker so the published progress summary now names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly alongside the other RC snapshot slice theorems.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the progress summary now names the final-heap positive-count theorem explicitly
- kept the broader Stage 4.2 ownership/freeing target narrow; this is still a helper-level collection theorem, not the full RC target

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

## 2026-04-17 — Stage 4.2 proof-boundary status sync

I refreshed the Stage 4.2 plan tracker after the RC snapshot proof slice widened to include the latest local-collection helper theorems (`releaseAndCollectDropsOriginalZeroCountCells` and `releaseAndCollectHeapCellsHavePositiveCount`) alongside the existing ownership-preservation corollaries and zero-count bookkeeping.

Completed follow-up:
- synced `TODO.md` so the proof-boundary widening summary names the latest collection-helper theorems and ownership-preservation corollaries explicitly
- synced `PLAN-4.2-STATUS.md` so the Stage 4.2 status tracker matches the current theorem inventory
- no spec update was needed because the published boundary and maturity wording were already current


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

## 2026-04-17 — Stage 4.2 heap-origin provenance sync

I refreshed the Stage 4.2 progress trackers after widening the RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapCellOrigin`, which makes the local `releaseAndCollect` helper's surviving-cell provenance explicit.

Completed follow-up:
- synced `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the Stage 4.2 progress notes name the heap-origin provenance theorem alongside the other RC helper invariants

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

## 2026-04-17 — Stage 4.2 target-cell retention sync

I synced the Stage 4.2 progress trackers and summary docs after widening the RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, so the published boundary and status notes now name the local collection helper's target-cell retention theorem explicitly.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the progress tracker, boundary manifest, and release claims stay in sync
- kept the Stage 4.2 claim narrow: this is still a helper-level retention theorem, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 target-cell retention wording sync

I synced the Stage 4.2 progress trackers so `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount` is named explicitly in the summary prose.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the current Stage 4.2 progress note explicitly names the target-cell retention theorem
- kept the broader Stage 4.2 ownership/freeing target incremental; this remains a local helper-level retention theorem

## 2026-04-17 — Stage 3.1 closure-layout specialization follow-up

I widened the Stage 3.1 optimizer's MIR-aware specialization path so shared closure-valued bindings now collapse to one specialized clone when multiple higher-order call sites share the same MIR layout signature.

Completed follow-up:
- updated `PLAN-3.1-STATUS.md` and `TODO.md` so the progress trackers now name the shared closure-layout specialization behavior explicitly
- kept the broader generic-instantiation and cross-module planner work as the remaining Stage 3.1 follow-up; this is still a layout-signature sharing step, not the full planner

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
