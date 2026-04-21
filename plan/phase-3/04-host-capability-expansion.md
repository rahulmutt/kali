# Stage 3.4 — Host Capability Expansion

**Phase:** 3 — Specialization, Optimization & Ecosystem Breadth  
**Spec refs:** [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.8 — Runtime & Execution](../phase-1/08-runtime-execution.md), [1.9 — Sandbox & Policy](../phase-1/09-sandbox-and-policy.md), and preferably [2.2 — Public Effect Reporting](../phase-2/02-public-effect-reporting.md) so public effect/policy behavior stays aligned

## Goal

Implement the Phase-3 host-capability rows that are broader than the Phase-1 standalone baseline
but still earlier than the late host/object-model work:

- broader Deno-oriented filesystem APIs
- mutable environment access
- subprocess spawning
- socket/listener networking

This stage is intentionally separate from Node compatibility. It owns the **capability growth** of
Kali's host/runtime contract, not the `--api node` package-compatibility lane.

## Workable Milestone

- The documented Phase-3 Deno-oriented host subset is available and sandbox-aware.
- Positive spawned-process and thread-budget semantics remain aligned with the maturity matrix.
- Network connect/listen support is mediated through the declared sandbox/resource model rather than
  hidden host escape hatches.
- The package corpus can claim the newly enabled support rungs with evidence tied to these exact
  host capabilities.

## Progress

- `kali_api_deno` now covers the broader Phase-3 filesystem surface with deterministic `open`,
  `create`, `rename`, and `lstat` helpers on top of the existing read/write/stat/remove/mkdir
  support.
- The Deno compatibility tests now exercise file creation, handle reads/writes, renames, and
  `stat`/`lstat` metadata round-trips so the broader filesystem contract stays regression-tested.

## Tasks

### 1. Broader Deno-oriented filesystem surface

Implement the Phase-3 Deno expansion from `specs/11-standard-apis.md`:

- `Deno.open`
- `Deno.create`
- `Deno.mkdir`
- `Deno.remove`
- `Deno.rename`
- `Deno.lstat`
- any sync/async pairing the owning spec expects for the chosen subset

Keep the filesystem effect mapping aligned with the canonical built-in effect names and policy keys.

### 2. Mutable environment access

Open the policy-controlled environment-mutation path:

- `Deno.env.set`
- Node-context mutation through the documented `process.env`-style path where the Phase-3 Node
  subset allows it
- effect reporting / policy comparison via `Process.EnvWrite`
- explicit denial behavior when policy or command context does not permit mutation

### 3. Subprocess spawning

Implement the Phase-3 subprocess lane:

- `Deno.Command`
- the corresponding runtime host-spawn path
- `Process.Spawn` effect mapping
- resource-limit enforcement through `maxSpawnedProcesses`
- `0` vs positive-budget behavior matching the shared feature-gated zero-capable budget rules

This stage must not silently promote arbitrary shelling-out into an always-on capability.

### 4. Socket/listener networking

Open the broader network capability family:

- `Network.Connect`
- `Network.Listen`
- `Deno.serve` / documented listener entrypoints
- policy allowlists and resource caps for connection/listener counts
- deterministic diagnostics for blocked or unavailable networking modes

### 5. Package and command-context handoff

Update package and command evidence for the new host capabilities:

- record which packages move from blocked-at-host-fit to checkable/buildable/executable
- add corpus fixtures that specifically require env mutation, subprocesses, or listener sockets
- keep browser-targeted and still-gated contexts honest
- ensure `run`, `test`, `build`, and `check/build --sandbox` all report the same capability gates

### 6. Tests

- integration tests for each newly opened Deno API
- policy and effect-report tests for `Process.EnvWrite`, `Process.Spawn`, `Network.Connect`, and
  `Network.Listen`
- resource-limit tests for `--max-spawned-processes`
- package-corpus tests demonstrating the exact newly opened support rung
- negative tests proving later host/control APIs (`pid`, `cwd`, `chdir`, `exit`) remain gated

## Out of Scope

- `--api node` package-compatibility work owned by Stage 3.2
- standalone browser runtime/test support owned by Stage 5.2
- process identity/control and working-directory APIs owned by Stage 5.4
- weak/finalization/proxy semantics owned by Stage 5.4

## Status

Planned.
