# 14 — Package Management

## npm Compatibility

Package loading is compile-time first: Kali resolves, analyzes, and links dependency graphs during build/check/run. For normal builds, application code and its static dependencies are emitted as one linked artifact rather than a fleet of runtime-linked WASM modules.

### Supported Packages
Kali supports npm packages that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

Phase simplification:
- **Phase 1 MVP**: packages that do not depend on unsupported Node core modules and fit the linked-artifact model.
- **Phase 3+**: broader compatibility for packages that expect the `node` API surface and additional Node built-ins.

This keeps the early ecosystem promise realistic: utility libraries, validators, parsers, and many framework packages are in scope early, while Node-host-heavy packages follow the Node compatibility work.

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

Important separation rules:
- runtime/code resolution must not treat `types` as a normal execution condition
- the Deno-oriented standalone surface should honor a package's explicit `deno` condition when present instead of behaving like an unspecified generic bundler
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

For npm packages, Kali should prefer the strongest sound information available without inventing fresh `any` merely to suppress analysis:
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
- Support for JSR (Deno's registry) as an alternative source

## Package Analysis

Before installing, Kali can analyze a package:
```bash
kali package-effects lodash                 # Show effects used by package
kali package-audit lodash                   # Security audit
```

`kali package-effects` depends on the Phase 2 effect-report pipeline. Until that lands, the command should be clearly unavailable or marked experimental rather than returning a partial bespoke format.

`kali package-audit` is a later tooling feature rather than a core compiler/runtime milestone. If unimplemented, Kali should say so explicitly instead of implying a partial audit guarantee.

This integrates with the effect system — know what a dependency does before you use it.
