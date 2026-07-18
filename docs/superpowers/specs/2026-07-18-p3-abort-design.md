# Stage P3 — AbortController/AbortSignal (abort surface + captured handle)

Date: 2026-07-18. Branch: `soundness-batch1-pra`. Predecessor: Stage P2
(`structuredClone`, shipped `c893d5835..7e3aacc02`). Canonical inventory:
`docs/superpowers/followups/stageD-triage.md` §8.6.

## Scope (user-ratified)

Of the §8.6 P3 bundle, this stage takes **AbortController/AbortSignal real
lowering + the minimal captured-handle machinery** needed to run the
webBaselineSmoke abort section byte-for-byte. Explicitly **deferred to P3b**:

- receiver widening (proving more EventTarget-shaped receivers in-lane),
- both silent-drop → total-deny conversions (out-of-envelope dispatch-arg;
  non-capturing listener on unproven receiver),
- the p39 registered-but-under-fired class (`event_target_locals` not shared
  across `FunctionEmitter` scopes),
- the Event-object repr (zero-parameter-listener restriction stays),
- signal-as-EventTarget: `signal.addEventListener('abort', cb)` **fails
  closed E5506 this stage** (ratified); `.abort()` flips the flag only.

Captured-object support is **abort-handle-only** (one new allowlist entry at
the existing choke point); the b2/b7/b2b/b5/b3 general-object denials are
untouched. General closure lowering for `is_scalar == false` cells remains
the §8.6 lifting-plan item.

## Why this shape (approach decision)

Three approaches were considered:

- **A (chosen) — global-heap abort cell**: one `__alloc_global` cell per
  instance; handle = i64 pointer; provenance via a new `Repr` variant.
  Reuses two proven mechanisms (P2's escape-safe global allocation, the
  Spec-1/P2 repr-axis pattern), and makes the captured handle sound **by
  construction**: the pointee outlives every frame.
- **B — per-allocation-site WASM global**: identity per *site*, not per
  *instance*; loop/multi-call sites alias their own flag and would need an
  executes-at-most-once analysis. Weaker envelope, comparable effort.
- **C — host intrinsics**: state is module-internal this stage (signal
  listeners fail closed, so the host observes nothing); every new host
  import must be hand-mirrored into the 4 browser-harness JS import lists
  (known LinkError footgun). Pure cost until signal-as-EventTarget lands.

## 1. Runtime representation

- `new AbortController()` → `__alloc_global(8)`, cell initialized to `0`
  (not aborted). The i64 pointer is the **abort handle**. No header, no
  tag: the handle is distinguished purely by compile-time provenance, never
  by inspection — which is why every position it can reach must be
  allowlisted (§2).
- `controller.signal` → identity (same handle). No second allocation; no
  runtime controller-vs-signal distinction.
