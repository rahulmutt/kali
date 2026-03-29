# Kali — Specification

Kali is an ahead-of-time TypeScript/JavaScript compiler targeting WebAssembly, built in Rust. It is primarily a TypeScript/JavaScript implementation and runtime rather than a brand-new language: Kali-specific surface syntax is intentionally kept small in the early phases. It extends TypeScript's type system with Hindley-Milner-style inference and an effect system, enabling sandboxed execution with compile-time-directed memory management (no tracing garbage collector). Designed as a secure, embeddable runtime for AI-generated code.

## Design Principles

1. **AOT only** — Kali itself performs no speculative or adaptive JIT compilation. The language compiler always compiles ahead of time to WebAssembly.
2. **No GC** — Ownership and lifetime analysis at compile time (stack vs heap, deterministic reference counting when needed). No runtime tracing collector.
3. **Sandbox-first** — Static effect analysis and runtime resource constraints.
4. **AI-native** — Token-efficient CLI output, machine-parseable errors, static effect JSON export.
5. **Embeddable** — C API for integration from any language.
6. **Pure Rust** — No C/C++ library dependencies are embedded or linked.
7. **Spec-compliant over time** — Latest ECMA-262 support, including dynamic features such as `eval`, is a compatibility goal reached incrementally through tracked implementation phases.
8. **Fast by default** — Lexing, parsing, type-checking, and codegen prioritize low latency first; more expensive optimizations are opt-in.
9. **Superset of TypeScript** — Kali extends TypeScript with stronger inference and effect tracking while keeping Kali-specific surface syntax as small as possible in the initial implementation.
10. **Single-threaded by default** — One event loop per runtime instance. Threads opt-in via WASM threads for `SharedArrayBuffer`/`Atomics`.

## Delivery Phases

To keep the project implementable, the spec is phased:

1. **Phase 1 — Core compiler**: parser, AST, TypeScript-compatible checking, baseline WebAssembly emission, core runtime, AI-friendly CLI.
2. **Phase 2 — Ownership + effects**: compile-time allocation decisions, static effect summaries, sandbox policy validation, embeddable APIs.
3. **Phase 3 — Specialization + ecosystem**: generic specialization, Node/Deno/browser compatibility layers, npm interoperability, better optimization.
4. **Phase 4 — Advanced features**: full `eval`, experimental algebraic effects/handlers, broader formal verification, deeper compatibility work.

When a section describes an advanced capability, it should be interpreted as a target architecture unless explicitly marked as required for the earlier phases.

## Cross-Spec Terminology

- **Linked artifact** — the single WASM build artifact produced from the resolved static module graph in Phases 1-3.
- **Compatibility mode** — an explicit opt-in path for dynamic or semantically expensive JavaScript features that are parsed earlier than they are fully supported at runtime.
- **Dynamic effect boundary** — a construct such as non-literal `import(expr)`, `eval`, or similar reflective behavior that makes static effect analysis conservative/incomplete and therefore requires runtime enforcement.

## Resolved Design Decisions

These decisions apply across all detailed specs and are intended to remove ambiguity:

1. **Sandbox policy format**: `kali.policy.json` is the canonical policy format. Policies are declarative by default; executable validators are an embedding-only later-phase extension.
2. **Artifact model**: early builds compile one static module graph into one linked WASM artifact. Runtime module linking is not part of the MVP.
3. **Runtime engine**: `wasmtime` is the required execution engine for Phases 1-3. Alternative engines are deferred.
4. **`eval` status**: parsed and effect-tracked early, but full runtime compatibility is Phase 4. MVP modes may reject it or require an explicit compatibility path.
5. **Weak-reference APIs**: `WeakMap`, `WeakSet`, and `FinalizationRegistry` are not early-phase commitments; they land only when semantics can be preserved without undermining the no-tracing-GC design.
6. **Package installation model**: dependency installation is fetch-and-link by default. npm lifecycle scripts are disabled unless explicitly opted into.
7. **CLI contract**: default success output stays minimal; JSON output is versioned and stable for tooling.
8. **Schemas**: CLI envelopes, diagnostics, effect reports, and sandbox policies use centralized versioned JSON schemas described in `specs/18-schemas.md`.
9. **Module resolution simplification**: `kali.json#imports` is the canonical path-alias/import-map mechanism. Early phases do not require a separate `paths`/`baseUrl` system.
10. **Canonical target profile**: the compiler targets `wasm32` linear memory and the Kali host ABI in Phases 1-3. Specs may mention optional WASM features, but pointer-sized runtime layouts, NaN-boxing, and host call conventions assume `wasm32` unless a later phase explicitly generalizes them.
11. **Regex compatibility rule**: `RegExp` must follow ECMAScript semantics. Pure-Rust implementation is required, but a generic Rust regex crate is not sufficient unless wrapped or extended to preserve JavaScript-visible behavior.

## Phase 1 MVP Checklist

