# Kali Specification

This document is the top-level contract for the Kali spec set. It defines the canonical terminology and cross-cutting rules that other chapters reference instead of restating.

Detailed subsystem design lives in [`specs/`](./specs).

## Purpose

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for:
- fast compilation and execution
- sandbox-first execution
- strong static analysis, including effect analysis
- AI-friendly CLI and diagnostics
- pure-Rust implementation and embeddability

## Bootstrap Requirement Map

This is the compact top-level breakdown of the bootstrap brief into canonical spec areas.

| Bootstrap concern | Canonical handling |
|---|---|
| AOT-only TS/JS → WASM compiler | [01 — Architecture](./specs/01-architecture.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md) |
| Latest ECMA-262 grammar coverage, broad syntax acceptance | [02 — Lexer & Parser](./specs/02-lexer-parser.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| Stronger-than-TS checking and inference | [04 — Type System](./specs/04-type-system.md) |
| No tracing GC; compile-time ownership/allocation | [06 — Memory Management](./specs/06-memory.md) |
| Aggressive specialization | [07 — Specialization](./specs/07-specialization.md) |
| Sandboxing and effect-aware execution | [09 — Sandboxing](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md) *(declarative policy first; later trusted host predicates for embedding)* |
| JSON effect reporting and policy schemas | [09 — Sandboxing](./specs/09-sandboxing.md), [18 — Schemas](./specs/18-schemas.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| Deno-first standalone runtime, browser-targeted analysis/build, later Node support | [11 — Standard APIs](./specs/11-standard-apis.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| Dynamic compatibility paths (`eval`, `Function()`, dynamic loading) | [10 — Runtime](./specs/10-runtime.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| npm / JSR / raw URL package workflows | [14 — Package Management](./specs/14-packages.md) |
| AI-friendly CLI and diagnostics | [12 — CLI](./specs/12-cli.md), [15 — Errors](./specs/15-errors.md), [18 — Schemas](./specs/18-schemas.md) |
| Rust embedding, C ABI, WIT, Component Model | [13 — Embedding](./specs/13-embedding.md), [18 — Schemas](./specs/18-schemas.md) |
| Conformance, regression, and package-evidence testing | [16 — Testing](./specs/16-testing.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| Lean-backed formal verification | [17 — Formal Verification](./specs/17-verification.md) |

## Hard Constraints

These constraints are global and should not be weakened by subsystem docs:
- **AOT only**: no JIT compilation
- **Pure Rust**: no embedded C/C++ libraries
- **No tracing/background GC**: deterministic ownership techniques only
- **Single linked core WASM payload** for the resolved static graph in early phases
- **No silent fallback** for unsupported semantics or unsupported host/profile combinations
- **Stable machine-readable contracts** for JSON output, diagnostics, and effect reports

## Canonical Goal Precedence

When bootstrap goals pull in different directions, Kali should break ties in this order:
1. **Semantic correctness** — accepted/supported code must preserve the documented language/host behavior for the selected command/profile/surface
2. **Sandbox honesty and auditability** — never describe a capability as controlled, verified, or enforced when Kali cannot actually guarantee that in the selected environment
3. **Determinism and explicitness** — prefer one explicit, repeatable behavior over hidden fallback, ambient mutation, or heuristic mode switching
4. **Predictable compilation cost** — fast/default workflows keep bounded compile-time behavior; more expensive optimization/inference belongs behind documented modes/phase gates
5. **Performance and compatibility breadth** — pursue speed and ecosystem coverage aggressively, but not by violating the four rules above

Interpretation rule:
- when a subsystem doc seems to trade correctness or sandbox guarantees for convenience, this precedence order wins unless a later top-level spec revision changes it explicitly

## Long-Term Target vs Phase Promise

Kali's long-term target is broad TypeScript/JavaScript compatibility, but phase promises are narrower.

Canonical rule:
- parser and analysis breadth may grow ahead of runtime support
- unsupported semantics must fail explicitly rather than pretending they already work
- feature maturity is defined by [specs/19-feature-maturity.md](./specs/19-feature-maturity.md), not by syntax acceptance alone

## Bootstrap Resolution Notes

The bootstrap brief intentionally mixes end-state goals with near-term implementation constraints. To keep the spec set honest and simpler to read, Kali resolves the biggest tensions this way:
- **"Support the latest ECMA-262 standard"** means grammar tracking is immediate, while semantic support claims stay feature-by-feature and evidence-backed.
- **"Support Deno, Node.js, and browser APIs"** means those are the canonical API-surface names from the start, but command/profile availability is phased rather than implied all at once.
- **"Support all features, including `eval`"** means dynamic compatibility paths are part of the long-term contract, but they stay explicitly gated until Kali can preserve semantics and sandbox honesty.
- **"Programmable sandbox conditions"** are satisfied by later host-registered predicates for trusted embeddings, while project policy files stay declarative data in schema v1.
- **"WIT / Component Model / C embedding"** are part of the canonical public-library and embedding story, but they are layered on top of the core linked-WASM artifact rather than becoming separate compilation models.

This section exists to keep the bootstrap goals and the phase matrix aligned without repeating the same clarification in every subsystem chapter.

## Compatibility Staging Model

To keep the bootstrap goals, feature-maturity matrix, and subsystem docs aligned, Kali treats compatibility as three separate questions:
1. **Parse** — does Kali accept the syntax?
2. **Analyze** — can Kali type-check / infer / summarize effects for it?
3. **Execute** — can Kali lower and run it faithfully for the selected command/profile?

Interpretation rules:
- a later-phase feature may be parsed before it is executable
- analysis support may exist before lowering/runtime support
- command help and diagnostics should describe which stage is unavailable instead of collapsing everything into a vague “supported/unsupported” label
- the authoritative per-feature staging still lives in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

This model is especially important for dynamic compatibility features such as `eval`, `Function()`, dynamic loading, `Proxy`, weak-reference APIs, and browser-targeted ambient globals: Kali may parse or analyze them earlier than it can faithfully execute them.

## Conformance Claim Model

To keep “latest ECMA-262 support” honest and non-ambiguous, Kali separates three different claim types:
- **grammar coverage** — the parser tracks the latest published ECMA-262 grammar
- **semantic support** — execution/checking claims are made feature-by-feature and command/profile-by-command/profile
- **evidence-backed support** — a feature is described as supported only when the matching test/evidence track exists

Interpretation rules:
- grammar coverage alone is not a blanket promise that every accepted construct is already executable in every mode
- semantic support for a feature still follows the phase/status matrix in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)
- support wording in docs should follow the evidence rules in [specs/16-testing.md](./specs/16-testing.md) rather than one-off demos or anecdotal package wins

## Early-Phase Product Posture

These assumptions are intentionally explicit so the rest of the spec set does not drift:
- **standalone execution is Deno-first**
- **browser support is analysis/build first** in early phases (`check --api browser`, `build --bundle --api browser`)
- **Node compatibility is a later ecosystem phase**, not an MVP promise
- **all early builds target one linked core WASM payload** for the resolved static graph
- **companion artifacts are allowed**, but they do not change the single-payload rule
- **no tracing garbage collector** is introduced as a hidden fallback
- **no JIT**; Kali is AOT-only

For phase labels and command/profile maturity, see [specs/19-feature-maturity.md](./specs/19-feature-maturity.md).

## Canonical Axes and Terms

Availability note:
- the command-family labels below describe **command shape**, not guaranteed current-phase support
- a command can belong to one of these families even if its availability is phase-gated elsewhere
- canonical availability promises still live in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

### API surface
The selected host-facing API family:
- `deno`
- `node`
- `browser`

CLI spelling: `--api ...`

Config spelling: `compilerOptions.apiSurface`

### Build mode
The optimization/compile-time tradeoff:
- `fast`
- `release`
- `release-advanced`

CLI spelling: `--fast`, `--release`, `--release-advanced`

Config spelling: `compilerOptions.buildMode`

### Runtime profile
Execution-capability switches that are separate from the API surface.

Examples:
- default single-threaded profile: `[]`
- later threaded profile: `wasm-threads`

CLI spelling: `--wasm-threads`

Config spelling: `compilerOptions.runtimeProfiles`

### Compatibility feature
An explicitly opted-in language/runtime compatibility escape hatch.

Schema-v1 stable name:
- `eval`

CLI spelling: `--compat eval`

Config spelling: `compat.features`

### Direct-entry command
A command that requires explicit entrypoint arguments and must not guess a project default entry.

Current CLI-vocabulary members of this family:
- `run`
- `build`
- `effects`

### Hybrid analysis command
A command that accepts explicit files, or falls back to project discovery when invoked without them.

Current CLI-vocabulary members of this family:
- `check`

### Project-oriented command
A command whose primary no-argument behavior is defined in terms of canonical project discovery over source files rather than a required explicit entrypoint.

Current CLI-vocabulary members of this family:
- `fmt`
- `lint`
- `test`

### Dependency-graph command
A command whose no-argument behavior is defined in terms of the discovered project dependency graph rather than a required explicit source entrypoint.

Current CLI-vocabulary members of this family:
- `install`

### Registry-analysis command
A command that analyzes one explicit registry package identifier rather than discovered source files or the whole project graph.

Current CLI-vocabulary members of this family:
- `package-effects`
- `package-audit`

Note:
- `check` is still the canonical **hybrid analysis command**
- when invoked without explicit files, `check` also uses canonical project discovery
- `install` also uses canonical project discovery when it needs to scan source files for raw URL imports
- when `package-effects` and `package-audit` are available, they stay single-package registry-analysis commands rather than growing an implicit whole-project mode
- in early phases, registry-analysis commands also avoid a second per-command `--api` / `--compat` flag family: `package-effects` reuses the inherited analysis context, and `package-audit` stays a single-package registry tool rather than a second host-mode selector
- this keeps each command in one primary category and avoids overlapping near-duplicate workflows

## Canonical Default Tuple

Unless explicitly overridden by CLI or config, command examples use:
- `apiSurface = deno`
- `buildMode = fast`
- `runtimeProfiles = []`
- `compat.features = []`

This tuple is the default interpretation for examples such as:
- `kali run main.ts`
- `kali build main.ts`
- `kali check main.ts`
- `kali test`
- later `kali effects main.ts`

## Effective Command Context Rule

Every command first computes one **effective command context** by applying:
1. built-in defaults
2. the discovered `kali.json`
3. explicit CLI flags

Interpretation rules:
- availability checks and contradiction checks always evaluate this **effective** context, not just the literal CLI spelling
- Kali must not silently rewrite the effective context just to make a command succeed
- if the effective context requests a real but unavailable feature/profile, fail with `E5006`
- if the effective context creates a contradictory command shape, fail with `E5008`

Canonical examples:
- if `kali.json` sets `compilerOptions.apiSurface = "node"`, then plain `kali run main.ts` still hits the same Node phase gate as `kali run --api node main.ts`
- if `kali.json` sets `compilerOptions.apiSurface = "browser"`, then plain `kali build main.ts` is still invalid early-phase command usage until `--bundle` is selected, just like `kali build --api browser main.ts`
- commands must not silently fall back from config-selected `browser`/`node` to `deno`

## Canonical Source-File Sets

### Executable/analyzable source set
These files can be used as runtime/build/effect entrypoints:
- `.ts`
- `.tsx`
- `.mts`
- `.cts`
- `.js`
- `.jsx`
- `.mjs`
- `.cjs`

### Declaration-only source set
These files are valid analysis/type-loading inputs, but not runtime/build/effect/test entrypoints:
- `.d.ts`
- `.d.mts`
- `.d.cts`

### Canonical project file set
The union of:
- executable/analyzable source set
- declaration-only source set

Command intent narrows from this set:
- runtime-bearing entrypoints use only the executable/analyzable source set
- `check`, `fmt`, and `lint` may operate on the full canonical project file set
- discovered test entrypoints use only the executable/analyzable source set

## Canonical Project Root and Discovery

### Project root
The effective project root is:
1. the directory containing the nearest `kali.json` found by searching the current working directory and then its ancestors, or
2. the current working directory if no `kali.json` exists

Relative paths in `kali.json` resolve relative to the directory containing that config.
Ordinary CLI path arguments resolve relative to the current working directory.

### Discovery walk
When a command uses project discovery, it should:
1. start at the effective project root
2. recursively walk files in that tree
3. stop recursion at nested child directories that contain their own `kali.json`, unless the user explicitly targeted files inside them
4. collect files from the canonical project file set
5. apply `include` / `exclude` filters from the effective `kali.json` when present

### Default excluded managed/generated directories
When discovery runs without an overriding `include` / `exclude` rule that explicitly brings them back, it should skip these directories by default:
- `.git/`
- `.jj/`
- `.svn/`
- `.hg/`
- `node_modules/`
- `.kali/`
- `dist/`
- `build/`
- `target/`
- `coverage/`

This keeps project discovery stable and avoids accidentally treating generated or managed dependency state as source input.

### Command-specific discovery narrowing
From the canonical project-discovery result:
- `check` uses the discovered file set directly
- `fmt` and `lint` use the discovered file set directly
- `test` matches `*.test.*` and `*_test.*` only across discovered executable/analyzable files
- `install` may scan the discovered file set, including declaration-only files, for source-level raw URL imports as part of dependency-graph reconciliation

## Canonical Command/Input Shape Rules

### Direct-entry commands
In early phases:
- `run`, `build`, and `effects` each take **exactly one** explicit primary entrypoint
- zero entrypoints is invalid usage (`E5008`)
- more than one explicit entrypoint is invalid usage (`E5008`)

### Input-kind rule
- declaration-only files are valid direct inputs for `check`, `fmt`, and `lint`
- declaration-only files are never valid entrypoints for `run`, `build`, `effects`, or `test`
- passing a declaration-only file where an executable entrypoint is required is the canonical invalid-entrypoint diagnostic (`E5007`)

### Package-argument rule
In early phases:
- `kali install [package]` accepts zero or one explicit package argument
- `kali package-effects <package>` accepts exactly one explicit registry-package argument
- `kali package-audit <package>` accepts exactly one explicit registry-package argument

## Artifact-Mode Matrix

`kali build` has one canonical early-phase artifact selector family:
- omitted selector → **default executable mode**
- `--bundle` → **browser bundle mode**
- `--lib` → **library mode**
- `--capi` → **public C embedding mode** *(Phase 2 target)*
- `--component` → **Component Model packaging mode** *(Phase 2 target)*

These selectors are mutually exclusive unless a later spec explicitly says otherwise.

### Canonical artifact matrix

| Build invocation shape | Artifact mode | Core artifact contract | Early-phase status |
|---|---|---|---|
| `kali build main.ts` | default executable | one linked `wasm-module` with role `primary-executable` | Phase 1 MVP |
| `kali build --bundle --api browser main.ts` | browser bundle | one linked `wasm-module` with role `primary-executable` plus browser JS glue with role `browser-glue` | Phase 1 MVP |
| `kali build --lib lib.ts` | library | one linked export-oriented `wasm-module` with role `primary-library`; no synthetic executable entry is invoked, and later public-library outputs also emit WIT | Phase 1 MVP |
| `kali build --capi lib.ts` | C embedding package | library core + WIT + generated C header + C-ABI metadata | Phase 2 target |
| `kali build --component lib.ts` | Component package | library core + WIT + wrapped `wasm-component` | Phase 2 target |

Interpretation rules:
- `--bundle` is **browser-only** in early phases and requires the effective `apiSurface` to be `browser`
- `--lib`, `--capi`, and `--component` are non-browser artifact modes in early phases
- library-oriented modes are **export-oriented**: they package the module's explicit exports for host use, omit any synthetic executable entry invocation, and still preserve ordinary ECMAScript module-instantiation semantics for top-level initialization when the host instantiates the module
- WIT is an output detail of public library/embedding/component modes, not a separate selector
- companion artifacts do not weaken the single linked-payload rule for the compiled program graph itself

## Canonical Host/API Summary

API-surface clarification:
- `deno`, `node`, and `browser` are always the canonical **API surface** names
- selecting an API surface does **not** by itself promise that every command/artifact/runtime combination exists for that surface
- in early phases, `browser` is primarily a browser-targeted analysis/build surface, while `node` is broadly phase-gated
- command/profile availability is determined by the combination of command shape, artifact mode, API surface, and runtime profile rather than by any one axis alone


### Shared Web baseline
The early shared baseline available across supported surfaces is intentionally small and capability-oriented. It includes the documented Web-platform baseline from [specs/11-standard-apis.md](./specs/11-standard-apis.md), such as:
- `fetch`
- timers
- `console`
- `URL` / `URLSearchParams`
- `TextEncoder` / `TextDecoder`
- `AbortController` / `AbortSignal`
- `EventTarget` / `Event` / `CustomEvent`
- `structuredClone`
- randomness via the small documented subset (`crypto.getRandomValues`)

### Surface summary table

| API surface | Early standalone/runtime meaning | Early analysis/build meaning |
|---|---|---|
| `deno` | canonical supported standalone surface | canonical default analysis surface |
| `browser` | not a standalone runtime promise in early phases | browser-targeted context for `check --api browser` and `build --bundle --api browser` |
| `node` | phase-gated | phase-gated |

Interpretation rules:
- browser-targeted analysis/build may expose real browser ambient typings, including DOM typings, without implying that Kali embeds a browser runtime
- browser-targeted builds rely on emitted JS glue plus the real browser host
- supported command/profile combinations should use the selected surface faithfully; unsupported ones fail explicitly rather than silently falling back

## Single-Payload and Dynamic-Loading Boundary

### Single-payload rule
Early-phase Kali compiles the full static module graph into one linked core WASM payload per build artifact.

Companion artifacts may still exist:
- browser JS glue
- WIT files
- C headers
- C-ABI metadata
- component wrappers
- source maps

### Dynamic-loading rule
- static `import` / `export` are part of the core model
- static CommonJS lowering is part of the core package-compatibility story
- literal-string `import()` is a later lowering over the already-linked graph, not permission for runtime WASM module linking
- non-literal `import(expr)` is a later compatibility path and a dynamic effect boundary
- dynamic `require()` is rejected by default in early phases

## Canonical Representation-Downgrade Ladder

Layout precision and ownership are separate axes.
A value may move from stack to owned heap or shared heap without losing a precise static layout.

When Kali must become more conservative about **representation/layout**, it should follow this ladder:
1. **Scalar/unboxed** — primitives and tightly known machine values
2. **Static structured layout** — fixed object/aggregate fields with known offsets
3. **Partially dynamic layout** — known fields plus a dynamic side slot for unknown properties
4. **Fully dynamic/hash-map layout** — dictionary-like or semantically open object representation

Interpretation rules:
- ownership changes alone do **not** force a layout downgrade
- shared ownership does **not** automatically imply hash-map representation
- crossing an `any`/dynamic boundary, computed property behavior, `delete`, `Proxy`, or other layout-destabilizing features may force a downgrade
- when analysis can still prove a closed shape, Kali should keep the more precise representation

This ladder is the shared representation contract referenced by [specs/05-ir.md](./specs/05-ir.md) and [specs/06-memory.md](./specs/06-memory.md).

## Sandbox-Domain Honesty Rule

Kali must distinguish between:
- **Kali-hosted execution enforcement** (`run`, `test`, and later embeddings under Kali-controlled hosts)
- **browser-targeted build/analysis compatibility checks** (`check --api browser`, `build --bundle --api browser`)

In early phases:
- browser-targeted commands may validate policy shape and capability compatibility
- they must **not** imply Kali-controlled post-deployment enforcement of CPU, memory, file, process, or thread budgets inside a real browser host

## Canonical Sandbox Attachment Matrix

This is the compact cross-spec meaning of attaching a sandbox policy.

| Invocation shape | Meaning of `--sandbox` / `kali.json#sandbox` |
|---|---|
| `kali run ...` / `kali test ...` | Validate the policy, then enforce it at runtime inside the Kali-hosted execution environment |
| `kali check ...` / `kali build ...` | Phase 1: validate policy schema/config only. Phase 2+: also validate inferred effects against the policy |
| `kali check --api browser ...` / `kali build --bundle --api browser ...` | Static compatibility only; must not be described as Kali-controlled post-deployment browser enforcement, and non-deny `resources.*` budgets are rejected |
| `kali effects ...` / `kali package-effects ...` | Reporting only; they do not take `--sandbox` in early phases |
| embedding with a Kali-controlled host | Same enforcement model as `run`/`test`, plus later embedding-only host-predicate extensions when that feature exists |

Trusted-policy-extension rule:
- the bootstrap goal of programmable sandbox conditions is satisfied only through the later **host-registered predicate** path
- project policy files remain declarative data in schema v1
- Kali must not execute project code just to decide whether a capability is allowed

## Machine-Readable Contract Rule

When Kali exposes machine-readable output:
- JSON is the stable tooling contract
- top-level machine-readable JSON documents carry `schemaVersion`
- `kali effects` and `kali package-effects` may emit their native payloads directly by default
- `--output json` wraps command results in the standard command envelope from [specs/18-schemas.md](./specs/18-schemas.md)
- machine-emitted arrays should use deterministic canonical ordering wherever the producer owns the order, so AI/tooling diffs do not depend on traversal or hash-map iteration order

## Cross-Spec Simplicity Rules

When a new feature is added, prefer:
- one canonical name per concept
- one command path for a workflow rather than overlapping near-duplicates
- explicit rejection over undocumented fallback
- extending existing artifact/effect/policy schemas rather than inventing parallel formats
- phase-gated honesty over partial compatibility claims

## Spec Map

- [01 — Architecture](./specs/01-architecture.md)
- [02 — Lexer & Parser](./specs/02-lexer-parser.md)
- [03 — AST](./specs/03-ast.md)
- [04 — Type System](./specs/04-type-system.md)
- [05 — Intermediate Representations](./specs/05-ir.md)
- [06 — Memory Management](./specs/06-memory.md)
- [07 — Specialization](./specs/07-specialization.md)
- [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- [09 — Sandboxing](./specs/09-sandboxing.md)
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
