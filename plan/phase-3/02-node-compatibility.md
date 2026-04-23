# Stage 3.2 — Node Compatibility

**Phase:** 3 — Specialisation, Optimisation & Ecosystem Breadth  
**Spec refs:** [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** Phase 1 complete; can proceed after [3.1 — Optimization & Specialization](01-optimization-and-specialization.md)

## Goal

Open the `--api node` command path for the **documented Phase-3 Node subset** so Node-oriented
programs and pure-JS packages that depend on common Node built-ins can be compiled, checked,
built, run, and tested under Kali without pretending to full Node parity.

## Workable Milestone

- `kali check/build/run/test --api node ...` are available for the documented Phase-3 Node subset.
- Pure-JS packages whose host fit depends on that subset can move from blocked-at-host-fit to the
  documented support rung.
- The Node subset stays aligned with the maturity matrix instead of importing later Node breadth
  early.

## Progress

**Status:** Complete for the Phase-3 subset.

The current repo hardening pass also keeps the semver probe honest: the browser-bundle
smoke harness now has explicit `kali:rt` console shims (`console_log`, `console_error`,
`console_warn`) so emitted browser bundles instantiate cleanly under the Node-based harness,
and the regression suite now pins both the default-standalone rejection for a semver-style
`require('../package.json')` package-bin entrypoint, the Node-path help-path smoke for a
`semver/bin/semver.js`-shaped fixture, and a Node-path package-json/version plus guest-argument
count smoke on the documented Node surface. The codegen/runtime path now keeps
`process.argv.length` wired to the invocation args, the Node compatibility helper now also
records a deterministic `argv0` projection from the host launch vector, and the helper test suite
now pins that projection directly through `NodeRuntimeProjection::from_host_context`, so the
semver probe and related package-bin cases stay anchored to the actual package-binary shape with
both no-arg and guest-arg coverage.

## Historical stage tasks

### 1. Node API layer (`kali_api_node`)

Implement the documented Node subset in one explicit compatibility layer.

Phase-3 target subset from the owning spec:

| Module / global | Phase-3 scope |
|---|---|
| `fs`, `fs/promises` | file-system operations needed by real packages first |
| `path` | path manipulation |
| `buffer` | `Buffer` class and common conversions |
| `events` | `EventEmitter` |
| `util` | selected utilities such as `promisify` / `inspect` / `format` |
| `url` | URL parsing / resolution |
| `assert` | assertion helpers |
| `process` | `env`, `argv`, and the selected query/control helpers explicitly in Phase 3 |

Important boundary:
- later-breadth modules such as `os`, `child_process`, `http`, `https`, `crypto`, `stream`, and
  other remaining core modules stay on the later-compatibility path unless and until the owning
  spec moves them.
- process identity/control surfaces such as `pid`, `exit`, and `cwd`/`chdir` remain outside this
  stage and are tracked with the later host-control work.

### 2. `--api node` command path

Wire the Node API surface through the ordinary source-graph commands:

```bash
kali check --api node [files...]
kali build --api node <file>
kali run --api node <file>
kali test --api node [files...]
```

Inherited config must behave the same way as the explicit flag. Node mode must not partially leak
through the default standalone context.

### 3. Package support expansion

Update the package-support decision order for Node-context projects:

- packages previously blocked only on **host/API fit** may now be checkable/buildable/executable
  when the needed built-ins are inside the documented Node subset
- the package corpus must record the exact rung claimed for each representative package
- native addons / N-API / binary/bootstrap-heavy packages remain outside scope

### 4. Evidence

- positive `check/build/run/test --api node` coverage for the documented subset
- package-corpus fixtures for representative Node-assuming pure-JS packages
- negative tests proving later Node breadth stays gated
- CLI and diagnostics tests keeping inherited Node config aligned with explicit `--api node`

## Follow-up work uncovered by the semver probe

The `semver` package is a useful real-world Phase-3 Node-subset probe because its CLI entrypoint
combines `process.argv`, CommonJS `require()`, package-relative JSON loading, and a non-trivial
help-text path.

### Systematic fix plan

1. Add `semver/bin/semver.js` as a representative Node-compatibility fixture.
2. Prove the fixture works both with no guest args (help path) and with `-- 1.2.3` argument
   passthrough.
3. Keep the expected surface narrowly scoped to the documented subset:
   - CommonJS `require()`
   - `process.argv`
   - package-relative file / JSON resolution
   - ordinary console output
4. Add a paired negative test under the default standalone surface so the same package bin still
   fails honestly when `--api node` is not selected.
5. Record the exact achieved support rung for `semver`: installable in Phase 1, library-consumable
   once CommonJS package lowering is faithful, and CLI-executable only on the Node path.

## Out of Scope

- full Node.js parity
- native addons / `node-gyp` / N-API packages
- process identity/control and working-directory APIs tracked as later compatibility
- later-breadth Node modules that the owning spec still leaves outside the Phase-3 subset
- executable `eval` / `Function()` compatibility (Phase 4)

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
