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
- Progress note: the limited literal-array `for...of` lowering slice is now live on the supported `check` / `build` / `run` / `test` paths, including browser-targeted smoke coverage in TS and `.js` input; the current slice covers literal arrays with literal elements, const numeric alias elements, const string alias elements, and simple variable bindings, including direct identifier bindings plus simple const-bound aliases and alias chains to those arrays in both TS and `.js` input, and the browser-harness `run` lane now mirrors the alias, alias-chain, string-alias, and identifier-binding smoke cases in both TS and `.js` input; `for await...of` now shares that same literal-array lowering slice for literal arrays with literal elements, const numeric alias elements, const string alias elements, and simple variable bindings, including browser-harness `run` / `test` smoke coverage in both TS and `.js` input, while broader iterator-protocol handling remains gated. `Math.trunc` and `Math.ceil` now also constant-fold statically-known numeric literals, including const numeric alias chains, on the supported `check` / `build` / `run` / `test` paths. `Math.abs` now also constant-folds statically-known integer-valued numeric literal operands, and `Math.sign` now also constant-folds statically-known numeric literal operands, on the supported smoke paths, including const numeric alias chains.

### 11.2 Missing expression and built-in semantics

- Promote nullish coalescing `??` and currently unsupported `Math` members only with runtime/checker/codegen coverage.
- Keep unsupported built-ins rejected explicitly rather than lowering to placeholders.
- Mirror accepted and rejected cases across TS/JS and JSON-output paths where user-visible.
- Progress note: `Math.abs` now also constant-folds statically-known integer-valued numeric literal operands, including const numeric alias-chain resolution, on the supported codegen/runtime smoke paths, and `Math.sign` now also constant-folds statically-known numeric literal operands, including const numeric alias-chain resolution, on the supported codegen/runtime smoke paths; `Math.floor` now has integer-only smoke coverage and also constant-folds statically-known numeric literals such as `1.6`; that `Math.floor` path now also resolves through const numeric alias chains; `Math.round` now also has integer-only smoke coverage, and statically-known numeric literal operands now constant-fold through the round path; `Math.sqrt` now has a statically-known perfect-square integer literal path that also follows const numeric alias chains; `Math.cbrt` now has a statically-known perfect-cube integer literal path that also follows const numeric alias chains, including negative perfect cubes; `Math.log2` now has a statically-known positive power-of-two integer literal path that also follows const numeric alias chains; `Math.log10` now has a statically-known positive power-of-ten integer literal path that also follows const numeric alias chains; `Math.log2` / `Math.log10` const alias-chain smoke is now also covered by direct `build` / `run` regression tests in the Deno `.js` and `.ts` inputs; `Math.pow` now has an integer-exponent path with checker/codegen/runtime smoke coverage on positive integer literals, including const numeric alias-chain handling for the same supported integer path, plus browser-bundle smoke coverage on both TS and `.js` input, and the negative-exponent host-import gate now returns a canonical error instead of panicking; nullish coalescing `??` is now implemented and covered across direct and browser-targeted smoke paths; `Promise.allSettled` now has runtime/checker/codegen coverage across standalone, browser-targeted, and Node smoke paths.

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
