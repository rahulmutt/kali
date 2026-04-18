# TODO

## Completed

### Stage 4.2 - Lowering value-preservation summary sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, `KaliIR.LoweringCorrectness.lower_preserves_step`, and `KaliIR.LoweringCorrectness.lower_preserves_steps` alongside the widened HIR lowering-correctness slice.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a widening of the HIR semantic-preservation target.

### Stage 4.2 - Pure release-helper positive-count wording sync
- ✅ `proofs/BOUNDARY.md` now explicitly says `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` preserves the release-only cells' positive count in addition to their original ownership tag.
- ✅ Kept the update narrow: this is wording sync for the published boundary, not a new proof target.

### Stage 4.2 - Live-reference filtering theorem naming sync
- ✅ `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and this tracker now name `KaliCore.Safety.releaseRefLiveRefsFiltered`, `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered`, and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered` explicitly alongside `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated` and the rest of the RC snapshot inventory.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins the exact live-reference filtering theorem names, so the proof-summary drift guard keeps the helper slice aligned with the published boundary inventory and the stage plan note.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Remaining bookkeeping wording sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliCore.Safety.releaseRecorded`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `releasedNotLive`, and `releasedNotLiveRef` explicitly alongside the rest of the RC snapshot inventory.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins the same remaining bookkeeping corollaries and checks `plan/phase-4/02-formal-verification-depth.md` for the canonical proof-backed summary and theorem inventory, so the proof-summary drift guard keeps the helper slice aligned with the published boundary inventory and the stage plan note.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Release-only helper wording sync
- ✅ `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and the TODO stage summary now name `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseRefLiveRefsFiltered`, and `KaliCore.Safety.releasePreservesWellFormed` explicitly alongside the rest of the RC snapshot inventory.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Unrelated-heap / other-live wording sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` explicitly alongside the rest of the RC snapshot helper slice.
- ✅ `PLAN-4.2-STATUS.md` now records the same wording sync in the Stage 4.2 progress notes.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins the unrelated-heap / other-live theorem names, so the proof-summary drift guard keeps the helper slice aligned with the published boundary inventory.

### Stage 4.2 - Positive-count anti-drift guard widening
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` explicitly, so the proof-summary guard keeps the surviving non-target positivity wording aligned with the published boundary inventory.
- ✅ `PLAN-4.2-STATUS.md` now records the same guard widening in the Stage 4.2 progress notes.

### Stage 4.2 - Decrement origin/positive-count progress-note sync
- ✅ `PLAN-4.2-STATUS.md` and the TODO stage summary now keep `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount` explicit alongside the rest of the RC snapshot inventory, closing out the follow-up that widened the decrement-path provenance/positivity wording.

### Stage 4.2 - Proof-boundary heap-characterisation inventory sync
- ✅ `proofs/BOUNDARY.md` now names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation` explicitly in the claimed theorem inventory, keeping the manifest aligned with the proof-state summary and the summary docs.
- ✅ The pure release helper now also has an explicit heap-characterisation theorem, `KaliCore.Safety.releaseRefHeapCharacterisation`, a plain origin theorem, `KaliCore.Safety.releaseRefHeapCellOrigin`, and a direct origin/ownership theorem, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount`, so the RC helper slice states the unchanged-heap case and the release-only provenance story directly alongside the decrement/collection heap characterisation theorems.
- ✅ `PLAN-4.2-STATUS.md` and the Stage 4.2 progress note now call out the release-only heap-characterisation theorem explicitly, keeping the pure release helper slice aligned with the published boundary inventory.

### Stage 4.2 - Pure release heap characterisation wording sync
- ✅ `TODO.md` now calls out `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCellOrigin` explicitly in the Stage 4.2 tracker, and the proof-boundary theorem inventory now also names `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` explicitly, so the pure release-helper slice stays named alongside the release-only live-reference and disjointness corollaries.
- ✅ The existing `PLAN-4.2-STATUS.md` progress note already reflects the same helper-level wording, keeping the proof-backed boundary inventory aligned with the current RC theorem set.

