# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases
6. **Single-channel machine output**: when `--output json` is selected, command metadata and any captured program streams are emitted through the JSON envelope rather than interleaving raw stdout/stderr text that would corrupt the payload

## Shared Flags

These flags are shared across the CLI, but some apply only to specific command families. For the canonical meaning of **API surface**, **build mode**, and **runtime profile**, see [SPEC.md](../SPEC.md). For command/profile gating, see [19 — Feature Maturity](19-feature-maturity.md).

Ownership rule:
- this chapter owns **CLI shape**: flags, arity, command-local behavior, output rules, and exit codes
- [19 — Feature Maturity](19-feature-maturity.md) owns whether a documented command/profile/feature is actually available in a given phase
- [18 — Schemas](18-schemas.md) owns the machine-readable JSON shapes
- when a rule is already defined in one of those owners, prefer a short cross-reference over repeating a second full version here

Command-family terminology used in this chapter:
- these labels describe **command shape and behavior**, not guaranteed current-phase availability
- commands such as `effects`, `package-effects`, or `package-audit` may still be phase-gated even though their command family is defined here
- canonical availability promises live in [19 — Feature Maturity](19-feature-maturity.md)
- **execution commands**: `run` and `test`
- **build-like commands**: `build`, plus the compile step embedded inside `run` and `test`
- **diagnostic-producing commands**: `check`, `effects`, `package-effects`, `build`, `run`, `test`, `fmt --check`, and `lint`
- **JSON-producing mode**: a command invocation that emits JSON as its primary success output, either because the command is one of schema v1's native-JSON reporting commands (`effects`, `package-effects`) or because `--output json` selected the standard command envelope

Canonical command-input mode rule (shared with [SPEC.md](../SPEC.md)):
- `run`, `build`, and `effects` are **direct-input commands** in early phases: they require exactly one explicit primary source input and do not guess `main.ts` or invent a project-default file
- for `run`, that source input is an executable/analyzable entrypoint
- for `build`, that source input is one explicit primary module input whose artifact role depends on the selected artifact mode
- for `effects`, that source input is one explicit analysis root
- `check` is a **hybrid analysis command**: it accepts explicit file arguments, or falls back to the canonical project-discovery result when no files are provided
- `fmt`, `lint`, and `test` are **project-oriented commands** when invoked without explicit file arguments
- `install` is the canonical **dependency-graph command**: with no explicit install target it reconciles the discovered project dependency graph, including raw URL imports found through project discovery
- `package-effects` and `package-audit`, when available, are the canonical **registry-analysis commands**: each takes one explicit registry package identifier and does not invent a no-argument whole-project analysis mode in early phases
- `init` is not a direct-input source command

Canonical early-phase direct-input arity rule:
- `run`, `build`, and `effects` each take **exactly one** explicit primary source input in early phases
- zero explicit source inputs for those commands is the canonical invalid-usage diagnostic `E5008`
- more than one explicit source input for those commands is also `E5008` unless a later spec introduces a documented multi-input mode
- `check`, `fmt`, `lint`, and `test` are the canonical **set-oriented explicit-file commands** from [SPEC.md](../SPEC.md): when explicit files are supplied, those paths are treated as one file set rather than as separate single-entry invocations

Canonical install-target and package-argument arity rule:
- `kali install [target]` accepts **zero or one** explicit install target in early phases
- that install target may be either a schema-v1 identity-only registry target or a raw URL target
- `kali package-effects <package>` accepts **exactly one** explicit registry-package argument
- `kali package-audit <package>` accepts **exactly one** explicit registry-package argument
- passing more than the allowed number of explicit install targets/package arguments is `E5008` rather than permission to invent an undocumented batch mode
- omitting the required explicit registry-package argument for a registry-analysis command is also `E5008`
- flags that conceptually modify an explicit registry-package target (for example `kali install --dev`) require that registry target in early phases; using them without one is also `E5008`

Canonical input-kind rule:
- `run`, `build`, `effects`, and discovered `test` entrypoints/primary inputs accept only the shared executable/analyzable source-file set (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`)
- `check`, `fmt`, and `lint` accept that same executable/analyzable set **plus** declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`)
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
- non-participating axes are ignored rather than gated: for example `check` ignores inherited `buildMode`, and early `package-audit` ignores inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox`
- examples: config-selected `apiSurface = node` still causes plain `kali run main.ts` or `kali test` to hit the Node phase gate (`E5006`), and config-selected `apiSurface = browser` still makes plain `kali build main.ts` invalid early-phase usage (`E5008`) until `--bundle` is selected
- config-selected `apiSurface = browser` also keeps plain `kali run main.ts` and plain `kali test` on the same browser-runtime/test gate as their explicit `--api browser` forms (`E5006`); omitting the flag does not cause a silent fallback to `deno`
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): command-shape/arity first, then base command availability, then finer inherited-context/profile gates inside that command

| Flag | Scope | Description |
|------|-------|-------------|
| `--verbose` | all commands | Detailed output: timing per phase, optimization decisions |
| `--output json` | all commands | Machine-parseable JSON output |
| `--pretty` | JSON-producing mode | Pretty-print the active JSON document without changing its schema; meaningful only for native-JSON reporting commands or when `--output json` is active |
| `--quiet` | all commands | Suppress non-error status/progress output; for data-producing commands such as `effects` and `package-effects`, it must not suppress the primary payload itself |
| `--max-errors N` | diagnostic-producing commands | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | text-output commands | Color output control |
| `--api deno\|node\|browser` | `check`, `effects`, `build`, `run`, `test` | Select host API surface; unsupported surfaces for the current command/profile must error explicitly (for example, early browser builds require `--bundle`) |
| `--compat <feature[,feature...]>` | `check`, `effects`, `build`, `run`, `test` | Enable documented compatibility features such as `eval` only when that feature is implemented for the selected phase/profile; in schema v1, `eval` also covers the `Function()` constructor path |
| `--fast` | `build`, `run`, `test` | Fastest compile time, minimal optimization (default build mode) |
| `--release` | `build`, `run`, `test` | Standard optimization profile |
| `--release-advanced` | `build`, `run`, `test` | Aggressive optimization profile |
| `--sandbox <policy>` | sandbox-aware commands | Attach and validate `kali.policy.json`; in Phase 1 this enforces at runtime for `run`/`test` and validates policy/config for `check`/`build` |
| `--max-memory <size>` | execution commands | Override the invocation memory cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-cpu <duration>` | execution commands | Override the invocation CPU cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-open-files N` | execution commands | Override the invocation open-file-handle cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-spawned-processes N` | execution commands | Override the invocation child-process cap; may only tighten the effective limit. `0` is always a valid explicit deny/tightening value, while non-zero values are rejected until the selected command/profile/API surface actually supports subprocesses |
| `--max-threads N` | execution commands | Override the invocation thread cap for the threaded runtime profile; may only tighten the effective limit. `0` is always a valid explicit deny/tightening value, while non-zero values are rejected unless threading is supported and enabled |
| `--wasm-threads` | `check`, `effects`, `build`, `run`, `test` | Opt into the later threaded runtime profile required for `SharedArrayBuffer` / `Atomics`; before that profile exists, or on unsupported targets, the command must fail with `E5006` |

