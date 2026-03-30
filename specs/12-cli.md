# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure and workflow vocabulary (`init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`), without implying flag-for-flag Deno parity or that every Deno command shape automatically exists in the same phase
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases
6. **Single-channel machine output**: when `--output json` is selected, command metadata and any captured program streams are emitted through the JSON envelope rather than interleaving raw stdout/stderr text that would corrupt the payload

## Shared Flags

These flags are shared across the CLI, but some apply only to specific command families. For the canonical meaning of **API surface**, **build mode**, **runtime profile**, and **availability context**, see [SPEC.md](../SPEC.md). For maturity/availability rules, see [19 — Feature Maturity](19-feature-maturity.md).

Ownership rule:
- this chapter owns **CLI shape**: flags, arity, command-local behavior, output rules, and exit codes
- [19 — Feature Maturity](19-feature-maturity.md) owns whether a documented command/feature is actually available in a given **availability context**
- [18 — Schemas](18-schemas.md) owns the machine-readable JSON shapes
- when a rule is already defined in one of those owners, prefer a short cross-reference over repeating a second full version here

Command-family terminology used in this chapter:
- these labels describe **command shape and behavior**, not guaranteed current-phase availability
- later command/artifact families in this chapter follow the shared **defined command family** rule from [SPEC.md](../SPEC.md): for example `effects`, `package-effects`, `package-audit`, `build --capi`, and `build --component` may already have stable schema-v1 shapes here while remaining phase-gated
- canonical availability promises live in [19 — Feature Maturity](19-feature-maturity.md)
- **execution commands**: `run` and `test`
- **build-like commands**: `build`, plus the compile step embedded inside `run` and `test`
- **diagnostic-producing commands**: `check`, `effects`, `package-effects`, `build`, `run`, `test`, `fmt --check`, and `lint`
- **JSON-producing mode**: use the shared term from [SPEC.md](../SPEC.md); in schema v1 this means either a **native-JSON command** in its default success mode or any invocation with `--output json`

Canonical command-input mode rule (shared with [SPEC.md](../SPEC.md)):
- `run`, `build`, and `effects` are schema-v1 **direct-input commands**: once available, they require exactly one explicit primary source input and do not guess `main.ts` or invent a project-default file
- for `run`, that source input is an executable/analyzable entrypoint
- for `build`, that source input is one explicit primary module input whose artifact role depends on the selected artifact mode
- for `effects`, that source input is one explicit analysis root
- `check` is a **hybrid analysis command**: it accepts explicit file arguments, or falls back to the canonical project-discovery result when no files are provided
- `fmt`, `lint`, and `test` are **project-oriented commands** when invoked without explicit file arguments
- `install` is the canonical **dependency-graph command**: with no explicit install target it reconciles the discovered project dependency graph, including raw URL imports found through project discovery
- `check` without explicit files, project-oriented no-argument discovery, and the source-discovery portion of `install` are all covered by the shared **discovery-driven command** term from [SPEC.md](../SPEC.md)
- `package-effects` and `package-audit`, when available, are the canonical **registry-analysis commands** and follow the shared **single-package registry-analysis command** rule from [SPEC.md](../SPEC.md)
- `init` is not a direct-input source command

Canonical early-phase direct-input arity rule:
- `run`, `build`, and `effects` each take **exactly one** explicit primary source input in schema v1
- zero explicit source inputs for those commands is the canonical invalid-usage diagnostic `E5008`
- more than one explicit source input for those commands is also `E5008` unless a later spec introduces a documented multi-input mode
- `check`, `fmt`, `lint`, and `test` are the canonical **set-oriented explicit-file commands** from [SPEC.md](../SPEC.md): when explicit files are supplied, those paths are treated as one file set rather than as separate single-entry invocations

Canonical install-target and package-argument arity rule:
- `kali install [target]` accepts **zero or one** explicit install target in early phases
- that install target may be either a schema-v1 identity-only registry target or a raw URL target
- `kali package-effects <package>` and `kali package-audit <package>` follow the shared **single-package registry-analysis command** rule from [SPEC.md](../SPEC.md)
- passing more than the allowed number of explicit install targets/package arguments is `E5008` rather than permission to invent an undocumented batch mode
- omitting the required explicit registry-package argument for a registry-analysis command is also `E5008`
- flags that conceptually modify an explicit registry-package target (for example `kali install --dev`) require that registry target in early phases; using them without one is also `E5008`

Canonical input-kind rule:
- `run`, `build`, `effects`, and discovered `test` entrypoints/primary inputs accept only the shared **executable/analyzable source-file class** from [SPEC.md](../SPEC.md)
- `check`, `fmt`, and `lint` accept that same class plus declaration-only files, per the shared **canonical source-file classes** rule in [SPEC.md](../SPEC.md)
- declaration-only files may therefore be checked/formatted/linted directly and may also participate in ambient type loading and package type resolution
- declaration-only files are never valid runtime-bearing entrypoints or build/effect primary inputs; passing one where an executable entrypoint or build/effect primary input is required should fail explicitly with the canonical invalid-entrypoint diagnostic described in [specs/15-errors.md](15-errors.md) rather than being treated as an empty program or silently ignored
- when a command runs without explicit file arguments, it should discover files using the canonical project-discovery rules from [SPEC.md](../SPEC.md) rather than inventing a command-local root walk

Naming rule:
- CLI keeps short flag names such as `--api`
- `kali.json` keeps the canonical leaf keys under `compilerOptions`: `apiSurface`, `buildMode`, and `runtimeProfiles`
- compatibility switches live in config under `compat.features`, while self-contained emitted JSON flattens that same semantic set to `compatFeatures`
- new docs, generated config, and machine-readable examples should use only these canonical names instead of inventing aliases

Canonical config-discovery rule:
- unless a later spec adds an explicit `--config` override, commands discover the effective project config by searching the current working directory and then its ancestors for the nearest `kali.json`
- if none exists, the command runs in the canonical **configless project mode** from [SPEC.md](../SPEC.md), with the current working directory as the effective project root
- `kali init` is the one early-phase exception: it is **current-directory-scoped** and does **not** reuse an ancestor `kali.json` as its target root
- explicit CLI file arguments do **not** relocate that chosen config/root; they resolve relative to the current working directory, while config-owned relative paths continue to resolve relative to the directory containing the discovered `kali.json`
- follow the canonical **explicit path boundary rule** from [SPEC.md](../SPEC.md): file-accepting source-command targets must stay inside the effective project root, must not point into a nested child project that has its own `kali.json`, and bypass `include` / `exclude` only after they are explicitly named
- recursive project discovery for no-argument `check` / `fmt` / `lint` / `test` and for no-package-argument `install` graph scanning must stop at nested child directories that contain their own `kali.json`; those child roots are separate projects in schema v1

Effective-context validation rule:
- command validation always runs against the fully merged **effective command context** (built-in defaults, then discovered config, then CLI flags)
- therefore config-selected values trigger the same maturity/usage checks as explicit flags for the axes that actually participate in that command's semantics; the CLI must not silently "fix up" an inherited participating context by falling back to some other API surface/profile
- non-participating axes are ignored rather than gated: for example `check` ignores inherited `buildMode`, and early `package-audit` follows **context-free registry analysis (schema v1)** from [SPEC.md](../SPEC.md)
- examples: config-selected `apiSurface = node` still causes plain `kali run main.ts` or `kali test` to hit the Node phase gate (`E5006`), and config-selected `apiSurface = browser` still makes plain `kali build main.ts` invalid early-phase usage (`E5008`) until `--bundle` is selected
- config-selected `apiSurface = browser` also keeps plain `kali run main.ts` and plain `kali test` on the same browser-runtime/test gate as their explicit `--api browser` forms (`E5006`); omitting the flag does not cause a silent fallback to `deno`
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): command-shape/arity first, then base command availability, then finer inherited-context/profile gates inside that command

