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
- The shared browser/runtime support library now also exposes a deterministic `navigator`
  metadata baseline, and the web-baseline corpus now exercises `navigator.userAgent`,
  `navigator.language`, and `navigator.onLine` so the ambient browser metadata slice stays
  deterministic without changing the support-rung story.
- The shared browser/runtime support library now also exposes a `random_uuid` helper for
  `crypto.randomUUID()`-style calls, plus a deterministic `Crypto` facade for the shared
  `crypto.getRandomValues()` / `crypto.randomUUID()` randomness subset; `kali_runtime` wires the
  matching `crypto_random_uuid` / `cryptoRandomUUID` host imports through that helper, and the
  web-baseline corpus now exercises that path so the browser UUID slice stays covered without
  changing the support-rung story.
- The same web-baseline corpus now also exercises `performance.now()` and `queueMicrotask`
  through the shared helper source so the timing and microtask primitives stay covered without
  changing the support-rung story.
- The curated package corpus now covers browser, utility, Node-runner, and scoped-package classes
  across representative real-world package shapes: exports-string roots, exports-map / subpath and
  `./*` pattern entries, browser-conditional exports, browser replacement maps, browser string
  overrides, browser false-blocking, browser internal-browser-rewrite chains, module-only and
  module-entry internal-dependency chains, dual-package and mixed-format entrypoints, and typed
  export branches.
- The Node-runner corpus now also exercises `mocha` and `ava` in the exports-map and mixed-format
  slices, so the test-runner breadth now covers one more representative package shape without
  changing the documented support rungs.
- The Node-assuming corpus now also exercises `dotenv` under the Node context, so one more common
  Node-only package shape stays concrete without changing the documented support rungs.
- The scoped browser corpus now also exercises `@mui/material`, `@floating-ui/react`, and `@heroicons/react` in the exports-map and browser-condition slices, adding three more representative UI package shapes to the evidence set without changing the documented support rungs.
- The scoped browser corpus now also exercises `@headlessui/react` across the web-baseline interop,
  exports-map, and browser-condition slices, adding one more representative UI package shape to the
  evidence set without changing the documented support rungs.
- The browser corpus now also exercises `react-dom` across the exports-map and browser-condition
  slices, and the scoped browser exports-map slice now also carries it, keeping the browser
  package-shape breadth concrete without changing the support-rung story.
- The browser corpus now also exercises `jotai` across the exports-map and browser-condition slices,
  keeping one more representative browser state-management package shape covered without
  changing the support-rung story.
- The scoped browser corpus now also exercises `@chakra-ui/react` across the exports-map and
  browser-condition slices, keeping one more representative UI package shape covered without
  changing the support-rung story.
- The scoped browser corpus now also exercises `@mantine/core` across the exports-map and
  browser-condition slices, keeping one more representative UI package shape covered without
  changing the support-rung story.
- The scoped browser corpus now also exercises `@radix-ui/react-dialog` across the exports-map and
  browser-condition slices, keeping one more representative dialog package shape covered without
  changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `vue` as another representative
  app-framework package name, keeping the browser package corpus breadth concrete without changing
  the support-rung story.
- The browser web-baseline interop corpus now also exercises `@chakra-ui/react`, `@mantine/core`,
  `@emotion/styled`, `@heroicons/react`, `lucide-react`, `@radix-ui/react-dialog`, and `react-dom` as more representative browser package
  names, keeping the browser package corpus breadth concrete without changing the support-rung
  story.
