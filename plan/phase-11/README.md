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
- Progress note: the limited literal-array `for...of` lowering slice is now live on the supported `check` / `build` / `run` / `test` paths, including browser-targeted smoke coverage in TS and `.js` input; the current slice covers literal arrays with literal elements, const numeric alias elements, const string alias elements, parenthesized const-alias wrappers, and simple variable bindings, including direct identifier bindings plus simple const-bound aliases and alias chains to those arrays in both TS and `.js` input, plus parenthesized identifier binding wrappers around the loop target, and the browser-harness `run` lane now mirrors the alias, alias-chain, string-alias, identifier-binding, parenthesized-binding, and parenthesized-wrapper smoke cases in both TS and `.js` input; `for await...of` now shares that same literal-array lowering slice for literal arrays with literal elements, const numeric alias elements, const string alias elements, parenthesized const-alias wrappers, and simple variable bindings, including browser-harness `run` / `test` smoke coverage in both TS and `.js` input, while broader iterator-protocol handling remains gated. The shared lowering helper now also peels transparent parenthesized/type-assertion/satisfies/chain wrappers around the supported array iterables/elements. `Math.trunc` and `Math.ceil` now also constant-fold statically-known numeric literals, including const numeric alias chains, on the supported `check` / `build` / `run` / `test` paths, and now also have direct `build` smoke coverage on const alias chains in the Deno and browser `.js` / `.ts` inputs. `Math.abs` now also constant-folds statically-known integer-valued numeric literal operands, and `Math.sign` now also constant-folds statically-known numeric literal operands, on the supported smoke paths, including const numeric alias chains. `Math.hypot` now also has an exact integer-literal perfect-square-sum slice in the same checker/codegen path, and browser-harness `run` / `test` smoke now also covers the `Math.log2` / `Math.log10` literal slice in TS and `.js` input, and `Math.exp` / `Math.log` now also have exact zero/one identity folds while the broader non-identity calls remain gated.

### 11.2 Missing expression and built-in semantics

- Promote nullish coalescing `??` and currently unsupported `Math` members only with runtime/checker/codegen coverage.
- Keep unsupported built-ins rejected explicitly rather than lowering to placeholders.
- Mirror accepted and rejected cases across TS/JS and JSON-output paths where user-visible.
- Progress note: `Math.abs` now also constant-folds statically-known integer-valued numeric literal operands, including const numeric alias-chain resolution, on the supported codegen/runtime smoke paths, and `Math.sign` now also constant-folds statically-known numeric literal operands, including const numeric alias-chain resolution, on the supported codegen/runtime smoke paths; `Math.max` / `Math.min` now also constant-fold statically-known integer-valued numeric literal operands, including const alias chains, on the supported codegen smoke paths; `Math.imul` now also constant-folds statically-known integer literal operands, including const alias chains; `Math.clz32` now also constant-folds statically-known integer literal operands, including const alias chains; `Math.floor` now has integer-only smoke coverage and also constant-folds statically-known numeric literals such as `1.6`; that `Math.floor` path now also resolves through const numeric alias chains; `Math.round` now also has integer-only smoke coverage, and statically-known numeric literal operands now constant-fold through the round path; `Math.sqrt` now has a statically-known perfect-square integer literal path that also follows const numeric alias chains; `Math.cbrt` now has a statically-known perfect-cube integer literal path that also follows const numeric alias chains, including negative perfect cubes; `Math.sqrt` / `Math.cbrt` now also have direct `build` smoke coverage in the Deno and browser `.js` / `.ts` inputs; `Math.log2` now has a statically-known positive power-of-two integer literal path that also follows const numeric alias chains; `Math.log10` now has a statically-known positive power-of-ten integer literal path that also follows const numeric alias chains; `Math.log2` / `Math.log10` const alias-chain smoke is now also covered by direct `build` / `run` regression tests in the Deno `.js` and `.ts` inputs; `Math.pow` now has an integer-exponent path with checker/codegen/runtime smoke coverage on positive integer literals, including const numeric alias-chain handling for the same supported integer path, plus browser-bundle smoke coverage on both TS and `.js` input, and the negative-exponent host-import gate now returns a canonical error instead of panicking; that negative-exponent `Math.pow` gate now also has direct JS-input `check` / `--output json check` smoke coverage; the supported `Math.max` codegen/runtime slice now also accepts `globalThis.Math.max` in JS input on the same lowering path; `Math.exp` / `Math.log` now also have exact zero/one identity folds, while representative non-identity calls still have explicit canonical E5506 rejection smoke alongside the existing `Math.sqrt`/`Math.log2`/`Math.log10` gates, and the build/check corpus now also pins those rejections across Deno and browser `.js` / `.ts` inputs, with standalone `run` / `test` plus browser-harness smoke now also pinning representative `Math.exp` / `Math.log` rejections on the JS-input math gate path and browser-targeted `check` / `build` smoke in `.jsx` / `.tsx` input; nullish coalescing `??` is now implemented and covered across direct and browser-targeted smoke paths; `Promise.allSettled` now has runtime/checker/codegen coverage across standalone, browser-targeted, and Node smoke paths.

### 11.3 Dynamic loading and module semantics

- Preserve literal-string `import()` as linked-graph lowering.
- Keep non-literal `import(expr)` gated until a host-mediated loading/effect model exists.
- Expand CJS/ESM interop tests only where the linked-artifact model remains deterministic.
- Progress note: the literal-string dynamic import path now also resolves raw backtick template-literal specifiers with statically evaluable interpolations, keeping `check` / `build` resolution and browser-bundle chunk discovery aligned for that narrower constant-expression slice; direct browser-bundle smoke now also covers the same raw backtick template-literal specifier slice in TS and `.js` input, and browser-targeted `check` / `build --bundle` now also pin the non-literal `import(expr)` gate in `.js` input with matching JSON-output rejection coverage.

### 11.4 Bounded inference hardening

- Improve local/intra-module inference under deterministic budgets.
- Add negative coverage for annotation-required public boundaries.
- Do not promote open-ended public-API or cross-module solving without a new solver budget and evidence lane.

## Exit gate

- Supported semantics have conformance and minimized regressions.
- Unsupported parser-accepted forms have canonical gates.
- `specs/19-feature-maturity.md` and affected owning specs match the actual implementation.