| Flag | Scope | Description |
|------|-------|-------------|
| `--verbose` | all commands | Detailed output: timing per phase, optimization decisions |
| `--output json` | all commands | Request the standard machine-readable JSON output mode for that command: wrap native-JSON payloads in the command envelope, or emit the envelope itself for envelope-only JSON commands |
| `--pretty` | JSON-producing mode | Pretty-print the active JSON document without changing its schema; meaningful only for **native-JSON commands** or when `--output json` is active (including **envelope-only JSON commands**) |
| `--quiet` | all commands | Suppress non-error status/progress output; for data-producing commands such as `effects` and `package-effects`, it must not suppress the primary payload itself |
| `--max-errors N` | diagnostic-producing commands | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | text-output commands | Color output control |
| `--api deno\|node\|browser` | `check`, `effects`, `build`, `run`, `test` | Select host API surface; unsupported surfaces for the current **availability context** must error explicitly (for example, early browser builds require `--bundle`) |
| `--compat <feature[,feature...]>` | `check`, `effects`, `build`, `run`, `test` | Enable documented compatibility features such as `eval` only when that feature is implemented for the selected **availability context**; in schema v1, `eval` also covers the `Function()` constructor path |
| `--fast` | `build`, `run`, `test` | Fastest compile time, minimal optimization (default build mode) |
| `--release` | `build`, `run`, `test` | Standard optimization profile |
| `--release-advanced` | `build`, `run`, `test` | Aggressive optimization profile |
| `--sandbox <policy>` | `check`, `build`, `run`, `test` | Attach and validate a sandbox policy file (canonical default filename: `kali.policy.json`, but any explicit path is allowed); in Phase 1 this enforces at runtime for `run`/`test` and validates policy/config for `check`/`build` |
| `--max-memory <size>` | execution commands | Override the invocation memory cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-cpu <duration>` | execution commands | Override the invocation CPU cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-open-files N` | execution commands | Override the invocation open-file-handle cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-spawned-processes N` | execution commands | Override the invocation child-process cap; may only tighten the effective limit. Follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value in Phase 1, while positive values stay availability-gated until subprocess support exists. |
| `--max-threads N` | execution commands | Override the invocation thread cap for the threaded runtime profile; may only tighten the effective limit. Follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value in Phase 1, while positive values stay availability-gated until the threaded profile exists. |
| `--wasm-threads` | `check`, `effects`, `build`, `run`, `test` | Opt into the later threaded runtime profile required for `SharedArrayBuffer` / `Atomics`; before that profile exists, or on unsupported targets, the command must fail with `E5006` |

`--fast`, `--release`, and `--release-advanced` are mutually exclusive; config files should use the single `compilerOptions.buildMode` field instead of parallel booleans. `run` and `test` inherit the selected build mode for their internal compile step. Runtime-profile toggles such as `--wasm-threads` map to entries in `compilerOptions.runtimeProfiles` rather than to separate booleans.

Shared source-graph gating rule:
- `check`, `effects`, `build`, `run`, and `test` share the same participating runtime-profile / compatibility axes for `--wasm-threads` and `--compat eval` unless a later command-specific row says otherwise
- explicit flags and inherited config are equivalent here too: `compilerOptions.runtimeProfiles = ["wasm-threads"]` and `compat.features = ["eval"]` must trigger the same maturity gate as the corresponding CLI flags instead of being silently ignored because the user omitted the explicit flag spelling
- command sections may show only representative examples for those shared gates to keep the CLI chapter readable; [19 — Feature Maturity](19-feature-maturity.md) remains the availability owner

Package-analysis flag/context simplification:
- follow the canonical command-context axis participation table, `analysis context` term, **registry-analysis context split**, and **registry-analysis command split** in [SPEC.md](../SPEC.md)
- practical shortcut: the canonical **source-graph commands** from [SPEC.md](../SPEC.md) own the explicit `--api` / `--compat` / `--wasm-threads` flag family; registry-analysis commands do not
- in schema v1, `package-effects` inherits only the semantic analysis axes through the shared **effective inherited analysis context**, records them in `report.analysisContext` using the emitted field names `apiSurface`, `runtimeProfiles`, and `compatFeatures`, and follows the shared **axis-aligned inherited analysis gating** rule
- in schema v1, `package-audit` follows **context-free registry analysis (schema v1)** and keeps the simpler envelope-only machine-output path owned by [18 — Schemas](18-schemas.md)
- `buildMode` and `sandbox` remain non-semantic for `package-effects` in early phases
- examples later in this chapter describe the canonical command shape/output contract for these registry-analysis commands, not an unconditional promise that they are already available in Phase 1

Sandbox-flag clarification:
- follow the shared **workflow-owner split** from [SPEC.md](../SPEC.md): in schema v1, `--sandbox <policy>` belongs only to the runtime-enforcement and static-policy-validation owners, not to reporting, install, formatting, linting, init, or registry-audit owners
- the CLI `--sandbox <policy>` flag is therefore reserved for the canonical sandbox-aware commands: `run`, `test`, `check`, and `build`
- the canonical default policy filename is `kali.policy.json`, but the flag accepts any explicit policy-file path; relative CLI paths resolve from the current working directory, while top-level `kali.json#sandbox` remains config-relative
- commands that merely ignore top-level `kali.json#sandbox` still do **not** accept a CLI `--sandbox` flag in early phases
- therefore `kali effects --sandbox ...`, `kali package-effects --sandbox ...`, `kali package-audit --sandbox ...`, `kali install --sandbox ...`, `kali fmt --sandbox ...`, `kali lint --sandbox ...`, and `kali init --sandbox ...` are all invalid command usage (`E5008`) unless a later spec explicitly adds such a mode

Build-mode continuity rule:
- these three build-mode names are stable from Phase 1 onward
- later phases deepen what `release` and `release-advanced` actually do, but they should not force users to learn a second generation of optimization-mode names just because MIR/LIR passes became more capable

## Command-Specific Flags

To keep the shared-flag table small and avoid implying that every convenience flag is globally meaningful, command-local switches are listed here.

| Flag | Scope | Description |
|------|-------|-------------|
| `--bundle` | `build` | In Phase 1, selects the browser-targeted artifact path and therefore requires the **effective** `apiSurface` to be `browser` (from CLI or config); it is not a generic "multi-artifact output" switch, and any future extension must be specified explicitly |
| `--lib` | `build`, `init` | For `build`: select the base library/export artifact mode, following the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md). For `init`: scaffold a library-oriented project template only |
| `--capi` | `build` | Emit the Phase-2 public C-embedding artifact set (`wasm-module` + `wit` + `c-header` + `cabi-metadata`) |
| `--component` | `build` | Emit a WebAssembly Component Model wrapper for a library/export-oriented build once that packaging path exists; phase-gated until the component flow is implemented |
| `--validate-ir` | `build` | Run internal IR validators as a debugging/developer aid |
| `--max-specializations N` | `build`, `run`, `test` | Override the specialization fan-out cap upper bound for a single invocation; this is an upper bound, not a promise that the current build mode will spend the full budget, and `--fast` may still skip most user-authored generic specialization entirely |
| `--fix` | `lint` | Apply only structured, tool-generated safe fixes for lint diagnostics in the selected file/project set |
| `--check` | `fmt` | Report formatting drift without rewriting files |
| `--filter <pattern>` | `test` | Run only matching tests |
| `--coverage` | `test` | Emit test coverage data once the coverage report contract is stabilized; before then this flag is phase-gated or explicitly experimental |
| `--dev` | `install` | Add the named registry dependency to `devDependencies` instead of `dependencies` |
| `--allow-scripts` | `install` | Opt into the schema-v1 **install-time npm-package hook path** for that install invocation only; meaningful only when the invocation has non-empty **effective npm-scriptable install work** from [SPEC.md](../SPEC.md) |

Interpretation rule:
- command-specific flags inherit the same phase/profile gating rules as the command they belong to
- documenting a command-specific flag here does **not** imply it needs a separate feature-maturity row unless it changes a phase promise or machine-readable contract
- build artifact-mode flags follow the canonical matrix in [SPEC.md](../SPEC.md): in early phases `--bundle`, `--lib`, `--capi`, and `--component` are one small closed set of mutually exclusive selectors unless a later spec explicitly says one implies another
- those selectors choose the build's shared **compile intent** from [SPEC.md](../SPEC.md): omitting all four keeps the default executable compile intent, `--bundle` keeps executable compile intent while selecting the browser-targeted output/host-adapter path, and `--lib` / `--capi` / `--component` are the explicit library compile-intent selectors
- supplying more than one explicit selector from that set (for example `--bundle --lib`, `--bundle --capi`, `--bundle --component`, `--lib --capi`, `--lib --component`, or `--capi --component`) should use the canonical invalid-usage diagnostic `E5008`, not a feature-maturity rejection
- in Phase 1, `--bundle` is the browser packaging selector only: `kali build --bundle ...` requires the **effective API surface** to be `browser`, and `kali build --bundle` under an effective API surface of `deno` or `node` is invalid command usage (`E5008`) rather than a feature-maturity rejection, because the browser bundle mode itself exists but the selected flag/config combination is contradictory
- in early phases, `--lib`, `--capi`, and `--component` are **library-oriented artifact modes**: non-browser, export-oriented build modes derived from a **statically known export surface** as defined in [SPEC.md](../SPEC.md)
- those library-oriented modes still obey the ordinary build-command API-surface gates: `kali build --lib --api browser ...`, `kali build --capi --api browser ...`, and `kali build --component --api browser ...` are `E5008` contradictions because browser mode is only defined for `--bundle`, while `kali build --lib --api node ...` remains on the same Phase 3 `E5006` path as other early `--api node` builds
- `--lib` is the **base library artifact** mode; `--capi` and `--component` are later **public embedding artifact flows** over that same exported-library contract rather than unrelated semantics
- because `--capi` and `--component` already choose exported-library semantics, users should not combine them with `--lib` in early phases; those flags are separate artifact-mode selectors, not additive modifiers
- WIT sidecars are not a separate artifact-mode selector: under the shared **embedding-stability split** from [SPEC.md](../SPEC.md), Phase 1 plain `--lib` emits the core library `wasm-module`, and once the Phase-2 **public embedding surface** stabilizes, the relevant library-oriented modes emit WIT by default so callers do not have to choose between separate "C ABI" and "component" interface-description paths

Config-array normalization rule:
- `compilerOptions.runtimeProfiles` and `compat.features` are set-like lists, not ordered pipelines
- entries should be unique; duplicates are config errors (`E5009`), not something tools silently deduplicate away
- unknown entries are diagnosed instead of ignored
- when those sets are re-emitted in machine-readable payloads such as `analysisContext`, producers should use stable lexical order so caches and diffs do not depend on original config ordering

Configuration precedence is intentionally simple:
1. CLI flags override the effective discovered `kali.json`
2. the effective discovered `kali.json` overrides built-in defaults
3. Sandbox policy caps, when a policy is attached, remain upper bounds for runtime capabilities and resource limits

That means command-line resource flags can tighten a run relative to policy/config, but they must not silently widen a sandbox policy. If no policy is attached, those direct invocation flags simply become the effective cap for the current command instead of being compared against an implicit allow-all policy. In Phase 1 this tightening path applies directly to `--max-memory`, `--max-cpu`, and `--max-open-files`. For later-gated caps such as `--max-spawned-processes` and `--max-threads`, reuse the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md).

Interpretation rule:
- the resulting merged values are the command's one **effective command context** for validation, lowering, and reporting
- unsupported inherited config values do not get ignored just because the user omitted the matching CLI flag

