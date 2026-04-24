# Current Implementation Baseline

This file records the planning baseline observed before cleaning up the old completed plan phases.

## Repository evidence

- `cargo test --workspace` passes in the current workspace.
- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current CLI command set.
- The workspace contains implementation crates for CLI, frontend, IR, codegen, runtime, sandbox/effects, package management, optimization, host API surfaces, embedding, C ABI, and bindings.
- The proof tree exists under `proofs/` and `proofs/BOUNDARY.md` states that Kali is proof-backed for the published boundary.

## Live surface described by specs and README

The current repo has already implemented the historical MVP and several later surfaces:

- `kali init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`.
- `kali build --validate-ir`, `--bundle`, `--lib`, `--capi`, and `--component`.
- `kali test --coverage`.
- Runtime smoke coverage includes `Object.keys()`, `Object.entries()`, and `Object.values()` enumeration semantics alongside the earlier arithmetic/built-in regressions.
- The parser now accepts generator function syntax and `yield` expressions as AST forms, but typechecking still gates generator lowering with the canonical `E5506` path until a real lowering packet lands; that rejection is now covered on both the `check` and `build` CLI smoke paths.
- `kali effects`, `kali package-effects`, and `kali package-audit`.
- Deno-oriented baseline plus documented Node subsets for source-graph and effect/reporting workflows.
- Browser-targeted `check` and `build --bundle` remain distinct from standalone browser runtime support.
- `--compat eval` exists for the documented compatibility path.
- The threaded runtime profile is now accepted on supported `run`/`test` execution paths when explicitly opted in with `--wasm-threads`, and `check` / `build` / `effects` now also accept that opt-in on the supported non-browser analysis/build paths; positive `--max-threads` values are honored only under that opt-in.
- Schema-v1 outputs and diagnostic envelopes are first-class contracts.

## Planning consequence

The old phase documents were historical implementation records. Keeping them as active checklists caused stale references to completed tasks. Active planning now starts from the current baseline and focuses on remaining spec-owned work only.

## Non-negotiable guardrails

Future phases must preserve:

- AOT-only guest-language compilation.
- Pure Rust implementation contract.
- No tracing/background GC.
- Sandbox-first honesty.
- Deterministic machine contracts.
- Public availability discipline from `specs/19-feature-maturity.md`.
- Proof-backed claim discipline from `proofs/BOUNDARY.md`.
