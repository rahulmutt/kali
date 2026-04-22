# Kali Specification

`SPEC.md` is the top-level normalization layer for the Kali spec set.

It exists to:
1. normalize [`prompts/bootstrap.md`](./prompts/bootstrap.md) into concrete phase-correct product claims,
2. define cross-spec rules and shared vocabulary that should not drift,
3. point readers to the owning chapter for subsystem details.

Detailed subsystem contracts live in [`specs/`](./specs). When this file and an owning chapter both discuss the same topic:
- use `SPEC.md` for cross-cutting normalization and claim-shaping rules,
- use the owning chapter for the subsystem contract,
- use [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for actual public availability,
- use [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) for the current proof-backed boundary.

## Normative ownership

Kali uses one explicit ownership split:

- [`prompts/bootstrap.md`](./prompts/bootstrap.md) is the input brief only.
- [`SPEC.md`](./SPEC.md) owns cross-spec normalization, shared vocabulary, and conflict resolution.
- the owning chapter in [`specs/`](./specs) owns the concrete subsystem contract.
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) owns availability and phase status.
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) owns the repository's current verification claim boundary.
- [`PLAN.md`](./PLAN.md) and [`plan/`](./plan) own implementation order, stage sequencing, dependencies, and completion gates.

Reading rule:
- to answer **whether** something is supported, read `SPEC.md` → `specs/19-feature-maturity.md` → the owning chapter;
- to answer **how** a supported feature works, read the owning chapter first and use this file only for shared rules;
- to answer **what gets built when**, read [`PLAN.md`](./PLAN.md) and the relevant stage file;
- to answer **what proof coverage is actually claimed today**, read [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md).

## Overview

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, implemented in Rust, designed around:
- strong static analysis,
- sandbox-first execution,
- deterministic machine-readable tooling,
- explicit memory/ownership decisions rather than tracing/background GC,
- aggressive but auditable specialization,
- phased compatibility growth instead of overclaiming a broad MVP.

Kali aims for wide JavaScript/TypeScript compatibility over time, but hard features are intentionally phased. Documented command shapes may appear before they ship; availability is always controlled by [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

## Hard invariants

These invariants hold across all phases unless the top-level spec is intentionally changed:

- **AOT-only guest-language compilation** — Kali completes TypeScript/JavaScript → WASM before execution; there is no language-level JIT tier.
- **Pure Rust implementation contract** — no embedded C/C++ implementation dependency path.
- **No tracing/background GC** — ownership, escape analysis, and reference-counted strategies may exist only where the owning chapters permit them.
- **Sandbox-first honesty** — policy and enforcement claims must not exceed what Kali can actually mediate.
- **Deterministic machine contracts** — JSON outputs, artifacts, diagnostics, and command behavior stay explicit and tool-friendly.

## Goal precedence

When goals compete, Kali resolves them in this order:
1. semantic correctness,
2. sandbox honesty and auditability,
3. determinism and explicitness,
4. predictable compilation cost,
5. performance and compatibility breadth.

Kali should reject, gate, or deopt before it silently guesses.

## Phase model and release-claim rule

Kali uses two different concepts that must not be blurred:

- **phase contract** — the earliest user-visible promise for a feature,
- **implementation order** — the recommended engineering sequence for getting there.

Rules:
- documented early does not mean shipped early;
- internal implementation does not automatically mean publicly available;
- release notes, README summaries, tests, and examples must read availability from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md), not from implementation status or planned stage order;
- implementation sequencing lives in [`PLAN.md`](./PLAN.md), not in this file.

## MVP cut at a glance

This is the normalized Phase-1 product contract.

