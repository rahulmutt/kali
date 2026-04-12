# Stage 1.13 — Diagnostics & Schemas

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/15-errors.md`](../../specs/15-errors.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/12-cli.md`](../../specs/12-cli.md)  
**Depends on:** [1.12 — Developer Workflow](12-developer-workflow.md) (all Phase-1 commands must exist before their JSON schemas are finalised)

## Goal

Finalise the human-readable and machine-readable diagnostic/output surfaces: stable error codes
with AI-friendly messages, `--output json` mode emitting schema-v1 JSON envelopes across all
commands, versioned artifact metadata schemas, and the `kali.json` config schema.

## Workable Milestone

- Every `kali` subcommand accepts `--output json` and emits a schema-v1 JSON envelope.
- All shipped `E1xxx`–`E9xxx` error codes have stable, concise, actionable messages.
- The schema-v1 JSON shapes for commands, diagnostics, artifacts, manifests, lock files, and
  policies are documented and validated against JSON Schema documents.

## Progress

- `kali --output json` now works across the shipped Phase-1 command surface (`check`, `build`, `run`, `test`, `init`, `install`, `fmt`, `lint`) and emits a single schema-v1 envelope instead of interleaving raw text with machine output.
- Runtime execution now captures guest stdout/stderr so `run` and `test` can surface program streams through the JSON envelope as well as the human CLI path.
- Added regression coverage for JSON-mode `init` and `check` envelopes so the command metadata and payload fields stay deterministic.
- Committed the repository `schemas/` documents for the command envelope, diagnostics, manifests, lockfiles, policies, artifact metadata, and the current shipped result payloads, including reserved later-phase shapes.
- Tightened the default human diagnostic renderer so severity labels and help/note lines follow the canonical `error[...]` / `= help:` / `= note:` style.

## Tasks

### 1. Human-readable diagnostic format

Finalise the default (non-JSON) diagnostic presentation in `kali_error`:

```
error[E3021]: type 'string' is not assignable to type 'number'
  --> src/main.ts:12:5
   |
12 |     const x: number = "hello";
   |                       ^^^^^^^ expected 'number', found 'string'
   |
   = note: strict null checks are enabled
```

Rules (from `specs/15-errors.md`):

- One error per diagnostic block; secondary labels may annotate related spans.
- The primary span is underlined with `^`; secondary spans use `-`.
- Error code in brackets (`[E3021]`); includes a link to the docs URL when `--verbose` is on.
- Notes (prefixed `= note:`) for additional context without repeating the primary message.
- **Concise by default**: one block per error, no multi-paragraph walls of text.
- **AI-friendly**: the primary message is a single short sentence that can be quoted verbatim
  in an AI tool response; secondary context lives in notes.

Severity levels: `error`, `warning`, `note`, `help` (with auto-fix hint).

### 2. Schema-v1 JSON envelope

When any `kali` command is invoked with `--output json`, the entire output (diagnostics, results,
and any captured program streams) is emitted through a single JSON envelope to stdout. No raw
text is interleaved.

Top-level envelope shape:

```json
{
  "$schema": "https://kali-lang.org/schemas/envelope/v1",
  "schemaVersion": 1,
  "command": "check",
  "exitCode": 1,
  "diagnostics": [...],
  "result": null,
  "programStdout": null,
  "programStderr": null
}
```

- `diagnostics`: array of structured diagnostic objects (see below).
- `result`: command-specific result payload (e.g. artifact paths for `build`, test counts for `test`).
- `programStdout` / `programStderr`: captured program output for `run` / `test`; null for other commands.
- `exitCode`: the exit code the process will use; included in the envelope so tools can read it
  without inspecting the process exit.

**Single-channel rule:** when `--output json` is active, all output goes through the envelope.
Diagnostic text, program output, and result metadata are never interleaved on raw stdout/stderr.

### 3. Structured diagnostic object

```json
{
  "code": "E3021",
  "severity": "error",
  "message": "type 'string' is not assignable to type 'number'",
  "spans": [
    {
      "file": "src/main.ts",
      "startLine": 12,
      "startColumn": 5,
      "endLine": 12,
      "endColumn": 12,
      "label": "expected 'number', found 'string'",
      "isPrimary": true
    }
  ],
  "notes": ["strict null checks are enabled"],
  "helpUrl": "https://kali-lang.org/errors/E3021"
}
```

The `SourceSpan` / `SourceLocation` shapes used here are the schema-v1 translations of the
internal `Span` type from `kali_common` (as specified in `specs/01-architecture.md`).

### 4. Command-specific result payloads

Define schema-v1 result shapes for each shipped command:

**`kali check` result:**

```json
{ "filesChecked": 5, "errorCount": 2, "warningCount": 1 }
```

**`kali build` result:**

```json
{
  "artifactKind": "executable",
  "outputPath": "dist/main.wasm",
  "sizeBytes": 123456,
  "buildMode": "fast",
  "sourceHash": "sha256-..."
}
```

