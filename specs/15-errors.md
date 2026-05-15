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
error[E5101]: Type 'string' is not assignable to type 'number'
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

Native-JSON command clarification:
- schema v1's **native-JSON commands** are `kali effects` and `kali package-effects` once those commands are available in the current phase
- they emit raw JSON payloads on stdout by default on success
- when they fail **without** `--output json`, their diagnostics stay human-oriented and should go to stderr so stdout does not become mixed text+JSON
- callers that need machine-readable failure diagnostics for those commands must request `--output json`

Terminology note:
- the compiler's internal `Span` is a byte-offset range used by the parser/AST/IR
- the JSON diagnostic `span` is a `SourceSpan` with `file`/`line`/`column` fields derived from that internal span
- if a JSON diagnostic also includes a top-level `file`, it is only a convenience mirror of `span.file`, not a second canonical location field
- when a diagnostic depends materially on the merged command/config state rather than only source text, the JSON form should also populate the optional `context` object from [specs/18-schemas.md](18-schemas.md) so tools do not have to recover effective values from free-form prose notes alone

## Error Code Ranges

Canonical cross-spec registry:
- **E5xxx** — checker-facing diagnostics, including type checking, syntax/name analysis, runtime/effect semantics, feature availability, and command/input/config validation.
- **E6xxx** — package management diagnostics, including package identity lookup, module/package resolution, and dependency materialization/install-state failures.
- **E9xxx** — sandbox / policy diagnostics.

Expanded public ranges used in schema v1:

| Range | Category |
|-------|----------|
| E0xxx | Internal compiler errors |
| E51xx | Type errors |
| E52xx | Syntax errors |
| E53xx | Name resolution errors |
| E54xx | Effect-system and runtime sandbox semantics |
| E55xx | Feature availability, command/input-shape, and config/policy-shape errors |
| E6xxx | Package management, module/package resolution, and dependency materialization |
| E7xxx | Memory/ownership errors |
| E8xxx | Runtime execution failures |
| E9xxx | Sandbox-policy and compile-time policy-validation diagnostics |
| W1xxx | Type warnings |
| W2xxx | Style/lint diagnostics |
| W3xxx | Performance warnings |

Range clarification:
- Kali intentionally uses both `E54xx` and `E9xxx` in the broader sandbox/effect story.
- `E54xx` is the runtime/effect-semantics side (for example a capability use denied during execution).
- `E9xxx` is the policy-validation side (for example compile-time inferred-effect-vs-policy rejection on sandbox-attached `check` / `build`).
- Keep that split explicit so policy-schema/availability failures do not drift into ad hoc runtime-only wording.
- Package-management failures use the `E6xxx` family even when they surface during non-install commands, so install/materialization issues stay distinct from checker and command-shape failures.

## Error Categories

### Type Errors (E51xx)
- `E5101`: Type mismatch (assignment, argument, return)
- `E5102`: Property does not exist on type
- `E5103`: Cannot invoke non-function type
- `E5104`: Missing required property
- `E5105`: Argument count mismatch
- `E5106`: Generic constraint not satisfied
- `E5107`: Cannot use 'as' to convert between unrelated types
- `E5108`: Effect type mismatch
- `E5109`: Purity violation (side effect in pure function)

### Syntax Errors (E52xx)
- `E5201`: Unexpected token
- `E5202`: Unterminated string literal
- `E5203`: Invalid regular expression
- `E5204`: Duplicate parameter name
- `E5205`: Invalid assignment target

### Name Resolution Errors (E53xx)
- `E5301`: Undefined variable or reference
- `E5302`: Duplicate declaration in same scope
- `E5303`: Cannot access before initialization (TDZ)
- `E5304`: Export not found in module

Clarification:
- if an unresolved identifier or call target survives into lowering/codegen, Kali must report the undefined-reference diagnostic instead of silently emitting an executable placeholder value; the fallback remains an explicit compatibility escape hatch, not a silent success path

### Sandbox, Effects, and Policy Errors (E54xx / E9xxx)
Runtime/effect side:
- `E5401`: Effect not permitted by sandbox policy during runtime enforcement
- `E5402`: API call not permitted
- `E5403`: Resource limit exceeded (compile-time provable)
- `E5404`: Dynamic effect detected (cannot statically verify)
- `E5408`: Dynamic import target is not in the linked graph or cannot be resolved statically