`--fast`, `--release`, and `--release-advanced` are mutually exclusive; config files should use the single `compilerOptions.buildMode` field instead of parallel booleans. `run` and `test` inherit the selected build mode for their internal compile step. Runtime-profile toggles such as `--wasm-threads` map to entries in `compilerOptions.runtimeProfiles` rather than to separate booleans.

Package-analysis flag/context simplification:
- follow the canonical command-context axis participation table, `analysis context` term, and **registry-analysis context split** in [SPEC.md](../SPEC.md)
- `kali package-effects` is a **Phase 2 target** and, once available, inherits only the semantic analysis axes (`apiSurface`, `runtimeProfiles`, `compat.features`) from config/defaults; it records that context in `report.analysisContext` using the emitted field names `apiSurface`, `runtimeProfiles`, and `compatFeatures` instead of growing package-analysis-specific `--api` / runtime-profile / `--compat` flags
- `buildMode` and `sandbox` remain non-semantic for `package-effects` in early phases
- `kali package-audit` is a **Later compatibility** single-package registry tool and, once available, stays context-free in early phases; inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and `sandbox` do not change its semantics
- examples later in this chapter describe the canonical command shape/output contract for these registry-analysis commands, not an unconditional promise that they are already available in Phase 1

Sandbox-flag clarification:
- the CLI `--sandbox <policy>` flag is reserved for the canonical sandbox-aware commands: `run`, `test`, `check`, and `build`
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
| `--lib` | `build`, `init` | For `build`: select the base library/export artifact mode (no synthetic executable entry invocation; ordinary top-level module initialization still occurs when instantiated). For `init`: scaffold a library-oriented project template only |
| `--capi` | `build` | Emit the Phase-2 public C-embedding artifact set (`wasm-module` + `wit` + `c-header` + `cabi-metadata`) |
| `--component` | `build` | Emit a WebAssembly Component Model wrapper for a library/export-oriented build once that packaging path exists; phase-gated until the component flow is implemented |
| `--validate-ir` | `build` | Run internal IR validators as a debugging/developer aid |
| `--max-specializations N` | `build`, `run`, `test` | Override the specialization fan-out cap upper bound for a single invocation; this is an upper bound, not a promise that the current build mode will spend the full budget, and `--fast` may still skip most user-authored generic specialization entirely |
| `--fix` | `check`, `lint` | Apply only structured, tool-generated safe fixes for the selected command |
| `--check` | `fmt` | Report formatting drift without rewriting files |
| `--filter <pattern>` | `test` | Run only matching tests |
| `--coverage` | `test` | Emit test coverage data once the coverage report contract is stabilized; before then this flag is phase-gated or explicitly experimental |
| `--dev` | `install` | Add the named registry dependency to `devDependencies` instead of `dependencies` |
| `--allow-scripts` | `install` | Opt into npm lifecycle scripts for that install invocation only; meaningful only when the invocation has non-empty **effective npm-scriptable install work** from [SPEC.md](../SPEC.md), and still rejects native addons, `node-gyp`, and install-time binary/bootstrap package contracts |

Interpretation rule:
- command-specific flags inherit the same phase/profile gating rules as the command they belong to
- documenting a command-specific flag here does **not** imply it needs a separate feature-maturity row unless it changes a phase promise or machine-readable contract
- build artifact-mode flags follow the canonical matrix in [SPEC.md](../SPEC.md): in early phases `--bundle`, `--lib`, `--capi`, and `--component` are one small closed set of mutually exclusive selectors unless a later spec explicitly says one implies another
- the omitted selector means the default executable artifact mode; supplying more than one explicit selector from that set (for example `--bundle --lib`, `--bundle --capi`, `--bundle --component`, `--lib --capi`, `--lib --component`, or `--capi --component`) should use the canonical invalid-usage diagnostic `E5008`, not a feature-maturity rejection
- in Phase 1, `--bundle` is the browser packaging selector only: `kali build --bundle ...` requires the **effective API surface** to be `browser`, and `kali build --bundle` under an effective API surface of `deno` or `node` is invalid command usage (`E5008`) rather than a feature-maturity rejection, because the browser bundle mode itself exists but the selected flag/config combination is contradictory
- in early phases, `--lib`, `--capi`, and `--component` are **library-oriented artifact modes**: non-browser, export-oriented build modes derived from a **statically known export surface** as defined in [SPEC.md](../SPEC.md)
- those library-oriented modes still obey the ordinary build-command API-surface gates: `kali build --lib --api browser ...`, `kali build --capi --api browser ...`, and `kali build --component --api browser ...` are `E5008` contradictions because browser mode is only defined for `--bundle`, while `kali build --lib --api node ...` remains on the same Phase 3 `E5006` path as other early `--api node` builds
- `--lib` is the base exported-library mode; `--capi` and `--component` are later packaging layers over that same exported-library contract rather than unrelated semantics
- because `--capi` and `--component` already choose exported-library semantics, users should not combine them with `--lib` in early phases; those flags are separate artifact-mode selectors, not additive modifiers
- WIT sidecars are not a separate artifact-mode selector: Phase 1 plain `--lib` emits the core library `wasm-module`, and once the public library/embedding surface stabilizes in Phase 2+, the relevant library-oriented modes emit WIT by default so callers do not have to choose between "C ABI" and "component metadata" paths

Config-array normalization rule:
- `compilerOptions.runtimeProfiles` and `compat.features` are set-like lists, not ordered pipelines
- entries should be unique; duplicates are config errors (`E5009`), not something tools silently deduplicate away
- unknown entries are diagnosed instead of ignored
- when those sets are re-emitted in machine-readable payloads such as `analysisContext`, producers should use stable lexical order so caches and diffs do not depend on original config ordering

Configuration precedence is intentionally simple:
1. CLI flags override the effective discovered `kali.json`
2. the effective discovered `kali.json` overrides built-in defaults
3. Sandbox policy caps, when a policy is attached, remain upper bounds for runtime capabilities and resource limits

That means command-line resource flags can tighten a run relative to policy/config, but they must not silently widen a sandbox policy. If no policy is attached, those direct invocation flags simply become the effective cap for the current command instead of being compared against an implicit allow-all policy. In Phase 1 this tightening path applies directly to `--max-memory`, `--max-cpu`, and `--max-open-files`. For later-gated caps such as `--max-spawned-processes` and `--max-threads`, the same tightening rule applies once the underlying capability exists; before then, `0` remains a valid explicit deny/tightening value while non-zero values stay phase/profile-gated.

