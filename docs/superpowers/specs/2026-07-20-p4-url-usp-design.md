# Stage P4 — URL + URLSearchParams (self-contained in-wasm lane)

Date: 2026-07-20. Branch: `soundness-batch1-pra`. Predecessor: Stage P3
(AbortController/AbortSignal, shipped `15c2b34f9..7cf130762`). Canonical
inventory: `docs/superpowers/followups/stageD-triage.md` §8.6, item
"Stage P4 — `URL` + `URLSearchParams`".

## Scope (user-ratified)

The parity roadmap is P2 structuredClone → P3 Abort → **P4 URL+USP** → P5
TextEncoder → byte-for-byte `webBaselineSmoke` acceptance. This stage lands
`URL` and `URLSearchParams` real lowering, exactly enough to run the
web-baseline `URLSearchParams`/`URL` block byte-for-byte against node.

**In scope** — the fixture surface (`runtime_smoke.rs:454–463`):

- `new URLSearchParams('alpha=1&beta=two+words')` — **string-literal arg only**
- `.append(k, v)` · `.set(k, v)` (dynamic value, e.g. `String(count)`) ·
  `.get(k)` · `.getAll(k)` (`.length` used) · `.has(k)` · `.toString()`
- `new URL('https://example.com/browser?alpha=1#fragment')` — **literal arg only**
- `.origin` · `.pathname` · `.search` · `.hash` · `.href` · `.searchParams`
  (composing `url.searchParams.get('alpha')`)

**Explicitly deferred / out of scope:**

- **Runtime-computed constructor args** (`new URL(someVar)`): fail closed
  E5506. Not exercised by the fixture; supporting them needs an in-wasm query
  parser, not just the compile-time parse this stage uses.
- **URL mutation** (`url.pathname = …`, `url.search = …`): URL is immutable
  this stage; any URL member *write* fails closed.
- **USP iteration / entries/keys/values/forEach/sort/delete**: not in the
  fixture; fail closed.
- The **Event-type block** and the **TextEncoder tail** of the web-baseline
  fixture remain excluded from the acceptance prefix (pre-existing gap /
  Stage P5 respectively).

## Why this shape (approach decision)

Three approaches were considered:

- **A — host-import lane** (mirror `crypto.subtle.digest`): `kali:rt` imports
  plus a host-side stateful URL/USP registry keyed by an opaque handle (the
  EventTarget model). Spec-correct via the `url` crate and browser-native, but
  it costs **five hand-mirrored registrations** (wasmtime + node + 2×`cmd_build`
  + 2×`harness`), new host statefulness, and boundary marshaling to return a
  growable array from `.getAll()`.
- **B — compile-time-parse hybrid, self-contained in-wasm** (**chosen**):
  literal constructor args are parsed at compile time with the already-vendored
  `url` / `urlencoding` crates; URL becomes a fixed struct of interned
  component string handles; USP is a runtime-mutable growable pair-store with
  synthetic-guest-fn methods. **Zero host glue** — the same wasm runs
  identically under `kali run` and in-browser, which is exactly what the
  eventual three-way `webBaselineSmoke` acceptance (`kali run` + browser +
  flipped build tests) requires.
- **C — full compile-time fold**: infeasible. `.set('beta', String(count))`
  carries a runtime value that `.get('beta')` must return, so USP
  fundamentally needs runtime state. Ruled out.

**Chosen: B.** It is self-contained, leans on the already-present `url` crate
for the hard parsing, and extends the proven `Repr::AbortHandle`
allowlist-at-choke-point discipline rather than introducing a new host
surface.

Two sub-decisions, both user-ratified:

1. **Constructor args: literal-only, fail closed** on runtime-computed args —
   consistent with every prior stage's default-deny on unproven provenance.
2. **`.toString()` percent-encoding: a synthetic guest fn** (`__percent_encode`),
   NOT a host helper — keeping the stage fully self-contained, which is the
   whole reason B was chosen. `.toString()` sits in an un-taken error branch in
   the fixture but must still *lower correctly* (whole-program compilation; a
   fail-closed `.toString()` would reject the module).

## Representation

Two new variants in `crates/kali_common/src/repr.rs`, both
compile-time-provenance-distinguished i64 handles (the `Repr::AbortHandle`
model — every position a handle can reach is allowlisted at the read site;
unproven flows keep the I64 default and every URL/USP operation on them fails
closed):

