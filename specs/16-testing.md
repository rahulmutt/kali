# 16 — Testing

This chapter defines Kali's evidence lanes and the minimum testing discipline required before a feature may be described as supported.

Planning ownership:
- this chapter defines **what evidence is required** for a claim
- [`PLAN.md`](../PLAN.md) and [`plan/`](../plan) own **when** test infrastructure is built, expanded, or promoted in CI
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the current proof-backed boundary

## Core rule

A feature may be documented before it ships, but it may be described as **supported** only when:
1. its maturity row is open in [19 — Feature Maturity](./19-feature-maturity.md), and
2. the matching evidence lane in this chapter exists and passes.

One demo, one fixture, or one anecdotal package success is not enough to widen a public support claim.

## Phase-correct testing rule

Treat workflow families according to their maturity owner:
- **Phase-1 shipped workflows** require positive integration coverage for every supported command/context combination.
- **Later documented workflows** may already have command shapes, schemas, or internal plumbing, but tests must assert unavailability until the matching maturity row opens.
- **Internal-only machinery** may be tested without being presented as a stable public CLI/API surface.

Examples:
- `run/test --sandbox`, the **Phase-1 static policy-validation surface**, and the **Phase-1 browser-targeted command set** need positive Phase-1 coverage.
- `kali effects`, `kali package-effects`, `kali package-audit`, inferred-effect-vs-policy rejection on `check/build --sandbox`, stable public embedding flows (`--capi`, `--component`, stable public `--lib` + WIT), and wider proof claims stay negative/gated until their maturity rows open.

## Evidence matrix

| Concern area | Minimum evidence before claiming support |
|---|---|
| Language syntax/semantics | parser tests, integration coverage, and the applicable test262/conformance subset |
| Type checking / inference | checker baselines, inference goldens, and targeted regressions |
| First-class JavaScript compilation | dedicated `.js` fixtures across `check` / `build` / `run`, JSDoc-hint coverage, and fallback-ladder cases |
| Host APIs / runtime behavior | integration tests that execute the API path plus sandbox/resource-limit coverage where relevant |
| Phase-1 browser-targeted command set | browser-targeted `check` coverage, browser-targeted `build --bundle` coverage, and emitted-bundle smoke runs in a real browser harness |
| Base library artifact (`kali build --lib`) | library-build integration tests, artifact/schema assertions, `E5011` negatives for unknown export surfaces, and deterministic rebuild checks |
| Package compatibility | curated package-corpus results recorded per shipped source-graph command/context and per claimed rung of the shared package-support ladder |
| Install workflow / opt-in npm lifecycle hooks | install-command integration tests for manifest/lock/materialization updates, hook gating, and invalid raw-URL / JSR combinations |
| Registry-analysis commands (`package-effects`, `package-audit`) | command-shape negatives, deterministic single-package version-selection tests, context-participation tests, and JSON-contract assertions |
| CLI behavior / JSON schemas | golden CLI snapshots, schema validation, exit-code assertions, and the `kali test --coverage` payload contract |
| Artifact reproducibility | repeated-build tests over pinned inputs plus stable artifact-byte and metadata assertions |
| Proof-backed claims | passing Lean proof jobs for the currently published proof boundary |

Interpretation rules:
- grammar coverage and execution-semantic support are separate claims
- package-corpus evidence for ordinary source-graph commands is separate from evidence for later registry-analysis commands
- proof evidence strengthens confidence only for the published proof boundary; it does not replace command/profile-specific implementation tests

## Test families

### Unit tests
Each implementation subsystem should have focused unit coverage for its own invariants, including at least:
- lexer tokenization and recovery
- parser / AST construction
- type inference and checking
- IR transformation correctness
- codegen validation
- sandbox policy parsing and enforcement helpers