Interpretation rule:
- the resulting merged values are the command's one **effective context** for validation, lowering, and reporting
- unsupported inherited config values do not get ignored just because the user omitted the matching CLI flag

Canonical path-resolution rule:
- ordinary CLI path arguments (entry files, explicit file lists, and `--sandbox <path>`) are resolved relative to the current working directory
- top-level `kali.json#sandbox` and other config-owned relative paths/globs are resolved relative to the directory containing that `kali.json`
- after resolution, commands should preserve one normalized absolute/canonical path internally so diagnostics and caching do not depend on the caller's original spelling

Canonical resource-literal rule:
- `--max-memory` accepts either a plain byte count or a size literal with one of: `kb`, `mb`, `gb`, `kib`, `mib`, `gib`
- `--max-cpu` accepts either a plain millisecond count or a duration literal with one of: `ms`, `s`, `m`
- `--max-open-files` accepts a plain non-negative integer count and mirrors `resources.maxOpenFiles`: it caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- `--max-spawned-processes` accepts a plain non-negative integer count
- `--max-threads` accepts a plain non-negative integer count
- CLI parsing normalizes these to bytes, milliseconds, and integer counts before comparing them with sandbox-policy limits
- follow the canonical numeric-limit semantics from [SPEC.md](../SPEC.md): `--max-memory`, `--max-cpu`, and `--max-open-files` must be **positive** when present, so `0` is invalid rather than a hidden deny form
- only `--max-spawned-processes` and `--max-threads` may use `0` as an explicit deny/tightening value, because zero concurrent uses is meaningful for those counters
- schema v1 policy files keep the simpler integer fields `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`, `resources.maxSpawnedProcesses`, and `resources.maxThreads`; CLI literals/counts are a convenience syntax over that same effective-limit model rather than a second resource schema

Canonical default tuple:
- `apiSurface = deno`
- `buildMode = fast`
- `runtimeProfiles = []`
- `compat.features = []`

This is the default interpretation of examples such as `kali run main.ts`, `kali test`, and `kali build main.ts` unless the example explicitly overrides a field. `kali check main.ts` and `kali effects main.ts` use the same default API surface selection.

