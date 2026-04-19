# TODO

## Current Remaining Work

The repository has closed the stage-plan milestones reflected in `PLAN.md`; the remaining work is
follow-up widening rather than unfinished baseline delivery:

- widen specialization depth beyond the current MIR-aware layout-specialized path; the optimizer
  already now covers owner-scoped MIR binding layouts, quoted string- and template-literal
  signatures, `null` / `undefined` / boolean / numeric / signed-zero / BigInt literal signatures,
  array-layout widening, direct array-literal shape widening, nested MIR-bound bindings,
  object-literal property-order canonicalization, the nested-call regression
  `release_recursively_specializes_nested_mir_call_sites`, and the literal-shaped
  concrete-argument fallback that can clone deterministic specializations even without MIR layout
  metadata, so the remaining work is the fuller generic-instantiation planner and cross-module
  specialization model while keeping the specialization budget and benchmark evidence honest,
- widen the representative package corpus and browser/runtime interoperability without overclaiming
  support rungs; the current browser/runtime baseline now includes deterministic event primitives for
  `AbortController`, `EventTarget`, and `CustomEvent`, plus stub surfaces for `BroadcastChannel`, `WebSocket`, `Worker`, and `IndexedDB` in addition to the shared `Blob` / `File` / `FormData` / storage / `FileReader` helpers, and the browser web-baseline interop corpus now also exercises those surfaces alongside `fetch`, `Headers`, `Request`, `Response`, `date-fns`, `lodash`, `lodash-es`, `ramda`, `uuid`, `recoil`, `mitt`, `swr`, `formik`, `jotai`, `nanostores`, `pinia`, `xstate`, `valtio`, `clsx`, `classnames`, `vue-router`, `react-router`, `zod`, `svelte`, `lit`, `next`, `hono`, `@vueuse/core`, `@emotion/react`, `@emotion/styled`, `@heroicons/react`, `react-dom`, `framer-motion`, `@storybook/react`, `@tanstack/react-form`, `@floating-ui/react`, `@headlessui/react`, `@chakra-ui/react`, `@mantine/core`, `@mui/material`, `@radix-ui/react-dialog`, `@tanstack/react-query`, `@tanstack/react-table`, `@testing-library/dom`, `@testing-library/user-event`, `URLSearchParams`, `TextEncoder`, `TextDecoder`, `superjson`, `msw`, and the existing browser representatives; the browser exports-map and pattern-exports corpus also now exercises `hono` explicitly, so the package-shape notes stay aligned with the narrower slice coverage,
