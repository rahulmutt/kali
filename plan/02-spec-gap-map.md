# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

Keep this map high-level. Exact evidence belongs in tests, schemas, maturity current-state notes, and [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not in active plan prose.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Promote parser-accepted but unavailable semantics only when checker, lowering, runtime, and conformance evidence exist.
- Implement full generator and async-generator execution semantics, including resumable state machines, `yield`, `yield*`, `return`, `throw`, async interaction, and finalization; keep unsupported forms behind canonical gates until then. Current browser harness smoke also mirrors the sequence-wrapped generator and async-generator rejection on the yield* slice across JS, TS, JSX, and TSX input, and build/check smoke now also pins sequence-wrapped generator and async-generator class-expression rejection on the same gate, with browser harness `check`/`run`/`test`/`build` now also rejecting anonymous default-export generator and async-generator declarations across JS, TS, JSX, and TSX input.
- Expand iterator and async-iterator protocol behavior beyond bounded static slices, including lookup, `next` result handling, abrupt completion, close/finalization, and async protocol behavior; current smoke also covers parenthesized direct constructor aliases for the supported `Set` / `Map` slices such as `new (globalThis["Set"])` and `new (globalThis['Map'])`, and the frozen-callable constructor inventories now also carry nullish/logical wrapper aliases around the root and `globalThis` constructor forms.
- Continue widening expression/operator, object-model, BigInt, Math, Promise, dynamic-import, reflection, and built-in semantics where translation-safe; the current smoke now also covers the `Math.hypot()` perfect-square-sum slice in browser-requested run/test and browser-bundle JSX/TSX inputs, and browser-targeted build smoke now also covers the same slice in JSX and TSX inputs, and browser-targeted build smoke now also covers the `Math.floor` / `Math.trunc` / `Math.ceil` alias-chain and bracketed literal slices in JSX and TSX input on the browser API-surface path, and the frozen `Math.hypot` inventory now also carries the parenthesized bracket-root and single-quoted bracket-root aliases in the browser-harness/browser-bundle smoke, plus the parenthesized root-object `Object.freeze((globalThis["Math"]))["hypot"]` / `Object.freeze((globalThis['Math']))["hypot"]` aliases and the parenthesized dot-root `Object.freeze((globalThis.Math))["hypot"]` / `Object.freeze((globalThis.Math))['hypot']` aliases, and the `Math.cbrt` frozen-callable inventory now also carries the parenthesized receiver-wrapped bracket-root spellings around `globalThis["Math"]` and `globalThis['Math']`, plus the corresponding parenthesized dot-root bracket-access forms, and the Promise.any smoke now also includes the mixed-bracket frozen-callable alias family around `globalThis["Promise"]['any']` plus the parenthesized receiver-wrapped bracket-root aliases around `globalThis["Promise"]` / `globalThis['Promise']` and the dotted-root bracketed frozen-callable aliases around `globalThis.Promise["any"]` / `globalThis.Promise['any']`; Promise.allSettled smoke now also includes the parenthesized bracket-root frozen aliases around `globalThis["Promise"]` and the single-quoted sibling alias on the shared browser-harness/browser-bundle slice, and Promise.race smoke now also covers bracketed `globalThis["Promise"]` / `globalThis['Promise']` aliases plus frozen bracketed aliases on the shared browser body, and now also carries the corresponding fully bracketed `globalThis["Promise"]["race"]` / `globalThis['Promise']['race']` forms plus parenthesized frozen wrappers in the shared browser body.
- Array callback methods such as `find`, `findIndex`, `findLast`, `findLastIndex`, `map`, `filter`, `some`, `every`, `reduce`, `reduceRight`, and `flatMap` remain on the canonical `E5506` gate path in the direct runtime until faithful callback lowering is implemented; current lowering and CLI smoke now also pin the full rejection matrix across build/check/run/test entrypoints, and browser-requested run/test harness coverage now also pins the same gate in JS input.
- Keep non-literal dynamic import, broad reflection, eval-adjacent behavior, and unsupported dynamic language forms explicitly gated unless maturity rows are promoted.
- Grow bounded TS/JS inference only within deterministic budget rules, especially at exported/public boundaries and cross-module analysis.
- Maintain concise conformance dashboards that distinguish supported slices from tested gates.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete guest-facing threaded runtime semantics beyond profile acceptance and budget validation, while preserving no-JIT, no-tracing-GC, and resource-budget honesty.
- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to a stable standalone browser runtime contract; specify host ownership, sandbox limits, summary behavior, and failure modes before any promotion.
- Add late host APIs only with explicit effect keys, policy behavior, resource budgets, command/API-surface gating, and JSON evidence.
- Triage late object/runtime APIs such as `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and broader object helpers against memory, threading, and optimization constraints before promotion; the shared late-object-model inventory now also carries frozen bracket-root spellings for `WeakRef` and `FinalizationRegistry`, including the single-quoted variants, so the runtime/browser smoke remains aligned on those aliases across the browser JS/JSX/TSX fixture variants.
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
- Keep optimization inventories as concise evidence snapshots rather than progress journals, including the current `math-round-builtin` / `math-round-builtin-js` pair for `Math.round` and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`).

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership, effects, type-system, and lowering slices in small named increments.
- Update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, diagnostics, source spans, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