Policy-validation side:
- `E9007`: Inferred effect not permitted by the active sandbox policy during compile-time `check` / `build --sandbox` validation

### Package-Management Diagnostics (E6xxx)
- `E6001`: Module/package not found or no selectable stable release
- `E6002`: Circular dependency detected
- `E6003`: Invalid module specifier
- `E6004`: Dependency state not installed or not materialized for the current lockfile
- `E6005`: Ambiguous resolution or registry-path conflict

### Availability, Command, and Config Diagnostics (E55xx)
- `E5506`: Feature unavailable in current phase, API surface, command/profile, or target configuration
- `E5507`: Invalid primary command input kind for the selected command
- `E5508`: Invalid CLI usage or flag/arity combination for the selected command
- `E5509`: Invalid project configuration
- `E5510`: Invalid sandbox policy file
- `E5511`: Cannot determine a statically known export surface for a library-oriented build

Quick boundary table for the most common `E6xxx` / `E55xx` choice:

| If the problem is primarily... | Use | Why |
|---|---|---|
| missing/stale lock/materialized dependency state | `E6004` | install/materialization state is missing or out of sync |
| a real documented feature/profile/context that exists in the spec but is unavailable here | `E5506` | this is availability gating, not malformed usage |
| the supplied primary input kind is wrong for an otherwise valid command shape | `E5507` | this is an input-kind mismatch |
| CLI/config usage shape is contradictory or malformed before a meaningful availability check | `E5508` | this is command-shape/arity/output-mode misuse |
| `kali.json` itself is malformed or semantically invalid | `E5509` | config schema/content failure |
| `kali.policy.json` itself is malformed or semantically invalid | `E5510` | policy schema/content failure |
| a library-oriented build is otherwise valid but Kali cannot prove one fixed export surface | `E5511` | export-surface determination failure |

Use `E6004` for dependency-state problems such as:
- project dependency inputs (`kali.json` registry dependencies, `kali.json#imports`, or source-level raw URL imports from the install-time project discovery set) have not been installed/materialized yet
- `kali.lock`, `node_modules/`, or `.kali/cache/urls/` is missing/stale for the dependency kinds the project uses
- the current declared dependency graph, lockfile entries, and required materialized artifacts no longer agree
- a file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) reaches additional raw URL imports from explicit files outside the last installed project discovery set
- the resolver needs explicit dependency installation/synchronization instead of silently re-resolving during `check`, `effects`, `build`, `run`, or `test`

Clarification:
- for `E6004`, "stale" is a **declaration/lock/materialization mismatch**, not a vague timestamp heuristic
- non-install commands should fail clearly and point to `kali install`; they should not repair dependency-owning manifest fields, lock state, or materialized dependency state as a side effect

Use `E6001` for module/package-not-found-or-no-selectable-stable-release problems such as:
- a referenced module or package cannot be found under the documented resolution rules
- an identity-only registry-target workflow (`kali install <pkg>`, `kali install --dev <pkg>`, `kali package-effects <pkg>`, `kali package-audit <pkg>`) found the package identity, but no non-yanked stable release exists to satisfy the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md)

Use `E6005` for resolution ambiguity or package-shape/host-fit conflicts such as:
- two candidate package/module edges remaining equally valid after applying the documented resolution rules
- a manifest/import setup that would require two distinct registry identities to collapse onto the same early-phase `node_modules` package path
- a published package whose shape or host assumptions are incompatible with the selected early-phase package-analysis/install path, for example a native addon, a Node-only host-API requirement, or another package-shape conflict that cannot be reconciled without inventing extra precedence rules
- any other situation where Kali cannot pick one faithful resolution target without inventing extra precedence rules not defined by the spec

### Canonical Feature-Maturity Diagnostic

Availability-gated features should share one primary diagnostic shape instead of inventing per-command or per-runtime wording.

Terminology rule:
- prefer the canonical term **API surface** (`deno`, `node`, `browser`) from [SPEC.md](../SPEC.md)
- prefer the shared term **availability context** from [SPEC.md](../SPEC.md) when the gate depends on the command plus its selected API surface, runtime profile, compatibility switches, and current maturity phase
- reserve **profile** for actual runtime-profile gating such as later `--wasm-threads`, not for contradictory browser build shapes that are already handled as command-shape errors

