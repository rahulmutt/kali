# Defects the array-bounds project measured and did NOT fix

**Filed** 2026-10-02 by the **array-bounds** project
(`docs/superpowers/specs/2026-10-02-array-bounds-design.md`), on the
convention `anon-array-return-discovered-defects.md` and its predecessors use:
a project that measures more than it fixes writes down what it left, so the
silence is not read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** `ba65b78bd` (branch `array-bounds`), on `target/debug/kali`
built from that commit (`cargo build -p kali_cli`, the `dev` profile). Probe
rows come from `tools/array-return-probes/run.sh` at that commit. A probe's
baseline column is `tools/array-return-probes/baseline-bounds.tsv`, measured
at the spec's baseline `016557d60`. Rows that are not probes were run by hand
as `node P.js` against `kali run P.js` and `kali check P.js`, with the program
text given in full. Baseline cells for those rows were measured on a `kali`
built from `016557d60`, except where a row says otherwise.

**Register:** no entry of `kali-silent-miscompile-register.md` moved lane, so
the register and `blast-radius-ranking.md` are not edited. R-21's `r21o`
oracle cases (`crates/kali_cli/tests/cases/oracle/tier2.toml`) were re-run
(`cargo test -p kali_cli --test cases -- oracle/`, 177 passed) and both are
still `silent`: they read a literal array, which takes the static-fold lane,
not the plain runtime-array lane this project guards. The probe
`r21_oob_control` (a `new Array(3).fill(4)` read of `a[5]`, the same program
as `bounds_n2`) is a different program and now traps (§5).

---

## §1. The growable lane's out-of-range read is still silent

Probe `bounds_g1`: `function main(){ const a = [1,2,3]; a.push(4); console.log(a[9]); } main();`
node prints `undefined`; kali prints `0` at exit 0, before and after this
project. The site is `emit_growable_index_read`
(`crates/kali_codegen/src/emit/growable.rs:470`, whose own doc comment says an
out-of-range read is unguarded). Its header layout differs from the plain
`[len][elem…]` array, so `__array_elem_addr` does not apply to it. Not fixed
(spec §1.1).

The control `bounds_ok_growable_push` pins that a growable `push` still
works. It is the two-separate-logs variant: a multi-argument `console.log`
that reads a growable array is a pre-existing `E5506`, independent of this
project.

## §2. In-place array mutators other than the five refused ones skip the call silently

Measured in Task 5 on a `new Array(3).fill(4)` binding inside
`function main(){…} main();`. `reverse`, `sort` and `copyWithin` take the same
terminal fallback `push` took (`emit/call.rs`): the call is dropped and the
array is unchanged, at exit 0. `fill` is correct. This project refuses only
`push`, `pop`, `shift`, `unshift`, `splice` and a `.length` write.

```
### rev: a[0] = 1; a.reverse(); console.log(a[0]);
node: 4
kali: 1
exit 0
### sort: a[0] = 9; a.sort(); console.log(a[0]);
node: 4
kali: 9
exit 0
### cw: a[2] = 7; a.copyWithin(0, 2); console.log(a[0]);
node: 7
kali: 4
exit 0
### fill: a.fill(5); console.log(a[1]);
node: 5
kali: 5
exit 0
```

**FIXED (fail-closed)** at `d46ee0dc7a2819b49347fc557f5fbc468226f344` by the literal-array-mutators
project (`docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`):
the call now refuses with `E5506` under `check` and `run`. It does not run;
node's output is still not produced. (`reverse`, `sort` and `copyWithin` on a plain runtime array refuse; `fill` stays allowed.)

## §3. The anonymous lane: `kali check` exits 0 where `kali run` refuses (amendment A-3)

For rows a1 and a3-a5 (`const f = () => [1,2,3]; …`) `kali check` exits 0
while `kali run` refuses with `E5506`. `kali_types` proves a receiver through
the resolver's runtime-array registry, which learns `const a = f()` from
`call_returns_runtime_array`; that keys by the bare callee name, while an
anonymous function's fact is keyed by its `__kali_fn_N` id. This is
`anon-array-return-discovered-defects.md` §9. It is fail-closed at `run`.
Rows d1, d3-d5, n1, n3, n4, n7, n8, n9 and w2 refuse under both commands. Fixing §9's key
would close this gap.

