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
- Progress: the deterministic package-corpus matrix now lives in [`package-corpus-matrix.md`](./package-corpus-matrix.md) and groups the current corpus evidence by browser-targeted, default-standalone, Node, and Deno slices so the rung/context split stays explicit.

### 8.2 Node ecosystem breadth

- Expand Node built-ins and package-resolution behavior where package-corpus evidence demands it.
- Keep late Node modules and process-control APIs gated until Phase 7 contracts exist.
- Progress: the Node package corpus now also exercises the documented Node build surface for the same `node:`-based package set that already has Node `check` / `run` coverage, so analysis, build, and execution evidence stays aligned for the current Node compatibility slice.
- Progress: the Node package corpus now also exercises the canonical pure-JS `semver` probe on `.js` input across the documented Node `check` / `build` / `run` / `test` lanes, and the package-corpus matrix now records that `.js`-input `semver` slice explicitly, keeping the Node-first-class-JavaScript evidence aligned with the browser and standalone corpus lanes.
- Progress: the Node package corpus now also exercises a `node:buffer` built-in import within the Node exports-map JS-input slice across the documented Node `check` / `build` / `run` / `test` lanes, keeping the Node package-resolution evidence aligned with first-class JavaScript compilation instead of only the TypeScript lane.
- Progress: the Node package corpus now also mirrors the runner-package exports-map and mixed-format-entry slices onto `.js` input for the explicit Node surface, keeping the Node package-resolution evidence aligned with first-class JavaScript compilation instead of only the TypeScript lane.
- Progress: the canonical host-heavier `@mariozechner/pi-coding-agent` probe now also covers the package content on the default standalone surface at the check/build rung, and the same package-content slice now also mirrors `.js` input; it now also has `.js` test-corpus coverage and now also executes on the default standalone surface in `.js` input, so the corpus distinguishes package-content support from the separate published bin entrypoint probe while keeping first-class JavaScript compilation in view. The default standalone `date-fns` utility slice now also mirrors `.js` input across `check` / `build` / `run` / `test`, the canonical semver probe now also mirrors `.js` input across `check` / `build` / `run` / `test`, and the package-corpus matrix now records the split default-standalone semver TS and JS rows explicitly, while the broader default standalone utility corpus now also mirrors the zod, p-limit, and ms slices onto `.js` input for `check` / `build` / `run` / `test`, and the pattern-exports utility slice now mirrors `.js` input across `check` / `build` / `run`, keeping the pure-JS utility ladder aligned with first-class JavaScript compilation too. The default standalone web-baseline primitive package slice now also has `.js` test coverage, keeping the baseline web-API rung aligned with the same first-class-JavaScript package evidence too.
- Progress: the default standalone exports-map mixed-format interop probe now also mirrors `.js` input on `check` / `build` / `run`, keeping the exports-map resolver evidence aligned with first-class JavaScript compilation instead of only the TypeScript lane.
- Progress: the default standalone module-entry corpus now also mirrors `.js` input for both the simple module-entry and module-entry-chain slices on `check` / `run`, keeping the first-class-JavaScript package evidence aligned with the same module-resolution paths as the TypeScript lane.
- Progress: the Deno package corpus now also exercises the documented Deno build surface for the same Deno-host package set that already has Deno `check` / `run` coverage, so analysis, build, and execution evidence stays aligned for the current Deno compatibility slice.
- Progress: the Deno package corpus now also exercises a canonical `jsr:@std/path` package fixture materialized at `node_modules/@std/path` on the Deno surface, keeping the `jsr:` registry prefix and on-disk path mapping honest in the package-resolution evidence. The same Deno host-control and JSR fixtures now also mirror `.js` input on `check` / `build` / `run`, keeping the Deno package-corpus slice aligned with first-class JavaScript compilation instead of only the TypeScript lane.

### 8.3 Browser package deployability

- Expand browser-targeted package checks and `build --bundle` smoke tests.
- Keep deployable-through-host claims distinct from standalone browser executable claims.
- Progress: browser-targeted `.js` package entrypoint coverage now includes the minimized mixed CommonJS/ESM interop slice plus the `vue/runtime-dom` browser branch, typed export branches, browser replacement-map JS-entrypoint coverage, and module-entry-chain JS-entrypoint coverage, so the browser corpus keeps the first-class-JavaScript path honest across replacement-map, typed export, interop, and chained-module cases. The browser corpus now also exercises the host-heavier `@mariozechner/pi-coding-agent` package-content probe on `.js` input for `check` and `build --bundle`, so the browser deployability lane now distinguishes package content from the separate standalone bin-entrypoint rejection. The browser runtime corpus now also records the browser-condition / browser-deno preference `.js` slice and now exercises that probe on the browser `run` path as well as `test`, so the runtime harness matrix stays explicit about the browser-vs-deno preference case too.
- Progress: browser-targeted exports-map package corpus coverage now also mirrors the same package set onto `.js` input, keeping the browser analysis/build lane aligned with first-class JavaScript compilation instead of only the TypeScript lane.
- Progress: the browser package corpus now also exercises the canonical pure-JS `semver` probe on `.js` input, and the package-corpus matrix now records that browser semver slice explicitly, keeping the support-rung evidence aligned with the browser-targeted first-class-JavaScript path instead of only the TypeScript lane.
- Progress: the browser package corpus now also mirrors the browser module-entry fixture onto `.js` input, keeping the browser deployability evidence aligned across the JS and TS lanes for the module-entry shape too.
- Progress: the browser package corpus now also mirrors the browser string-entry, string-export, browser-condition export, browser dual-exports, web-baseline primitive, and internal browser-rewrite fixtures onto `.js` input, keeping the browser deployability evidence aligned across the JS and TS lanes.
- Progress: the browser package corpus now also exercises browser condition / browser-deno preference packages on the browser-targeted `check` / `build --bundle` path, including mirrored `.js` input, so the browser resolution surface stays aligned with the runtime browser-vs-deno probe.
- Progress: the browser runtime corpus now also mirrors the browser package fixtures on `.js` input for both `run` and `test`, including the browser-vs-deno condition-preference probe in `.js` input, keeping browser deployability evidence aligned across the JS and TS lanes.

### 8.4 Registry-analysis evolution

- Extend `package-effects` / `package-audit` only through explicit command/schema revisions.
- Do not add batch, raw-URL, local-path, or project-graph behavior accidentally.
- Progress: `package-effects` now also has native JSON pretty-output coverage and quiet-mode coverage, keeping the native-JSON registry-analysis payload contract explicit across presentation controls as well as the inherited-analysis lane.
- Progress: `package-audit` now also rejects `--api`, `--compat`, and `--wasm-threads` in both text and JSON output modes, keeping the command-shape gate aligned with the shared package-analysis-specific-flag contract.

## Exit gate

- Package support statements name exact rung and context.
- Corpus results are deterministic and reproducible.
- Missing/stale dependency state remains `E6004`; non-install commands do not auto-repair.
