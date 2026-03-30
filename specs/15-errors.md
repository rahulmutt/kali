# 15 — Error Reporting

## Design Goals

1. **AI-friendly**: Structured, parseable, minimal tokens for AI feedback loops
2. **Human-friendly**: Clear, colorful, with context and suggestions
3. **Consistent**: Every error has a code, message, location, and optional fix suggestion

Ownership rule:
- this chapter owns diagnostic-code meaning, error-boundary guidance, and human-readable diagnostic conventions
- [19 — Feature Maturity](19-feature-maturity.md) owns whether a documented feature/profile exists in a given phase
- [12 — CLI](12-cli.md) owns command spelling/arity and exit-code behavior
- [18 — Schemas](18-schemas.md) owns JSON envelope/diagnostic field shapes

## Error Format

### Default (Human)
```
error[E1001]: Type 'string' is not assignable to type 'number'
  --> src/main.ts:5:10
  |
5 |   let x: number = "hello";
  |          ------   ^^^^^^^ expected 'number', found 'string'
  |          expected type
  |
  = help: Remove the type annotation or change the value
```

### JSON (`--output json`)
Diagnostics are emitted inside the CLI's versioned command envelope. The canonical JSON schemas for both the envelope and individual diagnostics live in [specs/18-schemas.md](18-schemas.md).

Native-JSON reporting command clarification:
- schema v1's native-JSON reporting commands are `kali effects` and `kali package-effects`
- they emit raw JSON payloads on stdout by default on success
- when they fail **without** `--output json`, their diagnostics stay human-oriented and should go to stderr so stdout does not become mixed text+JSON
- callers that need machine-readable failure diagnostics for those commands must request `--output json`

Terminology note:
- the compiler's internal `Span` is a byte-offset range used by the parser/AST/IR
- the JSON diagnostic `span` is a `SourceSpan` with `file`/`line`/`column` fields derived from that internal span
- if a JSON diagnostic also includes a top-level `file`, it is only a convenience mirror of `span.file`, not a second canonical location field
- when a diagnostic depends materially on the merged command/config state rather than only source text, the JSON form should also populate the optional `context` object from [specs/18-schemas.md](18-schemas.md) so tools do not have to recover effective values from free-form prose notes alone

## Error Code Ranges

| Range | Category |
|-------|----------|
| E0xxx | Internal compiler errors |
| E1xxx | Type errors |
| E2xxx | Syntax errors |
| E3xxx | Name resolution errors |
| E4xxx | Sandbox/effect violations |
| E5xxx | Import/module/availability errors |
| E6xxx | Runtime errors |
| E7xxx | Memory/ownership errors |
| W1xxx | Type warnings |
| W2xxx | Style/lint warnings |
| W3xxx | Performance warnings |

## Error Categories

### Type Errors (E1xxx)
- `E1001`: Type mismatch (assignment, argument, return)
- `E1002`: Property does not exist on type
- `E1003`: Cannot invoke non-function type
- `E1004`: Missing required property
- `E1005`: Argument count mismatch
- `E1006`: Generic constraint not satisfied
- `E1007`: Cannot use 'as' to convert between unrelated types
- `E1008`: Effect type mismatch
- `E1009`: Purity violation (side effect in pure function)

### Syntax Errors (E2xxx)
- `E2001`: Unexpected token
- `E2002`: Unterminated string literal
- `E2003`: Invalid regular expression
- `E2004`: Duplicate parameter name
- `E2005`: Invalid assignment target

### Name Resolution Errors (E3xxx)
- `E3001`: Undefined variable or reference
- `E3002`: Duplicate declaration in same scope
- `E3003`: Cannot access before initialization (TDZ)
- `E3004`: Export not found in module

### Sandbox Errors (E4xxx)
- `E4001`: Effect not permitted by sandbox policy
- `E4002`: API call not permitted
- `E4003`: Resource limit exceeded (compile-time provable)
- `E4004`: Dynamic effect detected (cannot statically verify)

