# Phase 21 — Semantic Completeness and Conformance

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

### 21.1 Generators and async generators

- Implement resumable generator and async-generator state-machine lowering.
- Cover `yield`, `yield*`, `return`, `throw`, `try/finally`, iterator close/finalization, and async interaction.
- Preserve generator/function-kind metadata through parser, AST, HIR, MIR, LIR, codegen, and export analysis.
- Async class methods now ride the shared async-function lowering path; generator class methods remain gated, and the direct runtime-entrypoint gate now also covers generator class expressions in JS/JSX/TSX input plus async-generator default-export class expressions in JS input. Generator codegen now also emits distinct generator vs async-generator gate wording while preserving the same canonical E5506 path.
- Current coverage also exercises async class methods through build smoke in TS, JS, JSX, and TSX input on both the Deno and browser artifact paths.
- The parser now accepts class-expression forms with preserved async/generator metadata, and direct `run` / `test` entrypoints now gate those async class expressions too; generator class expressions now also hit the direct runtime E5506 gate before lowering, including the direct runtime JS/TS coverage path.
- HIR, MIR, LIR, and function-plan tests now also preserve class-expression generator metadata alongside the existing class-method coverage.
- Direct `run` / `test` execution for async class methods now has an explicit E5506 gate; keep generator-class lowering gated until the dedicated packet lands.
- Keep all unsupported forms behind canonical `E5506` gates until the full runtime path exists.

### 21.2 Iterator and async-iterator protocols

- Expand `for...of`, `for await...of`, spreads, and iterable consumption beyond current bounded static slices; current smoke now also covers the direct string-concatenation slice in browser bundle JS, TS, JSX, and TSX input, and browser-requested run/test browser-harness smoke now also covers that same string-concatenation slice in JS, TS, JSX, and TSX input with JSON-output coverage.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, and async iterator finalization.
- Current smoke now also covers the single-quoted bracketed `Reflect` alias over frozen object literals in the direct and browser-harness paths, plus sequence-expression wrappers around the supported static `Reflect.ownKeys(...)` slice in JS input, and browser-requested run/test coverage now also exercises the frozen-object `Reflect.ownKeys(...)` slice in JS, TS, JSX, and TSX input with JSON-output coverage, alongside the browser-bundle string-concatenation slice across `for...of` / `for await...of` in JS, TS, JSX, and TSX input and the matching browser-harness string-concatenation smoke in the same input matrix.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned; current coverage also includes standalone `run` / `test` smoke for the `Number.isFinite` / `Number.isNaN` / `Number.isInteger` / `Number.isSafeInteger` primitive-value slice across JS and TS input with JSON-output coverage, plus browser-bundle smoke for that same slice across JS, TS, JSX, and TSX input, including mixed bracket/dot spellings such as `globalThis["Number"].isNaN`, and the fully bracketed `globalThis["Math"]["sqrt"]` / `globalThis["Math"]["cbrt"]` math slice now also covers JS, TS, JSX, and TSX input on the browser bundle/harness paths. Promise.allSettled now also has standalone build/runtime and browser-requested browser-harness coverage across JS, TS, JSX, and TSX input. The same object-identity slice now also has type-system regression coverage for the `Object.is` alias spellings used by the browser/runtime smoke paths, including parser-backed bracketed `Object.is` and `Number.is*` alias spellings in JS input, and browser-harness `Object.is` same-reference alias-chain smoke now also exercises the bracketed, mixed, root, and dot alias spellings alongside the frozen-object variant.
- Keep non-literal dynamic import, broad reflective APIs, and eval-adjacent forms gated unless their spec rows are promoted.
- Pair each promotion with checker, lowering, runtime, browser/context, and JSON-output evidence where applicable.

### 21.4 Bounded TS/JS inference

- Grow inference inside deterministic budgets only.
- Preserve annotation-required boundaries for exported/public and cross-module surfaces when inference would exceed the bounded contract.
- Add positive and negative checker baselines for TS and first-class JS input.

### 21.5 Conformance hygiene

- Maintain compact dashboards of supported vs gated semantics.
- Remove implementation-journal prose from plan files; exact coverage belongs in tests and maturity current-state notes.

## Exit gate

- Each promoted semantic slice has parser/checker/lowering/runtime evidence.
- Unsupported but recognized forms fail through the canonical diagnostic path.
- Maturity and current-state docs are updated only for evidence-backed availability changes.
