# Phase 16 — Semantic Closure and Conformance Promotion

## Goal

Close remaining language gaps by either implementing faithful semantics with evidence or preserving explicit, tested availability gates.

## Owning specs

- `specs/02-lexer-parser.md`
- `specs/03-ast.md`
- `specs/04-type-system.md`
- `specs/05-ir.md`
- `specs/10-runtime.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

## Work packets

### 16.1 Generators and iterator protocol

- Generators need a dedicated resumable-iterator representation before faithful lowering can land; the current HIR path does not yet preserve enough function-kind metadata for that work.
- Split the generator packet into prerequisite plumbing, then state-machine lowering, then async-generator/finalization coverage.
- Expand `for...of` and `for await...of` toward full iterator/async-iterator protocol semantics beyond bounded static slices, including spread of supported `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` slices where that remains shape-safe.
- Keep unavailable forms on canonical `E5506` gates with TS/JS/JSX/TSX and JSON-output regressions where applicable.
- Current progress: the browser-targeted check/build smoke and browser-requested run/test/browser-bundle smoke now also exercise the `globalThis.Object.values` / `globalThis.Object["values"]` / `globalThis["Object"].values` / `globalThis["Object"]["values"]` spread slices over `Object.fromEntries(...)` in the supported JS/TS/JSX/TSX matrix, including the `Object.freeze(Object.fromEntries(...))` variant, and now also exercise the matching `globalThis.Object.keys` / `globalThis.Object.entries` alias spellings for the same spread slices in the browser-bundle matrix. Standalone run/test smoke now also covers the shared `for...of` spread-element slice in JSX and TSX input, plus the frozen `Object.freeze(Object.fromEntries(...))` spread slice in JS and TS input with JSON-output coverage. Browser-requested run/test browser-harness coverage now also exercises the shared `for await` spread slices over `Object.fromEntries(...)` operands in JSX and TSX input with JSON-output coverage, and now also covers the mixed-bracket `globalThis.Object["values"]` / `globalThis["Object"].keys` / `globalThis["Object"]["entries"]` spread aliases for the same `Object.fromEntries(...)` operands in that harness matrix. The checker also now invalidates those static iterable/object-helper slices on rebinding so the supported let-bound cases stay honest while rebounded inputs continue through the canonical `E5506` rejection path. Browser-targeted check/build smoke now also covers the matching `Object.keys(...)` / `Object.entries(...)` spread slices over `Object.fromEntries(...)` operands in TS, JSX, and TSX input, and the browser bundle/browser-requested spread-regression lanes now also cover frozen `Object.fromEntries(...)` operands for the shared `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` spread slice. Browser bundle and browser-harness smoke now also cover the frozen `Object.hasOwn(...)` helper slice over `Object.fromEntries(...)` operands in JS, TS, JSX, and TSX input, browser-targeted check/build smoke now also covers the frozen `Object.freeze(Object.fromEntries(...))` spread slice in JS input, and the supported iterator smoke now also covers spread-of-`Reflect.ownKeys(...)` slices over static object-literal operands in JS/TS/JSX/TSX input, and standalone run smoke now also covers the matching `Reflect.ownKeys(...)` spread slice over static object-literal operands in JS and TS input with JSON-output coverage. `Object.is` primitive-literal smoke now also includes simple decimal BigInt slices on the supported build path. Late-object-model smoke now also covers the `globalThis.Proxy["revocable"]` alias in the same gated slice. Async-generator lowering gate coverage now also mirrors the same JSX/TSX input slices on the core check/build/run/test smoke paths, and the browser bundle matrix now also covers string-primitive enumeration for `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` in JS, TS, JSX, and TSX input, with browser bundle/browser-harness smoke now also covering the corresponding `for await` string-primitive enumeration slices for those helpers in the same source-class matrix.
- Codegen unit coverage now also pins the mixed-bracket `globalThis.Object["values"]`, `globalThis["Object"].keys`, and `globalThis["Object"]["entries"]` spread aliases over `Object.fromEntries(...)` operands.

#### 16.1a Generator prerequisites

- Preserve generator/async-generator kind metadata through the lowering pipeline.
- Add resumable iterator/state-machine plumbing needed for `yield`, `yield*`, async interaction, and finalization.
- Current progress: HIR/MIR now preserve function-flavor metadata for sync, async, generator, and async-generator forms; HIR coverage now also pins the same preservation for named function expressions, and MIR coverage continues to do the same; `check_source_file` now mirrors the canonical generator/async-generator lowering gate across Deno and browser TS/JS/JSX/TSX inputs; `check_source_file` and `build_source_file` now also reject class generator and async-generator method syntax through the same canonical unavailable-feature path so class bodies do not silently reinterpret generator modifiers as plain methods; resumable execution lowering remains gated.

#### 16.1b Generator lowering

- Implement generator and async-generator lowering only after the prerequisite plumbing lands, with state-machine, `yield` / `yield*`, async interaction, error, and finalization coverage.

### 16.2 Expression, built-in, and object semantics

- Promote additional operators, compound assignment targets, dynamic language forms, and built-ins only with checker/codegen/runtime evidence.
- Keep unsupported `Math`, object-model, dynamic import, `eval`-adjacent, and reflective forms explicitly gated.
- Verify observable JavaScript semantics before exposing optimization-sensitive folds.
- Current progress: standalone `test` smoke now also covers the zero-base / positive-integer-exponent `Math.pow` slice in JS input, including JSON-output coverage, and `Object.is` now also accepts `Object.freeze`-wrapped same-reference aliases for statically-known object bindings; standalone `run` / `test` smoke now also covers the primitive-literal `Object.is` slice in JS input with JSON-output coverage, and browser-requested `run` / `test` browser-harness smoke now also exercises that same primitive-literal `Object.is` slice across JS, TS, JSX, and TSX input with JSON-output coverage; browser-harness/browser-bundle smoke now also covers the mixed-bracket `globalThis.Math["floor"]` / `globalThis.Math["trunc"]` / `globalThis.Math["ceil"]` forms in TS and `.js` input, with browser-requested run/test JSON-output coverage in JSX and TSX input, and browser-targeted check smoke now also covers the frozen `Object.hasOwn(...)` helper slice over `Object.fromEntries(...)` operands in JS, TS, JSX, and TSX input.

### 16.3 TypeScript and JavaScript inference

- Grow inference inside the bounded-inference contract only.
- Require annotations or conservative `unknown` fallbacks at public/exported or cross-module boundaries when budgets are exceeded.
- Mirror accepted and rejected cases in checker baselines and CLI JSON diagnostics.

### 16.4 Conformance evidence hygiene

- Maintain minimized conformance fixtures for supported and gated behavior.
- Keep dashboards and snapshots concise; do not use plan files as progress logs.

## Exit gate

- Each promoted semantic slice has parser/checker/lowering/runtime evidence.
- Unsupported but recognized forms fail through the canonical diagnostic path.
- Maturity and current-state docs are updated only for evidence-backed availability changes.
