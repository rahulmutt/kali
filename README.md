# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

## How to read this repository
- [`BOOTSTRAP.md`](./BOOTSTRAP.md) is the input brief, not the final normalized contract.
- [`SPEC.md`](./SPEC.md) owns cross-spec normalization, shared terminology, and conflict resolution.
- The owning chapter in [`specs/`](./specs) owns each subsystem's concrete contract.
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) owns availability and phase status.
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) owns the current verification claim boundary and proof-state wording.

Recommended reading paths:
- **To answer “is this supported yet?”** read [`SPEC.md`](./SPEC.md) → [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) → the owning chapter in [`specs/`](./specs)
- **To answer “how does the supported thing work?”** read the owning chapter first, then fall back to [`SPEC.md`](./SPEC.md) for shared rules and [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for phase gating
- **To answer “what proof coverage is actually claimed today?”** read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)

## Hard invariants
These stay fixed across phases unless the top-level spec is intentionally changed:
- **AOT only** — no language-level JIT
- **Pure-Rust implementation contract** — no embedded C/C++ implementation dependencies
- **no tracing/background GC**
- **sandbox-first honesty** — no overclaiming what Kali can actually mediate
- deterministic machine-readable contracts for CLI output, diagnostics, and artifacts

## Phase 1 snapshot
Phase 1 is intentionally narrow:
- **Deno-first** standalone execution
- browser support is limited to the shared **Phase-1 browser-targeted command set**: browser-targeted `check` (either the project-discovery no-file form or an explicit file set) plus its supported `--sandbox` variants, and browser-targeted `build --bundle <file>` plus its supported `--sandbox` variants, in both explicit-flag and equivalent inherited-config forms when the effective `apiSurface` is `browser`
- broader `--api node` support comes later
- internal effect bookkeeping may exist, but the stable public effect-report surface is later and intentionally split into a **reporting** half (`kali effects`, `kali package-effects`) and a **policy-comparison** half (compile/check-time inferred-effect-vs-policy validation on `check/build --sandbox`)
- `kali build --lib` ships only as a Phase-1 **base library artifact** for exact-version/internal consumers in the shared **Deno-oriented build context (schema v1)**; the stable public embedding surface comes later
- verification in Phase 1 is **proof-ready**, not automatically **proof-backed**; read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) for the exact current boundary and proof-state claim instead of inferring it from summary prose

For the compact shipped/not-shipped answer, see the **Phase-1 Shipped Surface Summary** in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

## Bootstrap normalization highlights
A few broad bootstrap asks are intentionally normalized into smaller cross-spec contracts:
- **“supports browser APIs”** means browser-targeted analysis/build first, not standalone browser `run`/`test`
- **“supports all features including eval”** means parser acceptance and later compatibility planning now, but executable `eval`/`Function()` only in the later gated compatibility path
- **“static JSON effect reporting”** means Phase 1 enforcement/policy validation first, with the stable public reporting surface opening later
- **“embeddable / C API / WIT / Component Model”** means a Phase-1 base `--lib` artifact first, then the stable public WIT-first embedding surface later

## Reading shortcuts
Use these shortcuts before interpreting any broad bootstrap aspiration as shipped support:
- command shape lives in [`specs/12-cli.md`](./specs/12-cli.md)
- package semantics live in [`specs/14-packages.md`](./specs/14-packages.md)
- diagnostics live in [`specs/15-errors.md`](./specs/15-errors.md)
- JSON/config/artifact schemas live in [`specs/18-schemas.md`](./specs/18-schemas.md)
- phase availability lives in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)

Useful normalized reminders:
- “supports browser APIs” does **not** mean standalone browser `run`/`test`
- package support should be read through the shared **package-support decision order** and **package-support ladder** in [`SPEC.md`](./SPEC.md)
- the CLI is Deno-inspired at the workflow level, not a promise of flag-for-flag Deno parity
- documented command shape and shipped availability are separate; availability always comes from the maturity matrix

## Repository posture
This repository is currently spec-first:
- the checked-in source of truth today is the spec set plus [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)
- example crate trees, CI layouts, Lean project layouts, and command examples in the spec describe the intended target shape, not necessarily files that already exist in the repo
- read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) for the exact current verification artifact and claim boundary; the Lean project tree described in the verification chapter is target-state documentation, not a guarantee that those proof files already exist in the repo

## Specification map
- Top-level normalization and terminology: [`SPEC.md`](./SPEC.md)
- Frontend and language design: [`01 — Architecture`](./specs/01-architecture.md), [`02 — Lexer & Parser`](./specs/02-lexer-parser.md), [`03 — AST`](./specs/03-ast.md), [`04 — Type System`](./specs/04-type-system.md)
- Lowering and codegen: [`05 — IR`](./specs/05-ir.md), [`06 — Memory Management`](./specs/06-memory.md), [`07 — Optimization & Specialization`](./specs/07-specialization.md), [`08 — WASM Codegen`](./specs/08-wasm-codegen.md)
- Runtime, sandboxing, APIs, embedding: [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md), [`10 — Runtime`](./specs/10-runtime.md), [`11 — Standard APIs`](./specs/11-standard-apis.md), [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md)
- Tooling and evidence: [`12 — CLI`](./specs/12-cli.md), [`14 — Package Management`](./specs/14-packages.md), [`15 — Errors`](./specs/15-errors.md), [`16 — Testing`](./specs/16-testing.md), [`17 — Formal Verification`](./specs/17-verification.md), [`18 — Schemas`](./specs/18-schemas.md), [`19 — Feature Maturity`](./specs/19-feature-maturity.md)

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