- The browser web-baseline interop corpus now also exercises `next` as one more browser
  app-framework package, `framer-motion` as one more representative browser UI package,
  `chart.js`, `recharts`, and `d3` as more representative browser charting packages, and
  `@storybook/react` as one more representative scoped browser package; the scoped browser corpus
  now also exercises `@storybook/react` across the exports-map and browser-condition slices,
  and the utility plain-package corpus now also carries `d3` on the default standalone surface too,
  keeping the browser package corpus breadth concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@remix-run/react` as another
  representative browser app-framework package name, and the scoped browser corpus now also
  exercises it across the exports-map and browser-condition slices, keeping the browser package
  breadth concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `react-helmet-async` as another
  representative browser head-management package name, keeping one more browser package shape
  concrete through the browser command path without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@stripe/react-stripe-js` as another
  representative browser payment/UI package name, and the scoped browser corpus now also exercises
  it across the exports-map and browser-condition slices, keeping one more browser package shape
  concrete through both the browser command path and the scoped browser shape coverage without
  changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `ajv` as another representative
  validation package name, and the utility plain-package corpus now also carries it on the default
  standalone surface, keeping one more common pure-JS package covered without changing the
  support-rung story.
- The scoped browser exports-map slice now also exercises `@remix-run/react`, so that app-framework
  package is now covered through the exports-map resolution path in addition to the existing
  browser-condition slice.
- The browser corpus now also exercises `rxjs`, and the browser web-baseline interop corpus now also
  exercises it too as another representative utility package name, keeping one more browser/utility
  package name covered without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@playwright/test` as one more representative browser test-runner package, keeping the browser package corpus breadth concrete without changing the support-rung story.
- The utility corpus now also exercises `vite` in the plain-package and web-baseline interop slices,
  and now also carries `lucide-react` in the plain-package slice, keeping one more modern build-tool / browser UI package name covered without changing the support-rung story.
- The utility corpus now also exercises `luxon` in the browser web-baseline, utility plain-package,
  utility web-baseline interop, and utility module-entry slices, broadening the representative
  date-time utility set without changing the support-rung story.
- The browser corpus now also exercises `path-to-regexp`, and the utility corpus now also carries
  `path-to-regexp` in the plain-package and web-baseline interop slices, keeping one more routing
  package name concrete across the browser and standalone package paths without changing the
  support-rung story.
- The utility corpus now also exercises `react` and `preact` in the plain-package slice on the
  default standalone surface, keeping the representative React/Preact package breadth concrete
  without changing the support-rung story.
- The browser web-baseline interop corpus, utility plain-package corpus, and utility module-entry
  corpus now also exercise `rambda` as another representative functional-utility package name,
  keeping the breadth widening concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `yaml`, and the utility plain-package / web-baseline interop corpus now also carries it on the standalone surface, keeping the representative pure-JS data-format package breadth concrete without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `@tanstack/router` as another
  representative scoped routing package name, keeping the browser/utility package breadth concrete
  without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@tanstack/react-router` as another
  representative scoped routing package name, and the scoped browser corpus now also carries it
  across the exports-map and browser-condition slices, keeping one more routing package shape
  concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@tanstack/table-core`, and the scoped
  browser corpus now also exercises it across the exports-map and browser-condition slices, keeping
  one more representative scoped table package name covered without changing the support-rung story.
- The scoped browser corpus now also exercises `@tanstack/router` across the exports-map and
  browser-condition slices, keeping one more representative scoped routing package name covered
  without changing the support-rung story.
- The scoped browser corpus now also exercises `zustand` across the exports-map and browser-condition
  slices, keeping one more representative state-management package name covered without changing
  the support-rung story.
- The browser web-baseline interop corpus now also exercises `@testing-library/react`,
  `@testing-library/dom`, and `@testing-library/user-event` as more representative scoped
  testing-library package names, keeping the browser package corpus breadth concrete without
  changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `mobx` and `classnames` as
  representative browser state-management / lightweight package names, keeping the browser package
  corpus breadth concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `redux` as another representative
  browser state-management package name, keeping the browser package corpus breadth concrete
  without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `recoil` as another
  representative browser state-management package name, keeping the breadth widening concrete
  without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `mitt` as another
  representative lightweight package name, keeping the browser and utility package breadth concrete
  without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `swr` as another
  representative browser/utility package name, keeping the breadth widening concrete without
  changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `formik` and `jotai` as
  another representative browser form / state-management package pair, keeping the browser and
  utility package breadth concrete without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `nanostores` as another
  representative browser/utility package name, keeping the breadth widening concrete without
  changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `pinia`, `xstate`, and
  `valtio` as additional representative Vue/state-management browser/utility package names,
  keeping the breadth widening concrete without changing the support-rung story.
- The browser and utility exports-map and pattern-exports corpus now also exercises `xstate` as
  another representative state-management package shape, keeping the shape coverage concrete
  without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `superjson` as another
  representative lightweight utility package name, and the utility corpus now also carries
  `chart.js`, `recharts`, `@emotion/react`, and `@emotion/styled` on the default standalone surface, keeping the breadth widening
  concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@jridgewell/sourcemap-codec` as another
  representative utility/source-map package name, keeping the browser package corpus breadth
  concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@babel/runtime` and
  `@npmcli/package-json` as another representative scoped utility package name pair, keeping one
  more browser-facing utility package shape concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@tanstack/query-core` alongside the
  utility plain-package slice, keeping one more representative scoped query package name concrete
  without changing the support-rung story.