## Commands

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
kali run --api browser main.ts             # Rejected in early standalone phases; browser is a browser-targeted context first
kali run --wasm-threads main.ts            # Enable WASM threads (SharedArrayBuffer, Atomics; opt-in only)
```

`kali run` is a direct-input command in early phases: it requires exactly one explicit executable/analyzable source entrypoint and does not guess a project default such as `main.ts`.

Initial implementations use wasmtime; alternative runtime backends are a later-phase feature. Feature flags and subcommands that depend on later phases should be hidden or clearly diagnosed when unavailable rather than exposed as silently nonfunctional options.

When a command or flag is rejected due to phase/profile maturity, the CLI should use the canonical feature-maturity diagnostic shape from [specs/15-errors.md](15-errors.md) rather than ad hoc wording.

Canonical interpretation rules:
- `--api` selects an **API surface**, but support is command-dependent.
- follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): supported early browser shapes are `check --api browser` and `build --bundle --api browser`; wrong browser build shapes use `E5008`, while browser execution/test requests use `E5006` until Kali defines a standalone browser runtime/test contract.
- `--api node` is phase-gated consistently across `check`, `effects`, `build`, `run`, and `test`; early phases reject it with `E5006` rather than exposing a partial Node surface.
- `--compat ...` is the one shared switch for later-phase dynamic compatibility features. If the named feature is not implemented yet, the command still fails with `E5006`.
- in schema v1, `--compat eval` is the only stable compatibility-feature spelling and it gates both direct `eval` and `Function()`; the CLI should not invent a separate `--compat function-constructor` alias.
- sandbox permission and compatibility enablement are separate axes: a policy that allows `effects.eval` does **not** implicitly turn on `--compat eval`, and `--compat eval` does **not** bypass a stricter sandbox policy.
- `--wasm-threads` selects a different runtime profile rather than a small optimization toggle. Until that threaded profile exists, the flag is rejected. After it exists, if the selected target/engine/profile cannot honor it, the command must still reject it explicitly instead of silently dropping thread support.
- `--max-spawned-processes N` is meaningful only when the selected command/profile/API surface exposes subprocess support. A non-zero process cap without effective subprocess support must be rejected explicitly rather than ignored.
- `--max-threads N` is meaningful only together with the threaded runtime profile. A non-zero thread cap without effective thread support must be rejected explicitly rather than ignored.

Sandbox flag behavior is intentionally phase-gated:
- `kali run --sandbox ...` is a Phase 1 feature for runtime policy enforcement.
- `kali check/build --sandbox ...` validate the policy file/config in Phase 1.
- Full inferred-effect-vs-policy validation is a Phase 2 feature.
- Policy validation must also reject policies that try to enable capabilities unavailable in the selected command/profile/phase (for example `effects.eval: true` before the eval compatibility path exists, `effects.eval: true` without effective `--compat eval`, `resources.maxSpawnedProcesses > 0` before subprocess support exists, or `resources.maxThreads > 0` before the threaded runtime profile exists).
- For browser-targeted `check --api browser --sandbox ...` and `build --bundle --api browser --sandbox ...`, follow the **canonical browser-targeted policy boundary** and the **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md): browser-targeted sandboxing is a static compatibility check over the documented mediated subset, and schema-v1 `resources.*` fields are treated as Kali-hosted execution budgets rather than as post-deployment browser guarantees.
- Policy files remain declarative; any later host-registered sandbox policy predicates are an embedding-oriented extension, not a second inline policy language.
- If neither CLI nor config attaches a policy, the command runs with **no project policy file**; direct resource flags such as `--max-memory` and later supported caps such as `--max-spawned-processes` still apply, but there is no hidden synthesized policy document behind the scenes.

### `kali build <file>`
AOT compile to a WASM module or linked artifact set.

Canonical artifact-mode rule:
- `kali build` is a direct-input command in early phases: it requires exactly one explicit executable/analyzable primary source input and does not guess a project default such as `main.ts`
- in executable artifact mode that source input behaves as the program entrypoint; in library-oriented artifact modes it is the primary module input whose exports define the host-facing surface
- artifact selection follows the canonical matrix in [SPEC.md](../SPEC.md)
- omitting `--bundle`, `--lib`, `--capi`, and `--component` selects the default executable artifact mode
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive artifact-mode selectors unless a later spec explicitly defines one as an implication of another
- `kali init --lib` chooses a project template only; it does not change the later default artifact mode of `kali build`
- WIT sidecars for public library/embedding outputs are an output detail of those artifact modes, not a separate mode flag
- these **library-oriented artifact modes** derive their host-facing surface from a **statically known export surface** as defined in [SPEC.md](../SPEC.md); they do not implicitly expose arbitrary internal declarations just because the source file was compiled in `--lib`/`--capi`/`--component` mode
- if Kali cannot prove that export surface, the library-oriented build fails with `E5011` instead of synthesizing reflection-based exports
- plain `--lib` is the Phase-1 **base library** artifact: it establishes the exported-library shape early, but the stable public embedding/WIT contract remains Phase 2 work
- they also keep the ordinary build-command API-surface semantics: Node-targeted library builds are still phase-gated with `E5006`, while browser-targeted library/embedding combinations are invalid command shapes (`E5008`) until a separate browser-library contract exists

`--capi` and other public embedding-oriented outputs follow the embedding maturity rules in [specs/19-feature-maturity.md](19-feature-maturity.md): the compiler is library-first internally in Phase 1, but stable public embedding artifacts are a Phase 2 target.

Sandbox clarification:
- `kali build --sandbox ...` never executes the program; in Phase 1 it validates policy/config, and in Phase 2+ it also performs effect-vs-policy validation.
- `kali build --bundle --api browser --sandbox ...` follows the **canonical browser-targeted policy boundary** from [SPEC.md](../SPEC.md): it is a build-time compatibility check over the documented mediated subset, not automatic runtime sandbox enforcement once the emitted browser bundle is deployed into a real browser host.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default; artifact: kind=wasm-module, role=primary-executable)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # main.wasm + main.js (artifacts: main.wasm kind=wasm-module role=primary-executable; main.js kind=js-glue role=browser-glue)
kali build --bundle main.ts                # Invalid usage (E5008) under the default config; --bundle requires the effective API surface to be browser
kali build --bundle --api node main.ts     # Invalid usage (E5008); --bundle is the browser-only artifact mode, so pairing it with a non-browser API surface is contradictory
kali build --api browser main.ts           # Invalid usage (E5008) in early phases; browser build path requires --bundle
kali build --api node main.ts              # Phase 3 target: Node API surface is not available early on build/check either
kali build --lib lib.ts                    # Export-oriented base library module (no synthetic executable entry invocation; ordinary top-level module initialization still occurs on instantiation; Phase 1 artifact: kind=wasm-module, role=primary-library; stable public embedding/WIT contract lands in Phase 2+, which then adds kind=wit, role=interface-wit by default)
kali build --lib --api node lib.ts         # Phase 3 target: Node API surface remains build-gated for library-oriented modes too
kali build --lib --api browser lib.ts      # Invalid usage (E5008) in early phases; browser mode is a browser-targeted context tied to `check` and `build --bundle`, not a library artifact mode
kali build --capi lib.ts                   # Phase 2 target: lib.wasm + lib.wit + lib.exports.h + metadata (artifacts: wasm-module + wit + c-header + cabi-metadata; roles: primary-library + interface-wit + embedding-header + embedding-metadata; see specs/13-embedding.md)
kali build --capi --api node lib.ts        # Phase 3 target: still gated by the Node build surface even after public embedding artifacts exist
kali build --component lib.ts              # Phase 2 target: lib.wasm + lib.wit + lib.component.wasm (artifacts: lib.wasm kind=wasm-module role=primary-library; lib.wit kind=wit role=interface-wit; lib.component.wasm kind=wasm-component role=primary-component)
kali build --component --api node lib.ts   # Phase 3 target: still gated by the Node build surface even after component packaging exists
kali build --sandbox kali.policy.json main.ts # Phase 1: validate policy file/config; Phase 2+: also validate inferred effects
kali build --bundle --api browser --sandbox kali.policy.json main.ts # Build-time policy compatibility only; no automatic browser-runtime enforcement is implied after deployment
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

### `kali check [files...]`
Type-check without compiling.
```bash
kali check                                 # Type-check the canonical project-discovery result
kali check main.ts                         # Type check executable/analyzable source
kali check src/a.ts src/b.ts               # Type check an explicit file set
kali check types.d.ts                      # Validate a declaration-only file directly
kali check --api browser                   # Browser-targeted project-discovery analysis context
kali check --api browser main.ts           # Browser-targeted analysis context for an explicit file set (no standalone DOM runtime implied)
kali check --api node main.ts              # Phase 3 target: Node API surface is phase-gated for checking too
kali check --sandbox kali.policy.json      # Phase 1: project-wide check + policy file/config validation; Phase 2+: effect-policy validation over the discovered project graph
kali check --api browser --sandbox kali.policy.json # Same browser-targeted validation path over the discovered project graph
kali check --sandbox kali.policy.json main.ts # Same validation, but scoped to the explicit file set
kali check --sandbox kali.policy.json src/a.ts src/b.ts # Same rule with multiple explicit files; --sandbox does not turn check into a direct-input command
kali check --fix main.ts                   # Apply only safe, compiler-provided suggested fixes
```
`kali check` is the hybrid analysis command: it accepts explicit file inputs, and without them it falls back to the canonical project-discovery result. That remains true under `--api browser`: browser targeting changes the analysis context, not the command's hybrid input behavior. The same rule applies when `--sandbox` is present: `kali check --sandbox <policy>` without file arguments validates the discovered project graph rather than becoming a separate command mode, and `kali check --sandbox <policy> [files...]` keeps the same set-oriented explicit-file behavior as plain `check`. Browser-targeted policy validation follows the same discovery-vs-explicit-file split: `kali check --api browser --sandbox <policy>` without file arguments validates the discovered project graph under the browser-targeted analysis context, while explicit files keep the same set-oriented behavior. Declaration-only files are valid explicit file inputs for `check`; `run`, `build`, `effects`, and `test` primary inputs may not be declaration-only, and that input-kind mismatch should use the canonical invalid-entrypoint diagnostic (`E5007`).

`--fix` is intentionally conservative: it is limited to unambiguous structured edits attached to diagnostics, not arbitrary refactors or speculative type rewrites.

### `kali effects <file>`
Output static effect analysis as JSON.

Status: Phase 2 target. In Phase 1, the command may be unavailable or explicitly marked experimental while the internal effect infrastructure stabilizes.
```bash
kali effects main.ts                       # Compact effect report JSON to stdout (default API surface: deno)
kali effects --api browser main.ts         # Browser-targeted effect analysis once the Phase 2 command exists
kali effects --api node main.ts            # Phase 3 target: Node API surface remains gated here too
kali effects --pretty main.ts              # Pretty-printed effect report JSON
kali effects --output json main.ts         # Command envelope + effect payload
```
By default, `kali effects` prints the effect report payload directly because JSON is the primary output of the command. With `--output json`, it is wrapped in the standard command envelope described below. See [specs/18-schemas.md](18-schemas.md) for the canonical payload schema.

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
- `kali effects` is a direct-input command in early phases: it requires exactly one explicit executable/analyzable source-file analysis root and does not fall back to project-wide discovery
- `kali effects` accepts only executable/analyzable source files; declaration-only files are type inputs, not effect-report primary inputs
- unless overridden by CLI/config, `kali effects` uses the same default API-surface selection as `kali check` (`apiSurface = deno`)
- `--api browser` follows the same browser API-surface analysis context as `kali check --api browser`; in Phase 2 this extends browser-targeted analysis to `effects` without implying standalone browser execution
- `--api node` remains phase-gated until the documented Node surface exists
- `--compat ...` affects effect analysis too: enabled compatibility paths such as `eval` change the reported effect set/dynamic reasons only when that compatibility feature is actually implemented for the selected phase/profile

Compatibility rule:
- plain `kali effects ...` emits the raw effect-report payload
- `kali effects --output json ...` emits the standard command envelope with that same effect report under `payload`
- `--pretty` changes formatting only; it does not change the effect-report schema or field names
- if `--pretty` and `--output json` are combined, formatting applies to the outer command envelope while the nested effect payload remains schema-identical

### `kali fmt [files...]`
Format source files (implemented in `kali_fmt`).
```bash
kali fmt                                   # Format all supported JS/TS source + declaration files in project (.ts/.tsx/.mts/.cts/.js/.jsx/.mjs/.cjs/.d.ts/.d.mts/.d.cts)
kali fmt --check                           # Check formatting (CI mode, exit code 1 if unformatted)
kali fmt main.ts                           # Format specific file
kali fmt src/a.ts src/b.ts                 # Format an explicit file set
```

Canonical discovery rule:
- `kali fmt` is project-oriented with no files, but when explicit paths are supplied it follows the shared **set-oriented explicit-file command** rule from [SPEC.md](../SPEC.md)
- project-oriented format discovery starts from the canonical project file set and then keeps the formatter's supported source-file set: executable/analyzable files plus declaration-only files (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.d.ts`, `.d.mts`, `.d.cts`)
- when explicit file arguments are supplied, those paths are formatted directly if they belong to that same supported set
- `--check` changes rewrite behavior only; it does not change discovery, supported file kinds, or the set-oriented explicit-file contract

