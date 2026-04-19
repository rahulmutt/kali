# Stage 3.3 — Ecosystem Breadth

**Phase:** 3 — Specialisation, Optimisation & Ecosystem Breadth  
**Spec refs:** [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [3.1 — Optimization & Specialization](01-optimization-and-specialization.md) (for layout-specialized package builds), [3.2 — Node Compatibility](02-node-compatibility.md) (for Node-assuming package corpus)

## Goal

Expand the package corpus coverage, browser packaging/interoperability beyond the Phase-1 bundle
baseline, and open the higher-cost cross-module constraint solving for public APIs. This stage
deepens all three Phase-3 breadth areas without introducing new hard invariant risks.

## Workable Milestone

- The curated npm/JSR package corpus passes at a significantly higher rate than Phase 1.
- Browser bundle output includes richer code-splitting and interoperability options.
- Cross-module constraint solving is available and evidence-backed.

## Progress

- Browser bundle output now emits deterministic source-map companions, supports both ESM and CJS
  wrappers, emits deterministic chunk artifacts for literal dynamic-import boundaries, and
  tree-shakes unused exports from the emitted bundle surface.
- The curated package corpus now covers browser, utility, Node-runner, and scoped-package classes
  across representative real-world package shapes: exports-string roots, exports-map / subpath and
  `./*` pattern entries, browser-conditional exports, browser replacement maps, browser string
  overrides, browser false-blocking, browser internal-browser-rewrite chains, module-only and
  module-entry internal-dependency chains, dual-package and mixed-format entrypoints, and typed
  export branches.
- The Node-runner corpus now also exercises `mocha` in the exports-map and mixed-format slices,
  so the test-runner breadth now covers one more representative package shape without changing the
  documented support rungs.
- The scoped browser corpus now also exercises `@mui/material` in the exports-map and browser-
  condition slices, adding one more representative UI package shape to the evidence set without
  changing the documented support rungs.
- Scoped browser conditional-exports coverage now exercises packages whose browser branch wins over
  import/require fallbacks, tightening the browser/runtime interoperability slice without changing
  the package-support rungs claimed for the corpus.
- The shared browser/runtime support library now also carries in-memory `Blob` and `File` primitives,
  which gives the browser interoperability slice a more realistic payload baseline without changing
  the package-support rungs claimed for the corpus.
- The shared browser/runtime support library now also carries deterministic in-memory `localStorage`
  and `sessionStorage` buckets, and the Deno compatibility surface reexports those storage helpers,
  giving the browser interoperability slice another browser-state baseline without changing the
  package-support rungs claimed for the corpus.
- The shared browser/runtime support library now also exposes an in-memory `FileReader` baseline,
  and the Deno compatibility surface reexports it so browser-style code can read the shared blob /
  file payloads deterministically without changing the documented support rungs.
- The shared browser/runtime support library now also exposes deterministic stub baselines for
  `WebSocket`, `Worker`, and `IndexedDB`, and the Deno compatibility surface reexports those names
  so browser-style code can exercise the ambient surface without changing the documented support
  rungs.
- The package corpus now also exercises `AbortController`, `EventTarget`, `structuredClone`, and
  `FileReader` in representative browser and utility package cases, widening the browser/runtime
  interoperability slice without changing the documented support rungs.
- The browser and utility corpus now also drive the deterministic `WebSocket`, `Worker`, and
  `IndexedDB` browser-runtime stubs through the existing web-baseline interop slice, keeping the
  interop widening concrete without changing the support-rung story.
- The browser corpus now additionally exercises `solid-js` in the web-baseline interop slice alongside
  the existing browser representatives in the exports-map slice, keeping the interoperability
  widening concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `date-fns` and `lodash-es` alongside the
  existing browser representatives, keeping two more utility-package names covered by the browser
  command path without changing the support-rung story.
- The browser corpus now also exercises `vue-router` in the exports-map and pattern-exports slices,
  adding one more representative browser package shape without changing the documented support
  rungs.
- The browser corpus now also exercises `react-router` in the web-baseline interop, exports-map,
  and pattern-exports slices, adding one more browser-router representative without changing the
  documented support rungs.
- The browser pattern-exports corpus now also exercises `solid-js`, broadening the representative
  browser package-shape coverage without changing the documented support rungs.
- The browser typed-export-branch corpus now also exercises `@reduxjs/toolkit`, broadening the
  representative scoped browser package coverage without changing the documented support rungs.
- The browser typed-export-branch corpus now also exercises `@tanstack/react-query`, broadening the
  representative scoped browser package coverage without changing the documented support rungs.
- The utility corpus now additionally exercises `rxjs` in the web-baseline interop slice alongside
  the existing utility representatives, widening the plain-package breadth without changing the
  documented support rungs.
- The utility corpus also now includes `dayjs` in the web-baseline, plain-package, and module-entry
  slices, broadening the representative utility set without changing the documented support rungs.
- The utility corpus also now includes `commander` alongside the existing utility representatives,
  widening the plain-package breadth without changing the documented support rungs.
- The browser web-baseline corpus and the utility plain-package corpus now also exercise `zustand` as
  another representative lightweight package name, keeping the breadth widening concrete without
  changing the support-rung story.
- The browser and utility corpus now also exercise `clsx` as another representative lightweight
  package name, keeping the breadth widening concrete without changing the support-rung story.
- The utility module-entry and mixed-format slices now also exercise `immer`, `typescript`, and
  `esbuild`, so the representative package corpus now spans a few more tooling-style package names
  across the stable shape tests without changing the support-rung story.
- Phase-3 cross-module inference smoke coverage now exercises a multi-file import chain with
  inferred public API types within the solver budget.
- Historical note: this stage originally introduced `kali package-audit` as a Phase-3 preview.
  The current repository has since advanced beyond that: Stage 4.1 made `kali package-audit`
  publicly available, so the preview-only note below is now a historical stage constraint rather
  than the current repo-level availability state.

## Tasks

### 1. Package corpus expansion

Extend the curated package corpus to include:

- React/Preact/Vue (browser context: **checkable** + **deployable-through-host** via `--bundle`).
- `typescript`, `esbuild` (as pure-JS tools invoked via Kali's API).
- `vitest`, `jest` (test runner libraries — in `--api node` context).
- Utility libraries: `ramda`, `rxjs`, `immer`, `uuid`.
- Representative package-shape cases that exercise the current browser and utility corpus breadth: exports-string roots, exports-map / subpath entries, `./*` exports-patterns, browser-conditional exports, browser replacement maps, browser string-entry overrides, browser false-blocking, browser internal-browser-rewrite chains, module-only entries, module-entry internal dependency chains, dual-package shapes, mixed-format packages, scoped packages, and typed-export-branch variants.

For each package, run through the full **package-support decision order** and document the exact
rung achieved (`checkable`, `buildable`, `executable`, `deployable-through-host`).

### 2. Browser packaging improvements

Extend `kali build --bundle` beyond the Phase-1 single-file output:

- **Code splitting**: split the bundle at `import()` boundaries (for literal-string `import()`
  which was parsed but gated in Phase 1; Phase 3 allows lowering to async-loaded sub-modules
  within the already-linked graph).
- **Tree shaking**: whole-program DCE removes unused exports from the bundle.
- **Source maps**: emit `.js.map` alongside the bundle JS glue for browser DevTools.
- **ESM and CJS targets**: `--bundle --format esm` (default) and `--bundle --format cjs` for
  broader bundler interoperability.

### 3. Broader browser API surface

Expand `kali_api_web` to cover more of the Web API baseline:

- `localStorage` / `sessionStorage` (in-memory simulation for WASM context).
- `WebSocket` (via a Rust WebSocket client).
- `Worker` (stub; full support is later due to threading).
- `IndexedDB` (stub; full support is later).
- `Blob`, `File`, `FileReader`.
- `CustomEvent`, `EventTarget`.

### 4. Open-ended cross-module constraint solving

Now that Phase-3 solver work has an explicit compile-time budget (from Stage 3.1), open the
constraint-solving capability for public API inference:

- Lift the **annotation-required inference boundary** for functions that are fully analyzable
  across module boundaries.
- Gate depth with `compilerOptions.maxSpecializations` and the solver budget.
- Document which inference cases improve and which still require explicit annotations.

### 5. `kali package-audit` planning

While `kali package-audit` remains **Later compatibility**, use this stage to:

- Finalise the schema for `schemas/result/package-audit/v1.json`.
- Implement the command as a Phase-3 opt-in preview (marked unavailable by default; behind an
  explicit `--preview` flag).
- This ensures the stable vocabulary is correct before it opens publicly.

### 6. Tests

- Package corpus expansion tests: all newly added packages pass at their documented rung.
- Code-splitting tests: a fixture with `import()` boundaries produces multiple bundle chunks.
- Tree-shaking tests: unused exports do not appear in the emitted bundle.
- Cross-module inference tests: public API types inferred without annotation in Phase-3 cases.
- All Phase-1 and Phase-2 tests continue to pass.

## Remaining Work

Stage 3.3 is complete. The remaining work in this area is breadth/depth follow-up rather than
unfinished Stage-3.3 implementation:

- keep widening the representative package corpus without overclaiming support rungs,
- deepen browser interoperability beyond the current bundle/chunk/source-map/tree-shaking slice and the current in-memory Blob/File/FileReader/storage baseline plus the stub WebSocket/Worker/IndexedDB surface,
- widen cross-module inference carefully within the published solver/specialization budgets.

## Historical Out of Scope for Stage 3.3

- Full Web API compatibility.
- `eval` / `Function()` (delivered later in Phase 4 under explicit compatibility gating).
- `kali package-audit` public availability (delivered later in Stage 4.1).
- Threading / `Worker` full support.

## Definition of Done

- [x] Package corpus expanded; all newly added packages pass at their documented support rung.
- [x] Code-splitting tests produce multiple bundle chunks from dynamic `import()` boundaries.
- [x] Tree-shaking tests confirm unused exports are absent from emitted bundles.
- [x] Cross-module inference tests pass within the Phase-3 solver budget.
- [x] `kali package-audit` reached the Stage-3.3 preview milestone that later allowed Stage 4.1 to open public availability.
- [x] All Phase-1 and Phase-2 tests continue to pass without regression.