- The scoped browser corpus now also exercises `@tanstack/query-core` across the exports-map and
  browser-condition slices, so one more representative scoped query package shape stays concrete
  without changing the support-rung story.
- The utility web-baseline interop corpus now also exercises `recharts` as another representative browser charting package name, and now also exercises `@emotion/styled` as another representative scoped UI package name, keeping the browser-style charting surface concrete without changing the support-rung story.
- The utility web-baseline interop corpus now also exercises `@storybook/react` as another representative scoped browser package name, keeping the browser-style package breadth concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `query-string`, and the utility plain-package corpus now also carries it on the default standalone surface, so one more query-string package name stays concrete without changing the support-rung story.
- The utility plain-package corpus now also exercises `cheerio` on the default standalone surface, so one more common DOM-parsing package name stays concrete without changing the support-rung story.
- The browser web-baseline interop corpus, utility plain-package corpus, and utility web-baseline interop corpus now also exercise `yup` as another representative validation-library package name, keeping one more common JS package name concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `graphql`, and the utility plain-package corpus now also carries it on the default standalone surface, keeping one more common JS package name covered without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `msw` as another representative
  browser networking package name, keeping the browser package corpus breadth concrete without
  changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@tanstack/react-form` as another
  representative scoped browser form package name, keeping the browser package corpus breadth
  concrete without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `react-hook-form` and
  `classnames` as representative browser-form / lightweight package names, keeping the breadth
  widening concrete without changing the support-rung story.
- The utility corpus now also exercises `msw` in the plain-package slice on the default standalone
  surface, keeping one more browser-networking package name covered without changing the support-
  rung story.
- The browser and utility web-baseline interop corpus now also exercises `@tanstack/react-table` as
  another representative scoped table package name, keeping the breadth widening concrete without
  changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@apollo/client` across the web-baseline,
  typed-export-branch, exports-map, and browser-condition slices, keeping the representative scoped
  browser package breadth concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `immer` as another representative
  utility package name, keeping the browser package corpus breadth concrete without changing the
  support-rung story.
- The utility corpus now also exercises `redux` across the exports-map, string-exports, and
  pattern-exports slices, keeping the state-management package breadth concrete without changing the
  documented support-rungs.
- The utility corpus now also exercises `reselect` across the exports-map and pattern-exports
  slices as another representative state-management package name, keeping the breadth widening
  concrete without changing the documented support-rungs.
- The utility scoped-package corpus now also exercises `@babel/runtime`, `@npmcli/package-json`,
  and `@reduxjs/toolkit` on the default standalone surface, keeping one more representative scoped
  utility package shape covered without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `hono`, `@vueuse/core`, `TextEncoder`, and
  `TextDecoder` as more representative browser/web-framework, browser utility, and text-codec
  primitives, keeping the browser package corpus breadth concrete without changing the support-rung
  story.