### `kali lint [files...]`
Lint source files (implemented in `kali_lint`).
```bash
kali lint                                  # Lint all supported JS/TS source + declaration files in project
kali lint --fix                            # Auto-fix where possible
kali lint src/a.ts src/b.ts                # Lint an explicit file set
```

Canonical discovery rule:
- `kali lint` is project-oriented with no files, but when explicit paths are supplied it follows the shared **set-oriented explicit-file command** rule from [SPEC.md](../SPEC.md)
- project-oriented lint discovery starts from the canonical project file set and then keeps the same supported source-file set as `kali fmt`: executable/analyzable files plus declaration-only files (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.d.ts`, `.d.mts`, `.d.cts`)
- when explicit file arguments are supplied, those paths are linted directly if they belong to that same supported set
- `--fix` is intentionally conservative, like `check --fix`: it applies only structured tool-provided edits rather than speculative rewrites or stylistic churn outside the selected lint rules

### `kali test [files...]`
Run test files.
```bash
kali test                                  # Run discovered tests matching supported executable source extensions
kali test --filter "math"                  # Filter by name
kali test --sandbox kali.policy.json       # Run tests in sandbox
kali test --coverage                       # Phase 2 target: with coverage report once the stable contract lands
kali test --api deno                       # Supported early standalone test profile
kali test --api node                       # Phase 3 target
kali test --api browser                    # Rejected in early phases; browser is an analysis/build context first
```

Canonical discovery rule:
- default test discovery starts from the canonical project-discovery result, then matches `*.test.*` / `*_test.*` only across the shared executable/analyzable source set (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`)
- declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`) are never test entrypoints even if they match the naming pattern
- if explicit file arguments are supplied to `kali test`, those paths bypass the naming-pattern discovery filter and are treated as direct test-module inputs instead
- each explicit `kali test` file must still belong to the executable/analyzable set; passing a declaration-only file is the canonical invalid-entrypoint error (`E5007`), not a silent skip

Canonical host/profile rule: `kali test` follows the same early-phase API-surface gating as `kali run`, and analysis/build commands (`kali check`, `kali effects`, `kali build`) follow the same API-surface maturity rules for `--api node` / `--api browser` unless [specs/19-feature-maturity.md](19-feature-maturity.md) explicitly says otherwise.

### `kali init`
Initialize a new project scaffold.
```bash
kali init                                  # Create the minimal project scaffold in the current dir (kali.json + smallest entry file)
kali init --lib                            # Library project template
```

Scaffold simplification rules:
- `kali init` is **current-directory-scoped** in schema v1: it scaffolds the current working directory and does not retarget itself to an ancestor project root discovered above it.
- if the current working directory already contains `kali.json`, `kali init` fails with `E5008` instead of overwriting the existing project config.
- if an ancestor directory contains `kali.json` but the current working directory does not, `kali init` may still create a nested child project rooted at the current working directory; later project discovery then treats that child as a separate project boundary.
- `kali init` should generate the **minimal canonical** `kali.json` shape unless the selected template truly needs more.
- For the default app template, that normally means a `kali.json` containing only `{ "schemaVersion": 1 }` plus `main.ts`.
- For the library template, that normally means the same minimal `kali.json` plus `lib.ts`.
- The default scaffold should not pre-populate empty `dependencies`, `devDependencies`, `compat`, `sandbox`, or other placeholder sections just to advertise features.
- `kali init --lib` may add library-oriented source/layout hints, but it should still reuse the same canonical config naming (`apiSurface`, `buildMode`, `runtimeProfiles`) instead of inventing template-specific aliases.
- `kali init --lib` selects a **project template**, not an implicit default for the later `kali build --lib` artifact selector; template choice and build artifact mode remain separate knobs.
- `kali init` should also create only the smallest source/layout skeleton needed for the chosen template (for example `main.ts` for the default app template or `lib.ts` for the library template) instead of emitting multiple unused example files.
- follow the canonical scaffold filename convention from [SPEC.md](../SPEC.md): `main.ts` for the default app template and `lib.ts` for the library template, unless a later template spec explicitly opts into a different filename.
- Dependency state is still created by `kali install`, not by `kali init`.

### `kali install [target]`
Install or materialize project dependencies.

Lifecycle scripts stay disabled by default. The one explicit opt-in is `--allow-scripts`, which permits npm lifecycle hooks for this install invocation only. Packages that require native addons or install-time binary/bootstrap artifacts remain unsupported even when scripts are enabled.

Boundary rule:
- `--allow-scripts` is an **install-time tooling escape hatch**, not a runtime/API-surface feature
- enabling it does **not** imply `--api node`, does not cause lifecycle scripts to participate in `kali effects`, and does not make project `--sandbox` / `kali.json#sandbox` govern install-time hook execution
- pairing `--allow-scripts` with an explicit raw URL install target is invalid command usage (`E5008`) because raw URLs do not expose npm lifecycle hooks
- pairing `--allow-scripts` with an explicit `jsr:` package target is also invalid command usage (`E5008`) in schema v1 because JSR packages do not participate in npm lifecycle-script execution
- plain `kali install --allow-scripts` is valid only when the invocation has non-empty **effective npm-scriptable install work**; on a URL-only, JSR-only, or otherwise no-npm-scriptable install graph it should fail with `E5008` instead of silently degenerating into plain `install`
- that npm-scriptable subset is **invocation-scoped**: it covers the npm package work the current install actually reconciles, including any directly requested npm target and any transitively touched npm dependencies in the same invocation
- mixed install graphs are still valid: if one invocation touches npm packages plus JSR packages and/or raw URLs, lifecycle scripts may run only for the npm subset while the non-npm subset stays on the normal script-free path
- package-compatibility claims for normal `check` / `build` / `run` / `test` remain separate from this narrower opt-in install behavior
```bash
kali install lodash                        # Add/install registry dependency from npm
kali install jsr:@std/path                 # Add/install registry dependency from JSR
kali install                               # Materialize all declared dependencies for the project
kali install --allow-scripts               # Permit lifecycle hooks for the invocation's effective npm-scriptable install work
kali install --dev vitest                  # Add/install dev dependency
kali install --allow-scripts lodash        # Opt into lifecycle scripts for one npm package install; invalid for explicit `jsr:` or raw-URL targets; still not a promise that binary/bootstrap-heavy packages are supported
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument-kind rules:
- `kali install [target]` accepts at most one explicit install target in early phases; multiple install targets are invalid command usage (`E5008`)
- a **registry install target** uses the canonical registry-package identifier grammar from [specs/14-packages.md](14-packages.md): normal npm package names (for example `lodash` or `@types/node`) and `jsr:`-prefixed JSR names (for example `jsr:@std/path`)
- in schema v1, that explicit registry install target is a **package identity only**, not an inline version/range selector
- adding a registry package through this identity-only CLI form uses the shared stable-release rule from [specs/14-packages.md](14-packages.md): resolve the latest non-yanked stable published version, write `kali.lock` with that concrete version, and record the manifest dependency using the canonical default range `^<resolvedVersion>`
- if that identity-only lookup finds the package but no acceptable non-yanked stable release, the command fails with `E5001` instead of silently selecting a prerelease or pretending the package was installable under the schema-v1 input form
- a **registry install target** updates `dependencies` or `devDependencies` in `kali.json`, then refreshes `kali.lock` and materialized state
- in the canonical configless project mode, an explicit registry-package add (`kali install <pkg>` or `kali install --dev <pkg>`) first creates the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then records the dependency there; this keeps package adds on one manifest path instead of inventing a configless side channel
- `kali install` does **not** take `--api` in early phases; install is profile-agnostic, so passing `--api ...` is invalid command usage (`E5008`) rather than a request for a second install graph
- `--dev` is valid only with a **registry install target**; using `--dev` without an explicit registry target or pairing it with a raw URL (`kali install --dev https://...`) is rejected explicitly rather than inventing a second URL-specific manifest bucket
- a **raw URL install target** pins/materializes that exact URL dependency in `kali.lock` and `.kali/cache/urls/`, but does **not** create a parallel manifest section or silently rewrite source/import-map entries
- in the canonical configless project mode, an explicit raw-URL install may still create `kali.lock` and `.kali/cache/urls/` state at the effective project root, but it must not create a placeholder `kali.json` by itself
- an ad hoc raw-URL install is therefore a **staging/pin workflow**; if the project does not reference that URL from source or `kali.json#imports`, a later plain `kali install` may prune it again
- plain `kali install` consumes the current manifest/import graph and reconciles lock + materialized state for the dependency source kinds actually used by the project
- in the canonical configless project mode, plain `kali install` is a no-op success when the effective project root contributes no manifest/import/source dependency inputs, and it must not create a placeholder `kali.json` just because the command ran
- because `kali install` normally has no explicit primary source input, source-level raw URL imports are discovered from the canonical project-discovery result (filtered by `include` / `exclude` when present, otherwise by the default project-discovery rules from [SPEC.md](../SPEC.md))
- this discovery step may be a cheap lexical/module-specifier scan rather than a full build, and it may scan declaration-only files too because they can participate in the project's type/import graph
- because raw URL entries are owned by the current source/import-map graph instead of a manifest dependency table, plain `kali install` may prune raw URL lock/cache entries that are no longer referenced
- `kali install` is intentionally **profile-agnostic** in early phases: it locks versions and materializes package contents once for the current manifest/import graph, but it does not pre-bake a separate install for each `--api` surface; later `check` / `effects` / `build` / `run` / `test` choose `deno`/browser-targeted package branches from the already-installed metadata at command time

