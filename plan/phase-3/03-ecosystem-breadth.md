# Stage 3.3 — Ecosystem Breadth

**Phase:** 3 — Specialisation, Optimisation & Ecosystem Breadth  
**Spec refs:** [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [3.1 — Specialisation & Optimisation](01-specialisation-and-optimisation.md) (for layout-specialised package builds), [3.2 — Node Compatibility](02-node-compatibility.md) (for Node-assuming package corpus)

## Goal

Expand the package corpus coverage, browser packaging/interoperability beyond the Phase-1 bundle
baseline, and open the higher-cost cross-module constraint solving for public APIs. This stage
deepens all three Phase-3 breadth areas without introducing new hard invariant risks.

## Workable Milestone

- The curated npm/JSR package corpus passes at a significantly higher rate than Phase 1.
- Browser bundle output includes richer code-splitting and interoperability options.
- Cross-module constraint solving is available and evidence-backed.

## Tasks

### 1. Package corpus expansion

Extend the curated package corpus to include:

- React/Preact/Vue (browser context: **checkable** + **deployable-through-host** via `--bundle`).
- `typescript`, `esbuild` (as pure-JS tools invoked via Kali's API).
- `vitest`, `jest` (test runner libraries — in `--api node` context).
- Utility libraries: `ramda`, `rxjs`, `immer`, `uuid`.

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

## Out of Scope

- Full Web API compatibility (Later compatibility).
- `eval` / `Function()` (Phase 4 target).
- `kali package-audit` public availability (Later compatibility; preview only here).
- Threading / `Worker` full support (Later compatibility).

## Definition of Done

- [ ] Package corpus expanded; all newly added packages pass at their documented support rung.
- [ ] Code-splitting tests produce multiple bundle chunks from dynamic `import()` boundaries.
- [ ] Tree-shaking tests confirm unused exports are absent from emitted bundles.
- [x] Cross-module inference tests pass within the Phase-3 solver budget.
- [x] `kali package-audit` available as a `--preview` opt-in; final public availability deferred
  to Phase 4.
- [ ] All Phase-1 and Phase-2 tests continue to pass without regression.
