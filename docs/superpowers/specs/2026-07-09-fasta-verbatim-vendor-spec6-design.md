# fasta Spec 6 — verbatim vendoring + large-N SHA-256 validation

**Status:** design approved 2026-07-09.
**Series:** last-but-one fasta item. Builds on Spec 5
([[kali-fasta-output-argv-spec5]], PR #13, main `ea44016fd`), which shipped the
full 3-section fasta shell byte-for-byte vs node with `n = +process.argv[2]`.

## Why this spec exists (and what changed from the roadmap)

Spec 6 was filed as **validation-only**: "vendor upstream `fasta-node-1`
verbatim + canonical N=25,000,000 SHA-256 two-tier validation." Running the
fully-verbatim upstream end-to-end on a freshly-built `kali` binary showed it is
**not** validation-only — it is two small pieces of real work plus a validation
harness, and canonical N=25M hits a **memory-reclamation wall** that is a
separate, larger lift:

| N | output | result on current `kali` (fuel-raised `--sandbox`) |
|---|--------|-----|
| 1M | 10 MB | ✅ byte-for-byte vs node |
| 2M | 20 MB | ✅ byte-for-byte vs node |
| 3M | 30 MB | ✅ byte-for-byte vs node |
| 5M | ~35 MB emitted | ❌ **E4000** runtime trap (allocation failure) |
| 25M (canonical) | 254 MB | ❌ **E4000** at ~68 MB emitted |

The fasta output loops **leak** their per-line `.join("")` / `substring`
temporaries — there is no per-line arena/region reclamation — so the wasm32 heap
exhausts around N≈4M. Reaching canonical N=25M needs the binary-trees Phase 1
escape-flow + arena-codegen machinery applied to the fasta while-loops
([[kali-binary-trees-phase1]], [[kali-interprocedural-escape-flow]]). That is a
lift comparable to binary-trees, not a validation footnote.

**Scope decision (user, 2026-07-09): split.** This spec (Spec 6) ships the
verbatim vendoring + a large-N SHA-256 tier **below** the leak wall. Canonical
N=25M + reclamation becomes **Spec 7**.

## What "verbatim" means here

The series has consistently targeted one specific `fasta-node-1` variant: the
`console.log`-per-60-column-line output layer (upstream `print` → `console.log`;
**no** byte-buffer writer), with the inline-constant LCG `rand`. Spec 5's
capstone deliberately deviated from the upstream *operator forms* to stay inside
the already-supported surface — it used `x = x + y` / `x = x - y` instead of
`+=`/`-=` and `i = i + 1` instead of `i++`, and split multi-declarator `var`s.
Those forms are numerically identical to upstream but not textually identical.

**Verbatim in this spec = restoring the upstream operator/syntax forms** on that
same targeted source:

- `+=` / `-=` (compound arithmetic assignment)
- `i++` (update expression)
- `var seqi = 0, lenOut = 60;` (multi-declarator — already supported, Spec 4a)
- braceless single-statement `for..in` body (already parses)
- `table[c] += table[prev]` (computed for-in-key object-field compound — already
  supported, Spec 4a)

It does **not** mean introducing a different upstream variant's structure (e.g. a
buffered `process.stdout.write` writer or a `Random`-object generator). The
`rand` shape and program structure are unchanged from the validated series
source.

## The verbatim target program

```js
var last = 42;
function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }
function makeCumulative(table) {
  var prev = null;
  for (var c in table) {
    if (prev) table[c] += table[prev];
    prev = c;
  }
}
function fastaRepeat(n, seq) {
  var seqi = 0, lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi += lenOut;
    } else {
      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));
      seqi = lenOut - (seq.length - seqi);
    }
    n -= lenOut;
  }
}
function fastaRandom(n, table) {
  var line = new Array(60);
  makeCumulative(table);
  while (n > 0) {
    if (n < line.length) line = new Array(n);
    for (var i = 0; i < line.length; i++) {
      var r = rand(1);
      for (var c in table) if (r < table[c]) break;
      line[i] = c;
    }
    console.log(line.join(""));
    n -= line.length;
  }
}
var ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" +
"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA" +
"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT" +
"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA" +
"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG" +
"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC" +
"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAAA";
var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };
var HomoSap = { a: 0.3029549426680, c: 0.1979883004921, g: 0.1975473066391, t: 0.3015094502008 };
var n = +process.argv[2];
console.log(">ONE Homo sapiens alu");
fastaRepeat(2 * n, ALU);
console.log(">TWO IUB ambiguity codes");
fastaRandom(3 * n, IUB);
console.log(">THREE Homo sapiens frequency");
fastaRandom(5 * n, HomoSap);
```

