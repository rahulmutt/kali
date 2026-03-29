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

Command-family terminology used in this chapter:
- these labels describe **command shape and behavior**, not guaranteed current-phase availability
- commands such as `effects`, `package-effects`, or `package-audit` may still be phase-gated even though their command family is defined here
- canonical availability promises live in [19 — Feature Maturity](19-feature-maturity.md)
- **execution commands**: `run` and `test`
- **build-like commands**: `build`, plus the compile step embedded inside `run` and `test`
- **diagnostic-producing commands**: `check`, `build`, `run`, `test`, `fmt --check`, and `lint`

Canonical command-input mode rule (shared with [SPEC.md](../SPEC.md)):
- `run`, `build`, and `effects` are **direct-entry commands** in early phases: they require explicit executable/analyzable entrypoint arguments and do not guess `main.ts` or invent a project-default entrypoint
- `check` is a **hybrid analysis command**: it accepts explicit file arguments, or falls back to the canonical project-discovery result when no files are provided
- `fmt`, `lint`, and `test` are **project-oriented commands** when invoked without explicit file arguments
- `install` is the canonical **dependency-graph command**: with no explicit package argument it reconciles the discovered project dependency graph, including raw URL imports found through project discovery
- `package-effects` and `package-audit`, when available, are the canonical **registry-analysis commands**: each takes one explicit registry package identifier and does not invent a no-argument whole-project analysis mode in early phases
- `init` is not a source-entrypoint command

Canonical early-phase entrypoint-arity rule:
- `run`, `build`, and `effects` each take **exactly one** explicit primary entrypoint in early phases
- zero entrypoints for those commands is the canonical invalid-usage diagnostic `E5008`
- more than one explicit entrypoint for those commands is also `E5008` unless a later spec introduces a documented multi-entry mode
- `check`, `fmt`, `lint`, and `test` may still accept multiple explicit file arguments because their contracts are set-oriented rather than single-program oriented

Canonical package-argument arity rule:
- `kali install [package]` accepts **zero or one** explicit package argument in early phases
- `kali package-effects <package>` accepts **exactly one** explicit registry-package argument
- `kali package-audit <package>` accepts **exactly one** explicit registry-package argument
- passing more than the allowed number of explicit package arguments is `E5008` rather than permission to invent an undocumented batch mode
- omitting the required explicit package argument for a registry-analysis command is also `E5008`
- flags that conceptually modify an explicit package target (for example `kali install --dev`) require that target in early phases; using them without one is also `E5008`

Canonical input-kind rule:
- `run`, `build`, `effects`, and discovered `test` entrypoints accept only the shared executable/analyzable source-file set (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`)
- `check`, `fmt`, and `lint` accept that same executable/analyzable set **plus** declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`)
- declaration-only files may therefore be checked/formatted/linted directly and may also participate in ambient type loading and package type resolution
- declaration-only files are never valid runtime-bearing entrypoints; passing one where an executable entrypoint is required should fail explicitly with the canonical invalid-entrypoint diagnostic described in [specs/15-errors.md](15-errors.md) rather than being treated as an empty program or silently ignored
- when a command runs without explicit file arguments, it should discover files using the canonical project-discovery rules from [SPEC.md](../SPEC.md) rather than inventing a command-local root walk

Naming rule:
- CLI keeps short flag names such as `--api`
- `kali.json` keeps the canonical leaf keys under `compilerOptions`: `apiSurface`, `buildMode`, and `runtimeProfiles`
- new docs, generated config, and machine-readable examples should use only these canonical config names

Canonical config-discovery rule:
- unless a later spec adds an explicit `--config` override, commands discover the effective project config by searching the current working directory and then its ancestors for the nearest `kali.json`
- if none exists, the command runs configless with the current working directory as the effective project root
- explicit CLI file arguments do **not** relocate that chosen config/root; they resolve relative to the current working directory, while config-owned relative paths continue to resolve relative to the directory containing the discovered `kali.json`
- recursive project discovery for no-argument `check` / `fmt` / `lint` / `test` and for no-package-argument `install` graph scanning must stop at nested child directories that contain their own `kali.json` unless the user explicitly names files inside them

Effective-context validation rule:
- command validation always runs against the fully merged **effective command context** (built-in defaults, then discovered config, then CLI flags)
- therefore config-selected values trigger the same maturity/usage checks as explicit flags; the CLI must not silently "fix up" an inherited context by falling back to some other API surface/profile
- examples: config-selected `apiSurface = node` still causes plain `kali run main.ts` or `kali test` to hit the Node phase gate (`E5006`), and config-selected `apiSurface = browser` still makes plain `kali build main.ts` invalid early-phase usage (`E5008`) until `--bundle` is selected

