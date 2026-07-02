# Float console output + reserved glue names (2026-07-02)

Two production fixes landed together (see the same-named plan for task detail).

**Float console/string bridge.** Root cause: the i64 tagged-value domain has no
float encoding and had no f64→string bridge; the console-import runtime path
and `emit_as_string` (string `+`) passed `Float`-shaped operands into i64-typed
calls unconverted, emitting type-invalid wasm ("expected type i64, found
f64.div"). Division was only the most common float seed — float locals,
params, returns, arrays, comparisons, and `toFixed` already handled floats
correctly in their own operations. Reads of mutable locals and params emit
shape `Unknown`, so the two fixed seams also consult the repr-based
`is_float_valued` predicate (the same signal the working float seams use).
Fix: unconditional `kali:rt float_to_string (f64) -> i64` host import (index
20; `COVERAGE_HIT_IMPORT_INDEX`/`FUNCTION_INDEX_OFFSET` bumped to 21),
mirrored in all four hand-mirrored JS import lists, with `Float`-arm
adaptations at both emit seams. Semantics: JS `String(number)` —
`NaN`/`Infinity`/`-Infinity`/`0` (for ±0) special-cased on the Rust host,
finite doubles use the ECMA-262 Number-to-String algorithm via `ryu-js`
otherwise; JS mirrors use native `String(value)`.

The formatting divergences originally recorded here (JS exponent notation for
very small magnitudes and for |x| >= 1e21) were closed on 2026-07-02: the
host now delegates finite doubles to `ryu-js` (exact ECMA-262
Number-to-String), so host and browser stdout agree — see
`2026-07-02-template-literal-interpolation-and-js-number-format-design.md`.
Statically-rendered literal console output (`render_static_value`) still
echoes source text (`console.log(0.0000001)` prints `0.0000001` on both
lanes, where JS engines print `1e-7`) — host/browser-consistent but
source-echoing; runtime-computed values are exact.

**Reserved glue export names.** `kali build --bundle` now fails with E5511
when a user export is named `load`, `loadWithImports`, `loadDynamicImport`, or
`start` — previously a green build emitted an unloadable ESM module (duplicate
declaration) or silently shadowed the CJS helper.

**Recorded, out of scope (since closed):** template literals printed their raw
source (`` console.log(`v: ${7 / 2}`) `` printed `v: ${7 / 2}`) — fixed by the
parser desugar in `2026-07-02-template-literal-interpolation-and-js-number-format-design.md`.
