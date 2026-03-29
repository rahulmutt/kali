# Kali — Specification

Kali is an ahead-of-time TypeScript/JavaScript compiler targeting WebAssembly, built in Rust. It extends TypeScript's type system with Hindley-Milner inference and an effect system, enabling sandboxed execution with compile-time memory management (no garbage collector). Designed as a secure, embeddable runtime for AI-generated code.

## Design Principles

1. **AOT only** — No JIT. All compilation happens ahead of time.
2. **No GC** — Ownership and lifetime analysis at compile time (stack vs heap, Rc when needed).
3. **Sandbox-first** — Static effect analysis and runtime resource constraints.
4. **AI-native** — Token-efficient CLI output, machine-parseable errors, static effect JSON export.
5. **Embeddable** — C API for integration from any language.
6. **Pure Rust** — No C/C++ dependencies.
7. **Spec-compliant** — Full ECMA-262 (16th edition) support including `eval`.
8. **Fast** — Blazing-fast lexing, parsing, type-checking, and codegen with optional advanced optimizations.

## Specification Breakdown

| # | Spec | Description |
|---|------|-------------|
| 1 | [Architecture](specs/01-architecture.md) | System overview, crate structure, compilation pipeline |
| 2 | [Lexer & Parser](specs/02-lexer-parser.md) | ECMAScript + TypeScript + Kali extensions parsing |
| 3 | [AST](specs/03-ast.md) | Abstract syntax tree design and representation |
| 4 | [Type System](specs/04-type-system.md) | Extended type system: HM inference, flow typing, effects, constraints |
| 5 | [Intermediate Representations](specs/05-ir.md) | Multi-level IR design with explicit memory layouts |
| 6 | [Memory Management](specs/06-memory.md) | Compile-time ownership, stack/heap decisions, reference counting |
| 7 | [Specialization & Optimization](specs/07-specialization.md) | Generic specialization, optimization passes, fast/advanced modes |
| 8 | [WebAssembly Codegen](specs/08-wasm-codegen.md) | WASM code generation and AOT compilation |
| 9 | [Sandboxing & Effects](specs/09-sandboxing.md) | Effect system, static analysis, runtime resource constraints, policies |
| 10 | [Runtime](specs/10-runtime.md) | WASM execution (wasmtime/wasmer), built-in APIs |
| 11 | [Standard APIs](specs/11-standard-apis.md) | Deno, Node.js, and Browser API compatibility layers |
| 12 | [CLI](specs/12-cli.md) | Command-line interface design, AI-friendly output |
| 13 | [Embedding & C API](specs/13-embedding.md) | C API, Rust library API for embedding |
| 14 | [Package Management](specs/14-packages.md) | npm compatibility, non-node-gyp package support |
| 15 | [Error Reporting](specs/15-errors.md) | Error message design for humans and AI agents |
| 16 | [Testing](specs/16-testing.md) | Test suite inspired by tsc, conformance tests |
| 17 | [Formal Verification](specs/17-verification.md) | Lean proofs for core implementation invariants |

## Inspirations

- **Engines**: V8, JavaScriptCore, SpiderMonkey, Boa, Hermes
- **Compilers**: tsc, Porffor
- **Runtimes**: Deno
- **Type theory**: Haskell, Idris, Agda, Lean
- **Systems**: Rust (ownership model, ergonomics)
