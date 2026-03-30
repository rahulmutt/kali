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

- **Language/frontend**: `.ts` and `.js` are both first-class inputs in Phase 1. JavaScript goes through the same core pipeline with bounded conservative inference rather than a transpile-only compatibility lane.
- **Project workflow**: `kali init`, `kali install`, `kali fmt`, `kali lint`, and `kali check [files...]` ship as the main Phase-1 authoring loop. Dependency state remains intentionally single-owner: non-install commands fail on missing/stale dependency state instead of auto-repairing it.
- **Execution**: standalone `kali run <file>` and `kali test [files...]` ship only in the default/inherited Deno-oriented standalone context, standardized on wasmtime for Kali-hosted execution in early phases.
- **Executable build**: `kali build <file>` ships in the shared **Deno-oriented build context (schema v1)**, and `kali build --sandbox <policy> <file>` performs the same Phase-1 static policy-schema/config validation described in the maturity matrix.
- **Export-oriented build**: `kali build --lib <file>` also ships in that same build context as the Phase-1 **base library artifact** for exact-version/internal consumers when Kali can determine a **statically known export surface**. `kali build --lib --sandbox <policy> <file>` reuses the same Phase-1 static policy-schema/config validation path. This is useful immediately, but it is still **not** the stable public embedding surface.
- **Browser-targeted support**: browser support in Phase 1 is exactly the shared **Phase-1 browser-targeted command set** — browser-targeted `check [files...]` plus browser-targeted `build --bundle <file>`, including supported `--sandbox` variants and equivalent inherited-config forms when the effective `apiSurface` is `browser`. This is **checkable** / **deployable-through-host** support, not a standalone browser `run`/`test` contract.
- **Sandboxing and effects**: `run/test --sandbox` enforce at runtime. Supported `check/build --sandbox` paths perform policy-schema/config validation only in Phase 1, including the default executable build path, the Phase-1 `build --lib` path, and browser-targeted `build --bundle`. Internal effect bookkeeping may exist, but there is no stable public effect-report command yet (`kali effects` or `kali package-effects`).
- **Packages and registry analysis**: Phase-1 package support is broad only inside the shared **pure JS/TS package contract**, plus the documented raw URL lock/cache workflow, and those claims should be read through the support ladder from `SPEC.md`: early Deno-oriented claims may be **checkable/buildable/executable**, while early browser-targeted claims are usually **checkable** or **deployable-through-host**. npm lifecycle scripts remain opt-in via `kali install --allow-scripts` and do **not** imply early `--api node` support. The separate single-package registry-analysis surface is still gated, so `kali package-effects` and `kali package-audit` are not shipped in Phase 1.
- **Node boundary**: broader `--api node` command paths remain gated beyond Phase 1.
- **Verification**: reuse the canonical summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.**

### Defined vs shipped surfaces
Some command families and artifact flows are documented before they ship so names, JSON schemas, and CLI vocabulary do not drift. Documentation of shape is **not** the same thing as Phase-1 availability.

Examples:
- `kali effects` and `kali package-effects` are defined early so the public effect-report vocabulary is stable, but they are still Phase-2 surfaces.
- `kali package-audit` is documented early as the separate registry-analysis/security-audit lane, but it is not a Phase-1 command.
- the stable public plain `--lib` + default WIT contract is defined early so the final embedding vocabulary does not drift, but Phase 1 plain `--lib` is still only the export-oriented **base library artifact** for exact-version/internal consumers.
- `kali build --capi` and `kali build --component` are defined early so embedding vocabulary is stable, but they are still later public embedding artifact flows rather than extra Phase-1 build modes.

When in doubt, read command/artifact **shape** from the owning chapter and **availability** from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

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
- broad “support” wording is intentionally split across the shared ladder terms: **checkable**, **buildable**, **executable**, and **deployable-through-host**
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