Canonical path-resolution rule:
- ordinary CLI path arguments (entry files, explicit file lists, and `--sandbox <path>`) are resolved relative to the current working directory
- top-level `kali.json#sandbox` and other config-owned relative paths/globs are resolved relative to the directory containing that `kali.json`
- after resolution, commands should preserve one normalized absolute/canonical path internally so diagnostics and caching do not depend on the caller's original spelling

Canonical resource-literal rule:
- `--max-memory` accepts either a plain byte count or a size literal with one of: `kb`, `mb`, `gb`, `kib`, `mib`, `gib`
- `--max-cpu` accepts either a plain millisecond count or a duration literal with one of: `ms`, `s`, `m`
- `--max-open-files` accepts a plain positive integer count and mirrors `resources.maxOpenFiles`: it caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- `--max-spawned-processes` accepts a plain non-negative integer count
- `--max-threads` accepts a plain non-negative integer count
- CLI parsing normalizes these to bytes, milliseconds, and integer counts before comparing them with sandbox-policy limits
- follow the canonical numeric-limit semantics from [SPEC.md](../SPEC.md): `--max-memory`, `--max-cpu`, and `--max-open-files` must be **positive** when present, so `0` is invalid rather than a hidden deny form
- only `--max-spawned-processes` and `--max-threads` use the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value for those counters because zero concurrent uses is meaningful there
- schema v1 policy files keep the simpler integer fields `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`, `resources.maxSpawnedProcesses`, and `resources.maxThreads`; CLI literals/counts are a convenience syntax over that same effective-limit model rather than a second resource schema

Default standalone context (schema v1):
- reuse the canonical term from [SPEC.md](../SPEC.md)

It is the default interpretation of examples such as `kali run main.ts`, `kali test`, and `kali build main.ts` unless the example explicitly overrides a field. `kali check main.ts` and `kali effects main.ts` reuse the same baseline only for the axes that actually participate in those commands, while inherited package analysis uses the narrower **default inherited analysis context (schema v1)** from [SPEC.md](../SPEC.md).

## Commands

Reading rule for the examples below:
- command examples in this chapter define **shape, flags, and output contracts** first
- examples that mention later-phase commands or contexts (for example `effects`, `package-effects`, `package-audit`, `--capi`, `--component`, `--api node`, or standalone browser `run` / `test`) do **not** override the availability owner in [19 — Feature Maturity](19-feature-maturity.md)
- when an example is both well-formed and phase-gated, read it as "this is the stable command spelling once that maturity row opens" rather than as an implied Phase-1 promise

### `kali run <file>`
Compile and execute a TypeScript/JavaScript file.
```bash
kali run main.ts                           # Run with default settings
kali run --sandbox kali.policy.json main.ts # Run with sandbox
kali run --max-memory 256mb main.ts        # Resource limit
kali run --max-cpu 10s main.ts             # CPU time limit
kali run --max-open-files 32 main.ts       # Open-file-handle limit
kali run --max-spawned-processes 0 main.ts # Disallow child processes for this run
kali run --api node main.ts                # Use Node.js API surface (Phase 3 target)
kali run --api deno main.ts                # Use Deno API surface (default)
kali run --api browser main.ts             # Later compatibility; unavailable in early standalone phases because browser is a browser-targeted context first
kali run --wasm-threads main.ts            # Enable WASM threads (SharedArrayBuffer, Atomics; opt-in only)
```

`kali run` is a direct-input command in early phases: it requires exactly one explicit executable/analyzable source entrypoint and does not guess a project default such as `main.ts`.

Initial implementations use wasmtime; alternative runtime backends are a later-phase feature. Feature flags and subcommands that depend on later phases should be hidden or clearly diagnosed when unavailable rather than exposed as silently nonfunctional options.

When a command or flag is rejected due to maturity/availability gating, the CLI should use the canonical feature-maturity diagnostic shape from [specs/15-errors.md](15-errors.md) rather than ad hoc wording.

Canonical interpretation rules:
- `--api` selects an **API surface**, but support is command-dependent.
- follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): supported early browser shapes are the shared **Phase-1 browser-targeted command set** (`kali check [files...]`, including both the project-discovery no-file form and explicit-file-set forms, and `kali build --bundle <file>` when the effective `apiSurface` is `browser`, including their supported `--sandbox` variants); wrong browser build shapes use `E5008`, while browser execution/test requests use `E5006` until Kali defines a standalone browser runtime/test contract.
- `--api node` is phase-gated consistently across `check`, `effects`, `build`, `run`, and `test`; early phases reject it with `E5006` rather than exposing a partial Node surface.
- explicit `--api ...` and inherited `compilerOptions.apiSurface = ...` are equivalent here too: plain `kali run main.ts` and plain `kali run --sandbox kali.policy.json main.ts` must validate against the same effective API surface and therefore hit the same Node/browser execution gates as their explicit `--api node` / `--api browser` forms instead of silently falling back to `deno`.
- `--compat ...` is the one shared switch for later-phase dynamic compatibility features. If the named feature is not implemented yet, the command still fails with `E5006`.
- in schema v1, `--compat eval` is the only stable compatibility-feature spelling and it gates both direct `eval` and `Function()`; the CLI should not invent a separate `--compat function-constructor` alias.
- sandbox permission and compatibility enablement are separate axes: a policy that allows `effects.eval` does **not** implicitly turn on `--compat eval`, and `--compat eval` does **not** bypass a stricter sandbox policy.
- `--wasm-threads` selects a different runtime profile rather than a small optimization toggle. Until that threaded profile exists, the flag is rejected. After it exists, if the selected target/engine/profile cannot honor it, the command must still reject it explicitly instead of silently dropping thread support.
- `--max-spawned-processes` and `--max-threads` follow the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value, while positive values must be rejected explicitly until subprocess/thread support actually exists.

Inherited execution-context shorthand:

| Effective `apiSurface` | Command spelling | Result |
|---|---|---|
| `deno` (default) | `kali run main.ts` / `kali run --sandbox kali.policy.json main.ts` | Supported early standalone execution path |
| `node` | `kali run main.ts` / `kali run --sandbox kali.policy.json main.ts` | Same Node execution gate as explicit `--api node`; no silent fallback to `deno` |
| `browser` | `kali run main.ts` / `kali run --sandbox kali.policy.json main.ts` | Same browser execution gate as explicit `--api browser`; no silent fallback to `deno` |

Sandbox flag behavior is intentionally phase-gated:
- `kali run --sandbox ...` is a Phase 1 feature for runtime policy enforcement.
- `kali check/build --sandbox ...` validate the policy file/config in Phase 1.
- Full inferred-effect-vs-policy validation is a Phase 2 feature.
- Policy validation must also reject policies that try to enable capabilities unavailable in the selected command/profile/phase (for example `effects.eval: true` before the eval compatibility path exists, `effects.eval: true` without effective `--compat eval`, or positive values for the **feature-gated zero-capable execution budgets** from [SPEC.md](../SPEC.md) before subprocess/thread support exists).
- For browser-targeted `check --api browser --sandbox ...` and `build --bundle --api browser --sandbox ...`, follow the **browser-targeted static sandbox contract** and the **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md): browser-targeted sandboxing is a static compatibility check over the documented mediated subset, and browser-policy validation should reference that one canonical `resources.*` rule instead of repeating a second list here.
- Policy files remain declarative; any later host-registered sandbox policy predicates are an embedding-oriented extension, not a second inline policy language.
- If neither CLI nor config attaches a policy, the command runs with **no project policy file**; direct resource flags such as `--max-memory` and later supported caps such as `--max-spawned-processes` still apply, but there is no hidden synthesized policy document behind the scenes. For `run`/`test`, those direct caps still contribute to the **effective execution envelope** from [SPEC.md](../SPEC.md).

### `kali build <file>`
AOT compile to a WASM module or linked artifact set.

Artifact-mode quick summary:

| Selector | Compile intent | Earliest phase | Early-phase meaning |
|---|---|---|---|
| *(default)* | executable | Phase 1 MVP | one linked executable-oriented WASM artifact |
| `--bundle` | executable | Phase 1 MVP | browser-targeted bundle path only, and only when the effective `apiSurface` is `browser` |
| `--lib` | library | Phase 1 MVP | Phase-1 **base library artifact** only; stable public library/WIT contract is later |
| `--capi` | library | Phase 2 target | public embedding artifact flow over the same **statically known export surface** |
| `--component` | library | Phase 2 target | Component Model packaging over the same **statically known export surface** |

Reading rule:
- this table is a CLI-local summary only
- the canonical artifact-mode matrix and cross-command compile-intent wording still live in [SPEC.md](../SPEC.md), and phase availability still lives in [19 — Feature Maturity](19-feature-maturity.md)

Canonical artifact-mode rule:
- `kali build` is a direct-input command in early phases: it requires exactly one explicit executable/analyzable primary source input and does not guess a project default such as `main.ts`
- in executable artifact mode that source input behaves as the program entrypoint; in library-oriented artifact modes it is the primary module input whose exports define the host-facing surface
- artifact selection follows the canonical matrix in [SPEC.md](../SPEC.md)
- omitting `--bundle`, `--lib`, `--capi`, and `--component` selects the default executable artifact mode and therefore the default **executable compile intent**
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive artifact-mode selectors unless a later spec explicitly defines one as an implication of another
- `--bundle` preserves executable compile intent while changing the host adapter/output contract to the browser-targeted bundle path
- explicit `--api ...` and inherited `compilerOptions.apiSurface = ...` are equivalent here too: plain `kali build main.ts`, `kali build --sandbox kali.policy.json main.ts`, `kali build --lib lib.ts`, `kali build --capi lib.ts`, and `kali build --component lib.ts` must validate against the same effective API surface as their explicit `--api ...` forms rather than silently falling back
- for the browser bundle shortcut specifically, the plain spelling `kali build --bundle main.ts` has two canonical outcomes owned by [19 — Feature Maturity](19-feature-maturity.md) — under the default/inherited non-browser API surface it is `E5008`, while under an inherited browser API surface it is the supported browser-bundle shortcut
- `--lib`, `--capi`, and `--component` switch the build to library compile intent
- reuse the shared **template selection vs build artifact mode split** from [SPEC.md](../SPEC.md): `kali init --lib` chooses a project template only and does not change the later default artifact mode of `kali build`
- WIT sidecars for public library/embedding outputs are an output detail of those artifact modes, not a separate mode flag
- these **library-oriented artifact modes** derive their host-facing surface from a **statically known export surface** as defined in [SPEC.md](../SPEC.md); they do not implicitly expose arbitrary internal declarations just because the source file was compiled in `--lib`/`--capi`/`--component` mode
- if Kali cannot determine that export surface statically, the library-oriented build fails with `E5011` instead of synthesizing reflection-based exports
- plain `--lib` is the Phase-1 **base library artifact**: it establishes the exported-library shape early, but under the shared **embedding-stability split** the stable public embedding/WIT contract remains part of the later Phase-2 **public embedding surface**
- they also keep the ordinary build-command API-surface semantics: Node-targeted library builds are still phase-gated with `E5006`, while browser-targeted library/embedding combinations are invalid command shapes (`E5008`) until a separate browser-library contract exists

