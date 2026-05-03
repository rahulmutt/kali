# Phase 12 — Runtime, Host, and Capability Expansion

## Goal

Expand host/runtime capability only where Kali can mediate, test, and describe it honestly.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 12.1 Threaded runtime semantics

- Complete guest-facing threaded execution beyond profile acceptance and host-import plumbing.
- Keep positive thread budgets valid only under the supported threaded profile and target.
- Preserve no tracing/background GC, AOT-only compilation, deterministic JSON, and resource-budget enforcement.

### 12.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` should become a stable standalone browser runtime contract.
- If promoted, specify host ownership, sandbox limitations, summary-file fallback rules, supported commands, and diagnostics before changing support wording.
- Keep browser-targeted `check` / `build --bundle` distinct from standalone browser execution and post-deployment sandbox enforcement.
- Progress note: the browser runtime contract diagnostics now also spell out the summary-file fallback rule, where stdout remains authoritative when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid.

### 12.3 Late host APIs

- Add environment materialization/mutation, process identity/control, cwd/chdir, subprocess, and socket/listener APIs only with explicit effect keys, policy behavior, and resource limits.
- Keep host visibility aligned with `apiSurface` and command context.
- Maintain canonical gates for unavailable Node/Deno/browser host members.
- Progress note: the documented Node surface now exposes the read-only `process.pid` query across the explicit and inherited `check` / `build` / `run` / `test` paths and now also exposes the read-only `process.cwd` query there; the documented Node surface now also allows `process.chdir` as the sandbox-policy-mediated cwd mutation API, and `process.exit` is now available on that Node surface as the process-control exit API, with direct CLI smoke now pinning that exit path in JS input across `check` / `build` / `run` / `test`; the Node surface now also accepts the bracketed `process["cwd"]` / `process["chdir"]` / `process["exit"]` forms and their `globalThis.process[...]` spellings, and the Node surface now also supports the `process.env.KEY = ...` / `delete process.env.KEY` property-mutation slice in JS input on the supported `check` / `build` / `run` / `test` paths, including the bracketed `process["env"]["KEY"] = ...` / `delete process["env"]["KEY"]` spellings and the nested bracketed `globalThis["process"]["env"]["KEY"] = ...` / `delete globalThis["process"]["env"]["KEY"]` spellings alongside the equivalent `globalThis.process[...]` forms. The default standalone surface now also exposes the read-only `Deno.cwd` query and now also allows `Deno.chdir` and `Deno.exit` for sandbox-policy-mediated cwd mutation and exit control, including the bracketed `Deno["cwd"]` / `Deno["chdir"]` / `Deno["exit"]` spellings and the `globalThis["Deno"]["cwd"]` alias coverage now pinned in smoke tests. The browser-targeted late-compatibility matrix now also rejects the `Deno.pid` / `Deno["pid"]` / `globalThis.Deno.pid` / `globalThis["Deno"]["pid"]` process-identity slice, closing the browser-only hole while keeping the default standalone exception intact. The Node helper layer also now offers a deterministic environment snapshot helper on the process projection, plus a JSON-ready snapshot helper for the same view, and the Deno runtime projection now mirrors that same deterministic snapshot helper for host-context plumbing, which keeps the later mutable-env materialization path concrete without changing the current CLI availability gate. `Deno.env.has` now shares the same read-only env view as `Deno.env.get`; the standalone run/test smoke now also mirrors the bracketed `Deno["env"]["has"]` / `globalThis["Deno"]["env"]["has"]` forms, `build_tests.rs` now also mirrors the same bracketed `has` forms in the executable build path and `.js` input, and the default standalone surface now also accepts `Deno.env.set` / `Deno.env.delete` plus their bracketed `Deno["env"]["set"]` / `Deno["env"]["delete"]` forms on that same env view. The standalone and browser late-compatibility smoke now also cover the mixed dot/bracket `globalThis.Deno["env"]["set/delete"]` and `globalThis["Deno"].env["set/delete"]` spellings on that mutable env slice. The default standalone late-compatibility smoke now also pins property-mutation rejection for `process.env.KEY = ...` / `delete process.env.KEY`-style source forms, including the bracketed and `globalThis.process[...]` spellings, plus the nested bracketed `globalThis["process"]["env"]["KEY"]` forms. `Deno.env.toObject` remains gated until a dedicated object-return lowering path exists, and do not widen browser support wording on the basis of the host snapshot helper alone. The Node surface rejection coverage also now includes the mixed-bracket `globalThis["Proxy"].revocable` spelling so the late object-model gate stays explicit across bracket/dot aliases. Coverage hardening now also pins the remaining mixed dot/bracket env-mutation spellings in the JS-input late-compatibility helpers so the source fixtures and rejection assertions stay aligned. The standalone Deno host-control smoke now also covers `globalThis.Deno["chdir"]` / `globalThis["Deno"]["chdir"]` and the `Deno.exit` / `Deno["exit"]` / `globalThis.Deno["exit"]` / `globalThis["Deno"]["exit"]` alias family on the supported standalone path.

### 12.4 Late object/runtime APIs

- Triage `Proxy`, own-property helpers, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, and `Atomics` against no-GC/no-JIT and optimization constraints.
- Promote only with conformance evidence and sandbox/resource implications documented.
- Progress note: the non-browser literal-object `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice now folds static object literals and const alias chains in JS input; browser-targeted static-object-call slices are now supported on the documented browser command set, browser-bundle smoke now also covers the bracketed `globalThis["Object"]["hasOwn"]` / `globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]` forms in TS and `.js` input, browser-requested `run` / `test` browser-harness smoke now also mirrors the same static-object-call slice in TS and `.js` input, and the supported static-object slice also accepts `as const` wrappers around the object operand in JS input; dynamic-object cases remain gated. The late object-model smoke now also explicitly exercises `new Proxy` / `new globalThis.Proxy` / `new globalThis["Proxy"]` rejection spellings on the gated path.

### 12.5 Object-aggregate materialization

- Add guest-language object aggregate lowering for host-snapshot materialization APIs such as `Deno.env.toObject` only once the object aggregate/value plumbing is in place.
- Keep `Deno.env.toObject` and related host-snapshot APIs gated until the compiler can build and pass object values honestly across the supported runtime surfaces.
- Progress note: the current env-snapshot helper plumbing is already present on the host-side Deno/Node projections, and the Deno runtime projection now also exposes a JSON-ready snapshot helper alongside the BTreeMap view, but the language-visible `Deno.env.toObject` path is still blocked by the missing object-aggregate lowering path, so the repository keeps that gate explicit instead of overclaiming the surface.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Browser/runtime support wording names exact command/context/profile.
- Unsupported host/object surfaces fail through the canonical diagnostic path.