A Phase 1 implementation is considered complete when it can:

- Parse and type-check real `.ts` and `.js` programs with strong TypeScript compatibility
- Emit and run a single linked WASM artifact via `wasmtime`
- Support the core Web Platform baseline plus the initial Deno-style host API surface
- Produce machine-parseable diagnostics and concise default CLI output
- Run `check`, `build`, `run`, `fmt`, and `test` workflows reliably
- Reject or explicitly gate unsupported dynamic features instead of silently miscompiling them

## Feature Maturity Highlights

To reduce drift across individual specs, the following features have a single canonical maturity status:

| Feature | Phase / Status | Notes |
|---|---|---|
| Static ESM imports | Phase 1 MVP | Core module model |
| Dynamic `import()` with literal string | Phase 3 target | May be lowered to the already-linked artifact; no runtime WASM module linking required |
| Dynamic `import()` with non-literal specifier | Later compatibility | Treated as a dynamic effect boundary |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early single-linked-artifact model; later compatibility mode is optional |
| `eval` / `Function()` | Phase 4 compatibility | Parsed and effect-tracked earlier |
| Built-in effect inference / `kali effects` | Phase 2 target | Foundation for sandbox-first workflows |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in capability model |
| Algebraic effect declarations / handlers | Later compatibility | Experimental, optional extension |
| `Proxy` | Later compatibility | Early phases may type-check but should reject unsupported runtime use |
| `WeakMap`, `WeakSet`, `FinalizationRegistry` | Later compatibility | Deferred until semantics fit the no-tracing-GC design |
| `SharedArrayBuffer` / `Atomics` | Opt-in only | Requires WASM threads and a later runtime profile |
| npm lifecycle scripts | Opt-in only | Disabled by default across all phases |

The full feature matrix lives in [specs/19-feature-maturity.md](specs/19-feature-maturity.md).

## Specification Breakdown

| # | Spec | Description |
|---|------|-------------|
| 1 | [Architecture](specs/01-architecture.md) | System overview, crate structure, compilation pipeline |
| 2 | [Lexer & Parser](specs/02-lexer-parser.md) | ECMAScript + TypeScript + Kali extensions parsing |
| 3 | [AST](specs/03-ast.md) | Abstract syntax tree design and representation |
| 4 | [Type System](specs/04-type-system.md) | Extended type system: HM inference, flow typing, effects, constraints |
| 5 | [Intermediate Representations](specs/05-ir.md) | Multi-level IR design with explicit memory layouts |
| 6 | [Memory Management](specs/06-memory.md) | Compile-time ownership, stack/heap decisions, reference counting |
| 7 | [Optimization & Specialization](specs/07-specialization.md) | Optimization passes, generic specialization, fast/advanced modes |
| 8 | [WebAssembly Codegen](specs/08-wasm-codegen.md) | WASM code generation and AOT compilation |
| 9 | [Sandboxing & Effects](specs/09-sandboxing.md) | Effect system, static analysis, runtime resource constraints, policies |
| 10 | [Runtime](specs/10-runtime.md) | WASM execution (wasmtime initially), built-in APIs |
| 11 | [Standard APIs](specs/11-standard-apis.md) | Deno, Node.js, and Browser API compatibility layers |
| 12 | [CLI](specs/12-cli.md) | Command-line interface design, AI-friendly output |
| 13 | [Embedding & C API](specs/13-embedding.md) | C API, Rust library API for embedding |
| 14 | [Package Management](specs/14-packages.md) | npm compatibility, non-node-gyp package support |
| 15 | [Error Reporting](specs/15-errors.md) | Error message design for humans and AI agents |
| 16 | [Testing](specs/16-testing.md) | Test suite inspired by tsc, conformance tests |
| 17 | [Formal Verification](specs/17-verification.md) | Lean proofs for core implementation invariants |
| 18 | [Schemas](specs/18-schemas.md) | Versioned JSON schemas for CLI, diagnostics, effects, and policies |
| 19 | [Feature Maturity](specs/19-feature-maturity.md) | Canonical phase/status matrix for dynamic and compatibility-heavy features |

## File Extensions

- `.ts`, `.tsx` — TypeScript (with optional JSX)
- `.js`, `.jsx`, `.mjs` — JavaScript (with optional JSX)
- All extensions are first-class and compiled through the same pipeline.

## Inspirations

- **Engines**: [V8](https://github.com/v8/v8), JavaScriptCore, SpiderMonkey, [Boa](https://github.com/boa-dev/boa), [Hermes](https://github.com/facebook/hermes)
- **Compilers**: [tsc](https://github.com/microsoft/Typescript), [Porffor](https://github.com/CanadaHonk/porffor)
- **Runtimes**: [Deno](https://github.com/denoland/deno)
- **Type theory**: Haskell, Idris, Agda, Lean
- **Systems**: Rust (ownership model, ergonomics)
