# Kali Specification

This document is the top-level contract for the Kali spec set. It defines the canonical terminology and cross-cutting rules that other chapters reference instead of restating.

Detailed subsystem design lives in [`specs/`](./specs).

## Chapter Guide

The top-level spec stays intentionally short on subsystem internals. The table below is the canonical map from the bootstrap brief to the detailed chapter set so readers can jump to the right owner quickly.

| Chapter | Owns | Why it exists in the bootstrap breakdown |
|---|---|---|
| [01 — Architecture](./specs/01-architecture.md) | crate boundaries, pipeline stages, global design posture | anchors the AOT-only, pure-Rust, sandbox-first compiler/runtime architecture |
| [02 — Lexer & Parser](./specs/02-lexer-parser.md) | syntax acceptance, parsing strategy, source compatibility posture | covers latest-published ECMA-262 grammar tracking and fast frontend behavior |
| [03 — AST](./specs/03-ast.md) | syntax tree shape and source-preserving frontend representation | keeps parsing/output contracts separate from later semantic lowering |
| [04 — Type System](./specs/04-type-system.md) | stronger-than-TS checking, inference, effect typing hooks | resolves the “TypeScript superset with more inference/constraints” goal |
| [05 — IR](./specs/05-ir.md) | compiler IR layers and lowering boundaries | captures the explicit-memory-layout, optimization-friendly middle-end story |
| [06 — Memory Management](./specs/06-memory.md) | ownership, allocation class, no-tracing-GC rules | owns the Rust-like compile-time allocation/borrowing direction |
| [07 — Optimization & Specialization](./specs/07-specialization.md) | generic specialization and optimization cost controls | owns aggressive specialization without hiding compile-time cost |
| [08 — WASM Codegen](./specs/08-wasm-codegen.md) | artifact lowering to linked WebAssembly outputs | turns the AOT compiler story into concrete executable/library artifacts |
| [09 — Sandboxing & Effects](./specs/09-sandboxing.md) | capability model, policy validation, effect reporting semantics | owns the sandbox-first and effect-analysis contract |
| [10 — Runtime](./specs/10-runtime.md) | execution engine, host ABI, event loop, dynamic-compatibility runtime behavior | owns wasmtime-first execution and long-term dynamic compatibility paths |
| [11 — Standard APIs](./specs/11-standard-apis.md) | Deno/Node/browser API-surface layering | keeps host compatibility promises phased and auditable |
| [12 — CLI](./specs/12-cli.md) | commands, flags, output behavior, command examples | owns the Deno-like AI-friendly CLI contract |
| [13 — Embedding](./specs/13-embedding.md) | Rust embedding, C ABI, WIT, Component Model packaging | owns the embeddability/public-library story |
| [14 — Package Management](./specs/14-packages.md) | npm/JSR/raw-URL resolution, install mutability, lock/materialization | owns ecosystem access without hidden dependency mutation |
| [15 — Errors](./specs/15-errors.md) | diagnostic boundaries and stable error-code meanings | owns AI-friendly, machine-parseable failure behavior |
| [16 — Testing](./specs/16-testing.md) | conformance strategy and evidence requirements | keeps support claims tied to tests instead of aspiration |
| [17 — Formal Verification](./specs/17-verification.md) | Lean verification scope and proof methodology | owns the “verify critical pieces while iterating” requirement |
| [18 — Schemas](./specs/18-schemas.md) | stable machine-readable JSON/config/policy contracts | centralizes all schema promises so CLI/docs do not drift |
| [19 — Feature Maturity](./specs/19-feature-maturity.md) | phase labels, command/profile availability, gating matrix | is the single source of truth for what is actually available when |

Simplification rule:
- when a question is “where is this designed?”, use the table above
- when a question is “is this available yet?”, use [19 — Feature Maturity](./specs/19-feature-maturity.md)
- when a question is “what does the JSON look like?”, use [18 — Schemas](./specs/18-schemas.md)

## Canonical Chapter Ownership

To keep the spec set easier to maintain and avoid near-duplicate rules drifting apart, each cross-cutting topic has one primary owner:

| Topic | Primary owner | Other chapters should mainly do |
|---|---|---|
| Top-level terminology, command families, default tuple, command-context rules, artifact matrix | [SPEC.md](./SPEC.md) | Reuse the canonical terms and link back instead of redefining them |
| Feature/phase availability and the canonical `E5006` gating boundary | [19 — Feature Maturity](./specs/19-feature-maturity.md) | Reference the matrix/status row instead of restating a new promise |
| CLI flags, command shapes, output behavior, exit codes | [12 — CLI](./specs/12-cli.md) | Reuse the command/flag contracts rather than inventing command-local variants |
| Package resolution, install mutability, lock/materialization rules | [14 — Package Management](./specs/14-packages.md) | Reuse the package-resolution and dependency-state rules |
| Machine-readable JSON contracts | [18 — Schemas](./specs/18-schemas.md) | Link to the schema instead of repeating object shapes inline |
| Diagnostics/code boundaries (`E5004`, `E5006`, `E5007`, `E5008`, `E5009`, `E5010`) | [15 — Errors](./specs/15-errors.md) | Reuse the canonical boundary rather than redefining new error semantics |
| Sandbox policy meaning, enforcement domains, effect/policy interaction | [09 — Sandboxing & Effects](./specs/09-sandboxing.md) | Reuse the same policy/enforcement model |

Interpretation rule:
- if two chapters appear to describe the same cross-cutting rule at different levels of detail, the primary owner above wins unless a later spec revision changes ownership explicitly
- non-owner chapters should prefer short cross-references over re-explaining the full rule

