# Phase 18 — Ecosystem Compatibility by Rung

## Goal

Broaden package compatibility using support-rung evidence instead of broad npm, Node, or browser claims.

## Owning specs

- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/14-packages.md`
- `specs/15-errors.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 18.1 Package-corpus stewardship

- Group evidence by package shape, source class, API surface, command, and support rung.
- Keep expected failures for native, binary, bootstrap-heavy, host-mismatched, and published-bin-entrypoint cases.
- Keep corpus snapshots deterministic and concise.

### 18.2 Node ecosystem breadth

- Add Node package support only when required built-ins and process semantics are explicitly supported.
- Keep late Node modules and process/thread/network APIs gated until Phase 17 contracts exist.
- Separate package-content support from published CLI/bin entrypoint support.

### 18.3 Browser package deployability

- Expand browser-targeted `check` / `build --bundle` and browser-harness package evidence by package shape.
- Keep deployable-through-host, executable-through-browser-harness, and standalone browser runtime claims separate.
- Reject packages whose browser path depends on unavailable host/native/binary behavior.

### 18.4 Registry-analysis boundaries

- Keep `package-effects` and `package-audit` on the schema-v1 single registry identifier contract.
- Do not add batch, raw-URL, local-path, or project-discovery behavior without spec/schema revisions.
- Current progress: registry-analysis command-shape coverage now also pins extra-argument failures in JSON mode, including `--output json` and `--pretty --output json`, so presentation flags do not bypass the single-package contract.

## Exit gate

- Every package support statement names rung and context.
- Missing/stale dependency materialization still fails with `E6004` outside `kali install`.
- Package-corpus evidence is deterministic and does not imply broader availability than the maturity matrix.
