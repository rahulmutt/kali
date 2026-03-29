# 14 — Package Management

## Registry Compatibility

Package loading is compile-time first: Kali resolves, analyzes, and links dependency graphs during build/check/run. For normal builds, application code and its static dependencies are emitted as one linked artifact rather than a fleet of runtime-linked WASM modules.

### Supported Packages
Kali supports registry packages (npm/JSR) that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

Phase simplification:
- **Phase 1 MVP**: packages that do not depend on unsupported Node core modules and fit the linked-artifact model.
- **Phase 3+**: broader compatibility for packages that expect the `node` API surface and additional Node built-ins.

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
- **Registry packages** — npm and JSR packages resolved by package name/version and materialized into `node_modules/`
- **Raw URL imports** — exact `https://...` dependencies cached under `.kali/cache/urls/`

Lockfile rule:
- `kali.lock` is the canonical reproducibility record for **both** source kinds
- registry packages and raw URL imports may use different on-disk materialization locations, but they share one lock discipline
- non-install commands must check the required materialized state for the dependency kinds actually used by the project instead of assuming `node_modules/` alone is always the full dependency state

This removes an ambiguity from the earlier wording: a URL-only project may have no `node_modules/` tree at all and still be fully installed.

### Package Resolution
Follow Node.js-style package resolution, but keep the early-phase rules explicit so browser, Deno, and package behavior do not drift.

Canonical early-phase code-resolution ladder:
1. Apply import-map rewrites from `kali.json#imports` before package resolution.
2. Resolve package self-references (`"name": "pkg"` imported as `pkg/...`) using the package's own `exports` map before walking upward into `node_modules`.
3. If the specifier names a package or package subpath, consult `package.json#exports` first.
4. Evaluate `exports` against the current API surface and edge kind:
   - distinguish **ESM import edges** from **CJS require edges**
   - resolve the exact requested subpath; do not flatten subpath exports into one package-wide entry
   - use the canonical condition order table below
   - unsupported or unmatched conditional branches are skipped; Kali should not guess a fallback branch that the package did not publish
5. If `exports` does not resolve the entry, fall back to legacy entry fields using the same API-surface intent **and still respecting edge kind**:
   - browser-targeted profile (`kali check --api browser` and `kali build --bundle --api browser`): apply `browser` replacement map semantics first where applicable; then for **ESM import edges** prefer `module`, then `main`, and for **CJS require edges** prefer `main`, then `module`
   - Deno-oriented standalone profile (`--api deno`, Phase 1 default): for **ESM import edges** prefer `module`, then `main`, and for **CJS require edges** prefer `main`, then `module`
   - later Node profile may add `node`-specific behavior before the generic fallback ladder when explicitly documented
6. Resolve relative/file entries with extension probing (`.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`).
7. Classify the resolved file as ESM or CJS using Node-compatible signals (`.mjs` / `.cjs`, nearest `package.json#type`, and syntax where necessary).

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

### Installation
```bash
kali install lodash                         # Install single package from npm
kali install                                # Install all dependencies from kali.json
kali install --dev vitest                   # Dev dependency
kali install https://deno.land/std/path/mod.ts  # URL import (cached locally)
```

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
`kali.lock` — deterministic lockfile (project root, committed to version control). Uses a line-oriented TOML-based format for clean diffs and carries its own format version in the file header rather than a JSON `schemaVersion` field:
```toml
# kali.lock v1 — do not edit manually

[[package]]
name = "lodash"
version = "4.17.21"
resolved = "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"
integrity = "sha256-..."

[[package]]
name = "zod"
version = "3.22.0"
resolved = "https://registry.npmjs.org/zod/-/zod-3.22.0.tgz"
integrity = "sha256-..."
dependencies = []
```

## Deterministic Install & Resolution Contract

To keep package behavior predictable across `install`, `check`, `build`, `run`, and `test`, Kali uses one simple rule set:
- `kali install` is the command that resolves dependency versions and writes `kali.lock`.
- `kali check`, `build`, `run`, and `test` consume the existing lockfile/materialized dependency state; they must not silently re-resolve packages or mutate dependency state as a side effect.
- If `kali.json` declares dependencies but the required materialized state for the active dependency source kinds is missing or stale, non-install commands fail with `E5004` and tell the user to run `kali install`.
- `node_modules/` is the materialized tree for registry packages (npm/JSR), while `.kali/cache/urls/` is the materialized cache for raw URL imports; `kali.lock` is the canonical reproducibility record for both.
- When `kali.lock` and the required materialized dependency state disagree, `kali install` is responsible for reconciling them. Other commands should fail clearly rather than guessing which source of truth to trust.
- `--allow-scripts` affects install-time behavior only; it does not change later `check`/`build`/`run` semantics for an already-installed package graph.

This is an intentional simplification: one command mutates dependency state, all other commands consume it deterministically.

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
- non-install commands may fetch only when the URL dependency is already pinned/authorized by the existing project state and a recoverable local cache miss occurs; they must not silently change the pinned dependency set or rewrite the lockfile
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

## CommonJS Compatibility

Baseline CommonJS support is part of the Phase 1 package story, but it is intentionally narrow and compile-time-oriented:
- CJS modules (`require`, `module.exports`) are transformed to ESM at compile time
- `require()` calls with static string arguments → ESM import
- Dynamic `require()` is **not** part of the Phase 1-3 linked-artifact model; it is rejected by default, and any later compatibility path must be documented in [specs/19-feature-maturity.md](19-feature-maturity.md) rather than invented ad hoc here
- `__dirname`, `__filename` → transformed to `import.meta.dirname`, `import.meta.filename`

## Dynamic Imports

To keep the module system aligned with the single-artifact architecture:
- static `import` is the primary and fully supported path
- literal-string `import("pkg")` is a later optimization/compatibility feature that may be rewritten against the already-linked graph
- non-literal `import(expr)` is treated as a dynamic effect boundary and rejected by default in early phases; any later compatibility path must be documented in [specs/19-feature-maturity.md](19-feature-maturity.md)

## Type Resolution

For registry packages, Kali should prefer the strongest sound information available without inventing fresh `any` merely to suppress analysis:
1. Check the package's own `types` / `typings` field in `package.json`
2. Apply `typesVersions` if present and relevant to the active resolution mode
3. Check for bundled `.d.ts` files alongside `.js` files
4. Check for `@types/<package>` in dependencies as a fallback when the package does not ship authoritative declarations
5. If package source is available as JS/TS, run the normal Kali checker/inference pipeline on that package and synthesize module-boundary types from the result
6. If Kali still cannot justify a precise exported type, fall back to `unknown` at the package boundary with a warning

Interpretation rules:
- bundled package types win over `@types` because they are the package author's authoritative declarations
- `typesVersions` refines how a package's own declarations are selected; it does not outrank the package's own top-level declaration ownership
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

`kali package-effects` depends on the Phase 2 effect-report pipeline. Until that lands, the command should be clearly unavailable or marked experimental rather than returning a partial bespoke format.

`kali package-audit` is a later tooling feature rather than a core compiler/runtime milestone. If unimplemented, Kali should say so explicitly instead of implying a partial audit guarantee.

This integrates with the effect system — know what a dependency does before you use it.
