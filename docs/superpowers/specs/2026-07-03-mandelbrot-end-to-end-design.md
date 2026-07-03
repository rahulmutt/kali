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
`f64` lane and adds integer **bitwise operators** plus a way to emit the packed bitmap.
It needs no garbage collector, hashtable, arbitrary-precision bignum, or regex engine.

**This slice deliberately opens two lanes** (bitwise-integer operators and faithful
binary stdout), because the chosen output strategy is a byte-for-byte-faithful PBM image
rather than a checksum adaptation. This is a conscious departure from the usual
one-lane-per-fixture cadence, taken so the fixture reproduces canonical mandelbrot's
actual output bytes.

## Goal

Vendor a `mandelbrot` fixture that executes end-to-end under `kali run` and writes the
**byte-for-byte canonical binary PBM image** to stdout, pinned as deterministic output —
recording execution-correctness coverage for a new **bitwise-integer** lane and a new
**binary-stdout** runtime lane, not a throughput claim.

## Non-goals

- No throughput / performance claim (consistent with the other CLBG maturity rows).
- No general typed-array / `Uint8Array` / `Deno.stdout` surface — the guest emits bytes
  through a narrow kali-namespaced intrinsic (see Output).
- **No binary stdout in the browser harness** — the binary sink is host-runtime only in
  this slice; the browser harness serializes stdout as a JSON string and is left on the
  existing text path (see Runtime plumbing → browser).
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

The canonical stdout is the ASCII header `P4\n<n> <n>\n` followed by the packed bitmap
bytes — for `n = 200`, an 11-byte header + 5000 bitmap bytes.

## Compiler work — lane 1: bitwise-integer operators

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

## Compiler + runtime work — lane 2: faithful binary stdout

### The problem

The runtime's stdout is a UTF-8 `String` end-to-end: `KaliHostState.stdout`
(`crates/kali_runtime/src/state.rs:29`), the public `RunOutcome.stdout`
(`crates/kali_runtime/src/outcome.rs:15`, cloned through ~8 sites in `execute.rs`), and
the CLI flush is `print!("{}", outcome.stdout)` (`crates/kali_cli/src/bin/cmd_run.rs:258`).
Arbitrary PBM bytes (0–255) are not valid UTF-8 and cannot ride that `String` path.

### Dual-sink runtime model

Add a **second, byte-capable sink** rather than converting the whole stdout pipeline to
`Vec<u8>` (which would also force the browser harness's JSON-string serialization to
base64/escape — out of scope here):

- `KaliHostState` gains `stdout_bytes: Vec<u8>` alongside the existing `stdout: String`.
- `RunOutcome` gains a matching `stdout_bytes: Vec<u8>` (threaded through the same
  `execute.rs` clone sites as `stdout`).
- New `kali:rt` host import **`stdout_write_bytes(handle: i64)`**: decodes an array handle
  from guest linear memory (same `[len@+0][elem@+8…]` layout the array lane uses; each
  `i64` element contributes its low byte) and appends those raw bytes to `stdout_bytes`.
  Modeled on the existing `string_concat` / `decode_string_handle_bytes` host decode path.
  Gated by the existing `HostOperation::Console` policy check, like `console_log`.
- `kali run` flush (`cmd_run.rs`): after the existing `print!("{}", outcome.stdout)` text
  flush, write the byte sink with `io::stdout().write_all(&outcome.stdout_bytes)` +
  flush. For mandelbrot the text sink is empty and the byte sink holds the whole PBM, so
  ordering is unambiguous; the general interleaving of text and binary output is not a
  concern this fixture exercises and is left undefined for now.

### Guest-facing surface — batched array write

The guest emits bytes through one narrow kali-namespaced intrinsic,
**`Kali.writeStdoutBytes(arr)`**, chosen over per-byte writes (chatty) and over
`Deno.stdout`/`Uint8Array` (a large typed-array feature out of scope):

- The program builds an ordinary array (the existing array lane) whose elements are byte
  values 0–255 — the ASCII PBM header bytes followed by the packed bitmap bytes — and
  passes it to `Kali.writeStdoutBytes(out)`.
- **Codegen** recognizes the `Kali.writeStdoutBytes(x)` call (alongside the existing
  `Kali.*` intrinsic recognition, e.g. `Kali.test`), emits `x` as the array handle
  (`i64`), and calls the `stdout_write_bytes` host import.
- **Type resolution** admits `Kali.writeStdoutBytes` as a known intrinsic member so it is
  not rejected during resolution (follow the existing `Kali` namespace handling).
- One host call per program; the array lane (alloc, element write, `.length`) is already
  proven by spectral-norm / n-body.

### Browser harness

The browser harness embeds stdout into a JSON summary **string** and cannot carry raw
binary without base64/escaping. Binary stdout is therefore **host-runtime only** in this
slice. If `Kali.writeStdoutBytes` is reached under the browser backend, it must
**diagnose/gate** rather than silently drop or corrupt output. The `clbg_*_runtime.rs`
tests already invoke the `kali` binary directly (host path), so the fixture's test is
unaffected.

## Reused lanes (no new work)

- `f64` arithmetic + int→float promotion (`2.0*y/n`, `Zr*Zi`, `Tr+Ti`) — spectral-norm lane.
- Arrays (`new Array(n)`, element write, `.length`) — spectral-norm / n-body lane.
- `break`, integer `for` loops, mutable `i64`/`f64` locals — fannkuch/spectral lanes.

## The fixture (shape, not final text)

`crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts`, e.g.:

