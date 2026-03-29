# 01 — Architecture

## Compilation Pipeline

```
Source (.ts/.tsx/.js/.jsx/.mjs)
  → Lexer              (specs/02-lexer-parser.md)
  → Parser             (specs/02-lexer-parser.md)
  → AST                (specs/03-ast.md)
  → Type Checker        (specs/04-type-system.md)
    ├─ Name Resolution  (symbol table, scopes)
    ├─ Type Inference    (HM unification + flow narrowing)
    └─ Effect Inference  (specs/09-sandboxing.md)
  → Typed AST
  → HIR                (specs/05-ir.md) — High-level IR, desugared
  → MIR                (specs/05-ir.md) — Mid-level IR, memory layouts + ownership
  → LIR                (specs/05-ir.md) — Low-level IR, WASM-ready
  → WASM Module        (specs/08-wasm-codegen.md)
  → Execution          (specs/10-runtime.md)
```

## Crate Structure

```
kali/
├── crates/
│   ├── kali_common/       — Shared utilities, string interning, source maps, spans
│   ├── kali_error/        — Diagnostic types and formatting
│   ├── kali_lexer/        — Tokenization
│   ├── kali_parser/       — Parsing to AST (including JSX)
│   ├── kali_ast/          — AST node definitions
│   ├── kali_types/        — Type system, inference engine, effect system
│   ├── kali_hir/          — High-level IR: desugaring from Typed AST
│   ├── kali_mir/          — Mid-level IR: memory layout decisions, ownership
│   ├── kali_lir/          — Low-level IR: WASM-oriented representation
│   ├── kali_codegen/      — WASM binary emission
│   ├── kali_optimize/     — Optimization and specialization passes
│   ├── kali_sandbox/      — Sandboxing policies, effect analysis
│   ├── kali_runtime/      — Runtime support library (Rust, compiled to WASM)
│   ├── kali_api_deno/     — Deno API compatibility (host functions)
│   ├── kali_api_node/     — Node.js API compatibility (host functions)
│   ├── kali_api_web/      — Browser/Web API compatibility (host functions)
│   ├── kali_fmt/          — Code formatter
│   ├── kali_lint/         — Linter
│   ├── kali_cli/          — CLI binary (ties everything together)
│   ├── kali_embed/        — Rust embedding API
│   ├── kali_capi/         — C FFI API
│   └── kali_npm/          — npm/JSR package resolution and loading
├── tests/                 — Integration and conformance tests
├── proofs/                — Lean 4 formal verification
└── Cargo.toml             — Workspace manifest
```

## Implementation Phases

The architecture is intentionally staged so the compiler can become useful early:

1. **Bootstrap**: lexer, parser, AST, name resolution, baseline TypeScript checking, HIR/LIR, simple WASM emission.
2. **Safety**: MIR, ownership/escape analysis, effect summaries, sandbox policy checking.
3. **Performance**: specialization, advanced layout selection, incremental compilation, stronger optimization.
4. **Compatibility**: broader Node/Deno/browser APIs, advanced dynamic features, full `eval`, and deeper verification.

Every crate should expose a stable internal boundary even if its initial implementation is partial.

## Key Design Decisions

### Pure Rust
All components are implemented in Rust. No C/C++ libraries are embedded or linked. External WASM execution uses `wasmtime` (pure Rust). The only external Rust crate dependencies allowed are pure-Rust crates (e.g., `wasmtime`, `rayon`, `regex-automata`).

### Query-Based Architecture
Follow a demand-driven (query-based) compilation model similar to rustc and Salsa. This enables:
- Incremental compilation
- Parallel type checking
- Lazy evaluation of unused modules
- Clean separation between source resolution, semantic analysis, and final whole-program linking

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

### Linking Strategy
Early-phase builds compile the full static module graph into a **single linked WASM artifact**. This avoids premature dependence on experimental WASM module-linking features and simplifies optimization, packaging, and embedding.

To keep the model simple and consistent:
- static `import` / `export` are part of the core MVP
- dynamic `import()` is **not** a general runtime-linking mechanism in early phases
- literal-string `import()` may be lowered later to an async view over the already-linked graph
- non-literal dynamic loading remains later-phase compatibility work and a dynamic effect boundary

### Compilation Modes

| Mode | Description |
|------|-------------|
| `--fast` | Minimal optimization, fastest compile time (default) |
| `--release` | Standard optimizations: inlining, dead code elimination, layout optimization |
| `--release-advanced` | Aggressive optimization: full specialization, WASM opt passes, LTO |

### Error Strategy
Compilation is resilient — continue after errors to report as many issues as possible in one pass. Use a `Diagnostics` collector that accumulates errors/warnings without aborting. See [specs/15-errors.md](15-errors.md).

### Canonical Target Assumption
Early phases target a single canonical execution profile:
- `wasm32` linear memory
- one linked module graph per build artifact
- the Kali host ABI implemented on top of wasmtime

This keeps pointer layout, tagged-value representation, allocator design, and host import conventions consistent across the rest of the spec. Later phases may add additional targets or backends, but they should be layered on top of this baseline rather than weakening it.
