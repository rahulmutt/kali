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

README scope note:
- this file is overview-first and intentionally non-normative
- `SPEC.md` owns shared terminology/normalization, `specs/19-feature-maturity.md` owns shipped availability, owning chapters own subsystem details, and `proofs/BOUNDARY.md` owns the repository's current verification-claim boundary
- when README summary bullets and the detailed specs differ, prefer the specs

## Hard invariants
These are fixed unless the top-level spec changes:
- **Guest-language AOT only** — no language-level JIT; host-engine WASM translation remains an execution detail rather than a second Kali compilation tier
- **Pure-Rust implementation contract** — no embedded C/C++ implementation dependencies
- **No tracing/background GC** — ownership/reference-counted strategies only where the owning chapters allow them
- **Sandbox-first honesty** — no overclaiming what Kali can actually mediate
- **Deterministic machine-readable contracts** — CLI output, diagnostics, and artifacts stay explicit and tool-friendly

## Phase 1 at a glance
Phase 1 is intentionally narrow. For exact boundaries, read the **Phase-1 Shipped Surface Summary** in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

- **Language/frontend**: `.ts` and `.js` are first-class inputs; Phase 1 JavaScript support is the bounded-inference path, not a downgraded parse-only compatibility mode.
- **Project workflow**: `kali init`, `kali init --lib`, `kali install`, `kali fmt`, `kali lint`, and `kali check [files...]` are the main authoring loop.
- **Execution**: `kali run <file>` and `kali test [files...]` ship only in the default/inherited Deno-oriented standalone context, using wasmtime for Kali-hosted execution. Production/embedding flows should prefer engine precompilation where avoiding launch-time translation matters.
- **Builds**: `kali build <file>` ships as the default executable build in the shared **Deno-oriented build context (schema v1)**; `kali build --lib <file>` ships as the Phase-1 **base library artifact** for **exact-version consumers** when Kali can determine a **statically known export surface**. Here, the Deno-oriented build context is the build/analysis default, not a claim that Phase-1 library outputs expose a Deno-specific public ABI.
- **Browser support**: Phase 1 browser support is exactly the shared **Phase-1 browser-targeted command set** from [`SPEC.md`](./SPEC.md), including equivalent inherited-config forms when the effective `apiSurface` resolves to `browser`.
- **Sandboxing/effects**: `run/test --sandbox` enforce at runtime; the shared **Phase-1 static policy-validation surface** from [`SPEC.md`](./SPEC.md) does static policy-schema/config validation only in Phase 1; later public effect reporting stays on the explicit `kali effects <file>` / `kali package-effects <package>` commands rather than a `run/test --dry` side path.
- **Packages**: early support is broad only inside the shared **pure JS/TS package contract** plus the documented raw-URL workflow, and every package claim should still be read through the same order: package shape → host/API fit → command maturity → exact support rung.
- **Registry analysis**: no stable public `kali package-effects` or `kali package-audit` command ships in Phase 1. `kali package-effects` is intentionally dual-classified once it does ship: it is a registry-analysis command by input shape and part of the public effect-report surface by output contract.
- **Verification**: reuse the canonical repository summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.**

## Important normalization highlights
A few bootstrap asks are intentionally narrower after normalization:
- **“supports browser APIs”** means browser-targeted analysis/build first, not standalone browser `run`/`test`
- **“supports npm packages”** means support is bounded by package shape, host/API fit, command maturity, and the exact support rung being claimed (`installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`) — not “everything without node-gyp works”
- **“supports all features including eval”** means parser acceptance and later compatibility planning now, but executable `eval`/`Function()` only in the later gated compatibility path
- **“static JSON effect reporting”** becomes two later public surfaces: reporting (`kali effects <file>`, `kali package-effects <package>`) and policy comparison (`check/build --sandbox`); when reporting ships, those reports are conservative upper bounds rather than exact execution traces, and `kali effects` stays a one-root reporting command rather than a hidden project-discovery or `run --dry` mode
- **“embeddable / C API / WIT / Component Model”** means a Phase-1 base `--lib` artifact first, then the stable public embedding surface later, with plain public `--lib` + WIT as the canonical contract and `--capi` / `--component` as explicit projections over that same export surface
- **“take inspiration from Boa / V8 / JavaScriptCore / SpiderMonkey / Deno / tsc / Porffor / Hermes / Bun”** means design references and benchmarking inputs, not architecture-copy promises, compatibility targets, or permission to pull in non-Rust implementation dependencies
- **“latest ECMA-262 support”** means the latest **published** ECMA-262 edition for shipped parser/semantic support claims; draft or Stage-3+ proposal semantics stay explicitly gated instead of being implied by that headline

## Defined shape vs shipped availability
Some command families and artifact flows are documented before they ship so names and JSON schemas do not drift.

Examples:
- `kali effects` and `kali package-effects` are defined early but remain Phase-2 surfaces
- `kali package-audit` is defined early but is not a Phase-1 command
- plain `--lib` is documented early as the future stable public WIT-first path, but in Phase 1 it is still only the export-oriented **base library artifact**
- `kali build --capi` and `kali build --component` are defined early but remain later public embedding artifact flows
- browser-targeted `check` / `build --bundle` availability does not imply standalone browser `run` / `test`

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
