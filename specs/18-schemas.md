# 18 — Schemas

## Purpose

Kali emits several machine-consumed JSON formats:
- CLI command envelopes
- diagnostics
- effect reports
- sandbox policies

To keep the specs consistent and AI-friendly, these formats are centralized here instead of being redefined independently in multiple chapters.

## Versioning Rules

- Every top-level machine-readable document carries `schemaVersion`
- Schema changes that break old consumers require a version bump
- Additive changes within a version are allowed only for optional fields
- CLI text output is human-oriented; JSON output is the stable tooling contract

## CLI Command Envelope

Used by commands that opt into `--output json`.

```json
{
  "schemaVersion": 1,
  "command": "check",
  "success": false,
  "errors": [],
  "warnings": [],
  "payload": null
}
```

### Required fields
- `schemaVersion: number`
- `command: string`
- `success: boolean`
- `errors: Diagnostic[]`
- `warnings: Diagnostic[]`

### Optional fields
- `payload: object | array | string | number | boolean | null`
- `artifacts: Artifact[]`
- `stdout: string`
- `timings: PhaseTiming[]`

### Notes
- `payload` holds command-specific structured data
- `kali effects` may emit the raw effect report by default, but with `--output json` it must be wrapped in this envelope
- Commands should avoid inventing top-level ad hoc fields when `payload` is sufficient

## Common Source Location Types

Kali uses two related but distinct span concepts:
- **Internal compiler `Span`**: compact byte-offset range + file ID, used in the parser/AST/IR for speed.
- **JSON `SourceSpan`**: human/tool-facing line/column range, derived from the internal span when emitting diagnostics, effect reports, stack traces, or other schemas.

This distinction prevents drift between the implementation-oriented frontend specs and the machine-readable CLI/output specs.

### `SourceLocation`

```json
{
  "file": "src/main.ts",
  "line": 5,
  "column": 10
}
```

Required fields:
- `file: string`
- `line: number` *(1-based)*
- `column: number` *(1-based)*

### `SourceSpan`

```json
{
  "file": "src/main.ts",
  "line": 5,
  "column": 10,
  "endLine": 5,
  "endColumn": 17
}
```

Required fields:
- `file: string`
- `line: number` *(1-based)*
- `column: number` *(1-based)*
- `endLine: number` *(1-based, inclusive line of end position)*
- `endColumn: number` *(1-based, exclusive column of end position)*

## Diagnostic Schema

```json
{
  "severity": "error",
  "code": "E1001",
  "message": "Type 'string' is not assignable to type 'number'",
  "file": "src/main.ts",
  "span": {
    "file": "src/main.ts",
    "line": 5,
    "column": 10,
    "endLine": 5,
    "endColumn": 17
  },
  "labels": [],
  "help": "Remove the type annotation or change the value",
  "related": [],
  "fix": null,
  "notes": []
}
```

### Required fields
- `severity: "error" | "warning" | "info" | "hint"`
- `code: string`
- `message: string`
- `span: SourceSpan`
- `labels: Label[]`

### Optional fields
- `file: string` *(convenience mirror of `span.file` for shallow consumers)*
- `help: string`
- `related: RelatedInfo[]`
- `fix: SuggestedFix`
- `notes: string[]`

## Reusable Supporting Types

These types are referenced by the envelope and diagnostic schemas above and should not be redefined ad hoc elsewhere.

### `Label`

```json
{
  "span": {
    "file": "src/main.ts",
    "line": 5,
    "column": 10,
    "endLine": 5,
    "endColumn": 17
  },
  "message": "expected 'number', found 'string'",
  "style": "primary"
}
```

Required fields:
- `span: SourceSpan`
- `message: string`
- `style: "primary" | "secondary"`

### `RelatedInfo`

```json
{
  "message": "variable declared here",
  "span": {
    "file": "src/main.ts",
    "line": 2,
    "column": 7,
    "endLine": 2,
    "endColumn": 8
  }
}
```

