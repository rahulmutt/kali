## 2026-04-18 — Stage 4.2 live-reference filtering theorem naming sync

I tightened the Stage 4.2 summary and tracker prose so the published boundary now names `KaliCore.Safety.releaseRefLiveRefsFiltered`, `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered`, and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered` explicitly alongside `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated` and the rest of the RC snapshot inventory.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md`, `TODO.md`, and `plan/phase-4/02-formal-verification-depth.md` so the Stage 4.2 progress notes and tracker now name the exact live-reference filtering theorem slice explicitly
- updated `crates/kali_cli/tests/schema_docs.rs` so the proof-summary anti-drift guard now pins the exact live-reference filtering theorem names alongside the rest of the RC snapshot inventory
- kept the change narrow: this is a wording / tracker sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 plan-note anti-drift guard widening

I widened the Stage 4.2 proof-summary guard so it now also checks `plan/phase-4/02-formal-verification-depth.md` for the canonical proof-backed summary and theorem inventory, keeping the stage plan note aligned with the published boundary wording.

Completed follow-up:
- updated `crates/kali_cli/tests/schema_docs.rs` so the proof-summary guard now also includes the Stage 4.2 plan note in its summary-doc coverage
- updated `plan/phase-4/02-formal-verification-depth.md` so the current progress note now carries the canonical proof-backed summary string and theorem inventory wording that the guard expects
- kept the change narrow: this is a plan-note / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 pure release-helper origin/ownership/positivity widening

I found one more symmetry gap in the Stage 4.2 RC snapshot tracker: the release-only helper already has heap-characterisation, origin, and origin/ownership theorems, but it does not yet package the surviving positive-count fact together with those provenance facts.

Completed follow-up:
- `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md` already name `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` explicitly, so the release-only helper provenance story is explicit at the same granularity as the decrement and collection helper families.
- kept the change narrow: this is a helper-theorem / progress-tracker sync for the published boundary, not a boundary widening beyond the current RC snapshot model

## 2026-04-18 — Stage 3.3 string-exports corpus coverage

I found one more common real-world package-shape case still missing from the Stage 3.3 corpus breadth: packages whose top-level `exports` field is a plain string rather than an exports map.

Completed follow-up:
- added browser and utility corpus coverage for plain string `exports` roots in `crates/kali_cli/tests/package_corpus.rs`
- synced `PLAN-3.3-STATUS.md`, `TODO.md`, and `plan/phase-3/03-ecosystem-breadth.md` so the Stage 3.3 corpus inventory now names the new shape explicitly alongside the existing exports-map / browser-field cases
- kept the claim narrow: this is another package-corpus breadth slice, not a new support rung

## 2026-04-18 — Stage 4.2 pure release-origin helper widening

I found a small proof-summary gap in the pure release helper slice: the RC snapshot boundary already names `KaliCore.Safety.releaseRefHeapCharacterisation` and `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, but it does not yet call out the plain origin theorem `KaliCore.Safety.releaseRefHeapCellOrigin` explicitly.

Planned update:
- add `KaliCore.Safety.releaseRefHeapCellOrigin` to `proofs/KaliCore/Safety.lean`, then sync the Stage 4.2 plan/progress notes (`plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md`) so the pure release-helper provenance story is explicit at the same granularity as the decrement and collection helper families
- keep the claim narrow: this is a helper-theorem / progress-tracker sync for the published boundary, not a boundary widening beyond the current RC snapshot model

## 2026-04-18 — Stage 4.2 unrelated-heap / other-live guard follow-up

I tightened the Stage 4.2 proof-summary anti-drift guard so it now also pins `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` explicitly alongside the existing RC snapshot inventory.

Completed follow-up:
- updated `crates/kali_cli/tests/schema_docs.rs` so the proof-summary guard now also checks the unrelated-heap / other-live theorem names explicitly
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the progress notes now reflect the widened anti-drift guard alongside the published helper slice
- kept the change narrow: this is a guard / progress-tracker sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 pure release helper wording follow-up

I tightened the Stage 4.2 progress summary so it now also names the pure release helper's heap-characterisation and origin/ownership theorems, `KaliCore.Safety.releaseRefHeapCharacterisation` and `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, explicitly alongside the release-only live-reference corollaries and the rest of the RC snapshot inventory.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the top-level memory-safety summary now names the pure release helper theorem pair explicitly alongside the existing RC inventory
- kept the change narrow: this is a plan-summary wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 release-only helper wording sync