| Axis | Phase 1 contract |
|---|---|
| Language/frontend | latest published ECMA-262 grammar, TypeScript compatibility where implemented, and first-class `.js` compilation under the **bounded inference contract** |
| Runtime model | guest-language AOT-only, one linked WASM payload, no tracing/background GC, Rust implementation, wasmtime for Kali-hosted execution |
| Host support | the **Default standalone context (schema v1)** is the default non-browser execution context; the **Deno-oriented build context (schema v1)** is the default non-browser build context; `--api browser` is limited to the **Phase-1 browser-targeted command set**; `--api node` remains gated |
| Sandboxing | declarative policy files, runtime enforcement for Kali-hosted execution, policy-schema/config validation for the **Phase-1 static policy-validation surface**, no project-executed policy code |
| Effects | internal bookkeeping may exist in Phase 1; the stable public effect-report surface opens later |
| Packaging | one lock/install state; registry and raw-URL support within the **pure JS/TS package contract**; no implicit dependency repair outside `kali install`; lifecycle scripts are opt-in; native/binary/bootstrap-heavy packages are rejected by default |
| Embedding | Phase-1 **base library artifact** via `kali build --lib` for exact-version consumers when the export surface is statically known; the stable public embedding surface is later |
| Verification | Phase-1 **proof-ready** baseline: published boundary manifest plus proof-CI trigger policy; proof-backed claims require a non-empty published boundary |
| Tooling | Deno-inspired CLI workflow, concise diagnostics, versioned JSON outputs, deterministic artifacts/reports, minimal `init` / `init --lib` scaffolds |

Use the table as a reading aid only. Detailed behavior belongs to the owning chapters and the maturity matrix.

## Phase-1 explicit non-goals

Phase 1 does **not** claim:
- standalone `run --api browser` or `test --api browser`;
- supported `--api node` execution or build paths;
- stable public `kali effects` or `kali package-effects`;
- compile/check-time inferred-effect-vs-policy rejection on `check/build --sandbox` beyond policy-schema/config validation;
- stable public `kali package-audit`;
- stable public Rust embedding API, WIT contract, C ABI, or Component Model flow beyond the Phase-1 base library artifact;
- executable project-local sandbox policy code;
- executable `eval` / `Function()` compatibility;
- threaded runtime support.

Phase-1 docs may still define later command or artifact shapes for vocabulary stability, but that does not promote them into the shipped Phase-1 surface.

## Guardrail splits

These distinctions are the main anti-overclaim boundaries:

- **browser-targeted context** ≠ **standalone browser runtime/test contract**
- **browser ambient typing surface** ≠ **browser mediated sandbox/effect subset**
- **base library artifact** ≠ **public embedding surface**
- **internal effect bookkeeping** ≠ **public effect-report surface**
- **public effect-report surface** ≠ **context-free registry-audit surface**
- **proof-ready** ≠ **proof-backed**

When a claim feels ambiguous, check whether it crossed one of these boundaries.

## Canonical shared command terms

To reduce drift, these command-set names are defined once here:

- **Phase-1 browser-targeted command set** = browser-targeted `kali check [files...]` plus browser-targeted `kali build --bundle <file>`, including supported `--sandbox` variants and equivalent inherited-config forms when the effective `apiSurface` is `browser`
- **Phase-1 static policy-validation surface** = `kali check --sandbox ...` plus the Phase-1 build lanes that accept static policy attachment: default executable-oriented `kali build --sandbox <policy> <file>`, `kali build --lib --sandbox <policy> <file>`, and browser-targeted `kali build --bundle --sandbox <policy> <file>`
- **defined command family** = a command, flag, or artifact family whose shape is documented for naming/schema stability even if its availability remains phase-gated in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- **discovery-driven command** = a command that may derive its working input set from canonical project discovery when explicit inputs are omitted
- **single-package registry-analysis command** = a command whose primary explicit input is exactly one package selector rather than a source file or project discovery result

Guardrail:
- attaching `--sandbox` never rescues an otherwise-invalid command shape or phase-gated API/artifact combination.

## Bootstrap Acceptance Snapshot

This section is the short answer to “did the derived spec set actually preserve the bootstrap brief?”

