# Kali Specification

This document is the top-level index and canonical overview for the Kali project.

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for:
- fast compilation and execution
- strong static analysis
- sandboxed execution
- AI-friendly diagnostics and tooling
- pure-Rust implementation and embedding

It references the detailed specifications in `specs/*.md` and defines the canonical vocabulary used across them so the spec set stays consistent.

## Product Summary

Kali aims to:
- compile TypeScript and JavaScript to WebAssembly ahead of time
- use richer static analysis than traditional TypeScript where it remains predictable and fast
- infer effects and support sandbox policies for untrusted or AI-generated code
- avoid tracing GC by making compile-time ownership and allocation decisions where possible
- aggressively specialize code and memory layouts when the program is analyzable
- provide a clean CLI and embeddable Rust/C APIs
- support practical ecosystems: Deno-first runtime behavior, browser-targeted builds early, broader Node compatibility later

## Hard Constraints

These constraints are project-wide and should not be weakened in lower-level specs:
- **AOT-only**: no language-level JIT compilation
- **Pure Rust**: no embedded C/C++ libraries
- **Sandbox-first**: runtime enforcement is a first-class requirement, not an afterthought
- **Single linked artifact early**: Phase 1-3 builds target one linked WASM artifact for the resolved static graph
- **No silent semantic fallback**: unsupported or phase-gated features must fail explicitly rather than degrade invisibly
- **AI-friendly machine contracts**: JSON output, diagnostics, and effect reports are stable, concise, and versioned

## Canonical Vocabulary

To reduce drift across the spec set, these terms are canonical:

- **API surface**: the host API family selected by CLI/config, e.g. `deno`, `node`, `browser`
- **Build mode**: optimization level, one of `fast`, `release`, `release-advanced`
- **Runtime profile**: semantic runtime capability profile orthogonal to API surface, e.g. the default single-threaded baseline or later `wasm-threads`
- **Feature maturity**: phase/status classification defined in `specs/19-feature-maturity.md`
- **Schema contract**: machine-readable JSON formats defined in `specs/18-schemas.md`
- **Linked artifact model**: compile the resolved static graph into one linked WASM artifact rather than relying on runtime WASM module linking

If another spec needs to describe maturity, schemas, or command/profile gating, it should reference the canonical doc instead of redefining it.

## Phase Roadmap

The phase names below are canonical across the spec set.

### Phase 1 — Core compiler
Deliver a practically useful compiler/runtime with:
- lexer, parser, AST, name resolution
- TypeScript-compatible checking and first-class JavaScript compilation with conservative inference
- HIR and LIR, with direct `HIR -> LIR` lowering allowed
- WASM emission and wasmtime-based execution
- runtime sandbox enforcement and resource limits
- Web baseline APIs plus Deno-first standalone runtime subset
- browser-targeted `check --api browser` and `build --bundle --api browser`
- core CLI workflows: `run`, `build`, `check`, `fmt`, `lint`, `test`, `install`

### Phase 2 — Ownership, effects, embedding
Add:
- MIR as the canonical ownership/layout IR
- escape analysis and deterministic memory management strategy
- stable effect reports and compile/check-time policy validation
- explicit `pure` and effect annotations for the built-in capability model
- stable public Rust embedding API and C ABI

### Phase 3 — Specialization and ecosystem breadth
Add:
- broader specialization and layout optimization
- incremental compilation
- broader Node compatibility
- broader browser packaging/interoperability
- broader npm compatibility beyond the early linked-artifact subset

### Phase 4 — Advanced compatibility
Add:
- hard dynamic compatibility features such as `eval` / `Function()`
- more difficult runtime/API compatibility surfaces
- broader proof coverage for critical subsystems

The detailed maturity matrix lives in `specs/19-feature-maturity.md`.

## Canonical Host Capability Table

This table is the compact cross-spec reference for what each host/API mode means in early phases.

