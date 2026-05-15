# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Promote parser-accepted but unavailable semantics only when checker, lowering, runtime, and conformance evidence exist.
- Implement full generator and async-generator semantics, including resumable state machines, `yield` / `yield*`, async interaction, error propagation, and finalization; until then keep unsupported forms behind canonical `E5506` gates.
- Expand `for...of` / `for await...of` from bounded static slices toward full iterator and async-iterator protocol behavior, including close/finalization/error semantics; current smoke now also includes the standalone TS `for await...of` string-concatenation slice plus browser-bundle/browser-harness coverage for the `for...of` template-literal slice over statically known string operands in JS, TS, JSX, and TSX input, in addition to the string-concatenation slice in JS, TS, JSX, and TSX input, browser-harness `Object.values([...Object.fromEntries(...)])` spread coverage over frozen operands, and browser iterator rejection coverage now also includes non-literal `Object.entries(...)` sources in the browser-targeted `check` / `build --bundle` lanes and their inherited browser-config forms, and the `globalThis["Reflect"]["ownKeys"]` alias spelling now also resolves on the supported `Reflect.ownKeys(...)` slice. Static `new Set(...)` constructor slices now also type-check, build, and execute on the supported iterable inputs, with browser-requested run/test browser-harness coverage now also spanning JS, TS, JSX, and TSX input, and static `new Map(...)` constructor slices now also ride the same supported input matrix; standalone run/test smoke now also covers the `for...of` template-literal slice in TS input with JSON-output coverage, and the constructor slices now also keep alias-chain / transparent-wrapper references covered through the bounded root-resolution path, including const-alias and parenthesized-alias `Set`/`Map` constructor coverage in runtime, browser-harness, codegen, and type-resolution smoke. The object-enumeration break/continue slice now also has standalone and browser-bundle smoke on the supported `Object.keys(...)` iteration path.
- Continue widening expression/operator, object-model, BigInt, Math, and dynamic-loading semantics where translation-safe; keep unsupported dynamic language forms explicitly gated. Recent work widened the static `Number.isInteger` / `Number.isSafeInteger` predicate slice alongside the existing `Number.isFinite` / `Number.isNaN` slice and now also keeps the mixed-bracket `globalThis.Number[...]` spellings covered in the smoke corpus. The type-resolution regression suite now also mirrors the `Number.isSafeInteger` bracketed alias family in JS input, browser-harness `Object.is` alias-chain coverage now also spans object/array same-reference and frozen-reference comparisons across the JS, TS, JSX, and TSX smoke matrix, including the bare `Object["is"]` root spelling on the browser-harness same-reference/frozen-reference path, the browser-harness `Object.values([...Object.fromEntries(...)])` spread slice now also covers frozen operands across the same input matrix, and standalone run/test smoke now also covers the `for await` spread-of-`Object.values` / `Object.keys` / `Object.entries` slice over `Object.fromEntries(...)` operands in JS and TS input with JSON-output coverage, the browser-requested run/test JSON smoke now mirrors that object/array/frozen-reference matrix across the same input set, the browser-requested `Math.atan2` smoke now also exercises the single-quoted bracketed zero slice across JS, TS, JSX, and TSX input with JSON-output coverage, and browser-harness/browser-bundle smoke now also covers that same single-quoted bracketed zero slice in TS and `.js` input, while browser-requested sequence-wrapped template-literal dynamic import smoke now also covers TS, JSX, and TSX input with JSON-output coverage, and type-resolution smoke now also keeps supported `Object.hasOwn(...)` and `Math.floor(...)` helper calls resolved through the same transparent wrapper path. BigInt arithmetic smoke now also covers subtraction and division slices in both TS and JS input. The browser-harness `Math.atan2` trailing-argument evaluation smoke now also has browser-bundle coverage in JS, TS, JSX, and TSX input with JSON-output coverage. Browser-requested browser-harness/browser-bundle smoke now also accepts const aliases that resolve to the supported `Object.is` and `Number.is*` callables, and the browser-targeted `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice now also accepts const aliases that resolve to those callables. Type-checker call targets wrapped in `as` or `satisfies` now resolve for direct callable references, and the wrapped-call-target corpus now also covers sequence-callable-target aliases on the documented Node `process.kill(0)` path; broader function-reference wrappers remain gated until that later compatibility path is promoted.
- Grow bounded TypeScript/JavaScript inference only within deterministic budget rules, especially at exported/public boundaries and cross-module analysis. The current regression corpus now also pins default-export alias-chain resolution across the source graph, including `export { default as ... }` re-export syntax, in the bounded export slice.
- Keep conformance dashboards concise: snapshots of supported/gated behavior, not implementation journals.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete guest-facing threaded runtime semantics beyond profile acceptance, while preserving no-GC/no-JIT invariants and resource-budget honesty.
- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to a stable standalone browser runtime contract; if yes, specify host ownership, sandbox limits, summary behavior, and failure modes first.
- Add late host APIs only with explicit effect keys, policy behavior, resource budgets, and command/API-surface gating: subprocesses, sockets/listeners, workers/threads, environment materialization, process control, and late Node/Deno modules.
- Triage late object/runtime APIs (`Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and broader object helpers) against memory, threading, and optimization constraints before promotion.
- Keep browser-targeted build/check support, browser harness execution, and Kali-hosted sandbox enforcement distinct.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Expand package-corpus coverage by package shape, host/API context, source class, command, and support rung without turning individual successes into blanket npm claims.
- Grow Node package support only where required Node built-ins and process APIs are explicitly supported or deliberately gated.
- Grow browser deployability and browser-harness package evidence while keeping standalone browser runtime claims separate.
- Keep native, binary, bootstrap-heavy, host-mismatched, and published-bin-entrypoint packages rejected by default unless specs introduce a mediated path.
- Keep registry-analysis commands (`package-effects`, `package-audit`) single-package and registry-identifier based unless a future spec/schema revision defines batch/local/raw-URL behavior; scheme-prefixed selectors such as `npm:lodash` stay rejected before lookup.

## Optimization, PGO, and performance

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Deepen `release` and `release-advanced` optimizations while preserving JavaScript-visible semantics, sandbox effects, proof boundaries, and deterministic artifacts.
- Treat `--profile` as a deterministic build-only additive input; do not create a hidden fourth build mode.
- Promote performance wording only when benchmark evidence names workload, build mode, baseline, and reproducibility constraints.
- Keep optimization inventories as concise evidence snapshots.

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership, effects, type-system, and lowering slices in small named increments.
- Update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, diagnostics, source spans, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