`--capi` and the other **public embedding artifact flows** follow the embedding maturity rules in [specs/19-feature-maturity.md](19-feature-maturity.md): under the shared **embedding-stability split**, Phase 1 ships the base library artifact while the stable public embedding surface is a Phase 2 target.

Sandbox clarification:
- `kali build --sandbox ...` never executes the program; in Phase 1 it validates policy/config, and starting in the Phase 2 target window it also performs effect-vs-policy validation.
- `kali build --bundle --api browser --sandbox ...` follows the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md): it is a build-time compatibility check over the documented mediated subset, not automatic runtime sandbox enforcement once the emitted browser bundle is deployed into a real browser host.
- the same effective-context rule applies to inherited browser config: plain `kali build --sandbox kali.policy.json main.ts` under an inherited browser API surface is still the same non-bundle browser-build contradiction as explicit `kali build --api browser --sandbox kali.policy.json main.ts`, so it stays `E5008` until a non-bundle browser build mode exists.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default; artifact: kind=wasm-module, role=primary-executable)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # main.wasm + main.js (artifacts: main.wasm kind=wasm-module role=primary-executable; main.js kind=js-glue role=browser-glue)
kali build --bundle main.ts                # Same browser-bundle request once discovered config already makes the effective apiSurface `browser`
kali build --bundle --api node main.ts     # Invalid usage (E5008); --bundle is the browser-only artifact mode, so pairing it with a non-browser API surface is contradictory
kali build --api browser main.ts           # Invalid usage (E5008) in early phases; browser build path requires --bundle
kali build --api node main.ts              # Phase 3 target: Node API surface is not available early for builds either
kali build --lib lib.ts                    # Phase-1 base library artifact following the shared library-oriented instantiation rule and embedding-stability split from SPEC.md (kind=wasm-module, role=primary-library; from the Phase 2 target onward the same plain --lib path becomes the stable public library/WIT contract and adds kind=wit, role=interface-wit by default)
kali build --lib --api node lib.ts         # Phase 3 target: Node API surface remains build-gated for library-oriented modes too
kali build --lib --api browser lib.ts      # Invalid usage (E5008) in early phases; browser mode is a browser-targeted context tied to `check` and `build --bundle`, not a library artifact mode
kali build --capi lib.ts                   # Phase 2 target: lib.wasm + lib.wit + lib.exports.h + lib.cabi.json (artifacts: wasm-module + wit + c-header + cabi-metadata; roles: primary-library + interface-wit + embedding-header + embedding-metadata; `lib.exports.h` is the program-specific exports header, and `lib.cabi.json` is the generated `cabi-metadata` file, not the host ABI header `kali.h`; see specs/13-embedding.md)
kali build --capi --api node lib.ts        # Phase 3 target: still gated by the Node build surface even after the public embedding artifact flow exists
kali build --capi --api browser lib.ts     # Invalid usage (E5008) in early phases; browser mode remains the bundle-only browser-targeted path rather than an embedding artifact mode
kali build --component lib.ts              # Phase 2 target: lib.wasm + lib.wit + lib.component.wasm (artifacts: lib.wasm kind=wasm-module role=primary-library; lib.wit kind=wit role=interface-wit; lib.component.wasm kind=wasm-component role=primary-component)
kali build --component --api node lib.ts   # Phase 3 target: still gated by the Node build surface even after component packaging exists
kali build --component --api browser lib.ts # Invalid usage (E5008) in early phases; browser mode remains the bundle-only browser-targeted path rather than a component artifact mode
kali build --sandbox kali.policy.json main.ts # Phase 1: validate policy file/config; from the Phase 2 target onward also validate inferred effects
kali build --bundle --api browser --sandbox kali.policy.json main.ts # Build-time policy compatibility only; no automatic browser-runtime enforcement is implied after deployment
kali build --bundle --sandbox kali.policy.json main.ts # Same browser-targeted static-policy-validation request once discovered config already makes the effective apiSurface `browser`
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

Inherited build-context shorthand summary:

| Effective `apiSurface` | Plain command spelling | Result |
|---|---|---|
| `deno` (default) | `kali build main.ts` / `kali build --sandbox kali.policy.json main.ts` | Supported early executable build path |
| `node` | `kali build main.ts` / `kali build --sandbox kali.policy.json main.ts` | Same Node build gate as explicit `--api node`; no silent fallback to `deno` |
| `browser` | `kali build main.ts` / `kali build --sandbox kali.policy.json main.ts` | Invalid usage (`E5008`): same contradiction as explicit `kali build --api browser ...` until a non-bundle browser build mode exists |
| `node` | `kali build --lib lib.ts` / `kali build --capi lib.ts` / `kali build --component lib.ts` | Same Node build gate as the corresponding explicit `--api node` library-oriented form; no silent fallback |
| `browser` | `kali build --lib lib.ts` / `kali build --capi lib.ts` / `kali build --component lib.ts` | Invalid usage (`E5008`): same contradiction as the corresponding explicit browser library-oriented form |
| non-browser (`deno` / `node`) | `kali build --bundle main.ts` | Invalid usage (`E5008`): `--bundle` is browser-only |
| `browser` | `kali build --bundle main.ts` | Same supported request as explicit `kali build --bundle --api browser main.ts` |
| non-browser (`deno` / `node`) | `kali build --bundle --sandbox kali.policy.json main.ts` | Invalid usage (`E5008`): `--sandbox` does not change the browser-only meaning of `--bundle` |
| `browser` | `kali build --bundle --sandbox kali.policy.json main.ts` | Same supported request as explicit `kali build --bundle --api browser --sandbox kali.policy.json main.ts` |

This table is only a shorthand; [19 — Feature Maturity](19-feature-maturity.md) remains the availability owner.

### `kali check [files...]`
Type-check without compiling.
```bash
kali check                                 # Type-check the canonical project-discovery result
kali check main.ts                         # Type check executable/analyzable source
kali check src/a.ts src/b.ts               # Type check an explicit file set
kali check types.d.ts                      # Validate a declaration-only file directly
kali check --api browser                   # Browser-targeted project-discovery analysis context
kali check --api browser main.ts           # Browser-targeted analysis context for an explicit file set (no standalone DOM runtime implied)
kali check --api browser src/a.ts src/b.ts # Same browser-targeted analysis context over an explicit multi-file set
kali check                                 # Under inherited browser config, this is the same supported request as explicit `kali check --api browser`
kali check main.ts                         # Under inherited browser config, this is the same supported request as explicit `kali check --api browser main.ts`
kali check --api node                      # Phase 3 target: Node API surface is phase-gated for project-discovery checking too
kali check --api node main.ts              # Phase 3 target: same Node analysis gate for an explicit file set
kali check --sandbox kali.policy.json      # Phase 1: project-wide check + policy file/config validation; from the Phase 2 target onward, effect-vs-policy validation over the discovered project graph
kali check --api browser --sandbox kali.policy.json # Same browser-targeted validation path over the discovered project graph
kali check --sandbox kali.policy.json      # Under inherited browser config, the same browser-targeted static policy-validation request as explicit `kali check --api browser --sandbox ...`
kali check --sandbox kali.policy.json main.ts # Same validation, but scoped to the explicit file set
kali check --sandbox kali.policy.json src/a.ts src/b.ts # Same rule with multiple explicit files; --sandbox does not turn check into a direct-input command
kali check --api browser --sandbox kali.policy.json src/a.ts src/b.ts # Same browser-targeted validation path over an explicit multi-file set
```
`kali check` is the hybrid analysis command: it accepts explicit file inputs, and without them it falls back to the canonical project-discovery result. That remains true under `--api browser`, `--api node`, and `--sandbox`: API-surface selection changes only the analysis context, not the command's file-arity model, and attaching a policy still does not turn `check` into a direct-input command.

Inherited check-context shorthand:

| Effective `apiSurface` | Command spelling | Result |
|---|---|---|
| `deno` (default) | `kali check [files...]` | Supported standalone/default analysis context |
| `node` | `kali check [files...]` | Same Node analysis gate as explicit `kali check --api node [files...]`; no silent fallback to `deno` |
| `browser` | `kali check [files...]` | Same browser-targeted request as explicit `kali check --api browser [files...]` |
| `deno` (default) | `kali check --sandbox kali.policy.json [files...]` | Supported standalone/default policy-validation request |
| `node` | `kali check --sandbox kali.policy.json [files...]` | Same Node policy-validation gate as explicit `kali check --api node --sandbox kali.policy.json [files...]`; no silent fallback to `deno` |
| `browser` | `kali check --sandbox kali.policy.json [files...]` | Same browser-targeted static policy-validation request as explicit `kali check --api browser --sandbox kali.policy.json [files...]` |

