# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Bootstrap-normalized headline assumptions:
- `BOOTSTRAP.md` is the input brief; [SPEC.md](./SPEC.md) plus [specs/19-feature-maturity.md](./specs/19-feature-maturity.md) are the normative source of truth after normalization
- hard invariants stay fixed across phases: **AOT only**, **pure Rust**, **no tracing/background GC**, **sandbox-first honesty**, and deterministic machine-readable contracts
- Phase 1 is intentionally narrow: **Deno-first** standalone execution plus the exact **Phase-1 browser-targeted command set** (`kali check [files...]` and `kali build --bundle <file>` when the effective `apiSurface` is `browser`, including supported `--sandbox` variants); broader Node support comes later
- stronger-than-`tsc` inference is still bounded: Kali improves local/obvious inference, but keeps an explicit annotation-required boundary instead of open-ended whole-program search
- latest ECMA-262 means the **latest published edition**; accepted grammar does not by itself imply same-phase runtime support for every feature
- the upstream project list in `BOOTSTRAP.md` is a **design-reference list**, not an architecture-copy or dependency promise
- early runtime standardization is **wasmtime first**; alternative engines are later extensions
- embedding is phased: Phase 1 ships a useful but unstable `kali build --lib` **base library artifact**; the stable public Rust/WIT/C ABI and Component Model surface is Phase 2
- effects are phased too: Phase 1 may use internal effect bookkeeping for sandboxing, but stable `kali effects` / `kali package-effects` are Phase 2 and `kali package-audit` is later compatibility
- verification is **proof-ready** before it is **proof-backed**: an empty published proof boundary is acceptable early, but releases must not market formal verification as shipped until that boundary names real modeled subsystems and theorem claims
- package installation stays context-agnostic in Phase 1, while package compatibility stays context-sensitive and limited to the shared **pure JS/TS package contract** under the **published-artifact-first package reading**

Quick Phase-1 non-goals:
- no general `--api node` command support yet across `check` / `effects` / `build` / `run` / `test`
- no standalone browser runtime or browser-hosted `run` / `test`
- no stable user-facing `kali effects`, `kali package-effects`, or `kali package-audit` workflow yet
- no `eval` / `Function()` support yet
- no threaded runtime profile yet
- no Phase-2 **public embedding surface** yet: no stable public Rust embedding API, no `--capi`, no `--component`, and no default WIT sidecars for plain `--lib`

Recommended Phase-1 implementation order:
1. frontend + checking foundation
2. deterministic install/package foundation
3. Deno-first Kali-hosted run/test foundation with sandbox enforcement
4. build outputs (`build`, browser bundle, Phase-1 `--lib`)
5. developer workflow polish (`check`, `fmt`, `lint`, diagnostics, JSON contracts)
6. evidence hardening (conformance, package corpus, browser smoke tests, determinism)

See the normative cross-spec version in [SPEC.md#recommended-phase-1-implementation-order](./SPEC.md#recommended-phase-1-implementation-order).

Quick support-reading checklist:
1. **What command shape is being asked for?** `build --bundle --api browser` and `run --api browser` are different requests.
2. **What rung of support is meant?** Use the shared **compatibility delivery ladder** in [SPEC.md](./SPEC.md): parser-accepted, checkable, buildable, executable, deployable-through-host, or policy/effect-modeled.
3. **What effective context is selected?** Read `apiSurface`, `runtimeProfiles`, `compat.features`, and any attached sandbox policy together.
4. **Which chapter owns the answer?** Command shape lives in `12-cli`, availability in `19-feature-maturity`, JSON shape in `18-schemas`, diagnostics in `15-errors`.

Use that order before treating any broad bootstrap aspiration as shipped support.

## Specification
- Top-level overview, implementation strata, cross-spec simplification rules, canonical terminology, chapter ownership, chapter guide, artifact-mode matrix, bootstrap traceability, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Bootstrap-brief normalization rule: [SPEC.md#bootstrap-normalization-rule](./SPEC.md#bootstrap-normalization-rule)
- Bootstrap triage rule for hard invariants vs phase-gated breadth: [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule)
- Cross-spec simplification rules: [SPEC.md#cross-spec-simplification-rules](./SPEC.md#cross-spec-simplification-rules)
- Bootstrap traceability table: [SPEC.md#bootstrap-traceability-matrix](./SPEC.md#bootstrap-traceability-matrix) *(includes triage bucket + earliest explicit phase promise for each bootstrap ask)*
- Detailed chapter set: [`specs/`](./specs)
- Single source of truth for gated command/profile availability: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

Reading rule:
- treat `BOOTSTRAP.md` as the input brief and the spec set as the normative source of truth after normalization
- when a bootstrap aspiration and a phase-specific promise seem to differ, prefer `SPEC.md` plus the owning chapter and the feature-maturity matrix
- when a support claim still feels ambiguous, use the shared **support-claim reading order** plus the **compatibility delivery ladder** in `SPEC.md` before assuming Kali means one undifferentiated notion of “support”
- remember the main naming splits used across the specs: config stores compatibility switches under `compat.features` while emitted reports use `compatFeatures`; cross-spec semantic axes use leaf names such as `apiSurface` / `buildMode` / `runtimeProfiles` while concrete `kali.json` storage uses paths such as `compilerOptions.apiSurface` / `compilerOptions.buildMode` / `compilerOptions.runtimeProfiles`; semantic effect kinds such as `FileSystem.Read` map onto policy/schema keys such as `effects.fileSystem.read`; and registry-package CLI/manifests/logical-root labels use the identifier spelling (`lodash`, `jsr:@std/path`) while structured JSON metadata uses the decomposed package-coordinate form (`registry`, `name`, `version`)
- for maintenance, keep the ownership split tight: command shape/flags live in `12-cli`, diagnostic semantics in `15-errors`, JSON field names in `18-schemas`, and phase availability in `19-feature-maturity`
- registry-analysis commands intentionally stay simpler than the canonical **source-graph commands** from [SPEC.md](./SPEC.md): `package-effects` inherits `apiSurface` / `runtimeProfiles` / `compat.features` from config/defaults rather than taking its own `--api` / `--wasm-threads` / `--compat` flag family, while `package-audit` is context-free in schema v1
- if that inherited `package-effects` context later resolves to `apiSurface = browser`, it reuses the same browser-targeted analysis context as other browser analysis commands, but only once `package-effects` itself exists; this still does **not** make it part of the exact Phase-1 browser-targeted command set

Quick navigation:
- frontend and language design: [01 — Architecture](./specs/01-architecture.md), [02 — Lexer & Parser](./specs/02-lexer-parser.md), [03 — AST](./specs/03-ast.md), [04 — Type System](./specs/04-type-system.md)
- lowering, memory, optimization, and code generation: [05 — IR](./specs/05-ir.md), [06 — Memory Management](./specs/06-memory.md), [07 — Optimization & Specialization](./specs/07-specialization.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- sandboxing, runtime, APIs, and embedding: [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md), [11 — Standard APIs](./specs/11-standard-apis.md), [13 — Embedding, WIT & C ABI](./specs/13-embedding.md)
- CLI, packages, diagnostics, schemas, testing, verification, and maturity: [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [15 — Errors](./specs/15-errors.md), [16 — Testing](./specs/16-testing.md), [17 — Formal Verification](./specs/17-verification.md), [18 — Schemas](./specs/18-schemas.md), [19 — Feature Maturity](./specs/19-feature-maturity.md)

## Project posture
This repository is currently spec-first: the top-level spec and chapter set are the source of truth for scope, staging, and machine-readable contracts.

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