| Flag | Scope | Description |
|------|-------|-------------|
| `--verbose` | all commands | Detailed output: timing per phase, optimization decisions |
| `--output json` | all commands | Machine-parseable JSON output |
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
| `--max-spawned-processes N` | execution commands | Override the invocation child-process cap; may only tighten the effective limit and is rejected when the selected command/profile/API surface does not support subprocesses |
| `--max-threads N` | execution commands | Override the invocation thread cap for the threaded runtime profile; may only tighten the effective limit and is rejected unless threading is supported and enabled |
| `--wasm-threads` | `check`, `effects`, `build`, `run`, `test` | Opt into the later threaded runtime profile required for `SharedArrayBuffer` / `Atomics`; before that profile exists, or on unsupported targets, the command must fail with `E5006` |

`--fast`, `--release`, and `--release-advanced` are mutually exclusive; config files should use the single `compilerOptions.buildMode` field instead of parallel booleans. `run` and `test` inherit the selected build mode for their internal compile step. Runtime-profile toggles such as `--wasm-threads` map to entries in `compilerOptions.runtimeProfiles` rather than to separate booleans.

Package-analysis flag/context simplification:
- follow the canonical command-context axis participation table and `analysis context` term in [SPEC.md](../SPEC.md)
- `kali package-effects`, when implemented, intentionally does **not** grow its own parallel analysis-context flag set in early phases (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`); instead it records the inherited analysis context in `report.analysisContext`
- that inherited package-effects context is limited to the semantic analysis axes (`apiSurface`, `runtimeProfiles`, `compat.features`); `buildMode` and `sandbox` remain non-semantic for the command in early phases
- `kali package-audit` likewise stays a single-package registry tool in early phases and does **not** add package-analysis-specific analysis-context flags (`--api`, runtime-profile flags, or `--compat`) before there is a documented need
- unlike `package-effects`, early `package-audit` is **context-free**: inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and `sandbox` do not change its semantics

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
| `--pretty` | `effects`, `package-effects` | Pretty-print the native JSON payload for effect-analysis commands without changing its schema |
| `--check` | `fmt` | Report formatting drift without rewriting files |
| `--filter <pattern>` | `test` | Run only matching tests |
| `--coverage` | `test` | Emit test coverage data once the coverage report contract is stabilized; before then this flag is phase-gated or explicitly experimental |
| `--dev` | `install` | Add the named registry dependency to `devDependencies` instead of `dependencies` |
| `--allow-scripts` | `install` | Opt into npm lifecycle scripts for that install invocation only; meaningful only when the effective install work includes at least one **npm** registry package, and still rejects native addons, `node-gyp`, and install-time binary/bootstrap package contracts |

Interpretation rule:
- command-specific flags inherit the same phase/profile gating rules as the command they belong to
- documenting a command-specific flag here does **not** imply it needs a separate feature-maturity row unless it changes a phase promise or machine-readable contract
- build artifact-mode flags follow the canonical matrix in [SPEC.md](../SPEC.md): in early phases `--bundle`, `--lib`, `--capi`, and `--component` are one small closed set of mutually exclusive selectors unless a later spec explicitly says one implies another
- the omitted selector means the default executable artifact mode; supplying more than one explicit selector from that set (for example `--bundle --lib`, `--bundle --capi`, `--bundle --component`, `--lib --capi`, `--lib --component`, or `--capi --component`) should use the canonical invalid-usage diagnostic `E5008`, not a feature-maturity rejection
- in Phase 1, `--bundle` is the browser packaging selector only: `kali build --bundle ...` requires the **effective API surface** to be `browser`, and `kali build --bundle` under an effective API surface of `deno` or `node` is invalid command usage (`E5008`) rather than a feature-maturity rejection, because the browser bundle mode itself exists but the selected flag/config combination is contradictory
- in early phases, `--lib`, `--capi`, and `--component` are **library-oriented artifact modes**: non-browser, export-oriented build modes derived from the module's explicit exports
- those library-oriented modes still obey the ordinary build-command API-surface gates: `kali build --lib --api browser ...`, `kali build --capi --api browser ...`, and `kali build --component --api browser ...` are `E5008` contradictions because browser mode is only defined for `--bundle`, while `kali build --lib --api node ...` remains on the same Phase 3 `E5006` path as other early `--api node` builds
- `--lib` is the base exported-library mode; `--capi` and `--component` are later packaging layers over that same exported-library contract rather than unrelated semantics
- because `--capi` and `--component` already choose exported-library semantics, users should not combine them with `--lib` in early phases; those flags are separate artifact-mode selectors, not additive modifiers
- WIT sidecars are not a separate artifact-mode selector: Phase 1 plain `--lib` emits the core library `wasm-module`, and once the public library/embedding surface stabilizes in Phase 2+, the relevant library-oriented modes emit WIT by default so callers do not have to choose between "C ABI" and "component metadata" paths

Config-array normalization rule:
- `compilerOptions.runtimeProfiles` and `compat.features` are set-like lists, not ordered pipelines
- entries should be unique
- unknown entries are diagnosed instead of ignored

Configuration precedence is intentionally simple:
1. CLI flags override the effective discovered `kali.json`
2. the effective discovered `kali.json` overrides built-in defaults
3. Sandbox policy caps, when a policy is attached, remain upper bounds for runtime capabilities and resource limits

That means command-line resource flags can tighten a run relative to policy/config, but they must not silently widen a sandbox policy. If no policy is attached, those direct invocation flags simply become the effective cap for the current command instead of being compared against an implicit allow-all policy. In Phase 1 this tightening path applies to `--max-memory`, `--max-cpu`, and `--max-open-files`; later resource flags such as `--max-spawned-processes` and `--max-threads` follow the same rule once their underlying capabilities exist.

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
- `--max-open-files` accepts a plain non-negative integer count
- `--max-spawned-processes` accepts a plain non-negative integer count
- `--max-threads` accepts a plain non-negative integer count
- CLI parsing normalizes these to bytes, milliseconds, and integer counts before comparing them with sandbox-policy limits
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

`kali run` is a direct-entry command in early phases: it requires exactly one explicit executable/analyzable source entrypoint and does not guess a project default such as `main.ts`.

Initial implementations use wasmtime; alternative runtime backends are a later-phase feature. Feature flags and subcommands that depend on later phases should be hidden or clearly diagnosed when unavailable rather than exposed as silently nonfunctional options.

When a command or flag is rejected due to phase/profile maturity, the CLI should use the canonical feature-maturity diagnostic shape from [specs/15-errors.md](15-errors.md) rather than ad hoc wording.

Canonical interpretation rules:
- `--api` selects an **API surface**, but support is command-dependent.
- browser mode is valid early for `check` and for `build` only when the selected artifact mode is the browser bundle path. In practice that means the **effective API surface** may be `browser` for `check`, and for `build` only together with `--bundle`; standalone `run` still rejects browser mode, and `build` with an effective API surface of `browser` but without `--bundle` is treated as invalid command usage (`E5008`) until a later runtime profile/output contract explicitly supports that mode.
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
- For browser-targeted `check --api browser --sandbox ...` and `build --bundle --api browser --sandbox ...`, non-deny `resources.*` policy budgets are rejected explicitly: those cross-cutting CPU/memory/file/process/thread budgets belong to Kali-hosted execution, not to the early browser deployment contract.
- Policy files remain declarative; any later host-registered sandbox policy predicates are an embedding-oriented extension, not a second inline policy language.
- If neither CLI nor config attaches a policy, the command runs with **no project policy file**; direct resource flags such as `--max-memory` and later supported caps such as `--max-spawned-processes` still apply, but there is no hidden synthesized policy document behind the scenes.

### `kali build <file>`
AOT compile to a WASM module or linked artifact set.

Canonical artifact-mode rule:
- `kali build` is a direct-entry command in early phases: it requires exactly one explicit executable/analyzable source entrypoint and does not guess a project default such as `main.ts`
- artifact selection follows the canonical matrix in [SPEC.md](../SPEC.md)
- omitting `--bundle`, `--lib`, `--capi`, and `--component` selects the default executable artifact mode
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive artifact-mode selectors unless a later spec explicitly defines one as an implication of another
- `kali init --lib` chooses a project template only; it does not change the later default artifact mode of `kali build`
- WIT sidecars for public library/embedding outputs are an output detail of those artifact modes, not a separate mode flag
- these **library-oriented artifact modes** derive their host-facing surface from the module's explicit exports; they do not implicitly expose arbitrary internal declarations just because the source file was compiled in `--lib`/`--capi`/`--component` mode
- plain `--lib` is the Phase-1 **base library** artifact: it establishes the exported-library shape early, but the stable public embedding/WIT contract remains Phase 2 work
- they also keep the ordinary build-command API-surface semantics: Node-targeted library builds are still phase-gated with `E5006`, while browser-targeted library/embedding combinations are invalid command shapes (`E5008`) until a separate browser-library contract exists

`--capi` and other public embedding-oriented outputs follow the embedding maturity rules in [specs/19-feature-maturity.md](19-feature-maturity.md): the compiler is library-first internally in Phase 1, but stable public embedding artifacts are a Phase 2 target.

Sandbox clarification:
- `kali build --sandbox ...` never executes the program; in Phase 1 it validates policy/config, and in Phase 2+ it also performs effect-vs-policy validation.
- For `kali build --bundle --api browser --sandbox ...`, this remains a **build-time** compatibility check only. It must not be described as automatic runtime sandbox enforcement once the emitted browser bundle is deployed into a real browser host.
- In that browser-targeted analysis/build context, cross-cutting `resources.*` budgets are not part of the supported contract and should be rejected when non-deny values are provided in the attached policy file.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default; artifact: kind=wasm-module, role=primary-executable)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # main.wasm + main.js (artifacts: main.wasm kind=wasm-module role=primary-executable; main.js kind=js-glue role=browser-glue)
kali build --bundle main.ts                # Invalid usage (E5008) under the default config; --bundle requires the effective API surface to be browser
kali build --bundle --api node main.ts     # Invalid usage (E5008); --bundle is the browser-only artifact mode, so pairing it with a non-browser API surface is contradictory
kali build --api browser main.ts           # Invalid usage (E5008) in early phases; browser build path requires --bundle
kali build --api node main.ts              # Phase 3 target: Node API surface is not available early on build/check either
kali build --lib lib.ts                    # Export-oriented base library module (no synthetic executable entry invocation; top-level init still runs on instantiation; Phase 1 artifact: kind=wasm-module, role=primary-library; stable public embedding/WIT contract lands in Phase 2+, which then adds kind=wit, role=interface-wit by default)
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
kali check --api browser main.ts           # Browser-targeted analysis context (no standalone DOM runtime implied)
kali check --api node main.ts              # Phase 3 target: Node API surface is phase-gated for checking too
kali check --sandbox kali.policy.json      # Phase 1: project-wide check + policy file/config validation; Phase 2+: effect-policy validation over the discovered project graph
kali check --sandbox kali.policy.json main.ts # Same validation, but scoped to the explicit file set
kali check --sandbox kali.policy.json src/a.ts src/b.ts # Same rule with multiple explicit files; --sandbox does not turn check into a direct-entry command
kali check --fix main.ts                   # Apply only safe, compiler-provided suggested fixes
```
`kali check` is the hybrid analysis command: it accepts explicit file inputs, and without them it falls back to the canonical project-discovery result. The same rule applies when `--sandbox` is present: `kali check --sandbox <policy>` without file arguments validates the discovered project graph rather than becoming a separate command mode, and `kali check --sandbox <policy> [files...]` keeps the same set-oriented explicit-file behavior as plain `check`. Declaration-only files are valid direct inputs for `check`; `run`, `build`, `effects`, and `test` entrypoints may not be declaration-only, and that input-kind mismatch should use the canonical invalid-entrypoint diagnostic (`E5007`).

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
- `kali effects <file>` summarizes effects for the full statically reachable graph rooted at that entrypoint under the selected API surface/profile; it is not limited to syntax that appears textually in the one named file
- `entryPoints` in the emitted payload identifies the analysis root(s), while `effects` summarizes the reachable program/dependency graph from those roots

Sandbox-interaction rule:
- `kali effects` reports inferred effects only; it does **not** accept `--sandbox`
- effect-vs-policy validation belongs to `kali check --sandbox ...` and `kali build --sandbox ...`
- rejecting `kali effects --sandbox ...` keeps one canonical policy-validation workflow instead of two overlapping ones
- that rejection is `E5008`, not a feature-maturity error: the command intentionally has no sandbox-comparison mode

Input-kind and host-selection rules:
- `kali effects` is a direct-entry command in early phases: it requires exactly one explicit executable/analyzable source-file entrypoint and does not fall back to project-wide discovery
- `kali effects` accepts only executable/analyzable source files; declaration-only files are type inputs, not effect-report entrypoints
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
```

### `kali lint [files...]`
Lint source files (implemented in `kali_lint`).
```bash
kali lint                                  # Lint all supported JS/TS source + declaration files in project
kali lint --fix                            # Auto-fix where possible
```

Canonical discovery rule:
- project-oriented lint discovery starts from the canonical project file set and then keeps the same supported source-file set as `kali fmt`: executable/analyzable files plus declaration-only files (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.d.ts`, `.d.mts`, `.d.cts`)
- when explicit file arguments are supplied, those paths are linted directly if they belong to that same supported set

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
kali init                                  # Create kali.json in current dir
kali init --lib                            # Library project template
```

Scaffold simplification rules:
- `kali init` should generate the **minimal canonical** `kali.json` shape unless the selected template truly needs more.
- For the default app template, that normally means a `kali.json` containing only `{ "schemaVersion": 1 }` plus the minimal entry source file.
- The default scaffold should not pre-populate empty `dependencies`, `devDependencies`, `compat`, `sandbox`, or other placeholder sections just to advertise features.
- `kali init --lib` may add library-oriented source/layout hints, but it should still reuse the same canonical config naming (`apiSurface`, `buildMode`, `runtimeProfiles`) instead of inventing template-specific aliases.
- `kali init --lib` selects a **project template**, not an implicit default for the later `kali build --lib` artifact selector; template choice and build artifact mode remain separate knobs.
- `kali init` should also create only the smallest source/layout skeleton needed for the chosen template (for example `main.ts` for the default app template or `mod.ts`/`lib.ts` for a library template) instead of emitting multiple unused example files.
- Dependency state is still created by `kali install`, not by `kali init`.

### `kali install [package]`
Install or materialize project dependencies.

Lifecycle scripts stay disabled by default. The one explicit opt-in is `--allow-scripts`, which permits npm lifecycle hooks for this install invocation only. Packages that require native addons or install-time binary/bootstrap artifacts remain unsupported even when scripts are enabled.

Boundary rule:
- `--allow-scripts` is an **install-time tooling escape hatch**, not a runtime/API-surface feature
- enabling it does **not** imply `--api node`, does not cause lifecycle scripts to participate in `kali effects`, and does not make project `--sandbox` / `kali.json#sandbox` govern install-time hook execution
- pairing `--allow-scripts` with an explicit raw URL argument is invalid command usage (`E5008`) because raw URLs do not expose npm lifecycle hooks
- pairing `--allow-scripts` with an explicit `jsr:` package target is also invalid command usage (`E5008`) in schema v1 because JSR packages do not participate in npm lifecycle-script execution
- plain `kali install --allow-scripts` is valid only when the effective install graph contains at least one **npm** registry package whose lifecycle hooks could run; on a URL-only, JSR-only, or otherwise no-npm-scriptable graph it should fail with `E5008` instead of silently degenerating into plain `install`
- package-compatibility claims for normal `check` / `build` / `run` / `test` remain separate from this narrower opt-in install behavior
```bash
kali install lodash                        # Add/install registry dependency from npm
kali install jsr:@std/path                 # Add/install registry dependency from JSR
kali install                               # Materialize all declared dependencies for the project
kali install --allow-scripts               # Permit lifecycle hooks for discovered npm packages in this install run
kali install --dev vitest                  # Add/install dev dependency
kali install --allow-scripts <pkg>         # Opt into lifecycle scripts for one npm package install; invalid for explicit `jsr:` or raw-URL targets; still not a promise that binary/bootstrap-heavy packages are supported
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument-kind rules:
- `kali install [package]` accepts at most one explicit package argument in early phases; multiple package arguments are invalid command usage (`E5008`)
- a **registry package argument** uses the canonical registry-package identifier grammar from [specs/14-packages.md](14-packages.md): normal npm package names (for example `lodash` or `@types/node`) and `jsr:`-prefixed JSR names (for example `jsr:@std/path`)
- in schema v1, that explicit registry-package install argument is a **package identity only**, not an inline version/range selector
- adding a registry package through this identity-only CLI form uses the shared stable-release rule from [specs/14-packages.md](14-packages.md): resolve the latest non-yanked stable published version, write `kali.lock` with that concrete version, and record the manifest dependency using the canonical default range `^<resolvedVersion>`
- a **registry package argument** updates `dependencies` or `devDependencies` in `kali.json`, then refreshes `kali.lock` and materialized state
- if no `kali.json` exists at the effective project root, an explicit registry-package add (`kali install <pkg>` or `kali install --dev <pkg>`) first creates the minimal canonical manifest `{ "schemaVersion": 1 }`, then records the dependency there; this keeps package adds on one manifest path instead of inventing a configless side channel
- `kali install` does **not** take `--api` in early phases; install is profile-agnostic, so passing `--api ...` is invalid command usage (`E5008`) rather than a request for a second install graph
- `--dev` is valid only with a **registry package argument**; using `--dev` without an explicit package or pairing it with a raw URL (`kali install --dev https://...`) is rejected explicitly rather than inventing a second URL-specific manifest bucket
- a **raw URL argument** pins/materializes that exact URL dependency in `kali.lock` and `.kali/cache/urls/`, but does **not** create a parallel manifest section or silently rewrite source/import-map entries
- an ad hoc raw-URL install is therefore a **staging/pin workflow**; if the project does not reference that URL from source or `kali.json#imports`, a later plain `kali install` may prune it again
- plain `kali install` consumes the current manifest/import graph and reconciles lock + materialized state for the dependency source kinds actually used by the project
- if no `kali.json` exists and the current project root also contributes no source/import-map dependency inputs, plain `kali install` is a no-op success and must not create a placeholder `kali.json` just because the command ran
- because `kali install` normally has no explicit entrypoint, source-level raw URL imports are discovered from the canonical project-discovery result (filtered by `include` / `exclude` when present, otherwise by the default project-discovery rules from [SPEC.md](../SPEC.md))
- this discovery step may be a cheap lexical/module-specifier scan rather than a full build, and it may scan declaration-only files too because they can participate in the project's type/import graph
- because raw URL entries are owned by the current source/import-map graph instead of a manifest dependency table, plain `kali install` may prune raw URL lock/cache entries that are no longer referenced
- `kali install` is intentionally **profile-agnostic** in early phases: it locks versions and materializes package contents once for the current manifest/import graph, but it does not pre-bake a separate install for each `--api` surface; later `check` / `effects` / `build` / `run` / `test` choose `deno`/browser-targeted package branches from the already-installed metadata at command time

