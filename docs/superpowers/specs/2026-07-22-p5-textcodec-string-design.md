# Stage P5 design — `String()` coercion + TextEncoder/TextDecoder → `webBaselineSmoke` finale

**Date:** 2026-07-22
**Branch:** `soundness-stage-p5` (off `main` `694607bb2`)
**Series position:** P2 structuredClone → P3 Abort → P4 URL+USP → **P5 (this stage, finale)**

## 1. Goal

Land the last two web-baseline primitives so the `webBaselineSmoke` fixture
runs **verbatim, byte-for-byte** across all three surfaces (`kali run`, browser,
bundle), retiring the parity series. Two lanes:

1. **`String(x)` runtime coercion** — currently G6-denied `E5506` (5 flip pins
   at the compile-time String frontier). Blocks the fixture's
   `String(left + right)` / `String(left)` calls.
2. **`TextEncoder` / `TextDecoder`** — the fixture's `encoder.encode(...)` /
   `decoder.decode(...)` roundtrip.

Everything else the fixture needs (structuredClone, Abort, Event, URL, USP)
shipped in P2–P4. The `'' + count` recorded adaptations in the acceptance
fixture are deleted this stage.

## 2. Non-goals

- No new host imports. Both lanes are pure linear-memory work (i64→decimal via
  the existing `int_to_string`; UTF-8 byte copy/re-tag). The 4-way `kali:rt`
  browser-harness import-sync surface is therefore untouched.
- No general `Uint8Array` type. `TextEncoder.encode` yields an opaque
  byte-array handle whose only sound consumers are `TextDecoder.decode` (and
  `===`/structural comparison if a test needs it). `.length`, indexing,
  iteration, `push`, `join` on it all fail closed.
- `Boolean` / `toString` / `split` / `JSON.stringify` stay in the G6 deny-set —
  only `String` flips this stage.
- No tracing/copying GC (project invariant).

## 3. Lane 1 — `String(x)` runtime coercion

### 3.1 What compiles

A call-shape arm in `emit_call` (`crates/kali_codegen/src/emit/call.rs`), placed
**before** the `deny_placeholder_lowering` deny-set, admits `String(<arg>)` when
ALL hold:

- `callee_name == "String"` and the callee is a **bare identifier** (no
  children). Gate-1 (`call_target_keeps_placeholder_lowering`) has already
  failed closed if the program binds `String` anywhere, so a callee reaching
  here denotes the intrinsic global.
- **Exactly one argument.**
- The argument is proven-coercible: its `ValueShape` / repr is one of
  `String` (identity passthrough), `Boolean` (→ `"true"`/`"false"`), `Float`
  (`float_to_string`), or i64/`Scalar` (`int_to_string`).

The arm routes the argument straight into the **existing `emit_as_string`
coercion ladder** (`crates/kali_codegen/src/emit/operators.rs:1591`) — the same
ladder `+` string concatenation, template rendering, and multi-arg `console`
already use. Result shape = `String`. Agreement with `+` rendering is by
construction (one shared ladder), not a second hand-mirrored path.

### 3.2 What fails closed (`E5506`)

- Any argument repr outside the four above: objects, arrays, `Unknown`, or an
  unproven value. (JS would produce `"[object Object]"` etc.; kali cannot, so it
  rejects rather than miscompiles.)
- **0-argument `String()`** and **multi-argument `String(a, b)`** — narrowest
  choice; the fixture is always 1-arg. (Decision locked: fail closed, do NOT
  emit `""`.)
- `globalThis.String(x)` / `globalThis["String"](x)` / an aliased
  `const S = String; S(x)` — **left as a documented over-deny residual**
  (P5-R1). These are member/aliased spellings that the bare-identifier arm does
  not match; they continue to fail closed exactly as today. The fixture uses
  bare `String(x)`. This is the same denylist-leak class as G6 R-A4-1..3; the
  eventual real close is an allowlist at the resolve choke point (register
  Group 3), out of scope here. Documented, not silently miscompiled.