### Integration tests
End-to-end coverage should include:
- source → compile → execute → expected output
- source → compile/check → expected diagnostics
- `.js` source across representative inference tiers
- source + policy → `run/test --sandbox` runtime enforcement
- source graph + policy → the **Phase-1 static policy-validation surface**
- install workflow: `kali install`, `kali install <pkg>`, `kali install --dev <pkg>`, and opt-in `kali install --allow-scripts ...`
- test discovery / explicit-file selection / `--filter` / `--coverage`
- `kali build --lib` for fixtures with a **statically known export surface**
- browser-targeted `check` / `build --bundle`
- repeated builds of identical pinned inputs for determinism

### Snapshot tests
Snapshot tests are appropriate for:
- HIR
- MIR once MIR is the canonical ownership/layout IR
- generated WASM text or other stable internal representations

Snapshots must stay deterministic and reviewable.

### Fuzzing
Fuzz the lexer, parser, checker, and codegen. The minimum invariant is: **the compiler must not panic on arbitrary input**.

### Conformance suites
- Use test262 for ECMAScript conformance.
- Use `tsc`-style baseline fixtures for typing and inference behavior.
- Keep parser-breadth tracking separate from execution-semantic support claims.

### Package corpus
Maintain a curated package corpus that records expected outcomes per shipped command/context and per claimed support rung. Keep excluded native/binary/bootstrap-heavy packages in a separate negative track so installer-hook evidence is not misreported as general compatibility.

## Determinism requirements

All machine-facing outputs used in support claims must be deterministic for pinned inputs, including:
- CLI JSON outputs
- build artifacts
- artifact metadata
- lockfiles
- report ordering

Equivalent dependency graphs should converge on byte-stable lockfile and artifact output rather than fetch order or hash-map iteration order.

## Browser-targeted evidence lane

Because Phase 1 explicitly ships the **Phase-1 browser-targeted command set**, it needs a dedicated evidence lane:
- browser-targeted type-check fixtures exercising browser ambient typings
- real-browser smoke tests for emitted bundles
- negative tests for unsupported standalone browser commands (`run --api browser`, `test --api browser`)

Mock-only DOM tests are not enough to justify browser-runtime support wording.

## Base-library artifact evidence lane

Because Phase 1 explicitly ships `kali build --lib` for **exact-version consumers** when the export surface is statically known, it needs a dedicated evidence lane:
- positive library-build fixtures
- deterministic artifact assertions
- negative `E5011` cases for inputs without a statically known export surface
- any host-consumption smoke test in this lane must be described as an **exact-version consumer** test, not as cross-version public ABI evidence

## Proof claim discipline

