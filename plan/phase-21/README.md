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
- Implement callback-bearing array methods such as `find`, `findIndex`, `findLast`, `findLastIndex`, `map`, `filter`, `some`, `every`, `reduce`, `reduceRight`, and `flatMap` only with faithful callback, abrupt-completion, and finalization behavior. The supported literal-array identity `values.map((value) => value)` slice is now live, the literal-array `values.filter((value) => value)` slice now also lives, the static literal-array `values.filter((value) => value > 1)` predicate iteration slice now also lives, the literal-array `values.some((value) => value)` / `values.every((value) => value)` boolean slices now also live, the literal-array `values.some((value) => value > 1)` / `values.every((value) => value > 1)` comparison slices now also live, the literal-array `values.flatMap((value) => [value])` slice now also lives, the static numeric `values.reduce((acc, value) => <numeric expr>, initial)` / `values.reduceRight((acc, value) => <numeric expr>, initial)` slices now also live for integer results, and the static literal-array `find` / `findIndex` / `findLast` / `findLastIndex` predicate slices now also live, including strict equality/inequality callbacks over statically known primitive operands; browser-requested run/test browser-harness coverage now also accepts those supported slices, standalone build/check smoke now also exercises the supported `map`/`filter`/`some`/`every`/`flatMap`/`reduce`/`reduceRight` JS slices, and browser-targeted `check` / `build --bundle` JSON-output coverage now pins the supported find-family slice across JS, TS, JSX, and TSX input, while remaining broader callback-bearing forms stay gated until their faithful lowering exists. The static primitive-literal `includes` / `indexOf` / `lastIndexOf` search slice over literal arrays is now also folded during lowering, with standalone JS runtime smoke covering the supported `fromIndex` behavior, including omitted `lastIndexOf` `fromIndex` defaulting to the array tail; the checker now gates dynamic receiver/search/fromIndex search forms with `E5506` instead of allowing partial lowering; the static literal-array `values.at(index)` slice now also lowers numeric indexes, including negative indexes from the tail and statically out-of-range indexes that emit `undefined`, while dynamic `at` forms remain gated with `E5506`; the static primitive-literal `values.join(separator?)` slice now also folds omitted and statically-known string separators, with dynamic separators on static literal-array receivers gated with `E5506`; the static ASCII string `includes` / `indexOf` / `lastIndexOf` / `startsWith` / `endsWith` search/prefix/suffix slice now also lowers statically-known receiver/search/fromIndex/position/endPosition operands and gates dynamic search/position operands or non-ASCII static operands with `E5506` when the receiver is statically known; the static ASCII `value.slice(start, end?)` slice now also lowers statically-known integer bounds, including negative-bound normalization and empty results, while dynamic bounds and non-ASCII static receivers remain gated with `E5506`; the static ASCII `value.substring(start, end?)` slice now also lowers statically-known integer bounds, including negative-bound clamping and swapped bounds when `start > end`, while dynamic bounds and non-ASCII static receivers remain gated with `E5506`; the static ASCII `value.charAt(index?)` slice now also folds omitted and statically-known integer indexes, including negative/out-of-range indexes that produce the empty string, while dynamic receivers/indexes and non-ASCII static receivers remain gated with `E5506`; the static ASCII no-argument `value.trim()` / `value.trimStart()` / `value.trimEnd()` slice now also lowers ASCII whitespace trimming over statically-known strings while argument-bearing and non-ASCII static receivers remain gated with `E5506`; the static ASCII no-argument `value.toLowerCase()` / `value.toUpperCase()` slice now also lowers ASCII-only case conversion over statically-known strings while argument-bearing and non-ASCII static receivers remain gated with `E5506`; the static ASCII `value.replace(search, replacement)` slice now also folds first-match replacement over statically-known ASCII string operands, including empty-search insertion when the replacement string contains no `$` substitution marker, while dynamic operands, non-ASCII static operands, RegExp search values, replacement-pattern semantics, and callback replacements remain gated with `E5506`; the static ASCII `value.repeat(count)` slice now also folds statically-known integer repeat counts from 0 through 1024, while non-ASCII receivers, dynamic counts, negative counts, and out-of-bounds counts remain gated with `E5506`; the static ASCII `value.padStart(length, padding?)` / `value.padEnd(length, padding?)` slice now also folds statically-known integer target lengths from 0 through 1024 with omitted/default-space or statically-known ASCII padding, while non-ASCII receivers or padding, dynamic lengths or padding, negative lengths, and out-of-bounds lengths remain gated with `E5506`; broader dynamic forms remain on the wider object/builtin semantics track.
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