Determinism rules:
- `kali install` is the command that updates dependency-owning manifest fields when needed, resolves versions, pins URL imports, and writes `kali.lock`.
- `kali check`, `effects`, `build`, `run`, and `test` consume existing project-managed dependency state; they must not silently modify dependency-owning parts of `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/` as a side effect. Missing URL-cache materialization is treated the same as missing `node_modules/`: fail with `E5004` and point the user to `kali install`.
- For `E5004`, "stale" means the current manifest/import graph, lockfile entries, and required materialized artifacts no longer match for the dependency kinds the project actually uses. It does **not** require ad hoc timestamp-based guessing by non-install commands.
- If dependency state is missing or stale for the dependency source kinds the project actually uses, those non-install commands fail with the canonical `E5004` path and point the user to `kali install`.
- If a file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) is pointed at explicit files outside the last installed project discovery set and those files reach additional raw URL imports, the command still fails with `E5004`; explicit targets bypass discovery filtering, but they do not retroactively widen the install-time declaration graph.
- the intended fix is to make those sources part of the install-time declaration graph (for example by widening `include` / `exclude` or adding the relevant source/import-map declaration) and then rerun `kali install`; non-install commands must not auto-install or mutate the dependency graph opportunistically.
- `--allow-scripts` is install-scoped only; it does not loosen later execution/build sandbox rules.
- lifecycle scripts enabled through `--allow-scripts` are outside the normal source-program sandbox/effect-report contract; they are install-time package hooks, not guest-program entrypoints.
- Registry packages (npm/JSR) are materialized into `node_modules/`; raw URL imports are materialized under `.kali/cache/urls/`. Non-install commands consume whichever of those stores are relevant to the current project instead of assuming every project must have both.

### `kali package-effects <package>`
Analyze effects of a registry package under the canonical schema-v1 registry-analysis rules.

