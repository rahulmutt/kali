# Stage 1.1 — Workspace & Crate Scaffold

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`SPEC.md`](../../SPEC.md), [`specs/01-architecture.md`](../../specs/01-architecture.md), [`specs/17-verification.md`](../../specs/17-verification.md)  
**Depends on:** none  
**Status:** ✅ Complete — workspace scaffolding, crate layout, mise/devenv workflows, CI baseline, and the proof-boundary baseline are all checked in

## Goal

Establish the repository shape that every later stage builds on: the Cargo workspace, the initial
crate split, the CLI entrypoint, the developer workflow bootstrap (`mise`, `devenv`, CI), and the
initial proof-boundary discipline.

## Workable Milestone

- `cargo build` succeeds for the workspace.
- The `kali` binary exists and reports a version.
- The core crate layout is in place (`kali_cli`, frontend/IR/codegen/runtime crates, package and
  sandbox crates, embedding crates, proof tree).
- `proofs/BOUNDARY.md` exists so the repository is at least **proof-ready** from the first stage.

## Current Repository State

Stage 1.1 is long closed and the repository has moved far beyond the original bootstrap:

- The Cargo workspace includes the CLI plus the language/frontend, IR, codegen, runtime, package,
  sandbox, embedding, optimization, and API-surface crates.
- `mise.toml`, `devenv.nix`, and `devenv.yaml` are present and define the canonical developer
  entrypoints used by the repo.
- `.github/workflows/ci.yml` includes the workspace test pipeline and the proof-check lane.
- `proofs/BOUNDARY.md` is no longer just a placeholder; the repository is now **proof-backed for
  the published boundary**.

## Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test --workspace` succeeds once the tracked documentation/status files are in sync
- ✅ `kali --version` is wired through the CLI binary target
- ✅ `proofs/BOUNDARY.md` exists and remains the canonical verification-claim boundary

## Remaining Follow-up

This stage has no unfinished implementation work of its own. The remaining work is maintenance:

- keep workspace/docs/CI wiring aligned with the actual crate tree,
- keep proof-boundary status docs synchronized with `proofs/BOUNDARY.md`, and
- keep new top-level developer workflows routed through canonical `mise` tasks.

## Definition of Done

- [x] Cargo workspace and crate layout are checked in
- [x] CLI binary target exists
- [x] Developer workflow bootstrap files exist (`mise`, `devenv`, CI)
- [x] `proofs/BOUNDARY.md` exists and anchors verification-claim discipline
- [x] Later stages can build on this scaffold without reorganizing the repository root