Determinism rules:
- `kali install` is the command that resolves versions, pins URL imports, and writes `kali.lock`.
- `kali check`, `effects`, `build`, `run`, and `test` consume existing dependency state; they must not silently modify `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/` as a side effect. Missing URL-cache materialization is treated the same as missing `node_modules/`: fail with `E5004` and point the user to `kali install`.
- For `E5004`, "stale" means the current manifest/import graph, lockfile entries, and required materialized artifacts no longer match for the dependency kinds the project actually uses. It does **not** require ad hoc timestamp-based guessing by non-install commands.
- If dependency state is missing or stale for the dependency source kinds the project actually uses, those non-install commands fail with the canonical `E5004` path and point the user to `kali install`.
- If a direct-entry command names a file outside the last installed project discovery set and that file reaches additional raw URL imports, the command still fails with `E5004`; non-install commands must not auto-install or mutate the dependency graph opportunistically.
- `--allow-scripts` is install-scoped only; it does not loosen later execution/build sandbox rules.
- lifecycle scripts enabled through `--allow-scripts` are outside the normal source-program sandbox/effect-report contract; they are install-time package hooks, not guest-program entrypoints.
- Registry packages (npm/JSR) are materialized into `node_modules/`; raw URL imports are materialized under `.kali/cache/urls/`. Non-install commands consume whichever of those stores are relevant to the current project instead of assuming every project must have both.

