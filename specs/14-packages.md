# 14 — Package Management

## Registry Compatibility

Package loading is compile-time first: Kali resolves, analyzes, and links dependency graphs during build/check/run. For normal builds, application code and its static dependencies are emitted as one linked WASM payload rather than a fleet of runtime-linked WASM modules. Output modes may still add companion artifacts such as JS glue, but they do not change the single linked-payload rule.

Ownership rule:
- this chapter owns package-resolution order, install mutability, lock/materialization behavior, and registry/raw-URL dependency rules
- [12 — CLI](12-cli.md) owns command-line flag/arity behavior for `install`, `package-effects`, and `package-audit`
- [19 — Feature Maturity](19-feature-maturity.md) owns whether those package-oriented commands are available in a given phase
- [18 — Schemas](18-schemas.md) owns the machine-readable payloads emitted by package-analysis commands

### Supported Packages
Kali supports registry packages (npm/JSR) that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

Phase simplification:
- **Phase 1 MVP**: packages that do not depend on unsupported Node core modules, native addons, or install-time binary/bootstrap steps, and that fit the linked-artifact model.
- **Phase 3 target**: broader compatibility for packages that expect the `node` API surface and additional Node built-ins.

This keeps the early ecosystem promise realistic: utility libraries, validators, parsers, and many framework packages are in scope early, while Node-host-heavy or binary-bootstrap-heavy packages follow the Node compatibility work.

Install-time binary/bootstrap clarification:
- packages whose normal install/runtime path depends on compiling native code, downloading platform-specific executables, or selecting prebuilt host binaries at install time are outside the Phase 1 compatibility promise even if the top-level package sources are mostly JS/TS
- `--allow-scripts` may permit the hook to run for analysis/installation workflows, but it must not be misread as a promise that Kali supports the resulting native/binary package contract end-to-end

## Canonical Phase-1 Package-Compatibility Interpretation

Early registry-package compatibility needs one explicit simplification so package support and host-mode support do not get conflated:
- **Phase 1 package compatibility is broader than "only Deno-authored packages"**.
- **Phase 1 package compatibility is narrower than "Node mode works"**.

Concretely, a package can be supported in Phase 1 when:
- its code can be resolved statically into the linked-artifact model,
- its module format can be handled by Kali's ESM/CJS pipeline,
- and its runtime needs are satisfied by the documented Phase 1 Web baseline plus Deno-oriented standalone surface.

A package is **not** automatically in scope just because it lives in npm or JSR. If it depends on broader Node globals/core modules or native addons, it stays phase-gated with the rest of Node compatibility.

## Dependency Source Kinds

To keep install, lock, and materialization rules simple, Kali distinguishes only these early source kinds:
- **Registry packages** — npm and JSR packages declared in `kali.json` under `dependencies` / `devDependencies`, resolved by package name/version, and materialized into `node_modules/`
- **Raw URL imports** — exact `https://...` dependencies declared in source code or `kali.json#imports`, cached under `.kali/cache/urls/`

Clarification:
- path/local alias rewrites in `kali.json#imports` are not a third dependency source kind; they are source-organization rewrites that do not create separate external lock/materialization state.
- schema v1 `kali.json#imports` stays in the URL/path-rewrite lane: it may target raw URLs or path/local rewrites, but it must not alias registry packages or canonical registry identifiers such as `jsr:@std/path`.

### Canonical Registry Package Identifiers

Kali uses one shared registry-package identifier grammar across `kali.json`, `kali install`, package-analysis commands, and lockfile provenance:
- **npm packages** use the normal bare package name, for example `lodash` or `@types/node`
- **JSR packages** use an explicit `jsr:` prefix, for example `jsr:@std/path`

Interpretation rules:
- bare package names default to the npm registry in CLI/package-manifest contexts
- the `jsr:` prefix is required for JSR so package identity stays unambiguous in `kali.json`, lockfiles, diagnostics, and install commands
- this prefix is a **registry identity marker**, not a request to invent a second installation layout; both npm and JSR registry packages still materialize into `node_modules/` in early phases
- the canonical on-disk materialization path is `node_modules/<package-name>` using the registry-native package name without the `jsr:` identity marker; for example npm `lodash` materializes at `node_modules/lodash`, and `jsr:@std/path` materializes at `node_modules/@std/path`
- because early phases use one shared `node_modules/` tree, Kali must reject a project that would require two distinct registry identities to occupy the same on-disk package path (for example npm `@scope/name` and `jsr:@scope/name`) rather than inventing shadow package trees or ambiguous resolution precedence
- docs and examples should prefer this canonical form instead of relying on context to guess whether `@scope/name` came from npm or JSR

Declaration-model rule:
- registry dependencies belong in the project manifest
- raw URL dependencies belong in source/import maps, not in a second manifest dependency table
- `kali install https://...` is therefore a pin/materialize workflow for the shared lock/cache model, not a request to invent a new top-level manifest section
- ad hoc raw-URL installs are a **staging/pin convenience**, not a second durable declaration channel; durable raw URL dependencies still belong in source imports or `kali.json#imports`
- because raw URL state is owned by the current source/import-map graph instead of a manifest dependency table, plain `kali install` may prune raw URL lock/cache entries that are no longer referenced