Example:
```
error[E5506]: feature unavailable in current phase: --api node
  --> <cli>:1:1
  |
  = note: Node.js API compatibility is a Phase 3 target
  = help: use --api deno for Phase 1, or enable the documented later-phase compatibility path
```

Use `E5506` for cases such as:
- `--api node` before the documented Node subset is implemented for a command family that still gates it, including `kali check --api node` and `kali build --api node`
- `eval` / `Function()` when the active availability context does not permit that compatibility path yet — both ordinary source use without effective `--compat eval` and an explicit `--compat eval` request before the Phase-4 path exists stay on `E5506`
- dynamic `require()` in early phases
- **recognized-but-unavailable compatibility members** from [SPEC.md](../SPEC.md), such as Phase-1 `Deno.permissions.request()` / `revoke()` and their statically-known string-literal bracketed property forms (these stay on the compatibility-member path, not the ordinary missing-property/type-error path)
- `Deno.permissions.query(...)` asked to evaluate a descriptor kind that Kali intentionally does not support in the current phase/API surface (for example an early-phase `ffi`/`sys`-style permission descriptor)
- `run --api browser` in early phases where browser support is limited to the shared **Phase-1 browser-targeted command set** and there is still no standalone browser runtime contract
- plain `kali run main.ts` or `kali run --sandbox kali.policy.json main.ts` under an inherited `compilerOptions.apiSurface = browser`, because effective-context inheritance must not silently fall back to `deno`; inherited `compilerOptions.apiSurface = node` now uses the supported Node execution subset instead of this gate
- plain `kali check`, `kali check main.ts`, or `kali check --sandbox kali.policy.json` under an inherited `compilerOptions.apiSurface = node`, because inherited checking contexts must hit the same Node availability gate as explicit `--api node` forms instead of silently falling back to `deno`
- before the base `kali effects` command is available, plain `kali effects main.ts` and the command's well-formed JSON-formatting forms (`kali effects --pretty main.ts`, `kali effects --output json main.ts`, `kali effects --pretty --output json main.ts`) still report the command-family availability gate (`E5506`) even if discovered config already selects an inherited browser/Node/runtime-profile/compatibility context; formatting selectors and config inheritance must not silently simplify the request into some smaller ad hoc mode
- once `kali effects` itself exists, inherited context stays axis-aligned with the corresponding explicit flags: inherited `compilerOptions.apiSurface = browser` behaves like explicit `--api browser` rather than a fallback to `deno`, while inherited `compilerOptions.apiSurface = node` now follows the supported Node effect-analysis path and inherited `compilerOptions.runtimeProfiles = ["wasm-threads"]` or `compat.features = ["eval"]` continue to hit the same `E5506` gates as their explicit forms until those later contexts actually ship
- more generally, any participating **source-graph command** (`check`, `effects`, `build`, `run`, `test`) under an inherited `compilerOptions.runtimeProfiles = ["wasm-threads"]` or `compat.features = ["eval"]` must hit the same `E5506` availability gate as the corresponding explicit `--wasm-threads` / `--compat eval` request instead of silently ignoring the inherited profile/compat selection
- plain `kali test` or `kali test --sandbox kali.policy.json` under an inherited `compilerOptions.apiSurface = browser`, for the same reason; inherited `compilerOptions.apiSurface = node` now uses the supported Node test-runtime subset instead of this gate
- plain `kali build main.ts` or `kali build --sandbox kali.policy.json main.ts` under an inherited `compilerOptions.apiSurface = node`, because inherited build contexts must not silently fall back to `deno`
- plain `kali build --lib lib.ts`, `kali build --lib --sandbox kali.policy.json lib.ts`, and later plain `kali build --capi lib.ts` / `kali build --component lib.ts` under an inherited `compilerOptions.apiSurface = node`, for the same inherited-context reason
- well-formed base invocations that hit the shared **registry-analysis availability boundary** from [SPEC.md](../SPEC.md), such as `kali package-effects lodash`, `kali package-effects --pretty lodash`, `kali package-effects --output json lodash`, or `kali package-effects --pretty --output json lodash` before Phase 2, or `kali package-audit lodash`, `kali package-audit --output json lodash`, or `kali package-audit --pretty --output json lodash` before Phase 4 opens
- `--wasm-threads` before the threaded runtime profile exists, or on targets that cannot support it
- positive values for the shared **feature-gated zero-capable execution budgets** from [SPEC.md](../SPEC.md) before the selected command/runtime-profile/API-surface combination actually supports subprocesses or threads
- an attached sandbox policy trying to enable a real capability that exists in the spec set but is unavailable in the current **availability context** (for example `effects.eval: true` before the eval path exists, `effects.eval: true` without effective `--compat eval`, or a browser-targeted `check` / `build --bundle` policy that violates the shared **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md))
- any parse-supported construct that is intentionally not semantically enabled in the current availability context