- widen the proof-backed boundary beyond the current published RC snapshot + lowering slice while
  keeping `proofs/BOUNDARY.md`, `PLAN-4.2-STATUS.md`, and summary docs synchronized; the current RC
  helper slice is already named explicitly through the release-only companion theorem
  `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the
  decrement-path companion theorem
  `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the
  decrement-path target origin/positive-count theorem
  `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount`, which the current Stage 4.2 progress wording now keeps explicit, the decrement-path iff bridge `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`, the
  collection target iff bridge `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`,
  the collection helper's heap-filter-and-linear-memory corollary
  `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`, the heap-characterisation companions
  `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`, the combined
  wellformedness/linear-memory corollaries
  `KaliCore.Safety.releaseRefPreservesWellFormedAndLinearMemory`,
  `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndLinearMemory`, and
  `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndLinearMemory`, the mechanized
  `KaliCore.Safety.noDanglingReference` theorem plus the helper-level corollaries
  `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`,
  `KaliCore.Safety.releaseAndCollectNoDanglingReference`, `releasedNotLive`, and `releasedNotLiveRef`, and the target-cell
  allocation corollaries `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`
  and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`.

- the current RC snapshot helper slice now also names `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` explicitly, matching the existing origin/ownership companion style.

## Completed

### Codegen optimization-placeholder cleanup
- ✅ Removed the stale `kali_codegen` no-op optimization placeholder; codegen now assumes the upstream optimizer has already run, and the target config no longer carries a dead `optimize` flag.

### Stage 3.1 - Tagged-parameter specialization widening
- ✅ `kali_optimize` can now specialize tagged parameters when the concrete call arguments have a stable literal or MIR-backed layout, so the MIR-aware monomorphisation path no longer stops at the existing non-tagged-layout gate.
- ✅ Tagged-parameter call sites that are too large to inline but still within the deterministic budget now also specialize, so the deeper monomorphisation path no longer depends on the old size cutoff.
- ✅ Added regression tests that prove the tagged-parameter path can still fold concrete calls down to literal results after specialization, including the non-inlined small-function case.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - String-literal call-site specialization widening
- ✅ Quoted string-literal call-site arguments now carry distinct specialization signatures, so different string literals can split into separate clones instead of collapsing onto the generic tagged fallback.
- ✅ Added a regression test that proves distinct quoted string-literal call sites produce different specialized clones while still respecting the deterministic specialization budget.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Template-literal call-site specialization widening
- ✅ No-substitution template-literal call-site arguments now reuse the same literal-signature path as quoted strings, so backtick-delimited string literals can split into distinct specialized clones instead of collapsing onto the generic tagged fallback.
- ✅ Added regression coverage that proves distinct template-literal call sites produce different specialized clones while still respecting the deterministic specialization budget.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Nullish-literal specialization widening
- ✅ `null` and `undefined` call-site arguments now carry distinct literal signatures instead of collapsing onto the old zero-valued fallback, so the MIR-aware specialization path stays honest about nullish constants.
- ✅ Added a regression test that proves `null` and `undefined` specialize to separate clones while repeated `null` call sites still share the same specialized function name.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Nested MIR-bound binding specialization widening
- ✅ Nested MIR-bound bindings inside object-literal call sites now contribute MIR layout signatures during call-site specialization, so same-shaped composite arguments can still split when their scoped binding layouts differ.
- ✅ Added regression coverage that drives identical object-literal call sites through different scoped binding layouts and proves the optimizer emits distinct specializations instead of one shared clone.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Numeric-literal specialization widening
- ✅ Numeric-literal call-site arguments now carry value-specific specialization signatures, so `1` and `2` no longer collapse onto the same specialized clone when the MIR plan can see them as constants.
- ✅ Added regression coverage that proves repeated numeric literals still share one specialized clone while distinct numeric call-site values split to separate specialized functions.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - BigInt-literal specialization widening
- ✅ BigInt-literal call-site arguments now carry distinct specialization signatures, so `1n` and `2n` no longer collapse onto the same specialized clone when the MIR plan can see them as constants.
- ✅ Added regression coverage that proves repeated BigInt literals still share one specialized clone while distinct BigInt call-site values split to separate specialized functions.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Signed-zero specialization widening
- ✅ Signed-zero numeric literals now preserve `-0` as a distinct specialization signature from `0`, so the optimizer keeps JavaScript's signed-zero edge case visible in the literal-signature path without changing the deterministic budget story.
- ✅ Added regression coverage that proves repeated `-0` call sites still share one specialized clone while `0` and `-0` split to separate specialized functions.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.3 - Browser/runtime interop corpus widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now exercises the web-baseline interop slice with representative browser and utility package cases that create `AbortController`, `EventTarget`, `Blob`, `File`, `FileReader`, and `structuredClone` payloads alongside package imports.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `solid-js` in the browser web-baseline interop slice and `rxjs` in the utility web-baseline interop slice, so the browser/runtime interoperability widening now covers a little more representative package breadth without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@testing-library/user-event` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now carries one more representative browser testing-library package through the browser command path without changing the documented support-rung story.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `date-fns`, `lodash-es`, and `nanoid` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now carries three more representative utility-package names through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `solid-js` in the browser exports-map corpus alongside the existing browser representatives, so the browser/runtime interoperability widening now covers one more representative package shape without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `clsx` in the browser and utility corpus slices, so the representative package corpus now carries one more lightweight package name through the existing support-rung checks without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `solid-js` in the browser pattern-exports corpus, `vue-router` in the browser exports-map / pattern-exports slices, `react-router` in the browser web-baseline interop / exports-map / pattern-exports slices, `@reduxjs/toolkit` in the browser typed-export-branch corpus, and `immer`, `typescript`, and `esbuild` in the utility module-entry and mixed-format slices, so the representative package corpus now spans a few more browser/tooling package shapes without changing the documented support rungs.
- ✅ The shared browser/runtime support library now also exposes an in-memory `FileReader` baseline, and `kali_api_deno` reexports it so browser-style code can read shared blob/file payloads deterministically without changing the public support-rung story.
- ✅ The shared browser/runtime support library now also exposes deterministic stub baselines for `WebSocket`, `Worker`, and `IndexedDB`, and `kali_api_deno` reexports those names so browser-style code can exercise the ambient surface without changing the public support-rung story.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also drives the deterministic `BroadcastChannel`, `WebSocket`, `Worker`, and `IndexedDB` browser-runtime stubs through the existing web-baseline interop slice, so the package corpus now carries the ambient browser stub surface through the browser and utility command paths without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `CustomEvent` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now covers one more browser event primitive without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `URLSearchParams` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now covers one more browser query-string primitive without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `zod` in the browser and utility web-baseline interop slices, so the representative package corpus now carries one more package name through both command paths without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@headlessui/react` across the browser web-baseline interop, exports-map, and browser-condition slices, so the scoped browser package corpus now covers one more representative UI package without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@chakra-ui/react` in the browser web-baseline interop slice, so the representative browser package corpus now covers one more scoped UI package without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `svelte` and `lit` in the browser web-baseline interop slice, so the representative package corpus now carries two more browser-oriented package names through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `ramda` and `uuid` in the browser web-baseline interop slice, so the representative package corpus now carries two more utility-package names through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@floating-ui/react`, `@mui/material`, `@radix-ui/react-dialog`, and `@tanstack/react-query` in the browser web-baseline interop slice, so the representative package corpus now carries four more scoped browser package names through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `axios` in the browser and utility web-baseline interop slices, so the representative package corpus now carries one more common pure-JS package through both command paths without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `next` in the browser web-baseline interop slice, so the representative browser package corpus now covers one more app-framework package without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `framer-motion` in the browser web-baseline interop slice, so the representative browser package corpus now carries one more browser UI package through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@testing-library/react` in the browser web-baseline interop slice, so the scoped browser corpus now carries one more representative testing-library package name through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@testing-library/dom` alongside `@testing-library/react` in the browser web-baseline interop slice, so the scoped browser corpus now carries both the React-oriented and DOM-oriented testing-library package names through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@emotion/styled` in the browser typed-export-branch slice, so the scoped browser package corpus now carries one more typed-export branch shape through a distinct UI package name without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `hono` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now carries one more representative browser/web-framework package name through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `hono` in the browser exports-map and pattern-exports slices, so the browser shape coverage now carries one more framework package through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@vueuse/core` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now carries one more representative scoped browser utility package name through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `TextEncoder` and `TextDecoder` in the browser web-baseline interop slice, so the browser/runtime interoperability widening now carries one more browser text-codec baseline through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `mitt` in the browser and utility web-baseline interop slices, so the representative package corpus now carries one more lightweight package name through both command paths without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `swr` in the browser and utility web-baseline interop slices, so the representative package corpus now carries one more browser/utility package name through both command paths without changing the documented support-rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `nanostores` in the browser and utility web-baseline interop slices, so the representative package corpus now carries one more browser/utility package name through both command paths without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `framer-motion` in the browser web-baseline interop slice, so the representative browser package corpus now carries one more browser UI package through the browser command path without changing the documented support rungs.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `crypto.randomUUID()` in the browser web-baseline interop slice, so the representative browser/runtime interop corpus now widens the browser crypto baseline without changing the documented support rungs.
- ✅ Kept the update narrow: this widens the package corpus and browser/runtime interoperability checks without changing the documented support rungs.

### Stage 3.3 - Scoped browser representative widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@radix-ui/react-dialog` in the browser web-baseline interop slice, so the browser package corpus now carries one more scoped browser package name through the browser command path without changing the documented support-rung story.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@heroicons/react` in the browser scoped exports-map and browser-condition slices, so the scoped browser corpus now carries one more representative UI package name through the browser command path without changing the documented support-rung story.
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `react-dom` across the browser exports-map and browser-condition slices, so the browser corpus now covers one more representative app-framework package shape without changing the documented support-rung story.
- ✅ `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` now name `@radix-ui/react-dialog` explicitly in the Stage 3.3 widening notes.
- ✅ `plan/phase-3/03-ecosystem-breadth.md` and `TODO.md` now name `@heroicons/react` explicitly in the Stage 3.3 widening notes.
- ✅ Kept the update narrow: this is another corpus-widening slice, not a support-rung change.

### Stage 3.1 - Recursive MIR-specialization revisit
- ✅ Newly created MIR-specialized clones are now recursively revisited under their own owner key, so clone-specific optimization can expose deeper specializable call sites while keeping the specialization budget deterministic.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimization model; it does not change the published benchmark or support claims.

### Stage 3.1 - Cross-owner generic specialization reuse
- ✅ Identical generic specializations are now reused across owners when the callee and argument signatures already match, so duplicate helper clones are avoided when the same generic call appears in multiple function scopes.
- ✅ Kept the update narrow: this is a specialization-dedup improvement inside the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Nested MIR-specialization depth regression
- ✅ `release_recursively_specializes_nested_mir_call_sites` now proves a specialized MIR clone can surface a second specializable call site inside its own body, so the deeper monomorphisation path stays regression-tested instead of stopping at the first clone.
- ✅ Kept the update narrow: this widens specialization depth inside the existing MIR-aware optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Array-layout specialization widening
- ✅ MIR-backed array bindings now preserve their element/length fingerprints during call-site specialization, so different array layouts can split into separate clones instead of collapsing onto a single shared body.
- ✅ Added regression coverage proving two callers with different array layouts now produce distinct specialized clones while still respecting the deterministic specialization budget.
- ✅ Kept the update narrow: this widens specialization depth inside the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Direct array-literal shape widening
- ✅ Direct array-literal call-site arguments now carry explicit `Value:array:len=...` shape signatures, so inline arrays of different lengths split into separate specialized clones even when the callee only sees a tagged parameter.
- ✅ Added regression coverage proving two direct array-literal call sites with different lengths produce distinct specialized clones while still respecting the deterministic specialization budget.
- ✅ Kept the update narrow: this widens specialization depth inside the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.1 - Object-literal property-order canonicalization
- ✅ Object-literal property order is now canonicalized in the specialization signature, so reordered but semantically identical object shapes reuse one specialized clone instead of splitting on insertion order.
- ✅ Added regression coverage proving object literals with the same properties in a different order now share a specialized clone while still respecting the deterministic specialization budget.
- ✅ Kept the update narrow: this widens specialization depth inside the existing optimizer model; it does not change the published benchmark or support claims.

### Stage 3.3 - Web Blob/File/FileReader baseline
- ✅ `kali_api_web` now exposes in-memory `Blob` and `File` primitives for the Web baseline, and `kali_api_deno` reexports them so the browser/runtime support library can model common blob/file payloads without changing the public support-rung story.
- ✅ `kali_api_web` now also exposes an in-memory `FileReader` baseline, and `kali_api_deno` reexports it so browser-style code can read shared blob/file payloads deterministically without changing the public support-rung story.
- ✅ `crates/kali_types` now recognizes `Blob`, `File`, and `FileReader` as builtin globals, keeping the ambient typing surface aligned with the support-library additions.

### Stage 3.3 - Web FormData baseline
- ✅ `kali_api_web` now also exposes an in-memory `FormData` baseline, `kali_api_deno` reexports it, and `crates/kali_types` recognizes `FormData` as a builtin global, so the browser/runtime support library can model deterministic multipart payloads without changing the public support-rung story.

### Stage 3.3 - In-memory browser storage baseline
- ✅ `kali_api_web` now exposes deterministic in-memory `localStorage` and `sessionStorage` buckets, so the browser interoperability slice has a deterministic shared-state baseline for browser-style code that expects storage APIs.
- ✅ `kali_api_deno` reexports the shared storage helpers alongside the other Web-baseline primitives, keeping the compatibility layer aligned with the browser support surface without changing any support-rung claims.

### Stage 4.2 - Lowering value-preservation summary sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, `KaliIR.LoweringCorrectness.lower_preserves_step`, and `KaliIR.LoweringCorrectness.lower_preserves_steps` alongside the widened HIR lowering-correctness slice.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a widening of the HIR semantic-preservation target.

### Stage 4.2 - Context lookup/remove groundwork
- ✅ Added `Context.lookup_remove_head` in `proofs/KaliCore/Semantics.lean` so the current proof model has a small context-sensitive lookup/remove helper available for future substitution widening.
- ✅ Added `Context.lookup_remove_head_other` alongside it so the context-removal groundwork now covers the matching non-head lookup case too.
- ✅ Added the more general `Context.lookup_remove_ne` helper so future substitution proofs can reuse a name-stable lookup/remove lemma without re-proving the non-head case.
- ✅ Kept the update narrow: this is helper groundwork for future proof work, not a new published boundary claim.

### Stage 4.2 - Pure release-helper positive-count wording sync
- ✅ `proofs/BOUNDARY.md` now explicitly says `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` preserves the release-only cells' positive count in addition to their original ownership tag.
- ✅ `PLAN.md`, `PLAN-4.2-STATUS.md`, and the current proof-boundary summaries now also keep the release-only linear-memory companion theorem named explicitly alongside the decrement and collection helper companions.
- ✅ Kept the update narrow: this is wording sync for the published boundary, not a new proof target.

### Stage 4.2 - Pure release helper origin/ownership follow-up closed
- ✅ `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership` is already present and named explicitly in the proof boundary plus the plan/progress summaries.
- ✅ Kept the stale follow-up closed without widening the published boundary.

### Stage 4.2 - Pure release-origin helper sync
- ✅ `KaliCore.Safety.releaseRefHeapCellOrigin` is still explicitly named in `proofs/BOUNDARY.md`, `plan/phase-4/02-formal-verification-depth.md`, and `PLAN-4.2-STATUS.md` alongside `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, and `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Live-reference filtering theorem naming sync
- ✅ `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and this tracker now name `KaliCore.Safety.releaseRefLiveRefsFiltered`, `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered`, and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered` explicitly alongside `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated` and the rest of the RC snapshot inventory.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins the exact live-reference filtering theorem names, so the proof-summary drift guard keeps the helper slice aligned with the published boundary inventory and the stage plan note.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Remaining bookkeeping wording sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliCore.Safety.releaseRecorded`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `releasedNotLive`, and `releasedNotLiveRef` explicitly alongside the rest of the RC snapshot inventory.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now also pins the same remaining bookkeeping corollaries and checks `plan/phase-4/02-formal-verification-depth.md` for the canonical proof-backed summary and theorem inventory, so the proof-summary drift guard keeps the helper slice aligned with the published boundary inventory and the stage plan note.
- ✅ Kept the update narrow: this is a wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Release-only helper wording sync
- ✅ `plan/phase-4/02-formal-verification-depth.md`, `PLAN-4.2-STATUS.md`, and the TODO stage summary now name `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated`, `KaliCore.Safety.releaseRefLiveRefsFiltered`, and `KaliCore.Safety.releasePreservesWellFormed` explicitly alongside the rest of the RC snapshot inventory.
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
- ✅ `proofs/BOUNDARY.md` now names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory` explicitly in the claimed theorem inventory, keeping the manifest aligned with the proof-state summary and the summary docs.
- ✅ The pure release helper now also has an explicit heap-characterisation theorem, `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, a plain origin theorem, `KaliCore.Safety.releaseRefHeapCellOrigin`, and a direct origin/ownership theorem, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, so the RC helper slice states the unchanged-heap case and the release-only provenance story directly alongside the decrement/collection heap characterisation theorems.
- ✅ `PLAN-4.2-STATUS.md` and the Stage 4.2 progress note now call out the release-only heap-characterisation theorem explicitly, keeping the pure release helper slice aligned with the published boundary inventory.

