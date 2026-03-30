# Kali Specification

This document is the top-level guide and normalization layer for the Kali spec set.

It exists for three reasons:
1. turn `BOOTSTRAP.md` into one coherent phased plan,
2. define cross-spec terminology/rules that should not drift between chapters,
3. point readers to the owning chapter for each detailed subsystem.

The detailed subsystem requirements live in [`specs/`](./specs). When this document and a detailed chapter both speak about the same topic, prefer this file for cross-cutting normalization and the owning chapter for the concrete subsystem contract.

## Overview

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, implemented in Rust, designed around:
- strong static analysis,
- sandbox-first execution,
- deterministic machine-readable tooling,
- explicit memory/ownership decisions rather than tracing/background GC,
- aggressive but auditable specialization,
- embeddability through Rust-first APIs with later stable C ABI and WIT/component packaging.

Kali aims for broad JavaScript/TypeScript compatibility over time, but the spec deliberately phases hard features instead of implying that every aspiration is part of the MVP.

## Bootstrap Normalization Rule

`BOOTSTRAP.md` is the input brief. This spec set is the normative source of truth after normalization.

Normalization rules:
- treat broad product goals in `BOOTSTRAP.md` as **directional requirements**, then map them onto explicit phase promises in the spec chapters;
- when the bootstrap says Kali “should support” something large or expensive, do **not** infer same-phase MVP support unless a chapter and the maturity matrix say so;
- when the bootstrap lists competing goals, preserve the stronger safety/determinism constraint first.

Canonical examples of that normalization:
- **“Support Node, Deno, and browser APIs”** → Phase 1 is Deno-first plus browser-targeted analysis/build; broad Node compatibility is Phase 3.
- **“Support all features including eval”** → `eval`/`Function()` are part of the long-term compatibility contract, but Phase 4-gated behind the single schema-v1 compatibility switch `eval`.
- **“Latest ECMA-262”** → latest **published** ECMA-262 grammar is Phase 1; draft/Stage-3+ proposal support is experimental rather than implied.
- **“Programmable sandbox policy conditions”** → project policy files stay declarative in early phases; later programmable narrowing is via host-registered predicates, not executable project policy code.
- **“Use wasmtime or wasmer”** → standardize on `wasmtime` first; alternative engines are later implementation extensions.
- **“Support WIT / Component Model”** → Phase 1 keeps a base exported-library artifact; stable WIT-first public embedding and component packaging are Phase 2 targets.
- **“No GC”** → no tracing/background GC is allowed; deterministic ownership/reference-counted strategies are acceptable where the owning chapters permit them.

If a bootstrap aspiration and a detailed chapter seem in tension, prefer:
1. this normalization rule,
2. the owning chapter,
3. the feature-maturity matrix in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

## Goal Precedence

When goals compete, Kali resolves them in this order:
1. semantic correctness,
2. sandbox honesty and auditability,
3. determinism and explicitness,
4. predictable compilation cost,
5. performance and compatibility breadth.

This ordering is intentional. Kali should reject or deopt before it silently guesses.

## Canonical Terminology

### API surface
The selected host-facing ambient/runtime family:
- `deno`
- `node`
- `browser`

`browser` is a **browser-targeted context** in early phases, not a promise of a standalone browser runtime.

### Build mode
The compilation-cost/performance dial:
- `fast`
- `release`
- `release-advanced`

### Runtime profile
An execution-capability profile orthogonal to API surface, for example:
- baseline single-threaded runtime (default)
- later `wasm-threads`

API surface and runtime profile must not be conflated.

### Compat feature
An explicitly gated compatibility switch for semantics that are intentionally off by default. In schema v1, the canonical stable compat feature name is:
- `eval`

That single name covers both direct `eval` and the `Function()` constructor path.

### Browser-targeted context
A command context whose effective `apiSurface` is `browser`.

In Phase 1, this means:
- `kali check --api browser`
- `kali build --bundle --api browser`

It does **not** mean:
- a standalone Kali-hosted browser runtime,
- DOM emulation inside `kali run`/`kali test`,
- permission to expose Deno/Node globals during browser-targeted analysis/build.

### Kali-hosted execution
Execution where Kali or an embedding host owns the runtime/import boundary, including:
- `kali run`
- `kali test`
- embedding hosts using Kali-controlled imports