Boundary clarification:
- follow the shared **support-claim reading order** from [SPEC.md](../SPEC.md): command shape first, then the intended support rung, then the resulting **availability context**
- use `E5506` when the requested feature is real but unavailable in the current **availability context**
- use `E5508` instead when the user combines otherwise-valid flags into a contradictory command shape (for example `kali build --bundle --api node`, where browser bundle mode exists but the selected API surface conflicts with it, or `kali build --api browser` without `--bundle` while browser builds are bundle-only)
- follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): wrong browser build shape (`build --api browser` without the required artifact mode, or browser + library-oriented build modes) is `E5508`, while requesting a browser execution/test contract that does not exist yet (`run --api browser`, `test --api browser`) is `E5506`
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): diagnostics report the outermost failing gate first — command-shape contradictions before maturity gates, and a command's own availability gate before narrower inherited-context/profile gates inside that command
- this applies to registry-analysis commands too: follow the shared **registry-analysis availability boundary** from [SPEC.md](../SPEC.md) instead of re-deciding the `E5508`-before-`E5506` split per command; output-format selectors such as `--output json` also do not create a second availability path for otherwise well-formed `package-effects` / `package-audit` requests
- practical shortcut: `kali package-effects --pretty lodash` is still on the `E5506` side because `package-effects` is a well-formed **native-JSON command** shape once that command exists, while `kali package-audit --pretty lodash` stays on the `E5508` side because schema v1's envelope-only audit mode requires `--output json` before `--pretty` is meaningful
- maturity-matrix rows that name the *earliest fully supported phase* for a combined command/context shape do not override this precedence rule; for example, `kali build --capi --api node ...` may be summarized as a Phase 3 combination while still reporting the `--capi` gate first in Phase 1
- a well-formed policy file that is semantically incompatible with the selected **availability context** still falls on the `E5506` side of this boundary
- malformed project config should use `E5509`; malformed policy JSON, unknown policy fields, or invalid policy numeric/path/pattern shapes should use `E5510`; export-surface determination failures for library-oriented builds should use `E5511`
- the same rule applies when the triggering value came from discovered config rather than a literal CLI flag; diagnostics should explain the effective value instead of pretending no selection was made
- in JSON mode, prefer filling structured diagnostic `context` metadata (`origin`, `configPath`/`flag`, and `effectiveValue` when useful) in addition to any human-oriented prose notes

Clarification:
- use `E5506` for **documented availability gating**
- use ordinary type/name diagnostics instead when user code simply references a global that is not present in the selected ambient surface (for example `document` under `--api deno` should normally be a regular unresolved-name/type error, not a feature-maturity error)

### Canonical Invalid-Entrypoint Diagnostic

Use `E5507` when the user passes a file/input kind that the selected command fundamentally cannot treat as its required primary source input, even though the file itself may still be meaningful elsewhere in the toolchain.

Boundary rule:
- `E5507` is for **input-kind mismatch** (for example a declaration-only file passed where an executable/analyzable runtime entrypoint or other command-required primary source input is required)
- follow the shared **validation-order rule** from [SPEC.md](../SPEC.md): use `E5507` only after the command itself is available and the overall command shape is otherwise valid
- missing required inputs, too many explicit direct-input arguments, conflicting build artifact-mode selectors (for example `--bundle --lib`), or other command-usage/arity mistakes should use the canonical CLI-usage diagnostic `E5508` instead of overloading `E5507`
- in the CLI exit-code model, those command-usage cases and `E5507` both typically exit with code `5`, even though `E5507` remains the structured diagnostic for the input-kind mismatch case