I found that the Stage 4.2 progress trackers still do not call out the release-only helper corollaries `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseRefLiveRefsFiltered`, and `KaliCore.Safety.releasePreservesWellFormed` as explicitly as the published boundary does.

Planned update:
- sync `PLAN-4.2-STATUS.md` and `TODO.md` so the progress notes name the release-only helper corollaries explicitly alongside the existing RC snapshot theorem inventory
- keep the claim narrow: this is a progress-tracker wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 lowering-correctness theorem-name sync

I tightened the Stage 4.2 progress trackers to name the explicit HIR lowering-preservation theorems `KaliIR.LoweringCorrectness.lower_preserves_step` and `KaliIR.LoweringCorrectness.lower_preserves_steps`, so the plan/progress notes keep the lower-correctness slice just as concrete as the RC snapshot inventory.

Completed follow-up:
- updated `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and `TODO.md` so the lower-correctness progress notes name both lowering-preservation theorems explicitly alongside the current HIR slice
- extended the proof-summary anti-drift guard in `crates/kali_cli/tests/schema_docs.rs` so the lowering theorem names are now checked alongside the existing proof-boundary inventory
- kept the change narrow: this is a progress-tracker / guard sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 original positive-count decrement theorem

I added the helper-level RC summary theorem `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`, which packages the positive-count survivorship story for the release-and-decrement helper so the plan/progress trackers can cite it directly.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md`, `TODO.md`, and `plan/phase-4/02-formal-verification-depth.md` so the progress trackers name the new theorem explicitly alongside the existing RC snapshot inventory
- kept the change narrow: this is a helper-theorem / progress-tracker widening on top of the published boundary, not a new boundary shape

## 2026-04-18 — Stage 4.2 original positive-count survivor theorem

I added the helper-level RC summary theorem `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`, which packages the positive-count survivorship story for the release-and-collect helper so the no-leak slice is easier to cite directly.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md`, `TODO.md`, `plan/phase-4/02-formal-verification-depth.md`, and the published verification summaries so the new theorem is named explicitly alongside the existing RC snapshot inventory
- kept the change narrow: this is a helper-theorem / progress-tracker widening on top of the published boundary, not a new boundary shape

## 2026-04-18 — Stage 4.2 proof-summary tracker coverage

I widened the proof-summary anti-drift guard so `crates/kali_cli/tests/schema_docs.rs` now also checks `TODO.md` for the current RC theorem inventory, keeping the progress tracker aligned with the already-published proof-boundary wording.

Completed follow-up:
- added `TODO.md` to the theorem-name drift guard so the progress tracker stays aligned with the published RC snapshot inventory
- kept the change narrow: this is a tracker-coverage sync, not a boundary widening

## 2026-04-18 — Stage 4.2 decrement origin/positive-count progress-note sync

Completed follow-up:
- `PLAN-4.2-STATUS.md` and `TODO.md` now keep `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount` explicit alongside the rest of the RC snapshot inventory, closing out the follow-up for the decrement-path provenance/positivity wording.

## 2026-04-18 — Stage 4.2 target-allocation ownership/allocation corollaries

I widened the Stage 4.2 RC progress trackers with target-specific ownership/allocation corollaries for the decrement and collection helpers: `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount` and `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`.

Completed follow-up:
- synced `PLAN-4.2-STATUS.md` and `TODO.md` so the progress trackers now name the new target-specific corollaries alongside the existing target-allocation bridge notes
- kept the update narrow: this is still a progress-tracker / helper-theorem widening, not the full ownership/freeing target

## 2026-04-18 — Stage 4.1 package-audit preview-shim removal

I removed the last acceptance path for the package-audit `--preview` compatibility shim: the CLI now rejects that flag with the canonical `E5008` invalid-usage diagnostic instead of treating it as a hidden no-op.

Completed follow-up:
- updated `PLAN-4.1-STATUS.md` and `TODO.md` so the Stage 4.1 progress trackers now say the removed `--preview` path is rejected with `E5008`
- kept the change narrow: this is a CLI-usage cleanup for the already-shipped package-audit command, not a new package-analysis surface

Suggested follow-up:
- keep the Stage 4.1 status text and help-output wording aligned if package-audit gets another presentation-only tweak later

## 2026-04-18 — Stage 4.2 target allocation follow-up

I plan to widen the current RC snapshot proof slice with explicit target-allocation corollaries for the refcount update helpers (`KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount` and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`), so the progress trackers can name the allocation bridge alongside the existing positive-count and provenance lemmas.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the progress trackers now name the target-allocation corollaries explicitly alongside the current RC helper slice
- the proof-boundary manifest and verification summaries already name the allocation bridge; this note keeps the progress tracker aligned with that published boundary wording

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level allocation bridge, not the full RC target

