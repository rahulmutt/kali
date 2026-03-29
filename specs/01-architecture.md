# 01 — Architecture

## Compilation Pipeline

```
Source (.ts/.js/.kali)
  → Lexer          (specs/02-lexer-parser.md)
  → Parser         (specs/02-lexer-parser.md)
  → AST            (specs/03-ast.md)
  → Type Checker   (specs/04-type-system.md)
  → Typed AST
  → HIR            (specs/05-ir.md) — High-level IR, close to source semantics
  → MIR            (specs/05-ir.md) — Mid-level IR, explicit memory layouts
  → LIR            (specs/05-ir.md) — Low-level IR, WASM-ready
  → WASM Module    (specs/08-wasm-codegen.md)
  → Execution      (specs/10-runtime.md)
```

## Crate Structure

```
kali/
├── crates/
│   ├── kali_lexer/        — Tokenization
│   ├── kali_parser/       — Parsing to AST
│   ├── kali_ast/          — AST node definitions
│   ├── kali_types/        — Type system, inference engine, effect system
│   ├── kali_hir/          — High-level IR: desugaring, name resolution
│   ├── kali_mir/          — Mid-level IR: memory layout decisions, ownership
│   ├── kali_lir/          — Low-level IR: WASM-oriented representation
│   ├── kali_codegen/      — WASM binary emission
│   ├── kali_optimize/     — Optimization passes (basic + advanced)
│   ├── kali_specialize/   — Generic function specialization
│   ├── kali_sandbox/      — Sandboxing policies, effect analysis
│   ├── kali_runtime/      — Runtime support library (compiled to WASM)
│   ├── kali_api_deno/     — Deno API compatibility
│   ├── kali_api_node/     — Node.js API compatibility
│   ├── kali_api_web/      — Browser/Web API compatibility
│   ├── kali_cli/          — CLI binary
│   ├── kali_embed/        — Rust embedding API
│   ├── kali_capi/         — C FFI API
│   ├── kali_error/        — Error types and formatting
│   ├── kali_npm/          — npm package resolution and loading
│   └── kali_common/       — Shared utilities, string interning, source maps
├── runtime/               — WASM runtime support (Rust, compiled to WASM)
├── tests/                 — Integration and conformance tests
├── proofs/                — Lean 4 formal verification
└── Cargo.toml             — Workspace manifest
```

## Key Design Decisions

### Pure Rust
All components are implemented in Rust. No C/C++ dependencies are linked. External WASM execution uses `wasmtime` (pure Rust).

### Query-Based Architecture
Follow a demand-driven (query-based) compilation model similar to rustc and Salsa. This enables:
- Incremental compilation
- Parallel type checking
- Lazy evaluation of unused modules

### Interning
All identifiers, string literals, and type representations are interned for fast comparison and low memory usage. Use a global `Interner` backed by a concurrent hash map.

### Source Spans
Every AST/IR node carries a `Span` (byte offset range + file ID) for error reporting. Spans are compact (8 bytes) and cheaply copyable.

### Arenas
AST and IR nodes are arena-allocated for cache-friendly traversal and bulk deallocation. Each compilation unit gets its own arena.

### Parallel Pipeline
- Lexing + parsing: per-file parallelism
- Type checking: per-module parallelism with dependency ordering
- Codegen: per-function parallelism
- Use `rayon` for data parallelism where appropriate.

### Compilation Modes

| Mode | Description |
|------|-------------|
| `--fast` | Minimal optimization, fastest compile time (default) |
| `--release` | Standard optimizations: inlining, dead code elimination, layout optimization |
| `--release-advanced` | Aggressive optimization: full specialization, WASM opt passes, LTO |

### Error Strategy
Compilation is resilient — continue after errors to report as many issues as possible in one pass. Use a `Diagnostics` collector that accumulates errors/warnings without aborting. See [specs/15-errors.md](15-errors.md).
