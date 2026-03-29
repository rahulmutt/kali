# 14 — Package Management

## npm Compatibility

### Supported Packages
Kali supports npm packages that:
- Are pure JavaScript/TypeScript (no native code)
- Do **not** use `node-gyp` or native addons
- Use standard module systems (ESM or CJS)

This covers the vast majority of the npm ecosystem (utility libraries, data processing, frameworks, etc.).

### Package Resolution
Follow Node.js module resolution algorithm, adapted for Kali:
1. Check `kali_modules/<package>/package.json` for `exports`, `main`, `module` fields
2. Support `exports` map conditions: `import`, `require`, `default`, `types`
3. Resolve relative imports with extension probing (`.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`)
4. Support `paths` and `baseUrl` from `kali.json`
5. Also check `node_modules/` for backwards compatibility with existing projects

### Installation
```bash
kali install lodash                         # Install single package from npm
kali install                                # Install all dependencies from kali.json
kali install --dev vitest                   # Dev dependency
kali install https://deno.land/std/path/mod.ts  # URL import (cached locally)
```

Uses a `kali_modules/` directory (not `node_modules/`) with flat structure:
```
kali_modules/
├── lodash@4.17.21/
│   ├── package.json
│   └── ...
├── zod@3.22.0/
│   └── ...
└── .cache/              — Cached URL imports
```

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
URL imports are cached in `kali_modules/.cache/`. Integrity is verified against the lock file.

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
- Dynamic `require()` → runtime resolution (flagged in effect analysis)
- `__dirname`, `__filename` → transformed to `import.meta.dirname`, `import.meta.filename`

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
