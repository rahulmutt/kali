# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases

## Shared Flags

These flags are shared across the CLI, but some apply only to specific command families. For the canonical meaning of **API surface**, **build mode**, and **runtime profile**, see [SPEC.md](../SPEC.md). For command/profile gating, see [19 — Feature Maturity](19-feature-maturity.md).

Command-family terminology used in this chapter:
- **execution commands**: `run` and `test`
- **build-like commands**: `build`, plus the compile step embedded inside `run` and `test`
- **diagnostic-producing commands**: `check`, `build`, `run`, `test`, `fmt --check`, and `lint`

Naming rule:
- CLI keeps short flag names such as `--api`
- `kali.json` keeps the canonical leaf keys under `compilerOptions`: `apiSurface`, `buildMode`, and `runtimeProfiles`
- new docs, generated config, and machine-readable examples should use only these canonical config names

| Flag | Scope | Description |
|------|-------|-------------|
| `--verbose` | all commands | Detailed output: timing per phase, optimization decisions |
| `--output json` | all commands | Machine-parseable JSON output |
| `--quiet` | all commands | Suppress all non-error output |
| `--max-errors N` | diagnostic-producing commands | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | text-output commands | Color output control |
| `--api deno\|node\|browser` | `check`, `build`, `run`, `test` | Select host API surface; unsupported surfaces for the current command/profile must error explicitly (for example, early browser builds require `--bundle`) |
| `--compat <feature[,feature...]>` | `check`, `build`, `run`, `test` | Enable documented compatibility features such as `eval` only when that feature is implemented for the selected phase/profile |
| `--fast` | `build`, `run`, `test` | Fastest compile time, minimal optimization (default build mode) |
| `--release` | `build`, `run`, `test` | Standard optimization profile |
| `--release-advanced` | `build`, `run`, `test` | Aggressive optimization profile |
| `--sandbox <policy>` | `run`, `test`, `check`, `build` | Attach and validate `kali.policy.json`; in Phase 1 this enforces at runtime for `run`/`test` and validates policy/config for `check`/`build` |
| `--max-memory <size>` | execution commands | Override the invocation memory cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-cpu <duration>` | execution commands | Override the invocation CPU cap; may only tighten the effective limit relative to config/policy, never widen it |
| `--max-threads N` | execution commands | Override the invocation thread cap for the threaded runtime profile; may only tighten the effective limit and is rejected unless threading is supported and enabled |
| `--wasm-threads` | `build`, `run`, `test` | Opt into the later threaded runtime profile required for `SharedArrayBuffer` / `Atomics`; before that profile exists, or on unsupported targets, the command must fail with `E5006` |

`--fast`, `--release`, and `--release-advanced` are mutually exclusive; config files should use the single `compilerOptions.buildMode` field instead of parallel booleans. `run` and `test` inherit the selected build mode for their internal compile step. Runtime-profile toggles such as `--wasm-threads` map to entries in `compilerOptions.runtimeProfiles` rather than to separate booleans.

## Command-Specific Flags

To keep the shared-flag table small and avoid implying that every convenience flag is globally meaningful, command-local switches are listed here.

| Flag | Scope | Description |
|------|-------|-------------|
| `--bundle` | `build` | Required for the early browser-targeted build path (`build --bundle --api browser`) and may later select other multi-artifact packaging modes |
| `--lib` | `build`, `init` | Build or scaffold a library-oriented project/artifact without automatic program start |
| `--capi` | `build` | Emit the Phase-2 public C-embedding artifact set (`wasm-module` + `c-header` + `cabi-metadata`) |
| `--validate-ir` | `build` | Run internal IR validators as a debugging/developer aid |
| `--max-specializations N` | `build`, `run`, `test` | Override the specialization fan-out cap for a single invocation |
| `--fix` | `check`, `lint` | Apply only structured, tool-generated safe fixes for the selected command |
| `--pretty` | `effects` | Pretty-print the effect-report JSON without changing its schema |
| `--check` | `fmt` | Report formatting drift without rewriting files |
| `--filter <pattern>` | `test` | Run only matching tests |
| `--coverage` | `test` | Emit test coverage data once the coverage report contract is stabilized; before then this flag is phase-gated or explicitly experimental |
| `--dev` | `install` | Add the named registry dependency to `devDependencies` instead of `dependencies` |
| `--allow-scripts` | `install` | Opt into npm lifecycle scripts for that install invocation only; still rejects native addons / `node-gyp` |

Interpretation rule:
- command-specific flags inherit the same phase/profile gating rules as the command they belong to
- documenting a command-specific flag here does **not** imply it needs a separate feature-maturity row unless it changes a phase promise or machine-readable contract
- build output-mode flags should not silently combine into ambiguous artifact contracts; in early phases `--bundle`, `--lib`, and `--capi` are mutually exclusive selectors, and unsupported combinations must fail explicitly rather than guessing which artifact set the user meant

Config-array normalization rule:
- `compilerOptions.runtimeProfiles` and `compat.features` are set-like lists, not ordered pipelines
- entries should be unique
- unknown entries are diagnosed instead of ignored

Configuration precedence is intentionally simple:
1. CLI flags override `kali.json`
2. `kali.json` overrides built-in defaults
3. Sandbox policy caps remain upper bounds for runtime capabilities and resource limits

That means command-line resource flags can tighten a run relative to policy/config, but they must not silently widen a sandbox policy.

Canonical default tuple:
- `apiSurface = deno`
- `buildMode = fast`
- `runtimeProfiles = []`
- `compat.features = []`

This is the default interpretation of examples such as `kali run main.ts`, `kali test`, and `kali build main.ts` unless the example explicitly overrides a field. `kali check main.ts` uses the same default API surface selection.

## Commands

### `kali run <file>`
Compile and execute a TypeScript/JavaScript file.
```bash
kali run main.ts                           # Run with default settings
kali run --sandbox kali.policy.json main.ts # Run with sandbox
kali run --max-memory 256mb main.ts        # Resource limit
kali run --max-cpu 10s main.ts             # CPU time limit
kali run --api node main.ts                # Use Node.js API surface (Phase 3 target)
kali run --api deno main.ts                # Use Deno API surface (default)
kali run --api browser main.ts             # Rejected in early standalone phases; browser is a build/check profile first
kali run --wasm-threads main.ts            # Enable WASM threads (SharedArrayBuffer, Atomics; opt-in only)
```

Initial implementations use wasmtime; alternative runtime backends are a later-phase feature. Feature flags and subcommands that depend on later phases should be hidden or clearly diagnosed when unavailable rather than exposed as silently nonfunctional options.

When a command or flag is rejected due to phase/profile maturity, the CLI should use the canonical feature-maturity diagnostic shape from [specs/15-errors.md](15-errors.md) rather than ad hoc wording.

Canonical interpretation rules:
- `--api` selects an **API surface**, but support is command-dependent.
- `--api browser` is valid early for `check` and `build --bundle`; it is rejected for standalone `run`, and `build --api browser` without `--bundle` is also rejected, until a later runtime profile/output contract explicitly supports those modes.
- `--api node` is phase-gated consistently across `check`, `build`, `run`, and `test`; early phases reject it with `E5006` rather than exposing a partial Node surface.
- `--compat ...` is the one shared switch for later-phase dynamic compatibility features. If the named feature is not implemented yet, the command still fails with `E5006`.
- `--wasm-threads` selects a different runtime profile rather than a small optimization toggle. Until that threaded profile exists, the flag is rejected. After it exists, if the selected target/engine/profile cannot honor it, the command must still reject it explicitly instead of silently dropping thread support.
- `--max-threads N` is meaningful only together with the threaded runtime profile. A non-zero thread cap without effective thread support must be rejected explicitly rather than ignored.

Sandbox flag behavior is intentionally phase-gated:
- `kali run --sandbox ...` is a Phase 1 feature for runtime policy enforcement.
- `kali check/build --sandbox ...` validate the policy file/config in Phase 1.
- Full inferred-effect-vs-policy validation is a Phase 2 feature.
- Policy validation must also reject policies that try to enable capabilities unavailable in the selected command/profile/phase (for example `effects.eval: true` before the eval compatibility path exists, or `resources.maxThreads > 0` before the threaded runtime profile exists).
- Policy files remain declarative; any later host-registered sandbox policy predicates are an embedding-oriented extension, not a second inline policy language.

### `kali build <file>`
AOT compile to a WASM module or linked artifact set.

`--capi` and other public embedding-oriented outputs follow the embedding maturity rules in [specs/19-feature-maturity.md](19-feature-maturity.md): the compiler is library-first internally in Phase 1, but stable public embedding artifacts are a Phase 2 target.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default; artifact kind: wasm-module)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # main.wasm + main.js (artifact kinds: wasm-module + js-glue)
kali build --api browser main.ts           # Rejected in early phases; browser build path requires --bundle
kali build --api node main.ts              # Phase 3 target: Node API surface is not available early on build/check either
kali build --lib lib.ts                    # Library module (exports, no start)
kali build --capi lib.ts                   # Phase 2 target: lib.wasm + lib.exports.h + metadata (artifact kinds: wasm-module + c-header + cabi-metadata; see specs/13-embedding.md)
kali build --sandbox kali.policy.json main.ts # Phase 1: validate policy file/config; Phase 2+: also validate inferred effects
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

### `kali check <file>`
Type-check without compiling.
```bash
kali check main.ts                         # Type check
kali check --api browser main.ts           # Browser-targeted analysis/profile (no standalone DOM runtime implied)
kali check --api node main.ts              # Phase 3 target: Node API surface is phase-gated for checking too
kali check --sandbox kali.policy.json main.ts # Phase 1: type check + policy file/config validation; Phase 2+: effect-policy validation
kali check --fix main.ts                   # Apply only safe, compiler-provided suggested fixes
```
`--fix` is intentionally conservative: it is limited to unambiguous structured edits attached to diagnostics, not arbitrary refactors or speculative type rewrites.

### `kali effects <file>`
Output static effect analysis as JSON.

Status: Phase 2 target. In Phase 1, the command may be unavailable or explicitly marked experimental while the internal effect infrastructure stabilizes.
```bash
kali effects main.ts                       # Compact effect report JSON to stdout
kali effects --pretty main.ts              # Pretty-printed effect report JSON
kali effects --output json main.ts         # Command envelope + effect payload
```
By default, `kali effects` prints the effect report payload directly because JSON is the primary output of the command. With `--output json`, it is wrapped in the standard command envelope described below. See [specs/18-schemas.md](18-schemas.md) for the canonical payload schema.

Compatibility rule:
- plain `kali effects ...` emits the raw effect-report payload
- `kali effects --output json ...` emits the standard command envelope with that same effect report under `payload`
- `--pretty` changes formatting only; it does not change the effect-report schema or field names
- if `--pretty` and `--output json` are combined, formatting applies to the outer command envelope while the nested effect payload remains schema-identical

### `kali fmt [files...]`
Format source files (implemented in `kali_fmt`).
```bash
kali fmt                                   # Format all supported JS/TS source files in project (.ts/.tsx/.mts/.cts/.js/.jsx/.mjs/.cjs)
kali fmt --check                           # Check formatting (CI mode, exit code 1 if unformatted)
kali fmt main.ts                           # Format specific file
```

### `kali lint [files...]`
Lint source files (implemented in `kali_lint`).
```bash
kali lint                                  # Lint all files
kali lint --fix                            # Auto-fix where possible
```

### `kali test [files...]`
Run test files.
```bash
kali test                                  # Run all *_test.ts / *.test.ts
kali test --filter "math"                  # Filter by name
kali test --sandbox kali.policy.json       # Run tests in sandbox
kali test --coverage                       # Phase 2 target: with coverage report once the stable contract lands
kali test --api deno                       # Supported early standalone test profile
kali test --api node                       # Phase 3 target
kali test --api browser                    # Rejected in early phases; browser is a check/build profile first
```

Canonical host/profile rule: `kali test` follows the same early-phase API-surface gating as `kali run`, and `kali check` / `kali build` follow the same API-surface maturity rules for `--api node` / `--api browser` unless [specs/19-feature-maturity.md](19-feature-maturity.md) explicitly says otherwise.

### `kali init`
Initialize a new project scaffold.
```bash
kali init                                  # Create kali.json in current dir
kali init --lib                            # Library project template
```

Scaffold simplification rules:
- `kali init` should generate the **minimal canonical** `kali.json` shape unless the selected template truly needs more.
- The default scaffold should not pre-populate empty `dependencies`, `devDependencies`, `compat`, `sandbox`, or other placeholder sections just to advertise features.
- `kali init --lib` may add library-oriented source/layout hints, but it should still reuse the same canonical config naming (`apiSurface`, `buildMode`, `runtimeProfiles`) instead of inventing template-specific aliases.
- `kali init` should also create only the smallest source/layout skeleton needed for the chosen template (for example `main.ts` for the default app template or `mod.ts`/`lib.ts` for a library template) instead of emitting multiple unused example files.
- Dependency state is still created by `kali install`, not by `kali init`.

### `kali install [package]`
Install or materialize project dependencies.

Lifecycle scripts stay disabled by default. The one explicit opt-in is `--allow-scripts`, which permits npm lifecycle hooks for this install invocation only. Packages that require native addons remain unsupported even when scripts are enabled.
```bash
kali install lodash                        # Add/install registry dependency from npm
kali install                               # Materialize all declared dependencies for the project
kali install --dev vitest                  # Add/install dev dependency
kali install --allow-scripts esbuild       # Opt into lifecycle scripts for this install only
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument-kind rules:
- a **registry package argument** updates `dependencies` or `devDependencies` in `kali.json`, then refreshes `kali.lock` and materialized state
- a **raw URL argument** pins/materializes that URL dependency in `kali.lock` and `.kali/cache/urls/`, but does **not** create a parallel manifest section or silently rewrite source/import-map entries
- plain `kali install` consumes the current manifest/import graph and reconciles lock + materialized state for the dependency source kinds actually used by the project