- The scoped browser corpus now also exercises `@vueuse/core` across the exports-map and
  browser-condition slices, keeping one more representative scoped browser utility package shape
  covered without changing the support-rung story.
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
- The shared browser/runtime support library now also exposes an in-memory `FormData` baseline,
  and the Deno compatibility surface reexports it so browser-style code can model deterministic
  multipart payloads without changing the documented support rungs.
- The shared browser/runtime support library now also exposes deterministic stub baselines for
  `BroadcastChannel`, `WebSocket`, `Worker`, and `IndexedDB`, and the Deno compatibility surface
  reexports those names so browser-style code can exercise the ambient surface without changing the
  documented support rungs.
- The shared browser/runtime support library now also exposes an `IndexedDB` alias for the in-memory
  stub, and the Deno compatibility surface reexports that browser-aligned name too so Rust-facing
  code can mirror the docs while the lower-case `indexedDB` global stays the corpus source of
  truth.
- The shared browser/runtime support library now also exposes deterministic `atob` / `btoa`
  helpers, and the Deno compatibility surface reexports them so browser-style code can round-trip
  binary strings without changing the documented support rungs.
- The browser and utility web-baseline interop corpus now also exercises `atob` / `btoa` in the
  shared helper source, keeping the browser/runtime interoperability slice concrete without
  changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `localStorage` /
  `sessionStorage` in the shared helper source, keeping the browser/runtime state baseline
  concrete without changing the support-rung story.
- The shared browser/runtime support library now also exposes a deterministic `URL` baseline, the
  browser and utility web-baseline interop corpus now also exercises `new URL(...)` in the shared
  helper source, and the Deno compatibility surface reexports the wrapper so browser-style code can
  model parsed URL state without changing the documented support rungs.
- The package corpus now also exercises `AbortController`, `EventTarget`, `CustomEvent`,
  `BroadcastChannel`, `URLSearchParams`, `FormData`, `fetch`, `Headers`, `Request`, `Response`, `structuredClone`, and `FileReader` in representative browser and
  utility package cases, widening the browser/runtime interoperability slice without changing the
  documented support rungs.
- The shared browser/runtime support library now also supports deterministic `EventTarget` listener
  removal, giving the browser interop slice a more faithful listener lifecycle baseline without
  changing the documented support rungs.
- The shared browser/runtime support library now also dispatches a deterministic `abort` event from
  `AbortController`, so the browser baseline exposes a more realistic abort lifecycle without
  changing the documented support rungs.
- The browser and utility corpus now also drive the deterministic `BroadcastChannel`, `WebSocket`, `Worker`, and
  `IndexedDB` browser-runtime stubs through the existing web-baseline interop slice, keeping the
  interop widening concrete without changing the support-rung story.
