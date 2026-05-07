# Phase 17 — Host/Runtime Contract Expansion

## Goal

Expand runtime and host capability only where Kali can mediate, test, and describe it honestly.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 17.1 Threaded runtime semantics

- Complete guest-facing threaded behavior beyond profile acceptance and helper plumbing.
- Define valid positive thread budgets by command, API surface, and runtime profile.
- Preserve AOT-only compilation, no tracing/background GC, deterministic JSON, and resource-limit honesty.
- Current progress: the positive `--max-threads` rejection path now carries both the resource-budget and threaded-profile config hints in text diagnostics, while the JSON error payload keeps the canonical `resources.maxThreads` message stable and now also includes structured CLI context for the explicit `--max-threads` request; runtime smoke now also exercises guest thread spawning through the host import path, including budget-exhaustion rejection for a second spawn under a one-thread limit, and browser-requested run/test harness smoke now also accepts positive `--max-threads` overrides when `--wasm-threads` is active in JS/TS/JSX/TSX input with JSON-output coverage, while standalone `check` / `run` / `test` smoke with an attached sandbox policy now also accepts positive `resources.maxThreads` when the threaded profile is active, and browser-targeted `check` / `build --bundle` rejection coverage for the same threaded-profile gate now also mirrors JSX and TSX input on both explicit and inherited browser API-surface paths.
- Current progress: browser runtime summary parsing now treats unexpected top-level keys as shape-invalid and falls back to stdout when the configured summary file drifts from the contract; browser-requested JS run/test smoke now also covers that unexpected-top-level-key fallback path end-to-end, and the dedicated TS/JSX/TSX browser-harness fallback suites now mirror that same drift path in JSON test coverage. Empty or whitespace-only `hostContract` / `runtimeBackend` summary labels are also treated as absent so browser harness summaries keep sourcing provenance labels from stdout when the file leaves them blank, trimmed canonical labels normalize to the same summary contract instead of tripping a spurious fallback, and the runtime regression suite now pins that whitespace-only-label fallback in both browser-requested and browser-bundle harness paths. The runtime regression suite also keeps the padded-label case aligned with the stdout merge path when `testsFailed` is absent from the file, so trimming and fallback stay coupled; JS input now also carries the missing `testsFailed` merge regression directly in the dedicated browser-summary fallback suite, and malformed `testsFailed` values now also mark the summary file shape-invalid so the browser harness falls back to stdout instead of partially trusting the file. Browser summary `args` / `tests` items now also reject leading or trailing whitespace and fall back to stdout, keeping the browser-harness summary contract canonical when those arrays drift.

### 17.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to stable standalone browser runtime support.
- Before any promotion, specify host ownership, summary JSON behavior, sandbox limitations, diagnostics, and failure modes.
- Keep browser-targeted `check` / `build --bundle`, harness execution, and post-deployment browser behavior separate.

### 17.3 Late host APIs and resources

- Add subprocess, socket/listener, worker/thread, env materialization, env mutation, cwd/process-control, and late Node/Deno module support only with policy/effect/resource contracts.
- Keep host visibility aligned with effective `apiSurface`, command family, runtime profile, and maturity gate.
- Preserve explicit gates for unavailable host members.
- Current progress: the late process-control gate now also keeps `process.kill` explicitly rejected in the standalone runtime-smoke lane, and the Node API-surface regression suite now mirrors that same rejection across JS, JSX, and TSX input on `check` / `build` / `run` / `test`, so the Node process-control row stays narrower than the documented `pid` / `cwd` / `chdir` / `exit` support slice. Browser late-compat smoke now also rejects `process.kill` in the browser JS, TS, JSX, and TSX compatibility lanes, and browser-targeted `check` / `build --bundle` smoke now also rejects that same late process-control slice on the explicit browser API surface for TS, JSX, and TSX input.

### 17.4 Late object/runtime APIs

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints.
- Promote only with conformance, sandbox/resource, and JSON-output evidence.
- Current progress: browser late-compat TSX smoke now also mirrors the full `Proxy.revocable` alias family, including the bracketed `globalThis["Proxy"].revocable` and `globalThis.Proxy["revocable"]` spellings, alongside the existing gated object-model aliases so the TSX late-object gate stays aligned with the JS/browser coverage; browser late-compat JSX smoke now also mirrors the same late object-model gate for the canonical broader-Intl / Proxy.revocable / weak-reference / finalization / SharedArrayBuffer / Atomics slice, keeping the JSX and TSX browser-harness gates aligned; runtime smoke now also pins the expanded `Proxy.revocable` alias family across the canonical `run` / `test` / `effects` regression lanes, keeping the canonical `proxy-traps` rejection and effect classification in sync; sandbox effect analysis now also classifies the bracketed `Proxy.revocable` family as `proxy-traps` so the `effects` surface matches the same late-object alias set; the broader-Intl browser late-compat JS fixtures now also include the `globalThis["Intl"].PluralRules` / `globalThis.Intl["PluralRules"]` alias forms alongside the existing broader-Intl spellings, and now also carry the bracket-root dot forms for `globalThis["Intl"].RelativeTimeFormat` / `Collator` / `DisplayNames` / `Segmenter` / `Locale` so the Intl alias family stays symmetric; the broader-Intl JS/JSX/TSX browser late-compat fixtures now also include the mixed `globalThis["Intl"].NumberFormat` / `globalThis.Intl["NumberFormat"]` and `globalThis["Intl"].DateTimeFormat` / `globalThis.Intl["DateTimeFormat"]` bracket spellings in the shared helper input, and now also carry the broader `RelativeTimeFormat` / `Collator` / `DisplayNames` / `Segmenter` / `Locale` / `PluralRules` spellings in the same late-compat matrix, while the surfaced rejection wording continues to normalize through the canonical broader-Intl gate.
## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