### Import/Module/Availability/Command-Input Errors (E5xxx)
- `E5001`: Module/package not found or no selectable stable release
- `E5002`: Circular dependency detected
- `E5003`: Invalid module specifier
- `E5004`: Dependency state not installed or not materialized for the current lockfile
- `E5005`: Ambiguous resolution or registry-path conflict
- `E5006`: Feature unavailable in current phase, API surface, command/profile, or target configuration
- `E5007`: Invalid primary command input kind for the selected command
- `E5008`: Invalid CLI usage or flag/arity combination for the selected command
- `E5009`: Invalid project configuration
- `E5010`: Invalid sandbox policy file
- `E5011`: Cannot prove a statically known export surface for a library-oriented build

Use `E5004` for dependency-state problems such as:
- project dependency inputs (`kali.json` registry dependencies, `kali.json#imports`, or source-level raw URL imports from the install-time project discovery set) have not been installed/materialized yet
- `kali.lock`, `node_modules/`, or `.kali/cache/urls/` is missing/stale for the dependency kinds the project uses
- the current declared dependency graph, lockfile entries, and required materialized artifacts no longer agree
- a file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) reaches additional raw URL imports from explicit files outside the last installed project discovery set
- the resolver needs explicit dependency installation/synchronization instead of silently re-resolving during `check`, `effects`, `build`, `run`, or `test`

Clarification:
- for `E5004`, "stale" is a **declaration/lock/materialization mismatch**, not a vague timestamp heuristic
- non-install commands should fail clearly and point to `kali install`; they should not repair dependency-owning manifest fields, lock state, or materialized dependency state as a side effect

Use `E5001` for module/package-not-found-or-no-selectable-stable-release problems such as:
- a referenced module or package cannot be found under the documented resolution rules
- an identity-only registry-target workflow (`kali install <pkg>`, `kali install --dev <pkg>`, `kali package-effects <pkg>`, `kali package-audit <pkg>`) found the package identity, but no non-yanked stable release exists to satisfy the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md)

Use `E5005` for resolution ambiguity problems such as:
- two candidate package/module edges remaining equally valid after applying the documented resolution rules
- a manifest/import setup that would require two distinct registry identities to collapse onto the same early-phase `node_modules` package path
- any other situation where Kali cannot pick one faithful resolution target without inventing extra precedence rules not defined by the spec

### Canonical Feature-Maturity Diagnostic

Phase-gated or profile-gated features should share one primary diagnostic shape instead of inventing per-command or per-runtime wording.

Terminology rule:
- prefer the canonical term **API surface** (`deno`, `node`, `browser`) from [SPEC.md](../SPEC.md)
- use **profile** for command/runtime-profile gating such as browser build-only paths or later `--wasm-threads`

Example:
```
error[E5006]: feature unavailable in current phase: --api node
  --> <cli>:1:1
  |
  = note: Node.js API compatibility is a Phase 3 target
  = help: use --api deno for Phase 1, or enable the documented later-phase compatibility path
```

