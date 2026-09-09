# The optimizer inlines an ALLOCATING array initializer, so `--release` loses the array identity `--fast` keeps

**Filed** 2026-09-09, at `71b5f42f6c`, by the **computed-member-static-name**
project (`docs/superpowers/sdd/2026-09-08-computed-member-static-name/`).
**This is the most consequential thing that project found, it is wider than that
project's surface, and it was not caused by it** — the project made it VISIBLE,
by turning a silent wrong value into a refusal that a test could see.

**Oracle:** `node v26.8.1`. **Baseline:** `dc19c3a040`. Controller ruling
**R22**.

## 1. The claim, in one sentence

At `--release` and `--release-advanced`, the optimizer inlines a
`new Array(n).fill(v)` declarator initializer **as if it were a pure constant**,
so the array a later read indexes is no longer the same array the declarator
allocated. At `--fast` it is. Every consequence below follows from that one
difference.

## 2. What is measured, and by whom

**Established by the controller, by dumping LIR on both sides of the
optimizer** (this is the part that identifies the mechanism, and it is recorded
here because the dump is not reproducible from a committed artefact):

* At `--fast`, the read base of `u[i]` is the **named binding `u`**.
* At `--release`, it is a **text-less wrapper around the same node id as the
  declarator's `new Array(n).fill(v)` initializer**. There is no shared array
  left to name, and every refused read carries `base_text=None,
  target_name=None`.

**Also established by the controller**, by reinstating the pre-project name
fabrication behind a temporary env gate and running the result — i.e. by
emulating what the release tiers had been emitting BEFORE this project:

| program | pre-project kali at `--release` | node |
|---|---|---|
| `const u = new Array(4).fill(7); let i = 0; u[i]` | `0` | `7` |
| an 11-line repro of the same shape | `0` | `1` |
| the array-parameter form | `0` | `8` |
| `spectral-norm-benchmark-v1.ts` | **invalid wasm** | `1.274219991` |
| the same spectral-norm at `--fast` (control) | `1.274219991` | `1.274219991` |

**Re-measured for this document at `71b5f42f6c`**, on
`.cache/cargo-target/debug/kali`, in a scratch directory:

| fixture / program | `--fast` | `--release` | `--release-advanced` |
|---|---|---|---|
| `spectral-norm-benchmark-v1.ts` | builds; `kali run` prints `1.274219991` | **exit 1**, 16 × `E5506` computed-member, 6 × `E3100` | **exit 1**, 12 × `E5506`, 5 × `E3100` |
| `nbody-benchmark-v1.ts` | builds; `kali run` prints `-0.169075164` / `-0.169087605` | **exit 1**, 6 × `E5506` | **exit 1**, 6 × `E5506` |
| `fannkuch-redux-benchmark-v1.ts` | builds; `kali run` prints `228` / `Pfannkuchen(7) = 16` | **exit 1**, 20 × `E5506`, 2 × `E3100` | **exit 1**, 10 × `E5506`, 1 × `E3100` |
| `const u = new Array(4).fill(7); let i = 0; console.log(u[i]);` | builds | **exit 1**, 1 × `E5506` | — |
| `const u = new Array(4).fill(7); console.log(u[0]);` (literal index) | builds | **builds, no diagnostic** | — |

**The fannkuch row was added 2026-09-09 at `6b59ddeef9`**, in the fix round that
unblocked PR #36, and it is a THIRD Benchmarks Game fixture in exactly this
shape: its three allocations are `const perm = new Array(n);`,
`const perm1 = new Array(n);` and `const count = new Array(n);` (lines 7-9), and
its release tiers lower them through the same zero placeholder. **It was not
found by the sweep that re-pinned the other two**, and the reason matters more
than the finding: a stale pre-project wasm artifact in the gitignored on-disk
incremental cache (`crates/kali_cli/tests/fixtures/.kali-cache/incremental/`,
dated 2026-07-16) short-circuited `compile_source_file` before codegen, so
fannkuch's three-mode build test read cached bytes and passed in ~0.00s on the
machine doing the sweeping. CI runners are cold and compiled for real, which is
where it surfaced. So **the sweep of this defect's fixtures was not exhaustive,
and a local green on this repository is only as trustworthy as its cache** --
filed as §6 of `computed-member-static-name-discovered-defects.md`.

All three `--fast` outputs are byte-identical to node. The `E5506` counts are an
artefact of how far the optimizer got before refusing and are deliberately not
asserted by any test; only the refusal and its code are.