Required fields:
- `message: string`
- `span: SourceSpan`

### `TextEdit`

```json
{
  "file": "src/main.ts",
  "start": { "file": "src/main.ts", "line": 5, "column": 10 },
  "end": { "file": "src/main.ts", "line": 5, "column": 17 },
  "newText": "42"
}
```

Required fields:
- `file: string`
- `start: SourceLocation`
- `end: SourceLocation`
- `newText: string`

Semantics:
- `start` is inclusive and `end` is exclusive
- `start.file` and `end.file` must match `file`
- edits inside one `SuggestedFix` must be non-overlapping

### `SuggestedFix`

```json
{
  "message": "Replace the string literal with a number",
  "edits": []
}
```

Required fields:
- `message: string`
- `edits: TextEdit[]`

### `PhaseTiming`

```json
{
  "phase": "typecheck",
  "milliseconds": 12.4
}
```

Required fields:
- `phase: string`
- `milliseconds: number`

### `Artifact`

```json
{
  "path": "main.wasm",
  "kind": "wasm-module",
  "bytes": 145408
}
```

Required fields:
- `path: string`
- `kind: string`
- `bytes: number`

Optional fields:
- `role: string` — command-specific role such as `primary`, `glue`, or `debug`

## Effect Report Schema

Produced by `kali effects`.

```json
{
  "schemaVersion": 1,
  "entryPoints": ["main"],
  "effects": [
    {
      "kind": "FileSystem.Read",
      "locations": [
        {
          "file": "program.ts",
          "line": 2,
          "column": 18,
          "function": "processFile"
        }
      ]
    }
  ],
  "dynamicEffects": false,
  "usesEval": false
}
```

### Required fields
- `schemaVersion: number`
- `entryPoints: string[]`
- `effects: EffectOccurrence[]`
- `dynamicEffects: boolean`
- `usesEval: boolean`

### `EffectOccurrence`
```json
{
  "kind": "FileSystem.Read",
  "locations": [
    {
      "file": "program.ts",
      "line": 2,
      "column": 18,
      "function": "processFile"
    }
  ]
}
```

Required fields:
- `kind: string`
- `locations: EffectLocation[]`

### `EffectLocation`
Uses the `SourceLocation` shape plus optional effect-specific context.

Optional fields:
- `function: string` — nearest enclosing function or method name when available

### Semantics
- `dynamicEffects: true` means the report is conservative but incomplete
- `usesEval: true` implies `dynamicEffects: true`
- Effect `kind` names must match the canonical built-in names derived from the type system and sandbox policy model
- Phase 1-2 effect reports are limited to built-in sandbox-relevant effect kinds; later experimental user-defined effects, if exposed, should use a reserved `Custom.<name>` namespace rather than overloading built-in policy keys
- Effect locations use `SourceLocation` fields and the same 1-based `line` / `column` convention as diagnostics so tools do not need separate coordinate systems for errors vs effect reports
- If a consumer needs a full range instead of a point location, it should use the same `SourceSpan` shape rather than inventing a command-specific span format

## Canonical Built-in Effect Names

To keep the checker, CLI, effect reports, and sandbox policy model aligned, built-in effect names use one canonical dotted namespace.

| Effect report / checker name | Policy key |
|---|---|
| `FileSystem.Read` | `effects.fileSystem.read` |
| `FileSystem.Write` | `effects.fileSystem.write` |
| `Network.Fetch` | `effects.network.fetch` |
| `Network.Connect` | `effects.network.connect` |
| `Network.Listen` | `effects.network.listen` |
| `Process.Spawn` | `effects.process.spawn` |
| `Process.EnvRead` | `effects.process.envRead` |
| `Process.EnvWrite` | `effects.process.envWrite` |
| `Timer.Schedule` | `effects.timer.schedule` |
| `Random.GetBytes` | `effects.random` |
| `Console.Write` | `effects.console` |
| `Eval` | `effects.eval` |

