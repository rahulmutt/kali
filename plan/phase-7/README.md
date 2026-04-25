# Phase 7 — Runtime, Host, and Platform Expansion

## Goal

Add runtime and host breadth without weakening sandbox honesty or confusing deployment targets.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 7.1 Threaded runtime profile

- Implement `--wasm-threads` / `runtimeProfiles = ["wasm-threads"]` as an explicit opt-in.
- Support positive `maxThreads` only when the threaded profile is active and supported.
- Add tests for zero-capable budgets vs positive thread budgets.
- Preserve no tracing/background GC and AOT-only compilation.
- Progress: the runtime and `run`/`test` CLI paths now accept the threaded profile on supported execution contexts, and `check` / `build` / `effects` now also accept it on the supported non-browser analysis/build paths; positive `--max-threads` values are honored when that opt-in is present. Browser-targeted and registry-analysis rows still gate the profile separately, and deterministic guest-facing thread-spawn host import plumbing is now in place. Regression coverage now also pins the browser-targeted and registry-analysis rejection paths for `--wasm-threads`, including browser-build smoke coverage in both text and JSON output modes, while fuller lowering / multi-worker execution semantics remain follow-up work. The browser-targeted `effects` lane now also has JSON-output rejection coverage for the threaded-profile gate, so the browser-analysis boundary stays explicit in both human and machine-readable output. The browser-targeted `check` / `build` rejection path for `--wasm-threads` now also mirrors `.js` input on both text and JSON output modes, keeping the browser-analysis/build boundary aligned with first-class JavaScript compilation. The zero-capable thread-budget contract is now also mirrored in JSON smoke coverage for `run` and `test`, so `--max-threads 0` stays accepted alongside the existing positive-budget rejection cases. `effects` JSON output now also exercises combined inherited `compat.features` + `compilerOptions.runtimeProfiles` normalization, inherited-browser `package-effects` rejection now also has a matching human-output regression for `--wasm-threads`, and the default standalone `package-effects` path now also has JSON-output rejection coverage for inherited threaded profiles, keeping mixed-axis inherited analysis-context serialization deterministic.

### 7.2 Standalone browser runtime contract

- Decide whether Kali will support `run --api browser` / `test --api browser` through a real browser host contract.
- If yes, specify runtime ownership, sandbox/effect limits, test harness behavior, and JSON outputs before implementation.
- Keep browser bundle/check support separate from standalone browser execution.
- Progress: the standalone browser-requested run/test path is wired behind `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` and now has regression coverage for the opt-in success path, including inherited browser `apiSurface` configs for both `run` and `test`; the default rejection path remains when the harness command is absent. The inherited-browser coverage now also exercises browser package resolution while keeping the host-contract split explicit, and the browser-harness smoke now also mirrors the package-resolution path through first-class `.js` inputs for both `run` and `test`. The browser runtime corpus now also mirrors the browser package fixtures on `.js` input for both `run` and `test`, including the browser-vs-deno condition-preference probe in `.js` input, and the basic browser-requested `run` / `test` acceptance lane now mirrors first-class `.js` inputs too, including object-enumeration smoke coverage under the configured browser harness and object-enumeration smoke coverage in `.ts` input; that lane now also exercises `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` semantics in both TS and JS input classes, and now also exercises the Web Crypto randomness subset via `crypto.getRandomValues()` on mirrored `.js` input, with the shared browser runtime formatter bounds-checking tagged string handles so negative numeric logs stay numeric. The same browser-requested run/test lane now also exercises browser Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in mirrored `.js` input, boolean conjunction/disjunction semantics in both TS and `.js` input classes, and `try/finally` sequencing in both TS and `.js` inputs under the configured browser harness, plus optional chaining in `.js` input. Browser bundle crypto web API smoke now also mirrors `crypto.subtle.digest()` / `crypto.randomUUID()` in both TS and `.js` inputs, keeping the browser-requested and browser-bundle execution surfaces aligned with first-class JavaScript compilation. Browser-targeted `check` / `build --bundle` coverage now also rejects late Deno/process host-control members on mirrored `.js` input, keeping the browser ambient surface separated from later standalone host-control APIs. The browser-requested `run` / `test` harness path now also carries JSON-output rejection coverage for late object-model members on mirrored `.js` input, and standalone `run` / `test` now also carry the same JSON-output rejection coverage for the late compatibility set in `.js` input, so the opt-in runtime contract stays aligned across both human-readable and machine-readable failure modes.

### 7.3 Late host APIs

- Add mutable env, subprocess, socket/listener, process identity/control, cwd/chdir, and similar APIs only with explicit policy/effect/resource contracts.
- Ensure host API visibility matches the selected `apiSurface`.

### 7.4 Late object/runtime APIs

- Triage `Proxy`, `WeakMap`, `WeakSet`, `FinalizationRegistry`, `SharedArrayBuffer`, `Atomics`, and broader `Intl`.
- Require conformance fixtures and memory-model review before promotion.
- Progress: late host-control, broader `Intl`, and late object-model rejection coverage now also has mirrored `.js` input fixtures across the `check` / `run` / `test` paths, and browser-targeted `check` / `build --bundle` now also carries mirrored `.js` rejection coverage for broader `Intl` plus late object-model members; the browser late-process-control browser lane now also has JSON-output coverage for `check` and `build`, and standalone `build` now also rejects the same late compatibility members on `.js` input, keeping the first-class JavaScript surface aligned with the current later-compatibility gates. The browser-requested `run` / `test` harness path now also rejects late object-model members on `.js` input and now also carries matching JSON-output coverage, keeping that opt-in runtime contract aligned with the same later-compatibility gate coverage. The threaded runtime boundary now also mirrors `SharedArrayBuffer` / `Atomics` rejection coverage onto `.js` inputs for the supported `check` / `run` / `test` paths, and the browser-requested harness path now also rejects those threaded globals in both human-readable and JSON output modes.

## Exit gate

- Every newly supported host/runtime capability has sandbox, effect, resource, and integration coverage.
- Browser runtime claims are backed by real browser execution evidence if opened.
- Unsupported contexts still fail with canonical diagnostics.