Example:
```
error[E5507]: invalid primary input for command 'run': declaration-only file
  --> types.d.ts:1:1
  |
  = note: declaration files participate in type checking and ambient typing, but they are not valid executable or analyzable primary inputs for this command
  = help: use 'kali check types.d.ts' to validate declarations, or pass an executable source file to 'kali run'
```

Use `E5507` for cases such as:
- `kali run types.d.ts`
- `kali build defs.d.mts`
- `kali test foo.test.d.ts`
- `kali effects defs.d.cts` *(once `kali effects` itself is available; before that, the command-family availability gate still reports `E5506` first)*
- any other direct command input where the selected command requires an executable/analyzable runtime entrypoint or other command-required primary source input but the supplied file is declaration-only

Clarification:
- `E5507` is about **input-kind mismatch**, not phase gating or general CLI misuse
- module-resolution issues inside an otherwise valid program still use the ordinary `E6001`-`E6005` family

### Canonical Invalid-Config Diagnostic

Use `E5509` when the discovered `kali.json` is malformed or semantically invalid independent of the command's source inputs.

Boundary rule:
- `E5509` is for **project configuration shape/content errors**, not for CLI-usage mistakes and not for a well-formed config that merely selects a phase-gated feature
- if the config is structurally valid but its effective value selects an unavailable documented feature in the resulting **availability context**, use `E5506` instead
- if the config is valid but combines values into an impossible command shape for the selected invocation, use `E5508`

Use `E5509` for cases such as:
- malformed `kali.json`
- unknown config keys or wrong value types in `kali.json`
- duplicate/invalid entries in set-like config arrays such as `compilerOptions.runtimeProfiles` or `compat.features`
- invalid registry dependency value shapes in `dependencies` / `devDependencies` (for example a SemVer range where schema v1 requires an exact version string)
- invalid `imports`, `include`, or `exclude` field shapes that violate the documented schema

### Canonical Invalid-Policy Diagnostic

Use `E5510` when an attached `kali.policy.json` is malformed or violates the documented policy schema.

Boundary rule:
- `E5510` is for **policy-file syntax/schema/shape/range errors**, not for a well-formed policy that requests a real but unavailable capability/profile
- if the policy is well-formed but tries to enable a feature that is phase-gated or unavailable in the effective command context, use `E5506`

Use `E5510` for cases such as:
- malformed `kali.policy.json`
- unknown policy keys or wrong value types
- invalid allowlist entry shapes or invalid path/URL matcher syntax under the schema-v1 matcher rules
- invalid numeric values such as `resources.maxMemoryMB = 0` or `effects.timer.maxActiveTimers = 0`

### Canonical Export-Surface Diagnostic

Use `E5511` when a library compile-intent build is selected but Kali cannot determine one **statically known export surface** for that build.

Boundary rule:
- `E5511` is for **export-surface determination failure** on a library compile-intent build, not for phase gating and not for contradictory CLI usage
- use `E5506` when the requested library-oriented mode itself is unavailable in the current phase/profile/API surface
- use `E5508` when the command shape is contradictory before export analysis even begins (for example `kali build --lib --api browser ...`)
- once a library-oriented mode is otherwise valid, failing to determine one fixed export surface is an `E5511` semantic build rejection rather than a maturity error

Example:
```
error[E5511]: cannot determine a statically known export surface for library-oriented build
  --> lib.cjs:1:1
  |
  = note: CommonJS exports vary dynamically across execution paths
  = help: rewrite the module to one fixed export set, or use an executable-oriented build mode instead
```

Use `E5511` for cases such as:
- dynamic CommonJS export mutation that prevents one fixed export set from being determined for `--lib`, `--capi`, or `--component`
- reflection-heavy export assembly that would require Kali to synthesize host-visible exports it cannot justify from static analysis
- any other library-oriented build where Kali would otherwise need reflective export discovery instead of the required explicit export surface

### Canonical Invalid-Usage Diagnostic

Use `E5508` when the command line itself is malformed for the selected command, even though the requested feature may otherwise exist.