### Stage 4.2 - Proof-summary anti-drift guard widening
- ✅ `crates/kali_cli/tests/schema_docs.rs` now pins a broader current RC snapshot helper inventory, including the no-dangling-reference corollaries, released-reference cons-shape theorems, target-cell bookkeeping, zero-count collection/removal, and heap-characterisation corollaries.
- ✅ The same drift guard now also checks `TODO.md` for the current RC theorem inventory, so the progress tracker stays aligned with the published boundary wording.

### Stage 4.2 - Heap-filter anti-drift guard
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` explicitly, and the verification summaries now name the filter theorem alongside the rest of the RC snapshot inventory.

### Stage 2.2 - Status-file backfill
- ✅ Added `PLAN-2.2-STATUS.md` so the Phase 2 stage tracker set now includes a dedicated public effect-reporting status summary.
- ✅ Kept the update narrow: this is a documentation backfill, not a new product surface.

### Stage 4.1 - Package-audit availability
- ✅ `kali package-audit` now runs without requiring `--preview`; the removed `--preview` path is rejected with the canonical `E5008` invalid-usage diagnostic instead of acting as a compatibility shim.

### Stage 4.1 - Eval compatibility gating
- ✅ `--compat eval` now accepts dynamically constructed eval / Function() strings derived from constant program-state fragments.
- ✅ `check` / `run` now reject `eval` and `Function()` usage unless the shared `--compat eval` gate is enabled.

### Plan completion-gate sync
- ✅ `PLAN.md` phase completion gates now reflect the current stage status, and the Phase 2 effect-report completion line now matches the schema-v1 contracts used by the stage docs and schema specs.

### Stage 4.2 exact releasedRefs wording sync
- ✅ Synced `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the verification summary now names `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons` explicitly alongside the existing RC snapshot inventory, and extended the proof-summary anti-drift guard so the released-reference cons-shape theorem names stay locked in the docs.

### Stage 4.2 no-dangling wording sync
- ✅ Synced `PLAN-4.2-STATUS.md` so the stage summary now names `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference` explicitly alongside the rest of the RC snapshot inventory.