| Surface / mode | Shared Web baseline | Deno additions | Node additions | Browser-only deployment behavior |
|---|---|---|---|---|
| `--api deno` (default standalone) | Yes | Yes | No | No |
| `--api node` | Yes | No by default; Node compatibility is its own surface | Phase 3 target subset only | No |
| `--api browser` for `check` | Analysis target only | No | No | No standalone runtime implied |
| `build --bundle --api browser` | Yes, targeting the real browser host | No | No | Emit WASM + JS glue for deployment in a real browser |
| `run/test --api browser` | Rejected by default in early phases | No | No | No embedded browser engine |

Interpretation rules:
- the **Web baseline** is the shared baseline across supported surfaces; `--api` selects additional globals/modules or a browser-targeted profile on top of that baseline
- early standalone execution is **Deno-first**
- Node compatibility is phase-gated and must not be implied by fallback shims
- browser support is initially a **check/build profile**, not a standalone runtime contract

## Canonical Default Execution Tuple

Unless a command, config file, or later feature gate says otherwise, the default execution/build tuple is:
- `apiSurface = deno`
- `buildMode = fast`
- `runtimeProfiles = []`
- `compat.features = []`

Interpretation rules:
- `runtimeProfiles = []` means the default single-threaded baseline runtime
- `compat.features = []` means no later-phase compatibility escape hatches are enabled
- `kali run main.ts`, `kali test`, and `kali build main.ts` should be read as using this tuple unless flags/config override it
- `kali check main.ts` uses the same default host/API selection (`apiSurface = deno`) even though build mode and runtime-profile switches are only meaningful for build/run-style commands

This tuple is the canonical simplification for examples across the CLI, embedding, runtime, and maturity specs.

## Canonical Sources of Truth

Use these files as the primary authority for each concern:

- **Architecture and crate layout**: `specs/01-architecture.md`
- **Lexing and parsing**: `specs/02-lexer-parser.md`
- **AST and symbols**: `specs/03-ast.md`
- **Type system and inference**: `specs/04-type-system.md`
- **IR pipeline**: `specs/05-ir.md`
- **Memory and ownership**: `specs/06-memory.md`
- **Optimization and specialization**: `specs/07-specialization.md`
- **WASM codegen**: `specs/08-wasm-codegen.md`
- **Sandboxing and effects**: `specs/09-sandboxing.md`
- **Runtime model**: `specs/10-runtime.md`
- **Standard APIs / host surfaces**: `specs/11-standard-apis.md`
- **CLI behavior**: `specs/12-cli.md`
- **Embedding and C API**: `specs/13-embedding.md`
- **Packages and resolution**: `specs/14-packages.md`
- **Diagnostics**: `specs/15-errors.md`
- **Testing strategy**: `specs/16-testing.md`
- **Formal verification**: `specs/17-verification.md`
- **JSON schemas**: `specs/18-schemas.md`
- **Feature maturity matrix**: `specs/19-feature-maturity.md`

## Cross-Spec Consistency Rules

These rules should be followed whenever the specs evolve:
- Do not restate a conflicting phase decision outside `specs/19-feature-maturity.md`
- Do not redefine JSON shapes outside `specs/18-schemas.md`
- Prefer one canonical term over near-synonyms (`apiSurface`, `buildMode`, `runtimeProfiles` in config)
- Keep **API surface** and **runtime profile** orthogonal: `deno` / `node` / `browser` are API-surface choices, while threading or other execution-capability knobs belong to runtime profiles
- If a feature is parse-supported but not semantically implemented yet, say so explicitly
- Prefer explicit rejection over undocumented emulation for unsupported behavior
- Keep Phase 1 promises narrow, dependable, and testable

## Canonical Representation-Downgrade Ladder

When Kali cannot keep a value or object on the most optimized path, it should degrade representation in this order instead of jumping unpredictably between ad hoc fallbacks:

1. **Static typed layout** — fixed object/aggregate layout, unboxed scalars where possible
2. **Owned structured heap layout** — still typed and layout-aware, but heap allocated due to escape/lifetime needs
3. **Shared structured heap layout** — typed layout preserved, but deterministic reference counting is introduced
4. **Tagged dynamic value** — value-level type uncertainty requires boxing/tagging
5. **Dynamic object layout** — partially known object shape with a dynamic side table / fallback slot
6. **Fully dynamic hash-map/object mode** — dictionary-like behavior with most layout optimizations disabled

Cross-spec rule:
- type-system uncertainty should widen types conservatively before IR/layout chooses a more dynamic representation
- IR lowering should preserve the highest representation rung still justified by the checker and analyses
- memory-management rules describe the ownership consequences of a downgrade, not a separate downgrade policy
- diagnostics may mention when a construct forces a lower rung if that materially impacts performance or sandbox reasoning

This ladder is the canonical simplification for reasoning about "dynamic" behavior across the type system, IR, memory, and optimization specs.

## Explicit Early-Phase Non-Goals

To keep the roadmap credible, the following are intentionally **not** Phase 1 goals even though they remain part of Kali's long-term direction:
- full Node.js API parity
- standalone browser runtime or DOM emulation
- full dynamic-loading compatibility (`eval`, `Function()`, non-literal `import()`)
- native addons, `node-gyp`, or any C/C++ dependency path
- a fully general algebraic-effect language surface
- broad formal verification of the full ECMAScript surface

These are deferred by design, not omitted accidentally. Where they matter to users, the compiler should reject them explicitly and point to feature maturity.

## Spec Amendment Rules

When extending the spec set:
- new phase or status claims must update `specs/19-feature-maturity.md`
- new machine-readable JSON fields or documents must update `specs/18-schemas.md`
- new CLI flags, subcommands, or config entry points must update `specs/12-cli.md` and, when machine-readable, `specs/18-schemas.md`
- new host API families or major API-surface promises must update `specs/11-standard-apis.md` and `specs/19-feature-maturity.md`
- new runtime profiles must update this file, `specs/12-cli.md`, and `specs/19-feature-maturity.md`
- if a change weakens an earlier simplification, the spec must explain why the extra complexity is worth it

## Intentional Simplifications

The spec intentionally makes a few simplifying choices to keep implementation tractable:
- one primary execution engine (`wasmtime`) first
- one linked WASM artifact per build in early phases
- one canonical machine-readable JSON contract per output type
- one primary standalone runtime surface early (`deno`), with browser as a check/build profile first
- one initial effect model centered on sandbox-relevant built-in capabilities

These simplifications are design choices, not omissions. They keep the project coherent while still leaving room for later compatibility layers.

## Spec Index

1. [01 — Architecture](specs/01-architecture.md)
2. [02 — Lexer & Parser](specs/02-lexer-parser.md)
3. [03 — AST](specs/03-ast.md)
4. [04 — Type System](specs/04-type-system.md)
5. [05 — Intermediate Representations](specs/05-ir.md)
6. [06 — Memory Management](specs/06-memory.md)
7. [07 — Optimization & Specialization](specs/07-specialization.md)
8. [08 — WebAssembly Code Generation](specs/08-wasm-codegen.md)
9. [09 — Sandboxing & Effects](specs/09-sandboxing.md)
10. [10 — Runtime](specs/10-runtime.md)
11. [11 — Standard APIs](specs/11-standard-apis.md)
12. [12 — CLI](specs/12-cli.md)
13. [13 — Embedding & C API](specs/13-embedding.md)
14. [14 — Package Management](specs/14-packages.md)
15. [15 — Error Reporting](specs/15-errors.md)
16. [16 — Testing](specs/16-testing.md)
17. [17 — Formal Verification](specs/17-verification.md)
18. [18 — Schemas](specs/18-schemas.md)
19. [19 — Feature Maturity](specs/19-feature-maturity.md)
