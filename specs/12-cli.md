# 12 — CLI

## Design Principles

1. **AI-agent optimized**: Concise output by default, verbose with `--verbose`
2. **Deno-inspired**: Familiar subcommand structure
3. **Single binary**: `kali` is one statically linked executable
4. **Zero config**: Sensible defaults, explicit configuration when needed

## Commands

### `kali run <file>`
Compile and execute a TypeScript/JavaScript file.
```bash
kali run main.ts                           # Run with default settings
kali run --sandbox policy.ts main.ts       # Run with sandbox
kali run --max-memory 256mb main.ts        # Resource limit
kali run --api node main.ts                # Use Node.js API surface
```

### `kali build <file>`
AOT compile to a WASM module.
```bash
kali build main.ts                         # → main.wasm (--fast mode)
kali build --release main.ts               # Optimized build
kali build --release-advanced main.ts      # Aggressively optimized
kali build --bundle main.ts                # WASM + JS glue
kali build --lib lib.ts                    # Library (no entry point)
```

### `kali check <file>`
Type-check without compiling.
```bash
kali check main.ts                         # Type check
kali check --sandbox policy.ts main.ts     # Type check + sandbox validation
```

### `kali effects <file>`
Output static effect analysis as JSON.
```bash
kali effects main.ts                       # JSON to stdout
kali effects --pretty main.ts              # Pretty-printed JSON
```

### `kali fmt [files...]`
Format source files.
```bash
kali fmt                                   # Format all .ts/.js in project
kali fmt --check                           # Check formatting (CI mode)
kali fmt main.ts                           # Format specific file
```

### `kali lint [files...]`
Lint source files.
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
```

### `kali init`
Initialize a new project.
```bash
kali init                                  # Create kali.json in current dir
kali init --lib                            # Library project template
```

### `kali install <package>`
Install npm packages.
```bash
kali install lodash                        # Install from npm
kali install                               # Install all dependencies from kali.json
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
Structured, parseable, concise:
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
        "target": "wasm",
        "strict": true,
        "api": "deno",
        "optimizationLevel": "release"
    },
    "sandbox": "./sandbox.policy.ts",
    "include": ["src/**/*.ts"],
    "exclude": ["**/*.test.ts"],
    "dependencies": {
        "lodash": "^4.17.21"
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