### Kali-mediated capability subset
The stable schema-v1 capability vocabulary shared across effects and sandbox policy:
- filesystem
- network
- timers
- random
- console
- process
- eval

This is the stable capability vocabulary, **not** a claim that every command/profile/API surface enables every capability.

### Analysis context
The semantic context that materially affects static analysis results:
- `apiSurface`
- `runtimeProfiles`
- `compatFeatures`

Build mode affects compile effort and optimization behavior, but for early effect/package-analysis contracts the main semantic analysis context is the trio above unless an owning chapter says otherwise.

### Direct-input command
A command that requires exactly one explicit primary source input in early phases:
- `run`
- `build`
- `effects`

### Hybrid analysis command
A command that accepts explicit files or falls back to project discovery:
- `check`

### Project-oriented command
A command that defaults to project discovery when no explicit files are given:
- `fmt`
- `lint`
- `test`
- plain `install` for dependency-graph scanning

### Current-directory-scoped scaffold command
A command whose target root is always the current working directory rather than the nearest discovered ancestor project:
- `init`

In schema v1, `init` is the canonical exception to ordinary ancestor-based config discovery. It may create a nested child project inside an existing ancestor project as long as the current working directory itself does not already contain `kali.json`.

### Registry-analysis command
A command that analyzes exactly one explicit registry package identity rather than a project graph in early phases:
- `package-effects`
- `package-audit`

These commands do not invent a no-argument whole-project analysis mode in schema v1.

### Library-oriented artifact modes
Non-browser, export-oriented build modes:
- `--lib`
- `--capi`
- `--component`

### Logical roots
The normalized “what this report/build/test run is about” identifiers carried in schemas as `entryPoints`. Examples:
- `src/main.ts`
- a discovered test label
- `lodash`

This is a naming bridge only: schema field `entryPoints` is the canonical JSON field name.

## Host/API Summary

Phase-1 host posture:
- **standalone execution** is Deno-first,
- **browser support** is analysis/build-first,
- **Node compatibility** is a later ecosystem phase,
- **wasmtime** is the standardized early runtime engine,
- **AOT only**; no language-level JIT,
- **pure Rust only**; no embedded C/C++ libraries,
- **no tracing/background GC**.

Shared API-loading rule:
- Web baseline APIs are the shared baseline across supported surfaces,
- `--api deno|node|browser` selects which additional ambient APIs/modules exist beyond that baseline,
- unsupported globals/modules are absent rather than shimmed by default.

## Browser Ambient Typing vs Mediated Capability Split

This is the most important cross-spec clarification for browser support.

In browser-targeted contexts:
- Kali should expose the real browser ambient typing layer needed for browser programs,
- but stable schema-v1 effects and sandbox policy reason only about the **Kali-mediated capability subset**,
- therefore browser-targeted analysis/build may know about `window`, `document`, DOM types, and browser globals without implying that Kali individually mediates or sandbox-governs every browser API at runtime.

Consequences:
- `check --api browser` and `build --bundle --api browser` type-check against browser ambient types,
- browser-targeted `--sandbox` is a static compatibility/build-time validation contract,
- deployed browser bundles do not automatically inherit Kali-hosted runtime enforcement.

## Canonical Browser-Surface Rejection Split

Use this rule everywhere:
- if the user asks for a **supported browser concept with the wrong command shape**, reject with `E5008`;
- if the user asks for a **browser execution/test/runtime contract that does not exist yet**, reject with `E5006`.

Examples:
- `kali build --api browser main.ts` → `E5008` (wrong build shape; browser builds are bundle-only early)
- `kali build --lib --api browser lib.ts` → `E5008`
- `kali build --bundle --api node main.ts` → `E5008`
- `kali run --api browser main.ts` → `E5006`
- `kali test --api browser` → `E5006`

## Canonical Browser-Targeted Policy Boundary

For browser-targeted contexts:
- `--sandbox` validates static compatibility against the documented **Kali-mediated capability subset**,
- it does not promise Kali-controlled post-deployment sandbox enforcement inside an arbitrary real browser host,
- cross-cutting `resources.*` budgets that would imply post-deployment CPU/memory/file/process/thread enforcement are outside the early browser guarantee and must be rejected where the schema chapter says so.

