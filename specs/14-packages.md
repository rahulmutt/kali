# 14 — Package Management

## Registry Compatibility

Package loading is compile-time first: Kali resolves, analyzes, and links dependency graphs during build/check/run. For normal builds, application code and its static dependencies are emitted as one linked WASM payload rather than a fleet of runtime-linked WASM modules. Output modes may still add companion artifacts such as JS glue, but they do not change the single linked-payload rule.

### Supported Packages
Kali supports registry packages (npm/JSR) that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

Phase simplification:
- **Phase 1 MVP**: packages that do not depend on unsupported Node core modules and fit the linked-artifact model.
- **Phase 3 target**: broader compatibility for packages that expect the `node` API surface and additional Node built-ins.

This keeps the early ecosystem promise realistic: utility libraries, validators, parsers, and many framework packages are in scope early, while Node-host-heavy packages follow the Node compatibility work.

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

### Canonical Registry Package Identifiers

Kali uses one shared registry-package identifier grammar across `kali.json`, `kali install`, package-analysis commands, and lockfile provenance:
- **npm packages** use the normal bare package name, for example `lodash` or `@types/node`
- **JSR packages** use an explicit `jsr:` prefix, for example `jsr:@std/path`

Interpretation rules:
- bare package names default to the npm registry in CLI/package-manifest contexts
- the `jsr:` prefix is required for JSR so package identity stays unambiguous in `kali.json`, lockfiles, diagnostics, and install commands
- this prefix is a **registry identity marker**, not a request to invent a second installation layout; both npm and JSR registry packages still materialize into `node_modules/` in early phases
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
   - browser-targeted profile (`kali check --api browser` and `kali build --bundle --api browser`): apply `browser` replacement map semantics first where applicable; then for **ESM import edges** prefer `module`, then `main`, and for **CJS require edges** prefer `main`, then `module`
   - Deno-oriented standalone profile (`--api deno`, Phase 1 default): for **ESM import edges** prefer `module`, then `main`, and for **CJS require edges** prefer `main`, then `module`
   - later Node profile may add `node`-specific behavior before the generic fallback ladder when explicitly documented