## §4. An out-of-range non-negative index is known only at run time (spec §1.1)

`kali check` exits 0 on `a[5]` over a length-3 runtime array, and `kali run`
traps with `E4000`. The index is not known statically, so no `check` mirror is
possible short of refusing every unproven index (spec §4, "compile-time
only"). It is a disclosed `check` / `run` gap, as are §3 and §6 (float index: `check` exits 0, `run` fails with `E4201`).

## §5. Capability loss and the probe diff, recomputed at the end of the branch

Recomputed at `ba65b78bd`:

```bash
tools/array-return-probes/run.sh "$SCRATCH/probes-final.tsv"
diff <(cut -f1,2 tools/array-return-probes/baseline-bounds.tsv | sort) <(cut -f1,2 "$SCRATCH/probes-final.tsv" | sort)
```

```
23,32c23,32
< bounds_a1	SILENT
< bounds_a2	SILENT
< bounds_a3	SILENT
< bounds_a4	SILENT
< bounds_a5	SILENT
< bounds_d1	SILENT
< bounds_d2	SILENT
< bounds_d3	SILENT
< bounds_d4	SILENT
< bounds_d5	SILENT
---
> bounds_a1	REFUSES
> bounds_a2	TRAPS
> bounds_a3	REFUSES
> bounds_a4	REFUSES
> bounds_a5	REFUSES
> bounds_d1	REFUSES
> bounds_d2	TRAPS
> bounds_d3	REFUSES
> bounds_d4	REFUSES
> bounds_d5	REFUSES
34,42c34,42
< bounds_n1	SILENT
< bounds_n2	SILENT
< bounds_n3	SILENT
< bounds_n4	SILENT
< bounds_n5	SILENT
< bounds_n6	SILENT
< bounds_n7	SILENT
< bounds_n8	SILENT
< bounds_n9	SILENT
---
> bounds_n1	REFUSES
> bounds_n2	TRAPS
> bounds_n3	REFUSES
> bounds_n4	REFUSES
> bounds_n5	TRAPS
> bounds_n6	TRAPS
> bounds_n7	REFUSES
> bounds_n8	REFUSES
> bounds_n9	REFUSES
48,50c48,50
< bounds_p1	SILENT
< bounds_w1	SILENT
< bounds_w2	SILENT
---
> bounds_p1	TRAPS
> bounds_w1	TRAPS
> bounds_w2	REFUSES
72c72
< r21_oob_control	SILENT
---
> r21_oob_control	TRAPS
```

23 probe rows moved: 22 `bounds_*` rows and `r21_oob_control`, all
SILENT to REFUSES or TRAPS, all wanted. No `bounds_ok_*` row moved. Capability
loss: not none, see "Measured capability loss" below. No existing cargo test
moved and none was re-pinned.

Task 5's triage table, verbatim (name | before | after | class):

```
bounds_a1 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_a2 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_a3 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_a4 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_a5 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_d1 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_d2 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_d3 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_d4 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_d5 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n1 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n2 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n3 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n4 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n5 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n6 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n7 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n8 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_n9 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_p1 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_w1 | SILENT | TRAPS | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
bounds_w2 | SILENT | REFUSES | wanted (pinned a silent value at 016557d60; OOB read/write or length-changing call whose result is observed)
r21_oob_control | SILENT | TRAPS | wanted (non-bounds_ probe; `new Array(3).fill(4); a[5]` is an out-of-bounds read, node prints undefined; silent 0 at 016557d60). Not the r21o static-fold lane.
bounds_g1 | SILENT | SILENT | unchanged (growable lane, out of scope)
bounds_ok_* | CORRECT | CORRECT | unchanged
```

`r21_oob_control` is cited in `array-return-discovered-defects.md` §6, which
now says it traps.

### Measured capability loss (accepted, fail-closed)

A `push` on a plain runtime array whose growth is never observed, or is never
reached, ran correctly at the baseline and is now an `E5506` refusal. Spec §5.1
reserves this trade-off for the human partner. **Decision (human partner,
2026-10-02): accept the refusals as fail-closed.** No refusal was narrowed or
removed. Rationale: the project's chosen scope was fail-closed (spec §0,
option A). A fixed-length array cannot perform a length change, so refusing
every such call is the simple sound rule. The refused programs never observe
the array after the call, or never reach it, whereas real code that pushes
almost always reads the array afterwards, and those are exactly the programs
that printed wrong values at baseline. Narrowing to "refuse only when the array
is later observed" would need whole-program read analysis (aliases, closures,
calls), and a miss in that analysis would bring back a silent miscompile, which
is worse than the refusal. The real fix is a feature, see §12. Each program below was run as
`node P.js` and as `kali run P.js` / `kali check P.js` on a `kali` built from
`016557d60` and on the HEAD build (`kali check` and `kali run` agree at HEAD in
every row).

