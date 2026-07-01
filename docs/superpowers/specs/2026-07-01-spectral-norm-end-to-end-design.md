# Design — spectral-norm End-to-End (First Floating-Point CLBG Slice)

Date: 2026-07-01
Status: Proposed (brainstorming output; awaiting user review before plan)

## 1. Problem and motivation

The fannkuch-redux slice (design `docs/superpowers/specs/2026-06-30-fannkuch-redux-end-to-end-design.md`,
merged to local `main`) built Kali's **integer imperative core**: real wasm loops, mutable
`i64` locals, function calls/returns/recursion, linear-memory `i64` arrays, and runtime
integer→string + concatenation. That established the first end-to-end-executing adapted
Computer Language Benchmarks Game (CLBG) fixture.

Every remaining CLBG program needs a new major capability beyond that integer core:
floating point (spectral-norm, n-body, mandelbrot), heap objects/GC (binary-trees),
stdin/hashmaps (k-nucleotide, reverse-complement), byte-oriented stdout (mandelbrot, fasta),
arbitrary-precision bignum (pidigits), or regex (regex-redux).

**spectral-norm is the next simplest.** It is the minimal extension of the fannkuch slice:
the same loops, functions, and arrays, changing only the element/value type from `i64` to
`f64` and adding float formatting. It introduces exactly one new value representation (f64)
and its output path — no objects, no GC, no stdin, no byte stdout, no bignum, no regex. It is
the canonical "first floating-point benchmark."

The reason no floating-point program executes today is architectural (confirmed by reading
codegen): the value model is **i64-only**. Every binary op hardcodes `I64*`
(`crates/kali_codegen/src/emit/operators.rs`); `Math.sqrt` has only a compile-time
constant-fold path returning a perfect-square `i64` root
(`crates/kali_codegen/src/intrinsics/math.rs:216`, `.../emit/call.rs:1750`) with no runtime
`f64.sqrt`; arrays store elements with `I64Store`/`I64Load` at `offset 8`
(`crates/kali_codegen/src/emit/literal.rs:287`); there is no runtime float→string. So
"make spectral-norm work" is **introducing a floating-point representation lane** through the
existing pipeline, using one CLBG program as the acceptance test.

## 2. Goal and non-goals

**Goal.** Make `kali run` execute `spectral-norm` correctly end-to-end and produce the exact
canonical CLBG output line for a pinned input `n`, by adding genuine (not pattern-matched)
floating-point lowering: a static int-vs-float representation decision, f64 arithmetic with
int→float promotion, f64-returning functions, f64 arrays with typed load/store, `.length`,
`.fill`, runtime `Math.sqrt`, and runtime float→fixed-decimal formatting for output.

**Non-goals (explicitly deferred):**

- Float→int truncation/conversion. spectral-norm indices and loop counters stay `i64`; a
  float never indexes an array or drives a counter. If such a coercion arises it stays on the
  current gated path (reject rather than silently miscompile), not implemented here.
- `memory.grow` (fixed generous initial pages, as in fannkuch).
- General `Math.*` beyond `sqrt`; growable arrays beyond `new Array(n)` + `.fill` + indexed
  read/write + `.length`; dynamic objects/classes; hash maps; stdin; byte-oriented stdout.
- Full ECMAScript `Number.prototype.toFixed` conformance (rounding-mode edge case — see §6).
- Performance/throughput claims. This slice proves **correctness of execution**, consistent
  with `plan/phase-24/README.md` §24.4 and the fannkuch precedent.

## 3. Target program

A vendored, idiomatic TS port of the published Node.js / JavaScript spectral-norm submission,
normalized to Kali's pipeline (per `specs/16-testing.md:44`): retains upstream CLBG
attribution, respects CLBG license terms, no benchmark-specific intrinsic tuning. `n` is a
compile-time integer literal (pinned `n = 100`, the canonical CLBG size). Expected output is
captured from a reference Node run and pinned in the test.

```ts
// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// spectral-norm — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
function A(i, j) {
  return 1 / ((i + j) * (i + j + 1) / 2 + i + 1);
}
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(i, j) * u[j];
    }
    v[i] = t;
  }
}
function Atu(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(j, i) * u[j];
    }
    v[i] = t;
  }
}
function AtAu(u, v, w) {
  Au(u, w);
  Atu(w, v);
}
function spectralnorm(n) {
  const u = new Array(n).fill(1);
  const v = new Array(n);
  const w = new Array(n);
  for (let i = 0; i < 10; i = i + 1) {
    AtAu(u, v, w);
    AtAu(v, u, w);
  }
  let vBv = 0;
  let vv = 0;
  for (let i = 0; i < n; i = i + 1) {
    vBv = vBv + u[i] * v[i];
    vv = vv + v[i] * v[i];
  }
  return Math.sqrt(vBv / vv);
}
console.log(spectralnorm(100).toFixed(9));
```

