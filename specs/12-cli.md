# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is distributed as one primary executable; static linking is preferred where practical but not required on every target
4. **Zero config**: Sensible defaults, explicit configuration when needed
5. **Stable machine contract**: JSON output is versioned and remains backward-compatible across minor releases

## Global Flags

Available on all subcommands:

| Flag | Description |
|------|-------------|
| `--verbose` | Detailed output: timing per phase, optimization decisions |
| `--output json` | Machine-parseable JSON output for all commands |
| `--api deno\|node\|browser` | Select host API surface |
| `--quiet` | Suppress all non-error output |
| `--max-errors N` | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | Color output control |

## Commands

### `kali run <file>`
Compile and execute a TypeScript/JavaScript file.
```bash
kali run main.ts                           # Run with default settings
kali run --sandbox kali.policy.json main.ts # Run with sandbox
kali run --max-memory 256mb main.ts        # Resource limit
kali run --max-cpu 10s main.ts             # CPU time limit
kali run --api node main.ts                # Use Node.js API surface
kali run --api deno main.ts                # Use Deno API surface (default)
kali run --api browser main.ts             # Use browser API surface (Web Platform APIs)
kali run --wasm-threads main.ts            # Enable WASM threads (SharedArrayBuffer, Atomics)
```

Initial implementations use wasmtime; alternative runtime backends are a later-phase feature.

### `kali build <file>`
AOT compile to a WASM module.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle main.ts                # WASM + JS glue for browsers
kali build --lib lib.ts                    # Library module (exports, no start)
kali build --capi lib.ts                   # C API-compatible library artifact + kali.h metadata (see specs/13-embedding.md)
kali build --sandbox kali.policy.json main.ts # Validate sandbox policy at compile time
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

### `kali check <file>`
Type-check without compiling.
```bash
kali check main.ts                         # Type check
kali check --sandbox kali.policy.json main.ts # Type check + sandbox policy validation
kali check --fix main.ts                   # Apply only safe, compiler-provided suggested fixes
```
`--fix` is intentionally conservative: it is limited to unambiguous structured edits attached to diagnostics, not arbitrary refactors or speculative type rewrites.

### `kali effects <file>`
Output static effect analysis as JSON.
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
```bash
kali package-effects lodash                # Show effects used by package (JSON)
```

### `kali package-audit [package]`
Security audit for dependencies.
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
        "release": false,
        "releaseAdvanced": false,
        "maxSpecializations": 16
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