| program | node | `016557d60` | HEAD |
|---|---|---|---|
| `function f(){return [1,2,3];} function main(){ const a=f(); a.push(4); console.log(a[0]); } main();` | `1` | `1`, exit 0 | E5506, exit 1 |
| `function main(){ const a = new Array(3).fill(4); if (a.length > 5) { a.push(1); } console.log(a[0]); } main();` (the `push` never runs) | `4` | `4`, exit 0 | E5506, exit 1 |
| `function f(){ return [1,2,3]; } function g(){ const a = f(); a.push(4); } console.log(f()[0]);` (`g` is never called) | `1` | `1`, exit 0 | E5506, exit 1 |
| `function f(){ return [1,2,3]; } function main(){ const a = f(); a.push(4); console.log(a[0]); } main();` (`unobserved_push`; same shape as the first row, the push is not observed) | `1` | `1`, exit 0 | E5506, exit 1 |

The refusal is static, so it fires whether or not the `push` executes and
whether or not the new length is read.

## §6. A float index on a plain runtime array produces invalid wasm

`function main(){ const a = new Array(3).fill(4); console.log(a[1.5]); } main();`
(and `a[-1.5]`). node prints `undefined`. `kali check` exits 0. `kali run`
fails with `error[E4201]: failed to compile` ("failed to load WASM module"),
an invalid module, at the baseline `016557d60` (function[41]) and at HEAD
(function[42], shifted by one by the A-1 helper). Pre-existing (spec §1.1
"No float index") and unchanged by this project. It is a fail-closed failure
at `run`, but reported as an internal `E4201`, not an honest refusal, and
`check` does not predict it.

## §7. Wrapped receivers: `check` exited 0, `run` refused (A-4), now closed

`(a).push(4)`, `a?.push(1)`, `a["push"](1)` and `(a as number[]).push(1)` on a
plain runtime array passed `kali check` (exit 0) while `kali run` refused with
the E5506 `.push()` message; `(a).length = 1`, `(a)[-1]` and
`(a as number[])[-1]` disagreed the same way. Cause: the `kali_types` predicate
`is_plain_runtime_array_receiver` did not unwrap transparent wrappers, and the
mutator gate required a non-computed callee, which excluded a string-literal
method key. Fixed: the predicate unwraps through the resolver's existing
`unwrap_transparent` (parentheses, `as`, `satisfies`, optional chain), and the
mutator gate no longer excludes a literal method key. `check` and `run` now
both refuse every shape above. A growable receiver in the same wrappers is not
caught by this gate, but the growable lane's own `E5506` already refuses
`a?.push` and `a["push"]`. `a!.push(1)` is `E3100` under both commands, a separate pre-existing
diagnostic. No wrapped shape is known to disagree.

## §8. A string-literal negative key is refused as a negative index

`a["-1"]` on a plain runtime array folds to the property `"-1"`, so `check`
and `run` both refuse it with the negative-index `E5506`. node prints
`undefined`. This is fail-closed and consistent between the two commands, but
no case pins it.

## §9. The literal-array lane still silently no-ops mutators (out of scope, filed)

