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

- Added phase-gated placeholders for later surfaces (`effects`, `package-effects`, `package-audit`, `build --capi`, `build --component`, and `run`/`test` API-surface selection) so the evidence suite can assert unavailability with the canonical `E5006` path instead of plain unknown-command parsing.
- Added runtime smoke coverage for those Phase-2+ gating paths alongside the existing Phase-1 JSON-envelope and artifact coverage.
- Added deterministic repeated-build smoke coverage for executable, base-library, and browser-bundle artifact outputs so the evidence suite now checks byte-for-byte stability across identical inputs.
- Added a repository regression test that pins the canonical proof-ready summary in both `README.md` and `proofs/BOUNDARY.md`, so the empty proof boundary stays aligned with the public status wording.
- Cleared the workspace-wide `cargo clippy --workspace -- -D warnings` warning set so the current CI lint lane is green instead of failing on legacy placeholder patterns.

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
  `chalk`, `commander`) that should be **installable/checkable/buildable** under Phase 1's
  package-support contract.
- For each package:
  - `kali install <pkg>` → assert lock file written and integrity verified.
  - `kali check` with the package as a dependency → assert 0 type errors from the package itself
    (unless known upstream typing issues are tracked).
  - `kali build` / `kali run` a small consumer program that uses the package → assert success.
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
- Negative test: `kali build --bundle --api node` → `E5008`.

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

- `kali install` on a project with no `kali.json` → clear error.
- `kali install --dev <pkg>` → adds to `devDependencies`, lock file updated.
- `kali install --allow-scripts <pkg>` with empty lifecycle scripts → clean no-op exit 0.
- `kali install --allow-scripts <raw-url>` → `E6009`.
- `kali install` twice with the same lock file → idempotent; lock file unchanged byte-for-byte.

### 8. Sandbox enforcement completeness

- `run/test --sandbox` positive: program within policy → completes normally.
- `run/test --sandbox` negative: program violates `allow.net` → `E4004`, exit 1.
- `run/test --sandbox` negative: program violates `allow.read` → `E4004`, exit 1.
- `check/build --sandbox` with a valid policy file → exits 0.
- `check/build --sandbox` with an invalid policy file → `E9003`, exit 1.
- `build --sandbox <policy> <file>` → artifact carries `kali:policy` section; assert section
  content matches the input policy file exactly.

### 9. Proof-ready baseline maintenance

- `proofs/BOUNDARY.md` must accurately reflect the current repository proof state.
- The proof-CI trigger policy must fire on every commit that touches `proofs/`.
- In Phase 1 the proof jobs are trivially-passing stubs, but they must be present and configured.
- Update `README.md` to quote the canonical short summary verbatim:
  **"Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet."**
- Assert in CI that no chapter prose, README summary, or test claims **proof-backed** status
  while `proofs/BOUNDARY.md` describes an empty boundary.

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
| proof-trigger | on `proofs/` change | Linux x86_64 |

Establish a compile-time budget check: `kali check` on the full fixture suite must complete in
under 10 seconds on the CI reference hardware. Regressions beyond 20% flag a CI warning.

## Out of Scope

- Proof-backed claims or non-empty Lean proof files (Phase 4 target; proof-ready is sufficient).
- Package corpus expansion to Phase-3 packages (Phase 3 target).
- Node compatibility tests (Phase 3 target).

## Definition of Done

- [ ] test262 conformance subset runs in CI with an explicit pass rate and exclusion list.
- [ ] TypeScript checker baselines committed and passing.
- [ ] Package corpus (installable + checkable + runnable) passes for the selected pure-JS/TS set.
- [ ] Native-addon packages rejected correctly in corpus tests.
- [ ] Browser bundle smoke tests pass in a headless harness.
- [ ] Determinism checks pass on Linux and macOS.
- [ ] All Phase-2+ gating tests are present and pass (assert unavailability).
- [ ] `proofs/BOUNDARY.md` is accurate; proof-CI trigger configured.
- [ ] README quotes the canonical proof-ready summary verbatim.
- [ ] CI matrix is complete and all jobs pass.
- [ ] No Stage 1.1–1.13 regressions.
