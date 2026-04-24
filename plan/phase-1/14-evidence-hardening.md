# Stage 1.14 — Evidence Hardening

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/16-testing.md`](../../specs/16-testing.md), [`specs/17-verification.md`](../../specs/17-verification.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.13 — Diagnostics & Schemas](13-diagnostics-and-schemas.md) (all Stage 1.1–1.13 features must exist to be evidenced)

## Goal

Harden the Phase-1 evidence base so Kali's Phase-1 maturity labels are honest and backed by
measurable tests across all canonical evidence tracks: language conformance, type-checker
baselines, package corpus, browser bundle smoke tests, determinism checks, and the proof-ready
verification baseline. This stage is not about adding new user-visible features — it is about
closing gaps in the test/CI coverage that previous stages may have left.

## Workable Milestone

- All evidence tracks from `specs/16-testing.md` have meaningful positive coverage for every
  Phase-1 shipped surface.
- Every Phase-2+ command/artifact family has explicit negative/gating tests asserting it is
  unavailable.
- The proof-ready baseline (`proofs/BOUNDARY.md`) is up-to-date and the proof-CI trigger policy
  is exercised in CI.
- The repository can honestly claim the Phase-1 maturity labels in `specs/19-feature-maturity.md`.

## Progress

- Added a recursive-discovery regression that proves no-argument `check` stops at nested child projects with their own `kali.json`, keeping the project-walk boundary honest instead of accidentally pulling child-project diagnostics into the parent workspace walk.
- Added an `effects` JSON-output regression that ignores a top-level `sandbox` config path, so source-graph analysis stays decoupled from policy-attachment plumbing just like the registry-analysis hardening already does.
- Added a dedicated determinism smoke lane in `scripts/check-determinism.sh` plus a matching `mise` task and CI job, so the repeated-build evidence is now exercised as an explicit repository workflow instead of living only inside the larger runtime smoke suite. The lane now also pins the `effects`, `package-effects`, and `package-audit` repeated-invocation envelopes, including the quiet-mode pretty JSON envelope variants that keep the browser-context effects path and the package-analysis reporting lanes deterministic alongside the build artifacts.
- Tightened the `package-audit` findings sort key so span position now acts as a final deterministic tie-breaker after severity, code, message, notes, and suggestion, preventing same-message findings from relying on incidental registry iteration order.
- Added a package-effects flag-family regression so the inherited-analysis lane now rejects `--api`, `--compat`, `--wasm-threads`, and `--sandbox` with the canonical package-analysis-specific `E5508` message instead of only pinning a single stray flag, and added the matching JSON-output regression so the schema-v1 envelope path stays pinned as well.
- Added a package-effects missing-dependency-state regression so the registry-analysis sibling now reports the canonical `E6004` package-management diagnostic when the package has not been materialized in the current project.
- Added a package-effects orthogonality regression that keeps inherited browser analysis context aligned with a top-level sandbox path in JSON output, so the registry-analysis effect-report lane now pins the browser-resolution and sandbox-attachment axes together instead of only checking them independently.
- Added inherited-context regressions for `kali effects` so the public effect-report lane now pins both inherited `compat.features = ["eval"]` payload preservation and inherited Node API-surface rejection instead of only covering the explicit-flag path.
- Added a `package-effects` quiet-mode regression for inherited `eval` compatibility so the JSON envelope path now stays pinned even when dynamic-analysis metadata is inherited from `kali.json`.
- Added phase-gated placeholders for later surfaces (`effects`, `package-effects`, `package-audit`, `build --capi`, `build --component`, and `run`/`test` API-surface selection) so the evidence suite can assert unavailability with the canonical `E5506` path instead of plain unknown-command parsing.
- Added dedicated node-API-surface rejection coverage for the Phase-1 command surface so the package-corpus and runtime smoke lanes now pin the Node availability gate across both explicit and inherited contexts instead of leaving those expectations only in the CLI parser tests.
- Added explicit `package-audit` rejections for inherited-looking package-analysis inputs (`--compat eval` and `--wasm-threads`) so the registry-analysis gate now pins the command's context-free contract beyond just the `--api` and `--sandbox` cases.
- Added CLI-shape regressions for `effects --sandbox` in both human and JSON output so the reporting-only effects lane now rejects the sandbox flag with the canonical CLI-usage diagnostic instead of falling through to Clap's generic unknown-argument path.
- Added explicit `--api node` coverage for `effects` plus inherited Node-context coverage for `package-effects`, keeping the Phase-3 analysis context pinned in the evidence suite alongside the existing browser and threaded-profile gating regressions.
- Added runtime smoke coverage for those Phase-2+ gating paths alongside the existing Phase-1 JSON-envelope and artifact coverage.
- Added explicit browser-runtime negative-gate hardening for standalone `run --api browser` and `test --api browser` invocations so the later browser runtime rejection path stays pinned even if the browser-harness helper is present in the environment.
- Added deterministic repeated-build smoke coverage for executable, base-library, and browser-bundle artifact outputs so the evidence suite now checks byte-for-byte stability across identical inputs. That lane now also exercises `build --validate-ir` on the plain executable path, keeping the debug-validation switch pinned on the default build shape as well.
- Added explicit and inherited `apiSurface=node` smoke coverage for the `--capi` and `--component` build flows, so the documented Node embedding-build subset now has direct evidence alongside the existing Node run/test and check/build cases.
- Added raw-URL install idempotence coverage so repeated `kali install` runs over the same raw URL graph now assert lockfile byte stability.
- Added a semver package-corpus regression that proves plain `kali install semver` succeeds without `--allow-scripts` when the package only carries non-install lifecycle metadata.
- Added a configless-project `kali install --dev semver` regression that records the package in `devDependencies` and materializes the lockfile, so the documented dev-dependency install path now has the same corpus-style evidence as the regular dependency path.
- Added a CLI smoke regression for `kali install --allow-scripts semver` with empty lifecycle scripts so the evidence suite now pins the documented no-op allow-scripts path on the explicit registry-target form as well.
- Added a default-standalone rejection regression for the semver package bin, so the evidence suite now pins the honest failure path for the Node-only CLI entrypoint alongside the documented Node-path smoke.
- Added package-shape coverage for `exports`-backed native addon entrypoints so install/audit rejection stays aligned with the pure JS/TS package contract instead of only checking `main` and `bin`.
- Added a companion `kali install --allow-scripts semver` regression so the evidence suite now
  covers the no-op allow-scripts path for packages with only non-install lifecycle hooks.
- Added a no-manifest `kali install` regression so the evidence suite now proves the command stays
  a no-op on empty workspaces instead of materializing placeholder project files.
- Added CLI smoke coverage for `kali install --allow-scripts <raw-url>` so the raw-URL opt-in path
  stays pinned to the canonical `E5508` invalid-usage diagnostic before any fetch or materialization
  work can begin.
- Added a dedicated Linux runtime-smoke CI lane for the browser smoke, determinism, and negative-gating regressions, plus a nightly package-corpus lane that runs the heavier corpus suite outside the per-commit path.
- Added a package-audit regression that confirms inherited Node API-surface context is ignored just like the browser and threaded-profile contexts, keeping the registry-analysis command's context-free contract explicit in the evidence suite.
- Added parser/lexer regression coverage for the semver probe's optional-chaining and multiline-template cases so `minVersion(... )?.version` and multi-line template bodies stay covered by the evidence suite.
- Added a default-surface semver consumer smoke test so `valid`, `satisfies`, and `minVersion` now stay covered by an end-to-end package/runtime regression with exact stdout assertions instead of only by package-bin probes.
- Refined codegen fallback guidance so unresolved imported bindings/call targets keep an explicit placeholder-fallback note, now with source-path context when available, instead of leaving that behavior implicit, and kept regression coverage on the warning path.
- Added a zod package-corpus regression so another widely used pure JS/TS package now exercises the default standalone check/build/run lane with deterministic output.
- Added a Node-path semver package-bin smoke that exercises `require('../package.json').version` and guest-argument counting on the documented Node subset, so the semver probe now covers both the package-json loading slice and the argument passthrough slice directly.
- Added a node-assuming package-corpus rejection regression so default standalone package imports that pull in Node-only host APIs now fail with the canonical `E6005` diagnostic instead of silently lowering through the compatibility path.
- Added negative `kali build --lib` coverage for sources without a statically known export surface so the Phase-1 base-library evidence lane keeps enforcing `E5511`.
- Added explicit browser-library contradiction coverage for both human and JSON build output so `kali build --lib --api browser` stays pinned to the canonical `E5508` shape error.
- Added matching browser-build-shape coverage for the remaining wrong-browser artifact modes (`kali build --capi --api browser` and `kali build --component --api browser`), including JSON-output coverage for the component path, so the browser-surface rejection split stays pinned across all documented Phase-1 build shapes.
- Added browser entrypoint coverage for the Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`) so the browser-harness alias table stays pinned beyond the existing browser entrypoint cases.
- Added browser entrypoint coverage for the Google Chrome stable wrapper spelling (`google-chrome-stable`) so the browser-harness alias table stays aligned with another common stable-channel browser name family.
- Added browser entrypoint coverage for the Firefox/Opera/Vivaldi stable spellings (`firefox-esr`, `opera-stable`, and `vivaldi-stable`) so the browser-harness alias table stays pinned across additional common distro wrapper names.
- Added JSON-output coverage for the remaining browser build-shape contradictions (`kali build --api browser` and `kali build --bundle --api node`) so the command-shape vs availability split stays pinned across both human and machine-readable envelopes.
- Added sandbox artifact coverage that now asserts the embedded `kali:policy` custom section matches the source policy bytes exactly, not just the presence of the section.
- Added explicit sandbox-policy custom-section coverage for the Phase-1 library artifact and browser-bundle artifact outputs, so the `kali:policy` embedding contract stays pinned across the documented static policy-validation build lanes.
- Added invalid sandbox-policy schema regressions for `check --sandbox` and `build --sandbox`, confirming malformed policy files fail early with `E5510` instead of reaching runtime or artifact emission.
- Added positive `run --sandbox` and `test --sandbox` regressions so the evidence suite now proves benign sandboxed workloads still complete normally under the canonical deny-by-default policy shape.
- Added a Node-based browser-bundle execution smoke harness that imports the generated ESM bundle, resolves the emitted WASM, and exercises the exported wrapper for both explicit and inherited browser API-surface builds.
- Added explicit `--api browser` + `--bundle` + `--sandbox` browser-bundle regressions in both human and JSON output so the browser-targeted build lane now pins the explicit-flag path alongside the inherited-config path.
- Added a JSON-envelope browser-bundle regression for inherited browser API-surface builds so the browser-targeted build lane now proves its schema-v1 machine-readable output as well as its executable bundle artifacts.
- Added a bigint-literal lexer/parser regression so browser-bundle smoke fixtures that return `0n` no longer split the literal into a stray identifier and false positive diagnostic.
- Added a repository regression test that pins the canonical proof-ready summary in both `README.md` and `proofs/BOUNDARY.md`, so the empty proof boundary stays aligned with the public status wording.
- Wired the CI proof-check job so it now listens for `proofs/**` changes, verifies the Lean proof-tree layout, checks that the declared Lake roots match the actual proof source directories, and runs `lake build` instead of relying on an unreferenced filter output.
- Expanded the GitHub Actions build/test job into a Linux + macOS matrix so the workspace test suite and determinism coverage now run on both platforms called for by the stage plan.
- Cleared the workspace-wide `cargo clippy --workspace -- -D warnings` warning set so the current CI lint lane is green instead of failing on legacy placeholder patterns.
- Synchronized the later-surface negative-gating assertions with the spec-owned `E5506` availability code so the evidence suite now matches the canonical maturity diagnostic number end to end.

