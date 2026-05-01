# Current Implementation Baseline

This file records the planning baseline for the active continuation roadmap. It is a current-state summary, not an active checklist and not an availability matrix.

## Repository evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current public command set.
- The workspace contains implementation crates for CLI, frontend, AST/HIR/MIR/LIR, codegen, runtime, sandbox/effects, package management, optimization, host APIs, embedding/C ABI, and bindings.
- The proof tree exists under `proofs/`, and `proofs/BOUNDARY.md` states that Kali is proof-backed for the published boundary.
- Canonical availability and current-state nuance remain in `specs/19-feature-maturity.md`.

## Live surface at a glance

The current repository has already implemented the historical MVP and several later surfaces:

- CLI commands: `doctor`, `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`, `effects`, `package-effects`, and `package-audit`.
- Build modes and artifacts: executable builds, browser bundles, `--validate-ir`, `--lib`, `--capi`, `--component`, and deterministic artifact metadata/sidecars.
- Execution/reporting: `run`, `test`, `test --coverage`, stable JSON envelopes, public effect reports, registry effect/audit reports, and schema-v1 payload validation.
- Host/API slices: Deno-oriented default surface, documented Node subsets, browser-targeted `check` / `build --bundle`, and browser-harness-assisted `run` / `test` paths where configured.
- Package evidence: default standalone, browser, Node, Deno/JSR, registry-analysis, and published-bin-entrypoint contrast coverage.
- Optimization evidence: real `fast`, `release`, and `release-advanced` optimization slices; deterministic PGO profile validation; version-pinned benchmark fixtures.
- Verification evidence: Lean proof tree and proof-backed status for the published boundary only.

## Planning consequence

Completed historical phase documents were removed from active planning. The active roadmap now starts at Phase 11 and focuses on remaining spec-owned work: language semantic closure, runtime/host expansion, package compatibility breadth, performance promotion, and verification/schema widening.

## Non-negotiable guardrails

Future phases must preserve:

- AOT-only guest-language compilation.
- Pure Rust implementation contract.
- No tracing/background GC.
- Sandbox-first honesty.
- Deterministic machine contracts.
- Public availability discipline from `specs/19-feature-maturity.md`.
- Proof-backed claim discipline from `proofs/BOUNDARY.md`.
