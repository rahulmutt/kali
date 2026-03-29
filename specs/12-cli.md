# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases

## Shared Flags

These flags are shared across the CLI, but some apply only to specific command families.

| Flag | Scope | Description |
|------|-------|-------------|
| `--verbose` | all commands | Detailed output: timing per phase, optimization decisions |
| `--output json` | all commands | Machine-parseable JSON output |
| `--quiet` | all commands | Suppress all non-error output |
| `--max-errors N` | diagnostic-producing commands | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | text-output commands | Color output control |
| `--api deno\|node\|browser` | compile/check/run commands | Select host API surface; unsupported surfaces for the current command/profile must error explicitly |
| `--compat <feature[,feature...]>` | compile/check/run commands | Enable documented compatibility features such as `eval` only when that feature is implemented for the selected phase/profile |
| `--fast` | compile-producing commands | Fastest compile time, minimal optimization (default build mode) |
| `--release` | compile-producing commands | Standard optimization profile |
| `--release-advanced` | compile-producing commands | Aggressive optimization profile |
| `--sandbox <policy>` | `run`, `test`, `check`, `build` | Attach and validate `kali.policy.json`; in Phase 1 this enforces at runtime for `run`/`test` and validates policy/config for `check`/`build` |
| `--max-memory <size>` | execution commands | Override/append memory limits for the current invocation |
| `--max-cpu <duration>` | execution commands | Override/append CPU limits for the current invocation |
| `--max-threads N` | execution commands | Override/append thread limits for the current invocation |
| `--wasm-threads` | compile/run/test commands | Opt into the threaded runtime profile required for `SharedArrayBuffer` / `Atomics`; unsupported targets or phases must fail with `E5006` |

`--fast`, `--release`, and `--release-advanced` are mutually exclusive; config files should use the single `buildMode` field instead of parallel booleans.

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
- `--api browser` is valid early for `check` and `build --bundle`; it is rejected for standalone `run` until a later runtime profile explicitly supports it.
- `--compat ...` is the one shared switch for later-phase dynamic compatibility features. If the named feature is not implemented yet, the command still fails with `E5006`.
- `--wasm-threads` selects a different runtime profile rather than a small optimization toggle. If the selected target/engine/profile cannot honor it, the command must reject it explicitly instead of silently dropping thread support.

Sandbox flag behavior is intentionally phase-gated:
- `kali run --sandbox ...` is a Phase 1 feature for runtime policy enforcement.
- `kali check/build --sandbox ...` validate the policy file/config in Phase 1.
- Full inferred-effect-vs-policy validation is a Phase 2 feature.

### `kali build <file>`
AOT compile to a WASM module.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle --api browser main.ts  # WASM + JS glue for browsers
kali build --lib lib.ts                    # Library module (exports, no start)
kali build --capi lib.ts                   # foo.wasm + generated kali.h/metadata for host-side embedding via kali_capi (see specs/13-embedding.md)
kali build --sandbox kali.policy.json main.ts # Phase 1: validate policy file/config; Phase 2+: also validate inferred effects
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

### `kali check <file>`
Type-check without compiling.
```bash
kali check main.ts                         # Type check
kali check --api browser main.ts           # Browser-targeted analysis/profile (no standalone DOM runtime implied)
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
By default, `kali effects` prints the effect report payload directly because JSON is the primary output of the command. With `--output json`, it is wrapped in the standard command envelope described below. See [specs/09-sandboxing.md](09-sandboxing.md) for the payload schema.

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
```

### `kali init`
Initialize a new project.
```bash
kali init                                  # Create kali.json in current dir
kali init --lib                            # Library project template
```

### `kali install [package]`
Install npm/JSR packages.
```bash
kali install lodash                        # Install from npm
kali install                               # Install all dependencies from kali.json
kali install --dev vitest                  # Dev dependency
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
Minimal — one line or nothing. For commands intended for automation, prefer either no success output or a single deterministic line. Human-friendly decoration belongs behind `--verbose`, not in the default contract:
```
$ kali check main.ts
No errors

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
- common optional top-level fields include `artifacts`, `stdout`, and `timings`

Exception: `kali effects` already emits JSON as its native output, so `--output json` wraps that payload in the envelope instead of changing the effect-report schema itself.

## Configuration (`kali.json`)

```json
{
    "compilerOptions": {
        "strict": true,
        "api": "deno",
        "buildMode": "fast",
        "maxSpecializations": 16
    },
    "compat": {
        "features": []
    },
    "sandbox": "./kali.policy.json",
    "include": ["src/**/*.ts"],
    "exclude": ["**/*.test.ts"],
    "imports": {
        "std/": "https://deno.land/std@0.220.0/",
        "~/": "./src/"
    },
    "dependencies": {
        "lodash": "^4.17.21"
    },
    "devDependencies": {
        "vitest": "^1.0.0"
    }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Compilation error (type error, syntax error) |
| 2 | Runtime error (uncaught exception) |
| 3 | Sandbox violation |
| 4 | Resource limit exceeded |
| 5 | Configuration error |
| 126 | Permission denied |
| 127 | File not found |
