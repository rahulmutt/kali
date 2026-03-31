# Stage 1.9 — Sandbox & Policy

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.8 — Runtime & Execution](08-runtime-execution.md)

## Goal

Implement `kali_sandbox` — declarative policy files, runtime enforcement for `run`/`test`, and
static policy-schema/config validation for `check`/`build`. This delivers the **Phase-1 static
policy-validation surface** and the runtime enforcement half of sandbox-first execution.

## Workable Milestone

- Policy files are parsed and validated against the schema-v1 policy schema.
- `kali run --sandbox <policy> <file>` enforces the policy at runtime; a violation produces `E4004`
  and exits non-zero.
- `kali check --sandbox <policy> [files...]` validates the policy file schema/config without
  runtime execution.
- `kali build --sandbox <policy> <file>` validates policy and embeds enforcement metadata in the
  artifact.

## Tasks

### 1. Policy file schema (schema v1)

Define the declarative policy format in JSON (or TOML — follow `specs/18-schemas.md` for the
canonical format choice). A minimal v1 policy has these top-level sections:

```json
{
  "$schema": "https://kali-lang.org/schemas/policy/v1",
  "allow": {
    "read": [],
    "write": [],
    "net": [],
    "env": [],
    "run": [],
    "ffi": []
  },
  "deny": {
    "read": [],
    "write": [],
    "net": [],
    "env": []
  },
  "resourceLimits": {
    "memoryMb": null,
    "cpuTimeSec": null
  }
}
```

Permission specifiers follow Deno's convention where applicable:

- `read` / `write`: array of path globs (strings) or `true` (allow all).
- `net`: array of host:port patterns or `true`.
- `env`: array of environment variable names or `true`.
- `run`: array of executable names or `true` (currently not executable in Phase 1; reserved for
  later compatibility).
- `ffi`: `false` (rejected by default in Phase 1).

### 2. Policy parsing and validation (`kali_sandbox`)

Implement:

- `Policy::from_file(path) -> Result<Policy, Diagnostics>` — parse and validate a policy file
  against the schema. Emit structured `E9xxx` diagnostics on schema violations rather than
  returning opaque errors.
- `Policy::validate_config(&self) -> Vec<Diagnostic>` — check that the policy itself is
  internally consistent (e.g. an entry is not in both `allow` and `deny`).
- `PolicyChecker::check(op: &HostOp, policy: &Policy) -> PolicyResult` — at runtime, check
  whether a host operation is allowed by the current policy.

`E9xxx` error codes for sandbox/policy:

| Code | Meaning |
|---|---|
| `E9001` | Policy file not found |
| `E9002` | Policy file parse error (invalid JSON/TOML) |
| `E9003` | Policy schema validation error (unknown key, wrong type) |
| `E9004` | Conflicting allow/deny entries |
| `E9005` | Permission specifier syntax error |
| `E9006` | Resource limit out of range |

### 3. Runtime enforcement

Wire `PolicyChecker` into `KaliHostState` (from Stage 1.8) so every host import call is checked
before execution:

- Before performing a file-system read → check `allow.read` against the resolved path.
- Before performing a file-system write → check `allow.write`.
- Before opening a network connection → check `allow.net` against the host:port.
- Before accessing an environment variable → check `allow.env`.

On a policy violation:

1. Do **not** perform the host operation.
2. Reject the WASM call with a structured `E4004` runtime diagnostic.
3. The guest program receives a thrown `PermissionDeniedError` (matches Deno's convention).
4. If the exception is not caught by the guest, `kali run` exits with code 1 and prints the
   diagnostic.

### 4. Resource limits

If `resourceLimits.memoryMb` is set, configure the `wasmtime::Engine` memory limit accordingly
before instantiation. If `resourceLimits.cpuTimeSec` is set, use wasmtime's fuel-based execution
to enforce a rough CPU-time budget.

### 5. Phase-1 static policy-validation surface

Implement the **Phase-1 static policy-validation surface** as defined in `SPEC.md`:

- `kali check --sandbox <policy> [files...]` — validate the policy file against the schema and
  report `E9xxx` errors; do **not** run the program. **No** inferred-effect-vs-policy rejection
  yet (that is a Phase 2 extension of this same command path).
- `kali build --sandbox <policy> <file>` — validate the policy and embed it as a WASM custom
  section in the output artifact so the host can enforce it at load time.
- `kali build --lib --sandbox <policy> <file>` — same for library artifacts.
- `kali build --bundle --sandbox <policy> <file>` — browser-targeted bundle validation (the
  bundle will carry the policy for static inspection; post-deployment runtime enforcement depends
  on the browser host).

**Guardrail:** attaching `--sandbox` to an otherwise-invalid command shape (e.g. `--api node`
before Phase 3) does not make it valid. The underlying command/context pair must itself be valid
first.

### 6. Policy embedding in artifacts

When `--sandbox <policy>` is used with a `build` command, embed the validated policy as a WASM
custom section named `kali:policy` containing the canonical JSON serialisation. The runtime host
can read this section at load time to enforce the policy without requiring a separate policy file
at execution time.

### 7. No executable project policy code

Policy files are declarative JSON/TOML only. The spec explicitly prohibits executable project
policy code in Phase 1. Enforce this: if the policy parser encounters a `code` or `script` key
it does not recognise, emit `E9003` (schema validation error) rather than attempting to execute
it.

### 8. Tests

- **Unit tests**: `Policy::from_file` correctly parses valid policies; each `E9xxx` error code
  produced by an appropriate malformed policy fixture.
- **Runtime enforcement integration tests**:
  - `kali run --sandbox fixtures/deny-net.json fixtures/fetch.ts` → exits 1 with `E4004`.
  - `kali run --sandbox fixtures/allow-net.json fixtures/fetch.ts` → exits 0.
  - `kali run --sandbox fixtures/deny-read.json fixtures/readfile.ts` → exits 1 with `E4004`.
- **Static validation integration tests**:
  - `kali check --sandbox fixtures/valid.json fixtures/app.ts` → exits 0.
  - `kali check --sandbox fixtures/bad-schema.json fixtures/app.ts` → exits 1 with `E9003`.
- **Resource limit tests**: programs that exceed `memoryMb` trap with `E4003`.

## Out of Scope

- Inferred-effect-vs-policy rejection on `check`/`build --sandbox` (Phase 2 target).
- Public `kali effects` / `kali package-effects` commands (Phase 2 target).
- Programmable / executable project policy code (explicit non-goal).
- `--api node` sandbox context (Phase 3 target).

## Definition of Done

- [ ] Policy files parse and validate against schema v1.
- [ ] `kali run --sandbox` enforces policy at runtime; violations produce `E4004`.
- [ ] `kali check --sandbox` validates policy schema; exits 0 on valid, 1 on invalid.
- [ ] `kali build --sandbox` embeds policy in artifact as `kali:policy` custom section.
- [ ] All `E9xxx` error cases covered by unit tests.
- [ ] Runtime enforcement integration tests pass.
- [ ] No Stage 1.1–1.8 regressions.
