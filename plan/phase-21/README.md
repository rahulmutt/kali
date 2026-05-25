# Phase 21 — Semantic Completeness and Conformance

## Goal

Close remaining language gaps by either implementing faithful semantics with evidence or preserving explicit, tested availability gates.

Keep this file at the sequencing level. Exact coverage belongs in tests, maturity/current-state notes, schemas, and proof files.

## Owning specs

- `specs/02-lexer-parser.md`
- `specs/03-ast.md`
- `specs/04-type-system.md`
- `specs/05-ir.md`
- `specs/10-runtime.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

## Work packets

### 21.1 Generators and async generators

- Implement resumable generator and async-generator state-machine lowering.
- Cover `yield`, `yield*`, `return`, `throw`, `try/finally`, iterator close/finalization, and async interaction.
- Preserve generator/function-kind metadata through parser, AST, HIR, MIR, LIR, codegen, and export analysis.
- Keep unsupported generator forms behind canonical `E5506` gates until the full runtime path exists.
- Current smoke now also exercises sequence-expression-wrapped generator and async-generator class expressions on the supported run/check/build paths, including the yield* delegation slice; the browser-requested JS, TS, JSX, and TSX harness lanes now mirror that same sequence-wrapped generator/async-generator rejection on the yield* slice. Library-export collection now also keeps generator and async-generator default-export expressions rejected through transparent wrappers such as parentheses and sequence expressions, so the canonical `E5506` gate stays consistent across the base-library artifact path.
- Browser check/build smoke now also rejects anonymous default-export generator and async-generator function declarations across JS, TS, JSX, and TSX inputs on the supported browser harness path.

### 21.2 Iterator and async-iterator protocols

- Expand `for...of`, `for await...of`, spreads, `Array.from`, object-enumeration helpers, `Set`/`Map` iteration, and iterable consumption beyond current bounded static slices.
- Current smoke now also pins mixed/bracketed frozen callable aliases for `Object.keys` / `Object.values` / `Object.entries` on the browser-requested `for await` helper path, plus parenthesized receiver-wrapped bracketed aliases for those helpers in js-like input; browser-harness coverage now also includes the logical-or `Object.entries` wrapper slice alongside the existing logical-and/nullish forms, and now also exercises the parenthesized receiver-wrapped bracketed helper variants for that same slice, including the single-quoted parenthesized `Object.freeze((globalThis['Object'])["entries"])` alias, with browser-bundle smoke now also covering the parenthesized receiver-wrapped bracketed `Object.entries` helper variant and the parenthesized bracket-root `Object.entries` helper variant, plus the parenthesized single-quoted bracketed `Object.entries` helper variant on that same browser-harness/browser-bundle matrix, plus the parenthesized receiver-wrapped bracketed `Object.keys` / `Object.values` helper variants on the same browser-harness/browser-bundle matrix, and the parenthesized single-quoted receiver-bracketed `Object.keys` helper variant on that same matrix, and the dotted parenthesized receiver-wrapped bracket-root `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` forms on that same matrix. Promise combinator smoke now also includes `Promise.any` on the standalone run/test and browser harness/bundle paths, including the mixed-bracket frozen-callable alias family around `globalThis["Promise"]['any']`; the browser-runtime spread fixture now matches the same `Object.entries` alias set, and browser object-values spread smoke now also carries the mixed double/single-quoted `globalThis["Object"]['values']` helper variant plus the parenthesized single-quoted receiver-bracketed `Object.values` callable alias. Browser-harness/browser-bundle spread smoke now also covers frozen callable `Object.keys` / `Object.entries` spread aliases, including the parenthesized receiver-wrapped bracketed and single-quoted forms, and Promise.any smoke now also includes the parenthesized receiver-wrapped dotted `Object.freeze((globalThis["Promise"]).any)` / `Object.freeze((globalThis['Promise']).any)` aliases.
- Current smoke now also covers the frozen root-object `Object.freeze((globalThis["Reflect"]))["ownKeys"]` alias on the supported `Reflect.ownKeys` slice in JS-like input, and the browser-bundle global-object `Object.freeze((globalThis['Object'])['keys'])` / `Object.freeze((globalThis['Object'])['values'])` aliases alongside the existing double-quoted root forms on the supported Object helper slices; the js-like object-enumeration helper smoke now also covers parenthesized single-quoted bracket-root `Object.values` / `Object.entries` aliases, and browser-harness/browser-bundle smoke now also covers the single-quoted parenthesized receiver-bracketed `Object.values` alias on that same helper slice. The `Set` / `Map` frozen-callable inventories now also carry nullish/logical wrapper aliases around the root and `globalThis` constructor forms on the same supported constructor slices, keeping that alias surface aligned with the constructor-result smoke.
- Current smoke now also pins the canonical `E5506` gate for array callback-produced iterables such as `find` / `findIndex` / `findLast` / `findLastIndex` / `map` / `filter` / `some` / `every` / `reduce` / `reduceRight` / `flatMap` until faithful callback lowering exists, with the broader array-callback rejection matrix now covered in the lowering unit, build/check smoke, runtime smoke, CLI smoke, and browser-harness coverage.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, async iterator finalization, and error propagation.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.
- Current smoke coverage now also exercises Set/Map constructor break/continue finalization on the supported iterator slice in run/test and browser-harness paths, plus frozen `Object.entries` helper-call coverage on the object-enumeration slice and the parenthesized frozen bracket-root `Object.entries` alias in browser bundle/harness paths, and `Array.from` over frozen `globalThis["Set"]` / `globalThis['Set']` / `globalThis['Map']` / `globalThis["Map"]` constructor results in the standalone and browser harness/bundle smoke lanes, with browser bundle/harness smoke now also covering the parenthesized direct constructor aliases `new (globalThis["Set"])` and `new (globalThis['Map'])` on that same supported slice. Browser-object-values spread smoke now also includes the single-quoted parenthesized receiver-bracketed `Object.values` alias.
- Mixed-quote `Object.fromEntries` aliases now also stay on the same static object-enumeration helper path in codegen and smoke.
- Current smoke now also includes direct mixed-quote `Array.from` aliases in the shared array-iteration inventory.
- Keep transparent wrapper handling only where it remains deterministic and evidence-backed.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, Promise, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned. Current smoke now also covers the single-quoted receiver-wrapped `Math.floor` / `Math.trunc` / `Math.ceil` frozen-callable aliases on the supported helper slice, and direct build smoke now also covers the `Math.sqrt` / `Math.cbrt` literal slices in `.jsx` and `.tsx` input on both the Deno and browser surfaces. The supported static-call inventory now also recognizes `Promise.any` aliases, including bracketed frozen-callable forms on the shared browser body, and the compiler build smoke now also exercises that shared `Promise.any` body across Deno and browser `ts` / `js` / `jsx` / `tsx` inputs; the browser harness/bundle path now also covers bracket-root frozen `globalThis["Promise"]["any"]` / `globalThis['Promise']['any']` aliases and the parenthesized receiver-wrapped dotted `Object.freeze((globalThis["Promise"]).any)` / `Object.freeze((globalThis['Promise']).any)` aliases. The object-has-own smoke now also covers the single-quoted, parenthesized single-quoted, and frozen single-quoted callable aliases on the supported `Object.fromEntries` path.
- Keep non-literal dynamic import, broad reflective APIs, eval-adjacent forms, and unsupported object-model/runtime APIs gated unless their maturity rows are promoted.
- Pair each promotion with checker, lowering, runtime, browser/context, and JSON-output evidence where applicable.

### 21.4 Bounded TS/JS inference

- Grow inference inside deterministic budgets only.
- Preserve annotation-required boundaries for exported/public and cross-module surfaces when inference would exceed the bounded contract.
- Add positive and negative checker baselines for TS and first-class JS input.
- Keep transparent wrapper handling aligned with bounded-literal/static-resolution paths only when cheap and deterministic.

### 21.5 Conformance hygiene

- Maintain compact dashboards of supported vs gated semantics.
- Remove implementation-journal prose from plan files.
- Keep diagnostic codes and wording aligned with `specs/15-errors.md`.

## Exit gate

- Supported semantics have parser/checker/lowering/runtime evidence.
- Unsupported semantics fail through canonical diagnostics.
- `cargo test --workspace` passes for affected Rust paths.
- Public availability wording remains aligned with `specs/19-feature-maturity.md`.