## Tasks

### 1. Language conformance suite

Following `specs/16-testing.md`:

- Import a representative subset of **test262** (the official ECMAScript conformance test suite)
  and run it against the Kali compiler pipeline.
- Target: all Phase-1 features (latest-published ECMA-262 grammar, static ESM, CommonJS
  `require("literal")`, first-class JS compilation) must pass their matching test262 cases.
- Track an explicit **unsupported-semantics list** for features that are correctly rejected by
  Kali in Phase 1 (e.g. dynamic `require()`, executable `eval`, non-literal `import()`). These
  must generate negative tests that assert unavailability rather than being silently skipped.
- Set up CI to run the conformance suite on every commit and report a pass/fail percentage.
  A regression in the pass rate (beyond the tracked exclusion list) is a CI failure.

### 2. TypeScript checker baselines

- Run Kali's type checker against a curated set of **TypeScript's own conformance fixtures**
  (from the `typescript` repository's `tests/cases/conformance/` directory).
- For each fixture, assert either that Kali produces the same diagnostic set as `tsc` or that
  any difference is intentional and documented (Kali is allowed to be stricter, not more lenient).
- Commit the baseline snapshots. A change in the snapshot is a CI failure until the baseline is
  deliberately updated.
- Focus on: generics, conditional types, mapped types, control-flow narrowing, module resolution,
  and first-class JS inference.

### 3. Package corpus tests

- Select a curated set of popular pure-JS/TS npm packages (e.g. `lodash`, `zod`, `date-fns`,
  `chalk`, `commander`, `semver`) that should be **installable/checkable/buildable** under Phase 1's
  package-support contract.
- For each package:
  - `kali install <pkg>` → assert lock file written and integrity verified.
  - `kali check` with the package as a dependency → assert 0 type errors from the package itself
    (unless known upstream typing issues are tracked).
  - `kali build` / `kali run` a small consumer program that uses the package → assert success.
- Add an explicit `semver` regression lane because it exercises multiple common failure modes at
  once:
  - plain `kali install semver` must succeed without `--allow-scripts` because `semver` has
    non-install scripts but no install-time lifecycle hooks
  - a consumer using `import { valid, satisfies, minVersion } from "semver"` must produce the same
    observable output as Node for representative calls
  - a consumer using `minVersion(... )?.version` must not regress into bogus `E3100` diagnostics
  - `kali run node_modules/semver/bin/semver.js` must fail honestly on the default standalone
    surface, while the Node-path fixture tracked in Phase 3 proves the documented gated behavior
- Select a set of packages that **must be rejected** (native addons, Node-only API users) and
  assert they produce the correct `E6004` or `E6005` diagnostic.

### 4. Browser bundle smoke tests

For the **Phase-1 browser-targeted command set**:

- Compile a fixture `kali build --bundle fixtures/browser-app.ts`.
- Load the bundle in a headless browser harness (playwright or Deno's `browser` API).
- Assert the bundle executes correctly and produces expected output.
- Cover inherited-config forms: set `compilerOptions.apiSurface = "browser"` in `kali.json`
  and run `kali build --bundle` without the explicit `--api browser` flag.
- Cover `--sandbox` variant: `kali build --bundle --sandbox fixtures/policy.json fixtures/app.ts`
  → bundle carries `kali:policy` custom section.
- Negative test: `kali build --bundle --api node` → `E5508`.

### 5. Determinism checks

Every artifact-producing command must be deterministic: identical inputs produce byte-identical
outputs.

Determinism CI step:

1. Build a representative set of fixture programs twice (clean build each time, same pinned
   package lock, same flags).
2. Assert `sha256(artifact_1) == sha256(artifact_2)` for every output file (`.wasm`,
   `.js`, `.meta.json`, `kali.lock`).
3. Run this step in CI on a matrix of platforms (Linux x86_64, macOS arm64) to catch
   platform-dependent non-determinism.

### 6. Negative / gating tests for Phase-2+ surfaces

Explicitly assert that Phase-2+ command surfaces are unavailable in Phase 1:

| Command / surface | Expected behaviour |
|---|---|
| `kali effects <file>` | exits with `E5xxx` (command unavailable) |
| `kali package-effects <pkg>` | exits with `E5xxx` (command unavailable) |
| `kali package-audit <pkg>` | exits with `E5xxx` (command unavailable) |
| `kali build --capi <file>` | exits with `E5xxx` (flag unavailable) |
| `kali build --component <file>` | exits with `E5xxx` (flag unavailable) |
| `kali run --api node <file>` | exits with `E5xxx` (API surface unavailable) |
| `kali run --api browser <file>` | exits with `E5xxx` (not a Phase-1 shipped surface) |
| `kali test --api browser [files]` | exits with `E5xxx` |
| `eval` at runtime | executes to a `PermissionDeniedError` / `ReferenceError`; does not succeed |

These tests must be committed and must continue to pass until the corresponding maturity row opens.

### 7. Install workflow tests (Phase-1 completeness)

Complete coverage of the `kali install` edge cases defined in `specs/16-testing.md`:

- `kali install` on a project with no `kali.json` → clean no-op success; it must not create a placeholder manifest.
- `kali install --dev <pkg>` → adds to `devDependencies`, lock file updated.
- `kali install --allow-scripts <pkg>` with empty lifecycle scripts → clean no-op exit 0.
- `kali install --allow-scripts <raw-url>` → `E5508`.
- `kali install` twice with the same lock file → idempotent; lock file unchanged byte-for-byte.

### 8. Sandbox enforcement completeness

- `run/test --sandbox` positive: program within policy → completes normally.
- `run/test --sandbox` negative: program violates `allow.net` → `E4004`, exit 1.
- `run/test --sandbox` negative: program violates `allow.read` → `E4004`, exit 1.
- `check/build --sandbox` with a valid policy file → exits 0.
- `check/build --sandbox` with an invalid policy file → `E5510`, exit 1.
- `build --sandbox <policy> <file>` → artifact carries `kali:policy` section; assert section
  content matches the input policy file exactly.

### 9. Proof-ready baseline maintenance

- `proofs/BOUNDARY.md` must accurately reflect the current repository proof state.
- The proof-CI trigger policy must fire on every commit that touches `proofs/`.
- The proof jobs must be present and configured, and the current repository state already runs real Lean checks through the published proof boundary.
- Update `README.md` to quote the canonical short summary verbatim:
  **"Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target."**
- Assert in CI that no chapter prose, README summary, or test claims **proof-backed** status
  while `proofs/BOUNDARY.md` describes an empty or otherwise unmechanized boundary.

### 10. CI matrix and performance budget

Complete the CI pipeline established in Stage 1.1 with the following jobs:

| Job | Frequency | Platform matrix |
|---|---|---|
| `cargo test --workspace` | every commit | Linux x86_64, macOS arm64 |
| `cargo clippy --workspace` | every commit | Linux x86_64 |
| `cargo fmt --check` | every commit | Linux x86_64 |
| conformance suite (test262 subset) | every commit | Linux x86_64 |
| checker baselines | every commit | Linux x86_64 |
| package corpus | nightly | Linux x86_64 |
| browser smoke tests | every commit | Linux x86_64 |
| determinism checks | every commit | Linux x86_64, macOS arm64 |
| negative/gating tests | every commit | Linux x86_64 |
| proof-check | on `proofs/` change | Linux x86_64 |

Establish a compile-time budget check: `kali check` on the full fixture suite must complete in
under 10 seconds on the CI reference hardware. Regressions beyond 20% flag a CI warning.

## Out of Scope

- Proof-backed claims or non-empty Lean proof files (Phase 4 target; proof-ready is sufficient).
- Package corpus expansion to Phase-3 packages (Phase 3 target).
- Node compatibility tests (Phase 3 target).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