Use `E5006` for cases such as:
- `--api node` before the documented Node subset is implemented
- `eval` / `Function()` without `--compat eval`
- dynamic `require()` in early phases
- `run --api browser` in early phases where browser support exists only as an analysis/build context
- `--wasm-threads` before the threaded runtime profile exists, or on targets that cannot support it
- `--max-spawned-processes N` with a non-zero value before subprocess support exists for the selected command/profile/API surface
- an attached sandbox policy trying to enable a real capability/profile that exists in the spec set but is unavailable in the current phase/profile/api surface or effective compatibility context (for example `effects.eval: true` before the eval path exists, `effects.eval: true` without effective `--compat eval`, or browser-targeted `check` / `build --bundle` policies that set browser-incompatible resource budgets such as `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, `resources.maxOpenFiles`, or positive `resources.maxSpawnedProcesses` / `resources.maxThreads` values)
- any parse-supported construct that is intentionally not semantically enabled in the current phase/profile

Boundary clarification:
- use `E5006` when the requested feature/profile is real but unavailable in the current phase/profile
- use `E5008` instead when the user combines otherwise-valid flags into a contradictory command shape (for example `kali build --bundle --api node`, where browser bundle mode exists but the selected API surface conflicts with it, or `kali build --api browser` without `--bundle` while browser builds are bundle-only)
- follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): wrong browser build shape (`build --api browser` without the required artifact mode, or browser + library-oriented build modes) is `E5008`, while requesting a browser execution/test contract that does not exist yet (`run --api browser`, `test --api browser`) is `E5006`
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): diagnostics report the outermost failing gate first — command-shape contradictions before maturity gates, and a command's own availability gate before narrower inherited-context/profile gates inside that command
- maturity-matrix rows that name the *earliest fully supported phase* for a combined command/context shape do not override this precedence rule; for example, `kali build --capi --api node ...` may be summarized as a Phase 3 combination while still reporting the `--capi` gate first in Phase 1
- a well-formed policy file that is semantically incompatible with the selected command/profile/api surface still falls on the `E5006` side of this boundary
- malformed project config should use `E5009`; malformed policy JSON, unknown policy fields, or invalid policy numeric/path/pattern shapes should use `E5010`; export-surface proof failures for library-oriented builds should use `E5011`
- the same rule applies when the triggering value came from discovered config rather than a literal CLI flag; diagnostics should explain the effective value instead of pretending no selection was made
- in JSON mode, prefer filling structured diagnostic `context` metadata (`origin`, `configPath`/`flag`, and `effectiveValue` when useful) in addition to any human-oriented prose notes

Clarification:
- use `E5006` for **documented feature/profile gating**
- use ordinary type/name diagnostics instead when user code simply references a global that is not present in the selected ambient surface (for example `document` under `--api deno` should normally be a regular unresolved-name/type error, not a feature-maturity error)

### Canonical Invalid-Entrypoint Diagnostic

Use `E5007` when the user passes a file/input kind that the selected command fundamentally cannot treat as its required primary source input, even though the file itself may still be meaningful elsewhere in the toolchain.

Boundary rule:
- `E5007` is for **input-kind mismatch** (for example a declaration-only file passed where an executable/analyzable runtime entrypoint or other command-required primary source input is required)
- missing required inputs, too many explicit direct-input arguments, conflicting build artifact-mode selectors (for example `--bundle --lib`), or other command-usage/arity mistakes should use the canonical CLI-usage diagnostic `E5008` instead of overloading `E5007`
- in the CLI exit-code model, those command-usage cases and `E5007` both typically exit with code `5`, even though `E5007` remains the structured diagnostic for the input-kind mismatch case

Example:
```
error[E5007]: invalid primary input for command 'run': declaration-only file
  --> types.d.ts:1:1
  |
  = note: declaration files participate in type checking and ambient typing, but they are not valid executable or analyzable primary inputs for this command
  = help: use 'kali check types.d.ts' to validate declarations, or pass an executable source file to 'kali run'
```

Use `E5007` for cases such as:
- `kali run types.d.ts`
- `kali build defs.d.mts`
- `kali effects defs.d.cts`
- `kali test foo.test.d.ts`
- any other direct command input where the selected command requires an executable/analyzable runtime entrypoint or other command-required primary source input but the supplied file is declaration-only

Clarification:
- `E5007` is about **input-kind mismatch**, not phase gating or general CLI misuse
- module-resolution issues inside an otherwise valid program still use the ordinary `E5001`-`E5005` family

### Canonical Invalid-Config Diagnostic

Use `E5009` when the discovered `kali.json` is malformed or semantically invalid independent of the command's source inputs.

Boundary rule:
- `E5009` is for **project configuration shape/content errors**, not for CLI-usage mistakes and not for a well-formed config that merely selects a phase-gated feature
- if the config is structurally valid but its effective value selects an unavailable documented feature/profile, use `E5006` instead
- if the config is valid but combines values into an impossible command shape for the selected invocation, use `E5008`

Use `E5009` for cases such as:
- malformed `kali.json`
- unknown config keys or wrong value types in `kali.json`
- duplicate/invalid entries in set-like config arrays such as `compilerOptions.runtimeProfiles` or `compat.features`
- invalid `imports`, `include`, or `exclude` field shapes that violate the documented schema

### Canonical Invalid-Policy Diagnostic

Use `E5010` when an attached `kali.policy.json` is malformed or violates the documented policy schema.

Boundary rule:
- `E5010` is for **policy-file syntax/schema/shape/range errors**, not for a well-formed policy that requests a real but unavailable capability/profile
- if the policy is well-formed but tries to enable a feature that is phase-gated or unavailable in the effective command context, use `E5006`

Use `E5010` for cases such as:
- malformed `kali.policy.json`
- unknown policy keys or wrong value types
- invalid allowlist entry shapes or invalid path/URL matcher syntax under the schema-v1 matcher rules
- invalid numeric values such as `resources.maxMemoryMB = 0` or `effects.timer.maxActiveTimers = 0`

### Canonical Export-Surface-Proof Diagnostic

Use `E5011` when a library-oriented build mode is selected but Kali cannot prove one **statically known export surface** for that build.

Boundary rule:
- `E5011` is for **export-surface proof failure**, not for phase gating and not for contradictory CLI usage
- use `E5006` when the requested library-oriented mode itself is unavailable in the current phase/profile/API surface
- use `E5008` when the command shape is contradictory before export analysis even begins (for example `kali build --lib --api browser ...`)
- once a library-oriented mode is otherwise valid, failing to prove one fixed export surface is an `E5011` semantic build rejection rather than a maturity error

Example:
```
error[E5011]: cannot prove a statically known export surface for library-oriented build
  --> lib.cjs:1:1
  |
  = note: CommonJS exports vary dynamically across execution paths
  = help: rewrite the module to one fixed export set, or use an executable-oriented build mode instead
```

Use `E5011` for cases such as:
- dynamic CommonJS export mutation that prevents one fixed export set from being proved for `--lib`, `--capi`, or `--component`
- reflection-heavy export assembly that would require Kali to synthesize host-visible exports it cannot justify from static analysis
- any other library-oriented build where Kali would otherwise need reflective export discovery instead of the required explicit export surface

### Canonical Invalid-Usage Diagnostic

Use `E5008` when the command line itself is malformed for the selected command, even though the requested feature may otherwise exist.

Boundary rule:
- `E5008` is for **CLI/config usage shape errors**, not language/runtime maturity gating and not malformed config/policy files
- use `E5006` when the user asked for a documented feature/profile that exists in the spec set but is unavailable in the current phase/profile
- use `E5007` when the problem is the supplied input kind rather than the overall command shape
- use `E5009` / `E5010` for malformed config / policy files respectively, and `E5011` for library-export proof failures
- config-derived contradictions count too: if discovered config makes the effective command shape impossible (for example `apiSurface = browser` for plain early-phase `kali build main.ts` without `--bundle`), the diagnostic is still `E5008`

Example:
```
error[E5008]: invalid command usage: conflicting build artifact modes '--bundle' and '--lib'
  --> <cli>:1:1
  |
  = help: choose exactly one artifact mode: default executable, --bundle, --lib, --capi, or --component
```

Use `E5008` for cases such as:
- `kali run` with no explicit entrypoint
- `kali build a.ts b.ts` in early phases where `build` is a single-entry direct command
- `kali effects --sandbox kali.policy.json main.ts`
- `kali install foo bar`
- `kali install --dev`
- `kali install --api deno`
- `kali install --allow-scripts` in a URL-only / JSR-only / no-npm project graph
- `kali install --allow-scripts https://example.com/mod.ts`
- `kali install --allow-scripts jsr:@std/path`
- `kali install --dev https://example.com/mod.ts`
- `kali run --max-memory 0 main.ts`
- `kali run --max-cpu 0 main.ts`
- `kali run --max-open-files 0 main.ts`
- `kali package-effects`
- `kali package-effects lodash react`
- `kali package-effects https://example.com/mod.ts`
- `kali package-effects ./local.ts`
- `kali package-effects --api browser lodash`
- `kali package-effects --wasm-threads lodash`
- `kali package-effects --compat eval lodash`
- `kali package-effects --sandbox kali.policy.json lodash`
- `kali package-audit --api browser lodash`
- `kali package-audit --wasm-threads lodash`
- `kali package-audit --compat eval lodash`
- `kali package-audit --sandbox kali.policy.json lodash`
- `kali package-audit` with no package argument
- `kali check --pretty` without `--output json`
- `kali package-audit --pretty lodash` without `--output json`
- `kali package-audit lodash react`
- `kali package-audit https://example.com/mod.ts`
- `kali package-audit ./local.ts`
- `kali check ../shared/main.ts` when that path escapes the effective project root
- `kali check packages/child/main.ts` from a parent project when `packages/child/` has its own `kali.json`
- `kali build --bundle --api node main.ts`
- `kali install --sandbox kali.policy.json`
- `kali fmt --sandbox kali.policy.json`
- `kali lint --sandbox kali.policy.json`
- `kali init --sandbox kali.policy.json`
- `kali init` when the current working directory already contains `kali.json`
- conflicting artifact-mode selectors such as `--bundle --lib`, `--bundle --capi`, or `--lib --component`
- other command-local flag/arity combinations that the CLI contract rejects independently of feature maturity

Clarification:
- `E5008` is for **invalid command shape**, not unsupported language/runtime semantics
- commands should still emit the normal versioned diagnostic/envelope structure in JSON mode rather than printing ad hoc usage text only
- where config caused the invalid shape, the help text should name the relevant config path (for example `compilerOptions.apiSurface`) so the user can fix either the config or the command line
- in JSON mode, prefer including that same information in structured diagnostic `context` metadata instead of leaving it only in prose notes/help text

### Runtime Errors (E6xxx)
- `E6001`: Uncaught exception
- `E6002`: Stack overflow
- `E6003`: Out of memory

### Memory/Ownership Errors (E7xxx)
- `E7001`: Value used after move
- `E7002`: Cannot prove lifetime safety (escaping reference)
- `E7003`: Potential reference cycle detected (info/suggestion)

### Performance Warnings (W3xxx)
- `W3001`: Dynamic object access forces hash map representation
- `W3002`: `eval` usage disables optimizations in scope (when `--compat eval` is enabled)
- `W3003`: Value escapes scope, requiring heap allocation
- `W3004`: Generic function exceeds specialization limit

## Error Principles

### Minimal for AI
Default output shows just what's needed to fix the issue:
- Error code (for programmatic handling)
- One-line message
- Source location with context
- Fix suggestion when available

No ASCII art, progress bars, or decorative elements in default mode.

For unsupported features, prefer one stable code (`E5006`) with a short note naming the required phase/status from [specs/19-feature-maturity.md](19-feature-maturity.md).

### Rich for Humans
With `--verbose` or in interactive terminals:
- Color-coded severity (red=error, yellow=warning, blue=info)
- Multi-line code context
- Related information (e.g., "declared here", "first used here")
- Suggested fixes with diff-like format

### Batch Reporting
- Continue compilation after errors (resilient parsing + type checking)
- Report all errors at once (not fail-fast)
- Deduplicate cascading errors (don't report downstream errors caused by a primary error)
- Sort by file, then line number
- Cap at 50 errors by default (`--max-errors N` to change)

## Diagnostic Struct

```rust
struct Diagnostic {
    severity: Severity,          // Error, Warning, Info, Hint
    code: DiagnosticCode,        // E1001, W3002, etc.
    file: FileId,                // Source file
    span: Span,                  // Internal byte-offset span
    message: String,             // Primary message
    labels: Vec<Label>,          // Annotated source spans
    help: Option<String>,        // Suggested fix (text)
    fix: Option<SuggestedFix>,   // Automated fix (structured)
    related: Vec<RelatedInfo>,   // Related locations
    notes: Vec<String>,          // Additional context
    context: Option<DiagnosticContext>, // Optional machine-readable command/config context
}

struct DiagnosticContext {
    origin: DiagnosticOrigin,          // Cli, Config, Default, Source
    config_path: Option<String>,       // e.g. compilerOptions.apiSurface
    flag: Option<String>,              // e.g. --api
    requested_value: Option<JsonValue>,
    effective_value: Option<JsonValue>,
}

struct SuggestedFix {
    message: String,
    edits: Vec<TextEdit>,        // File edits to apply the fix
}
```

`SuggestedFix` enables `kali check --fix` to auto-apply fixes for certain diagnostics.