### Stage 4.2 - Pure release heap characterisation wording sync
- ✅ `TODO.md` now calls out `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseRefHeapCellOrigin` explicitly in the Stage 4.2 tracker, and the proof-boundary theorem inventory now also names `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly, so the pure release-helper slice stays named alongside the release-only live-reference and disjointness corollaries.
- ✅ The existing `PLAN-4.2-STATUS.md` progress note already reflects the same helper-level wording, keeping the proof-backed boundary inventory aligned with the current RC theorem set.

### Stage 4.2 - Proof-summary anti-drift guard widening
- ✅ `crates/kali_cli/tests/schema_docs.rs` now pins a broader current RC snapshot helper inventory, including the no-dangling-reference corollaries, released-reference cons-shape theorems, target-cell bookkeeping, zero-count collection/removal, and heap-characterisation corollaries.
- ✅ The same drift guard now also checks `TODO.md` for the current RC theorem inventory, so the progress tracker stays aligned with the published boundary wording.
- ✅ The drift guard now also pins the live-reference ownership/allocation projection theorem, the ownership-preservation corollaries, the surviving-live-reference corollary on the collection path, the released-not-live theorems, and the decrement-path positive-count guard so those proof-summary claims stay explicitly tracked.
- ✅ The same guard now also names `KaliCore.Safety.liveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `releasedNotLive`, and `releasedNotLiveRef` explicitly.

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

