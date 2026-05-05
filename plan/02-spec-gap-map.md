# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Promote parser-accepted but unavailable semantics only when checker, lowering, runtime, and conformance evidence exist.
- Implement full generator and async-generator semantics, or keep all unsupported forms behind canonical `E5506` gates.
- Expand `for...of` / `for await...of` from bounded static slices toward full iterator and async-iterator protocol behavior, including close/finalization/error semantics. Current browser-bundle coverage now also includes `Object.entries(...)` iteration over static object literals, the bounded array-iteration slice now also accepts const-identity aliases such as boolean/null/Infinity/NaN elements, the standalone runtime path now also covers `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` over static `Object.fromEntries(...)` operands, including duplicate-key overwrite cases, and browser-requested run/test browser-harness coverage now also mirrors the `Object.keys(...)` / `Object.entries(...)` spread slices over `Object.fromEntries(...)` operands in JSX and TSX input, while broader iterator-protocol handling remains gated.
- Continue widening expression/operator and built-in semantics where translation-safe; keep unsupported `Math`, object-model, dynamic import, compound-assignment, and dynamic language forms explicitly gated.
- Grow bounded TypeScript/JavaScript inference only within deterministic budget rules, especially around exported/public surfaces and cross-module analysis.
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
- Keep registry-analysis commands (`package-effects`, `package-audit`) single-package and registry-identifier based unless a future spec/schema revision defines batch/local/raw-URL behavior.

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
