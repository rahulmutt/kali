# Stage C triage — entry baseline + four capture-miscompile probes

> Investigation-first stage entry. This doc records the frozen failure
> baseline that gates every Stage C task and the four baseline
> capture-miscompile probes named in the plan.
> Plan: `docs/superpowers/plans/2026-07-16-stageC-closures.md`
> (env-pointer closures C1-C4).
> Design: `docs/superpowers/specs/2026-07-16-stageC-closures-design.md`.

## 1. Branch / baseline commit

- Branch: `soundness-batch1-pra`.
- HEAD at triage time: `cf56ee382` ("docs(soundness): stageC implementation
  plan — env-pointer closures C1-C4 (8 tasks) [stageC]").
- `192984c39` confirmed an ancestor of HEAD (`git merge-base --is-ancestor
  192984c39 HEAD` → `OK`).

## 2. Frozen failure baseline — 731, zero drift

Built `kali_cli` fresh (`cargo build -p kali_cli`, clean success, no product
code changes). Then ran two independent full-workspace enumerations, each
detached and polled to completion:

```
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > stageC-pre-runN.txt
```

- `$SCRATCH/stageC-pre-run1.txt`: **731** failing test names.
- `$SCRATCH/stageC-pre-run2.txt`: **731** failing test names.
- `diff stageC-pre-run1.txt stageC-pre-run2.txt` → **empty** (zero drift).
- `sort -u stageC-pre-run1.txt stageC-pre-run2.txt > $SCRATCH/stageC-pre.txt`
  → **731** lines (same set both runs — union equals either run).

**Canonical entry baseline: `$SCRATCH/stageC-pre.txt`, 731 entries.** This is
the set every later Stage C gate diffs against (`comm -13` for newly-red,
`comm -23` for drain).

## 3. Four baseline capture-miscompile probes

Each probe was written to a standalone file under `$SCRATCH`, executed via
`target/debug/kali run <file>` on the freshly-built binary, and cross-checked
against `node` (v26.5.0) on the same source. Values below are the actual
observed stdout/stderr/exit code — not copied from the spec.

### Probe 1 — synchronous write to enclosing scalar (`c += 1` from nested fn)

```js
function o(){ let c=0; function inc(){ c+=1; } inc(); inc(); console.log(c); } o();
```

- **node**: stdout `2`, exit 0.
- **kali**: fails closed, exit 1:
  ```
  error[E5506]: compound assignment lowering is unavailable for binding 'c' unless it is a mutable local binding; use a mutable variable or the later compatibility path
  ```
- Verdict: **fail-closed E5506** on the write path, as expected — not a silent
  miscompile. This is the write-path fail site C1 must close.

### Probe 2 — synchronous read of enclosing scalar (`return c` from nested fn)

```js
function o(){ let c=7; function rd(){ return c; } console.log(rd()); } o();
```

- **node**: stdout `7`, exit 0.
- **kali**: stdout `0`, exit 0.
- Verdict: **silent miscompile** — kali runs to completion and prints the
  wrong value (`0` instead of `7`) with no diagnostic. This is the read-path
  fail site C1 must close.

### Probe 3 — heap read via nested fn (`obj.n` captured, not the scalar itself)

```js
function o(){ let obj={n:1}; function rd(){ return obj.n; } console.log(rd()); } o();
```

- **node**: stdout `1`, exit 0.
- **kali**: stdout `0`, exit 0.
- Verdict: **silent miscompile**, same shape as probe 2 but through a heap
  object field read instead of a scalar. Confirms the read-path gap covers
  heap-object capture too, not only scalars.

### Probe 4 — module-scope `queueMicrotask` capture (control — must stay working)

```js
let count=0; queueMicrotask(function(){ count+=1; }); console.log("sync="+count);
```

- **node**: stdout `sync=0`, exit 0 (the microtask callback runs later and
  drains after the synchronous log line, so no further stdout is produced by
  this program before/at the point `console.log` runs).
- **kali**: stdout `sync=0`, exit 0.
- Verdict: **matches** — this module-scope path already works (module
  globals, no function-nesting env needed) and must **not** regress as Stage C
  lands nested-function env-pointer closures.

## 4. Summary

| probe | shape | kali | node | class |
|---|---|---|---|---|
| 1 | write to enclosing scalar, sync call | E5506, exit 1 | `2` | fail-closed (write path) |
| 2 | read of enclosing scalar, sync call | `0`, exit 0 | `7` | **silent miscompile** (read path) |
| 3 | read of enclosing heap-object field, sync call | `0`, exit 0 | `1` | **silent miscompile** (heap read path) |
| 4 | module-scope capture via `queueMicrotask` | `sync=0`, exit 0 | `sync=0`, exit 0 | already correct — control, must stay green |

Probes 1-3 are the exact shapes Stage C (env-pointer closures, Phase C1
onward) targets: the write path currently fails closed (safe, but blocks
valid programs); the read paths (scalar and heap-object) currently miscompile
silently and must become either correct or fail-closed as part of the same
fix. Probe 4 isolates the pre-existing module-scope capture path, which must
remain unaffected.
