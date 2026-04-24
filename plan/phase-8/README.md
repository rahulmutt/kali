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

### 8.3 Browser package deployability

- Expand browser-targeted package checks and `build --bundle` smoke tests.
- Keep deployable-through-host claims distinct from standalone browser executable claims.

### 8.4 Registry-analysis evolution

- Extend `package-effects` / `package-audit` only through explicit command/schema revisions.
- Do not add batch, raw-URL, local-path, or project-graph behavior accidentally.

## Exit gate

- Package support statements name exact rung and context.
- Corpus results are deterministic and reproducible.
- Missing/stale dependency state remains `E6004`; non-install commands do not auto-repair.
