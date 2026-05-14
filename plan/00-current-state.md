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
- Build/artifact lanes: executable builds, browser bundles, IR validation, library artifacts, C ABI artifacts, component artifacts, deterministic metadata, and sidecar manifests.
- Runtime/reporting lanes: `run`, `test`, `test --coverage`, source-graph effects, package effects, package audit, deterministic JSON envelopes, schema-v1 payload validation, bracketed `Object["keys"]` / `globalThis["Object"]["entries"]` object-enumeration lowering coverage, browser-harness sequence-wrapper coverage over the supported object-enumeration calls, direct `Reflect.ownKeys(...)` iterator lowering over static object-literal slices, the current string-concatenation iterable slice for `for...of` / `for await...of` over statically known string operands, plus the `for...of` template-literal slice over statically known string operands, and the `Object.is` slice for static primitive values, same-reference object/array aliases, `Object.freeze`-wrapped same-reference aliases, and distinct fresh object/array literal comparisons.
- Frontend metadata lanes: class generator methods now preserve async/generator flags in the parser, AST, and HIR while later phases still gate lowering with the canonical generator diagnostic.
- Host/API slices: default standalone Deno-oriented APIs, browser-targeted `check` / `build --bundle`, configured browser-harness execution, and documented Node-compatible analysis/build/runtime slices.
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