7. Resolve relative/file entries with extension probing (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`).
8. Classify the resolved file as ESM or CJS using the canonical early-phase rule set:
   - `.mts` / `.mjs` → always ESM
   - `.cts` / `.cjs` → always CommonJS
   - `.ts` / `.tsx` / `.js` / `.jsx` inside a package boundary follow the nearest applicable `package.json#type`
   - when those ambiguous extensions appear outside an applicable package boundary, default to ESM unless the documented resolver/classifier rules require a specific CommonJS interpretation
   - the chosen module kind for a resolved file is shared by resolution, checking, and lowering; Kali must not let one subsystem treat the same file as ESM while another treats it as CJS

Canonical `exports` condition order:

| API surface / profile | Condition order |
|---|---|
| Deno-oriented standalone (`--api deno`, Phase 1 default) | `deno`, then edge kind (`import` or `require`), then `default` |
| browser-targeted profile (`check --api browser`, `build --bundle --api browser`) | `browser`, then edge kind, then `default` |
| later Node profile | `node`, then edge kind, then `default` |

Phase-1 simplification:
- only the canonical conditions above plus `default` are part of the early stable resolution contract
- if a package's `exports` tree requires additional environment conditions to choose a branch faithfully, Kali should reject that edge with the canonical availability path instead of guessing bundler-specific precedence

Important separation rules:
- runtime/code resolution must not treat `types` as a normal execution condition
- the Deno-oriented standalone surface should honor a package's explicit `deno` condition when present instead of behaving like an unspecified generic bundler
- `--api node` package resolution is part of the same Phase 3 Node-compatibility gate as the rest of the Node API surface; early phases should not resolve packages as though Node mode were already implemented for `check` or `build`
- the browser-targeted profile should honor a package's explicit `browser` mapping/condition consistently in both `check` and `build --bundle`, so analysis and emitted artifacts do not resolve different files by accident
- `package.json#module` is treated only as a legacy bundler-compatibility fallback when `exports` is absent; it must not override an explicit `exports` map, and it should not outrank `main` on a legacy CJS `require` edge
- when a package explicitly marks a path as unavailable for the active profile (for example `browser: false`), Kali must respect that instead of probing alternate files heuristically
- declaration/type lookup follows the separate ladder in [Type Resolution](#type-resolution)

To keep configuration simple, `kali.json#imports` is the canonical aliasing mechanism in early phases. A separate TypeScript-style `paths`/`baseUrl` compatibility layer may be added later if ecosystem pressure justifies it, but it is not part of the MVP contract.

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
kali install --dev vitest                   # Add/install dev dependency
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument semantics are intentionally simple:
- registry package arguments use the canonical registry-package identifier grammar from this chapter (`lodash`, `@types/node`, `jsr:@std/path`)
- registry package arguments mutate `kali.json` (`dependencies` or `devDependencies`) and then refresh lock/materialized state
- `--dev` applies only to registry package arguments; `kali install --dev https://...` is rejected explicitly instead of inventing a raw-URL dev-dependency table
- raw URL arguments update the shared lock/cache state only; they do not invent a second manifest section and should not rewrite source/import-map declarations implicitly
- a raw-URL install is therefore best understood as **pin/materialize this exact URL in the shared dependency state**, not as a request to add a new named dependency kind
- if that URL is not actually referenced from source or `kali.json#imports`, it is only staged materialization and may disappear on the next plain `kali install`
- plain `kali install` reconciles the current manifest + import graph with `kali.lock`, `node_modules/`, and `.kali/cache/urls/`, and may prune raw URL entries that are no longer reachable from that graph

Install-graph discovery rule:
- because `kali install` usually runs without an explicit entrypoint, source-level raw URL imports are discovered from the canonical project-discovery result rather than from one ad hoc command entrypoint
- that install-time scan set is filtered by `kali.json` `include` / `exclude` when present, or by the default project-discovery rules from [SPEC.md](../SPEC.md) when those fields are omitted
- discovery may use a cheap lexical/module-specifier scan of those files plus `kali.json#imports`; it does not require a full check/build just to decide which raw URLs belong in the lock/cache state
- the install-time scan may include declaration-only files too, because they can own type-only imports that still belong to the project's declared dependency graph
- pruning of raw URL lock/cache entries is judged against this install-time declaration graph, not against arbitrary unrelated files elsewhere in the repository

Installation is **fetch-and-link by default**, not "execute package scripts" by default. To preserve sandbox-first behavior:
- npm lifecycle scripts (`preinstall`, `install`, `postinstall`) are not executed unless the user explicitly opts in with `kali install --allow-scripts`
- `--allow-scripts` applies only to that install invocation; it is not an ambient project default
- packages requiring native build steps are rejected as unsupported even when lifecycle scripts are enabled
- package metadata and tarballs can still be analyzed before linking

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
`kali.lock` — deterministic lockfile (project root, committed to version control). Uses a line-oriented TOML-based format for clean diffs and carries its own format version in the file header rather than a JSON `schemaVersion` field.

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
- `[[url]]` entries are for exact raw URL imports after import-map expansion/pinning
- future lockfile revisions may add optional metadata fields, but they should preserve this top-level split instead of collapsing both source kinds into one ambiguous record shape

## Install-Time vs Command-Time Resolution Boundary

Because package resolution can vary by API surface/profile (`deno`, browser-targeted bundle mode, and later `node`), Kali needs one explicit boundary so `install`, lockfiles, and ordinary commands do not drift:

- `kali install` is **profile-agnostic** in Phase 1-3. It locks package versions, fetches/materializes package contents, and records reproducibility data, but it does **not** pre-resolve one permanent `exports`/`browser`/`deno` branch for every future command.
- `check`, `effects`, `build`, `run`, and `test` perform the final **command-time package edge selection** from the already-installed package metadata using the active API surface/profile.
- therefore one `kali.lock` and one materialized package tree can serve both the default Deno-oriented standalone path and the browser-targeted `check` / `build --bundle` path without requiring separate per-profile installs.
- this is possible because early-phase profile differences choose between files that are already present inside the installed package contents; they do not require separate version solves for each supported profile.
- if a later feature truly requires profile-specific solving or materially different dependency graphs, that complexity must be introduced explicitly in a future lockfile/versioning revision rather than being implied accidentally by Phase 1 package wording.

Practical consequence:
- `kali install` does not take `--api` in early phases, and `compilerOptions.apiSurface` does not cause `install` to write a different lockfile for the same manifest/import graph.
- changing `--api` between `deno` and browser-targeted build/check affects which already-installed package entry files are chosen at command time, not whether the project is considered installed.
- if a direct-entry command later points at a file outside the last installed project discovery set and that file reaches additional raw URL imports, the command should fail with `E5004` and tell the user to rerun `kali install` after updating the project's discoverable sources or import map.

## Deterministic Install & Resolution Contract

To keep package behavior predictable across `install`, `check`, `effects`, `build`, `run`, and `test`, Kali uses one simple rule set:
- `kali install` is the command that resolves dependency versions and writes `kali.lock`.
- `kali check`, `effects`, `build`, `run`, and `test` consume the existing lockfile/materialized dependency state; they must not silently re-resolve packages or mutate dependency state as a side effect.
- If the project's declared dependency inputs (`kali.json` registry dependencies, `kali.json#imports`, or source-level raw URL imports from the install-time project discovery set) require materialized state that is missing or stale, non-install commands fail with `E5004` and tell the user to run `kali install`.
- Here, "stale" means the current declared dependency graph, the corresponding `kali.lock` entries, and the required materialized artifacts no longer agree. Non-install commands should not try to infer staleness from arbitrary mtimes or repair it opportunistically.
- `node_modules/` is the materialized tree for registry packages (npm/JSR), while `.kali/cache/urls/` is the materialized cache for raw URL imports; `kali.lock` is the canonical reproducibility record for both.
- When `kali.lock` and the required materialized dependency state disagree, `kali install` is responsible for reconciling them. Other commands should fail clearly rather than guessing which source of truth to trust.
- `--allow-scripts` affects install-time behavior only; it does not change later `check`/`build`/`run` semantics for an already-installed package graph.

This is an intentional simplification: one command mutates dependency state, all other commands consume it deterministically. For raw URL imports, the source/import-map graph is the declaration source of truth and the lock/cache are the materialized state derived from it.

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
        "~/": "./src/",
        "path/": "jsr:@std/path/"
    }
}
```

Interpretation rule:
- `imports` is part of the canonical dependency declaration path for URL-based and alias-based resolution
- raw URL dependencies discovered through source code or expanded import-map entries participate in the same `kali.lock` + `.kali/cache/urls/` discipline as direct URL specifiers
- registry dependencies still belong under `dependencies` / `devDependencies`; `imports` is not a second registry manifest
- import-map targets may still point at canonical registry package identifiers such as `jsr:@std/path/`, which preserves registry identity without inventing a second package namespace

## CommonJS Compatibility

Baseline CommonJS support is part of the Phase 1 package story, but it is intentionally narrow and compile-time-oriented:
- CJS modules (`require`, `module.exports`) are transformed to ESM at compile time
- `require()` calls with static string arguments → ESM import
- Dynamic `require()` is **not** part of the Phase 1-3 linked-artifact model; it is rejected by default, and any later compatibility path must be documented in [specs/19-feature-maturity.md](19-feature-maturity.md) rather than invented ad hoc here
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

- Default registry: `https://registry.npmjs.org`
- Configurable in `kali.json` or `KALI_REGISTRY` env var
- Support for private registries with auth tokens
- Support for JSR (Deno's registry) as an alternative registry source, following the same lock/materialization model as npm packages unless a later phase documents a stronger divergence

## Package Analysis

Before installing, Kali can analyze a package:
```bash
kali package-effects lodash                 # Show effects used by package
kali package-audit lodash                   # Security audit
```

Argument-kind simplification:
- `kali package-effects <pkg>` and `kali package-audit <pkg>` accept only canonical **registry-package identifiers** (`lodash`, `@scope/name`, `jsr:@std/path`)
- raw URLs and local file paths are rejected for these commands instead of creating a parallel analysis path that overlaps confusingly with project/import-graph handling
- raw URL dependencies are analyzed through the ordinary project workflow (`kali install` + `kali effects` / `check` / `build`) because their durable declaration source is the source/import-map graph, not a registry package coordinate

Isolation rule:
- package-analysis commands may fetch package metadata/tarballs into a temporary analysis cache
- they must **not** mutate `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/`
- promoting a package from "analyzed" to "installed dependency" remains the responsibility of `kali install`

`kali package-effects` depends on the Phase 2 effect-report pipeline. Until that lands, the command should be clearly unavailable or marked experimental rather than returning a partial bespoke format.

Canonical output simplification:
- `kali package-effects <pkg>` should reuse the same effect vocabulary and `dynamicReasons` contract as `kali effects`
- the native payload adds only package-specific metadata (see [specs/18-schemas.md](18-schemas.md)) instead of inventing a second unrelated effect schema
- the nested shared effect report includes `analysisContext` so the chosen `apiSurface`, `runtimeProfiles`, and `compatFeatures` travel with the report instead of living only in ambient CLI/config state
- the nested shared effect report still summarizes the full statically reachable package graph selected for analysis under that recorded context; it is not just a manifest-level metadata report
- `--output json` wraps that payload in the standard CLI command envelope; it does not create a third package-effects-only outer format

`kali package-audit` is a later tooling feature rather than a core compiler/runtime milestone. If unimplemented, Kali should say so explicitly instead of implying a partial audit guarantee.

This integrates with the effect system — know what a dependency does before you use it.
