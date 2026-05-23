# Phase 21 — Semantic Completeness and Conformance

## Goal

Close remaining language gaps by either implementing faithful semantics with evidence or preserving explicit, tested availability gates.

Keep this file at the sequencing level: exact coverage belongs in tests, schemas, proof files, and maturity/current-state notes, not in plan prose.

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

- Expand `for...of`, `for await...of`, spreads, and iterable consumption beyond current bounded static slices, including transparent `Object.freeze(...)` wrappers around direct `Set`/`Map` constructor targets where the iterable shape stays static, including parenthesized `Object.freeze((new Set(...)))` / `Object.freeze((new Map(...)))` variants, and the parenthesized frozen-result smoke now also accepts statically decidable nullish/logical wrappers around those `Set`/`Map` constructor targets. Current smoke also covers `Array.from(Object.freeze(new Set(...)))` / `Array.from(Object.freeze(new Map(...)))` constructor-result wrappers, including parenthesized variants, in the browser harness/bundle matrix, and the browser-requested/browser-bundle `Array.from` alias smoke now also includes the single-quoted `globalThis.Array['from']` and `globalThis['Array'].from` frozen-callable aliases on that same supported slice; the same smoke inventory now also carries the parenthesized dot-root `Object.freeze((globalThis.Array).from)` / `Object.freeze((globalThis.Array)["from"])` / `Object.freeze((globalThis.Array)['from'])` aliases, and the browser-array-from-set-map harness/bundle sources now mirror that shared alias inventory, including the parenthesized double-quoted bracket-root property-access alias `Object.freeze((globalThis["Array"])["from"])` on that same supported slice; the shared `Reflect.ownKeys` frozen-callable inventory now also includes the bare `Reflect.ownKeys` / `Object.freeze((Reflect.ownKeys))` pair alongside the existing globalThis variants, plus the parenthesized bracket-root `Object.freeze((globalThis["Reflect"]).ownKeys)` / `Object.freeze((globalThis['Reflect']).ownKeys)` and `Object.freeze((globalThis["Reflect"]["ownKeys"]))` / `Object.freeze((globalThis['Reflect']['ownKeys']))` forms, and the dot-root `globalThis.Reflect["ownKeys"]` / `globalThis.Reflect['ownKeys']` aliases plus their `Object.freeze(...)` wrappers. The same iterator slice also now accepts statically decidable nullish/logical wrappers around supported object-enumeration helper aliases, including `Object.freeze((null ?? Reflect.ownKeys))`, `Object.freeze((true && Reflect.ownKeys))`, and `Object.freeze((false || Reflect.ownKeys))`, on the same transparent-wrapper path; the browser-bundle smoke lane now mirrors that same `Reflect.ownKeys` wrapper inventory too, and the codegen regression matrix now also covers the `false ||` wrapper form for `Array.from` and `Reflect.ownKeys`.
- Implement protocol lookup, `next` result handling, abrupt completion, iterator close, and async iterator finalization; the direct runtime smoke now also mirrors the expanded `Promise.allSettled` alias/freeze inventory already exercised by the build lanes, including the parenthesized bracket-root dot-call aliases, and the object-enumeration smoke now also carries break/continue finalization coverage for `Object.keys(...)`, `Object.values(...)`, `Object.entries(...)`, and `Reflect.ownKeys(...)`. The browser harness/bundle Array.from set/map smoke now also mirrors the mixed-quote bracket-root aliases `globalThis["Array"]['from']` / `globalThis['Array']["from"]`, plus their parenthesized transparent-wrapper variants on the same supported slice, and now also includes the nullish-wrapped `globalThis['Array']['from']` alias on that same supported slice.
- Add conformance fixtures for supported built-ins and negative diagnostics for unimplemented protocol edges.
- Current browser and checker evidence also keeps the bracketed `globalThis["Array"].from` freeze alias on the static set/map slice aligned across the existing smoke lanes, the fully bracketed `globalThis["Array"]["from"]` alias now also shares that coverage, the direct double-quoted `Array["from"]` freeze alias now also joins the standalone and browser-requested smoke lanes, and the single-quoted root `globalThis['Set']` / `globalThis['Map']` constructor spellings now also ride the standalone smoke lane; the browser bundle/harness smoke now also includes parenthesized frozen `Set` / `Map` constructor aliases, and the Array.from source inventory now also includes the double-quoted parenthesized bracket-root `Object.freeze((globalThis["Array"]).from)` alias alongside the single-quoted sibling. The same iterator slice now also keeps the single-quoted `globalThis['Reflect'].ownKeys` / `globalThis['Reflect']['ownKeys']` frozen callable aliases aligned across the supported object-helper smoke lanes, and the browser bundle Array.from assertions now track the full helper inventory while the harness wrapper stays aligned with the shared alias source; checker/codegen coverage also picked up the bracketed `globalThis["Array"].from` wrapper path through the transparent `Object.freeze((true && ...))` form.

### 21.3 Dynamic language and built-in semantics

- Widen object-model, Math, BigInt, dynamic import, reflection, and operator semantics only when observable JavaScript behavior is pinned; the current smoke also keeps the optional-chain-wrapped `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` frozen aliases aligned with the existing helper slices, including the optional-chain-wrapped property-call bracket forms, and now also exercises the parenthesized optional-chain-root `Object.hasOwn` aliases through the shared helper inventory; browser-requested run/test harness smoke now also covers logical/nullish wrappers around frozen literal dynamic-import specifiers in JS input.
- Keep non-literal dynamic import, broad reflective APIs, and eval-adjacent forms gated unless their spec rows are promoted.
- Pair each promotion with checker, lowering, runtime, browser/context, and JSON-output evidence where applicable.

### 21.4 Bounded TS/JS inference

- Grow inference inside deterministic budgets only.
- Preserve annotation-required boundaries for exported/public and cross-module surfaces when inference would exceed the bounded contract.
- Add positive and negative checker baselines for TS and first-class JS input.
- Keep transparent wrapper handling aligned with the bounded-literal path when it stays cheap and deterministic (for example, await-wrapped numeric literals, optional-chain-wrapped or direct same-reference static member comparisons, and statically decidable logical/nullish wrappers); current smoke now also covers the optional-chain-wrapped `globalThis?.Math.round` root spelling, the frozen optional-chain-wrapped `Object.freeze(globalThis?.Math.round)` alias, and the parenthesized optional-chain alias `Object.freeze((globalThis?.Math.round))` in the browser harness/bundle matrix, while the direct run/test smoke lane now also mirrors `Object.freeze((Math.round))` on the same slice. The same browser-harness/browser-bundle matrix now also covers the parenthesized single-quoted bracketed `globalThis['Math']['round']` alias and the parenthesized mixed-quoted-bracketed and single-quoted-bracketed-dot aliases, and the checker regression matrix now also exercises the canonical `Math.round` frozen-callable inventory across JS/JSX/TS/TSX input.

### 21.5 Conformance hygiene

- Maintain compact dashboards of supported vs gated semantics.
- Remove implementation-journal prose from plan files; exact coverage belongs in tests and maturity current-state notes.
- Keep diagnostic codes and wording aligned with `specs/15-errors.md`.

## Exit gate

- Supported semantics have parser/checker/lowering/runtime evidence.
- Unsupported semantics fail through canonical diagnostics.
- `cargo test --workspace` passes for affected Rust paths.
- Public availability wording remains aligned with `specs/19-feature-maturity.md`.
