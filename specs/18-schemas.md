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
- `stderr: string`
- `timings: PhaseTiming[]`
- `exitCode: number` — canonical process exit code for the command invocation when the caller needs it in-band

### Notes
- `payload` holds command-specific structured data
- `command` is intentionally an open-ended string so new CLI subcommands do not force a schema-version bump; stable built-in command names should mirror the CLI subcommand path in kebab-case (for example `check`, `build`, `package-effects`)
- `kali effects` and `kali package-effects` may emit their native JSON payloads by default, but with `--output json` they must be wrapped in this envelope
- for execution-style commands in JSON mode, guest/program stdout and stderr belong in the envelope's `stdout` / `stderr` fields rather than being interleaved as raw text around the JSON payload
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
- `primary-executable` — the main executable-style artifact from `kali build foo.ts`
- `primary-library` — the main export-oriented library artifact from `kali build --lib foo.ts` (no synthetic executable entry invocation; hosts instantiate it and call its explicit exports)
- `primary-component` — the main Component Model wrapper artifact from `kali build --component foo.ts`
- `browser-glue` — browser-targeted JS glue emitted alongside a browser bundle
- `interface-wit` — canonical WIT interface description emitted for public library/embedding/component outputs
- `embedding-header` — generated program-specific C exports header from `kali build --capi`
- `embedding-metadata` — generated C-ABI/embedding metadata from `kali build --capi`
- `debug-source-map` — source-map/debug companion artifact

Interpretation rules:
- `kind` stays the primary cross-command type discriminator (`wasm-module`, `wasm-component`, `js-glue`, `wit`, `c-header`, `cabi-metadata`, `source-map`)
- `role` exists so tools do not have to infer semantic intent from filenames alone when multiple artifact modes reuse the same `kind`
- in component-oriented outputs, the wrapped core `wasm-module` normally keeps role `primary-library` while the outer `wasm-component` carries role `primary-component`; this avoids making tools guess which artifact is the deployable wrapper versus the linked core payload
- adding a new stable `role` value is a schema-contract change and should get the same review discipline as new artifact `kind` values

## Effect Report Schema

Produced by `kali effects`.

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
- `entryPoints: string[]` — logical analysis roots for this report (for example a normalized CLI entry path such as `src/main.ts`, a discovered test entry label, or an exported embedding entry name)
- `effects: EffectOccurrence[]`
- `dynamicEffects: boolean`
- `dynamicReasons: string[]` — canonical reason codes explaining why the report is conservative/incomplete; empty when `dynamicEffects` is `false`

Early-phase interpretation rule:
- for the Phase 2 CLI command `kali effects <file>`, `entryPoints` normally contains exactly one element because the command takes one explicit primary entrypoint in early phases
- for direct CLI entrypoints, the canonical label should be the normalized user-facing entry path (preferably project-root-relative when that root is known) rather than an implementation-specific symbol ID or opaque internal module handle
- `analysisContext` records the semantic knobs that materially affect the report: selected `apiSurface`, enabled `runtimeProfiles`, and enabled `compatFeatures`
- the report covers the full statically reachable program/dependency graph rooted at those entry points under that recorded analysis context; it is not a file-local AST scan of only the named source file
- the field stays an array so the same schema can later cover package-wide, test-runner, or embedding-oriented reports without inventing a second effect-report shape

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
- including `analysisContext` keeps effect payloads self-describing for caches, tooling, embedding, and AI-agent loops; the same entrypoint may have materially different effect results under different API surfaces or compatibility features

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
- `dynamicReasons` uses canonical reason strings so tools do not have to infer *why* the report became conservative from free-form notes alone
- Schema v1 reason strings are: `eval`, `function-constructor`, `dynamic-import`, `proxy-traps`, and `computed-host-access`
- Because `dynamicReasons` is a stable machine contract, adding new canonical reason strings requires a schema-version bump
- `dynamicReasons` must be empty when `dynamicEffects` is `false`
- If `dynamicReasons` contains `eval` or `function-constructor`, the report should also include the built-in `Eval` effect in `effects`
- Effect `kind` names must match the canonical built-in names derived from the type system and sandbox policy model
- Phase 1-2 effect reports are limited to built-in sandbox-relevant effect kinds; later experimental user-defined effects, if exposed, should use a reserved `Custom.<name>` namespace rather than overloading built-in policy keys
- Effect locations use `SourceLocation` fields and the same 1-based `line` / `column` convention as diagnostics so tools do not need separate coordinate systems for errors vs effect reports
- If a consumer needs a full range instead of a point location, it should use the same `SourceSpan` shape rather than inventing a command-specific span format
- To keep reports diff-friendly and AI-friendly, producers should emit a deterministic order: sort `effects` by `kind`, then sort each occurrence list by normalized `file`, `line`, `column`, and `function` when present
- `dynamicReasons` should be deduplicated and emitted in stable lexical order

## Package Effect Report Schema