| Bootstrap ask | Normalized answer in the spec set |
|---|---|
| AOT-only TS/JS → WASM compiler, no JIT | preserved as a hard invariant across all phases |
| Rust-only implementation, no embedded C/C++ | preserved as a hard invariant across all phases |
| No tracing/background GC; compile-time memory/ownership decisions | preserved as a hard invariant, with concrete memory-model ownership in [`specs/06-memory.md`](./specs/06-memory.md) |
| Sandboxing first, policy-controlled execution | preserved, but split honestly into static policy validation, runtime enforcement, and later effect reporting |
| Static JSON effect visibility | preserved, but normalized into later explicit reporting commands plus policy-comparison workflows rather than a `run --dry` shadow mode |
| TypeScript superset / stronger inference | preserved under the bounded-inference contract and annotation-required boundary |
| Aggressive specialization and layout-aware IR | preserved as optimization-direction guidance, with phase-gated delivery |
| Benchmark suite / Rust-competitive performance aspiration | preserved as a later optimization-evidence lane and benchmarking program, not as a Phase-1 performance guarantee |
| Deno / browser / Node API support | preserved, but split by context and maturity; Phase 1 is Deno-oriented plus the browser-targeted command set, while Node is later |
| npm ecosystem access | preserved through the pure JS/TS package contract and support ladder, not as an unqualified “all npm works” claim |
| Real-package e2e validation (for example `semver` and `@mariozechner/pi-coding-agent`) | preserved as phase-correct package-corpus evidence: representative package probes must assert the right rung and expected outcome for the current host/API maturity, rather than assuming every named package is Phase-1 executable |
| Embeddability / WIT / C ABI / Component Model | preserved, but split into a Phase-1 base library artifact and a later stable public embedding surface |
| `eval` and hardest dynamic features | preserved as later compatibility only; not allowed to violate the AOT-only invariant |
| Lean verification | preserved through the proof-ready/proof-backed split and the proof-boundary manifest discipline |
| Deno-like CLI and AI-friendly machine output | preserved through the CLI/schema/error chapters |

Verdict:
- the derived spec set adheres to the bootstrap brief's **directional goals**,
- and it does so by making the phase boundaries and non-goals explicit instead of overclaiming an everything-at-once MVP.

## Bootstrap normalization rule

`prompts/bootstrap.md` is directional input, not the post-normalization source of truth.

Normalization rules:
- broad goals in `prompts/bootstrap.md` must be mapped onto explicit phase promises;
- expensive or wide compatibility asks do not imply same-phase MVP support unless the owning chapter and maturity matrix say so;
- when goals compete, preserve the stronger safety and determinism constraint first.

Examples:
- **“Support Node, Deno, and browser APIs”** → Phase 1 centers on the Default standalone context plus the Phase-1 browser-targeted command set; broad Node compatibility is later.
- **“Support all features including eval”** → executable `eval` / `Function()` is phase-gated and must still preserve the AOT-only invariant.
- **“Statically get JSON output of all potential effects”** → Phase 1 may keep internal effect bookkeeping, but the stable public effect-report surface is later and remains distinct from runtime enforcement.
- **“Latest ECMA-262”** → latest published grammar is in scope; draft/proposal semantics remain gated.
- **“Programmable sandbox policy conditions”** → early policy files remain declarative; executable project policy code is not part of the early contract.

## Bootstrap Traceability Matrix

Use this table when a bootstrap sentence sounds broader than the normalized spec surface.