### Stage 4.2 - Collection target origin/positive-count wording sync
- ✅ Added `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, which packages the surviving collection-target provenance and positive-count fact explicitly.
- ✅ Synced `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN.md`, and the proof-summary anti-drift guard so the new theorem is named explicitly across the current boundary summaries.
- ✅ Kept the update narrow: this is another helper-level proof-summary sync for the published boundary, not a broader ownership/freeing widening.

### Stage 4.2 - Final-heap positive-count wording sync
- ✅ `PLAN-4.2-STATUS.md` and `plan/phase-4/02-formal-verification-depth.md` now name `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly, so the final-heap positivity story stays direct in the stage tracker.

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
- ✅ `TypeChecker::typecheck` now drains any pending annotation-resolution diagnostics from the shared context before returning, so the facade preserves the error set instead of acting like a pure no-op and keeps the stage 1.5 error story explicit at the facade boundary.

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
- ✅ `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` now makes the surviving release-and-collect heap cells' original ownership tag explicit alongside their provenance and name preservation.

### Stage 4.2 - Release-and-decrement ownership follow-up
- ✅ `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership` now makes the decrement helper's surviving heap provenance explicit alongside its original ownership tag, `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount` packages the surviving-cell provenance/positivity split, and `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount` keeps the ownership tag explicit.

- ✅ `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory` now give exact heap-membership characterisations for the decrement and collection helpers.

