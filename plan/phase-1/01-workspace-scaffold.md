# Stage 1.1 — Workspace & Crate Scaffold

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/01-architecture.md`](../../specs/01-architecture.md), [`specs/17-verification.md`](../../specs/17-verification.md), [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md)

## Goal

Establish the Cargo workspace, crate boundaries, CI skeleton, and proof-ready repository baseline
so that every subsequent stage has a stable structural foundation to build on.

At the end of this stage `cargo build` succeeds, `kali --version` prints a version string, and the
repository is already **proof-ready** (even though no mechanized proofs exist yet).

## Workable Milestone

- `cargo build --workspace` completes without errors.
- `kali --version` prints the canonical version string and exits 0.
- CI runs `cargo test --workspace`, `cargo clippy`, and `cargo fmt --check` on every commit.
- `proofs/BOUNDARY.md` exists with the placeholder proof-boundary manifest and an explicit
  proof-CI trigger policy; the repository may claim **proof-ready**, not **proof-backed**.

## Tasks

### 1. Cargo workspace

Create the top-level `Cargo.toml` workspace manifest listing every crate defined in the target
architecture from `specs/01-architecture.md`:

```
kali_common / kali_error / kali_lexer / kali_parser / kali_ast /
kali_types / kali_hir / kali_mir / kali_lir / kali_codegen /
kali_optimize / kali_sandbox / kali_runtime / kali_api_deno /
kali_api_node / kali_api_web / kali_fmt / kali_lint / kali_cli /
kali_embed / kali_capi / kali_npm
```

Each crate starts as a stub (`lib.rs` with a single `pub fn placeholder() {}`). This lets the
workspace compile immediately while later stages fill in real implementations.

### 2. Shared utilities crate (`kali_common`)

Implement the first real functionality:

- **String interner** — a global, thread-safe `Interner` backed by a concurrent hash map;
  identifiers and string literals are interned throughout the pipeline.
- **Source file registry** — assigns a compact `FileId` to each loaded source file.
- **`Span` type** — `(FileId, start_byte: u32, end_byte: u32)`; cheaply copyable, used by every
  subsequent AST/IR node.
- **`SourceMap`** — maps a `Span` back to human-readable `(file, line, column)` for diagnostics.

### 3. Diagnostic skeleton (`kali_error`)

Introduce the `Diagnostic` type and the `Diagnostics` collector that accumulates errors/warnings
without aborting compilation (resilient-compilation strategy from `specs/01-architecture.md`).

Define the top-level error-code namespaces (`E1xxx` lex, `E2xxx` parse, `E3xxx` type, `E4xxx`
runtime, `E5xxx` CLI/command-shape) as empty enums — concrete codes land in later stages.

### 4. CLI binary stub (`kali_cli`)

Wire a minimal `clap`-based CLI binary that accepts `--version` / `--help` and exits cleanly.
No subcommands yet. The binary links all workspace crates so the dependency graph is validated
from the first CI run.

Build mode flags (`--fast`, `--release`, `--release-advanced`) and API-surface flags (`--api`) are
registered as *defined-but-not-yet-active* at this stage so their names are reserved from the start.

### 5. CI pipeline

Configure CI (e.g. GitHub Actions) with the following jobs:

| Job | Command | Required to pass? |
|---|---|---|
| `build` | `cargo build --workspace` | yes |
| `test` | `cargo test --workspace` | yes |
| `clippy` | `cargo clippy --workspace -- -D warnings` | yes |
| `fmt` | `cargo fmt --workspace --check` | yes |
| `proof-trigger` | placeholder — runs if `proofs/` changes | yes (no-op passes) |

### 6. Proof-ready baseline

Create `proofs/BOUNDARY.md` with the placeholder proof-boundary manifest:

- Current boundary: **empty** (no mechanized coverage yet).
- Proof-CI trigger policy: CI must run Lean proof jobs when any file under `proofs/` changes; in
  the placeholder state those jobs are trivially-passing stubs.
- Canonical short summary (must be verbatim in any repository summary that mentions verification):
  **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.**

### 7. Repository housekeeping

- `README.md` — one-paragraph description referencing `SPEC.md` and `PLAN.md`; verification status
  quoted verbatim from `proofs/BOUNDARY.md`.
- `LICENSE` — confirm license file is present.
- `.gitignore` — exclude `target/`, editor artefacts, and any generated proof object files.

## Out of Scope

- Actual lexing, parsing, or type-checking logic (Stages 1.2–1.5).
- Any user-visible `kali` subcommands beyond `--version` / `--help`.
- MIR crate implementation (Phase 2 target; stub only here).
- `kali_embed` / `kali_capi` public API (Phase 2 target; stub only here).
- `kali_api_node` functional implementation (Phase 3 target; stub only here).

## Definition of Done

- [ ] `cargo build --workspace` passes.
- [ ] `cargo test --workspace` passes (stubs have trivial passing tests).
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] `kali --version` prints version and exits 0.
- [ ] `proofs/BOUNDARY.md` exists with placeholder manifest and proof-CI trigger policy.
- [ ] CI pipeline runs and all jobs pass.