### Stage 4.2 decrement origin/positive-count anti-drift guard
- ✅ The proof-summary anti-drift guard in `crates/kali_cli/tests/schema_docs.rs` now also pins `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, keeping the decrement-path provenance/positivity slice aligned with the published boundary inventory and the verification summaries locked to the mechanised theorem inventory.

### Stage 4.2 release-and-decrement origin/ownership/positivity follow-up
- ✅ `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount` now packages the decrement helper's surviving-cell provenance, ownership tag, and positive-count fact in one helper theorem, and the proof-boundary / verification summaries now name it explicitly alongside the current RC helper inventory.
- ✅ The proof-summary anti-drift guard in `crates/kali_cli/tests/schema_docs.rs` now also pins `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount` so the decrement-path provenance/ownership/positivity slice stays locked to the published boundary inventory.

### Stage 1.3 - Parser & AST
- ✅ Parser compiles successfully
- ✅ `cargo test -p kali_parser --lib` passes
- ✅ `cargo test -p kali_parser --test parser_integration` passes
- ✅ `cargo test --workspace` passes
- ✅ Parser handles variable declarations, blocks, functions, classes, control flow, try/catch, switch, debugger, throw, break/continue
- ✅ Parser handles primary expressions, function expressions, call chains, member access, binary expressions, and `new`
- ✅ Parser now accepts import declarations and literal dynamic `import()` expressions, which keeps package-corpus analysis and later code-splitting work on the real AST path
- ✅ Lexer fixes landed for punctuation advancement, `debugger`, and division tokens

### Stage 1.4 - Name Resolution
- ✅ Resolver reports unresolved identifiers, duplicate bindings, and missing import targets
- ✅ `kali check` is wired to the resolver and passes CLI smoke coverage
- ✅ `cargo test -p kali_types --lib` passes
- ✅ `cargo test -p kali_cli --test runtime_smoke` passes
- ✅ `cargo test --workspace` passes

### Stage 1.5 - Type Annotation Resolution
- ✅ Type annotation strings now resolve identifier references against the current scope and global bindings, so undefined type references surface through the existing name-resolution diagnostic path.

### Stage 1.6 - HIR/LIR Lowering
- ✅ Deterministic AST/statement → HIR lowering implemented
- ✅ HIR → MIR lowering implemented
- ✅ MIR → LIR lowering implemented
- ✅ Representative parser-backed lowering tests pass

### Stage 1.9 - Sandbox & Policy
- ✅ Declarative policy files parse and validate against schema v1
- ✅ `kali run --sandbox` enforces policy at runtime and reports `E4001` on violations
- ✅ `kali check --sandbox` validates policy schema/config without executing the program
- ✅ `kali build --sandbox` embeds the validated policy as `kali:policy` in the emitted WASM artifact
- ✅ Runtime policy enforcement and build embedding are covered by CLI/runtime tests

### Stage 1.10 - Package Management
- ✅ Manifest collision preflight now rejects registry identities that would materialize to the same `node_modules/` path
- ✅ Semver ranges now resolve deterministically to the highest matching published version
- ✅ Transitive install-path conflicts are rejected with `E6002`, and stale registry lock entries are pruned during `kali install`
- ✅ `kali install` now reconciles package cache and `node_modules/` state when the lock graph is already present
- ✅ Raw URL reconciliation now follows project-discovery/import-map declarations and prunes stale
  URL cache entries when the declaration graph changes
- ✅ Package-shape and host-fit coverage now rejects Node-only host APIs surfaced through direct imports/requires with `E6005`
- ✅ Registry metadata lookups now use a process-local cache, avoiding redundant refetches during repeated resolution within one install run

### Stage 1.11 - Build Artifacts
- ✅ `kali build` now emits deterministic `kali:metadata` custom sections in the executable `.wasm` artifact
- ✅ `kali build --lib` now emits `.lib.wasm` plus a deterministic `.lib.meta.json` export inventory
- ✅ `kali build --bundle` now emits a browser bundle directory with `.wasm`, `.js`, and `.meta.json` outputs
- ✅ CLI/runtime smoke coverage exercises the new library and bundle artifact flows

### Stage 2.2 - Public Effect Reporting
- ✅ `kali effects` emits native JSON effect reports for source roots
- ✅ `kali package-effects` emits native JSON package effect reports for installed packages
- ✅ `check/build --sandbox` reject inferred effects that exceed the active policy
- ✅ Positive CLI/runtime smoke coverage replaces the old unavailable-command gates

### Stage 1.8 - Deno API compatibility scaffold
- ✅ `kali_api_deno` now exposes the Deno-oriented host-support layer on top of the shared Web baseline
- ✅ Read-only env/args views, deterministic filesystem helpers, and the query-only permissions facade are available for the Phase-1 standalone context

### Stage 2.1 - HIR object-literal normalization follow-up
- ✅ Object-literal properties now lower through a dedicated `ObjectProperty` HIR node
- ✅ Property keys lower as literals, so MIR escape analysis no longer mistakes them for bindings
- ✅ Stable heap-store shapes now feed the ownership analyzer for object-literal value escapes
- ✅ Array-element and member-assignment heap-store flows now have explicit MIR ownership coverage
- ✅ Aliased function-expression calls now preserve direct-callee escape precision for local function-valued bindings
- ✅ Alias chains of function expressions now resolve to the canonical lowered target, including anonymous function expressions

### Stage 4.1 - Runtime dynamic import graph lookup
- ✅ Browser bundle JS now normalizes runtime `loadDynamicImport(specifier)` requests before target lookup, so path-equivalent runtime specifiers resolve through the bundle-local map instead of requiring an exact static spelling.
- ✅ Browser bundle smoke coverage now exercises a normalized runtime specifier (`./sub/../lazy.ts`) against a discovered chunk target.
- ✅ Browser bundle chunk discovery now folds const-bound static `import(...)` fragments before emitting the chunk graph, so `import((root + name))` can discover the same linked target as the literal concatenation cases.

### Stage 4.2 - Ownership-envelope preservation follow-up
- ✅ `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, and `KaliCore.Safety.releaseAndCollectPreservesOwnership` now keep the ownership environment unchanged across the release-only, decrement, and collection helpers.