### `kali package-effects <package>`
Analyze effects of an npm/JSR package independently of project install state.

Argument-kind rule:
- `kali package-effects <package>` takes exactly one explicit package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- `<package>` uses the same canonical registry-package identifier grammar as `kali install`: normal npm package names (for example `lodash` or `@types/node`) and `jsr:`-prefixed JSR names
- early schema-v1 package analysis takes a **package identity only**, not an inline version/range selector
- to keep registry analysis deterministic and project-independent in schema v1, the command uses the shared stable-release rule from [specs/14-packages.md](14-packages.md) and records the resolved version in the output payload
- `package-effects` therefore does **not** consult the current project's manifest or lockfile to choose a different version in early phases; a later explicit version/range or lock-aware mode would need its own documented selector
- raw URLs and local file paths are rejected for `package-effects`; this command analyzes registry packages, while raw URL dependencies remain part of the project/import-graph workflow handled by `kali install` + `kali effects`

Project-state rule:
- `kali package-effects <package>` may fetch package metadata/tarballs into an ephemeral analysis cache, but it must **not** mutate `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/`
- turning an analyzed package into a project dependency remains the job of `kali install`

Status: depends on the Phase 2 effect-report pipeline; if package-level analysis is not yet implemented, the CLI should report that clearly instead of returning partial ad hoc output.
```bash
kali package-effects lodash                # Analyze npm package
kali package-effects jsr:@std/path         # Analyze JSR package
kali package-effects --pretty lodash       # Pretty-printed package-effect report JSON
kali package-effects --output json lodash  # Command envelope + package-effect payload
```
By default, `kali package-effects` emits its native JSON payload directly, following the same simplification as `kali effects`. With `--output json`, that payload is wrapped in the standard command envelope. `--pretty` changes formatting only; if combined with `--output json`, it formats the outer envelope while leaving the nested package-effect payload schema-identical. See [specs/18-schemas.md](18-schemas.md) for the canonical package-effect payload schema.

