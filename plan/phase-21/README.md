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
- Tighten direct-runtime element-access evidence for arrays produced by static helper folds before using length-only smoke as evidence for broader indexed array semantics.
- Keep transparent wrapper handling only where it remains deterministic and evidence-backed.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, Promise, dynamic import, reflection, operators, array helpers, string helpers, numeric helpers, and predicates only when observable JavaScript behavior is pinned by checker/lowering/runtime evidence.
- Treat the existing bounded static slices as the current implementation baseline rather than as a permission to generalize dynamic behavior. This includes the supported literal-array callback/search/join/`toString`/`at` slices, static ASCII string helper folds (including omitted-search string search defaulting, `String.prototype.at`, and `trimLeft` / `trimRight` alias folding within the static slice), the static no-argument string identity helper slice (`toString` / `valueOf` over statically-known string primitives), static integer-result `parseInt` / `Number.parseInt` folds, the static integer-result `parseFloat` / `Number.parseFloat` ASCII slice, static numeric-only global `isFinite` / `isNaN` predicate folds, static zero-identity `Math.fround(0)` folds, and static `Array.isArray(value)` predicate folds, including the supported false-result slice for static `Set` / `Map` constructor targets.
- Keep broader callback-bearing forms, dynamic receivers/operands, non-ASCII or RegExp-dependent string behavior, broad reflection, non-literal dynamic import, eval-adjacent forms, and unsupported object-model/runtime APIs gated with canonical diagnostics unless their maturity rows are promoted.
- Pair each promotion with checker, lowering, runtime, browser/context, and JSON-output evidence where applicable.
- Progress: the static ASCII `parseFloat` / `Number.parseFloat` integer-result slice now has checker and lowering coverage for direct, `globalThis`, `Number`, bracketed, and frozen callable aliases, while dynamic, non-ASCII, fractional, argument-bearing, and no-digit forms remain on the canonical feature gate. The static ASCII `String.fromCharCode` / `String.fromCodePoint` slice now folds direct, `globalThis`, bracketed, and frozen callable aliases for statically-known 0–127 integer code units, while dynamic, fractional, negative, and non-ASCII code units remain gated.

### 21.4 Bounded TS/JS inference

- Grow inference inside deterministic budgets only.
- Preserve annotation-required boundaries for exported/public and cross-module surfaces when inference would exceed the bounded contract.
- Add positive and negative checker baselines for TS and first-class JS input.
- Keep transparent wrapper handling aligned with bounded-literal/static-resolution paths only when cheap and deterministic.

### 21.5 Conformance hygiene

- Maintain compact dashboards of supported vs gated semantics.
- Keep plan files free of implementation-journal prose; use compact baseline summaries and leave exhaustive evidence to tests, specs, schemas, and proof files.
- Keep diagnostic codes and wording aligned with `specs/15-errors.md`.
- Mirror support and rejection coverage across source classes and command contexts when claims span those contexts. Current conformance hygiene now pins the static ASCII string-search helper slice on standalone run, JSON check, and browser-targeted bundle paths across the JS/TS/JSX/TSX source-class matrix while preserving dynamic-operand gates. The static ASCII `String.prototype.split` helper slice now has JSON check, browser-targeted bundle source-class matrix, standalone run indexed-element evidence, and canonical dynamic/non-ASCII/limit gate coverage. The static ASCII `String.prototype.toLowerCase` / `toUpperCase`, `repeat`, `padStart` / `padEnd`, `replace`, and `replaceAll` helper slices now also mirror supported browser-targeted bundle evidence across JS, TS, JSX, and TSX input, including JSON-output coverage, while preserving canonical argument-bearing, non-ASCII, dynamic-receiver/operand, and substitution-marker gates.

## Exit gate

- Supported semantics have parser/checker/lowering/runtime evidence.
- Unsupported semantics fail through canonical diagnostics.
- `cargo test --workspace` passes for affected Rust paths.
- Public availability wording remains aligned with `specs/19-feature-maturity.md`.