### Stage 4.2 - Ownership provenance follow-up
- ✅ `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` now makes the surviving release-and-collect heap cells' original ownership tag explicit alongside their provenance and name preservation.

### Stage 4.2 - Release-and-decrement ownership follow-up
- ✅ `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership` now makes the decrement helper's surviving heap provenance explicit alongside its original ownership tag, `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount` packages the surviving-cell provenance/positivity split, and `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount` keeps the ownership tag explicit.

- ✅ `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation` now give exact heap-membership characterisations for the decrement and collection helpers.

## Next Work
- [x] Stage 4.2 pure release-origin helper sync closed
  - Confirmed `KaliCore.Safety.releaseRefHeapCellOrigin` is already present in the proof-backed boundary and that the summary / tracker docs are already aligned with the published RC snapshot wording for the pure release helper slice.
  - Closed the stale planned-update note without widening the published boundary.
- [x] Stage 4.2 heap-positive testing-summary sync
  - Synced `specs/16-testing.md` so the repository-state note and proof-backed-claims guidance now explicitly name the latest RC snapshot theorem inventory, including the zero-count collection/removal and positive-count/target-cell helper theorems.
  - Synced `specs/19-feature-maturity.md` so the verification-baseline clarification now names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly alongside the other RC snapshot lemmas.
- [x] Stage 4.2 heap-characterisation wording sync
  - Synced `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-backed boundary summary now names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation` explicitly alongside the surrounding RC snapshot inventory.
  - Extended the proof-summary anti-drift guard so `crates/kali_cli/tests/schema_docs.rs` now also checks `specs/16-testing.md` for the canonical proof-backed summary and RC theorem inventory.
- [x] Browser bundle source-map companions
  - `kali build --bundle` now emits a deterministic `.js.map` companion and appends the matching `sourceMappingURL` footer.
- [x] Browser bundle chunk artifacts for literal dynamic imports
  - `kali build --bundle` now emits deterministic chunk directories for literal `import("...")` boundaries, including `.wasm`, `.js`, `.map`, and metadata companions for each discovered chunk.
- [x] Broader package-shape / host-fit diagnostics matrix coverage
  - Added host-fit coverage for `node:fs` and `require("child_process")` package entrypoints.
- [x] CLI integration coverage for install repair/prune scenarios
  - Added CLI smoke coverage for pruning stale registry layouts back to an empty install state.
- [x] Build direct-input shape enforcement
  - `kali build` now rejects multi-file invocations with the canonical `E5008` usage diagnostic and remains a single-primary-input command.
- [x] Phase-gated later-surface placeholders and smoke coverage
  - Added `effects`, `package-effects`, `package-audit`, `build --capi`, `build --component`, and `run`/`test` API-surface gating so Phase-2+ surfaces now fail with the canonical `E5006` path instead of plain unknown-command parsing.
- [x] Shared `--compat` CLI plumbing for the Phase-4 compatibility vocabulary
  - Source-graph commands now parse `--compat` / `compat.features` requests, surface them in the command context, and reject the unavailable `eval` path through the canonical `E5006` gate instead of silently dropping the request.
- [x] Function() compatibility path for simple statically-resolved bodies
  - `new Function("return 1 + 2;")()` now rewrites through the shared `--compat eval` path and executes in the runtime smoke suite.
