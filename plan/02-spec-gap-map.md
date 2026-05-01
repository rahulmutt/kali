# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Implement or deliberately keep gated high-cost language semantics that are still unavailable despite parser/diagnostic coverage: generator lowering, broader `for...of` / iterator lowering beyond the supported literal-array slice (literal elements and simple variable bindings), broader `for await...of` / async-iterator semantics beyond that same literal-array slice, the remaining unsupported `Math` members (with `Math.floor` now also constant-folding statically-known numeric literals, while the statically-known perfect-square integer literal path for `Math.sqrt` and the statically-known perfect-cube integer literal path for `Math.cbrt` stay covered on the current integer-only subset), and non-literal `import(expr)`.
- Expand TypeScript/JavaScript inference only inside the bounded inference contract; keep open-ended public-API and cross-module solving gated until deterministic budgets and evidence exist.
- Continue turning parser-only acceptance into either faithful runtime/checker support or explicit canonical gates (`E5506`) with mirrored TS, JS, JSX, TSX, browser, Node, and JSON-output regressions where applicable.
- Maintain and simplify conformance dashboards as snapshots of supported/gated semantics rather than progress logs.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete threaded-runtime semantics beyond opt-in profile acceptance and host-import plumbing, including guest-facing multi-worker/thread behavior where the spec permits it.
- Decide whether `run --api browser` / `test --api browser` should graduate from harness-assisted later compatibility to a stable standalone browser runtime contract; if yes, specify host ownership, sandbox limits, summary JSON behavior, and failure modes first.
- Add late host APIs only with explicit policy/effect/resource contracts: environment materialization/mutation, process cwd/chdir/exit, subprocess spawning, socket/listener APIs, and late Node built-ins.
- Triage late object/runtime APIs (`Proxy`, own-property helpers, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`) with conformance and no-GC/no-JIT compatibility evidence before promotion.
- Keep browser-targeted build/check support, browser harness execution, and Kali-hosted sandbox enforcement distinct.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Expand package-corpus coverage by host/API context and support rung without turning individual package successes into blanket npm claims.
- Grow Node package support only where the required Node built-ins and process APIs are explicitly supported or deliberately gated.
- Grow browser deployability and browser-harness package evidence while keeping published binary entrypoints and native/bootstrap-heavy packages rejected by default.
- Keep registry-analysis commands (`package-effects`, `package-audit`) single-package and registry-identifier based unless a future spec revision defines batch/local/raw-URL behavior.

## Optimization, PGO, and performance

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Deepen `release` and `release-advanced` optimizations while preserving JavaScript-visible semantics, sandbox effects, and deterministic artifacts.
- Treat `--profile` as a deterministic build-only additive input; do not create a hidden fourth build mode.
- Promote performance wording only when benchmark evidence names workload, build mode, baseline, and reproducibility constraints.
- Keep optimization inventories as concise current-evidence snapshots, not implementation journals.

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership/effects/lowering in small named slices, and update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