Boundary rule:
- `E5508` is for **CLI/config usage shape errors**, not language/runtime maturity gating and not malformed config/policy files
- use the shared **support-claim reading order** from [SPEC.md](../SPEC.md): if the request fails before a meaningful availability check exists, it belongs on the `E5508` / `E5507` side rather than on `E5506`
- use `E5506` when the user asked for a documented feature that exists in the spec set but is unavailable in the current **availability context**
- use `E5507` when the problem is the supplied input kind rather than the overall command shape
- use `E5509` / `E5510` for malformed config / policy files respectively, and `E5511` for library-export proof failures
- output-format flags still obey the shared **JSON-producing mode** rule from [SPEC.md](../SPEC.md): `--pretty` without active JSON output is `E5508`, and `--output json` never creates a second command-availability path
- shorthand: `--pretty` alone is valid for **native-JSON commands** such as `kali effects` / `kali package-effects` once those commands exist, but not for **envelope-only JSON commands** such as schema-v1 `package-audit`
- config-derived contradictions count too: if discovered config makes the effective command shape impossible (for example `apiSurface = browser` for plain early-phase `kali build main.ts` without `--bundle`), the diagnostic is still `E5508`

Example:
```
error[E5508]: invalid command usage: conflicting build artifact modes '--bundle' and '--lib'
  --> <cli>:1:1
  |
  = help: choose exactly one artifact mode: default executable, --bundle, --lib, --capi, or --component
```

Use `E5508` for cases such as:

Registry-analysis shorthand:
- any violation of the shared **single-package registry-analysis command** rule from [SPEC.md](../SPEC.md) is also `E5508`
- follow the canonical **shared flag buckets**, **semantic/context flag surface**, and **JSON-mode selectors** terms from [SPEC.md](../SPEC.md): schema-v1 `package-effects` and `package-audit` keep the package selector as their semantic/context flag surface, may still accept their documented JSON/output selectors, and continue to allow ordinary shared presentation/control flags under the shared CLI rules. Package-analysis-specific `--api` / `--compat` / `--wasm-threads` flags and `--sandbox` are invalid usage unless a later spec adds them

Examples:
- `kali run` with no explicit entrypoint
- `kali build a.ts b.ts` in early phases where `build` is a single-entry direct command
- `kali effects --sandbox kali.policy.json main.ts`
- `kali install foo bar`
- `kali install --dev`
- `kali install --api deno`
- `kali install --allow-scripts` in a URL-only / JSR-only / clean already-synchronized / otherwise no-npm project graph
- `kali install --allow-scripts https://example.com/mod.ts`
- `kali install --allow-scripts jsr:@std/path`
- `kali install --dev https://example.com/mod.ts`
- `kali run --max-memory 0 main.ts`
- `kali run --max-cpu 0 main.ts`
- `kali run --max-open-files 0 main.ts`
- `kali init --api browser`
- `kali fmt --api browser`
- `kali lint --api browser`
- `kali package-effects`
- `kali package-effects lodash react`
- `kali package-effects https://example.com/mod.ts`
- `kali package-effects ./local.ts`
- `kali package-effects --api browser lodash` *(representative of any package-analysis-specific `--api` / `--compat` / `--wasm-threads` flag or `--sandbox` on `package-effects` in schema v1)*
- `kali package-effects "   "`
- `kali package-audit --api browser lodash` *(representative of any package-analysis-specific `--api` / `--compat` / `--wasm-threads` flag or `--sandbox` on `package-audit` in schema v1)*
- `kali package-audit` with no package argument
- `kali package-audit "   "`
- `kali package-effects jsr:`
- `kali package-audit jsr: foo`
- `kali check --fix`
- `kali check --pretty` without `--output json`
- `kali package-audit --pretty lodash` without `--output json`
- `kali package-audit lodash react`
- `kali package-audit https://example.com/mod.ts`
- `kali package-audit ./local.ts`
- `kali check ../shared/main.ts` when that path escapes the effective project root
- `kali check packages/child/main.ts` from a parent project when `packages/child/` has its own `kali.json`
- `kali build --api browser main.ts`
- plain `kali build --sandbox kali.policy.json main.ts` under an inherited browser API surface
- plain `kali build --lib lib.ts` under an inherited browser API surface
- plain `kali build --lib --sandbox kali.policy.json lib.ts` under an inherited browser API surface
- plain `kali build --capi lib.ts` under an inherited browser API surface
- plain `kali build --capi --sandbox kali.policy.json lib.ts` under an inherited browser API surface
- plain `kali build --component lib.ts` under an inherited browser API surface
- plain `kali build --component --sandbox kali.policy.json lib.ts` under an inherited browser API surface
- `kali build --bundle main.ts` under an effective API surface other than `browser`
- `kali build --bundle --sandbox kali.policy.json main.ts` under an effective API surface other than `browser`
- `kali build --lib --api browser lib.ts`
- `kali build --capi --api browser lib.ts`
- `kali build --component --api browser lib.ts`
- `kali build --bundle --api node main.ts`
- `kali install --sandbox kali.policy.json`
- `kali fmt --sandbox kali.policy.json`
- `kali lint --sandbox kali.policy.json`
- `kali init --sandbox kali.policy.json`
- `kali init` when the current working directory already contains `kali.json`
- conflicting artifact-mode selectors such as `--bundle --lib`, `--bundle --capi`, or `--lib --component`
- other command-local flag/arity combinations that the CLI contract rejects independently of feature maturity

