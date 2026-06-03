# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

Keep this map high-level. Exact evidence belongs in tests, schemas, maturity current-state notes, and [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not in active plan prose.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Promote parser-accepted but unavailable semantics only when checker, lowering, runtime, and conformance evidence exist.
- Implement full generator and async-generator execution semantics, including resumable state machines, `yield`, `yield*`, `return`, `throw`, async interaction, and finalization.
- Expand iterator and async-iterator protocol behavior beyond bounded static slices, including lookup, `next` result handling, abrupt completion, close/finalization, and async protocol behavior.
- Implement callback-bearing array methods and other callback-driven built-ins only when faithful callback lowering and error/finalization semantics are present. The supported literal-array identity `values.map((value) => value)` slice now has a live lowering path, and the truthy-literal identity `values.filter((value) => value)` slice is now live on the same direct-runtime path and now also resolves through `Array.from(...)` / spread consumers, static literal-array `values.filter((value) => value > 1)` predicate iteration is now live for supported `for...of` consumers, and strict equality/inequality callbacks now ride the supported static `find` / `findIndex` / `findLast` / `findLastIndex` plus `some` / `every` literal-array slices; keep the remaining callback-bearing methods gated until faithful callback lowering exists.
- Continue widening expression/operator, object-model, BigInt, Math, Promise, dynamic-import, reflection, and built-in semantics where translation-safe. The static literal-array search helper slice now also pins omitted-`fromIndex` `lastIndexOf` semantics to search from the array tail, the static literal-array `at` helper slice now emits `undefined` for statically out-of-range indexes, the static literal-array `slice(...)[index]` direct-consumption slice now folds statically-known finite bounds, the static literal-array `concat(...)[index]` direct-consumption slice now folds statically-known array/primitive operands, the static ASCII string relational slice now lowers `<`, `<=`, `>`, and `>=` for statically-known ASCII string primitives, including transparent `Object.freeze(...)` wrappers, and the static ASCII string search helper slice now covers statically-known `includes` / `indexOf` / `lastIndexOf` / `startsWith` / `endsWith` operands plus omitted-search defaulting to the ECMAScript `undefined` search string and the distinct ECMAScript negative-position rules for `includes` and `indexOf`, while keeping dynamic search/fromIndex/position and non-ASCII cases gated for statically-known string receivers. The static ASCII `String.prototype.slice(start, end?)` helper slice now also folds statically-known integer bounds while preserving the same dynamic-bound/non-ASCII gates, the static ASCII `String.prototype.substring(start, end?)` helper slice now also folds statically-known integer bounds while preserving the same dynamic-bound/non-ASCII gates, the static ASCII `String.prototype.charAt(index?)` and `String.prototype.charCodeAt(index?)` helper slices now fold omitted and statically-known integer indexes while preserving dynamic-receiver/dynamic-index/non-ASCII gates, the static ASCII `String.prototype.codePointAt(index?)` helper slice now folds omitted and statically-known integer indexes, including `undefined` for negative/out-of-range static indexes, while preserving dynamic-receiver/dynamic-index/non-integer/non-ASCII gates, and the static ASCII no-argument `String.prototype.trim` / `trimStart` / `trimEnd` helper slice now folds ASCII whitespace trimming, including the `trimLeft` / `trimRight` alias spellings, while preserving argument-bearing/non-ASCII gates. The static ASCII `String.prototype.concat` helper slice now has standalone run, JSON check, browser-targeted bundle source-class, JSON browser-bundle, bracketed-call, freeze-wrapper, and canonical dynamic/non-ASCII gate evidence; the static ASCII `String.prototype.split` helper slice now also has checker/browser-source-class evidence and standalone run indexed-element evidence while preserving dynamic/non-ASCII/limit gates. The static `Array.isArray` predicate slice now also folds statically-known array/object/primitive operands while preserving dynamic-argument gates, the static ASCII `String.fromCodePoint` helper now shares the existing `String.fromCharCode` 0–127 code-unit fold and gates broader code-point semantics, and the static numeric-only global `isFinite` / `isNaN` predicate slice now folds the direct `isFinite` and `globalThis` direct-runtime spellings, with checker coverage for transparent callable aliases, while preserving non-numeric and dynamic-argument gates. The static ASCII string case-conversion slice now includes the no-argument `toLocaleLowerCase` / `toLocaleUpperCase` forms while preserving locale-argument, non-ASCII, and dynamic-receiver gates. The static ASCII `String.prototype.normalize` helper slice now folds no-form and statically-known `NFC` / `NFD` / `NFKC` / `NFKD` forms as ASCII identity calls while preserving dynamic-form, invalid-form, extra-argument, and non-ASCII gates. Static string `.length` now folds statically-known string primitives and transparent `Object.freeze(...)` wrappers using UTF-16 code-unit counts with standalone run, JSON check, lowering, and browser-targeted bundle source-class evidence. The static ASCII `parseFloat` / `Number.parseFloat` integer-result slice now also has standalone run, JSON check, browser-targeted bundle source-class matrix, JSON browser-bundle, and canonical dynamic/non-ASCII/fractional/extra-argument/no-digit gate coverage.
- Keep non-literal dynamic import, broad reflection, eval-adjacent behavior, and unsupported dynamic language forms explicitly gated unless maturity rows are promoted.
- Grow bounded TS/JS inference only within deterministic budget rules, especially at exported/public boundaries and cross-module analysis.
- Maintain concise conformance dashboards that distinguish supported slices from tested gates.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete guest-facing threaded runtime semantics beyond profile acceptance and budget validation while preserving no-JIT, no-tracing-GC, and resource-budget honesty.
- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to a stable standalone browser runtime contract; specify host ownership, sandbox limits, summary behavior, and failure modes before any promotion.
- Add late host APIs only with explicit effect keys, policy behavior, resource budgets, command/API-surface gating, and JSON evidence.
- Triage late object/runtime APIs such as `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and broader object helpers against memory, threading, and optimization constraints before promotion.
- Keep browser-targeted build/check support, browser harness execution, and Kali-hosted sandbox enforcement distinct.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Expand package-corpus coverage by package shape, host/API context, source class, command, and support rung.
- Grow Node package support only where required Node built-ins and process APIs are supported or deliberately gated.
- Grow browser deployability and browser-harness package evidence while keeping standalone browser runtime claims separate.
- Keep native, binary, bootstrap-heavy, host-mismatched, and published-bin-entrypoint packages rejected by default unless specs introduce a mediated path.
- Keep registry-analysis commands on the schema-v1 single registry identifier contract unless a future schema revision defines batch, local, raw-URL, or package-set behavior.

## Optimization, PGO, and performance

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Deepen `release` and `release-advanced` optimizations while preserving JavaScript-visible semantics, sandbox effects, proof boundaries, and deterministic artifacts.
- Keep `--profile` as a deterministic build-only additive input; do not create a hidden fourth build mode.
- Promote performance wording only when benchmark evidence names workload, build mode, baseline, and reproducibility constraints.
- Keep optimization inventories as concise evidence snapshots rather than progress journals. The `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`).

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership, effects, type-system, and lowering slices in small named increments.
- Update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, diagnostics, source spans, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