Feature surface distilled: f64 `+ - * /`; int→float promotion inside the mixed index
expression `(i + j) * (i + j + 1) / 2 + i + 1`; f64-returning functions (`A`, `spectralnorm`);
f64 arrays (`u`, `v`, `w`) with typed indexed read/write; `.length`; `.fill(1)`; runtime
`Math.sqrt`; `.toFixed(9)`. Integers remain: loop counters `i`, `j`, the `10` iteration count,
`n`, and the array-length header. `u`, `v`, `w` are `const` bindings (no array-reassignment,
so fannkuch follow-up F-1 is not exercised).

## 4. Architecture of the slice

All work flows through the existing pipeline AST → HIR → MIR → LIR → wasm, executed by the
wasmtime host. The value model is unchanged for integers and strings; **f64 slots in as a
second machine representation for `number`, chosen statically**, exactly the type-directed
philosophy fannkuch used for untagged array offsets. No value-model rewrite: fannkuch's i64
encoding is untouched.

### 4.1 The crux — int-vs-float representation inference

TS `number` does not distinguish integer from float, so Kali must decide an i64-vs-f64 machine
representation per value. It cannot be decided locally: `u` is `.fill(1)` (looks integer) yet
read inside `A(i, j) * u[j]` (float), and flows across function boundaries via `Au(u, w)`. The
decision is therefore an **interprocedural, additive union-find inference**:

1. **Nodes.** Every `number`-typed program point gets a representation node: each
   binding/parameter/return, and each array binding's *element* representation.
2. **Equality edges** (union) along: assignments — including flow-aware reassignment, reusing
   the machinery built for string-typedness tracking (`0e9c430e`); `return` value ↔ call-site
   result; argument ↔ parameter; array-element read/write ↔ that array's element repr.
3. **Float seeds.** Mark a node `float` at every float-producing operation: any `/`, a float
   literal, a `Math.sqrt` result, a `.toFixed` receiver.
4. **Solve.** Any node unified with a float seed ⇒ **f64**; every other `number` node defaults
   ⇒ **i64**.

This makes `u`, `v`, `w`, `t`, `vBv`, `vv`, the `A`/`spectralnorm` returns, and the `1/(…)`
subexpression all f64, while `i`, `j`, `n`, `10`, and the length header stay i64. Crucially,
**fannkuch has zero float seeds**, so every fannkuch node defaults to i64 and its lowering is
byte-identical — the slice is provably additive.

**Placement.** The inference is **computed in `kali_types`** (during the existing single
whole-program resolve walk, which already visits every function body and call site) and its
result — a representation table keyed by `(function name, binding name)` plus per-parameter,
return, and array-element reprs — is **threaded to codegen as a side table**, not carried on
IR nodes. Concretely (confirmed by reading the pipeline): the shared `Repr`/`ReprTable` types
live in `kali_common` (already depended on by both `kali_types` and `kali_codegen`, so no new
dependency edge or cycle); `kali_types` returns the table on its `ResolutionResult`; the driver
(`crates/kali_cli/src/build/compile.rs`) carries it out of `analyze_source_file` onto
`AnalyzedSource`, into `CodegenCtx`, and `FunctionEmitter`/`lower.rs` consume a per-function
slice. This mirrors the existing `TargetConfig`/`source_path`-on-`CodegenCtx` pattern and needs
no IR schema change. (Note: codegen today receives *no* type table and re-derives string/array
shape structurally from the LIR; this slice adds the first analysis result plumbed across the
`kali_types` → codegen boundary. Because codegen currently emits every wasm param/result/local
as `i64`, the table must also drive **wasm signature and local-declaration generation**, not
only instruction selection.) Promoting this to a general MIR representation/layout pass
(alongside the existing `ObjectLayout` classification) is an explicit **future follow-up**, not
built here — the bounded slice does not need it.

### 4.2 Codegen lowering (all type-directed off the repr table)

Once the repr table is known, codegen is mechanical. Six pieces, each independently
micro-testable before the full benchmark (fannkuch A–E discipline):

**1. f64 arithmetic & promotion.** Where the repr table says a binary op's result is float,
emit `F64Add/Sub/Mul/Div`; insert `f64.convert_i64_s` on any operand whose node is i64 but the
op is float (covers `(i + j) * (i + j + 1) / 2 + i + 1`, where the `/2` promotes the running
value and subsequent `+ i + 1` adds i64 operands into it). Division `/` on float-repr operands
is `F64Div`; note `/` is always a float seed, so `1 / (…)` is float even with integer operands.
Micro-acceptance: `console.log((1 / 2).toFixed(1))` → `0.5`.

**2. f64 arrays.** `new Array(n)` allocates the same `[len@+0][elems@+8…]` layout via the
existing `__heap` bump path; the **element instruction/width is repr-directed** — float arrays
use `F64Store`/`F64Load` at `base + 8 + i*8`, integer arrays keep `I64Store`/`I64Load`
(fannkuch unchanged). Micro-acceptance:
`const a = new Array(2); a[0] = 1.5; a[1] = a[0] * 2; console.log(a[1].toFixed(1))` → `3.0`.