This table is a CLI reading aid only; [19 — Feature Maturity](19-feature-maturity.md) remains the availability owner.

Declaration-only files are valid explicit file inputs for `check`; `run`, `build`, `effects`, and `test` primary inputs may not be declaration-only, and that input-kind mismatch should use the canonical invalid-entrypoint diagnostic (`E5007`).

Checker diagnostics may still carry structured `SuggestedFix` metadata for editors, embedders, and JSON consumers, but schema v1 keeps CLI autofix simpler: `--fix` is lint-only until the checker rewrite contract is mature enough to stabilize across project graphs, config-discovery mode, and overlapping multi-diagnostic edits.

### `kali effects <file>`
Output static effect analysis as JSON.

Status: Phase 2 target. This section documents a **defined command family** in schema v1; in Phase 1 the command may still be unavailable or explicitly marked experimental while the internal effect infrastructure stabilizes.
```bash
kali effects main.ts                       # Compact effect report JSON to stdout (default API surface: deno)
kali effects --api browser main.ts         # Browser-targeted effect analysis once the Phase 2 command exists
kali effects --api node main.ts            # Phase 3 target: Node API surface remains gated here too
kali effects --compat eval main.ts         # Phase 4 compatibility: dynamic-eval path reflected in effect analysis too
kali effects --pretty main.ts              # Pretty-printed effect report JSON
kali effects --output json main.ts         # Command envelope + effect payload
```
`kali effects` is a schema-v1 **native-JSON command** once it is available: by default it prints the effect-report payload directly, and with `--output json` it wraps that same payload in the standard command envelope. See [specs/18-schemas.md](18-schemas.md) for the canonical payload schema.

Analysis scope rule:
- the emitted payload includes `analysisContext`, which records `apiSurface`, `runtimeProfiles`, and emitted JSON field `compatFeatures` (the flattened report form of config key `compat.features`; see [SPEC.md](../SPEC.md))
- `kali effects <file>` summarizes effects for the full statically reachable graph rooted at that analysis input under the selected API surface/profile; it is not limited to syntax that appears textually in the one named file
- schema-v1 keeps the shared field name `entryPoints`, but for `kali effects` it records the report's logical root labels and therefore normally contains the single explicit analysis-root label for this command
- `effects` then summarizes the reachable program/dependency graph from those recorded logical roots

Sandbox-interaction rule:
- `kali effects` reports inferred effects only; it does **not** accept `--sandbox`
- effect-vs-policy validation belongs to `kali check --sandbox ...` and `kali build --sandbox ...`
- rejecting `kali effects --sandbox ...` keeps one canonical policy-validation workflow instead of two overlapping ones
- that rejection is `E5008`, not a feature-maturity error: the command intentionally has no sandbox-comparison mode

Input-kind and host-selection rules:
- once available, `kali effects` keeps that schema-v1 direct-input shape: it requires exactly one explicit executable/analyzable source-file analysis root and does not fall back to project-wide discovery
- `kali effects` accepts only the shared **executable/analyzable source-file class** from [SPEC.md](../SPEC.md); declaration-only files are type inputs, not effect-report primary inputs
- unless overridden by CLI/config, `kali effects` uses the same default API-surface selection as `kali check` (`apiSurface = deno`)
- `--api browser` follows the same browser API-surface analysis context as `kali check --api browser`; in Phase 2 this extends browser-targeted analysis to `effects` without implying standalone browser execution
- `--api node` remains phase-gated until the documented Node surface exists
- `--compat ...` affects effect analysis too: enabled compatibility paths such as `eval` change the reported effect set/dynamic reasons only when that compatibility feature is actually implemented for the selected phase/profile
- explicit flags and inherited config are equivalent here too: plain `kali effects main.ts` must validate against the full effective analysis context, so inherited `compilerOptions.apiSurface = browser|node`, inherited `compilerOptions.runtimeProfiles = ["wasm-threads"]`, or inherited `compat.features = ["eval"]` must hit the same gates as the corresponding explicit `--api ...`, `--wasm-threads`, or `--compat eval` forms instead of silently falling back to a simpler analysis mode

Inherited analysis-context shorthand:

| Effective context slice | Command spelling | Result |
|---|---|---|
| default (`apiSurface = deno`, no extra runtime profiles / compat features) | `kali effects main.ts` | Default standalone-style effect analysis once the Phase 2 command exists |
| `apiSurface = browser` | `kali effects main.ts` | Same browser-targeted effect-analysis request as explicit `kali effects --api browser main.ts` |
| `apiSurface = node` | `kali effects main.ts` | Same Node-gated request as explicit `kali effects --api node main.ts`; no silent fallback to `deno` |
| `runtimeProfiles = ["wasm-threads"]` | `kali effects main.ts` | Same threaded-profile gate as explicit `kali effects --wasm-threads main.ts`; no silent profile drop |
| `compat.features = ["eval"]` | `kali effects main.ts` | Same `eval` compatibility gate as explicit `kali effects --compat eval main.ts`; no silent compat-feature removal |

Compatibility rule:
- plain `kali effects ...` emits the raw effect-report payload
- `kali effects --output json ...` emits the standard command envelope with that same effect report under `payload`
- `--pretty` changes formatting only; it does not change the effect-report schema or field names
- if `--pretty` and `--output json` are combined, formatting applies to the outer command envelope while the nested effect payload remains schema-identical

### `kali fmt [files...]`
Format source files (implemented in `kali_fmt`).
```bash
kali fmt                                   # Format the canonical project file set relevant to formatting (executable/analyzable sources plus declaration-only files)
kali fmt --check                           # Check formatting (CI mode, exit code 1 if unformatted)
kali fmt main.ts                           # Format specific file
kali fmt src/a.ts src/b.ts                 # Format an explicit file set
```

Canonical discovery rule:
- `kali fmt` is project-oriented with no files, but when explicit paths are supplied it follows the shared **set-oriented explicit-file command** rule from [SPEC.md](../SPEC.md)
- project-oriented format discovery starts from the shared **canonical project file set** from [SPEC.md](../SPEC.md), which already covers executable/analyzable files plus declaration-only files
- when explicit file arguments are supplied, those paths are formatted directly if they belong to that same supported set
- `--check` changes rewrite behavior only; it does not change discovery, supported file kinds, or the **set-oriented explicit-file command** contract

### `kali lint [files...]`
Lint source files (implemented in `kali_lint`).
```bash
kali lint                                  # Lint the canonical project file set
kali lint --fix                            # Auto-fix where possible
kali lint src/a.ts src/b.ts                # Lint an explicit file set
```

Canonical discovery rule:
- `kali lint` is project-oriented with no files, but when explicit paths are supplied it follows the shared **set-oriented explicit-file command** rule from [SPEC.md](../SPEC.md)
- project-oriented lint discovery starts from the shared **canonical project file set** from [SPEC.md](../SPEC.md), matching the same source-file coverage as `kali fmt`
- when explicit file arguments are supplied, those paths are linted directly if they belong to that same supported set
- `--fix` is intentionally conservative: it applies only structured tool-provided edits rather than speculative rewrites or stylistic churn outside the selected lint rules
- when multiple lint fixes overlap, Kali must not partially apply a conflicting subset by guesswork; it should either choose one documented deterministic winner later or, in schema v1, leave the overlapping diagnostics unapplied and report them normally

### `kali test [files...]`
Run test files.
```bash
kali test                                  # Run discovered tests from the executable/analyzable source-file class
kali test --filter "math"                  # Filter by name
kali test --sandbox kali.policy.json       # Run tests in sandbox
kali test --coverage                       # Phase 2 target: with coverage report once the stable contract lands
kali test --api deno                       # Supported early standalone test profile
kali test --api node                       # Phase 3 target
kali test --api browser                    # Later compatibility; unavailable in early phases because browser support is limited to the shared Phase-1 browser-targeted command set first
```

Canonical discovery rule:
- default test discovery starts from the canonical project-discovery result, then matches `*.test.*` / `*_test.*` only across the shared **executable/analyzable source-file class** from [SPEC.md](../SPEC.md)
- declaration-only files are never test entrypoints even if they match the naming pattern
- if explicit file arguments are supplied to `kali test`, those paths bypass the naming-pattern discovery filter and are treated as direct test-module inputs instead
- each explicit `kali test` file must still belong to the shared **executable/analyzable source-file class**; passing a declaration-only file is the canonical invalid-entrypoint error (`E5007`), not a silent skip

Canonical host/profile rule: `kali test` follows the same early-phase API-surface gating as `kali run`, and analysis/build commands (`kali check`, `kali effects`, `kali build`) follow the same API-surface maturity rules for `--api node` / `--api browser` unless [specs/19-feature-maturity.md](19-feature-maturity.md) explicitly says otherwise.

Inherited execution-context shorthand:

| Effective `apiSurface` | Command spelling | Result |
|---|---|---|
| `deno` (default) | `kali test` / `kali test --sandbox kali.policy.json` | Supported early standalone test path |
| `node` | `kali test` / `kali test --sandbox kali.policy.json` | Same Node test-runtime gate as explicit `--api node`; no silent fallback to `deno` |
| `browser` | `kali test` / `kali test --sandbox kali.policy.json` | Same browser test-runtime gate as explicit `--api browser`; no silent fallback to `deno` |

### `kali init`
Initialize a new project scaffold.
```bash
kali init                                  # Create the minimal project scaffold in the current dir (kali.json + main.ts)
kali init --lib                            # Create the minimal library scaffold (kali.json + lib.ts)
```

