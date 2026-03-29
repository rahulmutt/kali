# Kali Specification

This document is the top-level roadmap for Kali and the canonical index into `specs/*.md`.

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, with three priorities:
1. **Fast compilation and fast generated code**
2. **Sandbox-first execution for untrusted and AI-generated programs**
3. **A machine-friendly developer experience**

It is implemented in Rust, avoids embedded C/C++ dependencies, emits WebAssembly ahead of time, and uses stronger static analysis than TypeScript where that improves safety, performance, or sandboxability.

## Core Product Definition

The goals in [BOOTSTRAP.md](BOOTSTRAP.md) are the **long-term product definition**. This spec set turns that vision into a phased plan so the project can ship in coherent increments without weakening the eventual target.

Kali should eventually provide:
- broad ECMAScript + TypeScript compatibility, tracking the latest published ECMA-262 edition as the language baseline while still phase-gating hard runtime features explicitly
- first-class `.js` support with type inference strong enough to compile plain JavaScript efficiently
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
- **Capability effects first**: the initial effect system is a conservative sandbox-capability summary, not a full algebraic-effect language.
- **Parse broad, enable narrowly**: the parser can accept syntax before the runtime/checker fully supports it; unsupported semantics must be gated explicitly.
- **Latest-standard baseline, phased semantics**: language/frontend work should target the latest published ECMA-262 edition rather than a frozen edition number, while runtime-heavy or semantically expensive features still follow the maturity matrix.
- **Compatibility is phased**: "support everything" is the long-term goal, not the MVP promise.

## Canonical Terminology

These terms are used across the spec set and should keep the same meaning everywhere:

- **Linked artifact**: one compiled WASM artifact that contains the fully resolved static module graph for a program or library build. Early phases standardize on this model.
- **API surface**: the host API family selected by `--api` such as `deno`, `node`, or `browser`.
- **Build mode**: the optimization level selected by `--fast`, `--release`, or `--release-advanced`.
- **Runtime profile**: execution-model switches that materially change runtime semantics or host requirements, such as the later `--wasm-threads` profile.
- **Profile**: shorthand for the effective combination of API surface, build mode, target assumptions, and any enabled runtime-profile switches.
- **Config naming**: in `kali.json`, the canonical config shape keeps these leaf keys under `compilerOptions`: `apiSurface`, `buildMode`, and `runtimeProfiles`; CLI flags remain `--api`, `--fast` / `--release` / `--release-advanced`, and later profile switches such as `--wasm-threads`.
- **Compatibility path**: an explicit, documented opt-in route for expensive or semantically difficult features such as `eval`; it is never an implicit fallback.
- **Dynamic/tagged value**: a runtime value carried in a generic tagged representation because the compiler could not keep it in a precise unboxed/static form.
- **Dynamic object layout**: an object representation that cannot rely on a fixed compile-time field-offset layout because keys, mutation patterns, or prototype behavior are too dynamic.

## Canonical Host/Profile Combinations

To reduce drift between the CLI, runtime, package, and host-API chapters, early phases should treat these combinations as the canonical baseline:

| API surface | Command/form | Early-phase status | Notes |
|---|---|---|---|
| `deno` | `run`, `build`, `check`, `test` | Phase 1 MVP | Default standalone host surface |
| `browser` | `check` | Phase 1 MVP | Analysis/build profile only; no standalone DOM/runtime promise |
| `browser` | `build --bundle` | Phase 1 MVP | Emits WASM + JS glue for a real browser host |
| `browser` | `run`, `test`, or plain `build` | Rejected by default | Must fail with the canonical feature-maturity diagnostic |
| `node` | `run`, `build`, `check`, `test` | Phase 3 target | Broader ecosystem profile once the documented Node subset exists |
| any API surface + `--wasm-threads` | compile/run/test | Later compatibility (opt-in only) | Selects a different runtime profile; never silently ignored |

This table is only a summary. The canonical command/profile matrix remains [specs/19-feature-maturity.md](specs/19-feature-maturity.md).

## Canonical Representation-Downgrade Rules

To keep the type checker, ownership analysis, IR, and codegen aligned, Kali should treat these representational downgrades as the canonical ladder when precision is lost:

| From | To | Typical triggers | Canonical consequence |
|---|---|---|---|
| fixed object layout | dynamic object layout | computed property writes, unstable key sets, prototype-sensitive mutation, reflective operations | object stays valid but loses fixed-offset layout assumptions |
| unboxed/static value | tagged/dynamic value | imprecise unions, dynamic operator behavior, unresolved JS boundary flows | later IR/codegen must preserve runtime tags and checks |
| stack/local allocation | unique heap allocation | value escapes its creating scope but still has one owner | ownership stays deterministic; free on owner drop |
| unique heap allocation | shared/ref-counted heap allocation | closure capture with mutation, aliasing across containers, multiple live owners | insert deterministic reference-counting operations |
| precise static semantics | feature gate rejection | unsupported runtime feature such as early-phase `eval`, dynamic loading, or unsupported host/profile mode | emit the canonical maturity diagnostic instead of widening to `any` or silently degrading |

These are semantic coordination rules, not just optimizer heuristics. Detailed per-subsystem behavior lives in [specs/04-type-system.md](specs/04-type-system.md), [specs/05-ir.md](specs/05-ir.md), and [specs/06-memory.md](specs/06-memory.md).

## Delivery Phases

The canonical phase matrix lives in [specs/19-feature-maturity.md](specs/19-feature-maturity.md). At a high level:

### Phase 1 — Core compiler
Deliver a practically useful compiler/runtime with:
- lexer, parser, AST
- name resolution and baseline TypeScript-compatible checking
- JavaScript compilation powered by the same inference pipeline, with conservative fallback to dynamic representations where needed
- HIR and LIR
- direct HIR → LIR lowering allowed
- simple WASM emission
- runtime sandbox enforcement and resource limits
- minimal Web + Deno host surface
- browser-targeted `check --api browser` and `build --bundle --api browser` support
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
- broader npm package compatibility beyond the Phase 1 linked-artifact/CJS baseline
- meaningful Node API support
- broader browser packaging/interoperability beyond the Phase 1 bundle baseline

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

## Phase-1 Simplification Rules

To keep the MVP realistic, Phase 1 explicitly does **not** promise:
- general runtime support for `eval`, `Function()`, dynamic `require()`, or non-literal `import(expr)`
- runtime dynamic module loading beyond the statically linked module graph
- standalone browser-host emulation or DOM APIs in `kali run`
- full Node.js parity just because some npm packages already work
- stable public embedding contracts yet, even though the implementation is library-first internally
- full static sandbox proofs; runtime enforcement comes first

These are deliberate staging choices, not reductions of the long-term goal.

## Canonical Phase-1 Non-Goals

This section is the short checklist other chapters should link to instead of restating their own partial caveats.

Phase 1 is **not**:
- a full Node compatibility release
- a browser-engine or DOM-runtime implementation
- a dynamic-code compatibility release (`eval`, `Function()`, dynamic module loading)
- a full static-effect-proof system
- a stable public embedding/ABI release

If a later chapter needs to mention one of these, it should reference this section and [specs/19-feature-maturity.md](specs/19-feature-maturity.md) rather than inventing a new phase promise.

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
- **Phase 1 host surface narrowed**: Deno `exit` / `cwd` / `chdir` are deferred so the initial sandbox/effect contract stays small and auditable.
- **C embedding artifacts disambiguated**: `kali_capi` owns the stable host header `kali.h`; `kali build --capi` emits program-specific headers such as `foo.exports.h`.
- **Config/profile naming normalized**: the spec set now uses `compilerOptions.apiSurface`, `compilerOptions.buildMode`, and `compilerOptions.runtimeProfiles` as the canonical `kali.json` terminology instead of mixing CLI flag names directly into config examples or inventing duplicate top-level keys.

## How To Use This Spec Set

When making design or implementation decisions:
1. Start here.
2. Check [specs/19-feature-maturity.md](specs/19-feature-maturity.md) for the current support phase.
3. Follow the relevant detailed chapter.
4. If a machine-readable format is involved, use [specs/18-schemas.md](specs/18-schemas.md).
5. If two chapters seem to conflict, prefer the more constrained interpretation and update the spec set to restore one canonical answer.