```ts
// The Computer Language Benchmarks Game — mandelbrot, TS port normalized to Kali.
function mandelbrot(n) {
  // header "P4\n<n> <n>\n" (n = 200) + n*n/8 packed bitmap bytes
  const out = new Array(11 + n * n / 8);
  out[0] = 80; out[1] = 52; out[2] = 10;               // "P4\n"
  out[3] = 50; out[4] = 48; out[5] = 48; out[6] = 32;  // "200 "
  out[7] = 50; out[8] = 48; out[9] = 48; out[10] = 10; // "200\n"
  let p = 11;
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
        out[p] = byte;
        p = p + 1;
        byte = 0;
        bits = 0;
      }
    }
  }
  Kali.writeStdoutBytes(out);
}
mandelbrot(200);
```

(`n = 200` divisible by 8, so `bits` always reaches 8 at row end — no flush-remainder
branch. The header digits are hard-coded ASCII for `n = 200`; if `n` is retuned the header
bytes and the pinned output re-pin together. Final text is settled during
implementation/TDD.)

## Reference generation and pinning

The pinned canonical output is produced offline by an **independent reference
implementation** of the exact canonical algorithm at the chosen `n` (a short throwaway
script), emitting the real PBM header + packed bytes. The test pins both the byte length
and the **sha256** of stdout (and may embed the reference bytes as a fixture asset). The
kali fixture must reproduce it exactly. This mirrors n-body's discipline of pinning an
externally-derived canonical result. (The reference script is a verification aid; it is
not vendored into the test tree.)

## Testing

Mirror `crates/kali_cli/tests/clbg_nbody_runtime.rs`:

- `clbg_mandelbrot_runtime.rs`:
  - `mandelbrot_runs_and_matches_canonical_output` — `kali run` the fixture, assert
    success and that raw stdout **bytes** equal the canonical PBM (compare length + sha256,
    or exact bytes). Uses the `Command` output's `Vec<u8>` stdout directly.
  - `mandelbrot_metadata_is_consistent` — parse `mandelbrot-benchmark-v1.json`, assert
    `benchmark`, `version`, `sourceFile`, `buildModes`, and that `sourceSha256` matches the
    fixture file digest.
- Codegen unit coverage for lane 1: `<<`, `>>`, `>>>`, `&`, `|`, `^` each produce the
  correct wasm/result (including a `>>>` uint32 case and a negative-operand `>>` vs `>>>`
  divergence), plus a test that a **bitwise op on an `f64` operand is rejected with
  `E5506`** (not miscompiled).
- Runtime unit coverage for lane 2: `stdout_write_bytes` appends the array's low-byte
  stream to the byte sink; `kali run` flushes raw bytes; a program mixing `console.log`
  text and `Kali.writeStdoutBytes` keeps the two sinks intact; browser backend gates the
  intrinsic rather than corrupting output.
- `mandelbrot-benchmark-v1.json` metadata with `buildModes: ["--fast", "--release", "--release-advanced"]`.

## Documentation and memory

- Add maturity rows to `specs/19-feature-maturity.md` for (a) the bitwise-integer-operator
  lane (real `i64`/JS-32-bit lowering; `f64`-operand reject) and (b) the host-only
  binary-stdout lane (`Kali.writeStdoutBytes`; browser-gated), and extend the
  optimization-evidence-lane row to name mandelbrot as the fixture exercising them.
- Update the `kali-heap-object-lane` / verification memories with the new fixture, the
  closed bitwise-`I64Add` miscompile, and the new dual-sink binary-stdout surface.

## Risks / validate early

1. **Fuel budget.** `n=200` ≈ 40k pixels × up to 50 iterations of ~5 `f64` ops. Compare
   to spectral-norm(100) (~12–15M fuel) against the 60M default. Measure first; if it
   trips fuel or CI wall-clock, drop to a smaller divisible-by-8 `n` (e.g. 128 or 64) and
   re-pin. The pinned output and the header bytes are `n`-specific by construction.
2. **f64 inference of zero-initialized accumulators.** `Zr/Zi/Tr/Ti` start at `0.0`;
   confirm repr inference marks them `f64` (seeded by the float literals and `/` and the
   `f64`-valued assignments). If inference instead leaves one on the `i64` path, the
   reject-don't-miscompile invariant means a compile error, not a wrong answer — but
   validate so the fixture actually compiles.
3. **Bitwise reject path.** Confirm the new `f64`-operand `E5506` rejection fires and the
   old silent-`I64Add` warning path is fully removed (no residual miscompile).
4. **Byte-sink threading.** `RunOutcome.stdout_bytes` must be carried through every
   `execute.rs` site that currently clones `stdout` (~8), or binary output silently
   vanishes on some run paths. The unit test that flushes bytes through `kali run` guards
   this.
5. **Array element → byte truncation.** `stdout_write_bytes` takes each `i64` element's
   low 8 bits; the fixture only stores 0–255, but the host decode must mask explicitly so
   an out-of-range element can never emit a multi-byte or sign-extended value.

## Follow-ups (deferred, out of scope here)

- **Browser binary stdout** — carry `stdout_bytes` through the browser harness (base64 in
  the JSON summary, decoded host-side) so the fixture also runs under the browser backend.
- **General text/binary stdout interleaving** — define ordering if a later program mixes
  `console.log` and byte writes.
- **Typed-array / `Deno.stdout` surface** — a faithful `Deno.stdout.writeSync(Uint8Array)`
  port, if a later fixture needs real typed arrays.
- **Float-operand bitwise coercion** (`ToInt32` of an `f64`), if a later fixture needs it.
