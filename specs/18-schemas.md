# 18 — Schemas

## Purpose

Kali emits several machine-consumed JSON formats:
- CLI command envelopes
- diagnostics
- effect reports
- project configuration (`kali.json`)
- sandbox policies

To keep the specs consistent and AI-friendly, these formats are centralized here instead of being redefined independently in multiple chapters.

## Versioning Rules

- Every top-level machine-readable **JSON** document carries `schemaVersion`
- Non-JSON machine-readable artifacts version themselves using their own canonical mechanism (for example `kali.lock v1` in the lockfile header)
- Schema changes that break old consumers require a version bump
- Additive changes within a version are allowed only for optional fields
- For enum-like machine strings (for example canonical `dynamicReasons`, stable effect names, or artifact `kind` values documented here), introducing new stable values requires a schema-version bump unless the field is explicitly documented as open-ended
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
- `exitCode: number` — canonical process exit code for the command invocation when the caller needs it in-band

### Notes
- `payload` holds command-specific structured data
- `command` is intentionally an open-ended string so new CLI subcommands do not force a schema-version bump; stable built-in command names should mirror the CLI subcommand path in kebab-case (for example `check`, `build`, `package-effects`)
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
  "dynamicReasons": []
}
```

### Required fields
- `schemaVersion: number`
- `entryPoints: string[]` — logical program entry names analyzed for this report (for example `main`, discovered test entrypoints, or exported embedding entry names)
- `effects: EffectOccurrence[]`
- `dynamicEffects: boolean`
- `dynamicReasons: string[]` — canonical reason codes explaining why the report is conservative/incomplete; empty when `dynamicEffects` is `false`

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

Simplification rule:
- schema v1 uses point locations for effect occurrences by default so effect reports stay compact for AI/tooling use
- if a future consumer needs full ranges, it should reuse `SourceSpan` rather than inventing a second effect-specific span schema

### Semantics
- `dynamicEffects: true` means the report is conservative but incomplete
- `dynamicReasons` uses canonical reason strings so tools do not have to infer *why* the report became conservative from free-form notes alone
- Schema v1 reason strings are: `eval`, `function-constructor`, `dynamic-import`, `proxy-traps`, and `computed-host-access`
- Because `dynamicReasons` is a stable machine contract, adding new canonical reason strings requires a schema-version bump
- `dynamicReasons` must be empty when `dynamicEffects` is `false`
- If `dynamicReasons` contains `eval` or `function-constructor`, the report should also include the built-in `Eval` effect in `effects`
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
- Adding a new stable built-in effect name that can appear in machine-readable output is a schema-contract change and should be accompanied by a schema-version review.

## Project Configuration Schema

Canonical filename: `kali.json`

```json
{
  "schemaVersion": 1,
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
- `schemaVersion: number` is required on `kali.json` like every other top-level machine-readable Kali JSON document
- `compilerOptions.apiSurface` is the canonical config name for the host API family; CLI uses `--api`
- `compilerOptions.buildMode` is one of `fast`, `release`, or `release-advanced`
- `compilerOptions.runtimeProfiles` is an array of semantic runtime-profile names; in schema v1 it is usually empty because later profiles such as `wasm-threads` are still phase-gated
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- `compilerOptions.strict` is the canonical strict-checking bundle switch in config; early phases should avoid multiplying near-duplicate strictness booleans unless a later schema revision documents them explicitly
- `compilerOptions.maxSpecializations` is the project-default specialization cap; CLI `--max-specializations` may override it per invocation
- top-level `sandbox` is an optional default sandbox-policy path; it is the config equivalent of supplying `--sandbox <path>` for commands that honor sandboxing, and an explicit CLI flag overrides it
- `compat.features` is the config equivalent of CLI `--compat`; entries use the same canonical feature names, are order-insensitive, and should be unique
- when set-like arrays such as `compilerOptions.runtimeProfiles` or `compat.features` are normalized by tooling, normalization should preserve semantics without inventing duplicates; preserving first-seen order for display/diff stability is preferred even though the arrays are semantically unordered
- `include` / `exclude` define project file discovery globs for project-oriented commands and editor/tooling integrations; they do not reinterpret an explicit CLI file argument as a different entry point
- `imports` is the canonical alias/import-map section for URL and path-like rewrites; it is not a second registry-dependency manifest
- `dependencies` and `devDependencies` are top-level package manifests for **registry packages** owned by `kali install`; they are not nested under `compilerOptions`
- raw URL dependencies are declared in source/import maps and tracked via `kali.lock`; schema v1 intentionally does **not** add a second manifest section for them
- Config should not mirror every CLI boolean directly when a more semantic field already exists
- Precedence is `CLI > kali.json > defaults`, except sandbox policy restrictions still bound effective runtime behavior
- Unknown config fields are rejected at every documented nesting level unless a future schema revision adds an explicit extension mechanism

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
- Unknown fields are rejected at every documented nesting level to keep policy evaluation deterministic and auditable
- Policy booleans mean fully allowed or fully denied for that capability
- Pattern-bearing fields (`read`, `write`, `fetch`, `connect`, `listen`) are allowlists when they take arrays
- Numeric limit fields inside `effects.*` constrain an otherwise-allowed capability locally; for example `timer.schedule: true` with `maxActiveTimers: 32` allows timers but caps timer concurrency
- `resources.*` is reserved for cross-cutting runtime budgets rather than capability-specific allowlists/caps
- `resources.maxOpenFiles` caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- `resources.maxSpawnedProcesses` caps concurrently active spawned processes once subprocess APIs exist; before then, validation should reject values greater than `0` instead of accepting a non-functional budget for an unavailable capability
- `resources.maxThreads` is reserved for the later threaded runtime profile; before that profile exists, validation should reject values greater than `0` instead of silently accepting them
- Policy validation should reject non-deny values for capability fields whose corresponding feature/API surface is unavailable in the selected command/profile/phase. For example: `effects.eval: true` before the eval compatibility path exists, `effects.process.spawn: true` before subprocess APIs exist, `effects.process.envWrite: true` before mutable environment APIs exist, and `resources.maxSpawnedProcesses > 0` before subprocess APIs exist.
- Per-invocation CLI resource overrides may only tighten these policy limits; they must not widen them
- Policy keys use the canonical built-in effect naming table above rather than redefining a separate namespace here
- In schema v1, `random` and `console` are intentionally coarse-grained booleans. Any built-in effect report entry whose kind starts with `Random.` matches `random`, and any kind starting with `Console.` matches `console`.
- Later experimental user-defined effects are outside the policy schema unless/until a future spec revision adds an explicit extension point

### Canonical Capability Field Shapes (schema v1)

To keep policy examples, validators, and runtime checks consistent, schema v1 uses these canonical value shapes:

| Policy field | Allowed shape(s) | Meaning |
|---|---|---|
| `effects.fileSystem.read` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching path patterns |
| `effects.fileSystem.write` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching path patterns |
| `effects.network.fetch` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching URL patterns |
| `effects.network.connect` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching outbound address/URL patterns |
| `effects.network.listen` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching bind address/port patterns |
| `effects.process.spawn` | `false` \| `true` \| `string[]` | deny all, allow all, or allow only matching executable/command patterns |
| `effects.process.envRead` | `false` \| `true` \| `string[]` | deny all, allow all environment reads, or allow only named variables |
| `effects.process.envWrite` | `false` \| `true` \| `string[]` | deny all, allow all environment writes, or allow only named variables |
| `effects.timer.schedule` | `boolean` | enable or disable timer creation |
| `effects.timer.maxTimeoutMs` | `number` | maximum allowed timeout/interval delay |
| `effects.timer.maxActiveTimers` | `number` | maximum concurrently active timers |
| `effects.network.maxConnections` | `number` | maximum concurrent outbound/inbound network connections |
| `effects.eval` | `boolean` | allow or deny `Eval` capability |
| `effects.random` | `boolean` | allow or deny `Random.*` capability family |
| `effects.console` | `boolean` | allow or deny `Console.*` capability family |

Interpretation rules:
- `true` means unrestricted for that capability within schema v1, subject to separate `resources.*` caps.
- `string[]` means an allowlist; an empty array therefore denies all practical uses of that capability.
- Field-specific arrays use canonical matching domains: filesystem paths for file APIs, URLs/addresses for network APIs, executable names/paths for process spawning, and exact environment-variable names for env access.
- Specs and examples should reuse these shapes instead of inventing per-command variants.

## Coverage Reporting Status

Coverage output is intentionally absent from schema v1.

Interpretation rule:
- `kali test --coverage` is a Phase 2 target because it needs its own stable machine-readable contract
- until that contract exists, docs and implementations must not imply that ad hoc text output is the canonical coverage format
- when coverage lands, its schema belongs in this file rather than being defined informally in the testing or CLI chapters

## Artifact Schema

For build-like commands. This is the canonical meaning of the reusable `Artifact` type above.

Canonical schema-v1 `kind` values:
- `wasm-module`
- `js-glue`
- `c-header`
- `cabi-metadata`
- `source-map`

Simplification rule:
- build-like commands should use these canonical artifact kinds instead of inventing near-synonyms such as `wasm`, `header`, or `metadata-json`
- adding a new stable artifact `kind` value is a schema-contract change and should get the same review discipline as other enum-like machine strings in this file

## Simplification Rule

If a schema needs more than one example across the spec set, the canonical structure belongs in this file and other specs should link here instead of duplicating the full object shape.

Additional simplification rule for diagnostics: `span` is the canonical source-range field. Any top-level `file` mirror is optional convenience data and must not diverge from `span.file`.