Determinism rules:
- `kali install` is the command that resolves versions, pins URL imports, and writes `kali.lock`.
- `kali check`, `build`, `run`, and `test` consume existing dependency state; they must not silently modify `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/` as a side effect. Missing URL-cache materialization is treated the same as missing `node_modules/`: fail with `E5004` and point the user to `kali install`.
- If dependency state is missing or stale for the dependency source kinds the project actually uses, those non-install commands fail with the canonical `E5004` path and point the user to `kali install`.
- `--allow-scripts` is install-scoped only; it does not loosen later execution/build sandbox rules.
- Registry packages (npm/JSR) are materialized into `node_modules/`; raw URL imports are materialized under `.kali/cache/urls/`. Non-install commands consume whichever of those stores are relevant to the current project instead of assuming every project must have both.

### `kali package-effects <package>`
Analyze effects of an npm/JSR package before installing.

Status: depends on the Phase 2 effect-report pipeline; if package-level analysis is not yet implemented, the CLI should report that clearly instead of returning partial ad hoc output.
```bash
kali package-effects lodash                # Show effects used by package (JSON)
```

### `kali package-audit [package]`
Security audit for dependencies.

Status: later tooling feature. It should not block Phase 1-2 compiler/runtime delivery, and if unimplemented the CLI should fail clearly rather than implying a partial security guarantee.
```bash
kali package-audit                         # Audit all installed dependencies
kali package-audit lodash                  # Audit specific package
```

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