**`kali run` result:**

```json
{ "exitCode": 0, "runtimeMs": 123 }
```

**`kali test` result:**

```json
{
  "total": 10,
  "passed": 9,
  "failed": 1,
  "skipped": 0,
  "failures": [
    {
      "name": "addition works",
      "file": "src/math.test.ts",
      "message": "Expected 5, got 4",
      "spans": [...]
    }
  ]
}
```

**`kali install` result:**

```json
{
  "installed": ["lodash@4.17.21"],
  "updated": [],
  "removed": []
}
```

**`kali fmt` result:**

```json
{ "filesFormatted": 3, "filesChecked": 5 }
```

**`kali lint` result:**

```json
{ "filesLinted": 5, "errorCount": 0, "warningCount": 2, "fixedCount": 1 }
```

### 5. Schema-v1 JSON Schema documents

Publish machine-readable JSON Schema (draft 2020-12) documents for every schema-v1 shape:

| Schema document | Covers |
|---|---|
| `schemas/envelope/v1.json` | Top-level JSON envelope |
| `schemas/diagnostic/v1.json` | Structured diagnostic object |
| `schemas/manifest/v1.json` | `kali.json` project manifest |
| `schemas/lock/v1.json` | `kali.lock` lock file |
| `schemas/policy/v1.json` | Sandbox policy file |
| `schemas/artifact-meta/v1.json` | WASM artifact metadata |
| `schemas/result/check/v1.json` | `kali check` result |
| `schemas/result/build/v1.json` | `kali build` result |
| `schemas/result/run/v1.json` | `kali run` result |
| `schemas/result/test/v1.json` | `kali test` result |
| `schemas/result/install/v1.json` | `kali install` result |
| `schemas/result/fmt/v1.json` | `kali fmt` result |
| `schemas/result/lint/v1.json` | `kali lint` result |

These schema documents live in the repository under `schemas/` and are embedded into the binary
for the `--output json` mode validation step.

### 6. Reserved schema shapes for later commands

Following the **Phase Contracts vs Implementation Order** rule from `SPEC.md`, define the stable
JSON shapes for later commands now so naming doesn't drift:

- `schemas/result/effects/v1.json` — `kali effects` result (Phase 2 target; native-JSON command).
- `schemas/result/package-effects/v1.json` — `kali package-effects` result (Phase 2 target).
- `schemas/result/package-audit/v1.json` — `kali package-audit` result (Later compatibility;
  envelope-only JSON command).
- `schemas/artifact-meta/lib-wit/v1.json` — `kali build --lib` with WIT sidecar (Phase 2 target).
- `schemas/artifact-meta/capi/v1.json` — `kali build --capi` (Phase 2 target).
- `schemas/artifact-meta/component/v1.json` — `kali build --component` (Phase 2 target).

Documenting these shapes early prevents naming drift. Their actual CLI availability still follows
`specs/19-feature-maturity.md`.

### 7. `--verbose` and `--quiet` flags

- `--verbose`: include additional context in human output (docs links, notes expanded, internal
  timing); in JSON mode include a `"verbose"` key with extended fields.
- `--quiet`: suppress all output except errors; useful in CI pipelines.
- `--color` / `--no-color`: ANSI colour in human output; detect terminal vs pipe automatically.

These are shared presentation/control flags available on all commands.

### 8. Exit-code contract

| Situation | Exit code |
|---|---|
| Success (no errors) | 0 |
| Diagnostic errors present | 1 |
| Command-shape / argument error (`E5xxx`) | 2 |
| Internal compiler error / panic | 3 |
| Sandbox policy violation at runtime | 1 (via `E4004`) |

### 9. Tests

- Golden/snapshot tests: run each Phase-1 command with `--output json` on a fixture input;
  assert the envelope JSON matches the stored golden snapshot and validates against the JSON
  Schema document.
- Diagnostic presentation tests: assert human-readable output for each `E1xxx`–`E9xxx` code
  has the correct format (code, span, message, note).
- Schema document validation: the JSON Schema files themselves are validated against the JSON
  Schema meta-schema.
- `--quiet` suppresses non-error output; `--verbose` adds docs links.
- Exit-code contract tested for each exit-code case.

## Out of Scope

- `kali effects` / `kali package-effects` JSON schemas (Phase 2 target; reserved shapes only here).
- `kali package-audit` JSON schema (Later compatibility; reserved shape only here).

## Definition of Done

- [ ] All shipped commands emit valid schema-v1 JSON envelopes under `--output json`.
- [ ] All `E1xxx`–`E9xxx` error codes have stable messages and are covered by golden tests.
- [ ] JSON Schema documents committed under `schemas/`.
- [ ] Golden/snapshot tests pass and are committed.
- [ ] Exit-code contract tested for all cases.
- [ ] No Stage 1.1–1.12 regressions.
