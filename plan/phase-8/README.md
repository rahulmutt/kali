# Phase 8 — Ecosystem Breadth and Package Compatibility

## Goal

Widen package compatibility with evidence by support rung instead of broad npm claims.

## Owning specs

- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/14-packages.md`
- `specs/15-errors.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 8.1 Package-corpus matrix

- Track packages by source kind, package shape, host/API fit, command, and support rung.
- Include Deno-oriented, browser-targeted, and Node contexts separately.
- Record expected failures for native/binary/bootstrap-heavy packages.

### 8.2 Node ecosystem breadth

- Expand Node built-ins and package-resolution behavior where package-corpus evidence demands it.
- Keep late Node modules and process-control APIs gated until Phase 7 contracts exist.
- Progress: the Node package corpus now also exercises the documented Node build surface for the same `node:`-based package set that already has Node `check` / `run` coverage, so analysis, build, and execution evidence stays aligned for the current Node compatibility slice.
- Progress: the Deno package corpus now also exercises the documented Deno build surface for the same Deno-host package set that already has Deno `check` / `run` coverage, so analysis, build, and execution evidence stays aligned for the current Deno compatibility slice.
- Progress: the Deno package corpus now also exercises a canonical `jsr:@std/path` package fixture materialized at `node_modules/@std/path` on the Deno surface, keeping the `jsr:` registry prefix and on-disk path mapping honest in the package-resolution evidence.

### 8.3 Browser package deployability

- Expand browser-targeted package checks and `build --bundle` smoke tests.
- Keep deployable-through-host claims distinct from standalone browser executable claims.
- Progress: browser-targeted `.js` package entrypoint coverage now includes the minimized mixed CommonJS/ESM interop slice plus the `vue/runtime-dom` browser branch and browser replacement-map JS-entrypoint coverage, so the browser corpus keeps the first-class-JavaScript path honest across both replacement-map and interop cases.
- Progress: the browser package corpus now also exercises the canonical pure-JS `semver` probe on `.js` input, keeping the support-rung evidence aligned with the browser-targeted first-class-JavaScript path instead of only the TypeScript lane.

### 8.4 Registry-analysis evolution

- Extend `package-effects` / `package-audit` only through explicit command/schema revisions.
- Do not add batch, raw-URL, local-path, or project-graph behavior accidentally.

## Exit gate

- Package support statements name exact rung and context.
- Corpus results are deterministic and reproducible.
- Missing/stale dependency state remains `E6004`; non-install commands do not auto-repair.