Lockfile rule:
- `kali.lock` is the canonical reproducibility record for **both** source kinds
- registry packages and raw URL imports may use different on-disk materialization locations, but they share one lock discipline
- non-install commands must check the required materialized state for the dependency kinds actually used by the project instead of assuming `node_modules/` alone is always the full dependency state

This removes an ambiguity from the earlier wording: a URL-only project may have no `node_modules/` tree at all and still be fully installed.

Registry-collision simplification rule:
- if two manifest entries would collapse to the same `node_modules` package path after stripping the optional `jsr:` registry marker, `kali install` must fail explicitly before materialization
- the failure should name both conflicting package identities so the user can choose one source of truth
- early phases prefer this explicit rejection over a more complex multi-registry shadow layout

### Canonical stable-release selection rule (schema v1)

Several early schema-v1 workflows intentionally accept the **identity-only registry target** form from [SPEC.md](../SPEC.md) instead of an inline version/range selector. To keep those workflows deterministic, they share one resolution rule:
- **latest non-yanked stable published version** means the highest published SemVer version for that package identity that has **no prerelease identifier** and is not yanked
- those identity-only workflows must fail explicitly rather than silently selecting a prerelease when no non-yanked stable version exists
- the canonical failure path for that case is `E5001`: the package identity resolved, but no acceptable stable release existed for the schema-v1 identity-only workflow

Schema-v1 uses this rule for:
- registry-analysis commands such as `kali package-effects <pkg>` and `kali package-audit <pkg>`
- explicit registry-package adds via `kali install <pkg>` and `kali install --dev <pkg>`

Install simplification:
- when `kali install <pkg>` or `kali install --dev <pkg>` adds a new manifest entry from a package-identity-only argument, it resolves that latest non-yanked stable published version, writes the lockfile using that concrete resolved version, and records the dependency in `kali.json` using the canonical default manifest range `^<resolvedVersion>`
- later explicit version/range selectors may be added, but they must be introduced as a separate documented CLI/input mode rather than inferred implicitly from the identity-only form

### Package Resolution
Follow Node.js-style package resolution, but keep the early-phase rules explicit so browser, Deno, and package behavior do not drift.

Canonical early-phase code-resolution ladder:
1. Apply import-map rewrites from `kali.json#imports` before package resolution.
2. Preserve any explicit registry qualifier on the package specifier (for example `jsr:@std/path`) so later resolution, lockfile lookup, and diagnostics keep the same package identity.
3. Resolve package self-references (`"name": "pkg"` imported as `pkg/...`) using the package's own `exports` map before walking upward into `node_modules`.
4. If the specifier names a package or package subpath, consult `package.json#exports` first.
5. Evaluate `exports` against the current API surface and edge kind:
   - distinguish **ESM import edges** from **CJS require edges**
   - resolve the exact requested subpath; do not flatten subpath exports into one package-wide entry
   - use the canonical condition order table below
   - unsupported or unmatched conditional branches are skipped; Kali should not guess a fallback branch that the package did not publish
6. If `exports` does not resolve the entry, fall back to legacy entry fields using the same API-surface intent **and still respecting edge kind**:
   - Phase-1 simplification: the supported `browser` and `deno` contexts share the same legacy fallback order, so keep one rule instead of near-duplicate per-surface ladders
   - supported browser-targeted context (Phase 1: `kali check --api browser` and `kali build --bundle --api browser`; later supported browser-targeted analysis commands such as `kali effects --api browser` and browser-context `kali package-effects`) and the Deno-oriented standalone API surface (`--api deno`, Phase 1 default): for **ESM import edges** prefer `module`, then `main`, and for **CJS require edges** prefer `main`, then `module`
   - later Node API surface may add `node`-specific behavior before that shared fallback ladder when explicitly documented
7. In browser-targeted contexts, after `exports` or the legacy fallback picks a package-published target, apply any `package.json#browser` replacement-map rewrite that covers that selected package-local path:
   - this rewrite layer is part of the one shared browser package-selection rule for `check --api browser`, `build --bundle --api browser`, and later browser-context analysis commands such as `effects --api browser` and inherited browser-context `package-effects`
   - if the browser map rewrites the selected path to another package-local file, continue resolution from that rewritten target
   - if the browser map marks the selected path as unavailable (`false`), reject that edge instead of probing alternate non-browser files heuristically
   - this browser-map stage refines the already chosen browser-targeted package edge; it does not restart package resolution under a second ad hoc condition-order algorithm
