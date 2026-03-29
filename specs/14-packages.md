# 14 — Package Management

## npm Compatibility

Package loading is compile-time first: Kali resolves, analyzes, and links dependency graphs during build/check/run. For normal builds, application code and its static dependencies are emitted as one linked artifact rather than a fleet of runtime-linked WASM modules.

### Supported Packages
Kali supports npm packages that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

This covers the vast majority of the npm ecosystem (utility libraries, data processing, frameworks, etc.).

### Package Resolution
Follow Node.js module resolution algorithm, adapted for Kali:
1. Apply import-map rewrites from `kali.json#imports` before package resolution
2. Check `node_modules/<package>/package.json` for `exports`, `main`, `module`, and `types` fields
3. Support `exports` map conditions: `import`, `require`, `default`, `types`, and `browser` where relevant
4. Resolve relative imports with extension probing (`.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`)

To keep configuration simple, `kali.json#imports` is the canonical aliasing mechanism in early phases. A separate TypeScript-style `paths`/`baseUrl` compatibility layer may be added later if ecosystem pressure justifies it, but it is not part of the MVP contract.

### Installation
```bash
kali install lodash                         # Install single package from npm
kali install                                # Install all dependencies from kali.json
kali install --dev vitest                   # Dev dependency
kali install https://deno.land/std/path/mod.ts  # URL import (cached locally)
```

Installation is **fetch-and-link by default**, not "execute package scripts" by default. To preserve sandbox-first behavior:
- npm lifecycle scripts (`preinstall`, `install`, `postinstall`) are not executed unless the user explicitly opts in
- packages requiring native build steps are rejected as unsupported
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
`kali.lock` — deterministic lockfile (project root, committed to version control). Uses a line-oriented format for clean diffs:
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
    "imports": {
        "std/": "https://deno.land/std@0.220.0/",
        "~/": "./src/"
    }
}
```

## CommonJS Compatibility

- CJS modules (`require`, `module.exports`) are transformed to ESM at compile time
- `require()` calls with static string arguments → ESM import
- Dynamic `require()` is **not** part of the Phase 1-3 linked-artifact model; it is rejected by default or requires an explicit later-phase compatibility mode, and is flagged in effect analysis
- `__dirname`, `__filename` → transformed to `import.meta.dirname`, `import.meta.filename`

## Dynamic Imports

To keep the module system aligned with the single-artifact architecture:
- static `import` is the primary and fully supported path
- literal-string `import("pkg")` is a later optimization/compatibility feature that may be rewritten against the already-linked graph
- non-literal `import(expr)` is treated as a dynamic effect boundary and rejected by default in early phases unless an explicit compatibility mode is enabled

## Type Resolution

For npm packages:
1. Check for `@types/<package>` in dependencies
2. Check package's `types` / `typings` field in `package.json`
3. Check for `.d.ts` files alongside `.js` files
4. Fall back to `any` type with a warning

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

This integrates with the effect system — know what a dependency does before you use it.
