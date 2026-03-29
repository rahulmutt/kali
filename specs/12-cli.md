# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases

## Shared Flags

These flags are shared across the CLI, but some apply only to specific command families. For the canonical meaning of **API surface**, **build mode**, and **runtime profile**, see [SPEC.md](../SPEC.md). For command/profile gating, see [19 — Feature Maturity](19-feature-maturity.md).

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

Config-array normalization rule:
- `compilerOptions.runtimeProfiles` and `compat.features` are set-like lists, not ordered pipelines
- entries should be unique
- unknown entries are diagnosed instead of ignored

Configuration precedence is intentionally simple:
1. CLI flags override `kali.json`
2. `kali.json` overrides built-in defaults
3. Sandbox policy caps remain upper bounds for runtime capabilities and resource limits

That means command-line resource flags can tighten a run relative to policy/config, but they must not silently widen a sandbox policy.

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

### `kali build <file>`
AOT compile to a WASM module.

`--capi` and other public embedding-oriented outputs follow the embedding maturity rules in [specs/19-feature-maturity.md](19-feature-maturity.md): the compiler is library-first internally in Phase 1, but stable public embedding artifacts are a Phase 2 target.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # WASM + JS glue for browsers
kali build --api browser main.ts           # Rejected in early phases; browser build path requires --bundle
kali build --api node main.ts              # Phase 3 target: Node API surface is not available early on build/check either
kali build --lib lib.ts                    # Library module (exports, no start)
kali build --capi lib.ts                   # Phase 2 target: foo.wasm + generated foo.exports.h/metadata for host-side embedding via kali_capi (see specs/13-embedding.md)
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

### `kali fmt [files...]`
Format source files (implemented in `kali_fmt`).
```bash
kali fmt                                   # Format all .ts/.js/.tsx/.jsx in project
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
kali test --coverage                       # With coverage report
kali test --api deno                       # Supported early standalone test profile
kali test --api node                       # Phase 3 target
kali test --api browser                    # Rejected in early phases; browser is a check/build profile first
```

Canonical host/profile rule: `kali test` follows the same early-phase API-surface gating as `kali run`, and `kali check` / `kali build` follow the same API-surface maturity rules for `--api node` / `--api browser` unless [specs/19-feature-maturity.md](19-feature-maturity.md) explicitly says otherwise.

### `kali init`
Initialize a new project.
```bash
kali init                                  # Create kali.json in current dir
kali init --lib                            # Library project template
```

### `kali install [package]`
Install npm/JSR packages.

Lifecycle scripts stay disabled by default. The one explicit opt-in is `--allow-scripts`, which permits npm lifecycle hooks for this install invocation only. Packages that require native addons remain unsupported even when scripts are enabled.
```bash
kali install lodash                        # Install from npm
kali install                               # Install all dependencies from kali.json
kali install --dev vitest                  # Dev dependency
kali install --allow-scripts esbuild       # Opt into lifecycle scripts for this install only
kali install https://deno.land/std/path/mod.ts  # URL import (cached)
```

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
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "deno",
    "buildMode": "fast",
    "runtimeProfiles": []
  }
}
```

Configuration simplification rules:
- `compilerOptions.apiSurface` is the config equivalent of the CLI `--api` flag
- `compilerOptions.buildMode` replaces separate optimization booleans
- `compilerOptions.runtimeProfiles` is an array of explicit semantic runtime-profile switches; an empty array means the default single-threaded baseline, while a future threaded config would use `"runtimeProfiles": ["wasm-threads"]`
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- `compilerOptions.apiSurface` and `compilerOptions.runtimeProfiles` describe different axes and must not be conflated: `deno`/`node`/`browser` select host APIs, while runtime profiles select execution capabilities such as threads
- `compat.features` is the config equivalent of CLI `--compat`; it uses the same canonical feature names, is order-insensitive, and should not duplicate them in alternate booleans
- generated config from `kali init` should prefer these canonical names and should not duplicate them as parallel top-level keys
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