Short form:
- **Kali-hosted execution** → runtime enforcement
- **browser-targeted analysis/build** → static compatibility only

## Artifact-Mode Matrix

Early documented build artifact modes form one small canonical matrix:

| Build invocation shape | Meaning |
|---|---|
| `kali build foo.ts` | default executable-oriented artifact flow |
| `kali build --bundle --api browser foo.ts` | browser-targeted bundle output |
| `kali build --lib lib.ts` | Phase-1 base exported-library artifact |
| `kali build --capi lib.ts` | Phase-2 C-embedding packaging layer over the library artifact |
| `kali build --component lib.ts` | Phase-2 Component Model packaging layer over the library artifact |

Rules:
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive unless a later chapter explicitly says otherwise,
- `--bundle` is browser-only and requires effective `apiSurface = browser`,
- library-oriented artifact modes are non-browser in early phases,
- companion artifacts such as JS glue, WIT, C headers, or component wrappers do not weaken the single linked core payload rule.

## Chapter Ownership

| Topic | Owning file |
|---|---|
| architecture and phases | [`specs/01-architecture.md`](./specs/01-architecture.md) |
| lexer/parser | [`specs/02-lexer-parser.md`](./specs/02-lexer-parser.md) |
| AST | [`specs/03-ast.md`](./specs/03-ast.md) |
| type system and inference | [`specs/04-type-system.md`](./specs/04-type-system.md) |
| IR pipeline | [`specs/05-ir.md`](./specs/05-ir.md) |
| memory/ownership | [`specs/06-memory.md`](./specs/06-memory.md) |
| specialization/optimization | [`specs/07-specialization.md`](./specs/07-specialization.md) |
| WASM/code emission/artifacts | [`specs/08-wasm-codegen.md`](./specs/08-wasm-codegen.md) |
| sandbox/effects/policy | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md) |
| runtime/host ABI | [`specs/10-runtime.md`](./specs/10-runtime.md) |
| standard APIs | [`specs/11-standard-apis.md`](./specs/11-standard-apis.md) |
| CLI shape and exit behavior | [`specs/12-cli.md`](./specs/12-cli.md) |
| embedding/C API/WIT | [`specs/13-embedding.md`](./specs/13-embedding.md) |
| packages/install/lock behavior | [`specs/14-packages.md`](./specs/14-packages.md) |
| diagnostics semantics | [`specs/15-errors.md`](./specs/15-errors.md) |
| testing/conformance evidence | [`specs/16-testing.md`](./specs/16-testing.md) |
| Lean verification | [`specs/17-verification.md`](./specs/17-verification.md) |
| JSON/config/policy schemas | [`specs/18-schemas.md`](./specs/18-schemas.md) |
| phase gating and maturity | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |

## Chapter Guide

Read in this order for a clean mental model:
1. this file,
2. `01` architecture,
3. `19` feature maturity,
4. language/frontend chapters `02`-`04`,
5. lowering/runtime chapters `05`-`11`,
6. toolchain/product chapters `12`-`18`.

## JSON Output Modes

To keep CLI, schemas, and command docs aligned, schema v1 uses one small output-mode model.

### Native-JSON reporting commands
Commands whose primary successful output is already a dedicated JSON payload:
- `effects`
- `package-effects`

Rules:
- on success **without** `--output json`, stdout is reserved for that payload only,
- human-oriented diagnostics for failures without `--output json` go to stderr,
- `--output json` wraps the same payload in the standard command envelope rather than inventing a second payload shape.

### Envelope-only JSON support
Some commands may support `--output json` even when schema v1 defines no dedicated success-payload schema for them yet.

Rules:
- the stable machine-readable contract is the standard command envelope itself,
- `payload` should be omitted or `null` rather than populated with ad hoc command-specific objects,
- `stdout` / `stderr` fields are for captured text streams only, not hidden structured result channels.

Canonical early example:
- `package-audit --output json`

Short form:
- **native JSON command** → payload by default, envelope on request
- **envelope-only JSON command** → envelope only

## Command/Context Axis Participation Table

This table exists so CLI, schemas, package analysis, diagnostics, and JSON-output rules use one shared model.

