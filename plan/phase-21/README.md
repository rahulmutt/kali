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
- Async class methods now ride the shared async-function lowering path; generator class methods remain gated.
- Current coverage also exercises async class methods through build smoke in TS, JS, JSX, and TSX input on both the Deno and browser artifact paths.
- Keep all unsupported forms behind canonical `E5506` gates until the full runtime path exists.

### 21.2 Iterator and async-iterator protocols

- Expand `for...of`, `for await...of`, spreads, and iterable consumption beyond current bounded static slices.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, and async iterator finalization.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned; current coverage also includes browser-bundle smoke for the `Number.isFinite` / `Number.isNaN` / `Number.isInteger` / `Number.isSafeInteger` primitive-value slice across JS, TS, JSX, and TSX input.
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