8. Resolve relative/file entries with extension probing (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`).
9. Classify the resolved file as ESM or CJS using the canonical early-phase rule set:
   - `.mts` / `.mjs` → always ESM
   - `.cts` / `.cjs` → always CommonJS
   - `.ts` / `.tsx` / `.js` / `.jsx` inside a package boundary follow the nearest applicable `package.json#type`
   - when those ambiguous extensions appear outside an applicable package boundary, default to ESM unless the documented resolver/classifier rules require a specific CommonJS interpretation
   - the chosen module kind for a resolved file is shared by resolution, checking, and lowering; Kali must not let one subsystem treat the same file as ESM while another treats it as CJS

Canonical `exports` condition order:

| Analysis/runtime context | Condition order |
|---|---|
| Deno-oriented standalone API surface (`--api deno`, Phase 1 default) | `deno`, then edge kind (`import` or `require`), then `default` |
| browser-targeted context *(Phase 1: `check --api browser`, `build --bundle --api browser`; later supported browser-targeted analysis commands reuse the same order)* | `browser`, then edge kind, then `default` |
| later Node API surface | `node`, then edge kind, then `default` |

Phase-1 simplification:
- only the canonical conditions above plus `default` are part of the early stable resolution contract
- if a package's `exports` tree requires additional environment conditions to choose a branch faithfully, Kali should reject that edge with the canonical availability path instead of guessing bundler-specific precedence

Important separation rules:
- runtime/code resolution must not treat `types` as a normal execution condition
- the Deno-oriented standalone surface should honor a package's explicit `deno` condition when present instead of behaving like an unspecified generic bundler
- `--api node` package resolution is part of the same Phase 3 Node-compatibility gate as the rest of the Node API surface; early phases should not resolve packages as though Node mode were already implemented for `check` or `build`
- the browser-targeted analysis/build context should honor a package's explicit `browser` condition and any applicable `package.json#browser` replacement-map rewrite consistently across every supported browser-targeted command so analysis and emitted artifacts do not resolve different files by accident
- `package.json#module` is treated only as a legacy bundler-compatibility fallback when `exports` is absent; it must not override an explicit `exports` map, and it should not outrank `main` on a legacy CJS `require` edge
- when a package explicitly marks a path as unavailable for the active profile (for example `browser: false`), Kali must respect that instead of probing alternate files heuristically
- declaration/type lookup follows the separate ladder in [Type Resolution](#type-resolution)

To keep configuration simple, `kali.json#imports` is the canonical aliasing mechanism in early phases. A separate TypeScript-style `paths`/`baseUrl` compatibility layer may be added later if ecosystem pressure justifies it, but it is not part of the MVP contract.

Import-map boundary rule:
- `kali.json#imports` may rewrite to raw URLs or path/local targets only.
- It must not be used to alias one registry package to another bare specifier or to a canonical registry identifier such as `jsr:@std/path`.
- Registry ownership stays in `dependencies` / `devDependencies` so install, lockfile provenance, diagnostics, and package-analysis commands all have one source of truth.

Canonical `kali.json#imports` matching rules (schema v1):
- keys without a trailing `/` are **exact-match** rewrites for the full module specifier
- keys with a trailing `/` are **prefix-match** rewrites and apply only when the imported specifier starts with that full prefix
- when multiple keys could match, the **longest matching key wins**
- a prefix key ending with `/` must rewrite to a target that also ends with `/` so the unmatched suffix can be appended without inventing path-join heuristics
- local path targets (`./...`, `../...`, or absolute path-like targets when supported by the host platform) are resolved relative to the directory containing the owning `kali.json`
- raw-URL targets stay absolute after rewrite and then participate in the normal lock/cache materialization flow
- import-map rewrites happen before package resolution; if no import-map entry matches, the original specifier continues into the normal relative/package resolution ladder
- schema v1 does **not** support wildcard/glob/regex import-map keys or targets; exact and prefix rewrites are the whole stable contract

Simplification rule: for any package-resolution edge case not yet modeled faithfully, prefer an explicit `E5006`/availability failure over bundler-style guesswork. This keeps package behavior deterministic and auditable for sandboxed builds.

Practical classifier note:
- package resolution owns the final module-kind decision for a resolved file edge
- parser/checker/codegen must consume that same decision rather than rerunning slightly different heuristics later
- this avoids a common cross-tool drift where `package.json#type`, extension-based classification, and TS/JS frontend assumptions disagree about the same dependency file

### Installation
```bash
kali install lodash                         # Add/install single registry package from npm
kali install jsr:@std/path                  # Add/install single registry package from JSR
kali install                                # Materialize all declared dependencies for the project
kali install --allow-scripts                # Permit lifecycle hooks only for the invocation's effective npm-scriptable install work
kali install --dev vitest                   # Add/install dev dependency
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument semantics are intentionally simple:
- `kali install` takes zero or one explicit **install target** in schema v1
- registry install targets use the canonical registry-package identifier grammar from this chapter (`lodash`, `@types/node`, `jsr:@std/path`)
- in schema v1, explicit registry install targets are **package identities only**, not inline version/range selectors
- adding a registry package through that identity-only CLI form uses the shared [canonical stable-release selection rule](#canonical-stable-release-selection-rule-schema-v1): resolve the latest non-yanked stable published version, refresh `kali.lock` using that concrete version, and record the manifest dependency with the canonical default range `^<resolvedVersion>`
- registry install targets therefore mutate `kali.json` (`dependencies` or `devDependencies`) and then refresh lock/materialized state
- in the canonical configless project mode from [SPEC.md](../SPEC.md), an explicit registry-package add (`kali install <pkg>` or `kali install --dev <pkg>`) first creates the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then records the dependency there; registry-package adds therefore stay on one manifest-based declaration path even in configless directories
- `--dev` applies only to registry install targets; `kali install --dev https://...` is rejected with `E5008` instead of inventing a raw-URL dev-dependency table
- raw URL install targets update the shared lock/cache state only; they do not invent a second manifest section and should not rewrite source/import-map declarations implicitly
- a raw-URL install is therefore best understood as **pin/materialize this exact URL in the shared dependency state**, not as a request to add a new named dependency kind
- in the canonical configless project mode, an explicit raw-URL install may still create `kali.lock` and `.kali/cache/urls/` state at the effective project root, but it must not create a placeholder manifest by itself
- if that URL is not actually referenced from source or `kali.json#imports`, it is only staged materialization and may disappear on the next plain `kali install`
- plain `kali install` reconciles the current manifest + import graph with `kali.lock`, `node_modules/`, and `.kali/cache/urls/`, and may prune raw URL entries that are no longer reachable from that graph
- in the canonical configless project mode, plain `kali install` is a no-op success when the effective project root contributes no manifest/import/source dependency inputs, and it must not create a placeholder manifest as a side effect
- because install is intentionally profile-agnostic in early phases, `kali install` does **not** take `--api`; passing `--api ...` is invalid command usage (`E5008`), not a request for a second install graph

Install-graph discovery rule:
- because `kali install` usually runs without an explicit primary source input, source-level raw URL imports are discovered from the canonical project-discovery result rather than from one ad hoc command-local source root
- the effective project config/root for that scan is the nearest `kali.json` found by searching the current working directory and then its ancestors; if none exists, install uses the current working directory as the project root
- that install-time scan set is filtered by `kali.json` `include` / `exclude` when present, or by the default project-discovery rules from [SPEC.md](../SPEC.md) when those fields are omitted
- recursive install-time discovery must stop at nested child directories that contain their own `kali.json`; those child roots are separate projects in schema v1
- discovery may use a cheap lexical/module-specifier scan of those files plus `kali.json#imports`; it does not require a full check/build just to decide which raw URLs belong in the lock/cache state
- the install-time scan may include declaration-only files too, because they can own type-only imports that still belong to the project's declared dependency graph
- pruning of raw URL lock/cache entries is judged against this install-time declaration graph, not against arbitrary unrelated files elsewhere in the repository

Installation is **fetch-and-link by default**, not "execute package scripts" by default.

Canonical term:
- **effective npm-scriptable install work** = the subset of the current `kali install` invocation that targets **npm registry packages** and could therefore expose npm lifecycle hooks
- this subset is **invocation-scoped**: it includes the npm package work the current install actually reconciles, including any directly requested npm target and any transitively touched npm dependencies in that same invocation
- raw URL targets and `jsr:` targets are outside this subset in schema v1

To preserve sandbox-first behavior:
- npm lifecycle scripts (`preinstall`, `install`, `postinstall`) are not executed unless the user explicitly opts in with `kali install --allow-scripts`
- `--allow-scripts` applies only to that install invocation; it is not an ambient project default
- pairing `--allow-scripts` with an explicit raw URL install target is invalid command usage (`E5008`) because raw URLs do not expose npm lifecycle hooks
- pairing `--allow-scripts` with an explicit `jsr:` package target is also invalid command usage (`E5008`) in schema v1 because JSR packages do not participate in npm lifecycle-script execution
- with **no explicit install target**, `kali install --allow-scripts` applies only to the invocation's **effective npm-scriptable install work**; if that subset is empty, the command should fail with `E5008` instead of silently acting like plain `install`
- mixed install graphs are still valid: if one invocation touches npm packages plus JSR packages and/or raw URLs, lifecycle scripts may run only for the npm subset while the non-npm subset stays on the normal script-free path
- packages requiring native build steps, postinstall-downloaded executables, or other platform-specific binary/bootstrap artifacts are rejected as unsupported even when lifecycle scripts are enabled
- package metadata and tarballs can still be analyzed before linking

Canonical lifecycle-script boundary:
- lifecycle scripts are an **install-time npm-package hook path**, not part of the ordinary Kali source-program execution model
- enabling `--allow-scripts` does **not** imply `--api node`, broader Node package/runtime compatibility, or coverage by the normal `kali effects` / `kali.policy.json` contract
- raw URL installs stay outside this escape hatch entirely because they have no registry lifecycle-script surface
- top-level project sandbox config is ignored by `kali install`, so lifecycle-script execution is intentionally outside the schema-v1 project-policy model rather than being half-governed by it
- package compatibility claims for normal `check` / `build` / `run` / `test` should therefore not be inflated by the existence of this opt-in installer escape hatch

Uses standard `node_modules/` layout by default for maximum ecosystem compatibility. Kali-specific caches live under `.kali/` instead of inventing a second package tree:
```
node_modules/
├── lodash/
│   ├── package.json
│   └── ...
└── zod/
    └── ...

.kali/
└── cache/
    └── urls/            — Cached URL imports and metadata
```
This simplifies interoperability with existing tools, package metadata, and source layouts.

### Lock File
`kali.lock` — deterministic lockfile stored at the effective project root (that is, beside the effective discovered `kali.json` when one exists, otherwise in the current working directory) and committed to version control. Uses a line-oriented TOML-based format for clean diffs and carries its own format version in the file header rather than a JSON `schemaVersion` field.

Canonical simplification for v1:
- registry packages and raw URL imports share **one** lockfile
- they use separate top-level entry kinds so tools do not have to infer source kind from ad hoc fields
- the lockfile records reproducibility data only; materialization location still follows the documented split (`node_modules/` for registry packages, `.kali/cache/urls/` for raw URLs)

Example:
```toml
# kali.lock v1 — do not edit manually

[[package]]
name = "lodash"
version = "4.17.21"
registry = "npm"
resolved = "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"
integrity = "sha256-..."
dependencies = []

[[package]]
name = "@std/path"
version = "1.0.8"
registry = "jsr"
resolved = "https://jsr.io/@std/path/1.0.8.tgz"
integrity = "sha256-..."
dependencies = []

[[url]]
specifier = "https://deno.land/std@0.220.0/path/mod.ts"
resolved = "https://deno.land/std@0.220.0/path/mod.ts"
integrity = "sha256-..."
```

Interpretation rules:
- `[[package]]` entries are for registry dependencies only and include the originating registry kind (`npm` or `jsr`)
- lockfile package records intentionally keep `registry` and `name` as separate fields for compact diff-friendly storage; the canonical registry identifier used for ordering, diagnostics, and cross-spec references is the derived pair (`lodash` for npm, `jsr:@std/path` for the example JSR entry above)
- `[[url]]` entries are for exact raw URL imports after import-map expansion/pinning
- future lockfile revisions may add optional metadata fields, but they should preserve this top-level split instead of collapsing both source kinds into one ambiguous record shape
- to keep lockfile diffs deterministic, producers should emit `[[package]]` entries sorted by canonical registry identifier, then version; emit `[[url]]` entries sorted by canonical pinned specifier; and sort per-entry dependency lists lexically by canonical dependency identifier
- the canonical registry identifier used for lockfile ordering and diagnostics is the same one used elsewhere in the spec set: npm packages keep their bare name, while JSR packages keep the explicit `jsr:` prefix

## Install-Time vs Command-Time Resolution Boundary

Because package resolution can vary by analysis/runtime context (`--api deno`, browser-targeted analysis/build contexts, and later `--api node`), Kali needs one explicit boundary so `install`, lockfiles, and ordinary commands do not drift.

This is the canonical package-management simplification for early phases: Kali keeps one shared installed package state, then performs the final context-sensitive package-edge choice at command time.

Scope note:
- this boundary is about **project commands** that consume project-managed dependency state (`check`, `effects`, `build`, `run`, `test`)
- single-package registry-analysis commands such as later `package-effects` / `package-audit` stay project-independent for version selection and do not consult the current project's installed dependency state


- `kali install` is **context-agnostic** in Phases 1-3. It locks package versions, fetches/materializes package contents, and records reproducibility data, but it does **not** pre-resolve one permanent `exports`/`browser`/`deno` branch for every future command.
- `check`, `effects`, `build`, `run`, and `test` perform the final **command-time package edge selection** from the already-installed package metadata using the active analysis/runtime context.
- therefore one `kali.lock` and one materialized package tree can serve both the default Deno-oriented standalone path and the supported browser-targeted analysis/build paths (`check --api browser`, `build --bundle --api browser`) without requiring separate per-context installs.
- this is possible because early-phase context differences choose between files that are already present inside the installed package contents; they do not require separate version solves for each supported context.
- if a later feature truly requires context-specific solving or materially different dependency graphs, that complexity must be introduced explicitly in a future lockfile/versioning revision rather than being implied accidentally by Phase 1 package wording.

Practical consequence:
- `kali install` does not take `--api` in early phases, and `compilerOptions.apiSurface` does not cause `install` to write a different lockfile for the same manifest/import graph.
- changing `--api` between `deno` and a supported browser-targeted analysis/build context affects which already-installed package entry files are chosen at command time, not whether the project is considered installed.
- lockfile/cache state belongs to the effective discovered project root; invoking commands from a subdirectory of the same project should still use that one shared `kali.lock`, `node_modules/`, and `.kali/` state rather than inventing nested installs.
- if a later file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) points at explicit files outside the last installed project discovery set and those files reach additional raw URL imports, the command should fail with `E5004` and tell the user to rerun `kali install` after updating the project's discoverable sources or import map.
- this is intentional: explicit file targets bypass discovery filtering for command input selection, but they do not retroactively redefine the install-time declaration graph that owns raw URL lock/cache state.

## Deterministic Install & Resolution Contract

This chapter follows the top-level [canonical dependency-management mutability rule](../SPEC.md): in early phases, `kali install` is the only command that mutates project-managed dependency state.

To keep package behavior predictable across `install`, `check`, `effects`, `build`, `run`, and `test`, Kali uses one simple rule set:
- `kali install` is the command that updates dependency-owning manifest fields in `kali.json` when needed, resolves dependency versions, writes `kali.lock`, and refreshes materialized dependency stores.
- `kali check`, `effects`, `build`, `run`, and `test` consume the existing declaration + lock + materialized dependency state; they must not silently re-resolve packages or mutate project-managed dependency state as a side effect.
- If the project's declared dependency inputs (`kali.json` registry dependencies, `kali.json#imports`, or source-level raw URL imports from the install-time project discovery set) require materialized state that is missing or stale, non-install commands fail with `E5004` and tell the user to run `kali install`.
- Here, "stale" means the current declared dependency graph, the corresponding `kali.lock` entries, and the required materialized artifacts no longer agree. Non-install commands should not try to infer staleness from arbitrary mtimes or repair it opportunistically.
- `node_modules/` is the materialized tree for registry packages (npm/JSR), while `.kali/cache/urls/` is the materialized cache for raw URL imports; `kali.lock` is the canonical reproducibility record for both.
- When declaration inputs, `kali.lock`, and the required materialized dependency state disagree, `kali install` is responsible for reconciling them. Other commands should fail clearly rather than guessing which source of truth to trust.
- `--allow-scripts` affects install-time behavior only; it does not change later `check`/`build`/`run` semantics for an already-installed package graph.
- lifecycle scripts executed during install are outside the normal source-program effect-report/sandbox-policy contract and therefore are not evidence that the installed package graph itself requires those same effects at runtime.

This is an intentional simplification: one command mutates project-managed dependency state, all other commands consume it deterministically. For raw URL imports, the source/import-map graph is the declaration source of truth and the lock/cache are the materialized state derived from it.

Diff-friendliness rule:
- lockfile writers should preserve canonical ordering when rewriting existing `kali.lock`
- equivalent dependency graphs should therefore converge on byte-stable lockfile ordering rather than reflecting fetch order or hash-map iteration order

## Import Styles

### ESM (Primary)
```typescript
import { groupBy } from "lodash";
import { z } from "zod";
```

### URL Imports (Deno-style)
```typescript
import { join } from "https://deno.land/std@0.220.0/path/mod.ts";
```
URL imports are cached in `.kali/cache/urls/`. Integrity is verified against the lock file.

Early-phase simplification:
- a URL import used by source code participates in the same lockfile discipline as registry packages
- URL-only projects may therefore have an empty or absent `node_modules/` tree without being considered uninstalled
- non-install commands do **not** repair or repopulate missing URL materialization on the fly; a missing `.kali/cache/urls/` entry is treated as missing dependency state and should fail with `E5004`
- refreshing or first-time pinning of URL dependencies belongs to `kali install` or another explicit dependency-management workflow, not to ordinary compilation

### Import Maps
Support import maps in `kali.json`:
```json
{
    "schemaVersion": 1,
    "imports": {
        "std/": "https://deno.land/std@0.220.0/",
        "~/": "./src/"
    }
}
```

Interpretation rule:
- `imports` is part of the canonical dependency declaration path for URL-based and path/local alias resolution
- raw URL dependencies discovered through source code or expanded import-map entries participate in the same `kali.lock` + `.kali/cache/urls/` discipline as direct URL specifiers
- registry dependencies still belong under `dependencies` / `devDependencies`; `imports` is not a second registry manifest
- schema v1 import-map targets are therefore limited to relative/absolute path-like rewrites and raw URLs; rewrites to bare package specifiers or canonical registry identifiers such as `jsr:@std/path` are rejected explicitly instead of creating a shadow registry-declaration path

## CommonJS Compatibility

Baseline CommonJS support is part of the Phase 1 package story, but it is intentionally narrow and compile-time-oriented:
- CJS modules (`require`, `module.exports`) are transformed to ESM at compile time
- `require()` calls with static string arguments → ESM import
- Dynamic `require()` is **not** part of the linked-artifact model for Phases 1-3; it is rejected by default, and any later compatibility path must be documented in [specs/19-feature-maturity.md](19-feature-maturity.md) rather than invented ad hoc here
- `__dirname`, `__filename` → transformed to `import.meta.dirname`, `import.meta.filename`

## Dynamic Imports

To keep the module system aligned with the single-artifact architecture and the canonical dynamic-loading boundary in [SPEC.md](../SPEC.md):
- static `import` is the primary and fully supported path
- literal-string `import("pkg")` is a **Phase 3 target** feature that may be rewritten against the already-linked graph rather than introducing runtime WASM module linking
- non-literal `import(expr)` is a **later compatibility** path, treated as a dynamic effect boundary and rejected by default in early phases unless the documented maturity path says otherwise

## Type Resolution

For registry packages, Kali should prefer the strongest sound information available without inventing fresh `any` merely to suppress analysis.

Canonical rule: declaration lookup must follow the **same exact package/subpath edge** chosen by code resolution. Type resolution may consult declaration-specific metadata (`exports` `types`, `types` / `typings`, bundled declarations), but it must not silently type-check one package subpath while runtime resolution executes another.

Type-resolution ladder for a resolved package edge:
1. If the resolved package/subpath publishes declaration-specific `exports` metadata for that exact edge (for example a `types` condition or declaration target associated with the chosen subpath), use it first.
2. Otherwise, for a package-root entry, check the package's own top-level `types` / `typings` field in `package.json`.
3. Apply `typesVersions` if present and relevant to the chosen declaration target.
4. Check for bundled declaration files (`.d.ts`, `.d.mts`, `.d.cts`) alongside the resolved source/entry files for that same package/subpath.
5. Check for `@types/<package>` in dependencies as a fallback when the package does not ship authoritative declarations.
6. If package source is available as JS/TS, run the normal Kali checker/inference pipeline on that package and synthesize module-boundary types from the result.
7. If Kali still cannot justify a precise exported type, fall back to `unknown` at the package boundary with a warning.

Canonical declaration-condition simplification:
- declaration lookup follows the **already chosen code edge**, then refines only within that same subpath/branch for declaration metadata
- if an `exports` object for that exact edge contains a declaration-specific branch, Kali should prefer `types` first, then the active edge-kind branch (`import` or `require`) when it points directly at declarations, then `default`
- API-surface conditions such as `deno`, `browser`, or later `node` are resolved during **code-edge selection** first; declaration lookup should not restart a second independent condition walk that might land on a different subpath
- package-root `types` / `typings` metadata is a fallback only when the resolved edge did not already publish a more specific declaration target

Interpretation rules:
- declaration lookup is **subpath-aware**: package-root metadata must not override a more specific declaration target published for the requested subpath
- bundled package types win over `@types` because they are the package author's authoritative declarations
- `typesVersions` refines selection within the package's own declaration ownership; it does not outrank an exact subpath declaration target
- runtime/code resolution must still ignore declaration-only metadata; the separation is that runtime picks the code edge first, then type resolution finds the matching declaration edge
- explicit `any` from upstream declarations is preserved as authored
- synthesized package-boundary `unknown` follows the same conservative fallback philosophy described in [specs/04-type-system.md](04-type-system.md)
- a later loose-compatibility mode may offer broader `any`-style interop, but that must be an explicit opt-in rather than the default package contract

## Registry

To keep schema v1 small and avoid undocumented config surface area, early-phase registry configuration is intentionally narrow:

- Default npm registry: `https://registry.npmjs.org`
- Early override path for the npm registry: `KALI_REGISTRY` environment variable
- The `jsr:` package namespace keeps using the JSR service; `KALI_REGISTRY` does **not** rewrite `jsr:` package identity into a second configurable registry family
- Per-project registry override fields in `kali.json` are **not** part of schema v1; specs must not imply a config key that the schema does not define
- Private-registry auth/config workflows are a later tooling extension unless/until a schema/CLI revision documents the exact contract
- JSR remains an alternative registry source selected by explicit `jsr:` package identifiers, following the same lock/materialization model as npm packages unless a later phase documents a stronger divergence

## Package Analysis

Independently of project install state, Kali can analyze a registry package through the **registry-analysis commands**.

Status boundary:
- `kali package-effects <pkg>` is a **Phase 2 target** that reuses the shared effect-report contract for one registry package
- `kali package-audit <pkg>` is a **later-compatibility** registry tool and should not be implied by Phase 1-2 compiler/runtime readiness
- the examples below describe the canonical command shape and result contract for these workflows, not an unconditional claim that both commands are already available in Phase 1

```bash
kali package-effects lodash                 # Show effects used by package (Phase 2 target)
kali package-audit lodash                   # Security audit (later compatibility)
kali package-audit --output json lodash     # Standard command envelope only until a dedicated audit payload schema exists
```

Argument-kind simplification:
- `kali package-effects <pkg>` takes **exactly one** explicit registry-package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- `kali package-audit <pkg>` takes **exactly one** explicit registry-package argument in early phases; omitting it or passing more than one package is invalid command usage (`E5008`)
- explicit package arguments for those commands must use the canonical **registry package identifier** spelling from [SPEC.md](../SPEC.md) (`lodash`, `@scope/name`, `jsr:@std/path`)
- early schema-v1 package-analysis commands take the **identity-only registry target** form from [SPEC.md](../SPEC.md), not an inline version/range selector
- version selection follows the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md)
- `kali package-effects` records the resolved version in its machine-readable payload, while early `package-audit` follows the same version-selection rule but does **not** promise command-specific machine-readable version metadata until a dedicated audit payload schema exists
- any later explicit version/range or lock-aware mode must be added as a separate documented selector rather than inferred implicitly
- this project-independence is about dependency state and version selection; `package-effects` may still inherit its analysis context from the **effective command context** as documented elsewhere
- any non-registry target is rejected for these commands in early phases, including raw URLs and local file paths, instead of creating a parallel analysis path that overlaps confusingly with project/import-graph handling
- raw URL dependencies are analyzed through the ordinary project workflow (`kali install` + `kali effects` / `check` / `build`) because their durable declaration source is the source/import-map graph, not a registry package coordinate

