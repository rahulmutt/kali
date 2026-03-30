# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Bootstrap-normalized headline assumptions:
- use the triage from [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule): first separate **hard invariants** from **phase contracts** and **phase-gated breadth targets**, then read the owning chapter plus the maturity matrix
- hard invariants from the bootstrap brief stay fixed across the early plan: **AOT only**, **pure Rust**, **no tracing/background GC**, **sandbox-first honesty**, and deterministic machine-readable contracts
- host support follows one small staircase: shared **Web baseline**, **Deno-first** standalone execution, **browser-targeted** analysis/build support (`check --api browser`, `build --bundle --api browser`), and broader **Node compatibility** as a later ecosystem phase
- latest ECMA-262 tracking means the **latest published edition**; grammar support does **not** imply blanket same-phase runtime support for every accepted feature, and draft / Stage-3+ proposal support is explicit and experimental rather than implied
- dynamic compatibility paths such as `eval` and `Function()` are part of the long-term contract, but remain explicitly phase-gated behind the single schema-v1 compatibility switch `eval`
- runtime/embedding behavior is standardized on **wasmtime first**; alternative WASM engines are a later extension, not an equal Phase-1 contract
- build artifact modes follow one canonical matrix: default executable compile intent, browser-bundle executable compile intent, a Phase-1 **base library artifact** for library compile intent, and later Phase-2 **public embedding surface** milestones layered on that same exported-library contract
- public static effect-report commands (`kali effects`, `kali package-effects`) are a **Phase-2** surface; Phase 1 may already rely on internal effect bookkeeping for sandboxing, but that does not imply a stable user-facing JSON report yet
- package installation stays **context-agnostic** in early phases: one lock/install state serves the default Deno path and supported browser-targeted analysis/build paths, while final `exports`/`browser` edge selection happens at command time
- package compatibility in Phase 1 stays inside the shared **pure JS/TS package contract**; `--allow-scripts` does not widen support to the excluded **native/binary/bootstrap-heavy package contract**

Quick Phase-1 non-goals:
- no general `--api node` command support yet across `check` / `effects` / `build` / `run` / `test`
- no standalone browser runtime or browser-hosted `run` / `test`
- no `eval` / `Function()` support yet
- no threaded runtime profile yet
- no Phase-2 **public embedding surface** yet: no stable public Rust embedding API, no `--capi`, no `--component`, and no default WIT sidecars for plain `--lib`

## Specification
- Top-level overview, cross-spec simplification rules, canonical terminology, chapter ownership, chapter guide, artifact-mode matrix, bootstrap traceability, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Bootstrap-brief normalization rule: [SPEC.md#bootstrap-normalization-rule](./SPEC.md#bootstrap-normalization-rule)
- Bootstrap triage rule for hard invariants vs phase-gated breadth: [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule)
- Cross-spec simplification rules: [SPEC.md#cross-spec-simplification-rules](./SPEC.md#cross-spec-simplification-rules)
- Bootstrap traceability table: [SPEC.md#bootstrap-traceability-matrix](./SPEC.md#bootstrap-traceability-matrix)
- Detailed chapter set: [`specs/`](./specs)
- Single source of truth for gated command/profile availability: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

Reading rule:
- treat `BOOTSTRAP.md` as the input brief and the spec set as the normative source of truth after normalization
- when a bootstrap aspiration and a phase-specific promise seem to differ, prefer `SPEC.md` plus the owning chapter and the feature-maturity matrix
- remember the three main naming splits used across the specs: config stores compatibility switches under `compat.features` while emitted reports use `compatFeatures`; semantic effect kinds such as `FileSystem.Read` map onto policy/schema keys such as `effects.fileSystem.read`; and registry-package CLI/manifests/logical-root labels use the identifier spelling (`lodash`, `jsr:@std/path`) while structured JSON metadata uses the decomposed package-coordinate form (`registry`, `name`, `version`)
- for maintenance, keep the ownership split tight: command shape/flags live in `12-cli`, diagnostic semantics in `15-errors`, JSON field names in `18-schemas`, and phase availability in `19-feature-maturity`

Quick navigation:
- frontend and language design: [01 — Architecture](./specs/01-architecture.md), [02 — Lexer & Parser](./specs/02-lexer-parser.md), [03 — AST](./specs/03-ast.md), [04 — Type System](./specs/04-type-system.md)
- lowering, memory, optimization, and code generation: [05 — IR](./specs/05-ir.md), [06 — Memory Management](./specs/06-memory.md), [07 — Optimization & Specialization](./specs/07-specialization.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- sandboxing, runtime, APIs, and embedding: [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md), [11 — Standard APIs](./specs/11-standard-apis.md), [13 — Embedding](./specs/13-embedding.md)
- CLI, packages, diagnostics, schemas, testing, and verification: [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [15 — Errors](./specs/15-errors.md), [16 — Testing](./specs/16-testing.md), [17 — Formal Verification](./specs/17-verification.md), [18 — Schemas](./specs/18-schemas.md)

## Project posture
This repository is currently spec-first: the top-level spec and chapter set are the source of truth for scope, staging, and machine-readable contracts.

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