- [x] Embedding API scaffolding
  - `kali_embed` now exposes `KaliCompiler`, `CompiledArtifact`, `LibArtifact`, and deterministic WIT sidecar generation for the statically known export surface.
- [x] Stage 3.1 optimization scaffolding
  - `kali_optimize` now performs release constant folding, constant-branch elimination, and release-advanced algebraic identities, the CLI build path wires those passes into WASM generation, and `--max-specializations` now overrides the deterministic specialization budget used by the optimizer/cache path.
  - Layout specialization now also folds const-bound array element reads when the index is statically known or bound to a constant numeric value, extending the object-layout fast path.
- [x] Stage 3.1 MIR-driven specialization follow-up
  - MIR-aware call-site specialization now clones larger functions whose parameter layouts are stable enough to justify partial substitution, then reoptimizes the specialized body so literal-heavy hot paths can fold further before codegen.

### Stage 3.1 - Closure/struct-layout specialization follow-up
- ✅ Shared closure-valued MIR bindings now collapse to one specialization when multiple higher-order call sites share the same layout signature.
- ✅ Shared struct-valued MIR bindings now also collapse to one specialization when multiple higher-order call sites share the same layout signature, and the regression now covers three matching call sites so the reuse shape stays pinned down.
- [x] Stage 3.2 Node API layer scaffold
  - Added `kali_api_node` helpers for process/path/crypto/events/buffer/util plus fs/url/os scaffolding and unit tests; the Node-targeted command path is now wired through check/build/run/test and node-only import resolution in the analysis context.
  - Expanded the helper layer with Node-style assertion helpers and a synchronous `util.promisify` bridge so the documented Node helper surface is closer to the planned phase-3 subset.
  - The helper layer now exposes `NodePath`, `NodeUrl`, `NodeCrypto`, `NodeUtil`, `NodeAssert`, and `NodeRuntimeProjection` facades so future linker registration has a single Node-host surface to project through.
  - `NodePath::relative` now rounds out the lexical path helper slice alongside normalize/join/resolve/dirname/basename/extname, and the runtime linker projects it through `kali:node`.
  - `NodeUrl::parse` / `NodeUrl::resolve` now round out the URL helper slice, and the runtime linker projects them through `kali:node`.
  - The runtime linker now consumes the Node projection facade for `fs/promises`, stream, HTTP, URL, and process argv/env host imports when the effective API surface is `node`.
  - Install-time package host-fit validation now keys off the project `compilerOptions.apiSurface`, so Node-targeted installs can accept Node-only builtins while the default standalone context still rejects them with `E6005`.
  - Runtime-linker coverage now also exercises Node util formatting, assert-equality, and buffer hex round-tripping imports with dedicated smoke coverage.
  - Runtime-linker coverage now also exercises Node-style event listener registration/emission imports with dedicated smoke coverage.