**Empirical gap isolation.** Running this file as-is on `kali` produces exactly
three `E5506` diagnostics — all on the parameter `n` (`n -= lenOut` twice-worth
plus `n -= line.length`). Patching only those three sites to `n = n - …` makes
the file run **byte-for-byte vs node v26.4.0** at N=8. Therefore
**parameter compound-assign is the sole compile gap** for verbatim vendoring.
Everything else in the file (`table[c] += …`, `seqi += lenOut` on a local, `i++`
on a loop var, braceless `for..in`, multi-declarator) already compiles.

---

## Task 1 — Compound-assign on a parameter binding

### Root cause

- `Scope::bind` (`kali_types/src/scope.rs:107`) inserts
  `mutable_bindings[name] = false` for **every** binding.
- `resolve_variable_declaration` (`resolve/mod.rs:753-761`) upgrades `var`/`let`
  (non-`const`) declarators to `true`.
- `bind_function_params` (`context.rs:326`) → `bind_current_scope` →
  `Scope::bind` — params get `false` and are **never** upgraded.
- The fail-closed compound/update/nullish gate calls `binding_is_mutable`
  (`resolve/expression.rs:1683`, and `:1786` for update expressions). A param is
  non-mutable there, so `n -= …` (and `n++`, `n ??= …`) reject `E5506`.

Plain `=` reassignment of a param already works (Spec 5 shipped `n = n -
lenOut`) because simple assignment does not route through this gate, and codegen
already treats params as locals.

### Fix

Mark **named** function params mutable at bind time — correct JS semantics
(parameters are reassignable). Concretely: `bind_function_params` sets
`mutable_bindings[param] = true` for each named param (equivalently, a
param-specific mutability set applied after `bind_current_scope`).

This drops params into the **same codegen local lane** `var` locals already use:
`+=` on a `var` local works, `=` on a param works, and codegen indexes params as
locals — so `param op= rhs` decomposes to `param = (param op rhs)` and hits the
existing repr-specific compound arms (`literal.rs`) with **no new codegen**.

### Fail-open safety (both-sides discipline)

`binding_is_mutable` has exactly three callers; the fix is safe at all three:

1. **`.length`-fold receiver** (`expression.rs:1096`,
   `expression_is_length_fold_receiver`): today a param resolves to `false` here
   via fall-through — params are not in `static_values`, so
   `resolve_static_string_expression(param)` is `None`. After the fix the early
   `if binding_is_mutable(name) return false` fires and yields the **same**
   `false`. **No behavior change.**
2. **Compound-assign gate** (`:1683`): params newly admitted → codegen local
   lane. *Intended change.*
3. **Update-expr gate** (`:1786`): params newly admitted for `param++`/`param--`
   → codegen local lane. Correct JS; not on the fasta path.

The gate is only reached for a **bare-identifier** target — the for-in-key
computed-member compound (`table[c] += …`) is handled by a special-case
*before* the mutability check (`:1662-1668`) and is unaffected. Admitting a param
therefore defers to the **same** repr-specific handling that already governs an
equivalent `var` local: no new fail-open surface.

### Reject-safety test matrix (proves no fail-open on non-i64 params)

- `n -= v` on an **i64 param** in a loop → compiles, runs correct (the fasta
  case; pinned via Task 2).
- `s += x` on a **string param** → the string compound arm (`literal.rs:620`)
  handles `+=` or rejects the other ops fail-closed, exactly as for a string
  `var` local.
- `arr += x` / `obj += x` on an **array/object param** → rejects fail-closed
  (numeric/string arms reject non-scalar repr).
- **for-in / for-in-key** behavior unchanged (keyed on shape tables, not on
  mutability): the Spec 4a/5 for-in pins stay green.
- `param++` / `param--` admitted but not exercised by fasta — covered by a
  reject-or-correct unit check, not a runtime pin.