## 2026-04-18 — Stage 4.2 origin/positivity helper wording sync

I found the plan-facing RC snapshot summary still referred to the origin/positivity conjunction theorem a bit too generically, even though the published boundary already names `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount` explicitly.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the progress note now names `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount` explicitly alongside the other RC snapshot helper theorems
- kept the change narrow: this is a wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 exact releasedRefs wording sync

I found one remaining verification-summary drift point: the published boundary already names the exact released-reference cons-shape theorems, but the plan-facing progress note still summarized that slice a bit too generically.

Completed follow-up:
- updated `plan/phase-4/02-formal-verification-depth.md` and `PLAN-4.2-STATUS.md` so the progress note now names `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons` explicitly
- extended `crates/kali_cli/tests/schema_docs.rs` so the proof-summary anti-drift guard now checks the released-reference cons-shape theorem names alongside the existing RC snapshot inventory
- kept the change narrow: this is a wording / guard sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 heap-characterisation wording sync

I found that the verification summary docs still describe the RC snapshot heap-characterisation story a bit too generically, even though the published boundary already names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation` explicitly.

Completed follow-up:
- updated `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the heap-characterisation theorems are named explicitly alongside the current RC snapshot inventory
- widened the proof-summary anti-drift guard in `crates/kali_cli/tests/schema_docs.rs` so it also checks `specs/16-testing.md` for the canonical proof-backed summary and theorem inventory

Suggested follow-up:
- keep the claim narrow: this is a wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 3.3 browser string-entry corpus follow-up

I widened the Phase-3 package corpus with browser-field string override coverage (`browser: "./index.browser.js"`) so the browser-targeted package-support track now exercises the string-entry browser rewrite shape in addition to the existing replacement-map, module-only, dual-package, and mixed-format cases.

Completed follow-up:
- updated `PLAN-3.3-STATUS.md` and `TODO.md` so the Stage 3.3 progress trackers now name browser string-entry coverage explicitly alongside the existing browser corpus shapes

Suggested follow-up:
- keep widening the browser corpus incrementally if additional real-world browser-field shapes are still missing
- continue treating this as a corpus-coverage slice, not a new support rung

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

## 2026-04-17 — Stage 3.3 module-only corpus coverage

I widened the Stage 3.3 package corpus with module-only entrypoint packages, so the representative browser and utility sets now exercise `package.json#module` fallback resolution as a published shape instead of only through mixed-format packages.

Suggested follow-up:
- keep the Stage 3.3 corpus/status notes aligned if the module-only coverage widens again
- continue treating this as a package-corpus widening slice, not a new support rung

## 2026-04-18 — Stage 4.2 exact releasedRefs wording sync

I synced the plan-facing progress tracker after the verification summaries were tightened to name the exact released-reference cons-shape theorems explicitly.

Completed follow-up:
- updated `TODO.md` so the Stage 4.2 progress log now names `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons` explicitly alongside the current RC snapshot inventory
- kept the change narrow: this is a wording / progress-note sync, not a boundary widening

## 2026-04-18 — Stage 4.2 release-and-decrement origin/ownership/positivity follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, so the stage tracker should name the new bundled helper theorem explicitly alongside the existing RC inventory.