Scaffold simplification rules:
- `kali init` is **current-directory-scoped** in schema v1: it scaffolds the current working directory and does not retarget itself to an ancestor project root discovered above it.
- if the current working directory already contains `kali.json`, `kali init` fails with `E5008` instead of overwriting the existing project config.
- if an ancestor directory contains `kali.json` but the current working directory does not, `kali init` may still create a nested child project rooted at the current working directory; later project discovery then treats that child as a separate project boundary.
- follow the shared **minimal canonical scaffold contract** from [SPEC.md](../SPEC.md): emit only the smallest valid schema-v1 scaffold for the selected template rather than extra example files, lockfiles, dependency state, or placeholder optional sections.
- reuse the shared **template selection vs build artifact mode split** from [SPEC.md](../SPEC.md): `kali init --lib` selects a project template only and does not imply later `kali build --lib`
- follow the **canonical scaffold filename convention** from [SPEC.md](../SPEC.md): `main.ts` for the default app template and `lib.ts` for the library template, unless a later template spec explicitly opts into a different filename.
- schema-v1 built-in scaffolds are intentionally tiny:

| Command | Files created by default | Files/directories intentionally not created by default |
|---|---|---|
| `kali init` | `kali.json`, `main.ts` | no `src/`, no `test/`, no `kali.lock`, no dependency state |
| `kali init --lib` | `kali.json`, `lib.ts` | no `src/`, no `test/`, no `kali.lock`, no dependency state |

- this scaffold contract is about exact minimal file presence first; starter-file contents may evolve, but they should stay minimal and valid for the selected template instead of growing extra boilerplate by default.
- both templates should keep the same canonical config vocabulary (`apiSurface`, `buildMode`, `runtimeProfiles`) instead of inventing template-specific aliases.

### `kali install [target]`
Install or materialize project dependencies.

Lifecycle scripts stay disabled by default. The one explicit opt-in is `--allow-scripts`, which permits npm lifecycle hooks for this install invocation only. Packages that fall into the shared **native/binary/bootstrap-heavy package contract** from [SPEC.md](../SPEC.md) remain unsupported even when scripts are enabled.

Boundary rule:
- follow the shared **workflow-owner split** from [SPEC.md](../SPEC.md): `kali install --allow-scripts` stays an install-time hook path only and does not become a second runtime/effect/policy workflow
- read package support through the shared **published-artifact-first package reading** from [SPEC.md](../SPEC.md): the presence of a repository build pipeline or optional lifecycle metadata does not by itself make a package unsupported if the published artifact Kali installs already contains the ordinary JS/TS files it needs
- `--allow-scripts` selects the schema-v1 **install-time npm-package hook path** from [SPEC.md](../SPEC.md), not a runtime/API-surface feature
- plain `kali install --allow-scripts` is valid only when the invocation has non-empty **effective npm-scriptable install work**; an explicit npm target such as `kali install --allow-scripts lodash` is the canonical valid shape, while a URL-only, JSR-only, clean already-synchronized, or otherwise no-npm-scriptable install graph should fail with `E5008` instead of silently degenerating into plain `install`
- that install work is **invocation-scoped**: it covers only the npm package work the current install actually reconciles in a lifecycle-hook-relevant way, including any directly requested npm target and any transitively touched npm dependencies in the same invocation
- a clean no-op install therefore keeps that install work empty even if the project already depends on npm packages; `--allow-scripts` does not ask Kali to re-run lifecycle hooks just because npm dependencies exist in the lockfile
- pairing `--allow-scripts` with an explicit raw URL install target is invalid command usage (`E5008`) because raw URLs do not expose npm lifecycle hooks
- pairing `--allow-scripts` with an explicit `jsr:` package target is also invalid command usage (`E5008`) in schema v1 because JSR packages do not participate in npm lifecycle-script execution
- mixed install graphs are still valid: if one invocation touches npm packages plus JSR packages and/or raw URLs, lifecycle scripts may run only for the npm install-work subset while the non-npm subset stays on the normal script-free path
- follow the same boundary from [SPEC.md](../SPEC.md): this path does **not** imply `--api node`, does not cause lifecycle scripts to participate in `kali effects`, does not make project `--sandbox` / `kali.json#sandbox` govern install-time hook execution, and does not make the excluded **native/binary/bootstrap-heavy package contract** supported
- package-compatibility claims for normal `check` / `build` / `run` / `test` remain separate from this narrower opt-in install behavior
```bash
kali install lodash                        # Add/install registry dependency from npm
kali install jsr:@std/path                 # Add/install registry dependency from JSR
kali install                               # Materialize all declared dependencies for the project
kali install --allow-scripts               # Permit lifecycle hooks only when this invocation actually has effective npm-scriptable install work; otherwise invalid usage (E5008)
kali install --dev vitest                  # Add/install dev dependency
kali install --allow-scripts lodash        # Opt into lifecycle scripts for one npm package install; invalid for explicit `jsr:` or raw-URL targets; still not a promise that the excluded native/binary/bootstrap-heavy package contract is supported
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument-kind rules:
- `kali install [target]` accepts at most one explicit install target in early phases; multiple install targets are invalid command usage (`E5008`)
- an explicit registry install target uses the shared **identity-only registry target** form from [SPEC.md](../SPEC.md): the user supplies one **registry package identifier** such as `lodash`, `@types/node`, or `jsr:@std/path`, not an inline version/range selector
- adding a registry package through that identity-only form follows the shared **stable-release selection rule (schema v1)** and **exact-version-first registry manifest rule (schema v1)** from [SPEC.md](../SPEC.md): resolve the latest non-yanked stable published version, write `kali.lock` with that concrete version, and record the manifest dependency as that same exact version string
- if that identity-only lookup finds the package but no acceptable non-yanked stable release, the command fails with `E5001` instead of silently selecting a prerelease or pretending the package was installable under the schema-v1 input form
- an explicit registry install target updates `dependencies` or `devDependencies` in `kali.json`, then refreshes `kali.lock` and materialized state
- in the canonical **configless install split** from [SPEC.md](../SPEC.md), an explicit registry-package add (`kali install <pkg>` or `kali install --dev <pkg>`) first creates the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then records the dependency there
- `kali install` does **not** take `--api` in early phases; install is profile-agnostic, so passing `--api ...` is invalid command usage (`E5008`) rather than a request for a second install graph
- `--dev` is valid only with a **registry install target**; using `--dev` without an explicit registry target or pairing it with a raw URL (`kali install --dev https://...`) is rejected explicitly rather than inventing a second URL-specific manifest bucket
- a **raw URL install target** pins/materializes that exact URL dependency in `kali.lock` and `.kali/cache/urls/`, but does **not** create a parallel manifest section or silently rewrite source/import-map entries
- in that same **configless install split**, an explicit raw-URL install may still create `kali.lock` and `.kali/cache/urls/` state at the effective project root, but it must not create a placeholder `kali.json` by itself
- explicit raw-URL installs follow the shared **raw-URL install staging/pin workflow** from [SPEC.md](../SPEC.md): they stage shared lock/cache state without creating a durable manifest entry, and a later plain `kali install` may prune that URL again if the project still does not reference it
- plain `kali install` consumes the current manifest/import graph and reconciles lock + materialized state for the dependency source kinds actually used by the project
- in that same **configless install split**, plain `kali install` is a no-op success when the effective project root contributes no manifest/import/source dependency inputs, and it must not create a placeholder `kali.json` just because the command ran
- because `kali install` normally has no explicit primary source input, source-level raw URL imports are discovered from the canonical project-discovery result (filtered by `include` / `exclude` when present, otherwise by the default project-discovery rules from [SPEC.md](../SPEC.md))
- this discovery step may be a cheap lexical/module-specifier scan rather than a full build, and it may scan declaration-only files too because they can participate in the project's type/import graph
- because raw URL entries are owned by the current source/import-map graph instead of a manifest dependency table, plain `kali install` may prune raw URL lock/cache entries that are no longer referenced
- `kali install` is intentionally **profile-agnostic** in early phases: it locks versions and materializes package contents once for the current manifest/import graph, but it does not pre-bake a separate install for each `--api` surface; later `check` / `effects` / `build` / `run` / `test` choose `deno`/browser-targeted package branches from the already-installed metadata at command time

Determinism rules:
- `kali install` is the command that updates dependency-owning manifest fields when needed, resolves versions, pins URL imports, and writes `kali.lock`.
- `kali check`, `effects`, `build`, `run`, and `test` consume existing project-managed dependency state; they must not silently modify dependency-owning parts of `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/` as a side effect. Missing URL-cache materialization is treated the same as missing `node_modules/`: fail with `E5004` and point the user to `kali install`.
- For `E5004`, "stale" means the current manifest/import graph, lockfile entries, and required materialized artifacts no longer match for the dependency kinds the project actually uses. It does **not** require ad hoc timestamp-based guessing by non-install commands.
- If dependency state is missing or stale for the dependency source kinds the project actually uses, those non-install commands fail with the canonical `E5004` path and point the user to `kali install`.
- If a file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) is pointed at explicit files outside the current **install-time declaration graph** from [SPEC.md](../SPEC.md) and those files reach additional raw URL imports, the command still fails with `E5004`; explicit targets bypass discovery filtering for command input selection, but they do not retroactively widen that install-owned declaration graph.
- the intended fix is to make those sources part of that **install-time declaration graph** (for example by widening `include` / `exclude` or adding the relevant source/import-map declaration) and then rerun `kali install`; non-install commands must not auto-install or mutate the dependency graph opportunistically.
- `--allow-scripts` is install-scoped only; it does not loosen later execution/build sandbox rules or request a second pass that re-runs already-settled lifecycle hooks on an otherwise clean install.
- lifecycle scripts enabled through `--allow-scripts` are outside the normal source-program sandbox/effect-report contract; they are install-time package hooks, not guest-program entrypoints.
- Registry packages (npm/JSR) are materialized into `node_modules/`; raw URL imports are materialized under `.kali/cache/urls/`. Non-install commands consume whichever of those stores are relevant to the current project instead of assuming every project must have both.

