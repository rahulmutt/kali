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

- Stage 1.10 is complete: the `kali install` command is wired up in the CLI and can reconcile a
  manifest/lock pair through the `kali_npm` implementation.
- Registry resolution supports npm packages and JSR compatibility names, writes deterministic
  `kali.lock` output, materialises packages under `.kali-cache/` plus `node_modules/`, and selects
  the highest matching published version for semver ranges.
- Bare import resolution consults the materialized package graph and uses the project root when
  the checked source file lives in a nested directory, so the Stage 1.4 resolver can follow
  installed packages instead of only local relative files.
- Type resolution recognizes bundled declaration entries and a matching `@types/<pkg>`
  devDependency, allowing package imports without bundled types to resolve through declaration
  packages when the project has installed them.
- Package-shape validation rejects obvious native-addon, `exports`-backed native entrypoints, node-gyp, and prebuild-install-style lifecycle-script cases.
- Manifest reconciliation fails fast when two registry identities would collapse onto the
  same `node_modules/` path before any materialization work begins, including transitive
  install-path conflicts during graph reconciliation.
- `kali install` prunes stale registry-package entries from the lock graph and rebuilds the
  package cache / `node_modules` layout when the lock graph already exists.
- Raw URL reconciliation follows project-discovery/import-map declarations and prunes stale
  URL cache entries when the declaration graph changes.
- `kali install --allow-scripts` treats packages without install-time lifecycle hooks as a
  no-op success, while the invalid raw-URL / JSR lifecycle-hook combinations are still rejected
  before any fetch.
- NPM lifecycle hooks (`preinstall`, `install`, `postinstall`) execute during install when the
  opt-in flag is present, and blank hooks are treated as deterministic no-ops.
- `kali install --dev` requires an explicit registry target and rejects raw-URL targets before
  materialization work begins.
- Non-install commands fail fast with `E6007` when an installed dependency graph is missing or
  stale.
- Package-shape coverage has explicit unit tests for node-gyp lifecycle scripts and native-addon
  entrypoints; host-fit coverage rejects Node-only builtins surfaced through direct
  imports/requires; and CLI smoke coverage exercises pruning stale registry layouts back to an
  empty install state.
- Registry metadata lookups use a process-local cache so repeated resolution within a single
  install run avoids redundant network round-trips.
- Install-time script detection now keys only off `preinstall` / `install` / `postinstall`, so
  packages whose `scripts` section only contains ordinary metadata such as `test`, `lint`,
  `postlint`, or `posttest` no longer trip the `--allow-scripts` guardrail.
- Added a semver-shaped regression fixture so the common pure-JS `semver` package metadata shape
  stays evidence-backed and does not regress back into the old `--allow-scripts` false positive.
- Added a package-install regression that proves plain `kali install semver` succeeds without
  `--allow-scripts` when only non-install lifecycle scripts are present.
- Added an explicit `kali install --allow-scripts semver` regression so the no-op allow-scripts
  path stays covered for packages whose metadata only carries non-install lifecycle hooks.
- Added a no-manifest `kali install` regression so an empty workspace still exits cleanly without
  synthesizing `kali.json` or `kali.lock`, matching the planned no-op install path.

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

## Follow-up work uncovered by the semver probe

A real-world `semver` install attempt exposed a Phase-1 package-management gap that should be
tracked explicitly even though the historical stage milestone is otherwise complete.

### Semver-specific regression to close

Observed behavior during `kali install semver`:
- Kali rejected the install with `E6006` (`npm lifecycle scripts require --allow-scripts`).
- The published `semver` package does **not** use install-time lifecycle hooks for normal
  installation; it declares non-install scripts such as `test`, `lint`, `postlint`, and
  `posttest`.

### Systematic fix plan

1. Narrow install-hook detection so `--allow-scripts` is required only for actual install-time npm
   lifecycle hooks that participate in the current invocation's **effective npm-scriptable install
   work**.
2. Treat non-install scripts (`test`, `lint`, `postlint`, `posttest`, etc.) as ordinary package
   metadata, not as install blockers.
3. Keep rejecting truly unsupported install/bootstrap cases (`preinstall`, `install`,
   `postinstall`, native builds, binary bootstraps) on the existing `E6004`/`E6006` paths.
4. Add a regression fixture using the real `semver` metadata shape so Phase-1 install support is
   evidence-backed for this common pure-JS package class.
5. Extend install tests to distinguish:
   - package has no install-time hooks → plain `kali install <pkg>` succeeds
   - package has install-time hooks → plain install rejects, `--allow-scripts` is required
   - package has non-install scripts only → plain install still succeeds

Current outcome:
- the semver-style install regression now passes in the package-management test suite, confirming
  that non-install scripts are treated as ordinary metadata instead of an install blocker

## Out of Scope

- `kali package-effects` (Phase 2 target).
- `kali package-audit` (Later compatibility).
- Automatic dependency repair outside `kali install` (explicit non-goal).
- Broad `--api node` package support (Phase 3 target).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
