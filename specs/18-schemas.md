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
- `file: string`
- `span: SourceSpan`
- `labels: Label[]`

### Optional fields
- `help: string`
- `related: RelatedInfo[]`
- `fix: SuggestedFix`
- `notes: string[]`

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

### Semantics
- `dynamicEffects: true` means the report is conservative but incomplete
- `usesEval: true` implies `dynamicEffects: true`
- Effect `kind` names must match the canonical names derived from the type system and sandbox policy model
- Effect locations use `SourceLocation` fields and the same 1-based `line` / `column` convention as diagnostics so tools do not need separate coordinate systems for errors vs effect reports
- If a consumer needs a full range instead of a point location, it should use the same `SourceSpan` shape rather than inventing a command-specific span format

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
- Policy keys map to canonical effect kinds as follows: `fileSystem.read` ↔ `FileSystem.Read`, `fileSystem.write` ↔ `FileSystem.Write`, `network.fetch` ↔ `Network.Fetch`, `network.connect` ↔ `Network.Connect`, `network.listen` ↔ `Network.Listen`, `process.spawn` ↔ `Process.Spawn`, `process.envRead` ↔ `Process.EnvRead`, `process.envWrite` ↔ `Process.EnvWrite`, `timer.schedule` ↔ `Timer.Schedule`, `random` ↔ `Random.*`, `console` ↔ `Console.*`, `eval` ↔ `Eval`

## Artifact Schema

For build-like commands.

```json
{
  "path": "main.wasm",
  "kind": "wasm-module",
  "bytes": 145408
}
```

Common `kind` values:
- `wasm-module`
- `js-glue`
- `c-header`
- `cabi-metadata`
- `source-map`

## Simplification Rule

If a schema needs more than one example across the spec set, the canonical structure belongs in this file and other specs should link here instead of duplicating the full object shape.
