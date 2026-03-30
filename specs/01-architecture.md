# 01 — Architecture

## Compilation Pipeline

```
Source (.ts/.tsx/.mts/.cts/.js/.jsx/.mjs/.cjs)
  → Lexer              (02-lexer-parser.md)
  → Parser             (02-lexer-parser.md)
  → AST                (03-ast.md)
  → Type Checker       (04-type-system.md)
    ├─ Name Resolution (symbol table, scopes)
    ├─ Type Inference  (TS-compatible checking + flow narrowing early; the shared bounded inference contract in Phase 1, with broader inference only in later phases)
    └─ Effect Inference (09-sandboxing.md; internal analysis may exist before stable user-facing reports)
  → Typed AST
  → HIR                (05-ir.md) — High-level IR, desugared
  → MIR                (05-ir.md) — Mid-level IR, memory layouts + ownership *(Phase 2+; Phase 1 may lower HIR → LIR directly)*
  → LIR                (05-ir.md) — Low-level IR, WASM-ready
  → WASM Module        (08-wasm-codegen.md)
  → Execution          (10-runtime.md)
```

TypeScript and JavaScript source files share this same pipeline. JavaScript is a first-class compiled input, not a transpile-only compatibility lane.

Declaration-only inputs (`.d.ts`, `.d.mts`, `.d.cts`) participate in parsing/checking/type loading as needed, but they are analysis-only side inputs rather than executable entrypoints in this pipeline.

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
│   ├── kali_runtime/      — Guest-side runtime support linked into emitted WASM artifacts
│   ├── kali_api_deno/     — Deno API compatibility (host functions)
│   ├── kali_api_node/     — Node.js API compatibility (host functions)
│   ├── kali_api_web/      — Web-platform baseline APIs and browser-bundle glue support (not a standalone DOM engine)
│   ├── kali_fmt/          — Code formatter
│   ├── kali_lint/         — Linter
│   ├── kali_cli/          — CLI binary (ties everything together)
│   ├── kali_embed/        — Rust embedding API crate (internal/pre-stable in Phase 1; stable public surface is Phase 2)
│   ├── kali_capi/         — C FFI/host-ABI crate (internal/pre-stable in Phase 1; stable public surface is Phase 2)
│   └── kali_npm/          — npm/JSR package resolution and loading
├── tests/                 — Integration and conformance tests
├── proofs/                — Lean 4 formal verification
└── Cargo.toml             — Workspace manifest
```

Interpretation rule:
- a crate appearing in this workspace layout does **not** by itself imply Phase 1 user-visible support or public-stability guarantees for that surface; for example, `kali_api_node` may exist as an internal staging area before `--api node` is part of the supported command/profile matrix, and `kali_embed` / `kali_capi` may exist as internal or pre-stable crates before the Phase-2 public embedding surface is frozen.

## Implementation Phases

The architecture is intentionally staged so the compiler can become useful early. The phase names and scope here are canonicalized to match [SPEC.md](../SPEC.md).

Reading rule:
- these are **phase contracts**, not the recommended engineering work queue
- for implementation sequencing, follow [SPEC.md](../SPEC.md)'s **Recommended Phase-1 Implementation Order** and the shared **Phase Contracts vs Implementation Order** rule
- later command/artifact families may still be documented earlier as **defined command families** without becoming Phase-1 support promises


1. **Phase 1 — Core compiler**: lexer, parser, AST, name resolution, TypeScript-compatible checking, first-class JavaScript compilation with conservative inference plus the shared **bounded inference contract** (locals, obvious unannotated parameters, analyzable return types, and the matching **annotation-required inference boundary** for the rest), HIR/LIR, simple WASM emission, a minimal Web/Deno host surface, the shared **Phase-1 browser-targeted command set**, runtime sandbox enforcement plus policy-file/config validation, the Phase-1 base `kali build --lib` artifact, the core CLI workflow, and a library-first internal architecture so the CLI is built on reusable compiler/runtime crates.
   - Source-kind clarification: `.mts` and `.cts` are part of the canonical TypeScript source set alongside `.ts` / `.tsx`.
   - `.mjs` / `.cjs` and package `type` metadata still control runtime module-kind interpretation where applicable.
   - File-extension support should not drift between the frontend, package resolver, CLI file discovery, and type-resolution rules.
2. **Phase 2 — Ownership + effects**: MIR, ownership/escape analysis, deterministic memory management, the stable **public effect-report surface** (`kali effects`, `kali package-effects`, and effect summaries/reporting), compile/check-time inferred-effect-vs-policy validation on top of the Phase-1 policy-schema/config validation path, and the stable **public embedding surface** (Rust API, public `--lib` + WIT contract, C ABI, and Component Model/C-embedding packaging).
3. **Phase 3 — Specialization + ecosystem**: specialization, advanced layout selection, broader npm package compatibility beyond the Phase 1 CJS/literal-`require` baseline, broader Node compatibility, broader browser packaging/interoperability beyond the Phase 1 bundle baseline, incremental compilation, and stronger optimization.
4. **Phase 4 — Advanced compatibility**: hardest dynamic features (`eval`, `Function()`, non-literal dynamic loading), deeper API coverage, and broader formal verification.

Every crate should expose a stable internal boundary even if its initial implementation is partial.

## Key Design Decisions

### Goal Precedence
The canonical tie-breaker order from [SPEC.md](../SPEC.md) applies here too:
1. semantic correctness
2. sandbox honesty and auditability
3. determinism and explicitness
4. predictable compilation cost
5. performance and compatibility breadth

Architecture, optimization, and embedding choices should be evaluated in that order rather than treating raw throughput as the only north star.

### Pure Rust
Follow the shared **Pure-Rust implementation contract** from [SPEC.md](../SPEC.md).

Concretely:
- Kali implementation crates and shipped dependencies remain Rust-only from the project/toolchain point of view.
- ordinary platform runtime/system libraries reached through the Rust toolchain or OS bindings do not, by themselves, violate that contract.
- bundling or requiring project-specific C/C++ implementation dependencies still violates it.
- early-phase external WASM execution is standardized on `wasmtime` (pure Rust), but that is an implementation default rather than a forever-exclusive backend choice.
- external crate dependencies should therefore stay in the pure-Rust lane (for example `wasmtime`, `rayon`, `regex-automata`).

### Query-Based Architecture
Follow a demand-driven (query-based) compilation model similar to rustc and Salsa. This enables:
- Incremental compilation
- Parallel type checking
- Lazy evaluation and caching of derived compiler queries/results
- Clean separation between source resolution, semantic analysis, and final whole-program linking

Important semantic guardrail:
- this is about lazily computing compiler data, **not** about skipping semantically required modules
- statically imported modules with side effects still participate in the resolved program graph and instantiation order even if their exported values are barely used

### Interning
All identifiers, string literals, and type representations are interned for fast comparison and low memory usage. Use a global `Interner` backed by a concurrent hash map.

### Source Spans
Every AST/IR node carries a compact internal `Span` (byte offset range + file ID) for error reporting. Spans should stay compact and cheaply copyable; the exact in-memory layout is an implementation detail rather than a frozen spec promise.

When Kali emits JSON diagnostics/effect reports, this internal span is translated into the schema-level `SourceSpan` / `SourceLocation` shapes defined in [specs/18-schemas.md](18-schemas.md). This keeps the implementation fast without forcing byte offsets into the external tooling contract.

### Arenas
AST and IR nodes are arena-allocated for cache-friendly traversal and bulk deallocation.

To keep the frontend and later pipeline stages aligned:
- parsed AST storage is **per file/module**
- typed side tables and later IR arenas may be **per compilation unit or resolved module graph**, depending on the pass

This avoids forcing one lifetime strategy onto every stage while keeping allocation ownership explicit.

### Parallel Pipeline
- Lexing + parsing: per-file parallelism
- Type checking: per-module parallelism with dependency ordering
- Codegen: per-function parallelism
- Use `rayon` for data parallelism where appropriate.

### Linking Strategy
Early-phase builds compile the full static module graph into a **single linked WASM payload**. This avoids premature dependence on experimental WASM module-linking features and simplifies optimization, packaging, and embedding.

Companion artifacts such as browser JS glue, WIT sidecars, or program-specific exports headers/embedding metadata may still be emitted by specific artifact modes, but they do not weaken the single-payload rule for the compiled program graph itself.

This is a semantic rule, not just a packaging preference: features that imply arbitrary runtime module loading must either lower back into the already-linked graph or be rejected/gated according to [specs/19-feature-maturity.md](19-feature-maturity.md).

To keep the model simple and consistent:
- static `import` / `export` are part of the core MVP
- dynamic `import()` is **not** a general runtime-linking mechanism in early phases
- literal-string `import()` may be lowered later to an async view over the already-linked graph
- non-literal dynamic loading remains later-phase compatibility work and a dynamic effect boundary

### Build Modes

This section uses the canonical term **build mode** to match [SPEC.md](../SPEC.md) and [12 — CLI](12-cli.md).

| Build mode | Description |
|------|-------------|
| `fast` | Minimal optimization, fastest compile time (default; selected by `--fast`) |
| `release` | Standard optimizations: inlining, dead code elimination, layout optimization (selected by `--release`) |
| `release-advanced` | Aggressive optimization: expanded specialization budget, optional separate-tool WASM post-pass, LTO (selected by `--release-advanced`) |

Compile-budget rule:
- `fast` is the canonical bounded-cost path and should avoid optimization or inference strategies whose worst-case behavior is hard to predict
- `release` may spend more compile budget on broadly beneficial whole-program improvements
- `release-advanced` is the only early documented place where materially more expensive optimization search/post-processing should be expected by default
- if an optimization, inference extension, or specialization strategy needs noticeably more compile budget, prefer gating it by build mode, an explicit flag, or a later phase rather than silently charging that cost to the default workflow

### Error Strategy
Compilation is resilient — continue after errors to report as many issues as possible in one pass. Use a `Diagnostics` collector that accumulates errors/warnings without aborting. See [specs/15-errors.md](15-errors.md).

### Canonical Target Assumption
Early phases target a single canonical execution profile:
- `wasm32` linear memory
- one linked module graph per build artifact
- the Kali host ABI implemented first on top of wasmtime

This keeps pointer layout, tagged-value representation, allocator design, and host import conventions consistent across the rest of the spec. Later phases may add additional targets or execution backends, but they should be layered on top of this baseline rather than weakening it.