**3. `.length`.** Load the base header at `offset 0` — the length slot already written at
allocation time. Works for int and float arrays alike. Micro-acceptance:
`const a = new Array(3); console.log(a.length)` → `3`.

**4. `.fill(v)`.** Lower to an initialization loop over `0..len` storing `v` at each element
with the repr-directed width (`.fill(1)` on a float array stores `1.0`). Micro-acceptance:
`const a = new Array(3).fill(2); console.log(a[2].toFixed(1))` → `2.0`.

**5. Runtime `Math.sqrt`.** A float-repr `Math.sqrt(x)` lowers directly to `F64Sqrt` on the
f64 argument (no host call). Micro-acceptance: `console.log(Math.sqrt(2).toFixed(6))` →
`1.414214`.

**6. `.toFixed(d)` float→string.** A new `kali:rt float_to_fixed(f64, i32) -> string handle`
host helper: the f64 receiver crosses to the host as a real f64 argument (repr-directed),
Rust formats it to `d` decimal places, and the result is allocated as a guest string via the
existing `alloc_guest_string` path that `int_to_string` already uses. `console.log` already
prints a memory-resident string handle (`crates/kali_runtime/src/host/io.rs`). Micro-acceptance:
`console.log((1.27421999).toFixed(9))` → `1.274219990`.

### 4.3 Output

`console.log` prints the single canonical line: `spectralnorm(100).toFixed(9)`, a float
formatted to 9 decimals via piece 6.

## 5. Acceptance criteria

1. `kali run <spectral-norm fixture>` prints exactly the canonical line for `n = 100`,
   byte-matching a reference Node.js run captured and asserted in the test.
2. Each of pieces 1–6 has its own passing micro-acceptance run-test (above).
3. The fixture ships schema-v1 benchmark metadata (`schemas/benchmark/v1.json`): `sourceFile`,
   validated `sourceSha256`, `buildModes`, plus CLBG attribution; the existing
   `assert_optimization_benchmark_fixture` compile-in-three-modes path passes for it.
4. `cargo test --workspace` is green; fannkuch and the entire integer slice are unchanged
   (zero float seeds ⇒ byte-identical i64 lowering).

## 6. Risks, constraints, and interactions

- **`.toFixed` rounding mode.** Rust's `format!("{:.9}", x)` rounds half-to-even; ECMAScript
  `Number.prototype.toFixed` rounds half **up** (picks the larger candidate on an exact tie).
  They differ only for values exactly halfway at the requested decimal — not hit by
  spectral-norm's computed value. This is documented as a known limitation in
  `specs/19-feature-maturity.md`; the slice does not claim full `toFixed` conformance. The
  reference-captured, pinned expected output guards the actual value.
- **Blast radius / existing tests.** The repr table defaults every `number` node to i64 and
  fannkuch has no float seeds, so the integer slice's observable outputs are unchanged. No
  observable-output test should need its expected string changed; if one does, that is a
  regression to investigate, not to re-baseline. Optimizer fixtures asserting on wasm
  size/instruction counts are evidence notes, updated to new values if real float lowering
  shifts them (per `plan/phase-24/README.md` §24.1/§24.4).
- **Spec governance / claim drift.** Add narrow, honest rows to `specs/19-feature-maturity.md`
  for: runtime f64 arithmetic + int→float promotion, f64 arrays with typed load/store,
  `.length`, `.fill`, runtime `Math.sqrt`, and float→fixed formatting — each scoped to exactly
  this spectral-norm surface, no over-claim beyond it. Keep `proofs/BOUNDARY.md` untouched (no
  new proof-backed claims).
- **Representation inference is bespoke, not a general MIR pass.** Logged as an explicit future
  follow-up (as fannkuch logged its deferred table). It is sufficient for the supported slice.
- **No-GC / pure-Rust invariants.** Reuses the fannkuch bump allocator (no free); no new
  non-Rust dependencies. f64 formatting uses Rust `std` only.
- **Conflicting repr (a `number` node forced both int and float).** Not expressible in
  spectral-norm (floats never index arrays or drive counters). If the inference ever unifies a
  node that also needs an integer-only context (e.g. array index), that stays on the current
  gated/reject path rather than silently truncating — float→int conversion is a non-goal here.

## 7. Suggested implementation sequencing

Representation inference (§4.1) first — it is the load-bearing analysis every piece consumes.
Then pieces in order: 1 (f64 arithmetic + promotion) → 2 (f64 arrays) → 3 (`.length`) →
4 (`.fill`) → 5 (`Math.sqrt`) → 6 (`.toFixed`). Then the vendored spectral-norm port +
end-to-end test + fixture metadata, plus the honest `specs/19-feature-maturity.md` rows. Each
piece lands with its micro-acceptance test before the next begins (TDD).