### Stage 4.2 - Target-allocation wording sync
- ✅ `PLAN-4.2-STATUS.md` and the TODO current-remaining-work note now name `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount` and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount` explicitly alongside the existing RC snapshot inventory, keeping the target-allocation bridge visible in the published boundary.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Decrement target-origin wording sync
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `proofs/BOUNDARY.md`, `PLAN-4.2-STATUS.md`, and `plan/phase-4/02-formal-verification-depth.md` now name `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` explicitly alongside the rest of the RC snapshot inventory.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Decrement target positive-count iff bridge
- ✅ `README.md`, `specs/16-testing.md`, and `specs/17-verification.md` now name `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` explicitly alongside the rest of the RC snapshot inventory.
- ✅ Kept the update narrow: this is a helper-level proof-summary sync for the published boundary, not a boundary widening.

### Stage 4.2 - RC predicate vocabulary sync
- ✅ The RC snapshot model now names the explicit `hasOwnership`, `allocated`, and `liveAnnotated` predicate vocabulary in the proof-boundary and progress-tracker summaries.
- ✅ `crates/kali_cli/tests/schema_docs.rs` now pins the same `hasOwnership` / `allocated` / `liveAnnotated` RC vocabulary so the proof-summary drift guard catches model-vocabulary wording drift in the published boundary docs.
- ✅ Kept the update narrow: this is a model-vocabulary wording sync, not a boundary widening.

## Recently Closed Work
- [x] Stage 4.2 collection target-cell iff summary sync
  - `PLAN-4.2-STATUS.md` now keeps `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` explicit in the published RC inventory, and the summary/docs already name the theorem consistently.
  - Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a boundary widening.

- [x] Stage 4.2 pure release-origin helper sync closed
  - Confirmed `KaliCore.Safety.releaseRefHeapCellOrigin` is already present in the proof-backed boundary and that the summary / tracker docs are already aligned with the published RC snapshot wording for the pure release helper slice.
  - Closed the stale planned-update note without widening the published boundary.
- [x] Stage 4.2 heap-positive testing-summary sync
  - Synced `specs/16-testing.md` so the repository-state note and proof-backed-claims guidance now explicitly name the latest RC snapshot theorem inventory, including the zero-count collection/removal and positive-count/target-cell helper theorems.
  - Synced `specs/19-feature-maturity.md` so the verification-baseline clarification now names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly alongside the other RC snapshot lemmas.
- [x] Stage 4.2 heap-characterisation wording sync
  - Synced `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-backed boundary summary now names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory` explicitly alongside the surrounding RC snapshot inventory.
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
- [x] Stage 3.1 concrete-argument specialization fallback
  - `kali_optimize` now clones deterministic generic/function specializations for literal-shaped call sites even when MIR layout metadata is unavailable, so the pure-LIR release path can still monomorphize concrete-argument helpers without waiting for the MIR-aware specialization pass.
- [x] Stage 3.1 MIR-driven specialization follow-up
  - MIR-aware call-site specialization now clones larger functions whose parameter layouts are stable enough to justify partial substitution, then reoptimizes the specialized body so literal-heavy hot paths can fold further before codegen.

### Stage 3.1 - Closure/struct-layout specialization follow-up
- ✅ Shared closure-valued MIR bindings now collapse to one specialization when multiple higher-order call sites share the same layout signature.
- ✅ Shared struct-valued MIR bindings now also collapse to one specialization when multiple higher-order call sites share the same layout signature, and the regression now covers three matching call sites so the reuse shape stays pinned down.
- ✅ The MIR-aware layout signature path now fingerprints struct/array descriptors more precisely and includes closure capture identities, so distinct struct shapes and distinct closure shapes stay distinct without breaking the shared higher-order reuse path.

### Stage 3.1 - Closure-capture identity widening
- ✅ Closure-valued MIR bindings now specialize by their capture identity list instead of only capture arity, so same-capture higher-order call sites can still share a clone while distinct closure shapes no longer collapse onto the same specialization.
- ✅ Added regression coverage for distinct closure capture bindings so the split specialization behavior stays pinned down.
- ✅ Kept the update narrow: this widens specialization depth within the existing optimizer model; it does not change the published benchmark or support claims.

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
  - `KaliCore.Safety.noDanglingReference` is mechanized for the current RC snapshot model, `liveRefsAreOwnedAndAllocated` projects live references back to ownership/allocation, `releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated` / the release-only helper theorem `releaseRefLiveRefsFiltered` and `releaseAndDecrementLiveRefsFiltered` / `releaseAndCollectLiveRefsFiltered` keep the live-reference list as the target-filtered original live set, the helper-level no-dangling-reference corollaries `releaseRefNoDanglingReference` / `releaseAndDecrementNoDanglingReference` / `releaseAndCollectNoDanglingReference` keep the release-path hygiene explicit, `releasePreservesWellFormed` records the live-to-released transition, `releaseAndDecrementPreservesWellFormed` keeps the refcount-decrement update helper honest, `releaseAndDecrementRecorded` / `releaseAndDecrementDecrementsTargetCell` / `releaseAndDecrementKeepsTargetCellWhenPositiveCount` / `releaseAndDecrementHeapCellOrigin` / `releaseAndDecrementHeapCellOriginAndOwnership` / `releaseAndDecrementHeapCellOriginAndPositiveCount` / `releaseAndDecrementKeepsOtherPositiveCountCells, releaseAndDecrementKeepsOriginalPositiveCountCells` / `releaseAndDecrementZeroesLastTargetCell` / `releaseAndCollectRecorded` / `releaseAndCollectDropsZeroCountCells` / `releaseAndCollectRemovesZeroCountCells` / `releaseAndCollectKeepsPositiveCountCells` / `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOrigin`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` / `releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` / `releaseAndCollectDropsOriginalZeroCountCells` / `releaseAndCollectHeapIsPositiveCountFilter` / `releaseAndCollectHeapCellOrigin` / `releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` / `releaseAndCollectHeapCellsHavePositiveCount` / `releaseAndCollectPreservesOtherLiveRefs` / `releaseAndCollectReleasedNotLiveRef` / `releaseAndDecrementReleasedNotLiveRef` / `releaseAndDecrementLiveRefsAreOwnedAndAllocated` / `releaseAndCollectLiveRefsAreOwnedAndAllocated` / `releaseRefReleasedRefsCons` / `releaseAndDecrementReleasedRefsCons` / `releaseAndCollectReleasedRefsCons` / `releaseRefPreservesReleasedRefs` / `releaseAndDecrementPreservesReleasedRefs` / `releaseAndCollectPreservesReleasedRefs` keep the helper's release bookkeeping explicit, and `releasedNotLive` / `releasedNotLiveRef` record the release-path liveness split and live/released disjointness.
  - `KaliIR.HIRModel` records the structural lowering equations for `lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_throw`, and `lower_tr`.
  - `KaliIR.Value` and `KaliIR.LoweringCorrectness.lower_preserves_value` add the current HIR value fragment to the lowering story, `KaliIR.LoweringCorrectness.lower_preserves_step` adds the small-step lowering-preservation bridge for the current HIR subset, including bare throw, and `KaliIR.LoweringCorrectness.lower_preserves_steps` lifts that result to finite traces.
  - `proofs/BOUNDARY.md` now publishes the proof-backed boundary for that slice, and the canonical repository summary is aligned with it.


