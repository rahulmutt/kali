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

### 4.0 Starting point (discovered during planning)

- **`encode` already exists but is inline-only and mis-typed.**
  `new TextEncoder().encode(<string>)` is recognized (built for
  `crypto.subtle.digest` in Stage 3) at `emit/call.rs:3236` +
  `intrinsics/host.rs:327` (`is_text_encoder_encode`), and `repr_infer.rs:2751`
  seeds its result **`Repr::String`**. It fires **only** when the receiver is
  structurally `new TextEncoder()` (a `Call` node) — a **bound** receiver
  `const enc = new TextEncoder(); enc.encode(...)` is NOT recognized. Because the
  result carries `Repr::String`, `console.log(encoded)` / `encoded.length` /
  `encoded === s` would silently take the STRING path and diverge from JS
  `Uint8Array` semantics — a **live latent hazard** masked today only because the
  sole consumer (`digest`) reinterprets it as bytes. Closing it is core P5 work.
- **`decode` does not exist at all.** `TextDecoder` is only a bare name in
  `builtins.rs:113`; no `.decode` handling anywhere in guest codegen.
- **`digest` depends on `encode`→`Repr::String`.** `emit/call.rs:3203` blindly
  `emit_node`s its second operand and reinterprets any i64 as a string handle,
  and accepts a **bound** `const encoded = ...` operand (the package corpus
  exercises exactly that). Migrating `encode` off `Repr::String` therefore
  REQUIRES `digest` to also admit the new provenance (§4.2).

### 4.1 Provenance mechanism — mirror URL/USP/Abort, no new `ValueShape`

Byte-array provenance uses the project's proven two-channel opaque-handle
pattern (identical to `AbortHandle`/`Url`/`UrlSearchParams`), NOT a new
`ValueShape` variant. `ValueShape` is a transient per-emit shape; provenance
must survive a `const` binding, which is the `Repr` axis' job. The result i64 is
the argument's existing `(buf,len)` string handle (a kali string is already
contiguous UTF-8) — a zero-copy reinterpret, exactly as the current inline
`encode` does; no fresh buffer is allocated.

- **New `Repr::Bytes`** in `crates/kali_common/src/repr.rs` (the cross-function /
  capture channel; add arms to the `repr.rs:212-280` classifier chains).
  `repr_infer` seeds the `encode` result `Repr::Bytes` instead of `Repr::String`
  (`repr_infer.rs:2751`), and a `bytes_bindings` seeding set mirrors
  `usp_bindings`/`abort_bindings`.
- **Per-emitter same-function side-tables** in
  `crates/kali_codegen/src/emitter.rs`, mirroring `usp_locals` /
  `abort_handle_locals`: `bytes_locals`, plus stateless-marker sets
  `text_encoder_locals` / `text_decoder_locals` (markers are same-function only;
  a captured/module-scope encoder is YAGNI → fail closed). Inserted at the
  declarator lane in `emit/control_flow.rs` (~957/1018/1067 neighborhood).
- **`admit_bytes_handle_read: bool`** flag mirroring `admit_url_handle_read`, set
  only while `decode`'s receiver-arg and `digest`'s operand are emitted.

### 4.2 What compiles

- `new TextEncoder()` / `new TextDecoder()` → a stateless marker binding
  (recorded in `text_encoder_locals` / `text_decoder_locals`); the marker exists
  only to dispatch `.encode` / `.decode` and may not be read as a value (§5).
- `enc.encode(<string-valued arg>)`, receiver either inline `new TextEncoder()`
  or a bound `text_encoder_locals` name → the existing zero-copy reinterpret, now
  returning **`Repr::Bytes`** provenance. A bound `const encoded = enc.encode(s)`
  records `encoded` in `bytes_locals`.
- `dec.decode(<Bytes-provenance arg>)`, receiver inline or bound → re-label the
  same `(buf,len)` handle back to a `String` result (shape `String`), flowing
  into the `__streq` content-equality lane (e.g.
  `decoder.decode(encoded) !== String(left + right)`). `decode` sets
  `admit_bytes_handle_read` while emitting its argument.
- `crypto.subtle.digest(algo, <Bytes-provenance arg>)` → unchanged runtime
  reinterpret, but its operand emit now sets `admit_bytes_handle_read` so a
  `bytes_locals` operand is admitted (migration of the existing consumer).

### 4.3 What fails closed (`E5506`)