Analysis scope rule:
- `kali package-effects <pkg>` summarizes the statically reachable package graph selected for that package analysis under the active analysis context; it is not just a shallow inspection of the package's top-level manifest
- in schema v1, that analysis starts from the package version selected by the shared stable-release rule from [specs/14-packages.md](14-packages.md) rather than from any already-installed project copy or lockfile entry
- the nested `report.entryPoints` field should name that analysis root using the same canonical registry identifier spelling the user targeted (`lodash`, `@types/node`, `jsr:@std/path`) rather than an opaque tarball URL or cache path
- in early phases, that analysis context is inherited from the effective `kali.json` / default analysis settings rather than from package-specific `--api` / `--compat` flags
- because the command intentionally reuses inherited context instead of growing a second near-duplicate flag family, `kali package-effects` does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` in early phases; passing any of them is invalid command usage (`E5008`) unless a later spec explicitly adds that mode
- the inherited context is still subject to the normal maturity rules for that command; for example, if config selects `apiSurface = node`, `runtimeProfiles = ["wasm-threads"]`, or `compat.features = ["eval"]` before those analysis modes are supported, `kali package-effects` should fail with `E5006` rather than silently analyzing under some other context
- inherited `apiSurface = browser` is the intended browser-targeted package-analysis path once `kali package-effects` exists in Phase 2; that keeps package analysis aligned with the same browser ambient/package-selection context used by `kali check --api browser`
- the nested `report.analysisContext` field records that inherited context explicitly so tools do not have to infer it from ambient project state
- the nested `report.entryPoints` field names those package-analysis roots using the shared effect-report schema

### `kali package-audit <package>`
Security audit for one registry package.

Argument-kind rule:
- `kali package-audit <package>` accepts exactly one explicit registry-package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- the package argument uses the canonical registry-package identifier grammar (normal npm package name or `jsr:`-prefixed JSR name)
- early schema-v1 package audit likewise takes a **package identity only**, not an inline version/range selector
- to keep audit behavior aligned with `package-effects` and avoid hidden project-state coupling, the command uses the shared stable-release rule from [specs/14-packages.md](14-packages.md) and, when it reports machine-readable/package metadata, includes that resolved version as result metadata rather than as part of the required input spelling
- raw URLs and local file paths are rejected for `package-audit`; package-audit is registry-package-oriented rather than a second raw-URL analysis path

Project-state rule:
- like `package-effects`, audit may use temporary fetched metadata but must not silently install or materialize dependencies into the project's managed state
- because schema-v1 package audit is registry-oriented rather than project-lock-oriented, it likewise resolves using the shared stable-release rule from [specs/14-packages.md](14-packages.md) instead of consulting the current project's manifest or lockfile by default

Status: later tooling feature. It should not block Phase 1-2 compiler/runtime delivery, and if unimplemented the CLI should fail clearly rather than implying a partial security guarantee.
```bash
kali package-audit lodash                  # Audit specific npm package
kali package-audit jsr:@std/path           # Audit specific JSR package
```
Additional flag-surface rule:
- like `package-effects`, `package-audit` does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` in early phases; passing them is invalid command usage (`E5008`) unless a later spec explicitly adds them
- unlike `package-effects`, early `package-audit` also does **not** inherit analysis context from `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, or `compat.features`; it remains a context-free registry tool
- top-level `kali.json#sandbox` is likewise ignored by `package-audit`, matching the broader sandbox-agnostic command rule from [SPEC.md](../SPEC.md)