- **`Repr::UrlSearchParams`** — an i64 handle to a **growable pair-store**.
  Reuse the existing growable machinery (`emit/growable.rs`,
  `[len][cap][data_ptr]`, `ARRAY_HANDLE_TAG`), storing **string handles** as
  interleaved i64 elements `[k0, v0, k1, v1, …]`. `len` counts element slots;
  pair count = `len / 2`. This is the only mutable structure in the stage.
  (String handles are ordinary tagged i64s, so they live in a growable-i64
  store mechanically; USP methods are bespoke synthetic fns that interpret the
  slots as string handles — they never route through `__join_growable_i64`,
  which would print them as numbers.)
- **`Repr::Url`** — an i64 handle to a **fixed heap struct** of six 8-byte
  slots: five interned string handles (`href, origin, pathname, search, hash`)
  and one `Repr::UrlSearchParams` handle for `.searchParams`. Built once at
  construction; immutable thereafter. Component byte offsets are a fixed
  layout constant (documented at the emitter).

Neither handle is ever reclaimed this stage (small, bounded — matches the
abort-cell precedent). No GC (Kali is GC-less by design).

## Construction (declarator choke point — `emit/control_flow.rs`)

Recognized at the same declarator dispatch that handles `new AbortController()`
/ `new EventTarget()`:

- **`const q = new URLSearchParams(<literal>)`** — parse the literal with
  `form_urlencoded` (Rust, incl. `+`→space and percent-decode) at compile time;
  intern each decoded key/value into the `StringPool`; emit
  `emit_growable_alloc` + seed pushes of the interned handles. Bind the handle,
  record the name in a new `usp_locals` provenance set, set
  `scalar_repr(name) = Repr::UrlSearchParams`.
- **`const u = new URL(<literal>)`** — parse with the `url` crate at compile
  time; intern `href/origin/pathname/search/hash`; build the embedded USP from
  the parsed query (same path as above); allocate the fixed struct via
  `__alloc_global`, store the six slots; record in `url_locals`, set
  `scalar_repr(name) = Repr::Url`.

**Gates (all → default I64 / E5506):** non-literal / non-string-constant arg;
a shadowed `URL` / `URLSearchParams` global (5-namespace unshadowed check, as
abort); `let`/reassignment of the binding (mutation of the handle binding
itself is out of scope — deny, matching the abort const-only lane).

## Method dispatch (`emit_call` choke point — `emit/call.rs`)

Ordered branches, keyed on callee text + **proven receiver repr**, added
alongside the existing abort/event/growable branches:

**On a proven `Repr::UrlSearchParams` receiver** (synthetic guest fns over
`__streq` + growable scan/push, hand-emitted in `lower.rs` like `__streq`):

| Method | Lowering |
| --- | --- |
| `.append(k, v)` | push `k`, push `v` |
| `.set(k, v)` | scan pairs, remove every slot-pair whose key `__streq` k (compacting the store), then push `k`, `v` |
| `.get(k)` | scan; return first matching value handle, else the **null sentinel `0`** |
| `.getAll(k)` | scan; build a fresh growable string-array of matching values (`Repr::GrowableArrayI64` of string handles); `.length` reads its header |
| `.has(k)` | scan; return `1`/`0` |
| `.toString()` | `__usp_tostring`: join pairs as `k=v` with `&`, each component through `__percent_encode` (space→`+`, unreserved pass-through, else `%XX`) |

**On a proven `Repr::Url` receiver:** `.origin/.pathname/.search/.hash/.href`
→ the stored component string handle (a fixed-offset load); `.searchParams` →
the embedded USP handle. `url.searchParams.get('alpha')` therefore composes:
the member read yields a `Repr::UrlSearchParams` value, and the outer `.get`
dispatches through the USP branch.

**Any other method, computed member (`usp[expr]`), or unproven receiver →
E5506.**

## Value-escape discipline (the recurring lesson — 6 stages running)

The consistent failure across every prior stage: a denylist of sinks leaks
forever; only an **allowlist at the read choke point** closes a class by
construction. A `Repr::UrlSearchParams` / `Repr::Url` handle is admitted ONLY
in these positions:

1. as a recognized method receiver (the tables above);
2. the `.searchParams` chain (URL → USP → method);
3. a URL component read (`.origin/.pathname/.search/.hash/.href`).

**Everything else is denied at the identifier / member-read choke point**
(the `is_module_scope_abort_handle` pattern): raw print / `console.log`,
arithmetic, `===`/`!==` (other than the null-sentinel comparison the fixture
never takes — see below), template/`+` concat, array-element store,
object-field store, argument pass, `return`, and any capture into a
closure/module-scope binding. Each → E5506, never a silent miscompile.

