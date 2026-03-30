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
- embeddability through a Phase-1 base library artifact, with a Phase-2 public embedding surface: stable Rust embedding API plus stable public `--lib` + WIT, C ABI, and Component Model packaging.

Kali aims for broad JavaScript/TypeScript compatibility over time, but the spec deliberately phases hard features instead of implying that every aspiration is part of the MVP.

## MVP Cut at a Glance

To keep the rest of the spec readable, the normalized Phase 1 MVP can be summarized in one page:

| Axis | Phase 1 MVP contract |
|---|---|
| Language/frontend | Latest published ECMA-262 grammar, TypeScript compatibility where implemented, and first-class `.js` compilation with bounded conservative inference |
| Runtime model | AOT-only, one linked WASM payload, no tracing/background GC, Rust implementation, standardized on wasmtime for Kali-hosted execution |
| Host support | `--api deno` for Kali-hosted execution; `--api browser` only for browser-targeted `check` and `build --bundle`; `--api node` remains gated |
| Sandboxing | Declarative policy files, runtime enforcement for Kali-hosted execution, policy-schema validation for `check`/`build`, no project-executed policy code |
| Effects | Internal effect bookkeeping may exist, but stable `kali effects` / `package-effects` reporting waits for Phase 2 |
| Packaging | One lock/install state, registry support first for the **pure JS/TS package contract**, and rejection by default for the **native/binary/bootstrap-heavy package contract** |
| Embedding | Phase-1 **base library artifact** via `kali build --lib`; the Phase-2 **public embedding surface** adds the stable Rust API plus the stable public `--lib` + WIT, C ABI, and Component Model packaging |
| Tooling | Deno-like CLI, concise AI-friendly diagnostics, versioned JSON outputs, deterministic artifacts/reports |

Use this table as a reading aid only. Detailed behavior still belongs to the owning chapters and the maturity matrix.

## Recommended Phase-1 Implementation Order

To keep the bootstrap brief actionable and avoid trying to build every aspiration at once, Phase 1 should be implemented in this order:

1. **Frontend + checking foundation** — lexer, parser, AST, name resolution, TypeScript-compatible checking, first-class JavaScript handling, and the bounded conservative inference promised for Phase 1.
2. **Deterministic package/install foundation** — `kali install`, shared lock/materialization rules, package resolution, and strict non-mutating behavior for non-install commands.
3. **Kali-hosted execution foundation** — one AOT pipeline to one linked WASM payload, `run`/`test` on the Deno-oriented standalone surface, and the Phase-1 runtime/resource sandbox contract.
4. **Build/artifact foundation** — default executable builds, browser-targeted `build --bundle --api browser`, and the Phase-1 `build --lib` base library artifact.
5. **Developer workflow foundation** — `check`, `fmt`, `lint`, AI-friendly diagnostics, and stable schema-v1 JSON envelopes/artifact metadata.
6. **Phase-1 evidence hardening** — conformance tests, package corpus coverage, browser-bundle smoke tests, and determinism checks required by the maturity matrix.

Sequencing rule:
- later Phase-1 work may deepen earlier layers, but should not bypass them with feature-specific shortcuts
- in particular, Phase-2/3 breadth work such as stable effect-report commands, public embedding flows, broader Node compatibility, or dynamic compatibility paths must not land by weakening the earlier hard invariants

## Bootstrap Normalization Rule

`BOOTSTRAP.md` is the input brief. This spec set is the normative source of truth after normalization.

Normalization rules:
- treat broad product goals in `BOOTSTRAP.md` as **directional requirements**, then map them onto explicit phase promises in the spec chapters;
- when the bootstrap says Kali “should support” something large or expensive, do **not** infer same-phase MVP support unless a chapter and the maturity matrix say so;
- when the bootstrap lists competing goals, preserve the stronger safety/determinism constraint first.

Canonical examples of that normalization:
- **“Support Node, Deno, and browser APIs”** → Phase 1 is Deno-first plus browser-targeted analysis/build; broad Node compatibility is Phase 3.
- **“Support all features including eval”** → `eval`/`Function()` are part of the long-term compatibility contract, but Phase 4-gated behind the single schema-v1 compatibility switch `eval`, and that later compatibility path must still preserve Kali's no-language-level-JIT invariant.
- **“Latest ECMA-262”** → latest **published** ECMA-262 grammar is Phase 1; draft/Stage-3+ proposal support is experimental rather than implied.
- **“Programmable sandbox policy conditions”** → project policy files stay declarative in early phases; later programmable narrowing is via host-registered predicates, not executable project policy code.
- **“Use wasmtime or wasmer”** → standardize on `wasmtime` first; alternative engines are later implementation extensions.
- **“Support WIT / Component Model”** → Phase 1 keeps a base exported-library artifact; stable WIT-first public embedding and component packaging are Phase 2 targets.
- **“Must be embeddable / expose a C API / be easy to use as a Rust library”** → Phase 1 is library-first internally and already includes the base `kali build --lib` artifact, but the stable public Rust embedding API, stable WIT contract, host-side C ABI, and component/C-embedding packaging are Phase 2 targets.
- **“No GC”** → no tracing/background GC is allowed; deterministic ownership/reference-counted strategies are acceptable where the owning chapters permit them.

## Bootstrap Traceability Matrix

This table is the compact “where did each bootstrap ask land?” view.

