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

### 21.2 Iterator and async-iterator protocols

- Expand `for...of`, `for await...of`, spreads, `Array.from`, object-enumeration helpers, `Set`/`Map` iteration, and iterable consumption beyond current bounded static slices.
- Current smoke now also pins mixed/bracketed frozen callable aliases for `Object.keys` / `Object.values` / `Object.entries` on the browser-requested `for await` helper path, plus parenthesized receiver-wrapped bracketed aliases for those helpers in js-like input; browser-harness coverage now also includes the logical-or `Object.entries` wrapper slice alongside the existing logical-and/nullish forms, and now also exercises the parenthesized receiver-wrapped bracketed helper variants for that same slice.
- Current smoke now also covers the frozen root-object `Object.freeze((globalThis["Reflect"]))["ownKeys"]` alias on the supported `Reflect.ownKeys` slice in JS-like input.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, async iterator finalization, and error propagation.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.
- Current smoke coverage now also exercises Set/Map constructor break/continue finalization on the supported iterator slice in run/test and browser-harness paths, plus frozen `Object.entries` helper-call coverage on the object-enumeration slice.
- Keep transparent wrapper handling only where it remains deterministic and evidence-backed.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, Promise, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned.
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