**Only the URL/USP handle itself is escape-restricted.** Method *results* are
ordinary values that flow through the normal lanes: `.get`/`.toString`/URL
components → `Repr::String`, `.getAll` → `Repr::GrowableArrayI64`, `.has` →
Boolean `1`/`0`. So `query.get('alpha') !== '1'` is a plain string `===` and is
NOT denied — the restriction bites only when a `Repr::Url` /
`Repr::UrlSearchParams`-typed value reaches a non-allowlisted position.

The whole-stage review MUST enumerate store-sites and generic value-sinks for
both new handle classes (this is where the last four stages' whole-stage
reviews each caught a CRITICAL the per-task reviews missed).

## Error handling / soundness invariants

- **Default-deny** at both choke points: unproven constructor arg, unproven
  receiver, unsupported method, computed member, shadowed global, binding
  reassignment → `E5506 FEATURE_UNAVAILABLE`.
- **Null sentinel** for a `.get` miss is `0`, tag-distinct from every real
  string handle (bit 63 set), so `__streq`/`===` treat it as unequal to any
  string. The fixture never takes a `.get`-miss branch, but the behavior is
  implemented (not faked) per reject-don't-miscompile.
- **`.toString()` must lower** even though the fixture never reaches it
  (un-taken error branch): whole-program compilation means an E5506 there would
  reject the module. It is implemented correctly, not stubbed.
- **URL immutability:** any URL member write fails closed.

## Testing & gate

- **Acceptance** — extend `acceptance_web_baseline_prefix_matches_node_byte_for_byte`
  (`soundness_abort.rs`, or a sibling in a new `soundness_url.rs`) to include
  the `URLSearchParams`/`URL` block from `runtime_smoke.rs:454–463`, run
  byte-for-byte against node. The Event-type block stays excluded (pre-existing
  gap); the TextEncoder tail stays excluded (Stage P5).
- **Soundness pins** — new `crates/kali_cli/tests/soundness_url.rs`
  enumerating every fail-closed shape: non-literal ctor arg, shadowed global,
  binding reassignment, unsupported method, computed member, each escape sink
  (print / arithmetic / concat / array store / field store / arg pass / return
  / closure capture / module-scope capture), and URL member write. Each asserts
  `E5506`.
- **Positive pins** — per-method behavior (`get`/`getAll.length`/`has`/`set`
  replace-semantics/`append`/`toString` round-trip; URL component reads;
  `searchParams` composition), asserting real output not just build success.
- **Gate** — honest-red stage-base baseline (712 failing INSTANCES / 694 unique
  NAMES — the reconciled count; see `pr16-honest-repin-inventory.md`),
  **0 newly-red** at every task and at close-out, **double-enumerated with zero
  drift**, main cross-check `cargo test --workspace` diffed against a MAIN
  worktree (never against the fake-green `.worktrees/kali-main`).
- **Whole-stage adversarial review** — the 6th consecutive stage where this is
  mandatory. Fresh-probe re-verify EVERY fix claim on a freshly-built binary
  (fix reports are unreliable); attack the allowed side of each new choke point;
  enumerate store-sites + value-sinks; confirm allowlist-beats-denylist.

## Files touched (anticipated)

- `crates/kali_common/src/repr.rs` — two new `Repr` variants + doc.
- `crates/kali_types/src/repr_infer.rs` — seed `Repr::Url` /
  `Repr::UrlSearchParams` from the recognized constructors; shadow-guarded;
  propagate through `.searchParams` and the `const`-alias lane.
- `crates/kali_codegen/src/emit/control_flow.rs` — the two constructor
  declarator branches; `usp_locals` / `url_locals` provenance sets.
- `crates/kali_codegen/src/emit/call.rs` — USP + URL method dispatch branches;
  the value-escape denials.
- `crates/kali_codegen/src/emit/` (new `url.rs` sibling to `abort.rs`) —
  member-read recognizers + component-load / USP-scan helpers + the read-site
  allowlist gate.
- `crates/kali_codegen/src/lower.rs` — synthetic guest fns `__usp_set`,
  `__usp_get`, `__usp_getall`, `__usp_has`, `__usp_tostring`,
  `__percent_encode` (declared + bodies, alongside `__streq`).
- `crates/kali_codegen/src/intrinsics/host.rs` — constructor / member
  recognizers (compile-time predicates; no new host import).
- `crates/kali_cli/tests/soundness_url.rs` (new) + acceptance extension.

## Follow-ups (recorded, out of scope)

- URL mutation (`url.pathname = …`), USP iteration/`delete`/`sort`/`entries`,
  runtime-computed constructor args, `URL` with a base argument
  (`new URL(rel, base)`), IDNA/punycode host normalization.
- After P5 (TextEncoder) lands, the final byte-for-byte `webBaselineSmoke`
  acceptance runs the whole fixture three ways.