Output simplification rule:
- unlike `kali effects` and `kali package-effects`, `kali package-audit` does **not** define a native bare-JSON payload in schema v1
- if/when machine-readable audit output is added, it should travel through the standard `--output json` command envelope instead of inventing a second ad hoc top-level format

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
- for `kali effects`, `kali package-effects`, and `--output json` modes, the requested JSON payload/envelope remains the primary output even under `--quiet`

Feature gating is part of the machine contract too: phase/profile rejections should serialize the same stable diagnostic code and note structure as human output. When the failure depends on merged CLI/config state (for example a config-selected API surface or a contradictory artifact-mode combination), JSON diagnostics should also populate the optional structured `context` metadata from [specs/18-schemas.md](18-schemas.md) so tools can see the effective value without scraping prose.

Rules:
- top-level output uses the versioned command envelope
- diagnostics reuse the shared diagnostic schema
- command-specific structured data goes in `payload`
- common optional top-level fields include `artifacts`, `stdout`, `stderr`, `timings`, and `exitCode`
- for execution-style commands in JSON mode, guest/program stdout and stderr are captured into the envelope fields instead of being interleaved as raw terminal text
- build-like commands should populate artifact `role` whenever it helps distinguish artifact mode without forcing tools to guess from filenames (for example default executable vs `--lib` `wasm-module`)