### 3.3 Deny-set edit

Remove **only** `"String"` from `deny_placeholder_lowering`
(`emit/call.rs:~3635`). The arm in §3.1 intercepts the admitted shapes upstream;
`String` shapes that fall through (0/multi-arg, non-coercible arg) must still
fail closed, so the arm emits its own `E5506` for those rather than relying on
the removed deny-set entry.

### 3.4 Pins flipped

The 5 compile-time String frontier pins in the `webBaselineSmoke` acceptance
path flip green. Coverage broadened: `String(<i64>)`, `String(<f64>)`,
`String(<bool>)`, `String(<string>)` green; `String(<object>)`,
`String(<array>)`, `String()` (0-arg), `String(a,b)` (multi-arg) → `E5506`.

## 4. Lane 2 — TextEncoder / TextDecoder

kali stores strings as contiguous **UTF-8 bytes** behind a tagged handle
(`STRING_HANDLE_TAG | offset << 32 | len`, `len` = byte length). This makes the
codec a thin, all-strings-sound provenance channel — no ASCII gate (unlike the
substring lane, which needs ASCII because JS substring indexes by UTF-16 code
unit; encode/decode have no such index mismatch).

### 4.1 New value shape

Introduce `ValueShape::Bytes` in `crates/kali_codegen/src/emitter.rs` — a tagged
byte-array handle over `(buf, len)` bytes. It is a **provenance channel**: the
only positions that may consume a `Bytes` value are `TextDecoder.decode(...)`
(and `===` / structural comparison if a test exercises it).

### 4.2 What compiles

- `new TextEncoder()` / `new TextDecoder()` → a stateless marker value. The
  constructors carry no fields; the receiver exists only to dispatch
  `.encode` / `.decode`.
- `enc.encode(<string-valued arg>)` → bump-alloc a fresh global buffer,
  `memory.copy` the argument string's UTF-8 bytes in, produce a `Bytes` handle.
  (A fresh buffer, not aliasing the source, so the result outlives any arena
  reset of the source — mirrors the join-result rule.)
- `dec.decode(<Bytes-provenance arg>)` → re-tag the `(buf, len)` as a `String`
  handle. Result shape `String`, flows into the `__streq` content-equality lane
  (e.g. `decoder.decode(encoded) !== String(left + right)`).

### 4.3 What fails closed (`E5506`)

- `enc.encode(<non-string arg>)`.
- `dec.decode(<arg not proven Bytes-provenance>)` — e.g. decode of a string
  literal, an i64, an object.
- **Every `Bytes`-escape position** — the soundness core (§5).

## 5. Soundness invariant (headline review risk)

`ValueShape::Bytes` **must not escape** to any string/i64/float sink. Per the
project's standing law — re-confirmed at for-in-key, throw-fallout Stage 5, and
G6 — this is enforced by an **allowlist at the single read/resolve site** (only
`decode(...)` and, if a test needs it, `===` consume a `Bytes` value), NOT by
denylisting sinks. Every other position default-denies a `Bytes` value:

- `console.log(bytes)` / print → `E5506` (`emit_as_string` /
  `emit_console_argument_as_string` reject `Bytes`).
- `"" + bytes`, `` `${bytes}` ``, `s += bytes` → `E5506`.
- `bytes.length`, `bytes[i]`, `for..of bytes`, `bytes.push(...)`,
  `bytes.join(...)` → `E5506`.
- Element / field store `a[i] = bytes`, `o.f = bytes` → `E5506`.
- `bytes === <string>` / mixed comparison → `E5506` (only `Bytes === Bytes`, if
  supported, is admitted).

This is the class the whole-stage adversarial review will hammer; the plan must
enumerate STORE sites and generic value sinks (the recurring per-task blind spot
called out in P2/P3/P4 reviews), not just the positions the fixture happens to
touch.

