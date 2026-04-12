# Stage 1.10 — Package Management

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.4 — Name Resolution](04-name-resolution.md) (for bare-specifier module resolution), [1.8 — Runtime & Execution](08-runtime-execution.md) (to exercise installed packages end-to-end)  
**Ordering note:** `SPEC.md` recommends the package/install foundation (spec step 2) before the execution foundation (spec step 3). This plan reverses them — see the [ordering note in PLAN.md](../../PLAN.md#ordering-note-package-management-after-execution) for the rationale.

## Goal

Implement `kali_npm` and the `kali install` command — deterministic resolution, lock file,
and materialisation of npm/JSR/raw-URL dependencies inside the **pure JS/TS package contract**.
Non-install commands (`check`, `build`, `run`, `test`) become aware of the installed package
graph without performing any mutations.

## Workable Milestone

- `kali install` resolves the project's declared dependencies, writes a deterministic lock file,
  and materialises packages into the local cache.
- `kali install <pkg>` adds a new dependency to `kali.json` and the lock file.
- The compiler pipeline (from Stage 1.4 onward) resolves bare module specifiers against the
  materialised package graph.
- Packages outside the **pure JS/TS package contract** (native addons, binary bootstraps) are
  rejected with a clear diagnostic.

## Progress

- The `kali install` command is now wired up in the CLI and can reconcile a manifest/lock pair
  through the new `kali_npm` implementation.
- Registry resolution now supports npm packages and JSR compatibility names, writes deterministic
  `kali.lock` output, materialises packages under `.kali-cache/` plus `node_modules/`, and now
  selects the highest matching published version for semver ranges.
- Bare import resolution now consults the materialized package graph, so the Stage 1.4 resolver can
  follow installed packages instead of only local relative files.
- Package-shape validation now rejects obvious native-addon and lifecycle-script cases.
- Manifest reconciliation now fails fast when two registry identities would collapse onto the
  same `node_modules/` path before any materialization work begins, including transitive
  install-path conflicts during graph reconciliation.
- `kali install` now prunes stale registry-package entries from the lock graph and rebuilds the
  package cache / `node_modules` layout when the lock graph already exists.
- Raw URL reconciliation now follows project-discovery/import-map declarations and prunes stale
  URL cache entries when the declaration graph changes.
- Non-install commands still fail fast with `E6007` when an installed dependency graph is missing
  or stale.
- Remaining stage work is mostly around install repair edge cases, the broader package-shape /
  host-fit diagnostics matrix, and the CLI integration coverage called out in the tasks below.

## Tasks

### 1. `kali.json` manifest

Define the schema-v1 `kali.json` project manifest (owned by `specs/18-schemas.md`):

```json
{
  "$schema": "https://kali-lang.org/schemas/manifest/v1",
  "name": "my-project",
  "version": "0.1.0",
  "dependencies": {
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "@types/lodash": "^4.14.0"
  },
  "compilerOptions": {
    "apiSurface": "deno",
    "buildMode": "fast",
    "strict": true
  }
}
```

Key fields:

- `dependencies` / `devDependencies`: npm-style version ranges for registry packages or raw URLs.
- `compilerOptions`: inherits the CLI flag vocabulary (`apiSurface`, `buildMode`, `strict`, etc.)
  so project-level config and CLI flags use the same canonical keys.

### 2. Lock file (`kali.lock`)

Schema-v1 lock file format (JSON):

```json
{
  "$schema": "https://kali-lang.org/schemas/lock/v1",
  "version": 1,
  "packages": {
    "lodash@4.17.21": {
      "registry": "npm",
      "integrity": "sha512-...",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "dependencies": {}
    }
  },
  "rawUrls": {
    "https://deno.land/x/std@0.200.0/http/server.ts": {
      "integrity": "sha256-...",
      "cached": ".kali-cache/raw/..."
    }
  }
}
```

The lock file pins exact versions and content hashes. It is the single source of truth for the
installed package graph. Non-install commands must read the lock file; they must never mutate it.

### 3. Registry resolution (`kali_npm`)

Implement package resolution against the npm and JSR registries:

- **Version range resolution**: given a semver range, select the highest matching version
  available in the registry (following npm's resolution algorithm). Cache registry metadata
  locally to avoid repeated network round-trips.
- **Integrity verification**: after downloading a tarball, verify the `sha512` integrity field
  matches the lock file entry. Fail with `E6003` on mismatch.
- **Transitive dependency closure**: resolve transitive dependencies and flatten them into the
  lock file. Detect version conflicts and report `E6002`.
- **JSR support**: treat `jsr:<scope>/<pkg>` specifiers using the JSR registry API.
- **Raw URL support**: download and cache raw-URL imports (`https://...`); record them in the
  `rawUrls` section of the lock file with their content hash.

`E6xxx` package error codes:

| Code | Meaning |
|---|---|
| `E6001` | Package not found in registry |
| `E6002` | Dependency version conflict |
| `E6003` | Integrity verification failed |
| `E6004` | Package falls outside the pure JS/TS package contract |
| `E6005` | Package requires Node-only host APIs not available in current context |
| `E6006` | Lifecycle script execution rejected (use `--allow-scripts` to enable) |
| `E6007` | `kali install` required before this command can proceed |
| `E6008` | Invalid package specifier |
| `E6009` | Raw-URL import not allowed in current registry context |

### 4. Package-shape validation

Before materialising any package, apply the **package-support decision order** from
`specs/14-packages.md`:

1. **Package shape**: inspect the package's `package.json`. Reject (with `E6004`) if:
   - `install` script invokes `node-gyp` or similar native-build tool.
   - `main` / `exports` resolves to a `.node` native addon.
   - A `bin` entry depends on a pre-built binary download.
2. **Host/API fit**: if the package declares Node-only built-in dependencies (e.g. direct use of
   `fs`, `path`, `os`, `child_process` beyond what Kali's Default standalone context provides),
   emit `E6005` with a note about the Phase-3 Node compatibility target.
3. **Command maturity**: if the requested command/context doesn't yet support this package rung,
   report accordingly.

### 5. `kali install` subcommand

```
kali install                          # reconcile the project dependency graph
kali install <pkg>                    # add a new dependency
kali install --dev <pkg>             # add a devDependency
kali install --allow-scripts <pkg>   # opt-in npm lifecycle hook execution
```

Behaviour:

- `kali install` (no target): read `kali.json`, resolve the full dependency graph (using the lock
  file for pinned versions if it exists), materialise packages into `.kali-cache/packages/`,
  write / update `kali.lock`.
- `kali install <pkg>`: add the package to `kali.json`'s `dependencies`, then reconcile.
- `kali install --dev <pkg>`: add to `devDependencies`.
- `kali install --allow-scripts <pkg>`: execute npm lifecycle hooks (`preinstall`, `install`,
  `postinstall`) only when there is non-empty **effective npm-scriptable install work** for this
  package. Do **not** execute scripts for packages that have no lifecycle hooks or for which the
  scripts are empty/no-ops.
- Raw-URL imports discovered through project discovery are also reconciled by `kali install`.

**Strict non-mutating rule:** all commands other than `kali install` must not mutate
`kali.json`, `kali.lock`, or the cache. If the lock file is stale or the cache is incomplete,
emit `E6007` (install required) and exit non-zero rather than silently repairing state.

### 6. Module specifier resolution in the compiler

Update the name-resolution stage (Stage 1.4) to resolve bare specifiers against the materialised
package graph:

1. Look up the specifier in the `package.json` `exports` map of the matching installed package.
2. Find the corresponding source file in the materialised cache.
3. If the lock file does not contain the package, emit `E6007` instead of `E3010`.

TypeScript type declarations:

- For packages that include `.d.ts` files in their exports, load those for type checking.
- For packages without bundled types, look for a corresponding `@types/<pkg>` package in
  `devDependencies`.

### 7. Cache layout

Materialised packages live under `.kali-cache/` (gitignore'd):

```
.kali-cache/
├── packages/
│   └── lodash@4.17.21/          # extracted tarball contents
│       ├── package.json
│       └── lodash.js
└── raw/
    └── sha256-abcdef.../        # cached raw-URL content
        └── server.ts
```

### 8. Tests

- `kali install` on a fixture project with `lodash` as a dependency → lock file written,
  package materialised, integrity verified.
- `kali install` is idempotent: running twice produces the same lock file byte-for-byte.
- `kali install <native-pkg>` → exits 1 with `E6004`.
- `kali check` without prior install → exits 1 with `E6007`.
- `kali install --allow-scripts <pkg>` with no lifecycle scripts → clean/no-op, exits 0.
- `kali install --allow-scripts <raw-url>` → exits 1 with `E6009` (not valid for raw-URL targets).

## Out of Scope

- `kali package-effects` (Phase 2 target).
- `kali package-audit` (Later compatibility).
- Automatic dependency repair outside `kali install` (explicit non-goal).
- Broad `--api node` package support (Phase 3 target).

## Definition of Done

- [ ] `kali install` resolves, locks, and materialises npm/JSR/raw-URL deps deterministically.
- [ ] Bare specifiers resolve in `kali check` / `kali build` / `kali run` after install.
- [ ] Native-addon packages rejected with `E6004`.
- [ ] Non-install commands emit `E6007` if lock file is stale.
- [ ] All `E6xxx` error cases covered by tests.
- [ ] `cargo test -p kali_npm` and integration tests pass.
- [ ] No Stage 1.1–1.9 regressions.
