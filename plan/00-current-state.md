# Current Implementation Baseline

This file records the planning baseline for the active continuation roadmap. It is a current-state summary, not an active checklist and not an availability matrix.

## Evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current public command set.
- The workspace contains Rust crates for CLI, lexer/parser, AST/HIR/MIR/LIR, type checking, codegen, runtime, sandbox/effects, package management, optimization, Deno/Web/Node API projections, embedding/C ABI, formatting, linting, and bindings.
- The spec-owned availability and current-state nuance remain in [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- The proof-backed boundary remains owned by [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

## Live surface at a glance

The checked-in repository already includes:

- CLI commands: `doctor`, `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`, `effects`, `package-effects`, and `package-audit`.
- Build/artifact lanes: executable builds, browser bundles, IR validation, library artifacts, C ABI artifacts, component artifacts, metadata, deterministic sidecars, and browser-bundle integer-like `Object.keys` / `Object.values` iteration coverage.
- Runtime object semantics: direct `Object.fromEntries([...])` operands now also participate in the static `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` helper slice in JS input, and static `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` lowering now also accepts `Object.fromEntries([...])` operands in the standalone run/test JS and TS smoke path; `for...of` array lowering now also accepts spread elements whose targets resolve to supported `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` slices in JS input, and the browser-targeted `check` / `build --bundle` smoke now also covers spread-of-`Object.values(...)` iterator slices in both `for...of` and `for await...of` forms on JS input plus spread-of-`Object.keys(...)` / `Object.entries(...)` iterator slices over `Object.fromEntries([...])` operands in JS input and const-bound `Object.keys(...)` iterator slices in `.js`, `.ts`, `.jsx`, and `.tsx` browser-targeted smoke plus the same const-bound `Object.keys(...)` slice in browser-requested `run` / `test` harness coverage; standalone `run` smoke now also covers the `for await` spread-of-`Object.values(...)` slice over `Object.fromEntries([...])` operands in JS input with output and JSON-output coverage. `Object.is` primitive-literal build smoke now also covers numeric, boolean, string, `null`, `Infinity`, `NaN`, and `-Infinity` slices across the Deno and browser build matrix.
- Runtime math semantics: `Math.pow` now also folds the zero-base / positive-integer-exponent slice on the supported checker/codegen path, and browser-harnessed `run` / `test` smoke now also covers the positive-integer-exponent alias-chain slice across JS, TS, JSX, and TSX input with JSON-output coverage; `Math.imul` now also accepts omitted trailing operands by folding zero-/one-argument slices to `0` on the supported checker/codegen path.
- Execution/reporting lanes: `run`, `test`, `test --coverage`, public source-graph effects, package effects, package audit, JSON envelopes, and schema-v1 payload validation.
- Host/API slices: default standalone Deno-oriented APIs, browser-targeted `check` / `build --bundle`, browser-harness-assisted execution where configured, and documented Node-compatible analysis/build/runtime slices.
- Package evidence: registry/raw-URL/install flows, package-corpus probes by context/rung, registry-analysis commands, and negative evidence for unsupported native/binary/bootstrap-heavy or published-bin-entrypoint cases.
- Embedding/evidence surfaces: public embedding artifacts and bindings metadata, optimization modes, deterministic PGO profile validation, benchmark fixtures, and Lean proof infrastructure for the published boundary.

## Planning consequence

The active plan starts from remaining spec gaps only. It does not preserve line-by-line progress notes for implemented slices; those belong in tests, specs, maturity current-state notes, and proof-boundary documents.

## Non-negotiable guardrails

Future phases must preserve:

- AOT-only guest-language compilation.
- Pure Rust implementation contract.
- No tracing/background GC.
- Sandbox-first honesty.
- Deterministic machine contracts.
- Public availability discipline from `specs/19-feature-maturity.md`.
- Proof-backed claim discipline from `proofs/BOUNDARY.md`.