## 6. Acceptance & testing

### 6.1 Acceptance

`webBaselineSmoke` runs **verbatim byte-for-byte** vs node across `kali run`,
browser, and bundle. The `'' + count` recorded adaptations are removed from the
fixture builders (`browser_bundle_web_baseline_source` in
`crates/kali_cli/tests/runtime_smoke.rs`, `write_web_baseline_interop_source` in
`crates/kali_cli/tests/package_corpus.rs`, and the P3 aliased-signal
`addEventListener` fail-closed expectations stay as-is — those are a separate,
ratified residual).

### 6.2 New test module

`crates/kali_cli/tests/soundness_textcodec.rs` (mirrors `soundness_url.rs`):

- **String() coercion:** green pins for `String(<i64>)`, `String(<f64>)`,
  `String(<bool>)`, `String(<string>)` (each asserting the program RUNS and
  prints the node-identical value); `E5506` pins for `String(<object>)`,
  `String(<array>)`, `String()`, `String(a, b)`, `globalThis.String(x)`.
- **Codec roundtrip:** green pin for
  `new TextDecoder().decode(new TextEncoder().encode(s)) === s` over ASCII and
  non-ASCII `s`; assert byte-for-byte.
- **Bytes escape:** one `E5506` pin **per** escape position in §5 (print,
  concat, template, `+=`, `.length`, index, `for..of`, `.push`, `.join`, element
  store, field store, `=== <string>`, decode-of-non-Bytes).
- **Acceptance:** `webBaselineSmoke` three-surface byte-for-byte.

### 6.3 Gate

Double green-baseline gate — `cargo test --workspace` stays **0-failed** before
and after (the CI command, diffed against a `main` worktree), zero drift.
`cargo fmt --check` + `clippy` clean. 6/6 CLBG goldens + web-baseline
byte-for-byte. Census pins (`int_to_string` substring counts, etc.) re-checked
additively per the established `string_tests/lookup.rs` procedure for every new
synthetic.

## 7. Residuals (to carry in `stageD-triage.md` §8.6 / register)

- **P5-R1** — `globalThis.String(x)` / `globalThis["String"](x)` / aliased
  `String` continue to fail closed (documented over-deny, not miscompile). Real
  close is the register Group-3 resolve-choke allowlist.
- **P5-R2** — 0-arg / multi-arg `String()` fail closed rather than JS-semantic
  `""` / first-arg coercion. Deliberate deny boundary.
- **P5-R3** — `Bytes`-in-exotic-position latent divergence: add a walk-4-style
  tripwire test asserting a `Bytes` value in an unusual nested position denies
  rather than silently coerces, in case a future walk arm forgets the shape.
- **P5-R4** — `TextEncoder`/`TextDecoder` are stateless markers; per-instance
  encoding options (`fatal`, `ignoreBOM`, non-UTF-8 labels) are unsupported and
  the constructor-arg forms fail closed.

## 8. Interfaces produced

- `emit_call` String-coercion arm + `emit_as_string` reuse (no new coercion
  logic).
- `ValueShape::Bytes` + its allowlisted consumers and default-deny sinks.
- `TextEncoder`/`TextDecoder` constructor + `.encode`/`.decode` dispatch,
  no host import.
- `soundness_textcodec.rs`; deleted fixture adaptations.

## References

- Predecessor: [[kali-url-usp-p4]] (P4, dual-lane stage template, green
  baseline).
- String deny origin: [[kali-g6-unimplemented-builtin-failclosed]] (deny-set,
  denylist-leak law, R-A4 aliased-spelling residuals).
- Standing law (allowlist-at-choke beats denylist-of-sinks): [[kali-forin-spec4a]],
  [[kali-throw-fallout-stage5]].
- UTF-8 string store + runtime-string flow: [[kali-runtime-string-value-flow]],
  [[kali-substring-runtime-spec2]].