| Command family | `apiSurface` | `buildMode` | `runtimeProfiles` | `compatFeatures` | `sandbox` |
|---|---:|---:|---:|---:|---:|
| `check` | yes | no semantic effect on checking contract | yes | yes | yes |
| `effects` | yes | no | yes | yes | no |
| `build` | yes | yes | yes | yes | yes |
| `run` / `test` | yes | yes | yes | yes | yes |
| `fmt` / `lint` | no | no | no | no | no |
| `install` | no | no | no | no | no |
| `package-effects` | inherited analysis context only | no | inherited analysis context only | inherited analysis context only | no |
| `package-audit` | no in early phases | no | no | no | no |

Interpretation:
- “yes” means the axis materially participates in command semantics,
- “inherited analysis context only” means the command does not take its own parallel flag family in early phases but still validates inherited config/default analysis context,
- `sandbox` participation means the command is one of the canonical sandbox-aware commands.

## Canonical Sandbox-Aware vs Sandbox-Agnostic Commands

### Sandbox-aware commands
- `run`
- `test`
- `check`
- `build`

### Effect-reporting commands
- `effects`
- `package-effects`

These are reporting commands, not alternate policy-validation entrypoints.

### Sandbox-agnostic commands
- `fmt`
- `lint`
- `install`
- `init`
- early `package-audit`

Top-level `kali.json#sandbox` is ignored by effect-reporting and sandbox-agnostic commands.

## Validation-Order Rule

Report the outermost failing gate first:
1. command shape / arity / contradictory flag combination,
2. base command availability,
3. narrower inherited-context or profile gating,
4. source-code diagnostics within the selected valid context.

Consequences:
- contradictory browser build shapes fail before any narrower feature gate,
- a command that is itself unavailable reports that fact before reporting a narrower inherited profile problem,
- config-derived invalid effective values trigger the same checks as explicit CLI values.

## Project Discovery

### Canonical source-file classes

Kali uses one cross-spec split for source-file kinds:
- executable/analyzable source files: `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`
- declaration-only side inputs: `.d.ts`, `.d.mts`, `.d.cts`

Command-facing rule:
- runtime-bearing entrypoints and other primary program inputs use only the executable/analyzable set,
- declaration-only files may still participate as type-loading side inputs,
- `check`, `fmt`, and `lint` may accept declaration-only files explicitly,
- passing a declaration-only file where a command requires an executable/analyzable primary input is the canonical input-kind mismatch path (`E5007`), not general CLI misuse.

### Canonical project file set

Project discovery starts from the union of those two source-file classes.

Runtime-bearing entrypoints and direct executable inputs still use only the executable/analyzable set.

### Default project-discovery rule

If a command needs discovery and no explicit files are supplied:
- start at the effective project root,
- include the canonical project file set,
- honor `include` / `exclude` from `kali.json` when present,
- otherwise skip default managed/generated directories.

### Default excluded managed/generated directories
- `node_modules/`
- `.kali/`
- `dist/`
- `build/`
- `coverage/`
- `.git/`

### Nested project boundary rule

Discovery stops at nested child directories containing their own `kali.json`. Those are separate projects in schema v1.

### Explicit path boundary rule

For file-accepting source commands (`run`, `build`, `check`, `effects`, `fmt`, `lint`, `test`):
- explicit file/path targets must stay inside the effective project root,
- explicit file/path targets must not point into a nested child project that has its own `kali.json`,
- crossing into another project root is invalid command usage (`E5008`),
- once a target is explicit, `include` / `exclude` no longer filter it out.

This keeps explicit inputs from silently redefining project boundaries while still letting users name concrete files directly.

## Config Discovery and Configless Project Mode

Config discovery:
- commands search the current working directory and ancestors for the nearest `kali.json`,
- if found, that directory is the effective project root,
- if none is found, commands run in **configless project mode** with the current working directory as the effective project root,
- the schema-v1 exception is the **current-directory-scoped scaffold command** `init`, which always targets the current working directory instead of retargeting to a discovered ancestor project.

Configless project mode rules:
- plain `kali install` is a no-op success when there are no dependency inputs,
- explicit registry-package add (`kali install <pkg>` / `kali install --dev <pkg>`) creates the minimal manifest `{ "schemaVersion": 1 }` first,
- explicit raw-URL install may create lock/cache state but must not create a placeholder manifest by itself.

## Dependency-Management Mutability Rule

In early phases, `kali install` is the only command that mutates project-managed dependency state.