Exception: `kali effects` and `kali package-effects` already emit JSON as their native outputs, so `--output json` wraps those payloads in the envelope instead of changing their underlying schemas.

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
- For the default app template, that usually means just `{"schemaVersion": 1}`.
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
- `include` / `exclude` constrain the canonical project-discovery result for project-oriented commands, the dependency-graph install scan, and hybrid no-argument discovery commands such as `check`; direct file arguments still name the primary entry explicitly
- unless overridden, project-oriented discovery still skips the default managed/generated directories named in [SPEC.md](../SPEC.md)
- `include` / `exclude` filter only the project's own discoverable files; they do not suppress transitive imports/dependencies reached from an accepted entrypoint and they are not a second package-resolution mechanism
- generated config from `kali init` should prefer these canonical names and should not duplicate them as parallel top-level keys
- `kali init` should not emit `sandbox`, `compat`, `dependencies`, or other optional sections unless the chosen template or user request actually needs them
- because absence of `sandbox` means “no policy attached” rather than “allow all by explicit policy”, tools should preserve omission when round-tripping minimal configs unless the user intentionally chooses a default policy path
- precedence is `CLI > kali.json > defaults`, except sandbox-policy restrictions still constrain effective runtime behavior

## Exit Codes

Interpretation rule:
- compile/check/build diagnostics over otherwise valid command inputs, including `E5004` dependency-state failures, `E5006` feature gating, and Phase 2+ compile-time sandbox/effect violations, exit with **1**
- this same `1` path also covers a **well-formed but context-incompatible** attached policy whose enabled capability/profile is unavailable for the effective command context (for example `effects.eval: true` before `--compat eval` exists, or non-deny `resources.*` budgets on early browser-targeted `check` / `build --bundle`)
- `fmt --check` and lint-style contract failures that report ordinary command diagnostics also exit with **1**
- runtime sandbox enforcement failures exit with **3**
- runtime resource exhaustion/fuel/memory-limit failures exit with **4**
- invalid CLI arguments, invalid config (`E5009`), invalid policy schema/ranges (`E5010`), and command-input/entrypoint-usage mistakes exit with **5**
- malformed/invalid policy files stay on the `5` path; only semantically valid policy files that hit documented feature/profile gating move onto the ordinary diagnostic `1` path

Command-input/entrypoint-usage mistakes include:
- missing required direct-entry arguments for `run`, `build`, or `effects`
- too many explicit direct-entry arguments for those same commands in early phases
- conflicting artifact-mode selectors for `build` (for example `--bundle --lib` or `--lib --capi`)
- `E5007` invalid-entrypoint/input-kind cases such as passing a declaration-only file to `run`, `build`, `effects`, or `test`

This keeps exit codes simple: command-time failures are separated from runtime enforcement failures.

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Compilation/check-style diagnostic failure (`E5004` dependency state, syntax, type, name resolution, build-time sandbox/effect violation, unsupported feature reported during compile/check, semantically valid but context-incompatible policy enablement such as `E5006`, `fmt --check`, lint contract failures) |
| 2 | Runtime error (uncaught exception) |
| 3 | Runtime sandbox violation |
| 4 | Runtime resource limit exceeded |
| 5 | Configuration (`E5009`) / malformed or schema-invalid policy file (`E5010`) / CLI usage (`E5008`) / invalid command input or entrypoint (`E5007`) |
| 126 | Permission denied |
| 127 | File not found |
