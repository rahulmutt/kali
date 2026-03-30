# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Early-phase headline assumptions:
- standalone execution is **Deno-first**
- browser support is **analysis/build first** (`check --api browser`, `build --bundle --api browser`)
- broader Node compatibility is a **later ecosystem phase**, not an implied MVP promise
- latest ECMA-262 grammar tracking does **not** imply blanket same-phase runtime support for every accepted feature
- “latest ECMA-262” means the **latest published edition**; draft / Stage-3+ proposal support is explicit and experimental rather than implied
- dynamic compatibility paths such as `eval` and `Function()` are part of the long-term contract, but remain explicitly phase-gated behind the single schema-v1 compatibility switch `eval`
- runtime/embedding behavior is standardized on **wasmtime first**; alternative WASM engines are a later extension, not an equal Phase-1 contract
- hard global constraints remain in force from the bootstrap brief: **AOT only**, **pure Rust**, and **no tracing/background GC**
- build artifact modes follow one canonical matrix: default executable, browser bundle, a Phase-1 **base library artifact**, and later Phase-2 **public embedding outputs** layered on that same exported-library contract
- public static effect-report commands (`kali effects`, `kali package-effects`) are a **Phase-2** surface; Phase 1 may already rely on internal effect bookkeeping for sandboxing, but that does not imply a stable user-facing JSON report yet
- package installation stays **context-agnostic** in early phases: one lock/install state serves the default Deno path and supported browser-targeted analysis/build paths, while final `exports`/`browser` edge selection happens at command time

Quick Phase-1 non-goals:
- no standalone `--api node` execution/checking yet
- no standalone browser runtime or browser-hosted `run` / `test`
- no `eval` / `Function()` support yet
- no threaded runtime profile yet
- no Phase-2 **public embedding outputs** yet: no stable public embedding ABI and no default WIT sidecars for plain `--lib`

## Specification
- Top-level overview, cross-spec simplification rules, canonical terminology, chapter ownership, chapter guide, artifact-mode matrix, bootstrap traceability, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Bootstrap-brief normalization rule: [SPEC.md#bootstrap-normalization-rule](./SPEC.md#bootstrap-normalization-rule)
- Cross-spec simplification rules: [SPEC.md#cross-spec-simplification-rules](./SPEC.md#cross-spec-simplification-rules)
- Bootstrap traceability table: [SPEC.md#bootstrap-traceability-matrix](./SPEC.md#bootstrap-traceability-matrix)
- Detailed chapter set: [`specs/`](./specs)
- Single source of truth for gated command/profile availability: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

Reading rule:
- treat `BOOTSTRAP.md` as the input brief and the spec set as the normative source of truth after normalization
- when a bootstrap aspiration and a phase-specific promise seem to differ, prefer `SPEC.md` plus the owning chapter and the feature-maturity matrix
- remember the two main naming splits used across the specs: config stores compatibility switches under `compat.features` while emitted reports use `compatFeatures`, and semantic effect kinds such as `FileSystem.Read` map onto policy/schema keys such as `effects.fileSystem.read`

Quick navigation:
- frontend and language design: [01 — Architecture](./specs/01-architecture.md), [02 — Lexer & Parser](./specs/02-lexer-parser.md), [03 — AST](./specs/03-ast.md), [04 — Type System](./specs/04-type-system.md)
- lowering, memory, optimization, and code generation: [05 — IR](./specs/05-ir.md), [06 — Memory Management](./specs/06-memory.md), [07 — Optimization & Specialization](./specs/07-specialization.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- sandboxing, runtime, APIs, and embedding: [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md), [11 — Standard APIs](./specs/11-standard-apis.md), [13 — Embedding](./specs/13-embedding.md)
- CLI, packages, diagnostics, schemas, testing, and verification: [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [15 — Errors](./specs/15-errors.md), [16 — Testing](./specs/16-testing.md), [17 — Formal Verification](./specs/17-verification.md), [18 — Schemas](./specs/18-schemas.md)

## Project posture
This repository is currently spec-first: the top-level spec and chapter set are the source of truth for scope, staging, and machine-readable contracts.

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
