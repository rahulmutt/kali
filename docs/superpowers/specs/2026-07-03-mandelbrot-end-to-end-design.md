# mandelbrot — end-to-end CLBG fixture (design)

Date: 2026-07-03
Status: proposed (awaiting user review)
Topic: next Computer Language Benchmarks Game (CLBG) fixture after n-body

## Context

kali has three vendored, end-to-end-executing adapted CLBG fixtures, each of which
opened one new representation lane:

- **fannkuch-redux** — integer imperative core (`i64` loops, mutable locals, calls).
- **spectral-norm** — floating-point (`f64`) arithmetic, `f64` arrays, `Math.sqrt`, `.toFixed`.
- **n-body** — fixed-shape bump-allocated heap objects (see the heap-object lane spec/plan).

There is no documented ordering for the remaining canonical benchmarks. Of them,
**mandelbrot** needs the least new compiler infrastructure: it extends the existing
`f64` lane and adds only integer **bitwise operators** plus a way to emit the packed
bitmap. It needs no garbage collector, hashtable, arbitrary-precision bignum, or regex
engine, so it keeps the established "one new lane per fixture" cadence.

## Goal

Vendor a `mandelbrot` fixture that executes end-to-end under `kali run` and reproduces
the canonical mandelbrot bitmap, pinned as deterministic output — recording
execution-correctness coverage for a new **bitwise-integer** lane, not a throughput claim.

## Non-goals

- No throughput / performance claim (consistent with the other CLBG maturity rows).
- No binary / raw-byte stdout runtime lane in this slice (deferred; see Follow-ups).
- No general JS numeric-tower or float↔int bitwise coercion beyond what is specified.
- No `?:` ternary, `continue`-in-`for`, or other unproven surface in the fixture.

## The benchmark and its canonical algorithm

Standard CLBG mandelbrot renders an `n × n` 1-bit image. Per pixel it iterates the
complex map `z ← z² + c` up to 50 times, treating the pixel as *in-set* while
`|z|² ≤ 4`. The canonical inner loop (transcribed from the upstream JavaScript
submission) is:

```
Zr = Zi = Tr = Ti = 0
for i in 0..50:
    Zi = 2*Zr*Zi + Ci
    Zr = Tr - Ti + Cr
    Tr = Zr*Zr
    Ti = Zi*Zi
    if Tr + Ti > 4.0: break
inSet = (Tr + Ti <= 4.0)   // 1 bit
```

with `Ci = 2.0*y/n - 1.0` and `Cr = 2.0*x/n - 1.5`.

Pixels are packed **8 per byte, most-significant-bit first**, matching the PBM `P4`
binary format where a set bit is a black (in-set) pixel:
`byte = (byte << 1) | inSet`. In PBM each *row* is padded to a byte boundary; we choose
`n` divisible by 8 (`n = 200`, 25 bytes/row) so no intra-row padding logic is needed.

## Compiler work — the new lane

### Real bitwise-operator lowering

Today `<<`, `>>`, `>>>`, `&`, `|`, `^` are recognized as binary operators
(`crates/kali_codegen/src/lower.rs`) but have **no emit arm** in
`crates/kali_codegen/src/emit/operators.rs`; they hit the catch-all at the end of the
binary-op match, which pushes a `UNIMPLEMENTED` **warning** and emits `I64Add`. That is
a silent **miscompile** (`a << 1` compiles to `a + 1`), violating the repo's
reject-don't-miscompile invariant.

This slice replaces the fall-through with real lowering for **`i64`-inferred operands**,
using **JS 32-bit truncation semantics** so results are faithful in general, not just for
mandelbrot's small operand ranges:

| Operator | Lowering (operands coerced to 32-bit first) |
|---|---|
| `&` `\|` `^` | `i32.and` / `i32.or` / `i32.xor`, sign-extended back to `i64` |
| `<<` | `i32.shl` (shift count masked to 5 bits, per JS), sign-extended to `i64` |
| `>>` | `i32.shr_s` (arithmetic), sign-extended to `i64` |
| `>>>` | `i32.shr_u` (logical), **zero**-extended to `i64` (uint32 result) |

Operand coercion = wrap `i64 → i32` (`i32.wrap_i64`), matching JS `ToInt32`/`ToUint32`.

For mandelbrot specifically only `<<` and `|` on small non-negative integers are used, so
the 32-bit-vs-64-bit distinction is invisible to the fixture; implementing the full,
correct semantics is what removes the miscompile for the language at large.

### Reject-don't-miscompile for unsupported operands

Bitwise operators whose operands are **`f64`-inferred** (or otherwise not integer) are
**rejected** with `E5506` (`FEATURE_UNAVAILABLE`) instead of the current silent
`I64Add`. This closes the miscompile on the path the fixture does not use, keeping the
invariant intact. (JS would `ToInt32`-coerce a float operand; that coercion path is
explicitly out of scope and rejected rather than half-implemented.)

## Reused lanes (no new work)

- `f64` arithmetic + int→float promotion (`2.0*y/n`, `Zr*Zi`, `Tr+Ti`) — spectral-norm lane.
- `%` on `i64` (the Adler-32 fold) — existing integer arithmetic.
- `break`, integer `for` loops, mutable `i64`/`f64` locals — fannkuch/spectral lanes.
- Literal-rooted `console.log` string concatenation of an integer — fannkuch output lane.

## Output strategy — checksum line

kali has no binary/raw-byte stdout today (only `console.log` of strings/ints via the
`int_to_string` / `string_concat` host helpers). Rather than introduce a second new lane
(binary I/O) in the same slice, the fixture emits a **checksum** of the packed bitmap:

- The program packs pixels into **PBM-body-identical bytes** (8/px, MSB-first, no padding
  because `n % 8 == 0`) and folds each completed byte through **Adler-32**:
  `a = (a + byte) % 65521; b = (b + a) % 65521;` (initial `a = 1, b = 0`), then prints
  `b * 65536 + a` on one line via the proven literal-rooted `console.log` path.
- Adler-32 uses only `+` and `%`, and every intermediate stays well within `i64`, so it
  rides existing lanes with no new runtime surface.

Because the fixture packs bytes exactly as the canonical PBM body does and checksums that
exact byte stream, the pinned checksum **equals Adler-32 of the real upstream PBM body**
at the same `n`. The checksum therefore *certifies the computed bitmap is byte-identical
to canonical mandelbrot* — a strong correctness signal without binary I/O.

### Reference generation and pinning

The pinned checksum is produced offline by an **independent reference implementation** of
the exact canonical algorithm at the chosen `n` (a short throwaway script), Adler-32'd
over its PBM body bytes. The kali fixture must reproduce that value. This mirrors n-body's
discipline of pinning an externally-derived canonical result, not a self-generated one.
(The reference script is a verification aid; it is not vendored into the test tree.)

## The fixture (shape, not final text)

`crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts`, in the same
imperative style as spectral-norm, e.g.:

```ts
// The Computer Language Benchmarks Game — mandelbrot, TS port normalized to Kali.
function mandelbrot(n) {
  let a = 1;              // Adler-32 low
  let b = 0;              // Adler-32 high
  for (let y = 0; y < n; y = y + 1) {
    const Ci = 2.0 * y / n - 1.0;
    let byte = 0;
    let bits = 0;
    for (let x = 0; x < n; x = x + 1) {
      const Cr = 2.0 * x / n - 1.5;
      let Zr = 0.0; let Zi = 0.0; let Tr = 0.0; let Ti = 0.0;
      for (let i = 0; i < 50; i = i + 1) {
        Zi = 2.0 * Zr * Zi + Ci;
        Zr = Tr - Ti + Cr;
        Tr = Zr * Zr;
        Ti = Zi * Zi;
        if (Tr + Ti > 4.0) { break; }
      }
      let bit = 0;
      if (Tr + Ti <= 4.0) { bit = 1; }
      byte = (byte << 1) | bit;
      bits = bits + 1;
      if (bits === 8) {
        a = (a + byte) % 65521;
        b = (b + a) % 65521;
        byte = 0;
        bits = 0;
      }
    }
  }
  return b * 65536 + a;
}
console.log("" + mandelbrot(200));
```

(`n = 200` divisible by 8, so `bits` always reaches 8 at row end — no flush-remainder
branch. Final text is settled during implementation/TDD.)

## Testing

Mirror `crates/kali_cli/tests/clbg_nbody_runtime.rs`:

- `clbg_mandelbrot_runtime.rs`:
  - `mandelbrot_runs_and_matches_canonical_output` — `kali run` the fixture, assert
    success and exact pinned checksum stdout.
  - `mandelbrot_metadata_is_consistent` — parse `mandelbrot-benchmark-v1.json`, assert
    `benchmark`, `version`, `sourceFile`, `buildModes`, and that `sourceSha256` matches the
    fixture file digest.
- Unit coverage for the new bitwise lane in the codegen crate: `<<`, `>>`, `>>>`, `&`,
  `|`, `^` each produce the correct wasm/result (including a `>>>` uint32 case and a
  negative-operand `>>` vs `>>>` divergence), plus a test that a **bitwise op on an
  `f64` operand is rejected with `E5506`** (not miscompiled).
- `mandelbrot-benchmark-v1.json` metadata with `buildModes: ["--fast", "--release", "--release-advanced"]`.

## Documentation and memory

- Add a maturity row to `specs/19-feature-maturity.md` for the bitwise-integer-operator
  lane (real `i64`/JS-32-bit lowering; `f64`-operand reject), and extend the
  optimization-evidence-lane row to name mandelbrot as the fixture exercising it.
- Update the `kali-heap-object-lane` / verification memories with the new fixture and the
  closed bitwise-`I64Add` miscompile.

## Risks / validate early

1. **Fuel budget.** `n=200` ≈ 40k pixels × up to 50 iterations of ~5 `f64` ops. Compare
   to spectral-norm(100) (~12–15M fuel) against the 60M default. Measure first; if it
   trips fuel or CI wall-clock, drop to a smaller divisible-by-8 `n` (e.g. 128 or 64) and
   re-pin the checksum. The checksum is `n`-specific by construction.
2. **f64 inference of zero-initialized accumulators.** `Zr/Zi/Tr/Ti` start at integer-ish
   `0.0`; confirm repr inference marks them `f64` (seeded by the float literals and `/`
   and the `f64`-valued assignments). If inference instead leaves one on the `i64` path,
   the reject-don't-miscompile invariant means a compile error, not a wrong answer — but
   validate so the fixture actually compiles.
3. **Bitwise reject path.** Confirm the new `f64`-operand `E5506` rejection fires and the
   old silent-`I64Add` warning path is fully removed (no residual miscompile).

## Follow-ups (deferred, out of scope here)

- **Faithful binary PBM output.** Add a runtime byte / binary-stdout host helper so a
  variant fixture writes the real `P4\n<n> <n>\n` + packed bytes, with the test pinning
  `sha256` of raw stdout. This is a second, orthogonal runtime-I/O lane; sequencing it
  after the checksum fixture keeps each slice to one lane.
- **Float-operand bitwise coercion** (`ToInt32` of an `f64`), if a later fixture needs it.
