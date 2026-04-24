# Current Implementation Baseline

This file records the planning baseline observed before cleaning up the old completed plan phases.

## Repository evidence

- `cargo test --workspace` passes in the current workspace.
- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current CLI command set.
- The workspace contains implementation crates for CLI, frontend, IR, codegen, runtime, sandbox/effects, package management, optimization, host API surfaces, embedding, C ABI, and bindings.
- The proof tree exists under `proofs/` and `proofs/BOUNDARY.md` states that Kali is proof-backed for the published boundary.

## Live surface described by specs and README

The current repo has already implemented the historical MVP and several later surfaces:

- `kali init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`.
- `kali build --validate-ir`, `--bundle`, `--lib`, `--capi`, and `--component`.
- `kali test --coverage`.
- Runtime smoke coverage includes `Object.keys()`, `Object.entries()`, and `Object.values()` enumeration semantics alongside the earlier arithmetic, async/await, and built-in regressions, now including integer-like key ordering plus `Math.abs()` and `Math.sign()` alongside the existing `Math.max()` / `Math.min()` coverage, now also mirrors BigInt addition into `.js` run/test coverage, now also exercises the Web baseline randomness subset via `crypto.getRandomValues()` on mirrored `.js` run/test coverage, and now also mirrors arithmetic precedence and array literal length into `test`-path `.js` smoke coverage in addition to the existing Math built-in mirrors on both TS and `.js` inputs.
- The parser now accepts async function declarations/expressions and async generator syntax as AST forms, plus generator function syntax and `yield` expressions, while typechecking still gates generator lowering with the canonical `E5506` path until a real lowering packet lands; that rejection is now covered on the `check`, `build`, `run`, and `test` CLI smoke paths, including async generator syntax and mirrored async-generator `.js` fixtures, and the browser-targeted `check` / `build --bundle` smoke lane now also pins the same generator-lowering rejection in the browser analysis/build context.
- `kali effects`, `kali package-effects`, and `kali package-audit`.
- Deno-oriented baseline plus documented Node subsets for source-graph and effect/reporting workflows.
- Browser-targeted `check` and `build --bundle` remain distinct from standalone browser runtime support, while the standalone browser-requested `run`/`test` harness coverage now also exercises inherited browser `apiSurface` configs when `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` is configured, including browser package-resolution coverage on the inherited path. The browser runtime corpus now also mirrors the browser package fixtures on `.js` input for both `run` and `test`, and the basic browser-requested `run` / `test` acceptance lane now mirrors `.js` inputs too, so the browser-requested execution surface stays aligned with first-class JavaScript compilation. Browser-targeted `check` / `build --bundle` coverage now also rejects late Deno/process host-control members on mirrored `.js` input, keeping the browser ambient surface separated from later standalone host-control APIs.
- Browser-targeted package-corpus coverage now includes `.js` entrypoints for browser replacement-map packages, including a scoped package case and a `vue/runtime-dom` browser replacement-map JS-entrypoint case, keeping first-class JavaScript compilation honest in the browser analysis/build lane.
- Default standalone package-corpus coverage now also mirrors its minimized mixed-format CommonJS/ESM interop fixture on `.js` input for `run`, `test`, and `build`, keeping first-class JavaScript package evidence aligned with the existing TypeScript lane.
- Node package-corpus coverage now also exercises the canonical pure-JS `semver` probe on `.js` input across the documented Node `check` / `build` / `run` lanes, keeping the Node compatibility slice aligned with first-class JavaScript compilation instead of only the TypeScript lane.
- Browser bundle chunk smoke coverage now also exercises literal and const-bound dynamic imports from `.js` input, and the browser bundle runtime smoke now also exercises the dynamic-import loader for `.js` input including directory-index targets, so the browser `build --bundle` lane keeps the linked-graph chunking and loader paths aligned with first-class JavaScript compilation.
- Deno package-corpus coverage now also exercises a canonical `jsr:@std/path` package fixture materialized at `node_modules/@std/path` on the Deno surface, keeping the `jsr:` registry prefix and on-disk path mapping honest in the package-resolution evidence.
- `--compat eval` exists for the documented compatibility path.
- The threaded runtime profile is now accepted on supported `run`/`test` execution paths when explicitly opted in with `--wasm-threads`, and `check` / `build` / `effects` now also accept that opt-in on the supported non-browser analysis/build paths; positive `--max-threads` values are honored only under that opt-in, and the runtime now also exposes deterministic guest-facing thread-spawn host import plumbing backed by the threaded topology model.
- Schema-v1 outputs and diagnostic envelopes are first-class contracts, including combined inherited analysis-context normalization for `effects` JSON output across `compat.features` and `compilerOptions.runtimeProfiles`.
- Schema contract checks now also pin the `schemas/result/package-effects/v1.json` object payload and `schemas/result/package-audit/v1.json` null payload shapes so the registry-analysis schema-v1 documents stay aligned with their CLI contracts.
- The schema-document regression now also covers the supporting diagnostic, manifest, lockfile, and sandbox-policy schema documents, keeping the config/policy/result sidecar contracts aligned with the CLI surface.
- `package-audit` quiet-mode coverage now also suppresses human output while inheriting `eval` plus `wasm-threads`, keeping the context-free registry-analysis command honest when `--quiet` is present.

## Planning consequence

The old phase documents were historical implementation records. Keeping them as active checklists caused stale references to completed tasks. Active planning now starts from the current baseline and focuses on remaining spec-owned work only.

## Non-negotiable guardrails

Future phases must preserve:

- AOT-only guest-language compilation.
- Pure Rust implementation contract.
- No tracing/background GC.
- Sandbox-first honesty.
- Deterministic machine contracts.
- Public availability discipline from `specs/19-feature-maturity.md`.
- Proof-backed claim discipline from `proofs/BOUNDARY.md`.