- [x] Stage 4.2 proof boundary widening
  - `KaliCore.Soundness` now mechanizes the widened closed fragment (literals, variables, closed functions, application, sequencing, conditionals, assignment, and try/catch).
  - `KaliCore.Safety.noDanglingReference` is mechanized for the current RC snapshot model, `liveRefsAreOwnedAndAllocated` projects live references back to ownership/allocation, `releaseRefLiveRefsAreOwnedAndAllocated` / the release-only helper theorem `releaseRefLiveRefsFiltered` and `releaseAndDecrementLiveRefsFiltered` / `releaseAndCollectLiveRefsFiltered` keep the live-reference list as the target-filtered original live set, the helper-level no-dangling-reference corollaries `releaseRefNoDanglingReference` / `releaseAndDecrementNoDanglingReference` / `releaseAndCollectNoDanglingReference` keep the release-path hygiene explicit, `releasePreservesWellFormed` records the live-to-released transition, `releaseAndDecrementPreservesWellFormed` keeps the refcount-decrement update helper honest, `releaseAndDecrementRecorded` / `releaseAndDecrementDecrementsTargetCell` / `releaseAndDecrementKeepsTargetCellWhenPositiveCount` / `releaseAndDecrementHeapCellOrigin` / `releaseAndDecrementHeapCellOriginAndOwnership` / `releaseAndDecrementHeapCellOriginAndPositiveCount` / `releaseAndDecrementKeepsOtherPositiveCountCells, releaseAndDecrementKeepsOriginalPositiveCountCells` / `releaseAndDecrementZeroesLastTargetCell` / `releaseAndCollectRecorded` / `releaseAndCollectDropsZeroCountCells` / `releaseAndCollectRemovesZeroCountCells` / `releaseAndCollectKeepsPositiveCountCells` / `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` / `releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` / `releaseAndCollectDropsOriginalZeroCountCells` / `releaseAndCollectHeapIsPositiveCountFilter` / `releaseAndCollectHeapCellOrigin` / `releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` / `releaseAndCollectHeapCellsHavePositiveCount` / `releaseAndCollectPreservesOtherLiveRefs` / `releaseAndCollectReleasedNotLiveRef` / `releaseAndDecrementReleasedNotLiveRef` / `releaseAndDecrementLiveRefsAreOwnedAndAllocated` / `releaseAndCollectLiveRefsAreOwnedAndAllocated` / `releaseRefReleasedRefsCons` / `releaseAndDecrementReleasedRefsCons` / `releaseAndCollectReleasedRefsCons` / `releaseRefPreservesReleasedRefs` / `releaseAndDecrementPreservesReleasedRefs` / `releaseAndCollectPreservesReleasedRefs` keep the helper's release bookkeeping explicit, and `releasedNotLive` / `releasedNotLiveRef` record the release-path liveness split and live/released disjointness.
  - `KaliIR.HIRModel` records the structural lowering equations for `lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_throw`, and `lower_tr`.
  - `KaliIR.Value` and `KaliIR.LoweringCorrectness.lower_preserves_value` add the current HIR value fragment to the lowering story, `KaliIR.LoweringCorrectness.lower_preserves_step` adds the small-step lowering-preservation bridge for the current HIR subset, including bare throw, and `KaliIR.LoweringCorrectness.lower_preserves_steps` lifts that result to finite traces.
  - `proofs/BOUNDARY.md` now publishes the proof-backed boundary for that slice, and the canonical repository summary is aligned with it.


### Stage 4.2 - RC decrement/zeroing follow-up
- ✅ `releaseAndDecrementHeapCellOrigin` now proves the decrement helper's surviving heap cells still come from the original heap, with only the released target decremented or left unchanged.
- ✅ `releaseAndDecrementKeepsTargetCellWhenPositiveCount` now proves the decrement helper keeps the target cell in the heap when the decremented count stays positive.
- ✅ `releaseAndDecrementZeroesLastTargetCell` now proves the decrement helper zeros the target cell when the released reference was the last live count.

### Stage 4.2 - Release-set monotonicity follow-up
- ✅ `releaseRefPreservesReleasedRefs`, `releaseAndDecrementPreservesReleasedRefs`, and `releaseAndCollectPreservesReleasedRefs` keep already-released references preserved across the release-only, decrement, and collection helpers.


### Stage 4.2 - RC decrement/live-preservation follow-up
- ✅ `releaseAndDecrementKeepsOtherHeapEntries` now proves the decrement helper leaves unrelated heap entries untouched.
- ✅ `releaseAndDecrementKeepsOtherPositiveCountCells, releaseAndDecrementKeepsOriginalPositiveCountCells` now proves positive-count cells from the original heap survive on the decrement path when they are not the released target.
- ✅ `releaseAndDecrementPreservesOtherLiveRefs` now proves non-target live references remain live after the decrement helper runs.
- ✅ `releaseAndDecrementLiveRefsAreOwnedAndAllocated` now keeps the surviving live refs anchored in ownership/allocation after the decrement step.

