# Stage 3.2 — Node Compatibility

**Phase:** 3 — Specialisation, Optimisation & Ecosystem Breadth  
**Spec refs:** [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** Phase 1 complete; can proceed in parallel with Phase 2 stages

## Goal

Open the `--api node` command path and the `kali_api_node` host-API implementation so Node.js
programs and pure-JS npm packages that rely on Node built-ins can be compiled and run under Kali.

## Workable Milestone

- `kali run --api node <file>` executes programs that use common Node built-ins (`fs`, `path`,
  `os`, `url`, `crypto`, `stream`, `events`, `http`).
- npm packages with Node-only dependencies are now **executable** (not just installable/checkable)
  when the host/API fit passes the `--api node` context check.
- `kali build --api node <file>` produces an executable WASM artifact for the Node context.

## Tasks

### 1. Node API layer (`kali_api_node`)

Implement the Phase-3 Node built-in host imports in `kali_api_node`. Priority order based on
package corpus coverage:

| Module | Phase-3 coverage target |
|---|---|
| `fs` / `fs/promises` | full async read/write/stat/mkdir/readdir/rm |
| `path` | all path utility functions |
| `os` | platform, EOL, homedir, tmpdir, cpus |
| `url` | `URL`, `URLSearchParams` (if not already in Web baseline) |
| `crypto` | `createHash`, `randomBytes`, `randomUUID`, `createHmac` |
| `events` | `EventEmitter` |
| `stream` | `Readable`, `Writable`, `Transform` (basic) |
| `http` / `https` | `createServer` (basic), `request` / `get` |
| `process` | `env`, `argv`, `exit`, `cwd`, `stdout`, `stderr` |
| `buffer` | `Buffer` class |
| `util` | `promisify`, `inspect`, `format` |
| `assert` | full assert module |
| `child_process` | `execFile`, `spawn` (sandbox-gated) |

Each built-in is implemented as Rust host-import functions registered with the wasmtime linker,
following the same pattern as `kali_api_deno`.

Progress note: the repository now has an initial pure-Rust Node helper layer in
`kali_api_node` covering process/path/crypto/events/buffer/util primitives and unit tests.
Runtime wiring and `--api node` enablement are still pending.

### 2. `--api node` command path

Wire the `--api node` flag:

```
kali run --api node <file>
kali build --api node <file>
kali check --api node [files...]
kali test --api node [files...]
```

The Node context uses a different host-import table from the Deno context. The type declarations
for Node built-ins come from `@types/node`.

### 3. Package support expansion

With `--api node` available, extend the package-support decision order to include Node-assuming
packages:

- Packages that previously failed at **host/API fit** (`E6005`) may now pass if their Node
  built-in usage is covered by the Phase-3 Node API layer.
- Update the package corpus tests to include Node-assuming packages.

### 4. Tests

- `kali run --api node fixtures/node-fs.ts` → reads/writes a file using `fs/promises`.
- `kali run --api node fixtures/node-crypto.ts` → computes a SHA-256 hash.
- `kali test --api node fixtures/node-tests/` → test suite using Node APIs passes.
- npm package corpus: `express` (basic), `axios`, `chalk` (Node colour output).
- Negative: `kali build --bundle --api node` → `E5008` (contradiction).

## Out of Scope

- Full Node.js API compatibility (Later compatibility; Phase 3 covers the most common built-ins).
- `node-gyp` / native addons (rejected by default; N-API bindings remain outside the pure JS/TS
  package contract).
- Executable `eval` / `Function()` (Phase 4 target).

## Definition of Done

- [ ] `kali run --api node <file>` executes programs using `fs/promises`, `path`, `crypto`,
  and `http` built-ins correctly.
- [ ] `kali test --api node <dir>` runs a test suite that uses Node APIs.
- [ ] npm package corpus expanded to include at least one Node-assuming package (e.g.
  `axios`, `express` basic); all newly added packages pass at their documented rung.
- [ ] Negative test: `kali build --bundle --api node` still returns `E5008`.
- [ ] Phase-1 and Phase-2 gating tests for `--api node` updated to positive coverage.
- [ ] All Phase-1 and Phase-2 tests continue to pass without regression.