- The browser corpus now additionally exercises `solid-js` in the web-baseline interop slice alongside
  the existing browser representatives in the exports-map slice, keeping the interoperability
  widening concrete without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@radix-ui/react-dialog` as one more
  representative scoped browser package name, keeping the browser package corpus breadth concrete
  without changing the documented support rungs.
- The browser web-baseline interop corpus now also exercises `date-fns`, `lodash-es`, `nanoid`, `ramda`, and
  `uuid` alongside the existing browser representatives, keeping five more utility-package names
  covered by the browser command path without changing the support-rung story.
- The browser and utility web-baseline interop corpus now also exercises `zod` as one more
  representative package name, keeping the breadth widening concrete without changing the documented
  support rungs.
- The browser web-baseline interop corpus now also exercises `svelte` and `lit` alongside the
  existing browser representatives, keeping two more browser-oriented package names covered by the
  browser command path without changing the support-rung story.
- The browser web-baseline interop corpus now also exercises `@emotion/react`, `@reduxjs/toolkit`, `@floating-ui/react`, `@mui/material`,
  `@radix-ui/react-dialog`, and `@tanstack/react-query` alongside the existing browser
  representatives, keeping six more scoped browser package names covered by the browser command
  path without changing the support-rung story.
- The browser corpus now also exercises `dayjs` across the exports-map and browser-condition slices,
  keeping one more date-time package shape concrete through the browser command path without
  changing the support-rung story.
- The browser router corpus now also exercises `vue-router` and `react-router` across the
  web-baseline interop, exports-map, and pattern-exports slices, and now also exercises
  `react-router-dom` across the browser exports-map and pattern-exports slices, adding three more
  representative browser package shapes without changing the documented support rungs.
- The browser exports-map and pattern-exports corpora now also exercise `hono`, widening the
  representative browser shape coverage by one more framework package without changing the
  documented support rungs.
- The browser pattern-exports corpus now also exercises `solid-js`, broadening the representative
  browser package-shape coverage without changing the documented support rungs.
- The browser typed-export-branch corpus now also exercises `@reduxjs/toolkit`, broadening the
  representative scoped browser package coverage without changing the documented support rungs.
- The browser typed-export-branch corpus now also exercises `@emotion/styled` alongside the existing
  scoped browser representatives, adding one more typed-export branch shape to the evidence set
  without changing the documented support rungs.
- The browser typed-export-branch corpus now also exercises `@floating-ui/react` and `@tanstack/react-query`, broadening the
  representative scoped browser package coverage without changing the documented support rungs.
- The utility corpus now additionally exercises `nanoid` and `rxjs` in the web-baseline interop slice alongside
  the existing utility representatives, widening the plain-package breadth without changing the
  documented support rungs.
- The utility corpus also now includes `dayjs` in the web-baseline, plain-package, and module-entry
  slices, broadening the representative utility set without changing the documented support rungs.
- The utility corpus also now includes `axios` in the plain-package slice on the default standalone
  surface, keeping one more common pure-JS package covered without changing the documented support
  rungs.
- The utility corpus also now includes `deepmerge` in the plain-package slice on the default
  standalone surface, keeping one more common pure-JS package covered without changing the documented
  support rungs.
- The utility corpus also now includes `commander` alongside the existing utility representatives,
  widening the plain-package breadth without changing the documented support rungs.
- The utility corpus now also exercises `redux` as another representative state-management package
  name across the plain-package and web-baseline interop slices, keeping the breadth widening
  concrete without changing the support-rung story.
- The utility corpus now also exercises `lodash` across the plain-package, exports-map,
  string-exports, pattern-exports, and web-baseline slices, keeping one more common CJS utility
  package covered without changing the support-rung story.
- The browser web-baseline corpus and the utility plain-package corpus now also exercise `zustand` as
  another representative lightweight package name, keeping the breadth widening concrete without
  changing the support-rung story.
- The browser and utility corpus now also exercise `clsx` as another representative lightweight
  package name, keeping the breadth widening concrete without changing the support-rung story.
- The utility module-entry and mixed-format slices now also exercise `immer`, `typescript`, and
  `esbuild`, so the representative package corpus now spans a few more tooling-style package names
  across the stable shape tests without changing the support-rung story.
- Phase-3 cross-module inference smoke coverage now exercises a multi-file import chain with
  inferred public API types within the solver budget, and the latest smoke also keeps that chain
  honest under an explicit `compilerOptions.maxSpecializations = 1` cap while flowing through an
  object-returning helper so the public-type inference story stays concrete.
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

- keep widening the representative package corpus without overclaiming support rungs; the current corpus now also carries `superjson`, `chart.js`, `recharts`, `@jridgewell/sourcemap-codec`, and `@emotion/react` through the default standalone surface alongside the existing browser/utility breadth, and the utility plain-package corpus now also carries `tailwindcss` on the default standalone surface as another build-tool package name, and the shared web-baseline helper source now also exercises `performance.now()` and `queueMicrotask` so the browser/runtime timing and microtask baseline stays concrete,
- deepen browser interoperability beyond the current bundle/chunk/source-map/tree-shaking slice and the current in-memory Blob/File/FileReader/storage baseline plus the stub WebSocket/Worker/IndexedDB surface,
- widen cross-module inference carefully within the published solver/specialization budgets, including higher-order helper chains that return function values across the re-export path.

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
