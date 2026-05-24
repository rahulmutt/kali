# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

Keep this map high-level. Exact evidence belongs in tests, schemas, maturity current-state notes, and [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not in active plan prose.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Promote parser-accepted but unavailable semantics only when checker, lowering, runtime, and conformance evidence exist.
- Implement full generator and async-generator execution semantics, including resumable state machines, `yield`, `yield*`, `return`, `throw`, async interaction, and finalization; keep unsupported forms behind canonical gates until then.
- Expand iterator and async-iterator protocol behavior beyond bounded static slices, including lookup, `next` result handling, abrupt completion, close/finalization, and async protocol behavior.
- Extend the supported object-enumeration helper slices with frozen `Object.entries` helper calls.
- Continue widening expression/operator, object-model, BigInt, Math, Promise, dynamic-import, reflection, and built-in semantics where translation-safe.
- Array callback methods such as `find`, `findLast`, `map`, `filter`, `some`, `every`, and their index/reduction variants remain on the canonical `E5506` gate path in the direct runtime until faithful callback lowering is implemented.
- Keep non-literal dynamic import, broad reflection, eval-adjacent behavior, and unsupported dynamic language forms explicitly gated unless maturity rows are promoted.
- Grow bounded TS/JS inference only within deterministic budget rules, especially at exported/public boundaries and cross-module analysis.
- Maintain concise conformance dashboards that distinguish supported slices from tested gates.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete guest-facing threaded runtime semantics beyond profile acceptance and budget validation, while preserving no-JIT, no-tracing-GC, and resource-budget honesty.
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
- Keep optimization inventories as concise evidence snapshots rather than progress journals, including the current `math-round-builtin` / `math-round-builtin-js` pair for `Math.round` and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`).

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership, effects, type-system, and lowering slices in small named increments.
- Update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, diagnostics, source spans, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