Produced by `kali package-effects`.

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
- the nested `report` is the same canonical effect-report payload shape documented above; tools should not expect a package-specific effect vocabulary
- inside that nested report, `analysisContext` records which API surface / runtime-profile / compatibility-feature selection the package was analyzed under
- in early phases, `kali package-effects` inherits that analysis context from the effective config/defaults rather than introducing a second package-analysis-only flag family; the schema records the chosen context, regardless of how it was selected
- the recorded context reflects the command's successfully selected analysis mode; it does **not** relax feature-maturity rules, so an unsupported inherited context (for example `apiSurface = node` before Node package analysis exists) still causes `E5006` instead of producing a report under a fallback surface
- `entryPoints` names the package-analysis roots (for example the canonical package root specifier) and the summarized effects still cover the full statically reachable graph selected for that package analysis, not only the top-level `package.json` metadata file
- `schemaVersion` at the outer package-effect layer versions the package-analysis payload; the nested `report.schemaVersion` continues to version the shared effect-report schema independently
- by default, `kali package-effects` may emit this payload directly; with `--output json`, it is wrapped in the standard CLI command envelope with this object under `payload`

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
  "include": ["src/**/*"],
  "exclude": ["dist/**"],
  "imports": {
    "std/": "https://deno.land/std@0.220.0/",
    "~/": "./src/"
  },
  "dependencies": {
    "lodash": "^4.17.21",
    "jsr:@std/path": "^1.0.8"
  },
  "devDependencies": {
    "vitest": "^1.0.0"
  }
}
```

### Rules
- The JSON block above is a **full illustrative example**, not the minimal scaffold that `kali init` should emit by default
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
- omitted `compilerOptions.strict` means `true`; the canonical semantics of this strictness bundle are defined in [specs/04-type-system.md](04-type-system.md)
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
- `compilerOptions.apiSurface` influences command-time API/package selection for `check` / `effects` / `build` / `run` / `test`, and the inherited analysis context used by `package-effects`, but schema v1 does **not** imply separate per-surface lockfiles or install trees for the same manifest/import graph and it does **not** change the semantics of early `package-audit`
- `compilerOptions.buildMode` is one of `fast`, `release`, or `release-advanced`
- `compilerOptions.runtimeProfiles` is an array of semantic runtime-profile names; in schema v1 it is usually empty because later profiles such as `wasm-threads` are still phase-gated
- `compilerOptions.runtimeProfiles` is order-insensitive and should not contain duplicates
- `compilerOptions.strict` is the canonical strict-checking bundle switch in config; its semantics are defined in [specs/04-type-system.md](04-type-system.md), and early phases should avoid multiplying near-duplicate strictness booleans unless a later schema revision documents them explicitly
- `compilerOptions.maxSpecializations` is the project-default specialization cap upper bound; schema v1 defaults it to `16`, and CLI `--max-specializations` may override it per invocation
- `compilerOptions.maxSpecializations` does not force every build mode to spend that full budget; `buildMode = fast` may still skip most user-authored generic specialization by design, while `release`-oriented modes consume the budget more aggressively
- top-level `sandbox` is an optional default sandbox-policy path; it is the config equivalent of supplying `--sandbox <path>` for sandbox-aware commands (`run`, `test`, `check`, `build`), and an explicit CLI flag overrides it
- if `sandbox` is a relative path, it is resolved relative to the directory containing that `kali.json`
- omitting top-level `sandbox` means no default project policy file is attached; schema v1 does **not** model that omission as an implicit serialized allow-all policy
- non-sandbox-aware commands (`init`, `fmt`, `lint`, `install`, `effects`, `package-effects`, `package-audit`) ignore top-level `sandbox` rather than treating it as an error or as an implicit request to perform policy validation
- `compat.features` is the config equivalent of CLI `--compat`; entries use the same canonical feature names, are order-insensitive, and should be unique
- when set-like arrays such as `compilerOptions.runtimeProfiles` or `compat.features` are normalized in on-disk config, normalization should preserve semantics without inventing duplicates; preserving first-seen order for minimal user-file churn is preferred even though the arrays are semantically unordered
- machine-emitted payloads that report those sets back out again (for example `analysisContext` in effect/package-effect JSON) should instead use stable lexical order so caches and diffs do not depend on original input ordering
- the effective project config is the nearest `kali.json` found by searching the current working directory and then its ancestors; if none exists, commands run configless from the current working directory
- `include` / `exclude` define globs over the canonical project-discovery result for project-oriented commands, the dependency-graph install scan, hybrid no-argument discovery commands such as `check`, and editor/tooling integrations; they do not reinterpret an explicit CLI file argument as a different entry point
- relative `include` / `exclude` globs are resolved relative to the directory containing the owning `kali.json`
- when omitted, project-oriented discovery, the dependency-graph install scan, and hybrid no-argument discovery fall back to the default project-root walk and default excluded managed/generated directories defined in [SPEC.md](../SPEC.md)
- recursive project discovery also stops at nested child directories that contain their own `kali.json` unless the user explicitly targets files inside them
- `include` / `exclude` filter only the project's own discoverable files; they do not suppress transitive imports that are reached from an accepted entrypoint, and they do not act as a second package-resolution filter
- for `kali install`, this same project-discovery result is also the install-time scan set used to discover source-level raw URL imports when no explicit entrypoint is provided
- project-oriented discovery starts from the canonical project file set from [SPEC.md](../SPEC.md): executable/analyzable files plus declaration-only files, then narrows by command intent (runtime-bearing entrypoint discovery uses executable/analyzable files only)
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
- because schema v1 registry dependencies materialize into one early-phase `node_modules/` tree, install must reject a manifest that would require two distinct registry identities to occupy the same on-disk package path
- raw URL dependencies are declared in source/import maps and tracked via `kali.lock`; schema v1 intentionally does **not** add a second manifest section for them
- an ad hoc `kali install https://...` therefore stages/pins materialization for that exact URL, but durable project ownership still comes from source imports or `imports`
- schema v1 intentionally has **no** per-project registry override/auth fields in `kali.json`; early npm-registry override, if supported, comes from the documented environment/host configuration path rather than from an undocumented project config key
- Config should not mirror every CLI boolean directly when a more semantic field already exists
- Precedence is `CLI > kali.json > defaults`, except sandbox policy restrictions still constrain effective runtime behavior
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
- Policy booleans mean fully allowed or fully denied for that capability
- Pattern-bearing fields (`read`, `write`, `fetch`, `connect`, `listen`) are allowlists when they take arrays
- Numeric limit fields inside `effects.*` constrain an otherwise-allowed capability locally; for example `timer.schedule: true` with `maxActiveTimers: 32` allows timers but caps timer concurrency
- `resources.*` is reserved for cross-cutting runtime budgets rather than capability-specific allowlists/caps
- `resources.maxOpenFiles` caps concurrently opened host file handles, including internal opens performed for higher-level file helpers
- `resources.maxMemoryMB` and `resources.maxCpuTimeMs` are the canonical schema-v1 storage fields for memory and CPU budgets; CLI flags such as `--max-memory 256mb` and `--max-cpu 10s` are convenience syntaxes that normalize into the same effective-limit model before comparison
- `resources.maxSpawnedProcesses` caps concurrently active spawned processes once subprocess APIs exist; before then, validation should reject values greater than `0` instead of accepting a non-functional budget for an unavailable capability
- `resources.maxThreads` is reserved for the later threaded runtime profile; before that profile exists, validation should reject values greater than `0` instead of silently accepting them
- schema v1 intentionally has no stable policy keys for process identity, process termination, or working-directory introspection/mutation (`Deno.pid`, `process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`); those APIs therefore remain unavailable until a future schema/effect-model revision adds an auditable policy contract for them
- the `resources.*` block is a **Kali-hosted execution budget contract** rather than a generic promise about every emitted artifact environment
- Policy validation should reject non-deny values for capability fields whose corresponding feature/API surface is unavailable in the selected command/profile/api surface/phase. For example: `effects.fileSystem.read: true` under `--api browser`, `effects.eval: true` before the eval compatibility path exists, `effects.process.spawn: true` before subprocess APIs exist, `effects.process.envWrite: true` before mutable environment APIs exist, `resources.maxSpawnedProcesses > 0` before subprocess APIs exist, and `resources.maxThreads > 0` before the threaded runtime profile exists.
- Under an effective API surface of `browser`, that rejection still applies to Deno/Node-only capabilities, but it also applies to **all non-deny `resources.*` values** because early browser-targeted `check` / `build --bundle` do not promise post-deployment enforcement of CPU, memory, file-handle, process, or thread budgets in the real browser host.
- The shared Web-baseline capability keys (`effects.network.fetch`, `effects.timer.*`, `effects.random`, and `effects.console`) remain valid schema-v1 policy targets for browser-targeted `check` / `build --bundle` at the capability-model level.
- Numeric limit fields constrain an already-defined capability family; they do not enable that family by themselves. For example, `effects.network.maxConnections` does not by itself turn on `fetch`/`connect`/`listen`, and `effects.timer.maxActiveTimers` does not by itself allow timer creation when `effects.timer.schedule` is `false`.
- absence of a policy file is distinct from a permissive policy object; schemas in this chapter describe the shape of an attached `kali.policy.json`, not a hidden default object that tools should synthesize when no policy is configured
- when a sandbox policy path comes from CLI, relative paths are resolved against the current working directory; when it comes from top-level `kali.json#sandbox`, relative paths are resolved against the directory containing that config file
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
- `false` is the canonical boolean **deny** value.
- `string[]` means an allowlist; an empty array therefore denies all practical uses of that capability and is the canonical array-shaped **deny** value.
- numeric limit fields are **constraints only**; they never imply that the surrounding capability is enabled.
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

## Simplification Rule

If a schema needs more than one example across the spec set, the canonical structure belongs in this file and other specs should link here instead of duplicating the full object shape.

Additional simplification rule for diagnostics: `span` is the canonical source-range field. Any top-level `file` mirror is optional convenience data and must not diverge from `span.file`.