Argument-kind rule:
- `kali package-effects <package>` takes exactly one explicit package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- `<package>` uses the canonical **registry package identifier** spelling from [SPEC.md](../SPEC.md): normal npm package names (for example `lodash` or `@types/node`) and `jsr:`-prefixed JSR names
- early schema-v1 package analysis takes the **identity-only registry target** form from [SPEC.md](../SPEC.md), not an inline version/range selector
- version selection follows the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md), and the resolved version is recorded in the output payload
- if that identity-only package lookup finds the package but no acceptable non-yanked stable release, the command fails with `E5001`
- any non-registry target is rejected for `package-effects` in early phases, including raw URLs and local file paths; this command analyzes registry packages only, while raw URL dependencies remain part of the project/import-graph workflow handled by `kali install` + `kali effects`

Project-state rule:
- follow the **registry-analysis project-independence rule** from [SPEC.md](../SPEC.md)
- `package-effects` may still inherit its analysis context from the **effective command context**
- turning an analyzed package into a project dependency remains the job of `kali install`

Status: **Phase 2 target**. Before then, if package-level analysis is unavailable, the CLI should report that clearly instead of returning partial ad hoc output.
```bash
kali package-effects lodash                # Analyze npm package
kali package-effects jsr:@std/path         # Analyze JSR package
kali package-effects --pretty lodash       # Pretty-printed package-effect report JSON
kali package-effects --output json lodash  # Command envelope + package-effect payload
```
By default, `kali package-effects` emits its native JSON payload directly, following the same simplification as `kali effects`. With `--output json`, that payload is wrapped in the standard command envelope. `--pretty` changes formatting only; if combined with `--output json`, it formats the outer envelope while leaving the nested package-effect payload schema-identical. See [specs/18-schemas.md](18-schemas.md) for the canonical package-effect payload schema.

