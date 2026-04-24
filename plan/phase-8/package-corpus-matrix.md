# Package Corpus Matrix

This matrix is a deterministic planning snapshot for the package-corpus evidence currently checked into the repository.

- It is **not** an availability matrix; public support still comes from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
- Rows group packages by source kind, package shape, host/API fit, command surface, and the support rung that the current corpus evidence exercises.
- Evidence links point to the corpus test file that owns the slice.

## Browser-targeted corpus

| Source kind | Package shape | Host/API fit | Command surface | Support rung exercised | Evidence |
|---|---|---|---|---|---|
| npm-style package corpus | exports-map packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser replacement-map packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser replacement-map packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser string-entry packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser string-export packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser-condition export packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser web-baseline primitive packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser internal browser-rewrite packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | mixed CommonJS/ESM interop packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | browser-condition / browser-string / web-baseline packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | scoped packages with exports maps | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |
| browser runtime corpus | browser package fixtures | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |

## Default standalone corpus

| Source kind | Package shape | Host/API fit | Command surface | Support rung exercised | Evidence |
|---|---|---|---|---|---|
| npm-style package corpus | mixed CommonJS/ESM interop packages | default standalone | `run`, `test`, `build` | executable / testable / buildable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | exports-map packages | default standalone | `run`, `test`, `build` | executable / testable / buildable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | module-entry packages and module-entry chains | default standalone | `run` | executable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | web-baseline primitive packages | default standalone | `run` | executable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | string-export packages | default standalone | `run` | executable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | pure JS utility packages (`date-fns`, `zod`, `plimit`, `ms`) | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | pure JS package with `test` coverage (`semver`) | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) | default standalone | `check`, `build` | checkable / buildable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | default standalone | `check`, `build` | checkable / buildable | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | scoped packages | default standalone | `run` | executable | `crates/kali_cli/tests/package_corpus.rs` |

## Node corpus

| Source kind | Package shape | Host/API fit | Command surface | Support rung exercised | Evidence |
|---|---|---|---|---|---|
| npm-style package corpus | runner packages (`vitest`, `jest`, `mocha`, `ava`) | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | runner packages with exports maps | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | runner packages with exports maps with `.js` entrypoints | Node | `run`, `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | runner packages with mixed-format entries | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | runner packages with mixed-format entries with `.js` entrypoints | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |
| npm-style package corpus | pure JS `semver` probe | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |
| binary-entrypoint probe | `@mariozechner/pi-coding-agent` bin entrypoints | Node | `run` | executable on the Node surface; rejected on the default standalone surface | `crates/kali_cli/tests/package_corpus.rs` |
| package-resolution corpus | Node-assuming packages | Node vs default standalone contrast | `check`, `run` vs rejection paths | gated on the Node surface; rejected by default standalone | `crates/kali_cli/tests/package_corpus.rs` |

## Deno corpus

| Source kind | Package shape | Host/API fit | Command surface | Support rung exercised | Evidence |
|---|---|---|---|---|---|
| Deno-host package corpus | host-control packages (`Deno.env`, `Deno.Command`, `Deno.listen`, `Deno.serve`) | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |
| JSR corpus | `jsr:` packages materialized as on-disk package entries | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |

## Notes

- The matrix intentionally tracks **current evidence slices**, not a full package-support catalog.
- Command availability and public support remain phase-gated elsewhere; this file only records where corpus evidence currently exists.
- Registry-analysis commands (`package-effects`, `package-audit`) are tracked in their own schema/command work packets and are not conflated with the package-corpus rows above.