A literal array (`[1,2,3]`) takes the static-fold lane, not the plain
runtime-array lane this project guards. Its mutators and `.length` write are
silently skipped at exit 0, identically at `016557d60` and at HEAD (measured on
both builds):

| program | node | `016557d60` | HEAD |
|---|---|---|---|
| `function main(){ const a=[1,2,3]; a.pop(); console.log(a.length);} main();` | `2` | `3` | `3` |
| `const a=[1,2,3]; a.push(4); console.log(a.length);` | `4` | `3` | `3` |
| `const a=[1,2,3]; a.length = 1; console.log(a.length);` | `1` | `3` | `3` |

`kali check` exits 0 on all three. Not fixed.

**FIXED (fail-closed)** at `d46ee0dc7a2819b49347fc557f5fbc468226f344` by the literal-array-mutators
project (`docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`):
the call now refuses with `E5506` under `check` and `run`. It does not run;
node's output is still not produced. (An alias or parameter receiver is refused by `run` only; see `literal-array-mutators-discovered-defects.md` §1.)

## §10. Write-trap ordering: `a[5] = v()` traps before `v` runs

`function v(){ console.log("side"); return 9; } function main(){ const a = new Array(3).fill(4); a[5] = v(); console.log(a[0]); } main();`
node prints `side` then `4`. At `016557d60` kali printed `side` then `4` too
(the out-of-range store was silently accepted). At HEAD kali traps with the
`E4000` out-of-bounds message at exit 1 and prints nothing, because the store
computes and checks the element address before evaluating the right-hand side.
The address-first order predates the branch; the baseline simply never trapped,
so the order was unobservable. JS evaluates the right-hand side first, so the
side effect is lost before the trap. Not fixed.

## §11. A shadowing inner binding is confused with the outer array

Both programs are pre-existing miscompiles of an inner `const a` that shadows an
outer `a`; the array facts are keyed by name.

* `shadow.js`: `function f(){ return [1,2,3]; } function main(){ const a = f(); { const a = []; a.push(5); console.log(a.length); } console.log(a.length); } main();`
  node prints `1` then `3`; kali prints `0` then `0` at `016557d60` and at HEAD,
  exit 0 (`check` exits 0).
* `shadow_obj.js`: `function main(){ const a = new Array(3).fill(4); { const a = [9]; a.push(1); console.log(a.length); } console.log(a[0]); } main();`
  node prints `2` then `4`. `016557d60` printed `0` then `0` (wrong, exit 0).
  HEAD refuses the inner growable `push` with E5506 under `check` and `run`
  (the outer name is a plain runtime array), so a wrong answer became a refusal.

Not fixed.

## §12. Follow-up feature: growable arrays from array-returning functions

Not started. This is the real fix for the §5 capability loss, and for `push` on
a call-bound array (`const a = f()`) generally: let array-returning functions
produce growable arrays, the way an in-function array literal already does. It
would turn the following rows into node-correct output instead of a refusal:
d3 and d4, and a3 and a4 once the anonymous-lane gap (§3,
`anon-array-return-discovered-defects.md` §9) is also closed. Until it is
built, the §5 refusals stand as accepted (fail-closed).

## §13. `a.push?.(1)` (optional call) on a plain runtime array is silent

`function main(){ const a = new Array(3).fill(4); a.push?.(1); console.log(a.length); } main();`

Measured on the HEAD build: `node` prints `4`; `kali check` exits 0; `kali run`
prints `3` at exit 0. A re-reviewer measured the same at baseline `016557d60`,
so this is not a regression. Neither gate sees an optional call, so the
`push` is neither refused by `check` nor by `run`, and the length change is
silently skipped. Not fixed.

**FIXED (fail-closed)** at `d46ee0dc7a2819b49347fc557f5fbc468226f344` by the literal-array-mutators
project (`docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`):
the call now refuses with `E5506` under `check` and `run`. It does not run;
node's output is still not produced. (This holds for the call `a.push?.(1)`; `(a.pop)?.()` is not covered, see `literal-array-mutators-discovered-defects.md` §8.)