| Bootstrap theme | Normalized contract | Primary owner(s) |
|---|---|---|
| TypeScript + first-class JavaScript compilation | TS compatibility stays broad; `.js` is a first-class input with stronger bounded inference rather than a downgraded mode | [`specs/04-type-system.md`](./specs/04-type-system.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Sandbox-first design + static effect reporting | Phase 1 ships runtime enforcement plus policy validation; stable `kali effects` and compile-time effect-vs-policy checks land in Phase 2 | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| AOT only / no JIT | Kali is language-level AOT only; runtime engine internals must not become part of the language contract | [`specs/01-architecture.md`](./specs/01-architecture.md), [`specs/10-runtime.md`](./specs/10-runtime.md) |
| No tracing GC / explicit memory decisions | No tracing/background GC; deterministic ownership, escape analysis, and layout decisions are the core memory story | [`specs/06-memory.md`](./specs/06-memory.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Aggressive specialization + layout-aware IR | Optimization is staged: explicit layout-aware IR plus specialization deepen over Phases 2-3 without weakening auditability | [`specs/05-ir.md`](./specs/05-ir.md), [`specs/07-specialization.md`](./specs/07-specialization.md) |
| Deno, Node, and browser support | Phase 1 is Deno-first with browser-targeted analysis/build; Node is phase-gated until Phase 3 | [`specs/11-standard-apis.md`](./specs/11-standard-apis.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| npm / JSR / raw-URL package access | Early package support is broad for packages inside the **pure JS/TS package contract** that fit the linked-artifact model, but narrow for the excluded **native/binary/bootstrap-heavy package contract** | [`specs/14-packages.md`](./specs/14-packages.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Embeddability, C ABI, WIT, Component Model | Phase 1 ships the base `--lib` artifact; the Phase-2 **public embedding surface** adds the stable Rust API plus the stable public `--lib` + WIT, C ABI, and component packaging | [`specs/13-embedding.md`](./specs/13-embedding.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Latest published ECMA-262 boundary | Kali tracks the latest **published** ECMA-262 edition; draft or proposal semantics stay explicitly experimental rather than implied | [`specs/02-lexer-parser.md`](./specs/02-lexer-parser.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Pure Rust implementation / no embedded C or C++ | Implementation choices must preserve the pure-Rust host/runtime/toolchain contract rather than smuggling in embedded C/C++ dependencies | [`specs/01-architecture.md`](./specs/01-architecture.md), [`specs/10-runtime.md`](./specs/10-runtime.md) |
| AI-friendly CLI and diagnostics | Human output stays concise; JSON contracts, stable codes, and AI-friendly machine payloads are explicit product requirements | [`specs/12-cli.md`](./specs/12-cli.md), [`specs/15-errors.md`](./specs/15-errors.md), [`specs/18-schemas.md`](./specs/18-schemas.md) |
| Lean-backed verification | Formal verification is phased and model-based rather than implied for the full implementation on day one | [`specs/17-verification.md`](./specs/17-verification.md) |

Use this table as a navigation aid only. The owning chapters and the maturity matrix remain normative.

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

## Bootstrap Triage Rule

To keep `BOOTSTRAP.md` actionable without turning every aspiration into an MVP promise, classify each bootstrap ask into one of three buckets before editing any chapter:

1. **hard invariant** — must remain true across all phases unless the top-level spec is intentionally changed;
2. **phase contract** — explicitly promised for a named phase by the owning chapter and the maturity matrix;
3. **phase-gated breadth target** — important long-term direction, but not yet part of the guaranteed user-visible contract.

Canonical **hard invariants** from the bootstrap brief:
- **AOT only** — no language-level JIT path;
- **pure Rust implementation contract** — no embedded C/C++ implementation dependencies;
- **no tracing/background GC** — ownership/reference-counted strategies may exist only where the owning chapters permit them;
- **sandbox-first honesty** — policy/enforcement claims must never overpromise what Kali can actually mediate;
- **deterministic machine contracts** — JSON output, artifact/report structure, and command behavior should stay explicit and tool-friendly.

Triage heuristics:
- if a feature widens the host/runtime contract, requires dynamic code loading/reflection, or introduces a second near-duplicate workflow vocabulary, treat it as a **phase-gated breadth target** unless a chapter and [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) explicitly promote it;
- optimization, inference, or compatibility work may deepen within the hard invariants, but must not silently weaken them;
- when in doubt, preserve the hard invariant and phase-gate the broader compatibility request.

This rule is what keeps bootstrap goals such as broad Node/browser support, `eval`, programmable policy logic, and Component Model packaging aligned with the rest of the spec without letting them erase the project's safety and determinism constraints.

## Cross-Spec Simplification Rules

To keep the spec set implementable and reduce drift between chapters, Kali intentionally standardizes on a few cross-cutting simplifications:
- **one guest-facing host ABI** realized through different host adapters, rather than separate guest contracts for standalone execution, browser bundles, and embedding;
- **one linked core payload per build**, with companion artifacts such as JS glue, WIT, headers, or component wrappers layered on top rather than becoming separate runtime-linked guest graphs;
- **one browser-targeted context model** reused across supported browser analysis/build commands, with later browser-context `package-effects` inheriting that context from config/defaults instead of growing a package-analysis-specific `--api` flag family;
- **one install/lock state** shared across the default Deno-oriented standalone path and supported browser-targeted analysis/build paths in schema v1;
- **one compatibility-feature name** (`eval`) for both direct `eval` and `Function()`;
- **one sandbox/effect vocabulary** for the Kali-mediated capability subset, rather than per-DOM/per-host-API policy keys.
- **one published-standard boundary**: latest **published** ECMA-262 grammar in Phase 1, current-edition non-Annex-B semantics for the features Kali marks as supported, and explicit gating for Annex B corners or draft/proposal features instead of letting “latest ECMA-262” mean “everything now”.
- **one pure-Rust implementation contract**: Kali itself and its shipped dependencies remain Rust-only from the project/toolchain point of view; ordinary platform runtime/system libraries reached through Rust toolchains or OS bindings do not count as smuggling in embedded C/C++ libraries, but bundling or requiring project-specific C/C++ implementation dependencies still violates the contract.
- **one specialization key model** based on observable layout/representation fingerprints plus the small set of semantic distinctions that still affect correctness, rather than blindly keying every specialization on the full inferred source-level type.

These are deliberate simplifications, not accidental omissions. Later phases may add capability, but should not fork the core vocabulary or workflow without a clear need.

## Spec-Maintenance Anti-Drift Checklist

When editing or extending the spec set, prefer referencing the owning chapter/term instead of re-explaining it with slightly different wording.

Use this checklist:
- command shape, flags, arity, `--output json`, and exit behavior belong to [`specs/12-cli.md`](./specs/12-cli.md)
- diagnostic-code meaning and error-boundary rules belong to [`specs/15-errors.md`](./specs/15-errors.md)
- JSON field names, payload schemas, artifact kinds/roles, and generated metadata-file shapes such as C ABI embedding metadata belong to [`specs/18-schemas.md`](./specs/18-schemas.md)
- phase availability belongs to [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- shared cross-spec tables/rules such as the **Command-context axis participation table**, the **canonical browser-targeted budget compatibility rule**, and the artifact-mode matrix should have exactly one normative copy in this file; other chapters should point here instead of restating a second near-duplicate table
- install/lock/materialization rules and command-time package selection belong to [`specs/14-packages.md`](./specs/14-packages.md)
- host/API-layering wording should reuse the **host-support staircase**
- browser ambient-typing versus sandbox/effect wording should reuse the **Browser ambient typing vs mediated capability split**
- browser command-shape versus browser-runtime availability wording should reuse the **canonical browser-surface rejection split**
- browser-targeted `--sandbox` wording should reuse the **browser-targeted static sandbox contract**
- zero-versus-positive wording for `resources.maxSpawnedProcesses` / `resources.maxThreads` and their matching CLI caps should reuse the **feature-gated zero-capable execution budgets** term instead of restating the same `0`-is-valid / positive-is-gated rule in each chapter
- compatibility-surface wording for query-only permission observation should reuse the **observation-only compatibility facade** and **recognized-but-unavailable compatibility member** terms
- library/export-oriented build wording should reuse the **compile intent**, **embedding-stability split**, **library-oriented instantiation rule**, **statically known export surface**, and **host ABI header vs program-specific exports header** terms
- single-package registry-analysis wording should reuse the **registry-analysis context split**, **registry-analysis project-independence rule**, **identity-only registry target**, and **stable-release selection rule (schema v1)**
- JSON machine-output wording should reuse the canonical **native-JSON command**, **envelope-only JSON command**, and **JSON-producing mode** terms instead of restating near-duplicate output-mode rules
- schema-v1 `package-audit` machine-output wording should point to [specs/18-schemas.md](./specs/18-schemas.md)'s **Package Audit JSON Output (schema v1)** section instead of restating a near-duplicate envelope-only rule
- project-install/discovery interactions for raw URL dependency state should reuse the **install-time declaration graph** term
- config-discovery/install interactions without a discovered `kali.json` should reuse the **configless install split** term
- config-field wording should reuse the **config leaf key vs full config path** split: use leaf names such as `apiSurface`, `buildMode`, and `runtimeProfiles` for cross-spec semantic axes, but use concrete schema paths such as `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, and `compat.features` when a chapter means actual `kali.json` storage or diagnostic `configPath` values
- registry-package CLI/manifest spelling versus structured JSON package metadata should reuse the **registry package identifier vs package coordinate** term instead of re-explaining the `jsr:` prefix split in slightly different ways
- schema-v1 registry dependency value wording should reuse the **exact-version-first registry manifest rule (schema v1)** instead of restating the exact-version requirement in slightly different prose
- package-audit semantics that intentionally ignore inherited host-analysis/runtime config should reuse **context-free registry analysis (schema v1)** instead of restating the ignored-axis list
- package-effects inherited-context maturity wording should reuse **axis-aligned inherited analysis gating** instead of re-listing the browser/node/runtime-profile/compatibility examples in each chapter
- Phase-1 internal effect machinery versus Phase-2 stable effect-report-command wording should reuse the **effect-surface split** instead of creating new near-duplicate “effects exist internally but not publicly yet” prose in each chapter
- install-lifecycle-script wording should reuse **install-time npm-package hook path** and **effective npm-scriptable install work** instead of re-explaining the `--allow-scripts` boundary in each chapter
- package-compatibility wording should reuse the **pure JS/TS package contract** and **native/binary/bootstrap-heavy package contract** terms instead of repeating slightly different native-addon / downloaded-binary exclusion lists
- source-file-kind wording should reuse **canonical source-file classes**, **executable/analyzable source-file class**, and **canonical project file set** instead of repeating long extension lists in every command chapter

Practical rule:
- if a chapter needs more than a short paragraph to restate one of those shared rules, add or reuse a canonical term here instead of creating another near-duplicate explanation.

## Canonical Terminology

### API surface
The selected host-facing ambient/runtime family:
- `deno`
- `node`
- `browser`

`browser` is a **browser-targeted context** in early phases, not a promise of a standalone browser runtime.

Rule:
- public APIs should preserve this term explicitly: use `apiSurface` as the canonical JSON/report field name and config leaf name (in schema-v1 `kali.json`, the concrete path is `compilerOptions.apiSurface`), `ApiSurface` in typed APIs, and `api_surface` / `apiSurface`-equivalent spellings in FFI surfaces rather than collapsing the concept to a generic `api` name that could be confused with a concrete host API namespace

### Effective API surface
The final `apiSurface` value after merging built-in defaults, discovered `kali.json`, and explicit CLI flags.

Rule:
- this is just the `apiSurface` slice of the broader **effective command context**
- chapters may use “effective API surface” as shorthand when only that one axis matters
- docs should not invent alternate names such as “resolved API mode” or “active host flavor” for the same concept

### Config leaf key vs full config path
Kali uses one deliberate naming split so cross-spec terminology can stay short without making the on-disk schema ambiguous.

Canonical examples:
- semantic/config **leaf keys**: `apiSurface`, `buildMode`, `runtimeProfiles`, `strict`, `maxSpecializations`
- concrete schema-v1 `kali.json` **paths**: `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, `compilerOptions.strict`, `compilerOptions.maxSpecializations`
- compatibility config is already stored at its canonical full path: `compat.features`

Rules:
- use the short leaf-key names when a chapter is talking about semantic axes, effective-context merging, report fields, or CLI/config vocabulary alignment
- use the full schema path when a chapter is talking about actual `kali.json` layout, defaults for a stored field, or diagnostic metadata such as `Diagnostic.context.configPath`
- docs should not blur these into competing vocabularies; `apiSurface` and `compilerOptions.apiSurface` are the same concept at different specificity levels, not two different settings

### Host-support staircase
Kali's host/API story is intentionally staged as one small staircase rather than three equally mature runtimes:
1. **Web baseline** — shared JS-visible baseline APIs used across supported surfaces
2. **Deno-oriented standalone surface** — the Phase-1 primary runtime/API surface for Kali-hosted execution
3. **Browser-targeted context** — Phase-1 ambient typing + bundle/build support that targets the real browser host rather than a standalone Kali browser runtime
4. **Node compatibility surface** — later package-driven compatibility work, not a second Phase-1 primary host

Rule:
- chapters should prefer this staircase when explaining how Web baseline, Deno, browser-targeted support, and later Node compatibility relate
- docs should avoid phrasing Node and browser support as though they were simply two more Phase-1 peers of the Deno standalone runtime
- browser-targeted support and Node compatibility may both expand later, but they start from different contracts and should not be described as one generic "compatibility layer"

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

### Compile intent
The host-visible meaning of one compilation request, orthogonal to API surface, build mode, and runtime profile:
- **executable intent** — the compiled module/artifact is expected to have an executable entry contract
- **library intent** — the compiled module/artifact is expected to expose a **statically known export surface** for host calls or export-oriented artifact flows

Rules:
- CLI artifact modes and embedding compile APIs should select compile intent explicitly rather than forcing hosts to infer it later from whether they try `run` versus `instantiate` / `call`
- executable-style helpers operate only on executable-intent modules and must fail explicitly on library-intent modules
- library-intent flows reuse the shared **library-oriented instantiation rule** and the **statically known export surface** requirement

### Compat feature
An explicitly gated compatibility switch for semantics that are intentionally off by default. In schema v1, the canonical stable compat feature name is:
- `eval`

That single name covers both direct `eval` and the `Function()` constructor path.

### Config compat selection vs emitted `compatFeatures`
Kali intentionally uses two closely related spellings for the same semantic set:
- config stores compatibility switches under `compat.features`
- emitted self-contained JSON reports flatten that same set to `compatFeatures`

Rule:
- this is a shape normalization only, not a second vocabulary
- docs should not invent alternatives such as `compatFlags` or `compatMode`

### Browser-targeted context
A command context whose effective `apiSurface` is `browser`.

In Phase 1, this means:
- `kali check --api browser`
- `kali build --bundle --api browser`

It does **not** mean:
- a standalone Kali-hosted browser runtime,
- DOM emulation inside `kali run`/`kali test`,
- permission to expose Deno/Node globals during browser-targeted analysis/build.

### Browser ambient typing vs mediated capability split
Kali keeps one explicit boundary between two browser-related layers that are easy to blur together:
- **browser ambient typing surface** — the globals/types visible during supported browser-targeted analysis/build (`Window`, `Document`, DOM types, `fetch`, `URL`, and similar browser-host types)
- **Kali-mediated capability subset** — the smaller stable sandbox/effect vocabulary used by policy validation and effect reporting

Rules:
- supported browser-targeted analysis/build commands may expose the broader browser ambient typing surface without implying that every such ambient API is individually modeled by Kali's sandbox/effect system
- schema-v1 sandbox/effect contracts remain scoped to the documented browser-applicable part of the **Kali-mediated capability subset**, not one policy/effect key per DOM API
- docs should reuse this term when explaining why browser-targeted `check`/`build --bundle` can understand DOM/browser programs while browser-targeted `--sandbox` still validates only the documented mediated subset

### Canonical browser-surface rejection split
Kali uses one shared rejection boundary for browser-related command shapes in early phases:
- use **`E5008` invalid command usage** when the user selected a contradictory browser build shape for a mode that otherwise exists (for example `kali build --api browser main.ts` without `--bundle`, or pairing `--api browser` with `--lib` / `--capi` / `--component`)
- use **`E5006` unavailable feature** when the user requested a browser runtime/test contract that Kali does not yet define (for example `kali run --api browser main.ts` or `kali test --api browser`)

Rule:
- chapters should reuse this split instead of restating near-duplicate prose about browser bundle/build availability versus missing browser runtime/test support

### Kali-hosted execution
Execution where Kali or an embedding host owns the runtime/import boundary, including:
- `kali run`
- `kali test`
- embedding hosts using Kali-controlled imports

### Host adapter
The implementation layer that satisfies Kali's one guest-facing host ABI/capability model for a concrete deployment mode.

Canonical early adapters:
- **native host adapter** — used for Kali-hosted execution (`run`, `test`, embedding)
- **browser host adapter** — generated JS glue used by `build --bundle --api browser`

Rule:
- Kali keeps one guest-facing host ABI and capability vocabulary across adapters
- adapters may differ in implementation technique, but they must not silently widen the documented command/profile contract
- browser-targeted analysis/build exposing browser ambient typings does not imply one adapter entry or one sandbox key per DOM API

### Pure-Rust implementation contract
The cross-spec interpretation of “implemented in Rust” / “no embedded C or C++ libraries”.

Rules:
- Kali's implementation crates and shipped dependency stack must remain Rust-only from the project/toolchain point of view; bundling or requiring project-specific C/C++ libraries violates the contract.
- ordinary platform runtime/system libraries reached through the normal Rust toolchain, system call bindings, or OS-provided interfaces do **not** by themselves violate the contract.
- exposing a C ABI for embedding does **not** weaken this rule; a Rust implementation may publish C-callable boundaries without embedding a C/C++ implementation.
- docs should reuse this term instead of re-explaining the distinction as “pure Rust except libc”, “no C/C++ in-tree”, or “C ABI is okay because only the boundary is C”.

### Pure JS/TS package contract
The shared early-phase package-compatibility boundary for registry packages Kali can treat as ordinary source packages.

A package stays inside this contract when:
- its shipped code is JavaScript/TypeScript rather than a native host module,
- it uses ordinary JS module systems that Kali models (`import`/ESM and supported CommonJS lowering),
- its normal install/runtime path does **not** require the **native/binary/bootstrap-heavy package contract**.

Rules:
- this term describes package-shape compatibility, not whether the package's chosen host APIs are already supported for the active `apiSurface`.
- staying inside this contract is necessary but not sufficient for support: packages may still be phase-gated by unavailable Node/browser/runtime features.
- docs should reuse this term instead of inventing near-duplicate phrases such as “pure JS packages”, “no native addons”, or “ordinary source-only packages” when the same boundary is meant.

### Native/binary/bootstrap-heavy package contract
The shared cross-spec name for package behaviors that fall outside Kali's early ordinary-source package model.

A package is in this contract when its normal install/runtime path depends on one or more of:
- native addons or `node-gyp`,
- N-API bindings or other compiled native code,
- prebuilt native modules,
- postinstall-downloaded executables,
- other platform-specific binary/bootstrap artifacts or selection steps.

Rules:
- this contract is rejected by default in early phases unless an owning chapter and the maturity matrix explicitly say otherwise.
- opting into npm lifecycle hooks through the **install-time npm-package hook path** does **not** promote these packages into the supported set.
- docs should reuse this term instead of repeating slightly different lists such as “native/N-API/prebuilt modules”, “binary/bootstrap-heavy packages”, or “native addon / downloaded executable packages” when the same exclusion boundary is meant.

### Kali-mediated capability subset
The stable schema-v1 capability vocabulary shared across effects and sandbox policy:
- filesystem
- network
- timer
- random
- console
- process
- eval

This is the stable capability vocabulary, **not** a claim that every command/profile/API surface enables every capability.

### Built-in effect kind vs policy/schema key
Kali intentionally uses two related naming layers for effects:
- semantic built-in effect kinds such as `FileSystem.Read`, `Network.Fetch`, `Process.EnvRead`, `Timer.Schedule`, `Random.GetBytes`, `Console.Write`, and `Eval`
- schema/policy keys such as `effects.fileSystem.read`, `effects.network.fetch`, `effects.process.envRead`, `effects.timer.schedule`, `effects.random`, `effects.console`, and `effects.eval`

Rule:
- built-in effect kinds are the semantic names used by the type/effect system and effect reports
- `effects.*` keys are the policy/schema paths used for configuration and authorization
- the mapping between those two layers is centralized in [`specs/18-schemas.md`](./specs/18-schemas.md) and should not be re-invented per chapter

### Effect-surface split
Kali keeps one explicit split between internal effect machinery and the later stable user-facing reporting surface:
- **internal effect bookkeeping** — conservative compiler/runtime effect facts that may exist in Phase 1 to support sandbox-first implementation, diagnostics, lowering decisions, or later-proofed integration work
- **public effect-report surface** — the stable user-facing effect-reporting and policy-comparison workflow (`kali effects`, `kali package-effects`, and compile/check-time inferred-effect-vs-policy validation) that becomes part of the supported contract in Phase 2+

Rules:
- Phase 1 may rely on **internal effect bookkeeping** without implying that effect JSON, command availability, or machine-readable report fields are already stable
- docs should use this split when they need to explain why sandbox-first implementation can start before the stable report commands land
- chapters should avoid phrasing that makes the absence of the **public effect-report surface** sound like the total absence of effect infrastructure

### Canonical browser-applicable mediated subset (schema v1)
When a chapter says browser-targeted policy/effect reasoning uses the browser-applicable part of the **Kali-mediated capability subset**, it means:
- `effects.network.fetch`, plus the capability-local cap `effects.network.maxConnections`
- `effects.timer.schedule`, `effects.timer.maxTimeoutMs`, `effects.timer.maxActiveTimers`
- `effects.random`
- `effects.console`
- later `effects.eval` only when the separate `eval` compatibility path itself exists and is enabled

It does **not** include early schema-v1 Deno/Node-oriented capability keys such as:
- `effects.fileSystem.*`
- `effects.process.*`
- `effects.network.connect`
- `effects.network.listen`

This browser-applicable subset is a **static compatibility/build-time vocabulary** for browser-targeted contexts in early phases. It is not a promise that deployed browser bundles inherit Kali-hosted runtime enforcement, and it does not create one policy/effect key per DOM or browser API.

### Observation-only compatibility facade
A host/API surface that lets programs inspect already-resolved runtime or policy state without negotiating new permissions or widening authority.

Canonical schema-v1 example:
- the read-only `Deno.permissions` facade, in its query-only compatibility form

Rules:
- these facades report state that Kali already resolved elsewhere; they are not interactive permission-prompt channels
- they are effect-free in schema v1 unless an owning chapter explicitly adds a new effect family later
- they must not imply a second sandbox-policy namespace just for observation APIs

### Deno-compatible permission descriptor subset (schema v1)
The only stable `Deno.permissions.query({ name })` descriptor names that Kali models in schema v1:
- `read`
- `write`
- `net`
- `env`
- later `run` once subprocess support exists

Rules:
- this subset exists so Kali can expose a useful Deno-compatible observation facade without inventing Kali-only permission names for unrelated capabilities such as timers, randomness, console, or `eval`
- descriptor names observe the **currently modeled capability slice**, not some future superset; in particular, `net` reflects only the network capabilities that actually exist for the active phase/API surface
- in Phase 1's standalone surface, that means `net` effectively reports the status of the modeled `fetch` path only, not future socket/listener powers
- unsupported descriptor names (for example `ffi`, `sys`, or any other non-modeled name in the current phase) follow the canonical availability failure path (`E5006`) instead of returning a misleading synthetic status
- in Phase 1, this effectively means the `read` / `write` / `net` / `env` subset only

### Stable permission status subset (schema v1)
The only stable status values for Kali's query-only `Deno.permissions` compatibility facade in schema v1:
- `granted`
- `denied`

Rules:
- Kali must not report a synthetic `prompt` state in schema v1, because the compatibility surface is observation-only and does not provide interactive escalation
- chapters should reuse this term instead of restating the same two-status rule in slightly different prose

### Recognized-but-unavailable compatibility member
An API member that Kali intentionally recognizes as part of a broader compatibility surface, but that is unavailable in the current phase/availability context and therefore fails through the canonical `E5006` path instead of behaving like an ordinary missing/unknown member.

Canonical schema-v1 examples:
- `Deno.permissions.request(...)`
- `Deno.permissions.revoke(...)`

Rules:
- use this term when the compatibility surface should remain visible/documented, but the specific member is still phase-gated
- these members must not degrade into silent no-ops, fake prompts, hidden policy mutation, or ordinary missing-member drift between checker and runtime
- ordinary absent globals/properties that are simply not part of the selected ambient surface are still handled by the usual name/type diagnostics rather than by this term

### Kali-hosted execution budgets
The schema-v1 cross-cutting `resources.*` limits used for Kali-controlled execution environments, such as:
- memory
- CPU time
- open files
- spawned processes
- threads

These budgets are part of the Kali-hosted runtime/embedding contract. They are not, by themselves, a promise that the same enforcement exists for deployed browser bundles.

### Effective execution envelope
The final runtime capability/resource ceiling for one Kali-hosted execution after all applicable limits are merged.

It is derived from:
1. intrinsic command/profile/phase/API-surface gating,
2. any attached declarative sandbox policy,
3. per-invocation tightening flags such as `--max-memory`, `--max-cpu`, `--max-open-files`, and later supported tightening caps.

Rules:
- CLI/runtime overrides may only tighten this envelope; they must not widen a stricter attached policy.
- when no sandbox policy is attached, direct invocation caps still contribute to the envelope without implying a synthesized allow-all policy file.
- this term applies to Kali-hosted execution (`run`, `test`, embedding), not to deployed browser bundles.

### Browser-targeted static sandbox contract
The canonical early-phase meaning of `--sandbox` in a browser-targeted context.

It consists of:
- static compatibility checking only,
- validation against the documented browser-applicable portion of the **Kali-mediated capability subset**,
- no promise of Kali-controlled post-deployment runtime enforcement inside a real browser host,
- no carry-over of cross-cutting **Kali-hosted execution budgets** into deployed browser bundles.

Rule:
- chapters should reference this term instead of restating near-duplicate prose about “build-time-only browser sandboxing”, “static browser policy validation”, or “no automatic browser runtime enforcement”.

### Feature-gated zero-capable execution budgets
The schema-v1 rule for execution-budget fields whose domain naturally allows an explicit zero-concurrency deny/tightening value, while any positive value still assumes the underlying capability/profile actually exists.

Canonical early examples:
- policy fields `resources.maxSpawnedProcesses` and `resources.maxThreads`
- matching CLI tightening caps such as `--max-spawned-processes` and `--max-threads`

Rules:
- omission means “no extra tightening from this source”, not an implicit zero
- `0` is a valid explicit deny/tightening value even before subprocess or threaded-profile support exists
- positive values remain availability-gated and must fail with `E5006` until the selected command/profile/API surface actually supports the corresponding capability/profile
- this rule is intentionally narrower than generic numeric-cap validation: it does **not** apply to positive-only capability-local caps such as `effects.timer.maxActiveTimers` or `effects.network.maxConnections`
- browser-targeted policy validation still follows the **canonical browser-targeted budget compatibility rule**, so in browser-targeted contexts these fields may be omitted or set to `0`, but positive values remain invalid

Rule:
- use this term instead of re-explaining the same `0`-is-valid / positive-is-gated split for these fields and flags in each chapter.

### Canonical browser-targeted budget compatibility rule
Because schema-v1 `resources.*` fields are **Kali-hosted execution budgets**, browser-targeted contexts treat them as a narrow validation boundary rather than as deployed-browser guarantees.

In schema v1 this means:
- `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` are invalid whenever present in a browser-targeted policy/context,
- `resources.maxSpawnedProcesses` and `resources.maxThreads` may be omitted or set to `0`, but positive values are invalid,
- capability-local browser-applicable caps such as `effects.network.maxConnections`, `effects.timer.maxTimeoutMs`, and `effects.timer.maxActiveTimers` remain the right place for static browser-targeted limits inside the documented mediated subset.

Rule:
- use this term when a chapter means “browser-targeted validation may talk about browser-applicable capability caps, but not about Kali-hosted `resources.*` runtime budgets as though they carried over into deployed browser bundles”.

### Analysis context
The semantic context that materially affects static analysis results:
- `apiSurface`
- `runtimeProfiles`
- `compatFeatures`

### Command-context axis participation table
To keep effective-context validation consistent across commands, schema v1 uses one shared participation table for the main semantic axes:

| Command family | `apiSurface` | `buildMode` | `runtimeProfiles` | `compat.features` | top-level `sandbox` |
|---|---|---|---|---|---|
| `run`, `test` | participates | participates | participates | participates | participates |
| `build` | participates | participates | participates | participates | participates |
| `check` | participates | ignored | participates | participates | participates |
| `effects` | participates | ignored | participates | participates | ignored |
| `package-effects` | inherited/participates | ignored | inherited/participates | inherited/participates | ignored |
| `package-audit` | ignored | ignored | ignored | ignored | ignored |
| `fmt`, `lint` | ignored | ignored | ignored | ignored | ignored |
| `install` | ignored | ignored | ignored | ignored | ignored |
| `init` | ignored | ignored | ignored | ignored | ignored |

Rules:
- “participates” means the effective value is part of validation and semantics for that command.
- “ignored” means the command does not validate or semantically use that axis in schema v1.
- “inherited/participates” means the command has no package-analysis-specific CLI flag for that axis in schema v1, but the effective inherited value from defaults/discovered config still materially affects semantics and gating.
- this table is about command semantics only; project root/config discovery, explicit path rules, and output-format flags are separate concerns.

### Layout/representation fingerprint
A canonical specialization key fragment describing the parts of a value that materially affect generated code shape.

It is based on things like:
- concrete scalar representation (`f64`, `i32` fast path, tagged)
- object/aggregate layout class and field-offset shape
- ownership/indirection facts only when they change calling convention, lifetime handling, or runtime operations
- dynamic/boxed fallbacks when layout is not statically stable

It is intentionally **not** the full source-level type identity. Distinct source types may share one fingerprint when they lower to the same observable code shape.

Rule:
- layout-driven specialization should key primarily on these fingerprints plus any remaining semantic distinctions that still affect correctness
- chapters should not require a separate codegen instantiation merely because two source-level types have different names while lowering to the same layout/behavioral contract
- the owning details live in [`specs/05-ir.md`](./specs/05-ir.md) and [`specs/07-specialization.md`](./specs/07-specialization.md)

Build mode affects compile effort and optimization behavior, but for early effect/package-analysis contracts the main semantic analysis context is the trio above unless an owning chapter says otherwise.

### Package-resolution context
The normalized context used when selecting package entry files/conditions:
- `apiSurface`
- module edge kind (`import` vs `require`)

Rule:
- supported browser-targeted commands share one browser package-resolution rule rather than inventing per-command ladders
- in schema v1, that browser rule means the browser `exports` condition order plus any applicable `package.json#browser` rewrites, as owned by [`specs/14-packages.md`](./specs/14-packages.md)
- later browser-targeted analysis commands should reuse that same package-resolution context once their own maturity rows allow them

### Effective command context
The fully merged invocation context that a command validates and executes against:
1. built-in defaults,
2. discovered `kali.json`,
3. explicit CLI flags.

Rules:
- validation runs against this merged result rather than against only the literal CLI spelling,
- config-derived values trigger the same gating and contradiction checks as explicit flags,
- commands must not silently fall back from an unsupported effective value just because the user omitted the matching flag.

### Availability context
The normalized context used for maturity and availability checks **after** command-shape validation succeeds.

It consists of:
- the selected command,
- any command-shape/artifact-mode choice that survived contradiction checks,
- `apiSurface`,
- `runtimeProfiles`,
- `compatFeatures`,
- the current implementation phase/maturity table.

Rules:
- use this term when a chapter means “the combination that determines whether Kali supports this request yet” rather than only the literal CLI spelling,
- command-shape contradictions still fail first and therefore stay outside this term's responsibility,
- docs should prefer this shared term over repetitive phrases such as “phase/profile/API-surface/compatibility gating” when the same idea is meant.

### Command-shape taxonomy vs availability
The command-shape terms in this section classify how a command behaves **when that command exists** in schema v1.

Rules:
- these terms describe arity, discovery, context inheritance, and output shape, not whether the command is already Phase 1 available,
- phase availability still comes from the owning chapter plus [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md),
- docs should avoid rephrasing a command-shape term as an availability promise.

### JSON-producing mode
A command invocation whose primary success output is JSON.

In schema v1 this happens in exactly two ways:
- the invocation is a **native-JSON command** in its default success mode, or
- `--output json` selects the standard command envelope.

Rules:
- `--pretty` is meaningful only in **JSON-producing mode**
- output-format flags do not create a second availability path or separate semantic context
- docs should reuse this term instead of spelling out slightly different “already JSON vs wrapped JSON” rules in each chapter

### Native-JSON command
A command whose default successful output is its command-specific JSON payload rather than the standard command envelope.

Schema-v1 examples once those commands are available:
- `kali effects`
- `kali package-effects`

Rules:
- default success output is the native payload on stdout with no interleaved status/progress text
- `--output json` wraps that same payload in the standard command envelope instead of changing the payload schema
- failures without `--output json` follow the ordinary human-diagnostic path; machine-readable failure output still requires the envelope request path

### Envelope-only JSON command
A command that may support `--output json` through the standard command envelope even though schema v1 defines no dedicated success-payload schema for it yet.

Canonical schema-v1 example once that later command exists:
- `kali package-audit`

Rules:
- the stable machine-readable contract is the standard command envelope itself
- `payload` should be omitted or `null` rather than populated with ad hoc command-specific objects
- `stdout` / `stderr` remain captured text-stream fields only, not hidden structured result channels
- docs should reuse this term instead of restating a near-duplicate “envelope but no payload schema” rule per command

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

### Set-oriented explicit-file command
A command whose explicit file arguments, when present, are interpreted as a file set rather than as one primary source input:
- `check`
- `fmt`
- `lint`
- `test`

This term is orthogonal to discovery mode:
- `check` is still the canonical **hybrid analysis command**
- `fmt`, `lint`, and `test` are still **project-oriented commands** when no explicit files are supplied

### Current-directory-scoped scaffold command
A command whose target root is always the current working directory rather than the nearest discovered ancestor project:
- `init`

In schema v1, `init` is the canonical exception to ordinary ancestor-based config discovery. It may create a nested child project inside an existing ancestor project as long as the current working directory itself does not already contain `kali.json`.

### Registry-analysis command
A command that analyzes exactly one explicit registry package identity rather than a project graph in early phases:
- `package-effects`
- `package-audit`

These commands do not invent a no-argument whole-project analysis mode in schema v1.

### Registry-analysis context split
To keep single-package tooling predictable and avoid a second near-duplicate flag family:
- `package-effects`, once that command exists, is **analysis-context-aware**: it inherits `apiSurface`, `runtimeProfiles`, and the effective compatibility-feature selection from config/defaults, then records that context in JSON using the emitted field name `compatFeatures` instead of taking package-analysis-specific `--api` / runtime-profile / `--compat` flags.
- because schema v1 intentionally omits package-analysis-specific context flags, non-default `package-effects` contexts come only from defaults or discovered config; in configless mode the command therefore uses the schema-v1 defaults unless/until a later spec adds explicit package-analysis context flags.
- `package-effects` follows the maturity of the inherited analysis axis instead of inventing its own separate gate table: inherited browser context lines up with browser-targeted effect analysis, inherited Node context lines up with the Node analysis gate, inherited `wasm-threads` lines up with the threaded-profile gate, and inherited compat features such as `eval` line up with their own compatibility-phase gates.
- `package-audit`, once that command exists, follows **context-free registry analysis (schema v1)**.
- both commands still follow the shared **registry-analysis project-independence rule** as well as the identity-only registry-target rule.

### Effective inherited analysis context
The semantic analysis context that `package-effects` uses in schema v1.

It consists of:
- built-in defaults,
- then discovered `kali.json` values for `apiSurface`, `runtimeProfiles`, and `compat.features`,
- with no package-analysis-specific CLI `--api` / runtime-profile / `--compat` override layer in schema v1.

Rules:
- use this term when a chapter means the inherited `package-effects` analysis knobs specifically, rather than the broader **effective command context** used by normal source commands,
- in configless mode, this context is therefore just the schema-v1 defaults,
- top-level `sandbox` and `buildMode` are outside this term in early phases,
- if a later spec adds package-analysis-specific CLI context flags, that later spec can extend this term explicitly instead of forcing every chapter to restate the whole inheritance rule.

### Axis-aligned inherited analysis gating
The schema-v1 rule for how `package-effects` availability interacts with its **effective inherited analysis context**.

In schema v1 this means:
- `package-effects` first follows its own base maturity row,
- once the command exists, each inherited analysis axis reuses the same maturity gate as the corresponding ordinary analysis/effect command path,
- Kali must not invent a package-analysis-specific shadow gate table or silently fall back to a smaller context.

Canonical consequences:
- inherited `apiSurface = browser` reuses the browser-targeted analysis gate,
- inherited `apiSurface = node` reuses the Node analysis gate,
- inherited `runtimeProfiles = ["wasm-threads"]` reuses the threaded-profile gate,
- inherited `compat.features = ["eval"]` reuses the compatibility-feature gate.

Rule:
- use this term instead of re-listing those axis-by-axis examples when a chapter means this exact package-effects maturity behavior.

### Context-free registry analysis (schema v1)
The early schema-v1 rule for registry-analysis commands whose semantics intentionally do not depend on inherited host-analysis/runtime configuration.

In schema v1 this means:
- inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox` do not change the command's semantics,
- the command still follows its own maturity row and command-shape rules,
- output-format selectors such as `--output json` still change only formatting/envelope behavior, not semantic analysis context.

Canonical early example:
- `package-audit`

Rule:
- use this term instead of restating the full ignored-axis list each time a chapter means this exact schema-v1 behavior
- this term is about semantic context participation only; it does not by itself imply anything about package version selection, cache identity, or project mutability

### Registry-analysis project-independence rule
Single-package registry-analysis commands intentionally analyze a registry package as a standalone target, not as "whatever version this project currently has installed."

Rules:
- version selection follows the shared **stable-release selection rule (schema v1)** unless an owning chapter later adds an explicit version-aware or lock-aware mode,
- the current project's `kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/` must not change which package version is analyzed,
- these commands must not mutate project-managed dependency state as a side effect,
- `package-effects` may still inherit its **effective inherited analysis context**, but that inherited context affects analysis semantics only and must not change project-independence for package identity/version selection,
- any fetched metadata/tarballs belong to the separate **registry-analysis cache**, not to project installation state.

### Registry-analysis cache
A non-project-managed cache that registry-analysis commands may use for fetched package metadata/tarballs.

Rules:
- it is outside project-managed dependency state (`kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/`),
- it may be discarded between invocations and must not be treated as an installed project dependency snapshot,
- cache identity is keyed by at least the canonical registry identifier plus the resolved concrete version,
- for analysis-context-aware registry analysis (`package-effects`), the **effective inherited analysis context** is also part of the cache identity so browser/deno/profile/compat analyses cannot collide accidentally.

### Configless install split
The canonical schema-v1 install behavior when config discovery finds no `kali.json` and the command therefore runs in **configless project mode**.

It has exactly three branches:
- **plain `kali install`** → succeed as a no-op when there are no dependency inputs; do not create a placeholder manifest just because the command ran
- **explicit registry-package add** (`kali install <pkg>` / `kali install --dev <pkg>`) → first create the minimal canonical manifest `{ "schemaVersion": 1 }`, then record the dependency there and continue with normal install work
- **explicit raw-URL install** (`kali install https://...`) → may create lock/cache state for that exact URL, but must not create a placeholder manifest by itself

Rule:
- chapters should reference this term instead of re-explaining the three-way configless install behavior in slightly different prose.

### Library-oriented artifact modes
Non-browser, export-oriented build modes:
- `--lib`
- `--capi`
- `--component`

### Embedding-stability split
Kali uses one shared stability split for library-oriented outputs:
- **base library artifact** — the Phase-1 `kali build --lib` output shape: export-oriented and useful immediately, but still the pre-stable Phase-1 half of the public embedding surface
- **public embedding surface** — the Phase-2 stabilized public embedding story built on that same exported-library contract: the stable Rust embedding API plus the stable public library/WIT contract, stable C ABI, and Component Model packaging path
- **public embedding artifact flows** — the artifact-producing part of that Phase-2 public embedding surface: stable public `--lib` + WIT, `--capi`, and `--component`

Rule:
- docs should reference this split instead of rephrasing it as “usable but not yet stable”, “public embedding contract”, “stable public library contract”, “library-first internally”, or “WIT/C ABI/component packaging lands later” in slightly different ways
- Phase 1 shipping the **base library artifact** does **not** by itself imply the Phase-2 **public embedding surface**: no stable public Rust API, stable public library/WIT contract, stable C ABI, or component packaging yet
- once Phase 2 promotes that path, plain public `--lib` is the canonical stable public library/WIT contract and emits WIT by default; `--capi` and `--component` are projections/wrappers over that same proved export surface rather than alternate export semantics

### Host ABI header vs program-specific exports header
Kali intentionally distinguishes the stable host-side C ABI header from build-emitted program-specific export declarations.

Canonical terms:
- **host ABI header** — the stable `kali.h` header shipped by `kali_capi` and versioned with the host C ABI
- **program-specific exports header** — the generated `<entry>.exports.h` header emitted by `kali build --capi` for one compiled library's proved export surface

Rules:
- docs should not use `kali.h` as a loose synonym for both headers
- `kali build --capi` emits the **program-specific exports header**, not a second copy of the **host ABI header**
- ABI/version-compatibility wording should keep the host-side `kali_capi` contract separate from the generated exported-function declarations for one library build

### Library-oriented instantiation rule
For library-oriented artifact modes:
- Kali omits any **synthetic executable entry invocation**,
- normal ECMAScript module-instantiation semantics still apply,
- therefore top-level module initialization still runs when the host instantiates the artifact,
- and the host-callable surface is the build's proved export set rather than a synthesized executable entry.

Rule:
- `--lib`, `--capi`, and `--component` all share this same instantiation rule unless an owning chapter explicitly says otherwise.
- Docs should prefer referencing this shared term instead of restating slightly different versions of the same behavior.

### Statically known export surface
The export set for a library-oriented build that Kali can prove after frontend lowering without relying on runtime reflection or host-side discovery.

Rules:
- ESM entry modules satisfy this directly from their explicit exports.
- CommonJS entry modules participate only when static CJS lowering can prove one fixed export set.
- If Kali cannot prove one stable export surface, library-oriented build modes fail rather than synthesizing reflective exports.

This term exists so `--lib`, `--capi`, `--component`, embedding docs, and the maturity matrix can all refer to the same export-surface requirement without restating slightly different versions.

### Logical roots
The normalized “what this report/build/test run is about” identifiers carried in schemas as `entryPoints`. Examples:
- `src/main.ts`
- a discovered test label
- `lodash`

This is a naming bridge only: schema field `entryPoints` is the canonical JSON field name.

## Phase-1 Non-Goals Snapshot

To keep the normalized bootstrap scope easy to scan, Phase 1 does **not** imply:
- general `--api node` command support across `check` / `effects` / `build` / `run` / `test`,
- standalone browser runtime or browser-hosted `run` / `test`,
- `eval` / `Function()` support,
- interactive permission-prompt / privilege-escalation flows such as `Deno.permissions.request()` / `revoke()`,
- threaded runtime profiles / `SharedArrayBuffer` / `Atomics`,
- the Phase-2 **public embedding surface**: stable public Rust embedding, stable public `--lib` + WIT, `--capi`, or `--component`.

These are all tracked elsewhere in the owning chapters and the maturity matrix; this snapshot exists only to make the early boundary obvious in one place.

Two additional bootstrap-driven scope clarifications belong here because they are easy to overread from the broad product brief:
- Phase 1 does **not** imply stable user-facing `kali effects`, `kali package-effects`, or `kali package-audit` workflows just because Kali is sandbox-first and internally tracks effects.
- Phase 1 does **not** imply automatic dependency installation/repair during `check` / `effects` / `build` / `run` / `test`; `kali install` remains the one project dependency mutator.

## Host/API Summary

Using the canonical **host-support staircase**:
- **standalone execution** is Deno-first,
- **browser support** is analysis/build-first,
- **Node compatibility** is a later ecosystem phase,
- **wasmtime** is the standardized early runtime engine,
- **AOT only**; no language-level JIT,
- **pure Rust only**; no embedded C/C++ libraries,
- **no tracing/background GC**,
- one guest-facing host ABI is realized through different **host adapters** rather than through unrelated per-deployment guest contracts.

Shared API-loading rule:
- Web baseline APIs are the shared baseline across supported surfaces,
- `--api deno|node|browser` selects which additional ambient APIs/modules exist beyond that baseline,
- unsupported globals/modules are absent rather than shimmed by default.

## Browser Ambient Typing vs Mediated Capability Split

This is the most important cross-spec clarification for browser support.

In browser-targeted contexts:
- Kali should expose the real browser ambient typing layer needed for browser programs,
- but stable schema-v1 effects and sandbox policy reason only about the **Kali-mediated capability subset**,
- and runtime-capable browser artifact paths use the **browser host adapter** rather than a standalone Kali-hosted browser runtime,
- therefore browser-targeted analysis/build may know about `window`, `document`, DOM types, and browser globals without implying that Kali individually mediates or sandbox-governs every browser API at runtime.

Consequences:
- `check --api browser` and `build --bundle --api browser` type-check against browser ambient types,
- later browser-targeted analysis commands such as `effects --api browser` and inherited browser-context `package-effects` reuse that same ambient-typing/package-resolution split instead of defining a second browser-analysis model,
- browser-targeted `--sandbox` is a static compatibility/build-time validation contract,
- deployed browser bundles do not automatically inherit Kali-hosted runtime enforcement.

## Canonical Browser-Surface Rejection Split

Use this rule everywhere:
- if the user asks for a **supported browser concept with the wrong command shape**, reject with `E5008`;
- if the user asks for a **browser execution/test/runtime contract that does not exist yet**, reject with `E5006`.

Examples:
- `kali build --api browser main.ts` → `E5008` (wrong build shape; browser builds are bundle-only early)
- `kali build --lib --api browser lib.ts` → `E5008`
- `kali build --capi --api browser lib.ts` → `E5008`
- `kali build --component --api browser lib.ts` → `E5008`
- `kali build --bundle --api node main.ts` → `E5008`
- `kali run --api browser main.ts` → `E5006`
- `kali test --api browser` → `E5006`

## Canonical Browser-Targeted Policy Boundary

For browser-targeted contexts, `--sandbox` follows the **browser-targeted static sandbox contract**:
- it validates static compatibility against the documented **Kali-mediated capability subset**,
- it does not promise Kali-controlled post-deployment sandbox enforcement inside an arbitrary real browser host,
- cross-cutting `resources.*` budgets are interpreted as **Kali-hosted execution budgets** and therefore sit outside the early browser deployment guarantee.

### Canonical Browser-Targeted Budget Compatibility Rule

The normative browser-targeted budget rule is the earlier canonical term of the same name in this file.

Keep only these consequences in mind when reading other chapters:
- browser-targeted `--sandbox` remains a static compatibility/build-time contract over the documented browser-applicable mediated subset
- cross-cutting `resources.*` fields are still **Kali-hosted execution budgets**, so they do not become post-deployment browser guarantees
- if the browser-targeted budget rule changes in a future schema revision, update that one canonical definition and let the rest of the spec inherit it by reference

## Artifact-Mode Matrix

Early documented build artifact modes form one small canonical matrix:

| Build invocation shape | Meaning |
|---|---|
| `kali build foo.ts` | default executable-oriented artifact flow |
| `kali build --bundle --api browser foo.ts` | browser-targeted bundle output |
| `kali build --lib lib.ts` | Phase-1 **base library artifact**; in Phase 2 the same selector becomes part of the stable public library/WIT contract and adds a default WIT sidecar |
| `kali build --capi lib.ts` | Phase-2 **public embedding artifact flow** for C embedding |
| `kali build --component lib.ts` | Phase-2 **public embedding artifact flow** for Component Model packaging |

Rules:
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive unless a later chapter explicitly says otherwise,
- omitting all four selects the default **executable compile intent**,
- `--bundle` is browser-only, requires effective `apiSurface = browser`, and keeps that same executable compile intent while swapping in the browser host adapter/output shape,
- `--lib`, `--capi`, and `--component` are the explicit **library compile-intent** selectors in early phases,
- library-oriented artifact modes are non-browser in early phases,
- Phase 1 plain `--lib` is the **base library artifact**, and in Phase 2 that same selector becomes part of the stable **public embedding surface** rather than introducing a second plain-library mode,
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
| embedding/C ABI/WIT | [`specs/13-embedding.md`](./specs/13-embedding.md) |
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

To keep CLI, schemas, and command docs aligned, schema v1 uses the shared **JSON-producing mode**, **native-JSON command**, and **envelope-only JSON command** terms defined earlier in this file.

Schema-v1 command assignment:
- **native-JSON commands** once available: `effects`, `package-effects`
- canonical **envelope-only JSON command** once available: `package-audit`

Rules:
- `--pretty` is meaningful only in **JSON-producing mode**
- `--pretty` does **not** by itself switch a command into JSON-producing mode; for an **envelope-only JSON command**, `--output json` is still required
- **native-JSON commands** reserve stdout for the success payload in their default success mode, and `--output json` wraps that same payload in the standard command envelope rather than inventing a second payload shape
- these are output-format classifications only; they must not be treated as separate command surfaces, second context models, or alternate availability paths

## Command/Context Axis Participation Table

The normative schema-v1 participation table is the earlier **Command-context axis participation table** in the canonical-terminology section of this file.

This section exists only to make the reuse rule explicit:
- CLI, schemas, package-analysis docs, diagnostics, and examples should all reuse that one table instead of maintaining a second copy here or in an owning chapter
- when prose needs the table, prefer saying that an axis **participates**, is **ignored**, or is **inherited/participates** using the canonical meanings already defined earlier in this file
- if a future schema revision changes command-axis participation, update the canonical table once and have the rest of the spec set inherit that change by reference rather than by parallel edits

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

## Canonical Validation-Order Rule

Report the outermost failing gate first:
1. command shape / arity / contradictory flag combination,
2. base command availability,
3. narrower inherited-context or profile gating,
4. source-code diagnostics within the selected valid context.

In other words:
- `E5008` owns contradictory command shape,
- `E5006` owns unavailable-but-real requests inside the chosen **availability context**.

Consequences:
- contradictory browser build shapes fail before any narrower feature gate,
- a command that is itself unavailable reports that fact before reporting a narrower inherited profile problem,
- config-derived invalid effective values trigger the same checks as explicit CLI values.

## Project Discovery

### Canonical source-file classes

Kali uses one cross-spec split for source-file kinds:
- **executable/analyzable source-file class**: `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`
- declaration-only side inputs: `.d.ts`, `.d.mts`, `.d.cts`

Command-facing rule:
- runtime-bearing entrypoints and other primary program inputs use only the **executable/analyzable source-file class**,
- declaration-only files may still participate as type-loading side inputs,
- `check`, `fmt`, and `lint` may accept declaration-only files explicitly,
- passing a declaration-only file where a command requires an executable/analyzable primary input is the canonical input-kind mismatch path (`E5007`), not general CLI misuse.

### Canonical project file set

Project discovery starts from the union of those two source-file classes.

Runtime-bearing entrypoints and direct executable inputs still use only the **executable/analyzable source-file class**.

### Default project-discovery rule

This is the canonical **default project-root walk**.

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

Configless project mode rules follow one shared **configless install split**:
- plain `kali install` is a no-op success when there are no dependency inputs,
- explicit registry-package add (`kali install <pkg>` / `kali install --dev <pkg>`) creates the minimal manifest `{ "schemaVersion": 1 }` first,
- explicit raw-URL install may create lock/cache state but must not create a placeholder manifest by itself.

## Canonical scaffold filename convention
In schema v1, `kali init` uses one minimal filename convention for the built-in templates:
- default app template → `main.ts`
- library template (`kali init --lib`) → `lib.ts`

Rules:
- these filenames are part of the default scaffold contract, not guesses made later by `run` or `build`
- later template specs may introduce other filenames only explicitly; they should not silently redefine the schema-v1 defaults
- docs should reference this convention instead of repeating the two filenames ad hoc in multiple chapters

## Canonical Dependency-Management Mutability Rule

In early phases, `kali install` is the only command that mutates project-managed dependency state.

Non-install commands must not silently:
- rewrite manifests,
- repair lockfiles,
- fetch and materialize missing dependency state as a hidden side effect.

They should fail with the canonical dependency-state diagnostic path instead.

## Shared Install State vs Command-Time Package Selection

Kali uses one deliberate simplification for early package management:
- `install` locks versions and materializes package contents,
- later commands choose the final package edge at command time from that already-installed metadata using the effective analysis/runtime context.

Consequences:
- one `kali.lock` plus one materialized package tree serves both the default Deno-oriented standalone path and the supported browser-targeted analysis/build paths in Phase 1,
- changing `apiSurface` between `deno` and a supported browser-targeted context changes package entry selection, not whether the project is considered installed,
- separate per-surface installs/lockfiles must not be implied unless a later lockfile revision explicitly introduces that complexity.

## Install-Time Declaration Graph

The dependency-owning declaration set that `kali install` reconciles for one effective project root.

It includes:
- registry dependencies declared in `kali.json` (`dependencies` / `devDependencies`),
- import-map declarations from `kali.json#imports`, with only raw-URL rewrites contributing external materialization state,
- source-level raw URL imports discovered from the project's canonical discovery result for that root.

Rules:
- plain `kali install` reconciles this graph into the shared project-managed dependency state (`kali.lock`, `node_modules/`, and `.kali/cache/urls/` as applicable),
- explicit file targets passed to non-install commands do **not** retroactively widen this graph,
- if explicit non-install command targets reach additional raw URL dependency state outside the currently installed graph, the command fails with the canonical dependency-state path (`E5004`) until the project's discoverable declaration set is updated and `kali install` is rerun.

This term exists so CLI, package-management, config-discovery, and dependency-state diagnostics can all refer to the same install-owned boundary without re-explaining it differently.

## Identity-Only Registry Target

Several early package workflows intentionally take only a registry **identity**, not an inline version selector. Canonical examples:
- `kali install lodash`
- `kali install --dev jsr:@std/path`
- `kali package-effects lodash`
- `kali package-audit jsr:@std/path`

The command then applies the shared **stable-release selection rule (schema v1)**. This keeps early CLI/package flows deterministic and simple.

## Registry Package Identifier

The canonical schema-v1 spelling for a registry package target. Examples:
- npm: `lodash`, `@types/node`
- JSR: `jsr:@std/path`

This term is used consistently across:
- `kali install`
- `kali package-effects`
- `kali package-audit`
- manifest keys under `dependencies` / `devDependencies`
- logical-root labels such as effect-report `entryPoints`

## Registry package identifier vs package coordinate

Kali intentionally uses two related representations for registry packages:
- **registry package identifier** — the user-facing string spelling used by CLI arguments, manifest keys, diagnostics, and logical-root labels such as effect-report `entryPoints`; examples: `lodash`, `jsr:@std/path`
- **package coordinate** — the structured JSON form used when a schema needs decomposed metadata, typically `{ registry, name, version }`

Rules:
- npm package coordinates keep `registry: "npm"` and `name` as the bare npm package name
- JSR package coordinates keep `registry: "jsr"` and `name` as the registry-native package name **without** the `jsr:` identity marker; the prefix stays represented by `registry`, not duplicated inside `name`
- when a schema needs a stable user-facing root label or diagnostic spelling, prefer the **registry package identifier** form rather than reconstructing an ad hoc string from a package coordinate
- docs should not invent a third spelling such as embedding the `jsr:` prefix into JSON `name` fields while also carrying `registry: "jsr"`

## Stable-Release Selection Rule (schema v1)

When a schema-v1 workflow accepts an **identity-only registry target**, Kali resolves exactly one concrete version using this rule:
- select the latest non-yanked stable published release for that registry package identifier,
- do not silently choose a prerelease,
- do not infer a different version from ambient project install state unless an owning chapter later adds an explicit lock-aware/version-aware mode,
- if no acceptable stable release exists, fail with the canonical `E5001` path.

This rule keeps early install and single-package analysis flows deterministic and project-independent.

## Exact-Version-First Registry Manifest Rule (schema v1)

When schema-v1 writes a registry dependency into `kali.json`, the recorded value is the exact resolved version string, not a SemVer range.

Rules:
- this applies to registry dependency values under `dependencies` and `devDependencies`,
- explicit registry adds via `kali install <pkg>` and `kali install --dev <pkg>` therefore use the **stable-release selection rule (schema v1)** first, then write that exact resolved version into the manifest,
- lockfile state and manifest intent should stay tightly aligned in schema v1,
- wider range syntax may be added later only as a separately documented manifest/CLI contract rather than being implied by identity-only install flows.

This keeps manifest edits deterministic and AI-friendly while avoiding a second hidden version-selection policy between `kali.json` and `kali.lock`.

## Registry-Analysis Project-Independence Rule

For `package-effects` and `package-audit` in schema v1:
- version selection follows the **stable-release selection rule (schema v1)**,
- current-project manifest/lock/install state does not pick a different version,
- commands may use the shared **registry-analysis cache**,
- commands must not mutate `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/`.

`package-effects` may still inherit its **effective inherited analysis context**; this rule is about dependency state and version selection, not about ambient analysis semantics.

## Effective npm-Scriptable Install Work

`--allow-scripts` is meaningful only when the current `install` invocation includes npm package work that could actually run lifecycle scripts.

Rules:
- this is **invocation-scoped**, not a project-wide switch;
- it includes directly requested npm package targets and any transitively touched npm dependencies that the current install must newly materialize, relink, or otherwise reconcile in a way that could run lifecycle hooks;
- a clean no-op install on an already-synchronized graph has **empty** effective npm-scriptable install work, even if the project depends on npm packages;
- if that set is empty, `kali install --allow-scripts` is invalid usage rather than permission to silently behave like plain `install`.

## Install-Time npm-Package Hook Path

The `--allow-scripts` escape hatch is the schema-v1 **install-time npm-package hook path**.

Rules:
- it is limited to the invocation's **effective npm-scriptable install work**;
- it is not meaningful for explicit `jsr:` targets, raw URL targets, or non-install commands;
- it does **not** imply Node runtime support, project sandbox participation for install hooks, or participation in normal `kali effects` / sandbox-policy contracts;
- it does **not** make the excluded **native/binary/bootstrap-heavy package contract** supported.

## Canonical Numeric-Limit Semantics

Kali uses one cross-spec numeric-limit rule:
- positive-budget dimensions use omission as the “unspecified” state and reject `0`,
- zero-capable concurrency counters may use `0` as an explicit deny/tightening value.

Examples:
- policy fields `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` must be positive when present,
- CLI overrides `--max-memory`, `--max-cpu`, and `--max-open-files` follow the same positive-only rule after unit normalization,
- `resources.maxSpawnedProcesses`, `resources.maxThreads`, `--max-spawned-processes`, and `--max-threads` may use `0` as an explicit deny/tightening value,
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