### Stage 4.2 - RC helper ownership/allocation follow-up
- ✅ `releaseAndCollectLiveRefsAreOwnedAndAllocated` now keeps the surviving live refs anchored in ownership/allocation after the local collection helper runs.

### Stage 4.2 - RC zero-count collection follow-up
- ✅ `releaseAndCollect` now filters zero-count cells after the decrement pass.
- ✅ `releaseAndCollectRecorded` keeps the local collection helper's release-recording explicit.
- ✅ `releaseAndCollectDropsZeroCountCells` explicitly removes zero-count cells from the decrement pass before the collected heap is returned.
- ✅ `releaseAndCollectRemovesZeroCountCells` proves the freed decrement target is not retained in the collected heap.
- ✅ `releaseAndCollectKeepsPositiveCountCells` proves the local collection helper keeps the positive-count cells from the decrement pass.
- ✅ `releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` proves positive-count cells from the original heap survive when they are not the released target and remain positive-count after collection.
- ✅ `releaseAndCollectDropsOriginalZeroCountCells` proves original zero-count cells are removed from the final heap.
- ✅ `releaseAndCollectPreservesOtherLiveRefs` now proves other live references remain live after the local collection helper runs.
- ✅ `releaseAndCollectPreservesWellFormed` proves the remaining live set stays well-formed after zero-count collection.
- ✅ `releaseAndCollectReleasedNotLiveRef` keeps the local collection helper's live/released disjointness explicit.
- ✅ `releaseAndCollectHeapIsPositiveCountFilter` records the local collection helper's heap as exactly the positive-count filter of the decrement pass.
- ✅ `releaseAndCollectHeapCellsHavePositiveCount` now states the local collection helper's final heap contains only positive-count cells.
- ✅ `releaseAndCollectHeapCellOrigin` proves every surviving collected heap cell still comes from the original heap, with only the released target decremented.
- ✅ `releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` now makes the surviving collected heap cells' original name and ownership tag explicit.
- ✅ `releaseAndCollectKeepsOtherHeapEntries` now keeps unrelated positive-count heap entries in the collected heap.
- ✅ `releaseRefPreservesOwnership`, `releaseAndDecrementPreservesOwnership`, and `releaseAndCollectPreservesOwnership` keep the ownership environment unchanged across the release-only, decrement, and collection helpers.


### Stage 4.2 - RC unrelated-heap preservation follow-up
- ✅ `releaseAndCollectKeepsOtherHeapEntries` now keeps unrelated positive-count heap entries in the collected heap, making the helper-level unrelated-heap preservation story explicit.

### Stage 4.2 - Pure release-helper follow-up
- ✅ `releaseRefLiveRefsAreOwnedAndAllocated` now keeps the pure release helper's surviving live references anchored in ownership and allocation, and `releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` now makes the release-only provenance story explicit.
- ✅ `PLAN-4.2-STATUS.md` now also names `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCellOrigin` and `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` in the top-level memory-safety summary, so the plan tracker stays as explicit as the published boundary.
- ✅ `releaseRefLiveRefsFiltered`, `releaseAndDecrementLiveRefsFiltered`, and `releaseAndCollectLiveRefsFiltered` now keep the live-reference list filtered to the released target across the release-only, decrement, and collection helpers.
- ✅ `releaseRefReleasedNotLiveRef` now keeps released references disjoint from the live set after the pure release helper runs.
- ✅ `releaseRecorded` still records the released reference in the released set after the pure release step.