Clarification:
- `E5508` is for **invalid command shape**, not unsupported language/runtime semantics
- commands should still emit the normal versioned diagnostic/envelope structure in JSON mode rather than printing ad hoc usage text only
- where config caused the invalid shape, the help text should name the relevant config path (for example `compilerOptions.apiSurface`) so the user can fix either the config or the command line
- in JSON mode, prefer including that same information in structured diagnostic `context` metadata instead of leaving it only in prose notes/help text

### Runtime Errors (E8xxx)
- `E8001`: Uncaught exception
- `E8002`: Stack overflow
- `E8003`: Out of memory
- `E8100`: Internal compiler/runtime invariant failure, including failed IR structural validation in developer-debug mode

### Memory/Ownership Errors (E7xxx)
- `E7001`: Value used after move
- `E7002`: Cannot prove lifetime safety (escaping reference)
- `E7003`: Potential reference cycle detected (info/suggestion)

### Performance Warnings (W3xxx)
- `W3001`: Dynamic object access forces hash map representation
- `W3002`: `eval` usage disables optimizations in scope (when `--compat eval` is enabled)
- `W3003`: Value escapes scope, requiring heap allocation
- `W3004`: Generic function exceeds specialization limit

### Style/Lint Diagnostics (W2xxx)

The initial Phase-1 lint registry uses stable `W2xxx` codes so `kali lint` can emit machine-friendly diagnostics. Unless otherwise noted, the listed rules are warnings; `no-debugger` and `no-unreachable` are hard failures, and `no-console` is warning-severity but off by default.

| Code | Rule | Default severity | Auto-fixable |
|---|---|---|---|
| `W2000` | `no-unused-vars` | warning | no |
| `W2001` | `no-unused-imports` | warning | yes |
| `W2002` | `no-explicit-any` | warning | no |
| `W2003` | `prefer-const` | warning | yes |
| `W2004` | `no-var` | warning | yes |
| `W2005` | `eqeqeq` | warning | yes |
| `W2006` | `no-debugger` | error | yes |
| `W2007` | `no-console` | warning (off by default) | no |
| `W2008` | `no-empty` | warning | no |
| `W2009` | `no-unreachable` | error | no |
| `W2010` | `no-undef` | warning | no |

## Error Principles

### Minimal for AI
Default output shows just what's needed to fix the issue:
- Error code (for programmatic handling)
- One-line message
- Source location with context
- Fix suggestion when available

No ASCII art, progress bars, or decorative elements in default mode.

For unsupported features, prefer one stable code (`E5506`) with a short note naming the required phase/status from [specs/19-feature-maturity.md](19-feature-maturity.md).

### Rich for Humans
With `--verbose` or in interactive terminals:
- Color-coded severity (red=error, yellow=warning, blue=info)
- Multi-line code context
- Related information (e.g., "declared here", "first used here")
- Suggested fixes with diff-like format
- Docs links for error codes in the canonical `https://kali-lang.org/errors/E####` form

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
    code: DiagnosticCode,        // E5101, E6004, E5506, W3002, etc.
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

`SuggestedFix` carries structured edit metadata that tooling can consume without scraping prose. In schema v1, CLI autofix is intentionally lint-only (`kali lint --fix`); checker diagnostics may still emit `SuggestedFix` for editors, embedders, and later checker-autofix work.
