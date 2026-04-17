# TODO

## Completed

### Stage 4.1 - Package-audit availability
- ✅ `kali package-audit` now runs without requiring `--preview`; the preview flag remains accepted as a no-op compatibility shim.

### Stage 4.1 - Eval compatibility gating
- ✅ `--compat eval` now accepts dynamically constructed eval / Function() strings derived from constant program-state fragments.
- ✅ `check` / `run` now reject `eval` and `Function()` usage unless the shared `--compat eval` gate is enabled.

### Plan completion-gate sync
- ✅ `PLAN.md` phase completion gates now reflect the current stage status, and the Phase 2 effect-report completion line now matches the schema-v1 contracts used by the stage docs and schema specs.

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

## Next Work
- [x] Stage 4.2 heap-positive testing-summary sync
  - Synced `specs/16-testing.md` so the repository-state note and proof-backed-claims guidance now explicitly name the latest RC snapshot theorem inventory, including the zero-count collection/removal and positive-count/target-cell helper theorems.
  - Synced `specs/19-feature-maturity.md` so the verification-baseline clarification now names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly alongside the other RC snapshot lemmas.
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

### Stage 3.1 - Closure-layout specialization follow-up
- ✅ Shared closure-valued MIR bindings now collapse to one specialization when multiple higher-order call sites share the same layout signature.
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
  - `KaliCore.Safety.noDanglingReference` is mechanized for the current RC snapshot model, `liveRefsAreOwnedAndAllocated` projects live references back to ownership/allocation, `releaseRefLiveRefsFiltered` / `releaseAndDecrementLiveRefsFiltered` / `releaseAndCollectLiveRefsFiltered` keep the live-reference list as the target-filtered original live set, `releasePreservesWellFormed` records the live-to-released transition, `releaseAndDecrementPreservesWellFormed` keeps the refcount-decrement update helper honest, `releaseAndDecrementRecorded` / `releaseAndDecrementDecrementsTargetCell` / `releaseAndDecrementHeapCellOrigin` / `releaseAndDecrementZeroesLastTargetCell` / `releaseAndCollectRecorded` / `releaseAndCollectDropsZeroCountCells` / `releaseAndCollectRemovesZeroCountCells` / `releaseAndCollectKeepsPositiveCountCells` / `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount` / `releaseAndCollectKeepsOtherPositiveCountCells` / `releaseAndCollectDropsOriginalZeroCountCells` / `releaseAndCollectHeapIsPositiveCountFilter` / `releaseAndCollectHeapCellOrigin` / `releaseAndCollectHeapCellsHavePositiveCount` / `releaseAndCollectPreservesOtherLiveRefs` / `releaseAndCollectReleasedNotLiveRef` / `releaseAndDecrementReleasedNotLiveRef` / `releaseAndDecrementLiveRefsAreOwnedAndAllocated` / `releaseAndCollectLiveRefsAreOwnedAndAllocated` / `releaseRefPreservesReleasedRefs` / `releaseAndDecrementPreservesReleasedRefs` / `releaseAndCollectPreservesReleasedRefs` keep the helper's release bookkeeping explicit, and `releasedNotLive` / `releasedNotLiveRef` record the release-path liveness split and live/released disjointness.
  - `KaliIR.HIRModel` records the structural lowering equations for `lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_throw`, and `lower_tr`.
  - `KaliIR.LoweringCorrectness` adds both a small-step lowering-preservation bridge and a finite-trace lowering-preservation bridge for the current HIR subset, including bare throw.
  - `proofs/BOUNDARY.md` now publishes the proof-backed boundary for that slice, and the canonical repository summary is aligned with it.


### Stage 4.2 - RC decrement/zeroing follow-up
- ✅ `releaseAndDecrementHeapCellOrigin` now proves the decrement helper's surviving heap cells still come from the original heap, with only the released target decremented or left unchanged.
- ✅ `releaseAndDecrementZeroesLastTargetCell` now proves the decrement helper zeros the target cell when the released reference was the last live count.

### Stage 4.2 - Release-set monotonicity follow-up
- ✅ `releaseRefPreservesReleasedRefs`, `releaseAndDecrementPreservesReleasedRefs`, and `releaseAndCollectPreservesReleasedRefs` keep already-released references preserved across the release-only, decrement, and collection helpers.


### Stage 4.2 - RC decrement/live-preservation follow-up
- ✅ `releaseAndDecrementKeepsOtherHeapEntries` now proves the decrement helper leaves unrelated heap entries untouched.
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
- ✅ `releaseAndCollectKeepsOtherPositiveCountCells` proves positive-count cells from the original heap survive when they are not the released target.
- ✅ `releaseAndCollectDropsOriginalZeroCountCells` proves original zero-count cells are removed from the final heap.
- ✅ `releaseAndCollectPreservesOtherLiveRefs` now proves other live references remain live after the local collection helper runs.
- ✅ `releaseAndCollectPreservesWellFormed` proves the remaining live set stays well-formed after zero-count collection.
- ✅ `releaseAndCollectReleasedNotLiveRef` keeps the local collection helper's live/released disjointness explicit.
- ✅ `releaseAndCollectHeapIsPositiveCountFilter` records the local collection helper's heap as exactly the positive-count filter of the decrement pass.
- ✅ `releaseAndCollectHeapCellsHavePositiveCount` now states the local collection helper's final heap contains only positive-count cells.
- ✅ `releaseAndCollectHeapCellOrigin` proves every surviving collected heap cell still comes from the original heap, with only the released target decremented.
- ✅ `releaseRefPreservesOwnership`, `releaseAndDecrementPreservesOwnership`, and `releaseAndCollectPreservesOwnership` keep the ownership environment unchanged across the release-only, decrement, and collection helpers.


### Stage 4.2 - Pure release-helper follow-up
- ✅ `releaseRefLiveRefsAreOwnedAndAllocated` now keeps the pure release helper's surviving live references anchored in ownership and allocation.
- ✅ `releaseRefLiveRefsFiltered`, `releaseAndDecrementLiveRefsFiltered`, and `releaseAndCollectLiveRefsFiltered` now keep the live-reference list filtered to the released target across the release-only, decrement, and collection helpers.
- ✅ `releaseRefReleasedNotLiveRef` now keeps released references disjoint from the live set after the pure release helper runs.
- ✅ `releaseRecorded` still records the released reference in the released set after the pure release step.

### Stage 3.3 - Package corpus breadth expansion
- ✅ Added browser, utility, and Node-runner corpus cases that resolve published exports maps and subpath entrypoints for `react`, `preact`, `vue`, `ramda`, `rxjs`, `uuid`, `vitest`, and `jest`, broadening the package-support corpus beyond the original single-entrypoint stubs.
- ✅ Added dual-package, browser-conditional-export, and mixed-format corpus coverage so the representative package set now exercises conditional exports across browser/import/require branches plus mixed CJS/ESM entrypoints.

### Stage 4.2 - Proof-boundary anti-drift test
- ✅ `crates/kali_cli/tests/schema_docs.rs` now asserts that the `proofs/BOUNDARY.md` covered-path inventory matches the actual `proofs/*.lean` source set, now checks the published theorem inventory against the concrete Lean theorem and lemma names, and now also verifies the canonical proof-summary docs keep the current RC theorem names and proof-backed summary string in sync, so deleting or adding a proof file or drifting summary prose without updating the manifest or docs fails `cargo test`; the progress tracker now calls out that theorem-name inventory and summary-doc inventory check alongside the path-level anti-drift guard.
