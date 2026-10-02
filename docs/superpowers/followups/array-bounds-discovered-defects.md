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

`reverse`, `sort` and `copyWithin` silently skip the call (wrong value,
exit 0). `fill` is correct.

## §3. The anonymous lane: `kali check` exits 0 where `kali run` refuses (amendment A-3)

For rows a1 and a3-a5 (`const f = () => [1,2,3]; …`) `kali check` exits 0
while `kali run` refuses with `E5506`. `kali_types` proves a receiver through
the resolver's runtime-array registry, which learns `const a = f()` from
`call_returns_runtime_array`; that keys by the bare callee name, while an
anonymous function's fact is keyed by its `__kali_fn_N` id. This is
`anon-array-return-discovered-defects.md` §9. It is fail-closed at `run`.
Rows d1, d3-d5, n1, n3, n4 and w2 refuse under both commands. Fixing §9's key
would close this gap.

## §4. An out-of-range non-negative index is known only at run time (spec §1.1)

`kali check` exits 0 on `a[5]` over a length-3 runtime array, and `kali run`
traps with `E4000`. The index is not known statically, so no `check` mirror is
possible short of refusing every unproven index (spec §4, "compile-time
only"). This is the one disclosed `check` / `run` gap besides §3.

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
loss: none. No existing cargo test moved and none was re-pinned.

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

## §6. A float index on a plain runtime array produces invalid wasm

`function main(){ const a = new Array(3).fill(4); console.log(a[1.5]); } main();`
(and `a[-1.5]`). node prints `undefined`. `kali check` exits 0. `kali run`
fails with `error[E4201]: failed to compile` ("failed to load WASM module"),
an invalid module, at the baseline `016557d60` (function[41]) and at HEAD
(function[42], shifted by one by the A-1 helper). Pre-existing (spec §1.1
"No float index") and unchanged by this project. It is a fail-closed failure
at `run`, but reported as an internal `E4201`, not an honest refusal, and
`check` does not predict it.

## §7. A parenthesised receiver: `check` exits 0, `run` refuses (A-4)

`function main(){ const a = new Array(3).fill(4); (a).push(4); console.log(a.length); } main();`
`kali check` exits 0, while `kali run` refuses with the E5506 `.push()`
message. The `kali_types` predicate does not unwrap parentheses, so the
mirror (§3.3 of the spec) misses it; codegen's refusal catches it. This is a
`check` / `run` disagreement, fail-closed at `run`. Not fixed.

## §8. A string-literal negative key is refused as a negative index

`a["-1"]` on a plain runtime array folds to the property `"-1"`, so `check`
and `run` both refuse it with the negative-index `E5506`. node prints
`undefined`. This is fail-closed and consistent between the two commands, but
no case pins it.
