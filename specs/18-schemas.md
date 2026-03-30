# 18 — Schemas

## Purpose

Kali emits several machine-consumed JSON formats:
- CLI command envelopes
- diagnostics
- effect reports
- package-effect reports
- schema-v1 envelope-only package-audit output
- project configuration (`kali.json`)
- sandbox policies

To keep the specs consistent and AI-friendly, these formats are centralized here instead of being redefined independently in multiple chapters.

Ownership rule:
- this chapter owns stable JSON field names, payload shapes, and schema-versioning rules
- [12 — CLI](12-cli.md) owns flag spelling and which commands expose `--output json`
- [19 — Feature Maturity](19-feature-maturity.md) owns whether a command/profile is available in a given phase
- [15 — Errors](15-errors.md) owns diagnostic-code meaning and error-boundary guidance

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
- `payload: object | array | string | number | boolean | null`

### Optional fields
- `artifacts: Artifact[]`
- `stdout: string`
- `stderr: string`
- `timings: PhaseTiming[]`
- `exitCode: number` — canonical process exit code for the command invocation when the caller needs it in-band; when present it follows the exit-code mapping from [12 — CLI](12-cli.md), so ordinary compile/check/build semantic failures (including library-export-proof failures such as `E5011`) still report `1`

### Notes
- `payload` holds command-specific structured data
- `command` is intentionally an open-ended string so new CLI subcommands do not force a schema-version bump; stable built-in command names should mirror the CLI subcommand path in kebab-case (for example `check`, `build`, `package-effects`)
- `payload` is always present in the schema-v1 command envelope so consumers can rely on one stable top-level shape; commands without a dedicated success payload emit `payload: null`
- a command may support `--output json` with the canonical **envelope-only JSON command** model from [SPEC.md](../SPEC.md) even when schema v1 does **not** define a dedicated success-payload schema for it; in that case the envelope itself is the stable contract and schema-v1 producers should emit `payload: null` rather than populating it with an ad hoc object
- envelope-only JSON command behavior is an output-format rule only; it does **not** promote a command to an earlier phase, create a second command surface, or bypass the command's ordinary maturity/context gates
- schema v1's **native-JSON commands** are `kali effects` and `kali package-effects` once those commands are available in the current phase; they may emit their native JSON payloads by default, but with `--output json` they must be wrapped in this envelope
- for those native-JSON commands, default success mode reserves stdout for the payload only; extra progress/status text must not be interleaved into stdout
- `--pretty` follows the cross-spec **JSON-producing mode** rule from [SPEC.md](../SPEC.md): it is meaningful only when the command is actively emitting JSON, and then it reformats the active JSON document (native payload by default, outer envelope when `--output json` is selected) without changing any field names or schema semantics
- `--pretty` does **not** by itself switch a command into JSON mode; envelope-only JSON commands still need `--output json` before `--pretty` becomes meaningful
- when those commands fail without `--output json`, human-oriented diagnostics should go to stderr; callers that need machine-readable failure output must request `--output json`
- commands that currently have only envelope-level JSON support in schema v1 (for example `package-audit`) are **envelope-only JSON commands** and may use standard envelope fields only for ordinary command metadata (such as generic diagnostics, captured text streams, timings, or exit code); they must not invent command-specific result objects outside `payload`, and they must not smuggle human prose through `payload`
- envelope-only JSON commands are not permission to repurpose `stdout` / `stderr` as hidden structured-result fields; those stream fields are for captured program/command text only
- for execution-style commands in JSON mode, guest/program stdout and stderr belong in the envelope's `stdout` / `stderr` fields rather than being interleaved as raw text around the JSON payload
- diagnostics inside the envelope may carry optional structured `context` metadata when a config/flag-derived effective command context materially caused the failure
- Commands should avoid inventing top-level ad hoc fields when `payload` is sufficient
- To keep JSON outputs diff-friendly and deterministic, producers should emit array fields in stable order when the producer naturally owns that order: diagnostics by file/line/column/code, artifacts by `role`, then `kind`, then path, and timings by canonical phase order

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
- `context: DiagnosticContext` *(structured machine-readable command/config context for diagnostics whose meaning depends on the effective invocation state)*

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

### `DiagnosticContext`

```json
{
  "origin": "config",
  "configPath": "compilerOptions.apiSurface",
  "effectiveValue": "browser"
}
```

Required fields:
- `origin: "cli" | "config" | "default" | "source"`

Optional fields:
- `configPath: string` — canonical config path when a discovered/inherited config value materially caused the diagnostic (for example `compilerOptions.apiSurface`)
- `flag: string` — canonical CLI flag spelling when an explicit flag materially caused the diagnostic (for example `--api` or `--sandbox`)
- `requestedValue: object | array | string | number | boolean | null` — user-requested value before normalization when that distinction matters
- `effectiveValue: object | array | string | number | boolean | null` — normalized effective value that the command actually validated against

Interpretation rules:
- this field exists primarily for AI/tooling-friendly diagnostics such as `E5006` and `E5008`, where the failure often depends on the merged command/config state or the resulting **availability context** rather than only the source span
- populate it only when the command/config selection materially contributes to the diagnostic; ordinary type/syntax errors usually do not need it
- when a discovered config value caused the failure, prefer `origin: "config"` plus `configPath` so tools do not have to scrape prose notes to learn that the user omitted the CLI flag but inherited the effective value
- when an explicit CLI flag caused the failure, prefer `origin: "cli"` plus `flag`
- producers may include both `requestedValue` and `effectiveValue` when normalization or merging matters (for example a config-derived browser API surface making plain `kali build main.ts` invalid because the effective API surface is `browser` even though the CLI spelled no `--api` flag)
- this field is explanatory metadata, not a second source of truth for the actual command semantics; the canonical rules still live in the CLI/spec chapters

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