Feature gating is part of the machine contract too: phase/profile rejections should serialize the same stable diagnostic code and note structure as human output.

Rules:
- top-level output uses the versioned command envelope
- diagnostics reuse the shared diagnostic schema
- command-specific structured data goes in `payload`
- common optional top-level fields include `artifacts`, `stdout`, `timings`, and `exitCode`

Exception: `kali effects` already emits JSON as its native output, so `--output json` wraps that payload in the envelope instead of changing the effect-report schema itself.

This is intentional simplification: Kali has one canonical effect-report payload schema, and the command envelope is an outer transport wrapper rather than a second competing effect schema.

## Configuration (`kali.json`)

The canonical full config schema and example live in [specs/18-schemas.md](18-schemas.md). This chapter only repeats the naming rules so CLI and schema docs do not drift.

Minimal canonical shape:
```json
{
  "schemaVersion": 1
}
```

Omission/default rule for minimal configs:
- `kali init` should emit only the smallest canonical shape needed for the chosen template.
- For the default app template, that usually means just `{"schemaVersion": 1}`.
- Omitted fields inherit documented schema/CLI defaults rather than creating placeholder sections.
- In schema v1, omitted `compilerOptions` means all compiler-option defaults apply.
- In schema v1, omitted `compilerOptions.strict` means the default strict-checking bundle is enabled.
- In schema v1, omitted `compilerOptions.maxSpecializations` means the project uses the default specialization cap of `16`.
- Omitted `compat` means `compat.features = []`.

