# Phase 13 — Ecosystem Compatibility Expansion

## Goal

Broaden package compatibility with support-rung evidence instead of broad npm or Node claims.

## Owning specs

- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/14-packages.md`
- `specs/15-errors.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 13.1 Package-corpus stewardship

- Keep package evidence grouped by package shape, source class, API surface, command, and support rung.
- Record expected failures for native, binary, bootstrap-heavy, host-mismatched, and published-bin entrypoint cases.
- Keep corpus snapshots concise and deterministic.
- Progress note: the package corpus now also keeps the `spawn-tools` Deno-host package shape exercised on the standalone Deno surface, the browser surface's check/bundle lanes, the browser-harness run/test lanes, and the inherited browser API-surface run/test lanes, preserving the support-rung boundary across contexts while the standalone browser runtime gate remains separate.

### 13.2 Node ecosystem breadth

- Add Node package support only when required built-ins and process semantics are explicitly supported.
- Keep late Node modules (`node:net`, `node:dns`, worker/thread modules, unresolved promise/timer subpaths, and process-control APIs) gated until Phase 12 contracts exist.
- Separate package-content support from published CLI/bin entrypoint support.
- Progress note: the Node package corpus now also exercises a `process.cwd` / `process.pid` / `process.chdir` / `process.exit` package slice on the documented Node surface, keeping package-content evidence aligned with the supported process semantics.

### 13.3 Browser package deployability

- Expand browser-targeted `check` / `build --bundle` and browser-harness package evidence by package shape.
- Keep deployable-through-host, executable-through-browser-harness, and standalone runtime claims separate.
- Reject packages whose browser path depends on unavailable host/native/binary behavior.

### 13.4 Registry-analysis boundaries

- Keep `package-effects` and `package-audit` on the schema-v1 single registry identifier contract.
- Do not add batch, raw-URL, local-path, or project-discovery behavior without spec/schema revisions.

## Exit gate

- Every package support statement names rung and context.
- Missing/stale dependency materialization still fails with `E6004` outside `kali install`.
- Package-corpus evidence is deterministic and does not imply broader availability than the maturity matrix.