Interpretation rules:
- `SuggestedFix` is reusable machine-readable edit metadata; producers may attach it to diagnostics even when the CLI for that command does not expose an auto-apply mode
- in schema v1, `kali lint --fix` is the canonical CLI autofix path, while checker diagnostics may still emit `SuggestedFix` metadata for editors, embedders, and JSON consumers without implying `kali check --fix`
- edits inside one `SuggestedFix` must already be conflict-free; cross-diagnostic fix merging is a command/tool policy choice rather than an implicit schema guarantee

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
  "role": "primary-executable",
  "bytes": 145408
}
```

Required fields:
- `path: string`
- `kind: string`
- `bytes: number`

Optional fields:
- `role: string` — canonical artifact role when the same `kind` can appear in multiple build modes

Canonical schema-v1 `role` values:
- `primary-executable` — the main executable-oriented core artifact for one build, used by both the default executable path (`kali build foo.ts`) and the browser-bundle path (`kali build --bundle foo.ts` when the effective `apiSurface` is `browser`)
- `primary-library` — the main export-oriented core artifact for one library-oriented build, used by `kali build --lib foo.ts` and by the shared linked core inside later `--capi` / `--component` outputs, following the shared **library-oriented instantiation rule** and using the build's **statically known export surface** as defined in [SPEC.md](../SPEC.md)
- `primary-component` — the main outer Component Model wrapper artifact from `kali build --component foo.ts`
- `browser-glue` — browser-targeted JS glue emitted alongside a browser bundle; this is the browser host adapter companion to the bundle's `primary-executable` core module
- `interface-wit` — canonical WIT interface description emitted for the stable public library/component/embedding flows once that Phase-2 public contract exists
- `embedding-header` — generated **program-specific exports header** from `kali build --capi` (distinct from the stable **host ABI header** `kali.h`; see [SPEC.md](../SPEC.md))
- `embedding-metadata` — generated C-ABI compatibility metadata from `kali build --capi` (the artifact `kind` remains `cabi-metadata`; this is the canonical `role` for that file)
- `debug-source-map` — source-map/debug companion artifact

Interpretation rules:
- `kind` stays the primary cross-command type discriminator (`wasm-module`, `wasm-component`, `js-glue`, `wit`, `c-header`, `cabi-metadata`, `source-map`)
- `debug-source-map` is a `role`, not a second source-map `kind`; the matching artifact `kind` remains `source-map`
- `role` exists so tools do not have to infer semantic intent from filenames alone when multiple artifact modes reuse the same `kind`
- within one emitted artifact list, `primary-executable`, `primary-library`, and `primary-component` are each unique roles: at most one artifact may carry each of those roles
- browser-bundle outputs therefore normally contain one `primary-executable` core `wasm-module` plus one `browser-glue` JS companion, rather than two competing "primary" artifacts of the same executable flow
- in component-oriented outputs, the wrapped core `wasm-module` normally keeps role `primary-library` while the outer `wasm-component` carries role `primary-component`; this avoids making tools guess which artifact is the deployable wrapper versus the linked core payload
- adding a new stable `role` value is a schema-contract change and should get the same review discipline as new artifact `kind` values

## Effect Report Schema

Produced by `kali effects` once that Phase-2 command is available; Phase 1 may still use compatible internal effect data without exposing this as a stable public CLI contract.

```json
{
  "schemaVersion": 1,
  "analysisContext": {
    "apiSurface": "deno",
    "runtimeProfiles": [],
    "compatFeatures": []
  },
  "entryPoints": ["src/main.ts"],
  "effects": [
    {
      "kind": "FileSystem.Read",
      "locations": [
        {
          "file": "src/main.ts",
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
- `analysisContext: EffectAnalysisContext`
- `entryPoints: string[]` — shared schema-v1 field for the report's **logical roots** (see the naming bridge in [SPEC.md](../SPEC.md)); examples include a normalized CLI input path such as `src/main.ts`, a discovered test label, or a package root specifier such as `lodash`
- `effects: EffectOccurrence[]`
- `dynamicEffects: boolean`
- `dynamicReasons: string[]` — canonical reason codes explaining why the report is conservative/incomplete; empty when `dynamicEffects` is `false`

Early-phase interpretation rule:
- `entryPoints` is a historical stable field name for logical roots, not a promise that every producer is describing a runtime entrypoint
- for the Phase 2 CLI command `kali effects <file>`, `entryPoints` normally contains exactly one element because schema v1 keeps the command at one explicit primary analysis root
- for direct CLI analysis inputs, the canonical label should be the normalized user-facing entry path (preferably project-root-relative when that root is known) rather than an implementation-specific symbol ID or opaque internal module handle
- `analysisContext` records the semantic knobs that materially affect the report: selected `apiSurface`, enabled `runtimeProfiles`, and enabled `compatFeatures`
- the report covers the command's full analysis graph under that recorded analysis context, not a file-local AST scan of only the named source file
- for source-graph producers such as `kali effects`, that graph is the same **resolved source graph** defined in [SPEC.md](../SPEC.md)
- the field stays an array so the same schema can later cover package-wide, test-runner, or other report producers without inventing a second effect-report shape

### `EffectAnalysisContext`
```json
{
  "apiSurface": "deno",
  "runtimeProfiles": [],
  "compatFeatures": []
}
```

Required fields:
- `apiSurface: "deno" | "node" | "browser"`
- `runtimeProfiles: string[]`
- `compatFeatures: string[]`

Interpretation rules:
- schema v1 uses the same canonical vocabulary as config/CLI: `apiSurface`, `runtimeProfiles`, and compatibility features
- because config stores compatibility features under the nested key `compat.features`, the effect-report field name is flattened to `compatFeatures` for a compact self-contained payload; this is a shape simplification, not a second vocabulary
- `runtimeProfiles` and `compatFeatures` are semantic sets encoded as arrays; they must be deduplicated, and in machine-emitted payloads they should be sorted in stable lexical order
- `apiSurface = "node"` or later compatibility/runtime-profile values may appear only when those modes are actually implemented for the command/profile; the schema records the chosen context, it does not relax feature-maturity rules
- including `analysisContext` keeps effect payloads self-describing for caches, tooling, embedding, and AI-agent loops; the same logical root may have materially different effect results under different API surfaces or compatibility features

### `EffectOccurrence`
```json
{
  "kind": "FileSystem.Read",
  "locations": [
    {
      "file": "src/main.ts",
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
- for **Kali-hosted execution** contexts, such a report means runtime sandbox enforcement must remain authoritative for the dynamic paths the static model could not fully classify
- for the shared **Phase-1 browser-targeted command set** and later browser-context analysis commands that explicitly reuse that same context, the same flag is still a static warning signal but must **not** be read as a promise of automatic post-deployment Kali runtime enforcement inside the browser host
- `dynamicReasons` uses canonical reason strings so tools do not have to infer *why* the report became conservative from free-form notes alone
- Schema v1 reason strings are: `eval`, `function-constructor`, `dynamic-import`, `proxy-traps`, and `computed-host-access`
- the separate `eval` and `function-constructor` reason codes do **not** imply separate compatibility-feature names; both still map to the single schema-v1 compatibility switch `eval`
- Because `dynamicReasons` is a stable machine contract, adding new canonical reason strings requires a schema-version bump
- `dynamicReasons` must be empty when `dynamicEffects` is `false`
- If `dynamicReasons` contains `eval` or `function-constructor`, the report should also include the built-in `Eval` effect in `effects`
- Effect `kind` names must match the canonical built-in names derived from the type system and sandbox policy model
- the reserved public effect-report schemas that start in the Phase 2 target window, together with the Phase-1/2 policy/config machine contracts, are limited to built-in sandbox-relevant effect kinds; later experimental user-defined effects, if exposed, should use a reserved `Custom.<name>` namespace rather than overloading built-in policy keys
- Effect locations use `SourceLocation` fields and the same 1-based `line` / `column` convention as diagnostics so tools do not need separate coordinate systems for errors vs effect reports
- If a consumer needs a full range instead of a point location, it should use the same `SourceSpan` shape rather than inventing a command-specific span format
- To keep reports diff-friendly and AI-friendly, producers should emit a deterministic order: sort `effects` by `kind`, then sort each occurrence list by normalized `file`, `line`, `column`, and `function` when present
- `dynamicReasons` should be deduplicated and emitted in stable lexical order

## Package Effect Report Schema

Produced by `kali package-effects` once that Phase-2 command is available; before then, this schema remains reserved rather than a promise of partial ad hoc output.

```json
{
  "schemaVersion": 1,
  "package": {
    "name": "lodash",
    "version": "4.17.21",
    "registry": "npm"
  },
  "report": {
    "schemaVersion": 1,
    "analysisContext": {
      "apiSurface": "deno",
      "runtimeProfiles": [],
      "compatFeatures": []
    },
    "entryPoints": ["lodash"],
    "effects": [],
    "dynamicEffects": false,
    "dynamicReasons": []
  }
}
```

### Required fields
- `schemaVersion: number`
- `package: PackageCoordinate`
- `report: object` — the exact effect-report payload shape defined in the previous section

### `PackageCoordinate`
```json
{
  "name": "lodash",
  "version": "4.17.21",
  "registry": "npm"
}
```

Required fields:
- `name: string`
- `version: string`
- `registry: "npm" | "jsr"`

Interpretation rules:
- `PackageCoordinate` is for **registry packages only**; schema v1 package-effect payloads do not use this shape for raw URLs or local paths
- follow the shared **registry package identifier vs package coordinate** split from [SPEC.md](../SPEC.md): this structured object is the decomposed package-coordinate form, while CLI arguments, diagnostics, and nested `report.entryPoints` use the user-facing registry package identifier spelling
- `package.name` therefore carries the registry-native package name only (`lodash`, `@types/node`, `@std/path`); for JSR packages the `jsr:` identity marker is represented by `package.registry = "jsr"`, not duplicated inside `package.name`
- `package.version` is the concrete resolved version actually analyzed. The CLI package argument is versionless in schema v1; for `package-effects`, that resolved version follows the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md) unless a later spec adds an explicit version/range selector.
- follow the shared **registry-analysis independence split** from [SPEC.md](../SPEC.md): this resolved version is not inferred from the current project's manifest or lockfile, even though `package-effects` may still inherit semantic analysis context from defaults/discovered config.
- the emitted payload must still record the exact resolved version so caches, diffs, and audit trails stay reproducible.
- the nested `report` is the same canonical effect-report payload shape documented above; tools should not expect a package-specific effect vocabulary
- inside that nested report, `analysisContext` records which API surface / runtime-profile / compatibility-feature selection the package was analyzed under
- as the analysis-context-aware half of the shared **registry-analysis command split** from [SPEC.md](../SPEC.md), `kali package-effects` inherits that analysis context through the shared **inherited analysis context** rather than introducing a second package-analysis-only flag family; the schema records the chosen context, regardless of how it was selected
- inherited-context maturity follows the shared **axis-aligned inherited analysis gating** rule from [SPEC.md](../SPEC.md) rather than a package-only shadow rule set
- the recorded context reflects the command's successfully selected analysis mode; it does **not** relax feature-maturity rules, so an inherited context that is still unavailable for package analysis still causes `E5006` instead of producing a report under a fallback surface
- `entryPoints` names the package-analysis logical roots (for example the canonical package root specifier) and the summarized effects still cover the command's full analysis graph for that package root, not only the top-level `package.json` metadata file
- for schema-v1 CLI package analysis, that root label should use the same canonical registry package identifier spelling the user targeted (`lodash`, `@types/node`, `jsr:@std/path`) rather than a tarball URL, cache path, or opaque internal package handle
- `schemaVersion` at the outer package-effect layer versions the package-analysis payload; the nested `report.schemaVersion` continues to version the shared effect-report schema independently
- by default, `kali package-effects` may emit this payload directly; with `--output json`, it is wrapped in the standard CLI command envelope with this object under `payload`

## Package Audit JSON Output (schema v1)

As the context-free half of the shared **registry-analysis command split** from [SPEC.md](../SPEC.md), `kali package-audit` intentionally has **no dedicated success-payload schema in schema v1** and is therefore the canonical schema-v1 **envelope-only JSON command**.

The machine-readable contract is therefore the standard CLI command envelope only:
- `--output json` emits the normal envelope
- schema-v1 producers should emit `payload: null`
- package/version/audit result metadata must not be invented as ad hoc top-level fields outside `payload`
- audit findings, when the command later exists, are surfaced through the standard `errors` / `warnings` diagnostic arrays rather than through a second audit-specific payload object
- a successful audit with no findings therefore appears as `success: true`, `payload: null`, and empty diagnostic arrays, not as a hidden result object in `payload`
- `stdout` / `stderr` remain captured text-stream fields only; they are not hidden structured-result channels
- `--pretty --output json` reformats that outer envelope only and does not create a second audit payload shape

Interpretation rule:
- this is an **output-format rule**, not a separate availability path
- if `package-audit` is unavailable in the current phase, `--output json` still fails on the ordinary command-availability gate after any earlier command-shape checks

This section exists so CLI, package-management, and maturity docs can all point to one schema-level rule instead of restating slightly different versions of the same envelope-only contract.

## Canonical Built-in Effect Names

To keep the checker, CLI, effect reports, and sandbox policy model aligned, built-in effect names use one canonical dotted namespace.

This table is the normative mapping for the cross-spec distinction from [SPEC.md](../SPEC.md) between semantic built-in effect kinds and `effects.*` policy/schema keys.

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
- existing host APIs should map onto these names before the spec adds new built-in effect families: for example `Deno.stat*` / `Deno.readDir*` map to `FileSystem.Read`, `Deno.env.get` / `Deno.env.toObject` map to `Process.EnvRead`, and query-only `Deno.permissions` observation remains effect-free rather than adding a new `Permissions.Query` effect
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
  "include": ["src/**/*"],
  "exclude": ["dist/**"],
  "imports": {
    "std/": "https://deno.land/std@0.220.0/",
    "~/": "./src/"
  },
  "dependencies": {
    "lodash": "4.17.21",
    "jsr:@std/path": "1.0.8"
  },
  "devDependencies": {
    "vitest": "1.0.0"
  }
}
```

### Rules
- The JSON block above is a **full illustrative example**, not the minimal scaffold that `kali init` should emit by default
- its dependency versions are illustrative manifest contents, not an alternate CLI input form: schema-v1 `kali install <pkg>` and registry-analysis commands still use the shared **identity-only registry target** workflow from [SPEC.md](../SPEC.md), then record/resolve versions through the package rules elsewhere in the spec set
- for schema-v1 registry manifests, follow the shared **exact-version-first registry manifest rule (schema v1)** from [SPEC.md](../SPEC.md): the canonical recorded value is the exact resolved version string, while wider version-range manifest syntax is intentionally deferred
- `schemaVersion: number` is required on `kali.json` like every other top-level machine-readable Kali JSON document
- `$schema: string` is an optional, recognized top-level metadata field for editor/tooling integration; it is not treated as an unknown extension field

### Schema-v1 defaulting and omission rules
To keep `kali.json` minimal and avoid placeholder churn, schema v1 uses a small canonical default set when fields are omitted.

Smallest valid schema-v1 config:
```json
{
  "schemaVersion": 1
}
```

Defaults:
- omitted `compilerOptions` means `{}`
- omitted `compilerOptions.strict` means `true`; the canonical semantics of this **strictness bundle** are defined in [SPEC.md](../SPEC.md) and [specs/04-type-system.md](04-type-system.md)
- omitted `compilerOptions.apiSurface` means `deno`
- omitted `compilerOptions.buildMode` means `fast`
- omitted `compilerOptions.runtimeProfiles` means `[]`
- omitted `compilerOptions.maxSpecializations` means `16`
- omitted `compat` means `{"features": []}`
- omitted `compat.features` means `[]`

Canonical compatibility feature names (schema v1):
- `"eval"` is the only stable compatibility feature name in schema v1
- enabling `"eval"` is the documented compatibility switch for both direct `eval` support and the `Function()` constructor path
- unknown compatibility feature names are rejected rather than ignored so tools do not silently diverge

Interpretation rules:
- `kali init` should prefer omission of default-valued optional fields over emitting empty placeholder sections
- a default app scaffold may therefore emit only `{"schemaVersion": 1}` unless the chosen template needs additional config
- tools may materialize these defaults internally, but should preserve a minimal on-disk config unless the user explicitly asks for a fuller form
- when a tool normalizes `kali.json`, it must not change semantics by adding or removing fields whose values equal these defaults
- `compilerOptions.apiSurface` is the canonical config name for the host API family; CLI uses `--api`
- command/context participation follows the canonical table in [SPEC.md](../SPEC.md): `compilerOptions.apiSurface` influences command-time API/package selection for `check` / `effects` / `build` / `run` / `test`, and the inherited analysis context used by `package-effects`, but schema v1 does **not** imply separate per-surface lockfiles or install trees for the same manifest/import graph
- because early `package-audit` follows **context-free registry analysis (schema v1)** from [SPEC.md](../SPEC.md), inherited `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, `compat.features`, and top-level `sandbox` do **not** change its semantics
- when `package-audit` later exists, schema v1 still reports findings through the standard envelope diagnostic arrays rather than through a dedicated success payload; config does not unlock a second audit-result object
- `compilerOptions.buildMode` is one of `fast`, `release`, or `release-advanced`
- `compilerOptions.runtimeProfiles` is an array of semantic runtime-profile names; in schema v1 it is usually empty because later profiles such as `wasm-threads` are still phase-gated
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- unknown runtime-profile names are rejected rather than ignored so config loaders do not silently diverge about which execution-capability set was requested
- `compilerOptions.strict` is the canonical **strictness bundle** switch in config; its semantics are defined in [SPEC.md](../SPEC.md) and [specs/04-type-system.md](04-type-system.md), and early phases should avoid multiplying near-duplicate strictness booleans unless a later schema revision documents them explicitly
- `compilerOptions.maxSpecializations` is the project-default specialization cap upper bound; schema v1 defaults it to `16`, and CLI `--max-specializations` may override it per invocation
- `compilerOptions.maxSpecializations` does not force every build mode to spend that full budget; `buildMode = fast` may still skip most user-authored generic specialization by design, while `release`-oriented modes consume the budget more aggressively
- top-level `sandbox` is an optional default sandbox-policy path; it is the config equivalent of supplying `--sandbox <path>` for the canonical sandbox-aware commands from [SPEC.md](../SPEC.md), and an explicit CLI flag overrides it
- if `sandbox` is a relative path, it is resolved relative to the directory containing that `kali.json`
- omitting top-level `sandbox` means no default project policy file is attached; schema v1 does **not** model that omission as an implicit serialized allow-all policy
- the canonical effect-reporting and sandbox-agnostic command classes from [SPEC.md](../SPEC.md) ignore top-level `sandbox` rather than treating it as an error or as an implicit request to perform policy validation
- in particular, `package-effects` still ignores `sandbox` even though it inherits the other semantic analysis axes, and `package-audit` follows **context-free registry analysis (schema v1)** from [SPEC.md](../SPEC.md)
- `compat.features` is the config equivalent of CLI `--compat`; entries use the same canonical feature names, are order-insensitive, and should be unique
- both `compilerOptions.runtimeProfiles` and `compat.features` follow the same schema-v1 validation rule: unknown entries and duplicate entries are config errors (`E5009`), not values tools silently ignore or deduplicate away
- when **valid** set-like arrays such as `compilerOptions.runtimeProfiles` or `compat.features` are normalized in on-disk config, normalization should preserve semantics without reordering entries unnecessarily; preserving first-seen order for minimal user-file churn is preferred even though the arrays are semantically unordered
- machine-emitted payloads that report those sets back out again (for example `analysisContext` in effect/package-effect JSON) should instead use stable lexical order so caches and diffs do not depend on original input ordering
- the effective project config is the nearest `kali.json` found by searching the current working directory and then its ancestors; if none exists, commands run configless from the current working directory
- `kali init` is the one early-phase exception to that ancestor-based lookup: it is current-directory-scoped and does not reuse an ancestor `kali.json` as its target root
- explicit CLI file/path arguments do not relocate that chosen config/root; follow the canonical **explicit path boundary rule** from [SPEC.md](../SPEC.md) for file-accepting source commands
- `include` / `exclude` define globs over the canonical project-discovery result for **discovery-driven commands** from [SPEC.md](../SPEC.md) and for editor/tooling integrations; they do not reinterpret an explicit CLI file argument as a different primary source input or analysis root, and they do not silently filter out an explicit file/path target once the user named it directly
- relative `include` / `exclude` globs are resolved relative to the directory containing the owning `kali.json`
- when omitted, **discovery-driven commands** fall back to the default project-root walk and default excluded managed/generated directories defined in [SPEC.md](../SPEC.md)
- recursive project discovery also stops at nested child directories that contain their own `kali.json`; those child roots are separate projects in schema v1
- `include` / `exclude` filter only the project's own discoverable files; they do not suppress transitive imports that are reached from an accepted entrypoint, and they do not act as a second package-resolution filter
- for `kali install`, this same project-discovery result contributes the source-discovery portion of the **install-time declaration graph** from [SPEC.md](../SPEC.md) and is used to discover source-level raw URL imports when no explicit primary source input is provided
- project-oriented discovery starts from the shared **canonical project file set** from [SPEC.md](../SPEC.md), then narrows by command intent (runtime-bearing entrypoint discovery uses only the shared **executable/analyzable source-file class**)
- `imports` is the canonical alias/import-map section for URL and path-like rewrites; it is not a second registry-dependency manifest
- schema v1 import-map targets are limited to raw URLs and path/local rewrites; rewrites to bare package specifiers or canonical registry identifiers such as `jsr:@std/path` are rejected explicitly so registry ownership stays in one place
- import-map keys without a trailing `/` are exact-match rewrites; keys ending in `/` are prefix rewrites
- when multiple import-map keys match, the longest matching key wins
- a prefix key ending in `/` must map to a target ending in `/` so the unmatched suffix is appended deterministically
- local path-like import-map targets are resolved relative to the directory containing that `kali.json`
- the effective lock/cache/materialization root for that config is the same project root as the owning `kali.json`; invoking a command from a subdirectory of the same project must not create a second implicit dependency state beside that subdirectory
- schema v1 import maps do not support wildcard/glob/regex keys or targets; exact and prefix rewrites are the complete stable contract
- `dependencies` and `devDependencies` are top-level package manifests for **registry packages** owned by `kali install`; they are not nested under `compilerOptions`
- dependency keys use the canonical registry-package identifier grammar from [specs/14-packages.md](14-packages.md): normal npm package names (for example `lodash` or `@types/node`) and `jsr:`-prefixed JSR names
- in schema v1, dependency values for those registry keys are exact resolved version strings rather than broad SemVer ranges
- range spellings such as `^1.2.3`, `~1.2.3`, or `>=1.2.3` are therefore invalid config in schema v1 instead of hidden alternate resolution modes
- when `kali install <pkg>` or `kali install --dev <pkg>` adds a new registry dependency from the shared **identity-only registry target** form, it uses the shared **stable-release selection rule (schema v1)** plus the **exact-version-first registry manifest rule (schema v1)** from [SPEC.md](../SPEC.md): resolve the latest non-yanked stable published version, write `kali.lock` with that concrete version, and record the manifest entry as that same exact version string
- in the canonical **configless install split** from [SPEC.md](../SPEC.md), an explicit registry-package add first creates the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then records the dependency there; schema v1 intentionally keeps registry-package ownership in one manifest file rather than inventing a configless dependency sidecar
- because schema v1 registry dependencies materialize into one early-phase `node_modules/` tree, install must reject a manifest that would require two distinct registry identities to occupy the same on-disk package path
- raw URL dependencies are declared in source/import maps and tracked via `kali.lock`; schema v1 intentionally does **not** add a second manifest section for them
- in that same **configless install split**, plain `kali install` is a no-op success when the effective project root contributes no manifest/import/source dependency inputs, and it does **not** create a placeholder `kali.json`
- an ad hoc `kali install https://...` therefore stages/pins materialization for that exact URL, but durable project ownership still comes from source imports or `imports`
- schema v1 intentionally has **no** per-project registry override/auth fields in `kali.json`; early npm-registry override, if supported, comes from the documented environment/host configuration path rather than from an undocumented project config key
- Config should not mirror every CLI boolean directly when a more semantic field already exists
- follow the canonical precedence rule from [specs/12-cli.md](12-cli.md): `CLI > kali.json > defaults`, while any attached sandbox policy still acts as an upper bound for runtime capabilities/resource limits rather than a lower-precedence preference value
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
- `$schema: string` is an optional, recognized top-level metadata field for editor/tooling integration; it is not treated as an unknown extension field
- Unknown fields are rejected at every documented nesting level to keep policy evaluation deterministic and auditable
- schema v1 intentionally has no `predicates`, `hooks`, `script`, or other executable-policy fields; those names are rejected as unknown fields rather than treated as soft extensions
- if later host-registered predicate support exists, it is an embedding-side runtime registration contract rather than an extension point inside `kali.policy.json`
- Policy booleans mean fully allowed or fully denied for that capability
- Pattern-bearing fields (`read`, `write`, `fetch`, `connect`, `listen`) are allowlists when they take arrays
- Numeric limit fields inside `effects.*` constrain an otherwise-allowed capability locally; for example `effects.timer.schedule: true` with `effects.timer.maxActiveTimers: 32` allows timer creation but caps timer concurrency
- `effects.timer.maxTimeoutMs`, `effects.timer.maxActiveTimers`, and `effects.network.maxConnections` must be positive integers when present; `0` is invalid for these fields rather than a hidden deny value because deny/disable semantics already live on the surrounding boolean/allowlist capability fields
- `resources.*` is reserved for cross-cutting runtime budgets rather than capability-specific allowlists/caps
- resource-budget fields are **not** one generic boolean/deny channel in numeric form; they keep field-specific numeric semantics
- omission is the canonical schema-v1 meaning of "no explicit budget provided" for `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles`
- `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` must be positive integers when present; `0` is invalid for those fields rather than a special deny value
- `resources.maxOpenFiles` caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- follow the canonical numeric-limit semantics from [SPEC.md](../SPEC.md): positive-budget dimensions use omission as the schema-v1 “unspecified” state and reject `0`, while zero-capable concurrency counters may use `0` as an explicit deny/tightening value
- `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`, `resources.maxSpawnedProcesses`, and `resources.maxThreads` are the canonical schema-v1 storage fields mirrored by the direct CLI resource-override flags; `--max-memory 256mb` and `--max-cpu 10s` use convenience literal syntaxes, while `--max-open-files N`, later `--max-spawned-processes N`, and later `--max-threads N` use integer counts that compare against the same effective-limit model
- in other chapters, capability snapshots may omit `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` when those tables are focused on phase-gated capability families; that omission should not be read as those positive-budget runtime limits being unavailable in Phase 1 Kali-hosted execution
- `resources.maxSpawnedProcesses` and `resources.maxThreads` follow the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): omission means no extra tightening from that source, `0` is a valid explicit deny/tightening value, and positive values remain availability-gated until subprocess/thread support exists
- schema v1 intentionally has no stable policy keys for process identity, process termination, or working-directory introspection/mutation (`Deno.pid`, `process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`); those APIs therefore remain unavailable until a future schema/effect-model revision adds an auditable policy contract for them
- the `resources.*` block is a **Kali-hosted execution budget contract** rather than a generic promise about every emitted artifact environment
- policy permission and compatibility-feature enablement are separate axes: a policy may authorize `effects.eval`, but it must not implicitly enable the separate `--compat eval` / `compat.features = ["eval"]` switch
- Policy validation should reject non-deny values for capability fields whose corresponding feature/API surface is unavailable in the selected command/profile/api surface/phase. For example: `effects.fileSystem.read: true` under `--api browser`, `effects.eval: true` before the eval compatibility path exists, `effects.eval: true` without effective `--compat eval`, `effects.process.spawn: true` before subprocess APIs exist, `effects.process.envWrite: true` before mutable environment APIs exist, or positive values for the shared **feature-gated zero-capable execution budgets** before subprocess/thread support exists.
- For numeric resource-budget fields whose semantics are ordinary positive budgets rather than boolean-like disable counters (`resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`), validation should treat omission as "unspecified" and reject `0` as an invalid value rather than interpreting it as a hidden deny form.
- Under an effective API surface of `browser`, that rejection still applies to Deno/Node-only capabilities, and schema-v1 also follows the **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md): `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` are rejected whenever present, while `resources.maxSpawnedProcesses` and `resources.maxThreads` are rejected when set to positive values. Those `resources.*` fields remain Kali-hosted execution budgets rather than post-deployment browser guarantees.
- In browser-targeted modes, policy/effect reasoning applies only to the browser-applicable part of the **Kali-mediated capability subset** defined by the **canonical browser-applicable mediated subset (schema v1)** in [SPEC.md](../SPEC.md). In schema v1 that means `effects.network.fetch` plus its capability-local cap `effects.network.maxConnections`, `effects.timer.*`, `effects.random`, and `effects.console`, plus later `effects.eval` only when its separate compatibility path exists and is enabled. Deno/Node-only capability keys such as `effects.fileSystem.*`, `effects.process.*`, `effects.network.connect`, and `effects.network.listen` stay unavailable there, and ambient DOM/browser APIs outside that subset do not gain one policy key per operation.
- Numeric limit fields constrain an already-defined capability family; they do not enable that family by themselves. For example, `effects.network.maxConnections` does not by itself turn on `fetch`/`connect`/`listen`, and `effects.timer.maxActiveTimers` does not by itself allow timer creation when `effects.timer.schedule` is `false`.
- absence of a policy file is distinct from a permissive policy object; schemas in this chapter describe the shape of an attached `kali.policy.json`, not a hidden default object that tools should synthesize when no policy is configured
- when a sandbox policy path comes from CLI, relative paths are resolved against the current working directory; when it comes from top-level `kali.json#sandbox`, relative paths are resolved against the directory containing that config file
- Per-invocation CLI resource overrides (`--max-memory`, `--max-cpu`, `--max-open-files`, later `--max-spawned-processes`, and later `--max-threads`) may only tighten these policy limits; they must not widen them
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
| `effects.timer.maxTimeoutMs` | positive integer | maximum allowed timeout/interval delay |
| `effects.timer.maxActiveTimers` | positive integer | maximum concurrently active timers |
| `effects.network.maxConnections` | positive integer | maximum concurrent connections within the modeled network capability subset (`fetch` in browser-targeted schema-v1 contexts; `fetch` / `connect` / `listen` on surfaces where those paths are supported) |
| `effects.eval` | `boolean` | allow or deny `Eval` capability |
| `effects.random` | `boolean` | allow or deny `Random.*` capability family |
| `effects.console` | `boolean` | allow or deny `Console.*` capability family |

Interpretation rules:
- `true` means unrestricted for that capability within schema v1, subject to separate `resources.*` caps.
- `false` is the canonical boolean **deny** value.
- `string[]` means an allowlist; an empty array therefore denies all practical uses of that capability and is the canonical array-shaped **deny** value.
- numeric limit fields are **constraints only**; they never imply that the surrounding capability is enabled.
- for `effects.timer.maxTimeoutMs`, `effects.timer.maxActiveTimers`, and `effects.network.maxConnections`, any present value must be a positive integer; `0` is invalid rather than a second disable channel.
- for `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles`, omission means "no explicit budget in this policy" and any present value must be a positive integer.
- `resources.maxSpawnedProcesses` and `resources.maxThreads` follow the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is valid as the explicit deny/tightening value, while positive values remain phase/profile-gated by feature availability.
- Field-specific arrays use canonical matching domains: filesystem paths for file APIs, URLs/addresses for network APIs, executable names/paths for process spawning, and exact environment-variable names for env access.
- Specs and examples should reuse these shapes instead of inventing per-command variants.

### Canonical matching rules (schema v1)

To keep policy validation, compile-time effect checks, and runtime enforcement aligned, schema v1 uses one shared matcher model:

- `effects.fileSystem.read` / `effects.fileSystem.write`
  - candidate paths are normalized before matching
  - matching uses `/` as the separator on every host; Windows-style `\` separators are normalized first
  - relative policy entries are resolved against the project root before matching
  - `*` matches within one path segment; `**` may cross `/` boundaries
- `effects.network.fetch`
  - candidates are matched against the normalized absolute URL string
  - scheme and host are normalized using standard URL serialization before matching
  - the same `*` / `**` wildcard rules apply over the serialized URL string
- `effects.network.connect` / `effects.network.listen`
  - candidates are matched against a normalized address string chosen by the host API (`host:port` for socket-style APIs, or an absolute URL string when the API is URL-shaped)
  - the same `*` / `**` wildcard rules apply
- `effects.process.spawn`
  - candidates are matched against a normalized executable identity string (absolute path when available, otherwise the invoked program name)
  - the same `*` / `**` wildcard rules apply
- `effects.process.envRead` / `effects.process.envWrite`
  - schema v1 arrays are exact environment-variable names, not glob patterns
  - matching is by exact string equality after the host's normal environment-name normalization rules are applied

Consistency rules:
- policy engines must use the same normalized matcher semantics at validation time and enforcement time
- implementations must not silently interpret the same policy array as a regex in one subsystem and a glob in another
- future schema revisions may add richer match objects, but schema v1 keeps the string form intentionally simple

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
- `wasm-component`
- `js-glue`
- `wit`
- `c-header`
- `cabi-metadata`
- `source-map`

Interpretation rule:
- `source-map` is a valid artifact kind when debug/source-map output is emitted, but ordinary Phase 1 builds do not need to produce source maps by default
- when a command emits artifact metadata, it should include `role` whenever that makes the artifact mode clearer (for example distinguishing the default executable `wasm-module` from a `--lib` `wasm-module`)

Simplification rule:
- build-like commands should use these canonical artifact kinds instead of inventing near-synonyms such as `wasm`, `header`, or `metadata-json`
- they should also prefer the canonical `role` values above instead of per-command ad hoc labels
- adding a new stable artifact `kind` or `role` value is a schema-contract change and should get the same review discipline as other enum-like machine strings in this file

## C ABI Metadata Schema (schema v1)

Produced as the contents of the `cabi-metadata` artifact emitted by `kali build --capi` (normally `role: embedding-metadata`).

This metadata exists to answer one narrow question deterministically: **can this generated library artifact be loaded by the available host-side `kali_capi` ABI layer?** It should not duplicate the WIT surface, the generated program-specific exports header, or the CLI artifact manifest.

```json
{
  "schemaVersion": 1,
  "kind": "cabi-metadata",
  "hostAbiVersion": 2,
  "minHostAbiVersion": 2,
  "artifacts": {
    "wasmModule": "lib.wasm",
    "wit": "lib.wit",
    "exportsHeader": "lib.exports.h"
  }
}
```

### Required fields
- `schemaVersion: number`
- `kind: "cabi-metadata"`
- `hostAbiVersion: number` — the exact host ABI version expected by the generated embedding artifact set
- `artifacts.wasmModule: string` — path or artifact-relative filename for the core linked library module
- `artifacts.wit: string` — path or artifact-relative filename for the canonical WIT interface description
- `artifacts.exportsHeader: string` — path or artifact-relative filename for the generated **program-specific exports header**

### Optional fields
- `minHostAbiVersion: number` — lowest compatible host ABI version when the compatibility policy allows a version window; if omitted, consumers should treat `hostAbiVersion` as the exact required version

Interpretation rules:
- this metadata is the canonical load-time compatibility record for `kali build --capi`; loaders should check it before instantiating the library artifact through `kali_capi`
- the intended host-side comparison point is the C-ABI helper `kali_host_abi_version()` described in [specs/13-embedding.md](13-embedding.md), whose naming intentionally matches `hostAbiVersion` / `minHostAbiVersion`
- the conventional emitted filename is `<entry>.cabi.json` (for example `lib.cabi.json`), but the schema is keyed by contents and artifact role/kind rather than by one hard-coded basename
- `hostAbiVersion` / `minHostAbiVersion` describe compatibility with the stable **host ABI header** / host-side `kali_capi` library, not the user program's exported function set
- exported-function shape belongs to WIT plus the generated **program-specific exports header**; this metadata should not duplicate that interface in a second ad hoc schema
- artifact references should point at the sibling outputs of the same `kali build --capi` invocation rather than at ambient global install locations
- schema v1 intentionally keeps this file small: enough for deterministic ABI/version checks and artifact association, but not a second full build manifest

## Simplification Rule

If a schema needs more than one example across the spec set, the canonical structure belongs in this file and other specs should link here instead of duplicating the full object shape.

Additional simplification rule for diagnostics: `span` is the canonical source-range field. Any top-level `file` mirror is optional convenience data and must not diverge from `span.file`.