Configuration simplification rules:
- `compilerOptions.apiSurface` is the config equivalent of the CLI `--api` flag
- `compilerOptions.buildMode` replaces separate optimization booleans
- `compilerOptions.runtimeProfiles` is an array of explicit semantic runtime-profile switches; an empty array means the default single-threaded baseline, while a future threaded config would use `"runtimeProfiles": ["wasm-threads"]`
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- `compilerOptions.apiSurface` and `compilerOptions.runtimeProfiles` describe different axes and must not be conflated: `deno`/`node`/`browser` select host APIs, while runtime profiles select execution capabilities such as threads
- `compilerOptions.strict` is the config-level strictness bundle; it should mirror the documented strict-checking behavior rather than introducing many parallel booleans in early phases
- `compilerOptions.maxSpecializations` caps specialization fan-out for generic/layout-driven optimization; CLI `--max-specializations` overrides it for a single invocation
- top-level `sandbox` is an optional default policy-file path equivalent to supplying `--sandbox <path>` for commands that honor sandboxing; an explicit CLI `--sandbox` overrides it
- `compat.features` is the config equivalent of CLI `--compat`; it uses the same canonical feature names, is order-insensitive, and should not duplicate them in alternate booleans
- `include` / `exclude` constrain project file discovery for project-oriented commands; direct file arguments still name the primary entry explicitly
- generated config from `kali init` should prefer these canonical names and should not duplicate them as parallel top-level keys
- `kali init` should not emit `sandbox`, `compat`, `dependencies`, or other optional sections unless the chosen template or user request actually needs them
- precedence is `CLI > kali.json > defaults`, except sandbox-policy restrictions still bound the effective runtime behavior

## Exit Codes

Interpretation rule:
- compile/check/build diagnostics, including `E5006` feature gating and Phase 2+ compile-time sandbox/effect violations, exit with **1**
- runtime sandbox enforcement failures exit with **3**
- runtime resource exhaustion/fuel/memory-limit failures exit with **4**
- invalid CLI arguments, invalid config, or invalid policy schema/ranges exit with **5**

This keeps exit codes simple: command-time failures are separated from runtime enforcement failures.

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Compilation/check error (syntax, type, name resolution, build-time sandbox/effect violation, unsupported feature reported during compile/check) |
| 2 | Runtime error (uncaught exception) |
| 3 | Runtime sandbox violation |
| 4 | Runtime resource limit exceeded |
| 5 | Configuration / CLI usage / invalid policy file error |
| 126 | Permission denied |
| 127 | File not found |