### Stage 4.2 - RC decrement/zeroing follow-up
- ✅ `releaseAndDecrementHeapCellOrigin` now proves the decrement helper's surviving heap cells still come from the original heap, with only the released target decremented or left unchanged.
- ✅ `releaseAndDecrementKeepsTargetCellWhenPositiveCount` now proves the decrement helper keeps the target cell in the heap when the decremented count stays positive.
- ✅ `releaseAndDecrementZeroesLastTargetCell` now proves the decrement helper zeros the target cell when the released reference was the last live count.

### Stage 4.2 - Decrement target positive-count iff bridge
- ✅ `releaseAndDecrementTargetCellPositiveCountIff` now makes the decrement target's post-update positive-count status explicit as an iff bridge against the original count.
- ✅ Synced `proofs/BOUNDARY.md`, `PLAN-4.2-STATUS.md`, and the verification summaries so the new decrement-path iff theorem is named explicitly alongside the current RC helper inventory.

### Stage 4.2 - Soundness-helper naming sync
- ✅ `proofs/BOUNDARY.md` and `PLAN-4.2-STATUS.md` now explicitly name `KaliCore.Soundness.subst_closed` alongside the widened closed-fragment soundness claims.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a support-rung change.

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
- ✅ `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` now name `KaliCore.Safety.releaseAndCollectHeapCellOrigin` explicitly, so the published verification summaries stay aligned with the helper-level provenance theorem.
- ✅ `releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` now makes the surviving collected heap cells' original name and ownership tag explicit.
- ✅ `releaseAndCollectKeepsOtherHeapEntries` now keeps unrelated positive-count heap entries in the collected heap.
- ✅ `releaseRefPreservesOwnership`, `releaseAndDecrementPreservesOwnership`, and `releaseAndCollectPreservesOwnership` keep the ownership environment unchanged across the release-only, decrement, and collection helpers.


### Stage 4.2 - RC unrelated-heap preservation follow-up
- ✅ `releaseAndCollectKeepsOtherHeapEntries` now keeps unrelated positive-count heap entries in the collected heap, making the helper-level unrelated-heap preservation story explicit.

### Stage 4.2 - Pure release-helper follow-up
- ✅ `releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated` now keeps the pure release helper's surviving live references anchored in ownership and allocation, and `releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` now makes the release-only provenance story explicit.
- ✅ `releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` now names the release-only linear-memory companion theorem directly, so the pure-release provenance slice stays aligned with the decrement and collection helper wording.
- ✅ `PLAN-4.2-STATUS.md` now also names `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseRefHeapCellOrigin` and `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` in the top-level memory-safety summary, so the plan tracker stays as explicit as the published boundary.
- ✅ `releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` now names the release-only combined linear-memory companion theorem directly in the published boundary and summary docs, so the pure-release provenance slice stays aligned with the surrounding helper companions.
- ✅ `releaseRefLiveRefsFiltered`, `releaseAndDecrementLiveRefsFiltered`, and `releaseAndCollectLiveRefsFiltered` now keep the live-reference list filtered to the released target across the release-only, decrement, and collection helpers.
- ✅ `releaseRefReleasedNotLiveRef` now keeps released references disjoint from the live set after the pure release helper runs.
- ✅ `releaseRecorded` still records the released reference in the released set after the pure release step.