## 3. The part that is still SILENT, and is the reason this is filed

The six `E3100` warnings spectral-norm emits at `--release` are the important
reading, not the `E5506`s:

```
4 × warning[E3100]: undefined call target 'Array' reached codegen and was lowered through a zero placeholder compatibility fallback
2 × warning[E3100]: undefined identifier 'n' reached codegen and was lowered through a zero placeholder compatibility fallback
```

The fixture's three allocations are `const u = new Array(n).fill(1);`,
`const v = new Array(n);` and `const w = new Array(n);` (lines 31-33). At
`--release` the allocation itself is lowered through a **zero placeholder**. So:

* A **non-literal** index read off such an allocation reaches this project's
  computed-member gateway and is REFUSED. That is loud, and it is what took the
  benchmark fixtures out of the measured list.
* A **literal** index read off the same allocation — `u[0]` — never reaches that
  gateway, because a literal index has a static name and takes the ordinary
  member lane. **It is miscompiled at `--release` in exactly the same way, and it
  stays SILENT.** The last row of the table above is the demonstration: the same
  allocation, the same tier, no diagnostic at all.

**So closing the loud half did not close the defect. It closed the half that
happened to route through a new gate.** Nothing in this repository currently
pins the silent half.

## 4. Why nobody noticed for as long as they didn't

`optimization_benchmark_suite_tracks_compile_time_size_and_speed`
(`crates/kali_cli/tests/runtime_smoke/misc.rs`) is green on a **build**, not on a
run: `assert_optimization_benchmark_fixture` counts instructions, adds and
tag-boxing ops in the emitted `.wasm` and **never executes the module**. A build
that produced zeros — or, for spectral-norm, an unloadable module — measured as
a passing benchmark. That is the observation worth carrying out of this file
even if the mechanism is fixed tomorrow: **a benchmark harness that never runs
the artefact is not a correctness gate, and this one was being read as one.**

Since `820753ee9a` the two fixtures sit in their own block in that test, which
pins the `--fast` build AND RUNS it byte-identically against node, and pins both
release tiers refusing. The other 59 fixtures are untouched, no exclusion list
grew, and the helper is not weakened.

## 5. Ruling R22: the refusal stands, and nothing was restored

The controller ruled that the release-tier refusal is **honest and correct**, and
that the two fixtures are **re-pinned, not restored**. Reasons, recorded so the
decision is arguable:

* The pre-project release tiers were emitting zeros and, for spectral-norm, an
  invalid module. Converting that into a visible refusal is what this project
  exists to do.
* The fixtures are the upstream Benchmarks Game programs; rewriting them to dodge
  the shape would change the workload and make the benchmark meaningless.
* The two `matches!` exclusion arms inside
  `assert_optimization_benchmark_fixture` still name `"spectral-norm"` and
  `"nbody"`. Those arms are now unreached and were **left in place deliberately**,
  so that restoring either fixture to the measured list restores a working
  configuration rather than a differently-broken one.

**A second class was identified and deliberately NOT admitted**: specialized-clone
array parameters (`in_array_bindings=false` at the gateway). It is addressable —
the recognizer could be taught the shape — but the pre-project binary printed
`0` for it too, so admitting it would be **widening the admitted set rather than
restoring a lane that used to work**, which is out of scope for a project whose
contract is that a folded bracket access behaves exactly as its dot spelling.

## 6. What would close it

Teach the optimizer that a `new Array(n)` / `new Array(n).fill(v)` initializer is
**allocating, not pure**, so inlining it duplicates an allocation rather than
copying a constant — or, equivalently, refuse to inline a declarator initializer
whose value has identity. That is a change in `kali_optimize`, not in the
computed-member gateway, and it is what would let `"spectral-norm"` and
`"nbody"` go back into the measured benchmark list, and fannkuch's three-mode
build assertion back to three.

Until then, two things are true and should be said together: `--release` REFUSES
some programs `--fast` compiles correctly, and `--release` SILENTLY MISCOMPILES
others that `--fast` compiles correctly. The second is the dangerous one.

## 7. Suggested home

**§2 of the register, silent**, for the literal-index half of §3 — it is a
silent wrong value at exit 0 on a shipped build tier. The refusing half belongs
in §7 (*fail-loudly-but-wrong*) or in the phase plan as a release-tier capability
gap, not in §2. Whoever files it should note that the register's oracle harness
measures the DEFAULT tier only, so a §0.2 row for this entry cannot be backed by
an oracle case as the harness stands today — which is itself a gap in the
instrument worth recording.