### Registry-analysis commands
These commands follow the shared **registry-analysis command split** from [SPEC.md](../SPEC.md) while sharing one early-phase target-selection contract:
- each takes exactly one explicit **registry package identifier** and rejects zero, multiple, raw-URL, or local-path targets with `E5008`
- the package argument uses the shared **identity-only registry target** form from [SPEC.md](../SPEC.md): `lodash`, `@types/node`, or `jsr:@std/path`, with no inline version/range selector
- version selection follows the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md); if the package identity exists but no acceptable non-yanked stable release exists, the command fails with `E5001`
- both commands follow the shared **registry-analysis project-independence rule** from [SPEC.md](../SPEC.md): current-project `kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/` do not change which package version is analyzed, and the commands do not mutate project-managed dependency state
- `package-effects` may still inherit semantic analysis context from discovered config/defaults once that command exists, but that inherited context affects analysis semantics only; it must not rewrite package identity/version selection or blur the project-independence rule
- turning an analyzed package into a project dependency remains the job of `kali install`

### `kali package-effects <package>`
Analyze effects of one registry package under the canonical schema-v1 registry-analysis rules.

Status: **Phase 2 target**. This section documents a **defined command family** in schema v1; before Phase 2, if package-level analysis is unavailable, the CLI should report that clearly instead of returning partial ad hoc output.
```bash
kali package-effects lodash                # Analyze npm package
kali package-effects jsr:@std/path         # Analyze JSR package
kali package-effects --pretty lodash       # Pretty-printed package-effect report JSON
kali package-effects --output json lodash  # Command envelope + package-effect payload
```

Base-gate clarification:
- follow the shared **registry-analysis availability boundary** from [SPEC.md](../SPEC.md): malformed invocations still fail first with `E5008`, while a well-formed base invocation such as `kali package-effects lodash` reaches the command's own availability gate (`E5006`) until Phase 2 opens
- once the base command exists, inherited-context gating follows the shared **axis-aligned inherited analysis gating** rule from [SPEC.md](../SPEC.md) rather than a package-analysis-specific shadow matrix
- practical simplification: schema-v1 `package-effects` takes only the package selector plus JSON-formatting flags (`--output json`, optionally `--pretty`); package-analysis-specific `--api` / `--compat` / `--wasm-threads` flags and `--sandbox` stay invalid usage instead of forming a second CLI vocabulary

Machine-output rule:
- this command is the `package-effects` half of the shared **registry-analysis command split**: once available, it is a schema-v1 **native-JSON command**
- by default it emits its package-effect payload directly, and with `--output json` it wraps that same payload in the standard command envelope
- `--pretty` changes formatting only; if combined with `--output json`, it formats the outer envelope while leaving the nested package-effect payload schema-identical
- see [specs/18-schemas.md](18-schemas.md) for the canonical package-effect payload schema

Analysis rule:
- `kali package-effects <pkg>` summarizes the statically reachable package graph selected for that package analysis under the active analysis context; it is not just a shallow inspection of the package's top-level manifest
- it inherits its semantic analysis context through the shared **effective inherited analysis context** from [SPEC.md](../SPEC.md) rather than taking package-analysis-specific `--api` / runtime-profile / `--compat` flags or `--sandbox` in schema v1; that inherited context changes analysis semantics only and does not alter which package/version was selected
- in configless mode, that inherited context is just the **default inherited analysis context (schema v1)** from [SPEC.md](../SPEC.md)
- if discovered config later makes that inherited context resolve to `apiSurface = browser`, plain `kali package-effects <pkg>` reuses the same browser-targeted analysis context once the command itself exists; this later inherited-context reuse does **not** widen the exact **Phase-1 browser-targeted command set**
- inherited-context availability follows the shared **axis-aligned inherited analysis gating** rule from [SPEC.md](../SPEC.md); if the inherited context is unavailable, the command fails with `E5006` rather than silently falling back to some smaller context
- canonical inherited examples once the command exists: `apiSurface = node` stays on the Node gate, `runtimeProfiles = ["wasm-threads"]` stays on the threaded-profile gate, and `compat.features = ["eval"]` stays on the `eval` compatibility gate rather than being silently dropped for package analysis
- the nested `report.entryPoints` field should name the package-analysis logical root using the same canonical registry identifier spelling the user targeted rather than an opaque tarball URL or cache path
- the nested `report.analysisContext` field records that inherited context explicitly so tools do not have to infer it from ambient project state

### `kali package-audit <package>`
Security audit for one registry package under the canonical schema-v1 registry-analysis rules.

Status: **Later compatibility**. This section also documents a **defined command family** in schema v1; it should not block Phase 1-2 compiler/runtime delivery, and if unimplemented the CLI should fail clearly rather than implying a partial security guarantee.
```bash
kali package-audit lodash                  # Audit specific npm package
kali package-audit jsr:@std/path           # Audit specific JSR package
kali package-audit --output json lodash    # Schema-v1 envelope-only JSON output
kali package-audit --pretty --output json lodash # Pretty-print that envelope; plain `--pretty` alone is invalid here because package-audit is not native JSON in schema v1
```

Base-gate clarification:
- follow the shared **registry-analysis availability boundary** from [SPEC.md](../SPEC.md): malformed invocations still fail first with `E5008`, while a well-formed base invocation such as `kali package-audit lodash` or `kali package-audit --output json lodash` reaches the command's own availability gate (`E5006`) until this later command exists
- output-format flags such as `--output json` or `--pretty` do not create a second availability path for the command itself
- practical simplification: schema-v1 `package-audit` takes only the package selector plus its envelope-format flags (`--output json`, optionally `--pretty`); package-analysis-specific `--api` / `--compat` / `--wasm-threads` flags and `--sandbox` stay invalid usage instead of growing a second context model

Audit rule:
- following the shared **workflow-owner split** from [SPEC.md](../SPEC.md), this command is the context-free registry-metadata/security-audit path rather than a second host-context-aware effect/policy command
- this command is the `package-audit` half of the shared **registry-analysis command split**: early `package-audit` follows **context-free registry analysis (schema v1)** and does not inherit the shared **effective inherited analysis context**
- early `package-audit` therefore does **not** take package-analysis-specific `--api` / runtime-profile / `--compat` flags or `--sandbox`
- in schema v1 it is an **envelope-only JSON command**, not a **native-JSON command**
- follow the schema-owned **Package Audit JSON Output (schema v1)** rule in [specs/18-schemas.md](18-schemas.md) for the exact envelope-only machine-output contract instead of restating it here
- practical shortcut: `package-audit` has one package selector and one optional envelope-format selector (`--output json`, optionally `--pretty`); it does not grow a second family of host-analysis or sandbox flags in schema v1
- because of that envelope-only model, `kali package-audit --pretty <pkg>` without `--output json` is invalid command usage (`E5008`) rather than an implicit request for JSON mode
- output-format flags do **not** create a separate availability path or context model for `package-audit`

## Output Design

### Success Output (Default)
Minimal — one line or nothing. For commands intended for automation, prefer no success output when there is no artifact or program stdout to report; otherwise print a single deterministic line. Human-friendly decoration belongs behind `--verbose`, not in the default contract:
```
$ kali check main.ts

$ kali build main.ts
main.wasm 142KB 23ms

$ kali run main.ts
Hello, world!
```

### Error Output (Default)
Structured, parseable, concise (see [specs/15-errors.md](15-errors.md)):
```
$ kali check main.ts
error[E1001]: Type 'string' is not assignable to type 'number'
  --> main.ts:5:10
  |
5 |   let x: number = "hello";
  |          ------   ^^^^^^^ expected 'number', found 'string'
  |          expected type

Found 1 error.
```

### Verbose Mode (`--verbose`)
Adds: timing per phase, IR dumps, optimization decisions, memory layout choices.

### JSON Output (`--output json`)
Machine-parseable output for commands that normally print human-oriented text. The canonical command-envelope schema lives in [specs/18-schemas.md](18-schemas.md).

Schema-v1 JSON-mode quick matrix:

| Command family | Default success mode | `--output json` behavior | Plain `--pretty` without `--output json` |
|---|---|---|---|
| `effects`, `package-effects` | native JSON payload | wrap that payload in the standard command envelope | valid once the command exists, because success output is already JSON |
| `package-audit` | non-JSON text/human mode | emit the standard command envelope only (**envelope-only JSON**) | invalid usage (`E5008`) |
| all other commands with JSON support | non-JSON text/human mode | emit the standard command envelope | invalid usage (`E5008`) |

Quiet-mode interaction rule:
- `--quiet` suppresses extra success/status text, not the command's primary payload
- for ordinary human-oriented commands, that usually means nothing is printed on success unless the command's main purpose is to emit stdout from the user program or a requested machine payload
- for schema v1 **native-JSON commands** (`kali effects`, `kali package-effects`) and for any other **JSON-producing mode**, the requested JSON payload/envelope remains the primary output even under `--quiet`

Pretty-print interaction rule:
- `--pretty` is meaningful only in **JSON-producing mode**
- `--pretty` does **not** opt a command into JSON mode by itself
- for schema v1 **native-JSON commands**, plain success output is already JSON, so `--pretty` reformats that native payload
- for any command with `--output json`, including **envelope-only JSON commands** such as early `package-audit --output json`, `--pretty` reformats the outer command envelope
- if a command is not otherwise emitting JSON (for example `kali check --pretty` without `--output json`, or early `kali package-audit --pretty lodash` without `--output json`), `--pretty` is invalid command usage (`E5008`) rather than a silent no-op
- `--pretty` changes formatting only; it must not change field names, ordering guarantees, or whether stderr/human diagnostics are emitted outside JSON mode
- JSON-selection flags do **not** bypass command maturity or create a second command surface: if `kali effects`, `kali package-effects`, or `kali package-audit` is still unavailable in the current phase, invocations such as `--pretty` / `--output json` still fail on the command's normal availability gate after any earlier command-shape checks

