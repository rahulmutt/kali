# Kali Specification

This document is the top-level roadmap for Kali and the canonical index into `specs/*.md`.

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, with three priorities:
1. **Fast compilation and fast generated code**
2. **Sandbox-first execution for untrusted and AI-generated programs**
3. **A machine-friendly developer experience**

It is implemented in Rust, avoids embedded C/C++ dependencies, emits WebAssembly ahead of time, and uses stronger static analysis than TypeScript where that improves safety, performance, or sandboxability.

## Core Product Definition

Kali should eventually provide:
- broad ECMAScript + TypeScript compatibility
- AOT compilation to WebAssembly only
- no tracing garbage collector
- deterministic ownership-based memory management
- aggressive specialization of generics and layouts
- static effect summaries for sandboxing
- runtime sandbox enforcement for dynamic behavior
- embeddable Rust and C APIs
- AI-friendly diagnostics and JSON schemas
- gradual formal verification in Lean 4

## Canonical Simplifications

To keep the project implementable, the spec set makes these simplifying choices:
- **Single linked artifact first**: early phases compile one resolved module graph into one linked WASM artifact.
- **wasmtime first**: Phases 1-3 standardize on wasmtime as the execution engine.
- **Library-first architecture**: the CLI is built on reusable crates; stable embedding surfaces follow once the core pipeline is solid.
- **Sandbox-first, proof-later**: runtime enforcement lands before full static effect-policy proofs.
- **Declarative policy first**: sandbox policy files stay data-only in the core phases; programmable validators are an embedding-oriented later extension.
- **Parse broad, enable narrowly**: the parser can accept syntax before the runtime/checker fully supports it; unsupported semantics must be gated explicitly.
- **Compatibility is phased**: "support everything" is the long-term goal, not the MVP promise.

## Delivery Phases

The canonical phase matrix lives in [specs/19-feature-maturity.md](specs/19-feature-maturity.md). At a high level:

### Phase 1 — Core compiler
Deliver a practically useful compiler/runtime with:
- lexer, parser, AST
- name resolution and baseline TypeScript-compatible checking
- HIR and LIR
- direct HIR → LIR lowering allowed
- simple WASM emission
- runtime sandbox enforcement and resource limits
- minimal Web + Deno host surface
- core CLI (`run`, `build`, `check`, `fmt`, `lint`, `test`, `install`)
- package support for pure JS/TS packages that fit the early host model

### Phase 2 — Ownership + effects
Add:
- MIR as the canonical ownership/layout IR
- escape analysis and deterministic memory management
- effect summaries and `kali effects`
- compile-time effect-vs-policy validation
- explicit `pure` / effect annotations
- first stable embedding surfaces

### Phase 3 — Specialization + ecosystem
Add:
- aggressive specialization and richer layout selection
- stronger optimizations and incremental compilation
- broader npm/CJS compatibility
- meaningful Node API support
- browser bundle workflow

### Phase 4 — Advanced compatibility
Add the hardest dynamic semantics:
- `eval` / `Function()` compatibility path
- harder dynamic loading modes
- deeper platform/API coverage
- broader proof coverage

## Spec Map

### 1. Frontend and semantic analysis
- [01 — Architecture](specs/01-architecture.md)
- [02 — Lexer & Parser](specs/02-lexer-parser.md)
- [03 — AST](specs/03-ast.md)
- [04 — Type System](specs/04-type-system.md)

### 2. IR, memory, optimization, codegen
- [05 — Intermediate Representations](specs/05-ir.md)
- [06 — Memory Management](specs/06-memory.md)
- [07 — Optimization & Specialization](specs/07-specialization.md)
- [08 — WebAssembly Code Generation](specs/08-wasm-codegen.md)

### 3. Runtime, sandboxing, host APIs
- [09 — Sandboxing & Effects](specs/09-sandboxing.md)
- [10 — Runtime](specs/10-runtime.md)
- [11 — Standard APIs](specs/11-standard-apis.md)

### 4. Tooling, embedding, ecosystem
- [12 — CLI](specs/12-cli.md)
- [13 — Embedding & C API](specs/13-embedding.md)
- [14 — Package Management](specs/14-packages.md)
- [15 — Error Reporting](specs/15-errors.md)

### 5. Validation and machine contracts
- [16 — Testing](specs/16-testing.md)
- [17 — Formal Verification](specs/17-verification.md)
- [18 — Schemas](specs/18-schemas.md)
- [19 — Feature Maturity](specs/19-feature-maturity.md)

## Cross-Cutting Rules

These rules override local ambiguity in individual chapters:
- **No JIT**: Kali is AOT-only at the language/compiler level.
- **No tracing GC**: ownership, escape analysis, stack allocation, unique ownership, and reference counting are the primary tools.
- **Pure Rust implementation**: no embedded C/C++ implementation dependency in the compiler/runtime stack.
- **Machine-readable stability**: JSON output and schemas are a first-class contract.
- **AI-friendly defaults**: concise success output, structured errors, explicit gating.
- **No silent fallback on unsupported semantics**: use the canonical feature-maturity diagnostic.

## Notable Improvements Applied While Consolidating

This top-level spec also resolves a few ambiguities across the existing chapters:
- **IR pipeline clarified**: MIR is canonical in Phase 2+, but Phase 1 may lower HIR directly to LIR.
- **Embedding clarified**: `kali build --capi` produces a WASM artifact plus generated embedding metadata/header; the native C ABI is provided by the host-side `kali_capi` library.
- **Phase language tightened**: embedding is library-first internally in Phase 1, with stable public embedding surfaces in Phase 2.
- **Feature gating centralized**: use [specs/19-feature-maturity.md](specs/19-feature-maturity.md) and `E5006` instead of repeating slightly different support claims.
- **No ad hoc compatibility flags**: if a dynamic feature needs a future opt-in path, define it once in the maturity matrix before referencing it elsewhere.

## How To Use This Spec Set

When making design or implementation decisions:
1. Start here.
2. Check [specs/19-feature-maturity.md](specs/19-feature-maturity.md) for the current support phase.
3. Follow the relevant detailed chapter.
4. If a machine-readable format is involved, use [specs/18-schemas.md](specs/18-schemas.md).
5. If two chapters seem to conflict, prefer the more constrained interpretation and update the spec set to restore one canonical answer.