Non-install commands must not silently:
- rewrite manifests,
- repair lockfiles,
- fetch and materialize missing dependency state as a hidden side effect.

They should fail with the canonical dependency-state diagnostic path instead.

## Identity-Only Registry Target

Several early package workflows intentionally take only a registry **identity**, not an inline version selector. Canonical examples:
- `kali install lodash`
- `kali install --dev jsr:@std/path`
- `kali package-effects lodash`
- `kali package-audit jsr:@std/path`

The command then applies the package chapter's stable-release selection rules. This keeps early CLI/package flows deterministic and simple.

## Effective npm-Scriptable Install Work

`--allow-scripts` is meaningful only when the current `install` invocation includes explicit npm package install work or the discovered dependency graph contains npm package work that could run lifecycle scripts.

It is not meaningful for:
- explicit `jsr:` targets,
- raw URL targets,
- non-install commands.

Even when enabled, it does **not** imply:
- Node runtime support,
- project sandbox participation for install hooks,
- support for native addons, `node-gyp`, or binary/bootstrap-heavy package contracts.

## Numeric-Limit Semantics

Kali uses one cross-spec numeric-limit rule:
- positive-budget dimensions use omission as the “unspecified” state and reject `0`,
- zero-capable concurrency counters may use `0` as an explicit deny/tightening value.

Examples:
- `maxMemory`, `maxCpu`, `maxOpenFiles` must be positive when present,
- `maxSpawnedProcesses` and `maxThreads` may use `0` as an explicit deny/tightening value,
- non-zero values for later-gated capabilities/profiles remain unavailable until those capabilities/profiles exist.

## Published-Standard Boundary

“Latest ECMA-262” means the latest **published** ECMA-262 edition.

It does not implicitly include:
- draft spec text,
- Stage-3+ proposals,
- proposal semantics not yet in the published standard.

Proposal support, if any, must be explicit and experimental.

## Canonical Dynamic-Loading Boundary

To preserve the single linked core payload model:
- static `import` / `export` are core,
- literal `require()` is supported when statically resolvable,
- dynamic `require()` is rejected by default early,
- literal-string `import()` is a later lowering path over the already-linked graph,
- non-literal `import(expr)` is later compatibility work.

Kali should prefer explicit gating over bundler-style guesswork.

## Representation-Downgrade Ladder

When optimization assumptions break, downgrade the representation as little as necessary:
1. keep static layout + stack ownership when possible,
2. move to static layout + owned/shared heap if lifetime/aliasing requires it,
3. use partially dynamic layout only when closed-shape reasoning is no longer sound,
4. use fully dynamic/hash-map representation only when semantics require it.

Dynamic layout is a semantic fallback, not a synonym for heap allocation.

## Reproducibility Goal

Build outputs and machine-readable reports should be deterministic by default for the same pinned inputs, config, and toolchain.

This applies to:
- emitted WASM artifacts,
- generated metadata sidecars,
- JSON envelopes/reports,
- diagnostics ordering where the producer naturally owns that order.

## Scaffold Filename Convention

Unless a later template spec says otherwise:
- app scaffolds use `main.ts`
- library scaffolds use `lib.ts`

## Chapter Navigation

- [01 — Architecture](./specs/01-architecture.md)
- [02 — Lexer & Parser](./specs/02-lexer-parser.md)
- [03 — AST](./specs/03-ast.md)
- [04 — Type System](./specs/04-type-system.md)
- [05 — IR](./specs/05-ir.md)
- [06 — Memory Management](./specs/06-memory.md)
- [07 — Optimization & Specialization](./specs/07-specialization.md)
- [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- [09 — Sandboxing & Effects](./specs/09-sandboxing.md)
- [10 — Runtime](./specs/10-runtime.md)
- [11 — Standard APIs](./specs/11-standard-apis.md)
- [12 — CLI](./specs/12-cli.md)
- [13 — Embedding](./specs/13-embedding.md)
- [14 — Package Management](./specs/14-packages.md)
- [15 — Errors](./specs/15-errors.md)
- [16 — Testing](./specs/16-testing.md)
- [17 — Formal Verification](./specs/17-verification.md)
- [18 — Schemas](./specs/18-schemas.md)
- [19 — Feature Maturity](./specs/19-feature-maturity.md)