Rules:
- Phase 1-2 stable machine-readable contracts are limited to these built-in names.
- Later experimental user-defined effects, if exposed, must use the reserved `Custom.<name>` namespace.
- Coarse policy keys may match a namespace prefix. In schema v1, `effects.random` matches any `Random.*` built-in effect, and `effects.console` matches any `Console.*` built-in effect.
- New built-in effect names must be added here before they appear in diagnostics, effect reports, or policy examples elsewhere in the spec set.

## Project Configuration Schema

Canonical filename: `kali.json`

```json
{
  "$schema": "https://kali.sh/schemas/config-v1.json",
  "compilerOptions": {
    "strict": true,
    "apiSurface": "deno",
    "buildMode": "fast",
    "runtimeProfiles": [],
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

### Rules
- `compilerOptions.apiSurface` is the canonical config name for the host API family; CLI uses `--api`
- `compilerOptions.buildMode` is one of `fast`, `release`, or `release-advanced`
- `compilerOptions.runtimeProfiles` is an array of semantic runtime-profile names; in schema v1 it is usually empty because later profiles such as `wasm-threads` are still phase-gated
- `dependencies` and `devDependencies` are top-level package manifests owned by `kali install`; they are not nested under `compilerOptions`
- Config should not mirror every CLI boolean directly when a more semantic field already exists
- `compilerOptions.api` may be accepted as a deprecated alias for migration, but tools should emit `apiSurface`
- Precedence is `CLI > kali.json > defaults`, except sandbox policy restrictions still bound effective runtime behavior
- Unknown top-level config fields should be diagnosed unless reserved for a documented extension mechanism

## Sandbox Policy Schema

Canonical filename: `kali.policy.json`

```json
{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": ["/data/**"], "write": false },
    "network": {
      "fetch": ["https://api.example.com/*"],
      "connect": false,
      "listen": false,
      "maxConnections": 16
    },
    "process": {
      "spawn": false,
      "envRead": ["PATH", "HOME"],
      "envWrite": false
    },
    "timer": {
      "schedule": true,
      "maxTimeoutMs": 5000,
      "maxActiveTimers": 32
    },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 10,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}
```

### Rules
- Policies are declarative data, not executable code
- Unknown fields are rejected to keep policy evaluation deterministic and auditable
- Policy booleans mean fully allowed or fully denied for that capability
- Pattern-bearing fields (`read`, `fetch`) are allowlists
- Numeric limit fields constrain otherwise-allowed capabilities; for example `timer.schedule: true` with `maxActiveTimers: 32` allows timers but caps concurrency
- `resources.maxOpenFiles` caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- `resources.maxSpawnedProcesses` caps concurrently active spawned processes
- `resources.maxThreads` is reserved for the later threaded runtime profile; before that profile exists, validation should reject values greater than `0` instead of silently accepting them
- Per-invocation CLI resource overrides may only tighten these policy limits; they must not widen them
- Policy keys use the canonical built-in effect naming table above rather than redefining a separate namespace here
- In schema v1, `random` and `console` are intentionally coarse-grained booleans. Any built-in effect report entry whose kind starts with `Random.` matches `random`, and any kind starting with `Console.` matches `console`.
- Later experimental user-defined effects are outside the policy schema unless/until a future spec revision adds an explicit extension point

## Artifact Schema

For build-like commands. This is the canonical meaning of the reusable `Artifact` type above.

Common `kind` values:
- `wasm-module`
- `js-glue`
- `c-header`
- `cabi-metadata`
- `source-map`

## Simplification Rule

If a schema needs more than one example across the spec set, the canonical structure belongs in this file and other specs should link here instead of duplicating the full object shape.

Additional simplification rule for diagnostics: `span` is the canonical source-range field. Any top-level `file` mirror is optional convenience data and must not diverge from `span.file`.
