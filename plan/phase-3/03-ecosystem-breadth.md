# Stage 3.3 — Ecosystem Breadth

**Phase:** 3 — Specialisation, Optimisation & Ecosystem Breadth  
**Spec refs:** [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/07-specialization.md`](../../specs/07-specialization.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [3.1 — Optimization & Specialization](01-optimization-and-specialization.md), [3.2 — Node Compatibility](02-node-compatibility.md), and whichever pieces of [3.4 — Host Capability Expansion](04-host-capability-expansion.md) a given package/browser corpus lane needs

## Goal

Expand the package corpus, browser packaging/interoperability, and cross-module inference breadth
without weakening the hard invariants or blurring the package-support ladder.

## Workable Milestone

- The curated package corpus passes at a meaningfully higher rate, with each result tied to an
  explicit support rung.
- Browser bundle output deepens beyond the Phase-1 baseline while staying inside the linked-artifact
  model.
- Literal-string `import()` lowering and evidence-backed cross-module inference are available for
  the documented Phase-3 cases.

## Progress

**Status:** Complete for the documented Phase-3 breadth milestone.

- Added a semver package-corpus regression so the Phase-3 breadth lane now includes a real pure-JS package on the default standalone library-consumption path, with check/build/run evidence for the canonical `valid` / `satisfies` / `minVersion` consumer shape.
- Added a `p-limit` package-corpus regression so the default standalone corpus now exercises another common ESM default-export utility shape alongside semver and zod.
- Added an `ms` package-corpus regression so the default standalone corpus now also pins another tiny module-only utility package with explicit check/build/run evidence.
- Added a scoped browser replacement-map corpus regression for `@reduxjs/toolkit` so the browser breadth lane now covers a scoped package using both the root entry and a browser-mapped subpath.
- Hardened browser bundle dynamic-import discovery so raw `import(` substrings inside comments and strings no longer create spurious chunk targets; only real dynamic-import syntax now drives the browser bundle chunk graph.
- Aligned browser/package exports resolution with the documented condition ladder so `exports` now takes precedence over legacy entry fields and browser-context branch selection prefers the `browser` condition over deno-only branches when both are published.
- Added a browser-vs-deno browser-surface regression so the package corpus now pins the browser-condition preference when both conditions are present in one published export map.

## Historical stage tasks

### 1. Package corpus expansion

Grow the corpus across representative shapes rather than anecdotal package wins:

- browser-context packages claimed as **checkable** / **deployable-through-host**
- Node-context packages claimed as **checkable** / **buildable** / **executable** only where the
  documented Node subset supports them
- utility packages, dual-format packages, scoped packages, exports-map cases, browser-condition
  cases, browser-rewrite cases, and mixed-format packages

Every corpus result must record the exact rung achieved instead of collapsing outcomes into one
broad “supported package” label.

### 2. Browser packaging improvements

Deepen `kali build --bundle` while preserving the spec's single-linked-core model:

- literal-string `import()` lowering into deterministic bundle chunks
- tree shaking
- source-map companions
- wrapper-format breadth where the owning CLI/spec rows allow it

### 3. Broader browser-targeted analysis/build breadth

Expand the browser-targeted package/API surface that the Phase-3 ecosystem story relies on,
without claiming a standalone browser runtime:

- more real-browser ambient-typing/package-resolution cases
- broader browser-targeted corpus fixtures
- emitted-bundle smoke runs in a real browser harness

### 4. Open-ended cross-module constraint solving

Lift the early annotation boundary only for the documented analyzable Phase-3 cases:

- explicit compile-time budgets
- deterministic solver cutoffs
- evidence-backed improvements at public/module boundaries where Kali can justify them

### 5. Registry-analysis handoff

Keep package-analysis planning aligned with the spec while leaving availability to the owning later
stage:

- browser/Node/runtime-profile/compatibility context inheritance stays aligned across analysis
  commands
- package-analysis command surfaces (`package-effects`, `package-audit`) remain owned by their
  Phase-2 and Phase-4 stages instead of gaining Phase-3 preview-only shadow modes
- schema, diagnostics, and package-corpus evidence stay consistent with that split

### 6. Evidence

- package-corpus results recorded per command/context/rung
- bundle/code-splitting/tree-shaking/source-map tests
- literal-string `import()` lowering tests
- cross-module inference goldens
- regression coverage proving still-gated dynamic import / browser-runtime / package-analysis cases
  remain gated

## Status

Stage 3.3 is complete.

Forward-looking package/browser/inference widening is already tracked in the owning spec chapters
instead of this completed stage document:
- [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md) for browser and host-surface breadth,
- [`specs/14-packages.md`](../../specs/14-packages.md) for package-compatibility rules and support-rung discipline, and
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md) for exact availability boundaries.

## Remaining Work

This stage's closed follow-up lane stays intentionally narrow:
- broader package/browser/inference widening remains owned by the spec chapters above
- the exact support rung still comes from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)
- evidence for broader corpus or browser-host cases continues to live in the test suite instead of reopening the stage checklist

This file remains the historical implementation playbook for the Phase-3 ecosystem-breadth
milestone rather than an open-ended corpus wishlist.