## Purpose

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for:
- fast compilation and execution
- sandbox-first execution
- strong static analysis, including effect analysis
- AI-friendly CLI and diagnostics
- pure-Rust implementation and embeddability

## Bootstrap Normalization Rule

`BOOTSTRAP.md` is the design input brief, but this spec set is the normative contract.

Interpretation rules:
- the bootstrap brief mixes **long-term goals**, **phase-1 expectations**, and **implementation preferences** in one list
- this top-level spec and the chapter set normalize that brief into explicit phase promises, non-goals, and machine-readable contracts
- when bootstrap wording and a phased chapter promise appear to pull in different directions, **SPEC.md** plus the relevant owning chapter win
- docs should therefore avoid citing the bootstrap brief as though it were a second parallel source of truth once the normalized rule already exists here

This rule keeps the repository simple: one bootstrap input, one normalized spec set, and one feature-maturity matrix.

## Bootstrap Requirement Map

This is the compact top-level breakdown of the bootstrap brief into canonical spec areas.

| Bootstrap concern | Canonical handling |
|---|---|
| AOT-only TS/JS → WASM compiler | [01 — Architecture](./specs/01-architecture.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md) |
| Latest ECMA-262 grammar coverage, broad syntax acceptance | [02 — Lexer & Parser](./specs/02-lexer-parser.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| First-class JavaScript compilation with conservative inference | [01 — Architecture](./specs/01-architecture.md), [04 — Type System](./specs/04-type-system.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
| Stronger-than-TS checking and inference | [04 — Type System](./specs/04-type-system.md) |
| No tracing GC; compile-time ownership/allocation | [06 — Memory Management](./specs/06-memory.md) |
| Aggressive specialization | [07 — Optimization & Specialization](./specs/07-specialization.md) |
| Sandboxing and effect-aware execution | [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md) *(declarative policy first; later trusted host predicates for embedding)* |
| JSON effect reporting and policy schemas | [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [18 — Schemas](./specs/18-schemas.md), [19 — Feature Maturity](./specs/19-feature-maturity.md) |
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
- **Single linked core WASM payload** for the resolved static graph in Phases 1-3
- **No silent fallback** for unsupported semantics or unsupported host/profile combinations
- **Stable machine-readable contracts** for every machine-readable surface Kali exposes (JSON output, diagnostics, effect reports, artifact metadata, config, and policy schemas)
- **One mutating dependency-management command** in early phases: `kali install` is the only command that writes project dependency state; `kali init` may scaffold a config file, but it does not add dependencies, write `kali.lock`, or materialize packages
- **Reproducible by default**: given the same source graph, config, lockfile, effective command context, and Kali version/toolchain, builds should produce byte-stable machine-readable outputs and artifact bytes unless the user explicitly opts into non-reproducible metadata

## Reference Inspirations

Kali should take implementation and design inspiration from projects such as:
- [Boa](https://github.com/boa-dev/boa)
- [V8](https://github.com/v8/v8)
- JavaScriptCore
- SpiderMonkey
- [Deno](https://github.com/denoland/deno)
- [TypeScript / `tsc`](https://github.com/microsoft/TypeScript)
- [Porffor](https://github.com/CanadaHonk/porffor)
- [Hermes](https://github.com/facebook/hermes)
- [Bun](https://github.com/oven-sh/bun)
- Rust, Haskell, Idris, Agda, and Lean for language/type-system design inspiration

Interpretation rule:
- these are **reference points**, not blanket compatibility or implementation-parity promises
- Kali should copy proven ideas where they fit the goal-precedence rules below, while keeping one coherent sandbox-first, AOT-first, pure-Rust design

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

### Published-standard boundary

The bootstrap goal “support the latest ECMA-262 standard” is scoped to the **latest published edition** of ECMA-262.

Interpretation rules:
- that goal does **not** automatically include in-flight TC39 proposals or draft text beyond the latest published edition
- proposal support, when Kali chooses to experiment with it, must be called out explicitly in the feature-maturity matrix or behind a documented experimental flag
- docs should avoid wording that makes “tracks the latest standard” sound like “implements every Stage-3+ proposal by default”
- once a proposal is incorporated into a published ECMA-262 edition, it moves back under the normal grammar/semantic/evidence claim model above

## Early-Phase Product Posture

These assumptions are intentionally explicit so the rest of the spec set does not drift:
- **standalone execution is Deno-first**
- **browser support is analysis/build first** in early phases (`check --api browser`, `build --bundle --api browser`)
- **Node compatibility is a later ecosystem phase**, not an MVP promise
- **all early builds target one linked core WASM payload** for the resolved static graph
- **companion artifacts are allowed**, but they do not change the single-payload rule
- **no tracing garbage collector** is introduced as a hidden fallback
- **no JIT**; Kali is AOT-only

### Deliberate Early-Phase Non-Goals

To keep the roadmap implementable and the specs honest, Kali should explicitly avoid implying these early:
- a standalone browser engine or DOM-emulating runtime
- full Node parity before the documented Phase 3 subset exists
- native addons, `node-gyp`, or install-time binary/bootstrap package contracts as part of Phase 1 package compatibility
- executable project policy code inside `kali.policy.json`
- automatic dependency installation or lockfile mutation during `check`, `effects`, `build`, `run`, or `test`

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

### Browser-targeted context
A command context whose **effective** `apiSurface` is `browser` for a command that actually supports browser targeting.

Interpretation rules:
- in Phase 1, the supported browser-targeted commands are `kali check --api browser` and `kali build --bundle --api browser`
- for `kali check --api browser`, this support covers both explicit-file invocations and the normal no-argument project-discovery form; browser targeting changes the analysis context, not the hybrid-input nature of `check`
- later analysis commands such as `kali effects --api browser` and inherited browser-context `kali package-effects` may reuse that same ambient/package-selection context once their own maturity rows allow it
- browser-targeted package resolution follows one shared rule across those commands: use the browser-specific `exports` condition order first, and when a package also publishes a `package.json#browser` replacement map, apply that same browser rewrite layer consistently instead of letting each command invent its own browser package-selection ladder
- this term names an **analysis/build context**, not a promise that Kali embeds a standalone browser runtime or DOM engine
- `run --api browser` and `test --api browser` therefore remain rejected until a later spec adds an explicit browser-runtime contract

### Canonical browser-surface rejection split
To keep browser-targeted support honest and machine-readable diagnostics consistent, Kali uses one cross-spec rule for early `--api browser` handling:
- `kali check --api browser ...` and `kali build --bundle --api browser ...` are the canonical supported early browser-targeted command shapes
- for `check`, that browser-targeted shape includes both explicit-file invocations and the ordinary no-argument project-discovery path
- if the command shape is **browser-targetable in principle** but the user selected an impossible early combination, the failure is **invalid command usage** (`E5008`); examples: plain `kali build --api browser main.ts`, `kali build --lib --api browser lib.ts`, `kali build --capi --api browser lib.ts`, `kali build --component --api browser lib.ts`
- if the user selected `--api browser` for a command that would require a standalone browser-runtime or test-runtime contract Kali does not yet define, the failure is **feature/profile unavailable** (`E5006`); examples: `kali run --api browser main.ts`, `kali test --api browser`
- commands must not silently fall back from an effective browser selection to `deno`

This keeps the browser story simple:
- **browser analysis/build contexts that exist but were requested with the wrong artifact shape** → `E5008`
- **browser execution/runtime contracts that do not exist yet** → `E5006`

### Canonical browser-targeted policy boundary
To keep sandbox wording consistent across CLI, sandboxing, API-surface, and maturity chapters, Kali uses one cross-spec rule for browser-targeted `--sandbox` handling:
- in early phases, the only browser-targeted sandbox-aware command shapes are `kali check --api browser --sandbox ...` and `kali build --bundle --api browser --sandbox ...`
- these are **static compatibility checks** over the documented **Kali-mediated capability subset** only; they do **not** imply Kali-hosted post-deployment runtime enforcement inside the real browser host
- capability-local `effects.*` policy keys remain valid only for that documented subset
- cross-cutting `resources.*` budgets that would imply Kali-hosted browser-deployment enforcement are rejected for this profile
- schema-v1 applies one concrete rejection rule: `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` are rejected whenever present; `resources.maxSpawnedProcesses` and `resources.maxThreads` reject positive values

This keeps the browser sandbox story simple:
- **browser-targeted capability compatibility within the documented mediated subset** → allowed to validate statically
- **browser-targeted post-deployment runtime-budget enforcement claims** → rejected until Kali defines a real browser-host contract for them

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

Interpretation rules:
- in schema v1, `eval` is the single stable compatibility-feature name for both direct `eval` and the `Function()` constructor path; Kali must not split those into separate flag/config names without an explicit later schema revision
- sandbox policy permission and compatibility-feature enablement are separate axes: allowing `effects.eval` in policy is only an authorization ceiling, not an implicit request to turn on `--compat eval` / `compat.features = ["eval"]`

CLI spelling: `--compat eval`

Config spelling: `compat.features`

### Analysis context
The semantic analysis tuple shared by analysis-oriented workflows:
- `apiSurface`
- `runtimeProfiles`
- `compat.features`

Interpretation rules:
- this is the compact cross-spec name for the semantic knobs that can change checking/effect results without changing artifact optimization mode
- `buildMode` is intentionally **not** part of the analysis context in early phases
- `package-effects` inherits this context from defaults/config instead of growing a second package-specific `--api` / `--compat` flag family
- early `package-audit` is intentionally **context-free** and does not inherit this analysis context

### Canonical naming bridge: config vs emitted reports
Kali keeps one semantic vocabulary even when config and emitted JSON use slightly different field shapes.

Canonical rule:
- config stores compatibility features under `compat.features`
- machine-emitted `analysisContext` objects flatten that field to `compatFeatures`
- this is a shape simplification for self-contained JSON payloads, not a second concept or a second compatibility namespace

Interpretation rule:
- prose may refer to the semantic axis as **compatibility features** or `compat.features`
- when a chapter is describing the exact emitted JSON field name, it should say `compatFeatures`

### Canonical naming bridge: logical roots vs `entryPoints`
Kali keeps one shared report concept even though the schema-v1 field name is historically runtime-flavored.

Canonical rule:
- prose should refer to the report roots as **logical roots** when the discussion is not specifically about executable runtime entrypoints
- schema-v1 effect-family payloads keep the field name `entryPoints`
- in `kali effects`, those `entryPoints` are normally the explicit analysis-root labels
- in `kali package-effects`, those `entryPoints` are normally the package-root labels such as `lodash` or `jsr:@std/path`

Interpretation rule:
- `entryPoints` is a stable machine-readable field name, not a claim that every producer is describing a runtime entrypoint
- chapters should therefore avoid re-explaining it as if package analysis or other report producers were forced into runtime-entrypoint terminology

### Direct-input command
A command that requires exactly one explicit primary source input and must not guess a project default file.

Interpretation rule:
- for `run`, that source input is an executable entrypoint
- for `build`, that source input is one explicit primary module input whose artifact role depends on the selected artifact mode
- for `effects`, that source input is one explicit analysis root

Current CLI-vocabulary members of this family:
- `run`
- `build`
- `effects`

### Hybrid analysis command
A command that accepts explicit files, or falls back to project discovery when invoked without them.

Current CLI-vocabulary members of this family:
- `check`

### Project-oriented command
A command whose primary no-argument behavior is defined in terms of canonical project discovery over source files rather than a required explicit source input.

Current CLI-vocabulary members of this family:
- `fmt`
- `lint`
- `test`

### Dependency-graph command
A command whose no-argument behavior is defined in terms of the discovered project dependency graph rather than a required explicit source input.

Current CLI-vocabulary members of this family:
- `install`

### Registry-analysis command
A command that analyzes one explicit registry package identifier rather than discovered source files or the whole project graph.

Interpretation rules:
- schema-v1 registry-analysis commands use the shared **identity-only registry target** form and its stable-release selection rule rather than consulting the current project's manifest or lockfile to pick a version
- `package-effects` may still inherit the effective analysis context (`apiSurface`, `runtimeProfiles`, `compat.features`), but that inherited analysis context does **not** change the project-independent version-selection rule
- `package-audit` stays context-free in early phases and is likewise project-independent for version selection

Current CLI-vocabulary members of this family:
- `package-effects`
- `package-audit`

### Install target
The optional single explicit argument accepted by `kali install`.

Schema-v1 install target kinds:
- **identity-only registry target** — a registry package identifier such as `lodash`, `@scope/name`, or `jsr:@std/path`
- **raw URL target** — an exact URL dependency such as `https://deno.land/std/path/mod.ts`

Interpretation rules:
- in early phases, `kali install` accepts zero or one explicit install target
- registry-target installs may update `dependencies` / `devDependencies`; raw-URL installs stage/pin shared lock/cache state only
- flags such as `--dev` apply only to the registry-target form, not to raw URL targets
- adding explicit version/range selectors later must be a separate documented target form rather than inferred from the identity-only registry form

### Identity-only registry target
An explicit registry package argument that names a package identity but not an inline version/range selector.

Schema-v1 workflows using this form:
- `kali install <pkg>`
- `kali install --dev <pkg>`
- `kali package-effects <pkg>`
- `kali package-audit <pkg>`

Interpretation rules:
- this form uses the shared stable-release selection rule from [14 — Package Management](./specs/14-packages.md)
- if no non-yanked stable release exists for that package identity, the workflow fails with the canonical package-selection diagnostic `E5001` rather than silently selecting a prerelease or consulting ambient project lockfile state
- it resolves from registry/package metadata rather than from an ambient project lockfile choice unless a later spec adds an explicit lock-aware or version-selecting mode
- adding explicit version/range selectors later must be a separate documented input mode rather than inferred from the identity-only form

### Effective npm-scriptable install work
The subset of one `kali install` invocation that targets npm registry packages and could therefore expose npm lifecycle hooks.

Interpretation rules:
- this subset may be smaller than the invocation's total install work; JSR packages, raw URLs, and no-op/project-reconciliation work outside npm stay outside it in schema v1
- this subset is **invocation-scoped**: it includes the npm package work the current install actually reconciles, including any directly requested npm target and any transitively touched npm dependencies in that same invocation
- `kali install --allow-scripts` affects only this subset
- mixed graphs are valid: when one install touches both npm and non-npm work, lifecycle scripts may run only for the npm subset while the non-npm subset remains on the normal script-free path
- if the subset is empty, `--allow-scripts` is invalid usage (`E5008`) rather than a silent no-op

Note:
- `check` is still the canonical **hybrid analysis command**
- when invoked without explicit files, `check` also uses canonical project discovery
- `install` also uses canonical project discovery when it needs to scan source files for raw URL imports
- when `package-effects` and `package-audit` are available, they stay single-package registry-analysis commands rather than growing an implicit whole-project mode
- in early phases, registry-analysis commands also avoid a second per-command **analysis-context flag family**: `package-effects` reuses the inherited analysis context instead of taking package-analysis-specific `--api`, runtime-profile flags, or `--compat` switches, while `package-audit` stays **context-free** (registry/package metadata focused) rather than becoming a second host-mode selector
- config-selected `apiSurface`, `runtimeProfiles`, and `compat.features` therefore influence `package-effects`, but they do not change the semantics of early `package-audit`
- unsupported inherited analysis-context values for `package-effects` fail with the same canonical availability path (`E5006`) used by direct analysis commands; Kali must not silently drop an inherited `node`, `wasm-threads`, or later compatibility feature just because `package-effects` has no parallel flag family of its own
- registry-analysis commands may still use ordinary project/config discovery for generic CLI behavior, but that discovery does **not** change the schema-v1 stable-release version-selection rule and does **not** permit mutation of project-managed dependency state
- for clarity, early `package-audit` still uses ordinary project/config discovery plus generic CLI behavior (for example project root selection, `--output`, and `--quiet`), but it intentionally ignores host-analysis/runtime knobs such as `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox`
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
- validation order is canonical across the spec set: check **command shape/arity first**, then the command's own base availability, then narrower inherited-context/profile gates inside that command
- this ordering keeps diagnostics stable for commands with inherited context; for example, before `package-effects` exists, plain `kali package-effects lodash` fails on the command row itself, while once the command exists an unsupported inherited `apiSurface = node` or `compat.features = ["eval"]` can fail on the narrower `E5006` gate instead
- a command may still document that some context axes are intentionally **non-semantic** for it in early phases; for example early `package-audit` still uses ordinary project/config discovery plus generic CLI behavior, but it intentionally ignores host-analysis/runtime knobs such as `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox`

Canonical examples:
- if `kali.json` sets `compilerOptions.apiSurface = "node"`, then plain `kali run main.ts` still hits the same Node phase gate as `kali run --api node main.ts`
- if `kali.json` sets `compilerOptions.apiSurface = "browser"`, then plain `kali build main.ts` is still invalid early-phase command usage until `--bundle` is selected, just like `kali build --api browser main.ts`
- if `kali.json` sets `compilerOptions.apiSurface = "browser"`, then plain `kali run main.ts` and plain `kali test` still hit the same browser-runtime/test gate as their explicit `--api browser` forms (`E5006`), because browser remains an analysis/build context first
- commands must not silently fall back from config-selected `browser`/`node` to `deno`

## Canonical Command-Context Axis Participation

The effective command context has one shared vocabulary, but not every command treats every axis as semantic in early phases.

| Command family | `apiSurface` | `buildMode` | `runtimeProfiles` | `compat.features` | `sandbox` |
|---|---|---|---|---|---|
| `run` | semantic | semantic | semantic | semantic | semantic |
| `test` | semantic | semantic | semantic | semantic | semantic |
| `build` | semantic | semantic | semantic | semantic | semantic |
| `check` | semantic | non-semantic | semantic when analysis depends on a runtime profile | semantic | semantic |
| `effects` | semantic | non-semantic | semantic | semantic | ignored in early phases |
| `package-effects` | semantic via inherited analysis context | non-semantic | semantic via inherited analysis context | semantic via inherited analysis context | ignored in early phases |
| `package-audit` | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases |
| `install` | CLI `--api` invalid; inherited config ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases |
| `fmt` / `lint` | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases |
| `init` | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases | ignored in early phases |

Interpretation rules:
- **semantic** means the axis can change command behavior, availability, or machine-readable results
- **non-semantic** means the axis exists in the shared vocabulary but does not change that command's contract in early phases
- `package-effects` reuses the inherited **analysis** axes (`apiSurface`, `runtimeProfiles`, `compat.features`) rather than adding its own package-analysis-specific analysis-context flag family (`--api`, runtime-profile flags, `--compat`)
- `package-audit` is intentionally **context-free** in early phases: inherited config values may still be discovered for generic CLI behavior, but they do not change the audit semantics or result shape
- for `install`, “CLI `--api` invalid; inherited config ignored” means exactly that: an explicit `--api` flag is rejected with `E5008`, while config/default host-analysis settings may still be discovered but do not create a second install graph or change install semantics
- `install` remains profile-agnostic in early phases even when the project config contains host-analysis/runtime settings for other commands

This table is the cross-spec simplification rule for statements like “inherits analysis context”, “ignores sandbox”, or “does not take `--api`”: other chapters should reference this participation model instead of drifting into near-duplicate command-by-command wording.

Flag-family clarification:
- in early phases, the CLI `--sandbox` flag belongs only to the canonical sandbox-aware commands (`run`, `test`, `check`, `build`)
- commands that merely ignore top-level `kali.json#sandbox` do **not** thereby accept a CLI `--sandbox` flag; passing `--sandbox` to sandbox-agnostic or effect-reporting commands is invalid command usage (`E5008`) unless a later spec explicitly adds such a mode

## Canonical Source-Language Posture

Kali treats TypeScript and JavaScript as two first-class source-language modes over one shared compiler pipeline.

Canonical rules:
- `.ts` / `.tsx` / `.mts` / `.cts` and `.js` / `.jsx` / `.mjs` / `.cjs` all enter the same frontend pipeline
- JavaScript support is **not** a transpile-only or editor-hint-only side mode; it participates in real inference, effect analysis, lowering, and optimization
- when JavaScript code lacks enough information for a precise static conclusion, Kali falls back conservatively (`unknown`, unions, dynamic representations) instead of inventing fresh `any`
- declaration-only files remain analysis/type-loading inputs rather than executable runtime entrypoints or build/effect primary inputs

This section exists to keep the bootstrap requirement of efficient JavaScript compilation visible at the top level instead of letting it disappear into type-checker details.

## Canonical Source-File Sets

### Canonical scaffold entry filenames
To keep `kali init`, build examples, embedding examples, and docs aligned, schema v1 uses one small default filename convention for generated templates:
- default app template entry file: `main.ts`
- library template entry file: `lib.ts`

Interpretation rules:
- these are scaffold defaults, not magical filenames used for command inference
- direct-input commands still require explicit inputs and must not guess `main.ts` or `lib.ts`
- alternative filenames remain valid user choices after scaffolding; the convention only avoids template/documentation drift

### Executable/analyzable source set
These files can be used as runtime entrypoints and as build/effect primary source inputs:
- `.ts`
- `.tsx`
- `.mts`
- `.cts`
- `.js`
- `.jsx`
- `.mjs`
- `.cjs`

### Declaration-only source set
These files are valid analysis/type-loading inputs, but not runtime entrypoints, build/effect primary inputs, or test entrypoints:
- `.d.ts`
- `.d.mts`
- `.d.cts`

### Canonical project file set
The union of:
- executable/analyzable source set
- declaration-only source set

Command intent narrows from this set:
- runtime-bearing entrypoints use only the executable/analyzable source set
- build/effect direct inputs use only the executable/analyzable source set
- `check`, `fmt`, and `lint` may operate on the full canonical project file set
- discovered test entrypoints use only the executable/analyzable source set

## Canonical Project Root and Discovery

### Project root
The effective project root is:
1. the directory containing the nearest `kali.json` found by searching the current working directory and then its ancestors, or
2. the current working directory if no `kali.json` exists

Relative paths in `kali.json` resolve relative to the directory containing that config.
Ordinary CLI path arguments resolve relative to the current working directory.

### `init` root-selection exception
`kali init` is the one early-phase exception to ancestor-based config discovery.

Interpretation rules:
- `kali init` is **current-directory scoped**: it scaffolds a project rooted at the current working directory rather than reusing the nearest ancestor project's root
- if the current working directory already contains `kali.json`, `kali init` fails with `E5008` instead of silently overwriting the existing project config
- if an ancestor directory contains `kali.json` but the current working directory does not, `kali init` may create a **nested child project** rooted at the current working directory
- once created, that child project follows the normal nested-project boundary rule from the discovery walk below

This exception keeps project creation simple and avoids the surprising behavior where `kali init` invoked in a subdirectory would mutate an ancestor project.

### Configless project mode
When no `kali.json` is discovered, Kali runs in a **configless project mode** rooted at the current working directory.

Interpretation rules:
- built-in defaults still provide the effective command context
- ordinary project discovery still starts from that current working directory
- explicit registry-package adds (`kali install <pkg>` / `kali install --dev <pkg>`) are the only early-phase workflow that auto-creates a minimal manifest in this mode
- explicit raw-URL pin/materialize workflows (`kali install https://...`) may still create `kali.lock` and `.kali/cache/urls/` state at that root, but they do **not** create a placeholder manifest by themselves
- plain `kali install` in configless mode is a no-op success when the current root contributes no manifest/import/source dependency inputs; running `install` by itself is not treated as an implicit scaffolding request
- commands must not invent a second hidden config file or dependency-state sidecar just because they ran in configless mode

### Discovery walk
When a command uses project discovery, it should:
1. start at the effective project root
2. recursively walk files in that tree
3. stop recursion at nested child directories that contain their own `kali.json`; those child roots are separate projects in schema v1
4. collect files from the canonical project file set
5. apply `include` / `exclude` filters from the effective `kali.json` when present

### Explicit target boundary
Explicit CLI file/path targets do **not** relocate the chosen config/root.

Schema-v1 simplification:
- explicit file/path targets for file-accepting source commands (`run`, `build`, `check`, `effects`, `fmt`, `lint`, `test`) must resolve inside the effective project root
- they must **not** point into a nested child project that has its own `kali.json`
- to operate on that child project, invoke Kali from that child project root (or one of its subdirectories) instead of reaching across project boundaries from the parent
- explicit file/path targets are accepted by explicit user selection even when they would not have been discovered by `include` / `exclude`; those filters constrain discovery, not the meaning of an already-explicit target

This keeps config selection, discovery, lockfile ownership, and diagnostics aligned around one project root per invocation without making explicitly named targets depend on discovery-only filters.

Dependency-state clarification:
- explicit file/path targets may legally sit outside discovery-glob coverage, but plain `kali install` still discovers raw URL dependencies from the canonical project-discovery result rather than from every future ad hoc explicit target
- therefore a later file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) may still fail with `E5004` if an explicit target outside the last installed discovery set reaches additional raw URL imports
- the fix is to make that source reachable from the install-time declaration graph (for example by widening `include` / `exclude` or adding the relevant import-map/source declaration) and then rerun `kali install`; non-install commands must not auto-install opportunistically

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
- explicit `kali test [files...]` arguments bypass that naming-pattern discovery filter and are treated as direct test-module inputs, but they must still come from the executable/analyzable source set
- `install` may scan the discovered file set, including declaration-only files, for source-level raw URL imports as part of dependency-graph reconciliation

## Canonical Command/Input Shape Rules

### Direct-input commands
In early phases:
- `run`, `build`, and `effects` each take **exactly one** explicit primary source input
- zero explicit source inputs is invalid usage (`E5008`)
- more than one explicit source input is invalid usage (`E5008`)

Interpretation rule:
- for `run`, that source input is the executable entrypoint
- for `build`, it is the primary module input for the selected artifact mode
- for `effects`, it is the primary analysis root

### Input-kind rule
- declaration-only files are valid direct inputs for `check`, `fmt`, and `lint`
- declaration-only files are never valid direct inputs for `run`, `build`, `effects`, or `test`
- passing a declaration-only file where an executable entrypoint or build/effect primary input is required is the canonical invalid-entrypoint diagnostic (`E5007`)

### Install-target and package-argument rule
In early phases:
- `kali install [target]` accepts zero or one explicit install target
- that install target may be either a schema-v1 **identity-only registry target** or a raw URL target
- `kali package-effects <package>` accepts exactly one explicit registry-package argument
- `kali package-audit <package>` accepts exactly one explicit registry-package argument
- those explicit registry-package arguments use the schema-v1 **identity-only registry target** form unless a later spec adds a separate version/range selector mode

## Canonical Command Participation Classes

### Sandbox-aware commands
These commands participate in the project sandbox contract:
- `run`
- `test`
- `check`
- `build`

Interpretation rules:
- top-level `kali.json#sandbox` applies only to this set
- `run` and `test` enforce the attached policy at runtime in Kali-hosted execution
- `check` and `build` validate sandbox policy shape/config in Phase 1 and add inferred-effect-vs-policy validation in Phase 2+

### Effect-reporting commands
These commands report effect information, but do not become alternate policy-validation entrypoints in early phases:
- `effects`
- `package-effects`

Interpretation rules:
- they do **not** accept `--sandbox` in early phases; passing it is invalid command usage (`E5008`)
- top-level `kali.json#sandbox` is ignored for them rather than being treated as an error
- `effects` reports over one explicit source input/analysis root; `package-effects` reports over one explicit registry package
- `package-effects` still inherits the effective analysis context

### Sandbox-agnostic commands
These commands do not participate in sandbox-policy attachment in early phases:
- `init`
- `fmt`
- `lint`
- `install`
- `package-audit`

Interpretation rules:
- top-level `kali.json#sandbox` is ignored for this set rather than being treated as an error
- early `package-audit` remains context-free with respect to `apiSurface`, `buildMode`, `runtimeProfiles`, and `compat.features`

## Canonical Dependency-Management Mutability Rule

Early-phase **project-managed dependency state** belongs to the effective project root and consists of:
- dependency-owning declaration fields in `kali.json` (`dependencies`, `devDependencies`, and `imports`)
- `kali.lock`
- `node_modules/`
- `.kali/cache/urls/`

Interpretation rules:
- `kali install` is the only command that mutates that project-managed dependency state
- `check`, `effects`, `build`, `run`, and `test` consume existing project-managed dependency state and fail with `E5004` when it is missing or stale
- `package-effects` and `package-audit` may use temporary analysis caches, but they do **not** mutate project-managed dependency state
- commands must not silently repair declaration/lock/materialization drift as a side effect of ordinary analysis or execution

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
| `kali build --lib lib.ts` | base library | one linked export-oriented `wasm-module` with role `primary-library`; no synthetic executable entry is invoked. This Phase-1 artifact establishes the library/export contract but does **not** by itself promise the later stable public embedding/WIT surface. In Phase 2+, the same selector is promoted into the stable public library contract and emits the default WIT sidecar. | Phase 1 MVP |
| `kali build --capi lib.ts` | C embedding package | library core + WIT + generated C header + C-ABI metadata | Phase 2 target |
| `kali build --component lib.ts` | Component package | library core + WIT + wrapped `wasm-component` | Phase 2 target |

Interpretation rules:
- `--bundle` is **browser-only** in early phases and requires the effective `apiSurface` to be `browser`
- `--lib`, `--capi`, and `--component` are **library-oriented artifact modes** in early phases: they are non-browser, export-oriented modes derived from a statically known module export surface
- plain `--lib` is the **base library artifact** in Phase 1: useful for internal/experimental host integration, but the stable public embedding/WIT contract is still Phase 2 work
- once that public contract stabilizes in Phase 2+, plain `--lib` becomes the canonical stable public library artifact and emits WIT by default; `--capi` and `--component` remain packaging layers over that same exported library surface rather than alternate reflection-based APIs
- the exported host-facing surface for every library-oriented mode comes only from the module's explicit exports after frontend lowering resolves the entry module to a fixed export set; these modes must not expose arbitrary internal declarations through reflection or artifact-specific special cases
- ESM entry modules satisfy this rule directly; CommonJS entry modules participate only when Kali's static CJS lowering can prove one fixed export set for the entry module, otherwise the library-oriented build must fail with `E5006` instead of inventing reflection-based exports
- library-oriented modes still obey the ordinary build-command API-surface gates: for example `kali build --lib --api node lib.ts` is a **Phase 3** Node build and therefore uses the same `E5006` gate as other early `--api node` builds, while `kali build --lib --api browser lib.ts` is an `E5008` contradiction because browser mode is only defined for `--bundle`
- library-oriented modes omit any synthetic executable entry invocation, but still preserve ordinary ECMAScript module-instantiation semantics for top-level initialization when the host instantiates the module
- WIT is an output detail of public library/embedding/component modes, not a separate selector
- companion artifacts do not weaken the single linked-payload rule for the compiled program graph itself

## Canonical WASM Engine Posture

To resolve the bootstrap prompt's open choice between `wasmtime` and `wasmer` without letting subsystem docs drift:
- **early-phase standardization is on `wasmtime`**
- this keeps runtime behavior, sandboxing, and embedding contracts anchored to one pure-Rust engine first
- a later engine abstraction may add `wasmer` or other backends only if they preserve the same user-visible sandbox/resource/diagnostic contracts
- Kali remains **AOT-only** regardless of backend; an engine's internal translation or caching strategy is not permission for language-level JIT design

This is the canonical top-level resolution of the engine choice so architecture/runtime docs do not each restate it differently.

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

### Browser ambient typing vs mediated capability split

To keep the bootstrap goal of “browser API support” honest and implementation-friendly, Kali treats two browser-facing concerns separately:
1. **browser ambient typing surface** — the globals/types visible during supported browser-targeted analysis/build commands, including normal DOM typings
2. **Kali-mediated capability model** — the smaller built-in capability subset used for stable effect reports, sandbox policy keys, and Kali-controlled runtime enforcement

Interpretation rules:
- `--api browser` for supported analysis/build commands selects the real browser ambient typing surface; it is not limited to only the stable sandbox/effect capability keys
- the stable sandbox/effect contract still talks only about the **Kali-mediated capability subset** defined below rather than one key per DOM/browser API
- this split is why browser-targeted `check` / `build --bundle` can type-check ordinary browser programs while still avoiding any claim that Kali embeds a standalone browser runtime or mediates every browser-host operation after deployment
- when docs mention browser support, they should say whether they mean **ambient typing/build targeting** or **stable sandbox/effect coverage** instead of collapsing both into one vague promise

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

## Canonical Kali-Mediated Capability Subset

To keep the browser-targeted sandbox/effect story precise, Kali uses one cross-spec term for the capabilities it models directly in schema v1.

The **Kali-mediated capability subset** is the built-in capability family shared by:
- stable effect reports
- sandbox policy keys
- runtime host enforcement where Kali controls the host
- browser-targeted static compatibility checks

In schema v1 this subset is the built-in capability namespace represented by:
- filesystem
- network (`fetch`, later `connect` / `listen`)
- process (`envRead`, later `envWrite` / `spawn`)
- timers
- randomness
- console
- `eval`

Interpretation rules:
- observation-only compatibility facades over already-resolved Kali state (for example Phase-1 `Deno.permissions.query`) do **not** create a second sandbox/effect namespace of their own; they stay derived views over the existing capability model
- this subset is narrower than the full browser ambient surface and narrower than “all globals visible during type checking”
- browser-targeted analysis/build may expose real browser ambient typings such as DOM globals without implying that every DOM operation has its own stable effect key or sandbox-policy knob
- browser-targeted policy validation and effect reporting therefore speak only about this **Kali-mediated capability subset**, not arbitrary ambient browser behavior outside the schema-v1 model
- when later specs add a new built-in stable effect/policy capability, it joins this subset explicitly rather than by implication

## Canonical Numeric-Limit Semantics

To keep CLI flags, policy validation, runtime enforcement, and JSON schemas aligned, Kali uses one shared rule for numeric capability/resource limits:
- capability-local numeric fields are **constraints only**; they never imply that the surrounding capability is enabled
- positive-budget dimensions such as `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`, and their matching CLI overrides (`--max-memory`, `--max-cpu`, `--max-open-files`) must be **positive when present**; `0` is invalid rather than a hidden deny form
- zero-capable concurrency counters such as `resources.maxSpawnedProcesses`, `resources.maxThreads`, and their matching CLI overrides may use `0` as an explicit deny/tightening value because zero concurrent uses is semantically meaningful there
- in policy files, omission is the canonical “unspecified” state for positive-budget dimensions; tools must not synthesize a second numeric deny convention
- browser-targeted policy attachment still follows the stricter early browser rule from the sandbox matrix below: cross-cutting `resources.*` budgets that would imply Kali-hosted post-deployment enforcement are rejected for that profile

This section exists to stop drift between CLI parsing, policy/schema validation, runtime enforcement, and error reporting.

## Canonical Sandbox Attachment Matrix

This is the compact cross-spec meaning of attaching a sandbox policy.

| Invocation shape | Meaning of `--sandbox` / `kali.json#sandbox` |
|---|---|
| `kali run ...` / `kali test ...` | Validate the policy, then enforce it at runtime inside the Kali-hosted execution environment |
| `kali check ...` / `kali build ...` | Phase 1: validate policy schema/config only. Phase 2+: also validate inferred effects against the policy |
| `kali check --api browser ...` / `kali build --bundle --api browser ...` | Static compatibility only; must not be described as Kali-controlled post-deployment browser enforcement, and browser-targeted policy attachment rejects any cross-cutting `resources.*` budget that would imply such enforcement (`maxMemoryMB`, `maxCpuTimeMs`, `maxOpenFiles` when present, and positive `maxSpawnedProcesses` / `maxThreads`) |
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
- schema v1 has two **native-JSON reporting commands**: `kali effects` and `kali package-effects`
- those commands may emit their native payloads directly by default
- in that native-payload mode, **stdout is reserved for the success payload only**; extra status/progress text must not be interleaved into stdout, and default human diagnostics on failure should go to stderr instead
- `--output json` wraps command results in the standard command envelope from [specs/18-schemas.md](./specs/18-schemas.md)
- a command with **envelope-only JSON support** does not need a command-specific success-payload schema in order to support `--output json`; in schema v1 it may expose only the standard envelope with `payload` omitted or `null`, but it must say that explicitly rather than inventing an ad hoc payload shape
- envelope-only JSON support also means exactly that: until a command has a dedicated payload schema, it should not promise command-specific machine metadata through undocumented top-level fields, ad hoc payload objects, or by repurposing `stdout` / `stderr` as hidden structured-result channels
- early `package-audit` is the canonical envelope-only JSON example if it ships before a dedicated audit payload schema lands
- `--output json` is also the canonical way to request a machine-readable **failure** result for native-JSON reporting commands, so tools do not have to parse human stderr diagnostics
- machine-emitted arrays should use deterministic canonical ordering wherever the producer owns the order, so AI/tooling diffs do not depend on traversal or hash-map iteration order

## Cross-Spec Simplicity Rules

When a new feature is added, prefer:
- one canonical name per concept
- one command path for a workflow rather than overlapping near-duplicates
- explicit rejection over undocumented fallback
- extending existing artifact/effect/policy schemas rather than inventing parallel formats
- phase-gated honesty over partial compatibility claims

## Cross-Spec Source-of-Truth Order

To keep future spec edits simple and avoid near-duplicate rules drifting apart, Kali uses one ownership order for overlapping topics:
1. **`SPEC.md`** owns shared terminology, global constraints, command-family vocabulary, and top-level tie-break rules.
2. **[19 — Feature Maturity](./specs/19-feature-maturity.md)** owns phase/status promises and the canonical `E5006` gating boundary.
3. **[18 — Schemas](./specs/18-schemas.md)** owns machine-readable JSON shapes and stable field names.
4. **[12 — CLI](./specs/12-cli.md)** owns user-facing flag spelling, command syntax, and invocation examples.
5. **Subsystem chapters** own implementation detail for their domain so long as they do not contradict the four layers above.

Interpretation rules:
- if two chapters appear to disagree about **when** a feature exists, the maturity matrix wins
- if two chapters appear to disagree about a **JSON field name or payload shape**, the schemas chapter wins
- if two chapters appear to disagree about a **flag/command spelling**, the CLI chapter wins
- subsystem chapters should prefer linking back to these source-of-truth layers instead of restating large command/schema matrices in slightly different prose

This ordering is a simplification rule for future edits as much as a reading guide for the current spec set.

## Spec Map

- [01 — Architecture](./specs/01-architecture.md)
- [02 — Lexer & Parser](./specs/02-lexer-parser.md)
- [03 — AST](./specs/03-ast.md)
- [04 — Type System](./specs/04-type-system.md)
- [05 — Intermediate Representations](./specs/05-ir.md)
- [06 — Memory Management](./specs/06-memory.md)
- [07 — Optimization & Specialization](./specs/07-specialization.md)
- [08 — WASM Code Generation](./specs/08-wasm-codegen.md)
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
