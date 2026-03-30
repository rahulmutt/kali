# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

## Read this repo in the right order
- [`BOOTSTRAP.md`](./BOOTSTRAP.md) — original input brief
- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — what is actually shipped in each phase
- owning chapter in [`specs/`](./specs) — subsystem details
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification claim boundary

Shortcuts:
- **Is this supported yet?** → `SPEC.md` → `specs/19-feature-maturity.md` → owning chapter
- **How does it work?** → owning chapter first, then `SPEC.md`
- **What proof coverage is claimed today?** → `proofs/BOUNDARY.md`

## Hard invariants
These are fixed unless the top-level spec changes:
- **AOT only** — no language-level JIT
- **Pure-Rust implementation contract** — no embedded C/C++ implementation dependencies
- **No tracing/background GC** — ownership/reference-counted strategies only where the owning chapters allow them
- **Sandbox-first honesty** — no overclaiming what Kali can actually mediate
- **Deterministic machine-readable contracts** — CLI output, diagnostics, and artifacts stay explicit and tool-friendly

## Phase 1 at a glance
Phase 1 is intentionally narrow. For exact boundaries, read the **Phase-1 Shipped Surface Summary** in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

- **Language/frontend**: `.ts` and `.js` are first-class inputs.
- **Project workflow**: `kali init`, `kali init --lib`, `kali install`, `kali fmt`, `kali lint`, and `kali check [files...]` are the main authoring loop.
- **Execution**: `kali run <file>` and `kali test [files...]` ship only in the default/inherited Deno-oriented standalone context, using wasmtime for Kali-hosted execution.
- **Builds**: `kali build <file>` ships as the default executable build in the shared Deno-oriented build context; `kali build --lib <file>` ships as the Phase-1 **base library artifact** for exact-version/internal consumers when Kali can determine a **statically known export surface**.
- **Browser support**: Phase 1 browser support is exactly the shared **Phase-1 browser-targeted command set** — browser-targeted `check [files...]` plus browser-targeted `build --bundle <file>` under an effective `apiSurface = browser`, including supported `--sandbox` variants and inherited-config equivalents.
- **Sandboxing/effects**: `run/test --sandbox` enforce at runtime; supported `check --sandbox`, default `build --sandbox`, `build --lib --sandbox`, and browser-targeted `build --bundle --sandbox` paths do static policy-schema/config validation only in Phase 1.
- **Packages**: early support is broad only inside the shared **pure JS/TS package contract** plus the documented raw-URL workflow.
- **Verification**: reuse the canonical repository summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.**

## Important normalization highlights
A few bootstrap asks are intentionally narrower after normalization:
- **“supports browser APIs”** means browser-targeted analysis/build first, not standalone browser `run`/`test`
- **“supports npm packages”** means support is bounded by package shape, host/API fit, and command maturity — not “everything without node-gyp works”
- **“supports all features including eval”** means parser acceptance and later compatibility planning now, but executable `eval`/`Function()` only in the later gated compatibility path
- **“static JSON effect reporting”** becomes two later public surfaces: reporting (`kali effects`, `kali package-effects`) and policy comparison (`check/build --sandbox`)
- **“embeddable / C API / WIT / Component Model”** means a Phase-1 base `--lib` artifact first, then the stable public embedding surface later

## Defined shape vs shipped availability
Some command families and artifact flows are documented before they ship so names and JSON schemas do not drift.

Examples:
- `kali effects` and `kali package-effects` are defined early but remain Phase-2 surfaces
- `kali package-audit` is defined early but is not a Phase-1 command
- plain `--lib` is documented early as the future stable public WIT-first path, but in Phase 1 it is still only the export-oriented **base library artifact**
- `kali build --capi` and `kali build --component` are defined early but remain later public embedding artifact flows

Rule of thumb:
- read **shape** from the owning chapter
- read **availability** from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)

## Repository posture
This repository is currently spec-first:
- the checked-in source of truth today is the spec set plus [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)
- example crate trees, CI layouts, Lean project layouts, and command examples describe the intended target shape, not necessarily files that already exist in the repo
- current verification claims must be read from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md), not inferred from roadmap prose

## Spec map
- Top-level normalization: [`SPEC.md`](./SPEC.md)
- Frontend/language: [`01 — Architecture`](./specs/01-architecture.md), [`02 — Lexer & Parser`](./specs/02-lexer-parser.md), [`03 — AST`](./specs/03-ast.md), [`04 — Type System`](./specs/04-type-system.md)
- Lowering/codegen: [`05 — Intermediate Representations`](./specs/05-ir.md), [`06 — Memory Management`](./specs/06-memory.md), [`07 — Optimization & Specialization`](./specs/07-specialization.md), [`08 — WebAssembly Code Generation`](./specs/08-wasm-codegen.md)
- Runtime/APIs/embedding: [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md), [`10 — Runtime`](./specs/10-runtime.md), [`11 — Standard APIs`](./specs/11-standard-apis.md), [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md)
- Tooling/evidence: [`12 — CLI`](./specs/12-cli.md), [`14 — Package Management`](./specs/14-packages.md), [`15 — Error Reporting`](./specs/15-errors.md), [`16 — Testing`](./specs/16-testing.md), [`17 — Formal Verification`](./specs/17-verification.md), [`18 — Schemas`](./specs/18-schemas.md), [`19 — Feature Maturity`](./specs/19-feature-maturity.md)