### Stage 3.3 - Package corpus breadth expansion
- ✅ Added browser, utility, and Node-runner corpus cases that resolve published exports maps and subpath entrypoints for `react`, `preact`, `vue`, `svelte`, `lit`, `ramda`, `rxjs`, `immer`, `uuid`, `typescript`, `esbuild`, `date-fns`, `lodash-es`, `vitest`, `jest`, and `mocha`, broadening the package-support corpus beyond the original single-entrypoint stubs.
- ✅ Added `vue` to the browser web-baseline interop corpus so the representative browser package coverage now carries one more app-framework name through the browser command path without changing the documented support rungs.
- ✅ Added `mocha` to the Node-runner exports-map and mixed-format corpus slices so the representative test-runner coverage now exercises one more package-shape variant without changing the documented support rungs.
- ✅ Added `@mui/material` and `@floating-ui/react` to the scoped browser exports-map / browser-condition slices so the representative browser corpus now covers one more popular UI package shape without changing the documented support rungs.
- ✅ Added scoped browser conditional-exports coverage so the browser corpus now exercises scoped packages whose `browser` branches win over import/require fallbacks without changing the documented support rungs.
- ✅ Added `commander` to the utility corpus breadth so the plain-package coverage now includes one more common CLI library shape without changing the documented support rungs.
- ✅ Added `lodash` to the utility corpus breadth so the representative common CJS utility package now exercises the plain-package, exports-map, string-exports, pattern-exports, and web-baseline slices without changing the documented support rungs.
- ✅ Added `redux` to the utility corpus and browser web-baseline interop slices so the representative package corpus now carries one more state-management package name through the existing support-rung checks without changing the documented support rungs.
- ✅ Added `recoil` to the utility corpus and browser web-baseline interop slices so the representative package corpus now carries one more state-management package name through the existing support-rung checks without changing the documented support rungs.
- ✅ Added `redux` to the utility exports-map, string-exports, and pattern-exports slices so the representative package corpus now carries one more state-management package shape through the existing support-rung checks without changing the documented support-rungs.
- ✅ Added `./*` exports-pattern corpus coverage so the representative browser and utility package sets now exercise wildcard subpath exports routed through nested `src/` subtrees.
- ✅ Added browser replacement-map coverage so the representative browser package set now exercises exact-path rewrites and `false` blocks after entry selection, alongside the existing exports-map/subpath and browser-conditional-export cases.
- ✅ Added dual-package, browser-conditional-export, mixed-format, browser string-entry, browser false-blocking, module-only, scoped-package, and typed-export-branch corpus coverage so the representative package set now exercises conditional exports across browser/import/require branches plus browser-string overrides, browser-field blocking, and mixed CJS/ESM entrypoints.
- ✅ Added module-only corpus coverage so the representative browser and utility package sets now exercise `package.json#module` fallback resolution as a standalone published shape.
- ✅ Added browser internal-browser-rewrite corpus coverage so the representative browser package set now exercises browser-field rewrites across an internal dependency chain instead of only top-level entrypoint rewrites.
- ✅ Added module-entry internal-dependency corpus coverage so the representative utility package set now exercises internal relative imports inside a module-only package instead of only a single-file module entrypoint.
- ✅ `plan/phase-3/03-ecosystem-breadth.md` now explicitly enumerates the representative browser/utility package-shape cases already covered by the corpus, keeping the implementation playbook aligned with the Stage 3.3 evidence.
- ✅ Added scoped-package corpus coverage so the representative package set now exercises `@scope/name` identities plus the scoped `@types/scope__name` fallback naming convention in both browser-targeted and standalone contexts.
- ✅ Added typed-export-branch corpus coverage so the representative browser package set now exercises `exports` objects that carry `types` conditions alongside the runtime branches, keeping the corpus aligned with common modern package metadata.
- ✅ Added `@floating-ui/react` to the browser typed-export-branch corpus so the representative browser package set now covers one more scoped UI package shape without changing the documented support rungs.
- ✅ Added `@tanstack/react-query` to the browser typed-export-branch corpus so the representative browser package set now covers one more scoped query-library shape without changing the documented support rungs.
- ✅ Added `vue-router` and `react-router` to the browser router corpus slices so the representative browser package set now covers two more router package shapes without changing the documented support rungs.
- ✅ Added `@emotion/react`, `@floating-ui/react`, `@mui/material`, and `@tanstack/react-query` to the browser web-baseline interop slice so the browser/runtime interoperability widening now carries four more scoped browser package names through the browser command path without changing the documented support rungs.
- ✅ Added `@reduxjs/toolkit` to the browser web-baseline interop slice so the browser/runtime interoperability widening now carries one more scoped browser package name through the browser command path without changing the documented support rungs.
- ✅ Added `@emotion/react`, `@emotion/styled`, `@heroicons/react`, and `react-dom` to the browser web-baseline interop slice so the browser/runtime interoperability widening now carries another browser package name through the browser command path without changing the documented support rungs.
- ✅ Added exports-string corpus coverage so the representative browser and utility package sets now exercise top-level string `exports` roots alongside the existing map-based exports cases.
- ✅ Added `dayjs` to the browser web-baseline, utility plain-package, and utility module-entry corpus slices so the breadth follow-up now carries one more common pure-JS utility package through the existing support-rung checks without changing the documented support rungs.
- ✅ Added `zustand` to the browser web-baseline and utility plain-package corpus slices so the representative package corpus now carries one more lightweight package name through the browser and standalone command paths without changing the documented support rungs.
- ✅ Added `xstate` to the browser and utility web-baseline interop slices so the representative package corpus now carries one more state-management package name through the browser and standalone command paths without changing the documented support rungs.
- ✅ Added `valtio` to the browser and utility web-baseline interop slices so the representative package corpus now carries one more state-management package name through the browser and standalone command paths without changing the documented support rungs.
- ✅ Added `superjson` to the browser and utility web-baseline interop slices so the representative package corpus now carries one more lightweight utility package name through the browser and standalone command paths without changing the documented support rungs.
- ✅ Added `@jridgewell/sourcemap-codec` to the browser web-baseline interop slice so the representative package corpus now carries one more utility/source-map package name through the browser command path without changing the documented support rungs.
- ✅ Added `msw` to the browser web-baseline interop slice so the representative browser package corpus now carries one more browser networking package name through the browser command path without changing the documented support rungs.
- ✅ Added `@tanstack/react-form` to the browser web-baseline interop slice so the representative browser package corpus now carries one more scoped form package name through the browser command path without changing the documented support rungs.
- ✅ Added `react-hook-form` to the browser and utility web-baseline interop slices so the representative package corpus now carries one more browser-form package name through both command paths without changing the documented support rungs.
- ✅ Added `immer` to the browser web-baseline interop slice so the representative package corpus now carries one more utility package name through the browser command path without changing the documented support rungs.
- ✅ Added `@storybook/react` to the browser web-baseline interop slice so the representative package corpus now carries one more scoped browser package name through the browser command path without changing the documented support rungs.

