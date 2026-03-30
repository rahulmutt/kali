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
- **No tracing/background GC** — deterministic ownership/reference-counted strategies only where the owning chapters allow them
- **Sandbox-first honesty** — no overclaiming what Kali can actually mediate
- **Deterministic machine-readable contracts** for CLI output, diagnostics, and artifacts

## Phase 1 snapshot
Phase 1 is intentionally narrow. Treat the bullets below as a quick overview only, and read exact shipped/not-shipped boundaries from the **Phase-1 Shipped Surface Summary** in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

- **Language/frontend**: `.ts` and `.js` are both first-class inputs in Phase 1. JavaScript is compiled through the same core pipeline with bounded conservative inference rather than a transpile-only compatibility lane.
- **Developer workflow**: `kali init`, `kali install`, `kali fmt`, `kali lint`, and `kali check [files...]` ship as the main Phase-1 project loop.
- **Deno-first execution/build**: standalone `kali run <file>`, `kali test [files...]`, and `kali build <file>` ship in the default/inherited Deno-oriented context; `kali build --lib <file>` also ships there, but only as the Phase-1 **base library artifact** for exact-version/internal consumers and only when Kali can determine a **statically known export surface**.
- **Browser and Node boundaries**: browser support is limited to the shared **Phase-1 browser-targeted command set** — browser-targeted `check [files...]` plus browser-targeted `build --bundle <file>`, including supported `--sandbox` variants and equivalent inherited-config forms when the effective `apiSurface` is `browser`. Standalone browser `run`/`test`, non-bundle browser builds, and broader `--api node` command paths are not shipped in Phase 1.
- **Sandboxing, effects, and packages**: `run/test --sandbox` enforce at runtime, while supported `check/build --sandbox` paths perform policy-schema/config validation only in Phase 1. That static Phase-1 build-time validation explicitly covers the default executable build path, the Phase-1 `build --lib` base-library path, and browser-targeted `build --bundle` in the shared **Phase-1 browser-targeted command set**. Internal effect bookkeeping may exist, but there is no stable public effect-report command yet (`kali effects` or `kali package-effects`). The separate single-package registry-analysis surface is also still gated, so `kali package-audit` is not shipped in Phase 1 either. Phase-1 package support is broad only inside the shared **pure JS/TS package contract**, plus the documented raw URL lock/cache workflow, and that support spans the Deno-first standalone path plus the shared **Phase-1 browser-targeted command set**.
- **Verification**: reuse the canonical summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.**

## Bootstrap normalization highlights
A few broad bootstrap asks are intentionally normalized into smaller cross-spec contracts:
- **“supports browser APIs”** means browser-targeted analysis/build first, not standalone browser `run`/`test`, and it does **not** mean Kali exposes one sandbox/effect key for every ambient DOM/browser API
- **“supports npm packages” / “supports non node-gyp packages”** means early support is scoped to the shared **pure JS/TS package contract** plus the documented raw-URL workflow, not a blanket promise that every package without `node-gyp` automatically works
- **“supports all features including eval”** means parser acceptance and later compatibility planning now, but executable `eval`/`Function()` only in the later gated compatibility path
- **“static JSON effect reporting”** means Phase 1 enforcement/policy validation first, with the later public effect surface split into reporting (`kali effects`, `kali package-effects`) and policy comparison (`check/build --sandbox`); schema v1 keeps the reporting commands explicit too, so `kali effects` is a one-root source-graph command and `kali package-effects` is a one-package registry-analysis command rather than a hidden project-discovery or dry-run workflow
- **`kali package-audit`** remains a separate later context-free registry-analysis/security-audit workflow, not part of the effect-reporting or policy-validation surface
- **“embeddable / C API / WIT / Component Model”** means a Phase-1 base `--lib` artifact first, then the stable public WIT-first embedding surface later

## Reading shortcuts
Use these shortcuts before interpreting any broad bootstrap aspiration as shipped support:
- command shape lives in [`specs/12-cli.md`](./specs/12-cli.md)
- package semantics live in [`specs/14-packages.md`](./specs/14-packages.md)
- diagnostics live in [`specs/15-errors.md`](./specs/15-errors.md)
- JSON/config/artifact schemas live in [`specs/18-schemas.md`](./specs/18-schemas.md)
- phase availability lives in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)

Useful normalized reminders:
- “supports browser APIs” does **not** mean standalone browser `run`/`test` or one sandbox/effect key per DOM/browser API
- package support should be read through the shared **package-support decision order** and **package-support ladder** in [`SPEC.md`](./SPEC.md): package shape first, then host/API fit, then command maturity
- browser-targeted package/build claims are usually **deployable-through-host**, not standalone-browser **executable** support
- the CLI is Deno-inspired at the workflow level, not a promise of flag-for-flag Deno parity
- documented command shape and shipped availability are separate; availability always comes from the maturity matrix
- later reporting/registry-analysis commands stay explicit in their targets: `kali effects` is direct-input and `kali package-effects` / `kali package-audit` take one explicit registry package target in schema v1
- later `kali package-effects` may inherit analysis context from defaults/discovered config, but that changes analysis semantics only; it does not let the current project pick a different package target/version or turn registry analysis into a project-installed-dependency workflow

## Repository posture
This repository is currently spec-first:
- the checked-in source of truth today is the spec set plus [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)
- example crate trees, CI layouts, Lean project layouts, and command examples in the spec describe the intended target shape, not necessarily files that already exist in the repo
- read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) for the exact current verification artifact and claim boundary; the Lean project tree described in the verification chapter is target-state documentation, not a guarantee that those proof files already exist in the repo

## Specification map
- Top-level normalization and terminology: [`SPEC.md`](./SPEC.md)
- Frontend and language design: [`01 — Architecture`](./specs/01-architecture.md), [`02 — Lexer & Parser`](./specs/02-lexer-parser.md), [`03 — AST`](./specs/03-ast.md), [`04 — Type System`](./specs/04-type-system.md)
- Lowering and codegen: [`05 — Intermediate Representations`](./specs/05-ir.md), [`06 — Memory Management`](./specs/06-memory.md), [`07 — Optimization & Specialization`](./specs/07-specialization.md), [`08 — WebAssembly Code Generation`](./specs/08-wasm-codegen.md)
- Runtime, sandboxing, APIs, embedding: [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md), [`10 — Runtime`](./specs/10-runtime.md), [`11 — Standard APIs`](./specs/11-standard-apis.md), [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md)
- Tooling and evidence: [`12 — CLI`](./specs/12-cli.md), [`14 — Package Management`](./specs/14-packages.md), [`15 — Error Reporting`](./specs/15-errors.md), [`16 — Testing`](./specs/16-testing.md), [`17 — Formal Verification`](./specs/17-verification.md), [`18 — Schemas`](./specs/18-schemas.md), [`19 — Feature Maturity`](./specs/19-feature-maturity.md)