Scope: **named** params only. Destructuring/default params are out of scope;
marking such a name mutable only defers to codegen's existing rejects, so it
cannot fail-open.

---

## Task 2 — Vendor verbatim + two-tier SHA-256 validation

### Fixture (matches mandelbrot/binary-trees convention)

Under `crates/kali_cli/tests/fixtures/benchmarks/`:

- `fasta-benchmark-v1.ts` — the verbatim target program above, byte-exact.
- `fasta-benchmark-v1.json` — benchmark metadata (`benchmark: "fasta"`,
  `version: 1`, `sourceFile`, matching the sibling fixtures' schema; pinned by a
  `schema_docs`-style meta test).
- `fasta-benchmark-v1.policy.json` — a `--sandbox` policy raising the CPU-fuel
  budget past the default runaway guard (the same pattern mandelbrot n=200 and
  binary-trees N=21 use; `maxCpuTimeMs` generous, `maxMemoryMB: null`, effects
  all denied except `console`).

### Test — `crates/kali_cli/tests/clbg_fasta_runtime.rs`, two tiers

Mirrors binary-trees' small-golden + canonical structure.

- **Tier 1 — small-N golden (N=8).** Run `kali run --api node <fixture> -- 8`,
  assert stdout **byte-for-byte** against the inline expected. Reuses the Spec 5
  capstone golden, now asserted against the *verbatim* fixture:
  ```
  >ONE Homo sapiens alu
  GGCCGGGCGCGGTGGC
  >TWO IUB ambiguity codes
  cttBtatcatatgctaKggNcata
  >THREE Homo sapiens frequency
  aatagctaaatcttgtgcttcgttagaagtctcgactacg
  ```
- **Tier 2 — large-N SHA-256 (N=2,000,000).** Run under the sandbox policy,
  assert `sha256(stdout)` equals the embedded node reference:
  ```
  a6b7308b4f7ea37cbaef69bdb05448c8623549978dc24d30e4e197026c1e073a
  ```
  (`sha2::{Digest, Sha256}` is already a dev-dependency — used by the
  binary-trees and mandelbrot runtime tests.) The reference is derived by
  running the fixture under node v26.4.0 at N=2,000,000; the run is fully
  deterministic (seed fixed at 42).

**Why N=2,000,000 (not 25M, not 3M).** The leak wall is at N≈4M (N=3M completes,
N=5M traps E4000). N=2M (20 MB output, ~1.5s wall-clock, verified matching) sits
with ~40% byte-headroom below the wall — a stable interim pin that still makes a
decisive scale jump from N=8, without pinning right at the ceiling where an
unrelated per-line allocation change would flip it red. The test carries a
`log`/comment recording the measured ceiling and that canonical N=25M awaits
Spec 7. Wall-clock is well inside convention (binary-trees N=21 runs ~8s and is
not `#[ignore]`d), so Tier 2 runs in the normal suite, not gated.

The node reference for the eventual Spec 7 canonical pin is already captured:
`sha256` at N=25,000,000 =
`6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee`.

---

## Scope boundaries (explicitly deferred to Spec 7)

- **Per-line arena/region reclamation** on the fasta output loops — the E4000
  wall at N≥~4M. This is the substantive Spec 7 work.
- **Canonical N=25,000,000 SHA-256** (reference hash captured above).
- `param++` / `param--` are *admitted* by Task 1 but not runtime-pinned (fasta
  never uses them); the reject-safety matrix is their only coverage.
- No change to `rand` structure, output layer, or any Spec 1–5 surface beyond
  the parameter-mutability flag.

## Testing / acceptance

- Task 1: the reject-safety matrix compiles/rejects as specified; the verbatim
  fixture compiles (three `E5506`s gone).
- Task 2: both tiers green.
- Full 5-crate verification gate (lexer/common/types/codegen/cli) +
  `cargo fmt --check` clean, per [[kali-repo-verification-env]]. All existing
  CLBG fixtures (nbody, mandelbrot n=200, binary-trees N=21, spectral-norm) stay
  byte-identical — Task 1 is additive (a previously-rejected form now compiles),
  so no existing behavior changes.
- Controller discipline ([[kali-fasta-output-argv-spec5]] headline lesson):
  re-run every reproducer on a **freshly-built** binary; trust behavior, not fix
  reports.
