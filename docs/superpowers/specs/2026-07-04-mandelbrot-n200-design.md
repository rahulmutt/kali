# Mandelbrot at canonical n=200 — design

**Date:** 2026-07-04
**Status:** approved (design), pending implementation plan
**Supersedes fixture size in:** [2026-07-03-mandelbrot-end-to-end-design.md](2026-07-03-mandelbrot-end-to-end-design.md)

## Goal

Grow the vendored CLBG mandelbrot fixture from the fuel-shrunk **n=128** to the
canonical **n=200** (the size every other CLBG entry uses). Grant the extra CPU
budget through a **scoped sandbox policy** so the global runaway/DoS guard stays
tight. Opportunistically reduce codegen fuel as a **non-blocking bonus**.

## Background — why n=128 in the first place

The fixture was deliberately shrunk to n=128 to fit the default ~60M CPU-fuel
budget; n=200 traps. Nothing about the algorithm or the runtime is wrong at
n=200 — it is purely a budget-ceiling question.

## Measured facts (verified, not assumed)

Instrumented `store.get_fuel()` around `_start` and ran both sizes:

| n           | fuel consumed | vs 60M default | output bytes |
|-------------|---------------|----------------|--------------|
| 128 (current) | 26,785,330  | 45%            | 2059         |
| 200 (canonical) | 65,429,267 | **109% — traps** | 5011      |

- Fuel is **exactly linear in pixels**: 65.4M / 26.8M = 2.44 = (200/128)². No
  memory pressure (output is ~5 KB; array of 5011 i64 ≈ 40 KB).
- The overshoot is only **~9%** over the default. This is a "just over an
  arbitrary line" problem, not a "too slow to run" problem.
- The default 60M budget is itself arbitrary: fuel ≠ wall-clock (65M fuel runs
  in well under a second), so the "60_000 ms × 1000" framing in `execute.rs` is
  a loose nominal calibration, not a real 60-second bound.

### Spike results (end-to-end, verified)

- **Header layout is unchanged.** "128" and "200" are both 3 digits, so the
  11-byte P4 header keeps its exact byte positions; only digit *values* change.
  Confirmed output header bytes: `50 34 0a 32 30 30 20 32 30 30 0a` = `P4\n200 200\n`.
- **Scoped policy works.** `kali run --sandbox <policy>` with `maxCpuTimeMs:
  100000` → exit 0, 5011 bytes. Without the policy → traps. n=128 unaffected.
- **Correctness holds.** kali's n=200 output is **byte-identical** to an
  independent Python reference implementing the same CLBG algorithm (same break
  condition, same MSB-first bit packing). Circularity ruled out.
- **Policy validation gotcha.** Unused numeric resource limits must be `null`,
  not `0`, or validation rejects with E5510 ("must be a positive integer when
  present"). `console: true` is required because `Kali.writeStdoutBytes` is
  gated by `HostOperation::Console`.

## Decisions (agreed with user)

1. **Fit honestly.** Ship n=200 via an honest budget grant, and separately chase
   easy codegen wins as a bonus — not by gaming the algorithm to squeak under 60M.
2. **Scoped policy, not global default.** The fuel cap doubles as the
   runaway-program guard. Keep it strict globally; relax it only for this one
   vetted fixture via `--sandbox`.

## Changes

### 1. Fixture source — `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts`
Two edits, structure untouched:
- Header digit bytes: `out[3..6]` `49;50;56` (`128 `) → `50;48;48` (`200 `);
  `out[7..10]` `49;50;56` (`128\n`) → `50;48;48` (`200\n`). (`out[6]`=32 space
  and `out[10]`=10 newline stay.)
- Call site: `mandelbrot(128);` → `mandelbrot(200);`

### 2. New scoped policy — `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.policy.json`
Minimal deny-all-except-console. `maxCpuTimeMs: 100000` (100M fuel ≈ 1.5× the
measured 65.4M — headroom for codegen drift in either direction). Verified shape:

```json
{
  "schemaVersion": 1,
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": null },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": null, "maxActiveTimers": null },
    "eval": false, "random": false, "console": true
  },
  "resources": {
    "maxMemoryMB": 256, "maxCpuTimeMs": 100000,
    "maxOpenFiles": null, "maxSpawnedProcesses": 0, "maxThreads": 0
  }
}
```

### 3. Golden — `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.expected.pbm`
Regenerated at n=200: 5011 bytes (11 header + 200²/8 = 5000). Pinned against an
independent canonical reference (already verified identical). The plan's
controller regenerates the reference fresh and cross-checks before committing.

### 4. Test — `crates/kali_cli/tests/clbg_mandelbrot_runtime.rs`
`mandelbrot_runs_and_matches_canonical_output` gains `--sandbox <policy>` on the
`kali run` invocation (via the `fixture()` helper). Length/byte assertions read
the golden dynamically, so no numeric literals change — only the golden file.

### 5. Metadata — `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.json`
Bump `sourceSha256` to the edited source's digest (the
`mandelbrot_metadata_is_consistent` test enforces this).

### 6. Docs / memory
Update the maturity-spec row (n=128 → n=200, note the scoped policy) and the
`kali-bitwise-and-binary-stdout-lane` memory.

## Bonus: fuel-reduction pass (timeboxed, non-blocking)

~65M fuel over ~1.2M source-ops ≈ ~50 fuel/op — high for single wasm
instructions, hinting at heavy per-op lowering. Candidate causes to investigate:
float boxing, `out[p]` store bounds checks, and the redundant `Tr+Ti` recompute
(fixture lines 22 and 25). Grab only **clean** wins.

**Hard rules:**
- Must NOT block shipping n=200. If nothing clean lands, n=200 ships on the
  policy alone.
- Any optimization must leave the golden **byte-identical** (pure computation).
  Re-verify and record the new fuel number if a win lands.
- Codegen changes ride the full gate (all crates), not just the fixture test.

## Out of scope (surfaced, not fixed)

Unbudgeted n=200 traps as an opaque **E4000** wasm backtrace, not the friendly
**E4003** resource-limit diagnostic. Fuel exhaustion arguably should map to
E4003. Real papercut, but a separate runtime concern — logged as a follow-up,
not folded in here.

## Testing / gate

- `mandelbrot_runs_and_matches_canonical_output` (now with `--sandbox`) and
  `mandelbrot_metadata_is_consistent` carry over.
- Full gate: 5-crate (lexer/common/types/codegen/cli) + `kali_parser` +
  `kali_runtime`, plus `fmt`.
- Integration per convention: push a PR and self-merge.

## Success criteria

1. `kali run --sandbox <policy> mandelbrot-benchmark-v1.ts` exits 0 and emits the
   5011-byte canonical n=200 PBM, byte-identical to the committed golden.
2. The golden was pinned against a freshly regenerated independent reference.
3. n=128-era behavior (other fixtures, default budget) is unchanged.
4. Full gate green; fmt clean.
5. (Bonus) any fuel reduction is recorded and leaves output byte-identical.
