# Phase 11 — Language Semantics and Conformance Closure

## Goal

Close high-value language gaps by either implementing faithful semantics or preserving explicit, tested availability gates.

## Owning specs

- `specs/02-lexer-parser.md`
- `specs/03-ast.md`
- `specs/04-type-system.md`
- `specs/05-ir.md`
- `specs/10-runtime.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

## Work packets

### 11.1 Iterator and generator semantics

- Implement generator lowering only when `yield` / `yield*`, async-generator interactions, and runtime state machines have conformance coverage.
- Implement `for...of` and `for await...of` only with iterator protocol semantics, error/finalization behavior, and browser/Node/Deno evidence.
- Until implemented, keep parser acceptance paired with canonical `E5506` gates.
- Progress note: the limited literal-array `for...of` lowering slice is now live on the supported `check` / `build` / `run` / `test` paths, including browser-targeted smoke coverage in TS and `.js` input; the current slice covers literal arrays with literal elements and simple variable bindings, including simple const-bound aliases to those arrays in both TS and `.js` input, while `for await...of` and the broader iterator-protocol path remain gated.

### 11.2 Missing expression and built-in semantics

- Promote nullish coalescing `??` and currently unsupported `Math` members only with runtime/checker/codegen coverage.
- Keep unsupported built-ins rejected explicitly rather than lowering to placeholders.
- Mirror accepted and rejected cases across TS/JS and JSON-output paths where user-visible.
- Progress note: `Math.floor` now has integer-only smoke coverage; `Math.round` now also has integer-only smoke coverage; `Math.sqrt` now has a statically-known perfect-square integer literal path; `Math.cbrt` now has a statically-known perfect-cube integer literal path; `Math.pow` now has an integer-exponent path with checker/codegen/runtime smoke coverage on positive integer literals; nullish coalescing `??` is now implemented and covered across direct and browser-targeted smoke paths; `Promise.allSettled` now has runtime/checker/codegen coverage across standalone, browser-targeted, and Node smoke paths.

### 11.3 Dynamic loading and module semantics

- Preserve literal-string `import()` as linked-graph lowering.
- Keep non-literal `import(expr)` gated until a host-mediated loading/effect model exists.
- Expand CJS/ESM interop tests only where the linked-artifact model remains deterministic.

### 11.4 Bounded inference hardening

- Improve local/intra-module inference under deterministic budgets.
- Add negative coverage for annotation-required public boundaries.
- Do not promote open-ended public-API or cross-module solving without a new solver budget and evidence lane.

## Exit gate

- Supported semantics have conformance and minimized regressions.
- Unsupported parser-accepted forms have canonical gates.
- `specs/19-feature-maturity.md` and affected owning specs match the actual implementation.