Isolation rule:
- follow the shared **registry-analysis project-independence rule** from [SPEC.md](../SPEC.md)
- promoting a package from "analyzed" to "installed dependency" remains the responsibility of `kali install`

Because `kali package-effects` is a Phase 2 target and depends on the shared effect-report pipeline, it should stay clearly unavailable or explicitly experimental until that pipeline lands rather than returning a partial bespoke format.

Canonical output simplification:
- `kali package-effects <pkg>` should reuse the same effect vocabulary and `dynamicReasons` contract as `kali effects`
- in schema v1, the analyzed package root is selected by the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md), not from an ambient project-installed copy
- the native payload adds only package-specific metadata (see [specs/18-schemas.md](18-schemas.md)) instead of inventing a second unrelated effect schema
- the nested `report.entryPoints` field should identify the package-analysis logical root with the same canonical registry identifier spelling the user targeted (`lodash`, `jsr:@std/path`) rather than an opaque tarball URL, extracted cache path, or internal package ID
- the nested shared effect report includes `analysisContext` so the chosen `apiSurface`, `runtimeProfiles`, and emitted JSON field `compatFeatures` (the flattened report form of config key `compat.features`; see [SPEC.md](../SPEC.md)) travel with the report instead of living only in ambient CLI/config state
- that nested `analysisContext` uses the schema field names `apiSurface`, `runtimeProfiles`, and `compatFeatures`, so downstream tools do not have to translate from ambient config terminology
- in early phases, that package-analysis context is inherited from the **effective command context** rather than from a second package-analysis-only analysis-context flag family (`--api`, runtime-profile flags, or `--compat`)
- in configless project mode, that inherited context is therefore just the schema-v1 defaults (`apiSurface = deno`, `runtimeProfiles = []`, `compat.features = []`); choosing a non-default package-analysis context requires real config, not package-analysis-specific CLI escape hatches
- because of that design, `kali package-effects` does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` in early phases; passing them is invalid command usage (`E5008`) unless a later spec explicitly adds those flags
- inherited analysis context follows the same axis-specific maturity gates as the rest of effect analysis rather than a package-only shadow rule set: browser inherits the browser-targeted analysis path, Node inherits the Node analysis gate, `wasm-threads` inherits the threaded-profile gate, and compat features such as `eval` inherit their own compatibility gate
- if inherited config/default analysis context selects a mode that is still unavailable for `package-effects`, the command should fail with `E5006` rather than silently analyzing under a smaller fallback context
- inherited `apiSurface = browser` is the intended browser-targeted package-analysis mode once `kali package-effects` exists in Phase 2; it reuses the same browser package-selection context as `kali check --api browser` without adding a second package-analysis-only flag family
- the nested shared effect report still summarizes the full statically reachable package graph selected for analysis under that recorded context; it is not just a manifest-level metadata report
- `--output json` wraps that payload in the standard CLI command envelope; it does not create a third package-effects-only outer format

`kali package-audit` is a later-compatibility tooling feature rather than a core compiler/runtime milestone.

Simplification rules:
- keep it **single-package** in early phases so it does not overlap with a future whole-project dependency-health workflow
- like `package-effects`, it does **not** take package-analysis-specific analysis-context flags (`--api`, runtime-profile flags such as `--wasm-threads`, or `--compat`) or `--sandbox` unless a later spec explicitly adds them
- unlike `package-effects`, early `package-audit` is **context-free**: inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox` do not change its semantics, whether the command runs under discovered config or in configless project mode
- in schema v1, its package target is selected by the shared **stable-release selection rule (schema v1)** from [SPEC.md](../SPEC.md) rather than from any ambient project lockfile selection
- if unimplemented, Kali should say so explicitly instead of implying a partial audit guarantee
- until a dedicated audit payload schema exists, `package-audit --output json` uses the standard command envelope alone and must not smuggle package/version metadata through ad hoc payload fields or by repurposing `stdout` / `stderr` as hidden result channels
- once a dedicated machine-readable audit payload is added, it should still travel through the same standard `--output json` command envelope instead of inventing a second native bare-JSON format

This integrates with the effect system — know what a dependency does before you use it.
