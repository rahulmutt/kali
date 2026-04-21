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
- The shared browser/runtime support library now covers a broader deterministic web baseline,
  including `navigator` metadata, `performance.now()`, `queueMicrotask`, `URL`, `atob`/`btoa`,
  `Blob`, `File`, `FileReader`, `FormData`, `localStorage`, `sessionStorage`, event primitives,
  `AbortController`, `BroadcastChannel`, and stubbed `WebSocket` / `Worker` / `IndexedDB` paths.
- The curated package corpus now spans representative browser, utility, scoped-package,
  Node-runner, and Node-assuming shapes across exports roots/maps/patterns, browser-condition and
  browser-rewrite cases, dual/mixed-format packages, typed-export branches, and module-entry
  internal dependency chains.
- Evidence breadth expanded substantially across modern browser/framework/state-management/tooling
  packages, while keeping every claim tied to documented support rungs rather than broad package
  compatibility wording.
- Cross-module inference smoke coverage now includes higher-order multi-file import chains within
  the solver budget, including the `factory` → `helper` → `bridge` → `public` → `main` path where
  the public-facing API returns another function.
- Historical note: this stage originally introduced `kali package-audit` as a Phase-3 preview;
  Stage 4.1 later opened it as a public command, so that preview wording is now historical only.

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

## Status

Stage 3.3 is complete.

Forward-looking package/browser/inference widening is already tracked in the owning spec chapters
instead of this completed stage document:
- [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md) for browser and host-surface breadth,
- [`specs/14-packages.md`](../../specs/14-packages.md) for package-compatibility rules and support-rung discipline, and
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md) for exact availability boundaries.

This file remains the historical implementation playbook for the Phase-3 ecosystem-breadth
milestone rather than an open-ended corpus wishlist.