- `controller.abort()` → `i64.store` of `1`. Idempotent by construction
  (matches node; agrees with `kali_api_web`'s `swap` semantics).
- `controller.signal.aborted` / `s.aborted` → `i64.load`, rendered as a
  boolean `1`/`0` per the established dynamic-boolean convention (P2).

**Provenance: new `Repr::AbortHandle` variant** in
`kali_common::repr::Repr` — NOT a codegen-side shadow set. Rationale (the
Spec-2 lesson): codegen oracles and `kali_types` predicates are
hand-mirrored; both sides must consult the same table or they desync.
Inference seeds it at exactly one point (`new AbortController()` `const`
declarator init) and propagates through allowlisted flows only: `const s =
c.signal` aliasing and direct member reads. **No** params, returns, object
fields, or array elements this stage — unproven flows keep the default I64
and every abort operation on them fails closed.

## 2. Sound envelope and the fail-closed set

Admitted surface (exhaustive):

| Form | Requirement | Lowering |
|---|---|---|
| `const c = new AbortController()` | declarator init, `const` only | `__alloc_global(8)`, store 0 |
| `c.abort()` | `c` proven AbortHandle | store 1 to cell |
| `c.signal` | value position feeding an admitted consumer only | same handle |
| `const s = c.signal` | `const` declarator | alias; repr propagates |
| `c.signal.aborted` / `s.aborted` | proven provenance | load; boolean render |
| `s instanceof AbortSignal` / `c.signal instanceof AbortSignal` | left proven AbortHandle; right is the global `AbortSignal` identifier, unshadowed | compile-time `i64.const 1` |
| boolean positions of `.aborted` (`if`, `!`, `&&`, ternary cond, `===` with a boolean) | via the load above | normal i64 boolean flow |

The `instanceof` allow lane is carved **before** the blanket
`in`/`instanceof` runtime trap in `emit/operators.rs` (~line 1544),
following the P2 Lane-3 precedent (`===` allow lane before the blanket
object gate): both sides proven, everything else falls through to the
existing trap.

**Everything else fails closed (E5506 at compile time).** Per the P2
standing lesson, the STORE sites and generic value-position sinks are
enumerated here, up front:

- **Generic value sinks**: `console.log(c)` / `console.log(c.signal)`,
  string concat / template interpolation, arithmetic, `===`/`!==` with the HANDLE itself as an operand (no
  identity lane this stage; `.aborted === true` is fine — that compares
  the loaded boolean, not the handle), `JSON.stringify`, return
  position, **argument position** (no receiver widening — passing `c` or
  `s` to any function denies), object-literal field, array element,
  `push`.
- **Store sites**: binding reassignment cannot arise (admission is
  `const`-declarator-only, so a proven handle binding is never
  assignable; `let c = new AbortController()` is itself denied at the
  init site as a non-`const`-declarator position, leaving `c` an
  unproven I64 whose abort operations all deny), `c.signal = x`, `s.aborted = x` (node silently ignores the write; kali
  denies loudly rather than emulate), `c.<anything> = x`, computed member
  `c[k]` read or write.
- **Other members / statics**: `s.addEventListener(...)` (ratified),
  `s.onabort`, `s.reason`, `s.throwIfAborted()`,
  `AbortSignal.timeout/abort/any`, `new AbortController()` in
  non-`const`-declarator positions (e.g. returned from a function).
  Loop-allocated controllers ARE admitted: each iteration allocates a
  fresh global cell — correct per-instance semantics, leaks by design on
  the GC-less model like every `__alloc_global` client.
- **`instanceof` stays the blanket trap** for every other shape, including
  `c instanceof AbortController` (node-true; inventoried for P3b, not
  implemented).

Mechanism: the Spec-4a/P2 discipline — a **position allowlist at the
single resolve/read site** for AbortHandle-repr bindings (the choke point
where `admit_growable_field_read` sits), not a denylist of bad sinks.

## 3. The capture lane

**Allowlist entry 3 at `unlowered_capture_denied`**
(`crates/kali_codegen/src/intrinsics/host.rs`). Default-deny stays the
law; the one new admitted class is a depth-1 captured binding with proven
`AbortHandle` repr. Soundness: the env record stores the handle (i64
pointer) by value, and the pointee is a never-reclaimed `__alloc_global`
cell — restoring the pointer after the owner frame dies dereferences live
memory. This is exactly the property captured arena-backed objects lack
(b2/b7), so the entry generalizes to nothing else; b2/b7/b2b/b5/b3 pins
stay red-proof.

**Repr crosses the function boundary via env-slot metadata.** Env slots
currently carry only the scalar/non-scalar bit; the layout gains per-slot
repr (minimally an is-abort-handle bit). The callback's `FunctionEmitter`
seeds `AbortHandle` for the captured binding from that metadata. Inside
the callback the **same §2 position allowlist** governs — `.abort()` and
`.signal.aborted` work; `console.log(controller)` denies — identically to
module scope. One discipline on both sides of the boundary.

**The hand-mirrored exclusion list flips, as designed.** `AbortController`
joins `Array`/`Uint8Array`/`EventTarget` in
`declarator_init_is_placeholder_construct`
(`crates/kali_codegen/src/lower.rs:2145`) in the same change that gives it
a real lowering — the documented procedure the Set tripwire guards.
Consequences:

- allowlist entry 2 (zero-placeholder constructs) no longer admits a
  captured `controller`; entry 3 admits it as a real value;
- `deferred_listener_nonscalar_placeholder_capture_still_builds` is
  re-scoped: the webBaselineSmoke listener builds because `count` is
  entry 1 and `controller` is entry 3, and the assertion upgrades from
  "builds, warn" to "builds AND the abort really lands";
- `deferred_capture_of_bound_set_placeholder_tripwire` and
  `deferred_capture_nested_shadow_placeholder_denies` stay green (Set
  still lowers to 0) and keep guarding the next constructor transition.

**Acceptance-listener dependencies that are NOT new machinery:**
`count += 1` visibility after the synchronous `dispatchEvent` rides the
existing Stage-C scalar-cell promotion (owner frame alive during sync
dispatch); the events lane's dispatch path is unchanged. If planning-time
exploration finds dispatch routes the callback through a path that loses
the env pointer, that is a blocker to surface, not to design around
silently.

## 4. Testing, gating, acceptance

**Acceptance (drives the stage):** a runnable fixture = the
webBaselineSmoke *prefix* (structuredClone section + abort/EventTarget
section, ending right before `URLSearchParams`), executed via `kali run`,
**byte-for-byte against node** — including the full listener round-trip:
`dispatchEvent` returns true, `count === 1`,
`controller.signal.aborted === true`.

**Deliberate-flip fixture ratchets one notch:** currently denies at
`controller.signal instanceof AbortSignal`; after P3 it must deny at the
next unsupported family (`URLSearchParams`), `success == false`
preserved.

**New pin file `soundness_abort.rs`:**

- positive pins for every §2 admitted row;
- E5506 pins for the enumerated fail-closed set (store sites + generic
  sinks pinned up front, not left for review to find);
- capture pins: entry-3 admit works end-to-end; a captured handle in a
  denied position denies; a captured non-abort object still denies;
- the §3 exclusion-list flip pins;
- an `instanceof` shadowing pin: a user-defined `AbortSignal` class must
  not hit the allow lane (p03c precedent).

**Gate (unchanged convention):** full-workspace `cargo test` diffed
against a **main worktree** (never a mid-branch baseline —
[[ci-gate-vs-poisoned-baseline]]), 694-baseline held, 0 newly-red,
double-enumerated, **`package_corpus` included in per-task gates** (the P2
miss). Newly-found reds bisect to the stage base before attribution. No
new WASM synthetic expected (abort lowering is inline loads/stores); if
one appears, the `SYNTHETIC_FUNCTIONS` census mirror syncs in the same
commit. Whole-stage adversarial review closes the stage (it has caught
the worst defect four stages running — enumerate store sites and generic
sinks for the new value class during that review too).

## 5. Follow-up inventory seeded for P3b

Receiver widening; both silent-drop → total-deny conversions; the p39
under-fired class; Event-object repr; signal-as-EventTarget
(`addEventListener('abort', cb)` firing on `.abort()`); `instanceof
AbortController`; `s.reason` / `throwIfAborted()` / `AbortSignal`
statics; abort-handle `===` identity.