Quick JSON-mode shorthand:
- `kali effects --pretty main.ts` and `kali package-effects --pretty lodash` are valid once those **native-JSON commands** exist, because their default success output is already JSON
- `kali package-audit --pretty --output json lodash` is the valid envelope-only audit form in schema v1
- `kali package-audit --pretty lodash` stays invalid (`E5008`) because `package-audit` does not become JSON-producing until `--output json` is present

Feature gating is part of the machine contract too: availability-gate rejections should serialize the same stable diagnostic code and note structure as human output. When the failure depends on merged CLI/config state (for example a config-selected API surface or a contradictory artifact-mode combination), JSON diagnostics should also populate the optional structured `context` metadata from [specs/18-schemas.md](18-schemas.md) so tools can see the effective value without scraping prose.

Rules:
- top-level output uses the versioned command envelope
- diagnostics reuse the shared diagnostic schema
- command-specific structured data goes in `payload` when that command has a dedicated success-payload schema in schema v1
- **envelope-only JSON commands** may still support `--output json` through the standard envelope alone; in that case `payload` should be omitted or `null` rather than filled with ad hoc prose/fields
- the **envelope-only JSON command** model also does **not** permit commands to smuggle structured success metadata through `stdout` / `stderr`; those fields remain reserved for captured text streams
- common optional top-level fields include `artifacts`, `stdout`, `stderr`, `timings`, and `exitCode`
- for execution-style commands in JSON mode, guest/program stdout and stderr are captured into the envelope fields instead of being interleaved as raw terminal text
- build-like commands should populate artifact `role` whenever it helps distinguish artifact mode without forcing tools to guess from filenames (for example default executable vs `--lib` `wasm-module`)

Exception: schema v1's **native-JSON commands** (`kali effects`, `kali package-effects`) already emit JSON as their native outputs, so `--output json` wraps those payloads in the envelope instead of changing their underlying schemas.

Native-JSON command stream rule:
- in default native-payload mode, stdout is reserved for the success payload only
- extra progress/status text must not be interleaved into stdout for those commands
- when those commands fail **without** `--output json`, they should emit the normal human-oriented diagnostics to stderr rather than corrupting stdout with mixed text/JSON output
- when callers need machine-readable failure results for those commands too, `--output json` is the canonical request path; both success and failure then use the standard command envelope
- `--pretty` changes formatting of the success payload or outer envelope only; it does not turn stderr diagnostics into JSON

This is an intentional simplification: Kali keeps one canonical effect-report payload family, and the command envelope is an outer transport wrapper rather than a second competing effect schema for every effect-analysis command.

## Configuration (`kali.json`)

The canonical full config schema and example live in [specs/18-schemas.md](18-schemas.md). This chapter only repeats the naming rules so CLI and schema docs do not drift.

Minimal canonical shape:
```json
{
  "schemaVersion": 1
}
```

Optional metadata field:
- `$schema` may be included for editor/schema tooling, but `kali init` should omit it by default unless the user/template explicitly asks for it

Omission/default rule for minimal configs:
- `kali init` should follow the shared **minimal canonical scaffold contract** from [SPEC.md](../SPEC.md) instead of growing command-local placeholder config or dependency state.
- Omitted fields inherit documented schema/CLI defaults rather than creating placeholder sections.
- In schema v1, omitted `compilerOptions` means all compiler-option defaults apply.
- In schema v1, omitted `compilerOptions.strict` means the default strict-checking bundle is enabled; its canonical semantics are defined in [specs/04-type-system.md](04-type-system.md).
- In schema v1, omitted `compilerOptions.maxSpecializations` means the project uses the default specialization cap of `16`.
- Omitted `compat` means `compat.features = []`.

Configuration simplification rules:
- `compilerOptions.apiSurface` is the config equivalent of the CLI `--api` flag
- `compilerOptions.apiSurface` influences command-time API/package selection for `check` / `effects` / `build` / `run` / `test`, and for inherited-context package analysis via `package-effects`, but it does **not** cause `kali install` to maintain separate lock/materialization state per API surface in early phases and it does **not** change the semantics of early `package-audit`
- `compilerOptions.buildMode` replaces separate optimization booleans
- `compilerOptions.runtimeProfiles` is an array of explicit semantic runtime-profile switches; an empty array means the default single-threaded baseline, while a future threaded config would use `"runtimeProfiles": ["wasm-threads"]`
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- `compilerOptions.apiSurface` and `compilerOptions.runtimeProfiles` describe different axes and must not be conflated: `deno`/`node`/`browser` select host APIs, while runtime profiles select execution capabilities such as threads
- `compilerOptions.strict` is the config-level **strictness bundle**; its semantics live in [SPEC.md](../SPEC.md) and [specs/04-type-system.md](04-type-system.md), and it should not be re-expanded into many parallel booleans in early phases
- `compilerOptions.maxSpecializations` caps specialization fan-out for generic/layout-driven optimization in modes that actively specialize; CLI `--max-specializations` overrides it for a single invocation
- `compilerOptions.maxSpecializations` is an upper bound rather than a promise that `buildMode = fast` will consume that full budget; `fast` may still skip most user-authored generic specialization by design
- top-level `sandbox` is an optional default policy-file path equivalent to supplying `--sandbox <path>` for the canonical sandbox-aware commands from [SPEC.md](../SPEC.md); an explicit CLI `--sandbox` overrides it
- relative `sandbox` paths in `kali.json` are resolved relative to the directory containing that config file rather than relative to whatever directory the user happened to run the command from
- omitting top-level `sandbox` means no default policy is attached; it does **not** ask tools to synthesize an implicit permissive policy file
- the canonical effect-reporting and sandbox-agnostic command classes from [SPEC.md](../SPEC.md) ignore the top-level `sandbox` setting rather than erroring or silently turning themselves into policy-validation commands
- `compat.features` is the config equivalent of CLI `--compat`; it uses the same canonical feature names, is order-insensitive, and should not duplicate them in alternate booleans
- in schema v1, the only canonical compatibility feature name is `"eval"`; it gates both direct `eval` support and the `Function()` constructor compatibility path
- `include` / `exclude` constrain the canonical project-discovery result for **discovery-driven commands** from [SPEC.md](../SPEC.md); direct file arguments still name the primary entry explicitly and are not silently filtered back out just because they sit outside the discovery globs
- unless overridden, **discovery-driven command** behavior still skips the default managed/generated directories named in [SPEC.md](../SPEC.md)
- `include` / `exclude` filter only the project's own discoverable files; they do not suppress transitive imports/dependencies reached from an accepted entrypoint and they are not a second package-resolution mechanism
- generated config from `kali init` should prefer these canonical names and should not duplicate them as parallel top-level keys
- `kali init` should not emit `sandbox`, `compat`, `dependencies`, or other optional sections unless the chosen template or user request actually needs them
- when schema-v1 registry dependencies are present in `dependencies` / `devDependencies`, their values are exact resolved version strings; broad SemVer ranges are invalid config (`E5009`) rather than a second supported manifest mode
- because absence of `sandbox` means “no policy attached” rather than “allow all by explicit policy”, tools should preserve omission when round-tripping minimal configs unless the user intentionally chooses a default policy path
- precedence is `CLI > kali.json > defaults`, except sandbox-policy restrictions still constrain effective runtime behavior

## Exit Codes

Interpretation rule:
- ordinary compile/check/build diagnostics over otherwise valid command inputs exit with **1**; this includes syntax/type/name errors, import/module/resolution failures, dependency-state failures (`E5001`-`E5006` as applicable), library-export proof failures (`E5011`), and compile-time sandbox/effect violations once the Phase 2 target opens
- this same `1` path also covers a **well-formed but context-incompatible** attached policy whose enabled capability/profile is unavailable for the effective command context (for example `effects.eval: true` before `--compat eval` exists, or browser-targeted `check` / `build --bundle` policies that violate the canonical browser-targeted budget rule by setting `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, or `resources.maxOpenFiles`, or by setting positive `resources.maxSpawnedProcesses` / `resources.maxThreads` values)
- `fmt --check` and lint-style contract failures that report ordinary command diagnostics also exit with **1**
- runtime sandbox enforcement failures exit with **3**
- runtime resource exhaustion/fuel/memory-limit failures exit with **4**
- invalid CLI arguments, invalid config (`E5009`), invalid policy schema/ranges (`E5010`), and command-input/entrypoint-usage mistakes exit with **5**
- malformed/invalid policy files stay on the `5` path; only semantically valid policy files that hit documented feature/profile gating move onto the ordinary diagnostic `1` path

Command-input/entrypoint-usage mistakes include:
- missing required direct-input arguments for `run`, `build`, or `effects`
- too many explicit direct-input arguments for those same commands in early phases
- conflicting artifact-mode selectors for `build` (for example `--bundle --lib` or `--lib --capi`)
- `E5007` invalid-entrypoint/input-kind cases such as passing a declaration-only file to `run`, `build`, `effects`, or `test`

This keeps exit codes simple: command-time failures are separated from runtime enforcement failures.

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Ordinary compile/check/build diagnostic failure (for example syntax, type, name, module/resolution, dependency-state, feature-gating, or library-export-proof failures such as `E5001`-`E5006` and `E5011`, plus build-time sandbox/effect violations, semantically valid but context-incompatible policy enablement such as browser-targeted budget incompatibilities, `fmt --check`, and lint contract failures) |
| 2 | Runtime error (uncaught exception) |
| 3 | Runtime sandbox violation |
| 4 | Runtime resource limit exceeded |
| 5 | Configuration (`E5009`) / malformed or schema-invalid policy file (`E5010`) / CLI usage (`E5008`) / invalid command input or entrypoint (`E5007`) |
| 126 | Permission denied |
| 127 | File not found |