| Bootstrap phrase | Normalized reading | Primary owner |
|---|---|---|
| “support Deno API, Node.js API, and browser API” | support is phase- and command-context-specific rather than one blanket claim | [`specs/11-standard-apis.md`](./specs/11-standard-apis.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| “sandboxing policy passed in when running” | runtime enforcement on `run/test --sandbox`; static validation on `check/build --sandbox` | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md), [`specs/12-cli.md`](./specs/12-cli.md) |
| “statically run a command and get JSON output of all potential effects” | later explicit effect-report commands and policy comparison, not a hidden `run/test` dry-run lane | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md), [`specs/12-cli.md`](./specs/12-cli.md), [`specs/18-schemas.md`](./specs/18-schemas.md) |
| “support non node-gyp packages from npm” | support is determined by package shape, host/API fit, command maturity, and support rung | [`specs/14-packages.md`](./specs/14-packages.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| “must be embeddable / expose a C API / support WIT” | Phase 1 ships only the base library artifact; stable embedding surfaces are later | [`specs/13-embedding.md`](./specs/13-embedding.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| “support all features including eval” | parser acceptance/planning does not imply executable support; runtime `eval` stays later compatibility | [`specs/10-runtime.md`](./specs/10-runtime.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| “formally verify implementation details with Lean” | repository claims are limited by the published proof boundary | [`specs/17-verification.md`](./specs/17-verification.md), [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) |

## Support-claim authoring rule

Prefer support claims in this form:

> **`<thing>` is `<rung>` for `<command/artifact>` in `<availability context>` starting in `<phase/status>`; it is not broader than that.**

This is the canonical **support-claim reading order**:
1. identify the exact thing being asked for,
2. identify the support rung being claimed,
3. identify the command/artifact and effective availability context,
4. confirm the phase/status in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md),
5. confirm the owning evidence lane if the question is about shipped support rather than planned shape.

Avoid broad statements like “Kali supports X” until the command, artifact, rung, and context are named.

## Chapter map

Each detailed spec chapter owns one primary slice of the design:

| Chapter | Owns |
|---|---|
| [`01 — Architecture`](./specs/01-architecture.md) | AOT-only pipeline, crate boundaries, pure-Rust implementation contract |
| [`02 — Lexer & Parser`](./specs/02-lexer-parser.md) | ECMAScript/TypeScript grammar acceptance and parser behavior |
| [`03 — AST`](./specs/03-ast.md) | source-level representation and node ownership |
| [`04 — Type System`](./specs/04-type-system.md) | TS-superset typing, first-class JavaScript inference, effects, constraint boundaries |
| [`05 — Intermediate Representations`](./specs/05-ir.md) | lowering stages and optimization-facing IR contracts |
| [`06 — Memory Management`](./specs/06-memory.md) | ownership classes, escape analysis, no-tracing-GC memory model |
| [`07 — Optimization & Specialization`](./specs/07-specialization.md) | specialization rules and build-mode cost budgets |
| [`08 — WebAssembly Code Generation`](./specs/08-wasm-codegen.md) | artifact shapes, code generation, host adapter outputs |
| [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md) | policy model, runtime enforcement, effect/policy workflow split |
| [`10 — Runtime`](./specs/10-runtime.md) | Kali-hosted runtime behavior and dynamic-compatibility boundaries |
| [`11 — Standard APIs`](./specs/11-standard-apis.md) | Deno/Web/Node API layering and host-surface delivery |
| [`12 — CLI`](./specs/12-cli.md) | command shapes, flags, arity, output behavior |
| [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md) | embedding surface, WIT-first library contract, C ABI, component packaging |
| [`14 — Package Management`](./specs/14-packages.md) | dependency resolution, install/lock rules, package-shape support, raw URLs |
| [`15 — Error Reporting`](./specs/15-errors.md) | diagnostic meanings, human-readable conventions, canonical error boundaries |
| [`16 — Testing`](./specs/16-testing.md) | evidence lanes and conformance strategy |
| [`17 — Formal Verification`](./specs/17-verification.md) | Lean verification program and proof-boundary discipline |
| [`18 — Schemas`](./specs/18-schemas.md) | machine-readable JSON/config/policy/artifact schemas |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | canonical phase/status matrix for support claims |

## Canonical terminology

Only the most reused cross-spec terms are defined here. Lower-level detail belongs in the owning chapter.

### AOT-only guest-language compilation

Kali must complete TypeScript/JavaScript → WASM compilation before execution. Host-engine translation or WASM precompilation is an execution detail, not a second Kali compilation tier.

### Pure-Rust implementation contract

Kali's implementation and required embedded runtime/toolchain path remain Rust-only. Host tooling may invoke standard platform components, but Kali does not depend on embedded C/C++ implementation subsystems as part of its language/runtime contract.

### First-class JavaScript compilation

`.js` inputs are first-class program inputs, not a downgraded compatibility mode. They participate in the same compiler pipeline, with inference limited by the **bounded inference contract**.

### Bounded inference contract

Phase 1 allows practical, budgeted inference strong enough to make TypeScript and JavaScript workable without implying unrestricted whole-program or arbitrarily expensive solving.

### Annotation-required inference boundary

Kali may require explicit annotations at exported/public boundaries or whenever inference would otherwise exceed the bounded inference contract.

### Default standalone context (schema v1)

The default non-browser Kali-hosted execution context for `run` and `test` in Phase 1.

### Deno-oriented build context (schema v1)

The default non-browser build context for Phase-1 build artifacts.

### Browser-targeted context

A compile/check/build context where the effective `apiSurface` is `browser`. It is not the same thing as a supported standalone browser runtime contract.

### Current-repository-state vs target-contract reading

Spec and plan prose may describe long-term target contracts, while some repository files report the current checked-in state. When those differ:
- `SPEC.md`, owning chapters, and the maturity matrix define the normative target contract,
- repository-status summaries should say explicitly when they are describing the current checked-in state,
- current proof claims always defer to [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md).

### Browser ambient typing vs mediated capability split

Browser API ambient types may be available independently of Kali-mediated sandbox/effect handling. Ambient typing breadth does not imply that every browser capability is part of Kali's policy model.

### Kali-hosted execution

Execution where Kali controls the runtime path and enforces its documented runtime/sandbox contract.

### Base library artifact

The Phase-1 `kali build --lib` output intended for exact-version consumers when lowering can determine a statically known export surface. It is not yet the stable public embedding surface.

### Public embedding surface

The later stable embedding contract: stable Rust embedding API plus the stable public WIT-first `--lib` contract, with `--capi` and `--component` as explicit projections over that same export surface.

### Binding-package sidecar manifest

The deterministic stem-specific embedding bundle index emitted alongside later public `--capi` and `--component` flows. In schemas/artifact manifests its canonical artifact `kind` is `binding-package`; it is a sidecar manifest for generated binding layouts, not a second primary linked-code artifact.

### Build-only additive PGO input

The `build` command's `--profile <file>` input is an explicit opt-in optimization add-on that loads deterministic profile data. It does not create a fourth build mode, does not rename the stable `fast` / `release` / `release-advanced` vocabulary, and does not define a separate artifact family.

### Guest AOT vs host-engine translation split

Kali's no-JIT invariant applies to Kali's guest-language pipeline: TypeScript/JavaScript must be fully compiled to WASM ahead of guest execution. A host WASM engine may still validate, optimize, or cache the emitted WASM as an engine implementation detail; that host-engine work does not count as a second Kali language tier.

### Deno-oriented standalone surface

The early standalone run/test host surface built around the shared Default standalone context and the shipped Deno-oriented API subset. It is the Phase-1 non-browser Kali-hosted execution surface, not a promise that every Deno API member already exists.

### Feature-gated zero-capable execution budgets

`maxSpawnedProcesses` and `maxThreads` are the two schema-v1 execution-budget axes where `0` is a meaningful explicit deny/tightening value. Positive values on those axes remain gated on the underlying subprocess/thread capability actually existing for the selected command/profile/context; this zero-capable rule does not generalize to unrelated numeric limits such as memory, CPU time, open files, timers, or network-connection caps.

### Pure JS/TS package contract

Packages that fit Kali's early support envelope: JavaScript/TypeScript published artifacts, no required native addon path, no mandatory bootstrap-heavy binary/tool download path, and host assumptions that fit the documented execution/build context.

### Native/binary/bootstrap-heavy package contract

Packages outside the early support envelope: native addons, N-API/binary dependencies, postinstall-downloaded executables, or other bootstrap-heavy assumptions outside Kali's documented host/runtime contract.

### Linked-artifact model

Kali resolves, checks, and builds against the published package artifacts it installs and links, rather than relying on hidden command-time dependency repair.

### Published-artifact-first package reading

Package support claims are evaluated against the published artifact Kali actually installs and links, not against an idealized source tree or an alternate unpublished build path.

### Package-support decision order

Read package support in this order:
1. package shape,
2. host/API fit,
3. command maturity,
4. exact support rung being claimed.

### Package-support ladder

Use explicit rungs such as:
- installable/materializable,
- checkable,
- buildable,
- executable,
- deployable-through-host.

Do not collapse those rungs into one broad “supported package” claim.

### Public effect-report surface

The later stable public reporting workflow for effects. It remains distinct from runtime sandbox enforcement and from the separate context-free registry-audit surface.

### Browser-targeted static sandbox contract

For browser-targeted `check` and `build --bundle`, attached sandbox policy handling is a static compatibility check over Kali's documented browser-applicable mediated subset. It is not Kali-hosted runtime enforcement and does not imply that a deployed browser bundle keeps Kali policy enforcement after handoff to a real browser host.

### Canonical browser-targeted budget compatibility rule

In browser-targeted static sandbox validation, schema-v1 Kali-hosted execution budgets are not treated as post-deployment browser guarantees. Positive `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` are therefore incompatible in browser-targeted policy validation; `resources.maxSpawnedProcesses` and `resources.maxThreads` may use `0` as explicit deny values under the shared zero-capable rule, but positive values remain unavailable there.

### Canonical browser-applicable mediated subset (schema v1)

The subset of the global schema-v1 sandbox vocabulary that Kali may honestly model for browser-targeted analysis/build flows: the shared Web-baseline capability families Kali mediates statically plus the browser-targeted policy compatibility rules above. Ambient browser/DOM typings outside that subset do not automatically gain one policy key per API.

### Effect-surface split

Kali intentionally separates:
- internal sandbox-oriented effect bookkeeping,
- the later public effect-report surface,
- pass/fail policy validation,
- and context-free registry-audit/security reporting.

### Registry-analysis command split

Kali keeps registry-analysis command families explicit:
- `package-effects` = single-package registry analysis for effect reporting,
- `package-audit` = single-package registry analysis for context-free audit/security output.

### Package-effects dual classification

`package-effects` belongs to two stories at once:
- by input shape it is a registry-analysis command,
- by output contract it is part of the public effect-report surface.

### Workflow-owner split

- `run/test --sandbox` enforce at runtime,
- `check/build --sandbox` validate statically,
- `effects` / `package-effects` report only.

### Resolved source graph

The full statically selected module/dependency graph a command actually analyzes, validates, builds, or reports over after config discovery, explicit roots, import resolution, and installed dependency state are applied. Cross-spec rules that mention graph scope refer to this resolved graph, not only to the immediately named root file or package manifest.

### Shared flag buckets

Kali distinguishes two shared CLI flag buckets:
- **presentation/control flags** change how output is presented or wrapped,
- **semantic/context flags** change the effective analysis, build, runtime, or sandbox context.

### Compile intent

For build-like commands, the compile intent is the artifact role being requested from the one explicit primary source input: executable by default, browser bundle with `--bundle`, or library-oriented with `--lib` / `--capi` / `--component`.

### Analysis context

The analysis context is the subset of effective command-context axes that affect static analysis semantics for a command.

### Default source-graph analysis context (schema v1)

The default analysis context for source-graph commands such as `check` and `effects` when no explicit non-default semantic flags or config overrides are active.

### Inherited analysis context

For commands that do not accept explicit per-invocation semantic flags in schema v1, the inherited analysis context is the analysis context derived from defaults plus discovered config.

### Default inherited analysis context (schema v1)

The default inherited analysis context for registry-analysis commands when no discovered config changes the semantic axes they inherit.

### Axis-aligned inherited analysis gating

If an inherited analysis-context axis would have produced a maturity or availability gate when passed explicitly, the inherited form must hit that same gate instead of being silently dropped, downgraded, or reinterpreted as fallback behavior.

### Registry-analysis context split

Registry-analysis commands are not source-graph commands. Their package selector stays explicit and project-independent even when some semantic analysis axes are inherited from config.

### Registry-analysis independence split

Inherited analysis context may change how a registry-analysis command interprets package code, but it must not silently rewrite the explicit package selector, chosen version, or single-package command shape.

### Context-free registry analysis (schema v1)

A registry-analysis mode whose semantics do not depend on the source-graph analysis context; schema-v1 `package-audit` uses this simpler context-free model.

### Registry-analysis target contract (schema v1)

Schema-v1 registry-analysis commands take exactly one explicit canonical registry package identifier as their primary target. They do not accept raw URLs, local file paths, project discovery in place of that target, or silent expansion into multi-package batch mode.

### Registry-analysis availability boundary

For registry-analysis commands, validate target/flag/output shape first and report malformed usage as command-shape failure. Only well-formed base invocations proceed to the command's own maturity/availability gate.

### JSON-producing mode

A command invocation whose success payload is JSON by contract: either a native-JSON command in its default success mode or any command run with `--output json`.

### Native-JSON command

A command whose default successful stdout payload is already JSON without requiring `--output json`. In schema v1 this applies to `effects` and `package-effects` once those commands are available.

### Envelope-only JSON command

A command whose JSON mode exists only as the standard CLI envelope selected with `--output json`; without that flag its default output is not JSON. In schema v1 this applies to `package-audit`.

### Sandbox-attachment orthogonality

Adding `--sandbox` does not legalize an otherwise-invalid command, API surface, or artifact mode. It only adds the sandbox workflow step to an already-valid underlying command/context pair.

### Effective npm-scriptable install work

`kali install --allow-scripts` is meaningful only when the effective install action includes at least one npm-target install step that can legally run lifecycle hooks. Pure JSR-only, raw-URL-only, or otherwise non-npm work does not satisfy this condition.

### Install-time npm-package hook path

The opt-in installer workflow opened by `kali install --allow-scripts` for npm lifecycle hooks during installation only. It is part of install behavior, not evidence that ordinary package execution/build/runtime support or sandbox semantics were widened.

### Proof-boundary manifest

[`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) is the published statement of what part of Kali is currently within the proof-backed claim boundary.

### Proof-ready vs proof-backed split

- **proof-ready** = the repository publishes the proof boundary manifest and proof-CI discipline;
- **proof-backed** = the published boundary is non-empty and the repository limits claims to what is actually proved.

### Executable/analyzable source-file class

The source-file class that can serve as a runtime-bearing or build-bearing primary input: ordinary `.ts`, `.tsx`, `.js`, `.jsx`, and related executable module sources defined by the owning CLI/schema chapters.

### Minimal canonical scaffold contract

`kali init` creates the smallest valid schema-v1 project scaffold for the selected template in the current working directory. It should avoid speculative dependencies, lockfiles, or optional config sections unless the template explicitly needs them.

### Template selection vs build artifact mode split

`kali init --lib` selects a library-oriented project template only. It does not preselect later `build` artifact mode for the project or change the meaning of plain `kali build`.

### Canonical source-file classes

Kali distinguishes at least two source-file classes in schema v1:
- executable/analyzable source files,
- declaration-only files.

### Configless project mode

When no `kali.json` is discovered, commands still run against the current working directory as the effective project root with built-in defaults.

### Explicit path boundary rule

Explicitly named source-command file paths must stay inside the effective project root and may not silently cross into a nested child project that has its own `kali.json`.

### Validation-order rule

Command validation proceeds in this order:
1. command shape and arity,
2. base command availability,
3. finer effective-context/profile/feature gates for that otherwise-valid command.

### Canonical browser-surface rejection split

When browser-targeted support is intentionally narrower than generic browser wording suggests, contradictory browser command shapes are invalid usage, while real but not-yet-supported standalone browser contracts are availability-gated. In schema v1 this means browser build-shape contradictions such as non-bundle browser builds stay on the command-shape path, while `run/test --api browser` stay on the availability-gate path.

### Embedding-stability split

Kali separates the Phase-1 base library artifact from the later stable public embedding surface. Early export-oriented builds establish deterministic artifact shape for exact-version consumers without yet claiming stable Rust/WIT/C ABI/component compatibility as a public cross-version contract.

### Observation-only compatibility facades

Compatibility APIs that only reveal already-determined runtime or sandbox state and do not themselves negotiate or widen privileges. In schema v1, query-only permission-observation facades belong here.

### Recognized-but-unavailable compatibility members

API members that Kali intentionally recognizes and diagnoses as documented-but-unavailable compatibility surface rather than pretending they do not exist. They remain on the normal maturity-gating path until a later phase explicitly opens them.

### Command-context axis participation table

Only some effective-context axes participate in each command's semantics.

| Command family | Participating semantic/context axes in schema v1 |
|---|---|
| `check`, `effects`, `build`, `run`, `test` | `apiSurface`, runtime profiles, compat features, and command-local semantic selectors |
| `package-effects` | inherited analysis context only; no explicit per-command `--api` / `--compat` / `--wasm-threads` flag family |
| `package-audit` | none; context-free registry analysis |
| `fmt`, `lint`, `init`, `install` | only their command-local semantic selectors |

### Availability context

A support claim must name its relevant command, artifact mode, API surface, and phase/status. “Supported” without context is usually too broad to be normative.

## Release-claim discipline

When a change affects public claims:
- update the owning chapter,
- update [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md),
- update any affected schema/CLI/error owners,
- update [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) as well if verification claims changed,
- keep README and summary wording aligned with the maturity matrix and proof boundary.

## Reading shortcut

- **What is Kali?** → this file and the owning chapter.
- **Is it supported yet?** → [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).
- **How does it work?** → the owning chapter.
- **What gets implemented when?** → [`PLAN.md`](./PLAN.md) and [`plan/`](./plan).
- **What is actually proof-backed today?** → [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md).