Analysis scope rule:
- `kali package-effects <pkg>` summarizes the statically reachable package graph selected for that package analysis under the active analysis context; it is not just a shallow inspection of the package's top-level manifest
- in schema v1, that analysis starts from the package version selected by the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md) rather than from any already-installed project copy or lockfile entry
- the nested `report.entryPoints` field should name that package-analysis logical root using the same canonical registry identifier spelling the user targeted (`lodash`, `@types/node`, `jsr:@std/path`) rather than an opaque tarball URL or cache path
- in early phases, that analysis context is inherited from the **effective command context** rather than from package-specific `--api` / `--compat` flags
- because the command intentionally reuses inherited context instead of growing a second near-duplicate flag family, `kali package-effects` does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` in early phases; passing any of them is invalid command usage (`E5008`) unless a later spec explicitly adds that mode
- practical consequence: non-default `package-effects` contexts currently come from defaults or discovered config only. In configless mode, the command therefore stays on the schema-v1 defaults (`apiSurface = deno`, `runtimeProfiles = []`, `compat.features = []`) instead of offering package-analysis-only CLI escape hatches.
- inherited analysis context follows the same axis-specific maturity gates as the rest of effect analysis rather than a package-only shadow rule set: browser inherits the browser-targeted analysis path, Node inherits the Node analysis gate, `wasm-threads` inherits the threaded-profile gate, and compat features such as `eval` inherit their own compatibility gate
- if inherited config/default analysis context selects a mode that is still unavailable for this command, `kali package-effects` should fail with `E5006` rather than silently analyzing under some other context
- inherited `apiSurface = browser` is the intended browser-targeted package-analysis path once `kali package-effects` exists in Phase 2; that keeps package analysis aligned with the same browser ambient typing layer and browser **package-resolution context** used by `kali check --api browser`
- the nested `report.analysisContext` field records that inherited context explicitly so tools do not have to infer it from ambient project state

### `kali package-audit <package>`
Security audit for one registry package under the canonical schema-v1 registry-analysis rules.

Argument-kind rule:
- `kali package-audit <package>` accepts exactly one explicit registry-package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- the package argument uses the canonical **registry package identifier** spelling from [SPEC.md](../SPEC.md)
- early schema-v1 package audit likewise takes the **identity-only registry target** form from [SPEC.md](../SPEC.md), not an inline version/range selector
- version selection follows the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md); once Kali defines a dedicated audit payload schema, that payload should record the resolved version as result metadata rather than forcing it into the required input spelling
- if that identity-only package lookup finds the package but no acceptable non-yanked stable release, the command fails with `E5001`
- any non-registry target is rejected for `package-audit` in early phases, including raw URLs and local file paths; package-audit is registry-package-oriented rather than a second raw-URL/local-path analysis path

Project-state rule:
- follow the same **registry-analysis project-independence rule** from [SPEC.md](../SPEC.md)
- unlike `package-effects`, early `package-audit` does not inherit semantic analysis context from the effective command context

Status: **Later compatibility**. It should not block Phase 1-2 compiler/runtime delivery, and if unimplemented the CLI should fail clearly rather than implying a partial security guarantee.
```bash
kali package-audit lodash                  # Audit specific npm package
kali package-audit jsr:@std/path           # Audit specific JSR package
kali package-audit --output json lodash    # Standard command envelope only until a dedicated audit payload schema exists
kali package-audit --pretty --output json lodash # Pretty-print that envelope; plain `--pretty` alone is invalid here because package-audit is not native JSON in schema v1
```
Additional flag-surface rule:
- like `package-effects`, `package-audit` does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` in early phases; passing them is invalid command usage (`E5008`) unless a later spec explicitly adds them
- unlike `package-effects`, early `package-audit` remains **context-free** with respect to the host-analysis/runtime/sandbox context bundle: inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox` do **not** change its semantics
- therefore config-selected host-analysis/runtime values such as `apiSurface = node`, `apiSurface = browser`, `runtimeProfiles = ["wasm-threads"]`, or `compat.features = ["eval"]` do **not** by themselves gate or rewrite `package-audit`; the command either remains unavailable by its own maturity row or runs with the same context-free semantics once implemented

Output simplification rule:
- unlike `kali effects` and `kali package-effects`, `kali package-audit` does **not** define a native bare-JSON payload in schema v1
- if `package-audit` supports `--output json` before a dedicated audit payload schema exists, it uses the canonical **envelope-only JSON support** model from [SPEC.md](../SPEC.md): the stable contract is the standard command envelope itself, with `payload` omitted or `null`
- because of that envelope-only model, `kali package-audit --pretty <pkg>` without `--output json` is invalid command usage (`E5008`) rather than an implicit request for JSON mode
- in that envelope-only phase, `package-audit` must not smuggle audit/package/version result metadata through `stdout`, `stderr`, or other prose-bearing envelope fields just because no dedicated payload exists yet
- if/when a dedicated machine-readable audit payload is added later, it should still travel through the standard `--output json` command envelope instead of inventing a second ad hoc top-level format

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

Quiet-mode interaction rule:
- `--quiet` suppresses extra success/status text, not the command's primary payload
- for ordinary human-oriented commands, that usually means nothing is printed on success unless the command's main purpose is to emit stdout from the user program or a requested machine payload
- for schema v1's native-JSON reporting commands (`kali effects`, `kali package-effects`) and for `--output json` modes, the requested JSON payload/envelope remains the primary output even under `--quiet`

Pretty-print interaction rule:
- `--pretty` is meaningful only in **JSON-producing mode**
- `--pretty` does **not** opt a command into JSON mode by itself
- for `kali effects` and `kali package-effects`, plain success output is already JSON, so `--pretty` reformats that native payload
- for any command with `--output json`, including envelope-only JSON commands such as early `package-audit --output json`, `--pretty` reformats the outer command envelope
- if a command is not otherwise emitting JSON (for example `kali check --pretty` without `--output json`, or early `kali package-audit --pretty lodash` without `--output json`), `--pretty` is invalid command usage (`E5008`) rather than a silent no-op
- `--pretty` changes formatting only; it must not change field names, ordering guarantees, or whether stderr/human diagnostics are emitted outside JSON mode

Feature gating is part of the machine contract too: phase/profile rejections should serialize the same stable diagnostic code and note structure as human output. When the failure depends on merged CLI/config state (for example a config-selected API surface or a contradictory artifact-mode combination), JSON diagnostics should also populate the optional structured `context` metadata from [specs/18-schemas.md](18-schemas.md) so tools can see the effective value without scraping prose.

Rules:
- top-level output uses the versioned command envelope
- diagnostics reuse the shared diagnostic schema
- command-specific structured data goes in `payload` when that command has a dedicated success-payload schema in schema v1
- commands with **envelope-only JSON support** may still support `--output json` through the standard envelope alone; in that case `payload` should be omitted or `null` rather than filled with ad hoc prose/fields
- envelope-only JSON support also does **not** permit commands to smuggle structured success metadata through `stdout` / `stderr`; those fields remain reserved for captured text streams
- common optional top-level fields include `artifacts`, `stdout`, `stderr`, `timings`, and `exitCode`
- for execution-style commands in JSON mode, guest/program stdout and stderr are captured into the envelope fields instead of being interleaved as raw terminal text
- build-like commands should populate artifact `role` whenever it helps distinguish artifact mode without forcing tools to guess from filenames (for example default executable vs `--lib` `wasm-module`)

Exception: schema v1's native-JSON reporting commands (`kali effects`, `kali package-effects`) already emit JSON as their native outputs, so `--output json` wraps those payloads in the envelope instead of changing their underlying schemas.

Native-JSON reporting command-stream rule:
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
- `kali init` should emit only the smallest canonical shape needed for the chosen template.
- For the default app template, that usually means just `{"schemaVersion": 1}` on disk plus `main.ts` in the source tree.
- For the library template, that usually means the same minimal config plus `lib.ts`.
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
- `compilerOptions.strict` is the config-level strictness bundle; its semantics live in [specs/04-type-system.md](04-type-system.md) and it should not be re-expanded into many parallel booleans in early phases
- `compilerOptions.maxSpecializations` caps specialization fan-out for generic/layout-driven optimization in modes that actively specialize; CLI `--max-specializations` overrides it for a single invocation
- `compilerOptions.maxSpecializations` is an upper bound rather than a promise that `buildMode = fast` will consume that full budget; `fast` may still skip most user-authored generic specialization by design
- top-level `sandbox` is an optional default policy-file path equivalent to supplying `--sandbox <path>` for the canonical sandbox-aware commands from [SPEC.md](../SPEC.md); an explicit CLI `--sandbox` overrides it
- relative `sandbox` paths in `kali.json` are resolved relative to the directory containing that config file rather than relative to whatever directory the user happened to run the command from
- omitting top-level `sandbox` means no default policy is attached; it does **not** ask tools to synthesize an implicit permissive policy file
- the canonical effect-reporting and sandbox-agnostic command classes from [SPEC.md](../SPEC.md) ignore the top-level `sandbox` setting rather than erroring or silently turning themselves into policy-validation commands
- `compat.features` is the config equivalent of CLI `--compat`; it uses the same canonical feature names, is order-insensitive, and should not duplicate them in alternate booleans
- in schema v1, the only canonical compatibility feature name is `"eval"`; it gates both direct `eval` support and the `Function()` constructor compatibility path
- `include` / `exclude` constrain the canonical project-discovery result for project-oriented commands, the dependency-graph install scan, and hybrid no-argument discovery commands such as `check`; direct file arguments still name the primary entry explicitly and are not silently filtered back out just because they sit outside the discovery globs
- unless overridden, project-oriented discovery still skips the default managed/generated directories named in [SPEC.md](../SPEC.md)
- `include` / `exclude` filter only the project's own discoverable files; they do not suppress transitive imports/dependencies reached from an accepted entrypoint and they are not a second package-resolution mechanism
- generated config from `kali init` should prefer these canonical names and should not duplicate them as parallel top-level keys
- `kali init` should not emit `sandbox`, `compat`, `dependencies`, or other optional sections unless the chosen template or user request actually needs them
- because absence of `sandbox` means “no policy attached” rather than “allow all by explicit policy”, tools should preserve omission when round-tripping minimal configs unless the user intentionally chooses a default policy path
- precedence is `CLI > kali.json > defaults`, except sandbox-policy restrictions still constrain effective runtime behavior

## Exit Codes

Interpretation rule:
- ordinary compile/check/build diagnostics over otherwise valid command inputs exit with **1**; this includes syntax/type/name errors, import/module/resolution failures, dependency-state failures (`E5001`-`E5006` as applicable), library-export proof failures (`E5011`), and Phase 2+ compile-time sandbox/effect violations
- this same `1` path also covers a **well-formed but context-incompatible** attached policy whose enabled capability/profile is unavailable for the effective command context (for example `effects.eval: true` before `--compat eval` exists, or browser-targeted `check` / `build --bundle` policies that request cross-cutting `resources.*` enforcement Kali cannot promise post-deployment)
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
| 1 | Ordinary compile/check/build diagnostic failure (for example syntax, type, name, module/resolution, dependency-state, feature-gating, or library-export-proof failures such as `E5001`-`E5006` and `E5011`, plus build-time sandbox/effect violations, semantically valid but context-incompatible policy enablement, `fmt --check`, and lint contract failures) |
| 2 | Runtime error (uncaught exception) |
| 3 | Runtime sandbox violation |
| 4 | Runtime resource limit exceeded |
| 5 | Configuration (`E5009`) / malformed or schema-invalid policy file (`E5010`) / CLI usage (`E5008`) / invalid command input or entrypoint (`E5007`) |
| 126 | Permission denied |
| 127 | File not found |