Completed follow-up:
- update `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress trackers name the new release-and-decrement origin/ownership/positivity theorem alongside the current helper slice
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level conjunction theorem, not the full RC target

## 2026-04-18 — Stage 4.2 decrement origin/positive-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, so the Stage 4.2 progress trackers should name the new bundled helper theorem explicitly alongside the existing RC helper inventory.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress trackers now name `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount` explicitly alongside the current helper slice
- reran the schema-doc anti-drift test so the published boundary wording stays locked to the mechanised theorem inventory

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level conjunction theorem, not the full RC target

## 2026-04-18 — Stage 4.2 heap-filter anti-drift guard

I tightened the proof-summary anti-drift guard so it now also pins `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` explicitly, and the verification summaries now name the filter theorem alongside the rest of the RC snapshot inventory.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the progress notes now mention `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` explicitly alongside the other RC snapshot helper theorems
- reran the schema-doc anti-drift test so the published boundary wording stays locked to the current theorem inventory

Suggested follow-up:
- keep the claim narrow: this is a wording / drift-guard sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 positive-count anti-drift guard widening

I widened the proof-summary anti-drift guard so it now also pins `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells` explicitly, keeping the surviving non-target positivity wording locked to the published boundary inventory.

Completed follow-up:
- updated `crates/kali_cli/tests/schema_docs.rs` to pin `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells` alongside the existing RC snapshot theorem inventory
- updated `PLAN-4.2-STATUS.md` and `TODO.md` so the Stage 4.2 progress notes record the guard widening

Suggested follow-up:
- keep the claim narrow: this is a test-guard / progress-tracker sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 pure release heap-characterisation wording sync

I found one remaining Stage 4.2 progress-note drift point: `PLAN-4.2-STATUS.md` was still missing the new pure release helper theorem `KaliCore.Safety.releaseRefHeapCharacterisation` in the large summary bullet / evidence bullets, even though the stage note and the proof-boundary docs already named it.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the status summary, proof-state inventory, notable deliverables, and current-limits note now name `KaliCore.Safety.releaseRefHeapCharacterisation` explicitly alongside the rest of the RC snapshot helper slice
- reran the schema-doc anti-drift test so the plan/status wording stays aligned with the published theorem inventory

Suggested follow-up:
- keep the Stage 4.2 status note aligned if the pure release helper slice widens again
- continue treating this as a wording / progress-tracker sync, not a boundary widening

## 2026-04-18 — Stage 2.2 status-file backfill

I noticed the plan tracker had complete Stage 2.2 implementation notes but no dedicated stage status file, so I added `PLAN-2.2-STATUS.md` to keep the Phase 2 stage set uniformly documented.

Completed follow-up:
- created `PLAN-2.2-STATUS.md` with the completed public effect-reporting summary, evidence, deliverables, and current limits
- kept the update narrow: this is a plan-tracker/documentation backfill, not a new product surface

Suggested follow-up:
- if any future plan-stage tracker is added late, keep the per-stage status files in sync so the phase index stays complete
## 2026-04-18 — Stage 3.1 struct-layout sharing follow-up

I widened the Stage 3.1 MIR-aware specialization coverage with a regression test that proves identical struct-layout bindings reuse the same MIR-specialized clone across three matching call sites, so the current closure-layout sharing path is no longer the only documented layout-sharing shape.

Completed follow-up:
- updated `crates/kali_optimize/src/lib.rs` with the regression coverage for three matching struct-layout call sites
- updated `PLAN-3.1-STATUS.md` and `TODO.md` so the stage tracker now names the expanded struct-layout regression explicitly alongside the existing closure-layout sharing note
- kept the change narrow: this is another layout-sharing regression check, not the fuller generic-instantiation planner

## 2026-04-18 — Stage 4.2 pure release heap characterisation wording sync

I tightened the Stage 4.2 tracker so `TODO.md` now calls out `KaliCore.Safety.releaseRefHeapCharacterisation` explicitly alongside the release-only live-reference and disjointness corollaries, keeping the pure release-helper slice aligned with the published boundary inventory.

Completed follow-up:
- updated `TODO.md` so the Stage 4.2 progress tracker names the pure release heap-characterisation theorem explicitly
- kept the change narrow: this is a wording / tracker sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 progress-note inventory alignment

I tightened the Stage 4.2 status summary so the proof-backed memory-safety slice now names the helper-level original-positive-count survivor theorems (`KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`) explicitly alongside the existing release/decrement/collection inventory, keeping the plan-facing summary closer to the published boundary wording.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the stage summary now mentions the helper-level positive-count survivors explicitly alongside the rest of the RC snapshot inventory
- kept the change narrow: this is a progress-tracker wording sync, not a boundary widening

## 2026-04-18 — Stage 4.2 pure release helper origin/ownership follow-up

I added the direct pure release-helper theorem `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, which packages origin and ownership preservation for `releaseRef` so the plan/status notes can name the release-only heap provenance story more explicitly alongside `releaseRefHeapCharacterisation`.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, `plan/phase-4/02-formal-verification-depth.md`, and the proof-boundary summaries so the new theorem is named explicitly
- keep the claim narrow: this is still a helper-level provenance theorem on the published boundary, not a broader ownership/freeing target

