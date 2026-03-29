# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is one statically linked executable
4. **Zero config**: Sensible defaults, explicit configuration when needed

## Global Flags

Available on all subcommands:

| Flag | Description |
|------|-------------|
| `--verbose` | Detailed output: timing per phase, optimization decisions |
| `--output json` | Machine-parseable JSON output for all commands |
| `--quiet` | Suppress all non-error output |
| `--max-errors N` | Cap reported errors (default: 50) |
| `--color auto\|always\|never` | Color output control |

## Commands

### `kali run <file>`
Compile and execute a TypeScript/JavaScript file.
```bash
kali run main.ts                           # Run with default settings
kali run --sandbox policy.ts main.ts       # Run with sandbox
kali run --max-memory 256mb main.ts        # Resource limit
kali run --max-cpu 10s main.ts             # CPU time limit
kali run --api node main.ts                # Use Node.js API surface
kali run --api deno main.ts                # Use Deno API surface (default)
kali run --runtime wasmer main.ts          # Use wasmer instead of wasmtime
```

### `kali build <file>`
AOT compile to a WASM module.
```bash
kali build main.ts                         # → main.wasm (--fast mode, default)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle main.ts                # WASM + JS glue for browsers
kali build --lib lib.ts                    # Library module (exports, no start)
kali build --capi                          # C API: libkali.a + libkali.so + kali.h
kali build --sandbox policy.ts main.ts     # Validate sandbox policy at compile time
kali build --validate-ir main.ts           # Run IR validators (debug aid)
kali build --max-specializations 32 main.ts # Override specialization cap
```

### `kali check <file>`
Type-check without compiling.
```bash
kali check main.ts                         # Type check
kali check --sandbox policy.ts main.ts     # Type check + sandbox policy validation
kali check --fix main.ts                   # Type check + auto-apply suggested fixes
```

### `kali effects <file>`
Output static effect analysis as JSON.
```bash
kali effects main.ts                       # Compact JSON to stdout
kali effects --pretty main.ts              # Pretty-printed JSON
```
See [specs/09-sandboxing.md](09-sandboxing.md) for JSON schema.

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
kali test --sandbox policy.ts              # Run tests in sandbox
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
Minimal — one line or nothing:
```
$ kali check main.ts
✓ No errors

$ kali build main.ts
✓ main.wasm (142 KB, 23ms)

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
Machine-parseable output for all commands:
```json
{
    "success": false,
    "errors": [
        {
            "code": "E1001",
            "message": "Type 'string' is not assignable to type 'number'",
            "file": "main.ts",
            "line": 5,
            "column": 10,
            "endLine": 5,
            "endColumn": 17
        }
    ],
    "warnings": []
}
```

## Configuration (`kali.json`)

```json
{
    "compilerOptions": {
        "strict": true,
        "api": "deno",
        "optimizationLevel": "fast",
        "maxSpecializations": 16
    },
    "sandbox": "./sandbox.policy.ts",
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