### Stage 3.3 - Package corpus breadth expansion
- ✅ Added browser, utility, and Node-runner corpus cases that resolve published exports maps and subpath entrypoints for `react`, `preact`, `vue`, `svelte`, `lit`, `ramda`, `rxjs`, `immer`, `uuid`, `typescript`, `esbuild`, `date-fns`, `lodash-es`, `vitest`, `jest`, and `mocha`, broadening the package-support corpus beyond the original single-entrypoint stubs.
- ✅ Added `./*` exports-pattern corpus coverage so the representative browser and utility package sets now exercise wildcard subpath exports routed through nested `src/` subtrees.
- ✅ Added browser replacement-map coverage so the representative browser package set now exercises exact-path rewrites and `false` blocks after entry selection, alongside the existing exports-map/subpath and browser-conditional-export cases.
- ✅ Added dual-package, browser-conditional-export, mixed-format, browser string-entry, browser false-blocking, module-only, scoped-package, and typed-export-branch corpus coverage so the representative package set now exercises conditional exports across browser/import/require branches plus browser-string overrides, browser-field blocking, and mixed CJS/ESM entrypoints.
- ✅ Added module-only corpus coverage so the representative browser and utility package sets now exercise `package.json#module` fallback resolution as a standalone published shape.
- ✅ Added browser internal-browser-rewrite corpus coverage so the representative browser package set now exercises browser-field rewrites across an internal dependency chain instead of only top-level entrypoint rewrites.
- ✅ Added module-entry internal-dependency corpus coverage so the representative utility package set now exercises internal relative imports inside a module-only package instead of only a single-file module entrypoint.
- ✅ `plan/phase-3/03-ecosystem-breadth.md` now explicitly enumerates the representative browser/utility package-shape cases already covered by the corpus, keeping the implementation playbook aligned with the Stage 3.3 evidence.
- ✅ Added scoped-package corpus coverage so the representative package set now exercises `@scope/name` identities plus the scoped `@types/scope__name` fallback naming convention in both browser-targeted and standalone contexts.
- ✅ Added typed-export-branch corpus coverage so the representative browser package set now exercises `exports` objects that carry `types` conditions alongside the runtime branches, keeping the corpus aligned with common modern package metadata.
- ✅ Added exports-string corpus coverage so the representative browser and utility package sets now exercise top-level string `exports` roots alongside the existing map-based exports cases.

### Stage 4.2 - Proof-boundary anti-drift test
- ✅ `crates/kali_cli/tests/schema_docs.rs` now asserts that the `proofs/BOUNDARY.md` covered-path inventory matches the actual `proofs/*.lean` source set, now checks the published theorem inventory against the concrete Lean theorem and lemma names, and now also verifies the canonical proof-summary docs keep the current RC theorem names and proof-backed summary string in sync, so deleting or adding a proof file or drifting summary prose without updating the manifest or docs fails `cargo test`; the progress tracker now calls out that theorem-name inventory and summary-doc inventory check alongside the path-level anti-drift guard.
- ✅ The proof-summary guard now explicitly pins the heap-characterisation theorem names `releaseAndDecrementHeapCharacterisation` and `releaseAndCollectHeapCharacterisation` as well, so the summary docs stay aligned with the published RC snapshot inventory.

### Stage 4.2 - Lowering value-preservation helper
- ✅ Added the HIR value fragment plus `KaliIR.LoweringCorrectness.lower_preserves_value`, which records that the current core-lifted HIR value forms lower back to core values in the proof model.

### Stage 4.2 - RC origin/positivity conjunction helper
- ✅ Added a reusable `releaseAndCollectHeapCellOriginAndPositiveCount` helper theorem that packages the surviving-cell provenance and positive-count facts for the local collection helper on top of the existing origin and positivity lemmas.
- ✅ Synced the boundary manifest and verification summaries so the new helper theorem is named explicitly alongside the rest of the RC snapshot slice.

### Stage 4.2 - RC target-allocation follow-up
- ✅ `releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, `releaseAndCollectTargetCellAllocatedWhenPositiveCount`, and `releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` now make the surviving target-cell allocation bridge explicit on the decrement and collection helpers when the count stays positive.
- ✅ Synced `PLAN-4.2-STATUS.md` and the proof-backed verification summaries so the progress trackers name the target-allocation corollaries alongside the existing RC helper slice.