## 2026-04-18 — Stage 3.3 pattern-exports corpus coverage

I widened the Stage 3.3 package corpus with `./*` exports-pattern coverage so the browser and utility corpus now exercise nested subpath exports routed through a `src/` subtree.

Completed follow-up:
- added pattern-exports corpus coverage for browser and utility packages in `crates/kali_cli/tests/package_corpus.rs`
- extended `crates/kali_npm/src/lib.rs` so `exports` pattern keys with a single `*` resolve deterministically before browser rewrites are applied
- kept the change narrow: this is a package-corpus / resolver breadth step, not a new support rung

## 2026-04-18 — Stage 4.2 memory-safety summary sync

I tightened the top-level Stage 4.2 status summary so it now names `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership` and `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` explicitly alongside the existing RC snapshot inventory, keeping the plan-facing summary aligned with the published boundary wording.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the top-level memory-safety summary now calls out the pure release-helper provenance theorem and the collection-helper heap/filter theorem explicitly
- kept the change narrow: this is a progress-summary wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 unrelated-heap / other-live wording sync

I found the Stage 4.2 summary tracker still described the unrelated-heap and other-live helper slices generically, even though the published boundary already mechanizes them.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` so the progress note names `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` explicitly alongside the rest of the RC snapshot inventory
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 3.3 plan inventory alignment

I aligned the Stage 3.3 plan wording with the now-implemented package corpus breadth so `plan/phase-3/03-ecosystem-breadth.md` explicitly lists the representative browser and utility package-shape cases already covered in the corpus.

Completed follow-up:
- updated the Stage 3.3 package-corpus task in `plan/phase-3/03-ecosystem-breadth.md` to enumerate the browser/utility shapes now exercised by the corpus
- kept the change narrow: this is a plan-text alignment for the existing corpus breadth, not a new support rung

## 2026-04-18 — Stage 3.3 internal dependency-chain corpus coverage

I widened the Stage 3.3 package corpus with browser-field internal rewrite chains and module-entry internal dependency chains so the browser and utility corpus now exercise package-local dependency graphs in addition to the existing exports/browser/module shapes.

Completed follow-up:
- added browser internal-browser-rewrite corpus coverage for browser packages in `crates/kali_cli/tests/package_corpus.rs`
- added module-entry internal-dependency corpus coverage for utility packages in `crates/kali_cli/tests/package_corpus.rs`
- kept the change narrow: this is a package-corpus widening slice, not a new support rung

## 2026-04-18 — Stage 4.2 live-reference filtering theorem naming sync

I tightened the Stage 4.2 plan/progress summary so it now names the decrement/collection live-reference filtering theorems `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered` explicitly alongside the release-only helper theorem, keeping the plan wording aligned with the published boundary inventory.

Completed follow-up:
- updated `PLAN-4.2-STATUS.md` so the top-level Stage 4.2 summary now names the decrement/collection live-reference filtering theorems explicitly
- updated the proof-summary drift guard in `crates/kali_cli/tests/schema_docs.rs` so the theorem inventory check now includes the decrement/collection live-reference filtering theorem names
- kept the change narrow: this is a plan/progress wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 collection target-cell iff helper surfacing

I promoted the local collection helper's target-cell survival/removal split to the public theorem `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` and synced the Stage 4.2 plan/status trackers plus the proof-boundary summaries so the next memory-safety widening step stays explicit.

Suggested follow-up:
- keep the claim narrow: this is a proof-boundary / tracker sync for the published RC snapshot slice, not a widening of the broader ownership/freeing target

## 2026-04-18 — Stage 4.2 lowering value-preservation progress note

I added `KaliIR.LoweringCorrectness.lower_preserves_value` and the small HIR value fragment it depends on. The Stage 4.2 progress docs should note the additional lowering helper alongside `lower_preserves_step` / `lower_preserves_steps`, and the proof-summary drift guard should pin the new theorem name once the boundary summary is widened.