Proof-related testing follows the shared **proof-ready vs proof-backed** split:
- proof-ready is a repository/process baseline
- proof-backed claims require a non-empty published boundary in [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md)
- the current proof claim is always read from that manifest, not from duplicated prose here
- the canonical short summary is: **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**
- current proof-backed boundary snapshot: **Verification**: reuse the canonical repository summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** The published boundary currently includes the widened closed fragment plus a small RC snapshot safety slice with live-reference ownership/allocation projection via the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, exact live-reference filtering via the release-only helper theorem `KaliCore.Safety.releaseRefLiveRefsFiltered` and the decrement/collection theorems `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`, the release-only helper's live-reference ownership/allocation corollary `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated`, the release-only helper's live-reference filtering corollary `KaliCore.Safety.releaseRefLiveRefsFiltered`, and the live-to-released transition preservation `KaliCore.Safety.releasePreservesWellFormed`, explicit release-recording and exact released-reference cons-shape via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCellOrigin`, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons` on the release-only, decrement, and collection helpers, pure release-helper ownership/allocation and disjointness corollaries, ownership-envelope preservation on the release-only, decrement, and collection helpers, release-set preservation on the release-only, decrement, and collection helpers via `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, and `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`, target-cell decrement bookkeeping, heap-origin provenance for the release-and-decrement helper, the release-and-decrement target-cell origin theorem `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`, the release-and-decrement provenance-and-ownership theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the heap-characterisation theorems `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, the release-and-decrement positive-count preservation theorem `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`, the release-and-decrement target-cell positive-count preservation theorem `KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, the release-and-decrement target-cell positive-count iff theorem `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`, the release-and-decrement target-allocation corollary `KaliCore.Safety.releaseAndDecrementTargetCellOrigin`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, the release-and-collect target-cell iff theorem `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, the release-and-collect target-allocation corollary `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOrigin`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`, and the bundled `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` helper theorem, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass via `releaseAndCollectDropsZeroCountCells`, zero-count removal from the collected heap via `releaseAndCollectRemovesZeroCountCells`, positive-count preservation on the local collection helper, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` that positive-count cells from the original heap survive when they are not the released target and remain positive-count after collection, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount` that the released target remains in the collected heap when its decremented count stays positive, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership` that the surviving collection-helper cells preserve their original name and ownership tag, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount` that the surviving collection-helper cells are both traceable to the original heap and positive-count, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOrigin` that every surviving release-and-collect heap cell still comes from the original heap with only the released target decremented, unrelated-heap preservation via `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries` and `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, the helper-level theorem that original zero-count cells are dropped from the final heap, other-live-reference preservation via `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs` and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs`, the helper-level ownership/allocation preservation corollaries on the decrement and collection paths, the mechanized `KaliCore.Safety.noDanglingReference` theorem plus the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, and a refcount-decrement update helper, plus the remaining bookkeeping corollaries `KaliCore.Safety.releaseRecorded`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `releasedNotLive`, and `releasedNotLiveRef`, plus a widened HIR lowering-correctness slice that now also includes `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, and bare throw. The RC snapshot provenance wording also now spells out `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly alongside `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, so the companion theorem is named directly where the summary needs it.
- additional proof-summary theorem names pinned here for the proof-backstop summary: `KaliCore.Soundness.subst_closed`, `KaliCore.litTy`
`releaseAndCollectLiveRefsAreOwnedAndAllocated`, `liveRefsAreOwnedAndAllocated`, `releaseAndDecrementPreservesOwnership`, `releaseRefHeapCharacterisationAndLinearMemory`, `releaseRefPreservesLinearMemory`, `releaseRefPreservesOwnershipAndLinearMemory`, `releaseAndCollectPreservesLinearMemory`, `releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`, `releaseAndCollectPreservesOwnershipAndLinearMemory`, `releaseRefPreservesWellFormedAndLinearMemory`, `releaseAndDecrementPreservesWellFormedAndLinearMemory`, `releaseAndCollectPreservesWellFormedAndLinearMemory`, `releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseRefPreservesWellFormedAndOwnership`, `releaseAndDecrementPreservesWellFormedAndOwnership`, `releaseAndCollectPreservesWellFormedAndOwnership`, `releaseAndDecrementPreservesLinearMemory`, `releaseAndDecrementPreservesOwnershipAndLinearMemory`, `releaseAndCollectPreservesOwnership`, `releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory`, `releaseAndCollectHeapIsPositiveCountFilter`, `releaseAndCollectPreservesLinearMemory`, `releaseAndCollectPreservesOwnershipAndLinearMemory`, `releaseAndCollectLiveRefsAreOwnedAndAllocated`, `releaseAndDecrementHeapCharacterisationAndLinearMemory`, `releaseAndCollectHeapCharacterisationAndLinearMemory`

If a release/support claim changes, update:
- this chapter,
- [17 — Formal Verification](./17-verification.md),
- [19 — Feature Maturity](./19-feature-maturity.md),
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), and
- any affected summaries such as [`README.md`](../README.md)

## Practical implementation note

Concrete CI layout, directory structure, benchmark automation, and staged evidence expansion belong to the implementation plan, primarily:
- [`PLAN.md`](../PLAN.md)
- [`plan/phase-1/14-evidence-hardening.md`](../plan/phase-1/14-evidence-hardening.md)
- later phase plan files when new evidence lanes open
