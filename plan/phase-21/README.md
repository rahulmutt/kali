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
- Keep unsupported generator forms behind canonical gates until the full runtime path exists.

### 21.2 Iterator and async-iterator protocols

- Expand `for...of`, `for await...of`, spreads, `Array.from`, object-enumeration helpers, `Set`/`Map` iteration, and iterable consumption beyond current bounded static slices.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, async iterator finalization, and error propagation.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.
- Keep transparent wrapper handling only where it remains deterministic and evidence-backed.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, Promise, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned.
- Implement callback-bearing array methods such as `find`, `findIndex`, `findLast`, `findLastIndex`, `map`, `filter`, `some`, `every`, `reduce`, `reduceRight`, and `flatMap` only with faithful callback, abrupt-completion, and finalization behavior. The supported literal-array identity `values.map((value) => value)` slice is now live, and the literal-array `values.flatMap((value) => [value])` slice now also lives; browser-requested run/test browser-harness coverage still rejects array callback-produced iterables with the canonical E5506 gate; keep the remaining callback-bearing methods gated until their faithful lowering exists.
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
- Mirror support and rejection coverage across source classes and command contexts when claims span those contexts.

## Exit gate

- Supported semantics have parser/checker/lowering/runtime evidence.
- Unsupported semantics fail through canonical diagnostics.
- `cargo test --workspace` passes for affected Rust paths.
- Public availability wording remains aligned with `specs/19-feature-maturity.md`.