### Stage 3.3 - classnames browser/utility corpus widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `classnames` in the browser web-baseline interop slice and the utility corpus, so the representative lightweight package breadth now carries one more pure-JS package through both command paths without changing the documented support rungs.
- ✅ `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name `classnames` explicitly in the Stage 3.3 progress notes, keeping the corpus-breadth wording aligned with the current evidence set.
- ✅ Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

### Stage 3.3 - @tanstack/react-table browser/utility corpus widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@tanstack/react-table` in the browser and utility web-baseline interop slices, so the representative table-package breadth now carries one more scoped package name through both command paths without changing the documented support rungs.
- ✅ `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name `@tanstack/react-table` explicitly in the Stage 3.3 progress notes, keeping the corpus-breadth wording aligned with the current evidence set.
- ✅ Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

### Stage 3.3 - @apollo/client browser corpus widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@apollo/client` in the browser web-baseline interop, typed-export-branch, exports-map, and browser-condition slices, so the representative scoped browser package breadth now carries one more large browser package through the browser command path without changing the documented support rungs.
- ✅ `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name `@apollo/client` explicitly in the Stage 3.3 progress notes, keeping the corpus-breadth wording aligned with the current evidence set.
- ✅ Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

### Stage 3.3 - @chakra-ui/react exports-map/browser-condition widening
- ✅ `crates/kali_cli/tests/package_corpus.rs` now also exercises `@chakra-ui/react` in the scoped browser exports-map and browser-condition slices, so the representative UI package breadth now carries one more browser package shape through those package-resolution paths without changing the documented support rungs.
- ✅ `plan/phase-3/03-ecosystem-breadth.md`, `PLAN.md`, and `TODO.md` now name `@chakra-ui/react` explicitly in the Stage 3.3 progress notes, keeping the corpus-breadth wording aligned with the current evidence set.
- ✅ Kept the update narrow: this is still a corpus / evidence widening within the existing package-support model, not a support-rung change.

### Stage 4.2 - Proof-boundary anti-drift test
- ✅ `crates/kali_cli/tests/schema_docs.rs` now asserts that the `proofs/BOUNDARY.md` covered-path inventory matches the actual `proofs/*.lean` source set, now checks the published theorem inventory against the concrete Lean theorem and lemma names, and now also verifies the canonical proof-summary docs keep the current RC theorem names and proof-backed summary string in sync, so deleting or adding a proof file or drifting summary prose without updating the manifest or docs fails `cargo test`; the progress tracker now calls out that theorem-name inventory and summary-doc inventory check alongside the path-level anti-drift guard.
- ✅ The proof-summary guard now explicitly pins the heap-characterisation theorem names `releaseAndDecrementHeapCharacterisation` and `releaseAndCollectHeapCharacterisation` as well, so the summary docs stay aligned with the published RC snapshot inventory.
- ✅ The proof-summary guard now also pins `releaseAndDecrementTargetCellOriginAndPositiveCount` and `releaseAndCollectHeapCellsHavePositiveCount` explicitly, so the decrement-path provenance/positivity story and the local collection helper's final-heap positivity story stay aligned with the published boundary inventory.

### Stage 4.2 - No-dangling-reference summary sync
- ✅ Synced `PLAN-4.2-STATUS.md` so the current Stage 4.2 status tracker now names `KaliCore.Safety.noDanglingReference` explicitly alongside the existing RC snapshot theorem inventory.
- ✅ Kept the update narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening.

### Stage 4.2 - Lowering value-preservation helper
- ✅ Added the HIR value fragment plus `KaliIR.LoweringCorrectness.lower_preserves_value`, which records that the current core-lifted HIR value forms lower back to core values in the proof model.

### Stage 4.2 - RC origin/positivity conjunction helper
- ✅ Added a reusable `releaseAndCollectHeapCellOriginAndPositiveCount` helper theorem that packages the surviving-cell provenance and positive-count facts for the local collection helper on top of the existing origin and positivity lemmas.
- ✅ Synced the boundary manifest and verification summaries so the new helper theorem is named explicitly alongside the rest of the RC snapshot slice.

### Stage 4.2 - RC target-allocation follow-up
- ✅ `releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, `releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `releaseAndCollectTargetCellOriginOwnershipAndPositiveCount`, and `releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` now make the surviving target-cell allocation bridge explicit on the decrement and collection helpers when the count stays positive.
- ✅ Synced `PLAN-4.2-STATUS.md` and the proof-backed verification summaries so the progress trackers name the target-allocation corollaries alongside the existing RC helper slice.

### Stage 4.2 - Linear-memory payload preservation
- ✅ The proof-backed RC snapshot model now carries an explicit linear-memory payload alongside the existing ownership / heap / live-reference state.
- ✅ The release-only, decrement, and collection helpers preserve that payload via `KaliCore.Safety.releaseRefPreservesLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesLinearMemory`.
- ✅ The same helpers now also package ownership and linear-memory preservation together via `KaliCore.Safety.releaseRefPreservesOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesOwnershipAndLinearMemory`, and the local collection helper now also packages the heap-filter characterization with the linear-memory payload via `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`.
- ✅ The same helpers now also package wellformedness and linear-memory preservation together via `KaliCore.Safety.releaseRefPreservesWellFormedAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndLinearMemory`.
- ✅ The same helpers now also package wellformedness, ownership, and linear-memory preservation together via `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`.

### Stage 4.2 - Decrement origin/ownership/positivity linear-memory companion
- ✅ `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` now packages the decrement helper's surviving-cell provenance, ownership tag, positive-count fact, and linear-memory payload together.
- ✅ `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and the proof-summary anti-drift guard now name the companion theorem explicitly.
- ✅ Kept the update narrow: this is another helper-level proof-summary sync for the published boundary, not a broader ownership/freeing widening.

### Stage 4.2 - Decrement target origin/positive-count wording sync
- ✅ `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` is now spelled out explicitly in the published boundary, summary docs, and proof-summary guard, so the decrement-path provenance/positivity slice stays aligned with the RC snapshot inventory.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a boundary widening.

### Stage 4.2 - Final-heap positive-count wording sync
- ✅ `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` is now spelled out explicitly in the published boundary, summary docs, and Stage 4.2 progress notes, so the local collection helper's final-heap positivity story stays direct rather than implied.
- ✅ Kept the update narrow: this is a proof-summary wording sync for the published boundary, not a boundary widening.