- `enc.encode(<non-string arg>)`; 0-arg / multi-arg `encode`.
- `dec.decode(<arg not proven Bytes-provenance>)` — decode of a string literal,
  an i64, an object; 0-arg / multi-arg `decode`.
- **Every `Bytes`-marker/handle escape position** — the soundness core (§5).

## 5. Soundness invariant (headline review risk)

A `Repr::Bytes` handle (and a stateless encoder/decoder marker) **must not
escape** to any string/i64/float sink. Per the project's standing law —
re-confirmed at for-in-key, throw-fallout Stage 5, and G6 — this is enforced by
an **allowlist at the single identifier-read choke** (`emit/control_flow.rs`
~1744, the same site that already denies `is_url` / `is_url_search_params` /
`is_abort_handle` reads unless the matching `admit_*` flag is set), extended to
deny `is_bytes_handle(text)` / `is_text_encoder_marker(text)` /
`is_text_decoder_marker(text)` reads unless `admit_bytes_handle_read` is set. The
only admitters are `decode`'s receiver-arg and `digest`'s operand.

Because a bare read of a `bytes_locals` binding is denied at that ONE choke,
every escape is closed **by construction** — `return encoded`, `f(encoded)`,
`a[i] = encoded`, `o.f = encoded`, `console.log(encoded)`, `"" + encoded`,
`` `${encoded}` ``, `encoded += x`, `encoded.length`, `encoded[i]`,
`for..of encoded`, `encoded.push(...)`, `encoded.join(...)`,
`encoded === <string>` are all reads at non-admit positions → `E5506`. This is
NOT a per-sink denylist; the plan must still add a **fail-closed test per escape
position** (the enumerate-store-sites-and-generic-sinks discipline the P2/P3/P4
reviews each caught being missed) to prove the single choke covers them.

**Coherence guard:** `repr_infer` must seed `encode` results `Repr::Bytes` (not
`Repr::String`) so `is_string_valued(encoded)` returns `false` — otherwise the
string oracle and the `bytes_locals` choke disagree and a read could fall through
the string path (repr-vs-codegen alias mismatch, cf. P4-R2).

## 6. Acceptance & testing

### 6.1 Acceptance

`webBaselineSmoke` runs **verbatim byte-for-byte** vs node across `kali run`,
browser, and bundle. The fixture **source** is ALREADY verbatim — both
`browser_bundle_web_baseline_source` (`runtime_smoke.rs:4442`) and
`write_web_baseline_interop_source` (`package_corpus.rs:201`) already use real
`String(...)` / `TextEncoder` / `TextDecoder` calls; there are **no `'' + x`
text adaptations to delete**. The flip is in the **test assertions**: the
`build_emits_browser_bundle_web_baseline_primitives*` family
(`runtime_smoke/build.rs:3708+`) currently asserts
`!success && stderr.contains("E5506") && stderr.contains("String")` (a
fail-closed pin) and must become an assert-success + byte-for-byte output check.
The P3 aliased-signal `addEventListener` fail-closed expectations
(`browser_corpus.rs`) stay as-is — a separate, ratified residual. The
`instanceof`/structured-clone run pins in `runtime_smoke/run.rs:380+` are a
DIFFERENT source and are out of P5 scope.

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
- `Repr::Bytes` axis + `bytes_bindings` (types) + `bytes_locals` /
  `text_encoder_locals` / `text_decoder_locals` side-tables +
  `admit_bytes_handle_read` flag + identifier-choke deny (no new `ValueShape`).
- Bound-receiver `enc.encode` recognition; net-new `dec.decode` dispatch;
  `digest` operand migrated to admit `Repr::Bytes`. No host import.
- `soundness_textcodec.rs`; flipped `build_emits_browser_bundle_web_baseline_primitives*`
  assertions (fixture source already verbatim).

## References

- Predecessor: [[kali-url-usp-p4]] (P4, dual-lane stage template, green
  baseline).
- String deny origin: [[kali-g6-unimplemented-builtin-failclosed]] (deny-set,
  denylist-leak law, R-A4 aliased-spelling residuals).
- Standing law (allowlist-at-choke beats denylist-of-sinks): [[kali-forin-spec4a]],
  [[kali-throw-fallout-stage5]].
- UTF-8 string store + runtime-string flow: [[kali-runtime-string-value-flow]],
  [[kali-substring-runtime-spec2]].
