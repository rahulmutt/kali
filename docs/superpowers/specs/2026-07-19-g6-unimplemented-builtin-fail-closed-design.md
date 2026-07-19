# G6 — recognized-but-unimplemented builtins fail closed (Group 2, item 4)

- **Date:** 2026-07-19
- **Branch:** `soundness-batch1-pra`
- **Register item:** §6 Group 2, item 4. Members: **R-19, R-20, R-15, R-25, R-24**.
- **Predecessor:** Wave 0 (semantic-core close-out) COMPLETE (`4ff523162..2727252f6`).
- **Direction:** *refuse, don't implement.* Convert silent wrong-value miscompiles into
  honest `E5506` errors. Actually implementing these builtins is Group 4 / architectural and
  explicitly out of scope.

---

## 1. Problem

Five recognized JS builtin operations silently lower to a constant `0`/empty result at exit 0,
with no diagnostic (or an E3100 *warning* that does not stop the build), instead of failing
closed. The user then consumes a plausible-looking `0` as if the call succeeded.

The register (§3 G6) hypothesised **one choke point, "unknown builtin ⇒ 0."** The cluster
experiment mandated by the register was run (2026-07-19) and **half-falsified** that hypothesis:

| call | kali | node | verdict |
|---|---|---|---|
| `String(x)` | `r=0` | `r=42` | silent-0 |
| `x.toString()` | `r=0` | `r=42` | silent-0 |
| `JSON.stringify(o)` | `r=0` | `r={"f":1}` | silent-0 |
| `"a,b,c".split(",")` (runtime-receiver, concat) | `len=0` | `len=3` | silent-0 |
| `[...a]` | `len=1 e0=0` | `len=2 e0=1` | silent-0 |
| `Number(x)` | **E3100** | `r=42` | already honest |
| `parseFloat(..)` | **E5506** | `r=1.5` | already honest |
| `Frobnicate(3)` | **E3100** | — | already honest |

**Key correction to the register:** the truly-*unknown* builtins (`Number`, `Frobnicate`)
already fail closed — they are rejected at resolve as undefined identifiers. The five members
are **recognized-but-stubbed**: their lowerings deliberately emit `0`. So this is not "unknown
builtin ⇒ fail closed at one site." The source trace found **three independent mechanisms**,
each with its own site and its own working lane to preserve.

---

## 2. Source trace (the three streams)

### Stream A — shared call fallback (R-19 `String`/`toString`, R-20 `JSON.stringify`, R-15 runtime-`split`)

`emit_call`'s terminal has two adjacent fallbacks in `crates/kali_codegen/src/emit/call.rs`:

```
call.rs:3242-3261   Fix-5 default-DENY   → E5506   (call through a first-class fn value)   [fail-closed]
call.rs:3263-3267   warning placeholder  → I64Const(0) + E3100 *warning*                   [silent-0]
```

`String(x)`, `x.toString()`, `JSON.stringify(x)` (and a runtime-receiver `split` whose static
recognizers all decline) **slip past the E5506 deny and land on the warning+0**, because
`call_target_keeps_placeholder_lowering` (`call.rs:3314`) *admits* them:

- bare free name not program-bound (`String`) → `call.rs:3370-3372` returns `true`;
- member/computed property not bound to a program function (`toString`, `stringify`) →
  falls through to `call.rs:3397` `true`.

That predicate is already a **positive allowlist** (introduced by Fix-5 for first-class
function values). Its *policy* is the defect: "an unimplemented host surface keeps the
warning+0 placeholder." That policy is the silent-0 sink.

Static folds sit **upstream** of this fallback and are unaffected:
- split static-ASCII array-fold: `call.rs:1040` `resolve_static_string_split_call`
  (recognizer `intrinsics/string.rs:978`), plus the literal-receiver fail-closed guard
  `call.rs:1195` and the types-side `E5` at `kali_types/.../string.rs:964`.
- static `toString`/`valueOf` folds: `call.rs:893` `resolve_static_string_identity_call`,
  `call.rs:1273` `resolve_static_array_to_string_call`.

### Stream B — array spread `[...a]` (R-25)

Fully independent — never touches `emit_call`. `is_array_literal` (`intrinsics/array.rs:5`,
`node.kind == Value && node.text.is_none() && !is_object_literal`) mis-classifies `[...a]` (a
textless `Value` whose single child is a `Value("spread")` node) as a **1-element array
literal**. The static length/index folds then read that shape:
- length: `operators.rs:334-336` → `I64Const(children.len())` = `1`.
- element 0: `operators.rs:436-443` / `call.rs:4867-4875` → emits the un-expanded spread node,
  which bottoms out at a zero placeholder (`emit/literal.rs:38`).

There is no "expand-if-foldable-else-fail" guard: the spread is silently swallowed. Object
spread `{...o}` by contrast **already** fails closed `E5506`.

### Stream C — `Object.freeze` / `Object.isFrozen` (R-24)

`is_object_freeze_call` (`intrinsics/object.rs:93`) is modeled as an **identity passthrough**
and consumed across ~15 sites (`object.rs`, `collections.rs`, `call.rs`, `host.rs`,
`literal.rs`, `math.rs`). No write barrier exists, and `Object.isFrozen` returns `0`.

- Silent miscompile (bind-first): `const o={x:1}; Object.freeze(o); o.x=99` → kali `x=99`,
  `isFrozen=0`; node `x=1`, `isFrozen=true`.
- Load-bearing passthrough that MUST be preserved: `Object.freeze(Math.round)(4.6)` → `5`
  (Stage-C intrinsic-hardening); frozen-array-from-call; folded-object-literal lanes.

Register R-24 caveat: verify with the **bind-first** probe. The folding probe
(`const o=Object.freeze({x:1}); o.x=99`) *hides* the defect (the literal folds and the write
is dropped for unrelated reasons, so kali and node agree while the defect is live).

---

## 3. Design

### 3.1 Stream A — allowlist-invert the terminal call fallback

**Decision (user-ratified): allowlist-invert, not narrow-denylist.** A denylist of intrinsic
names (`String`, `JSON.stringify`, `toString`) would leak on the next unimplemented builtin —
the shape this repo has been burned by ~8 times (register §6 Group 3 note; the Spec-4a for-in
key class took 6 denylist rounds before a structural default-deny closed it).

- The terminal fallback at `call.rs:3263-3267` becomes **fail-closed `E5506` by default**.
- The current `call_target_keeps_placeholder_lowering == true` population is split by a
  **positive keep-warn+0 allowlist** — only genuinely fail-soft surfaces stay on warn+0:
  1. **Deferred-registration surfaces** — the existing carve-out (`call.rs:3336`,
     `is_deferred_registration_surface && scheduling_call_args_provably_safe`). These *rely*
     on warn+0 to drop provably non-capturing callbacks (`setTimeout(cb,0)`,
     `addEventListener`); they must keep it.
  2. **Host fail-soft no-op surfaces** the acceptance/golden suite proves must stay soft —
     enumerated explicitly (see convergence below).
- Everything else → `E5506`. `String`/`JSON.stringify`/`toString`/runtime-`split` are not on
  the allowlist, so they fail closed **by construction**, and so does the next unimplemented
  builtin. No denylist to leak.

**Convergence procedure (the real cost of Stream A).** An allowlist at a choke point refuses
programs that currently compile, turning currently-green fixtures red — the *correct*
direction (refusing beats lying), but it must be budgeted (register §6 Group 3). The
keep-warn+0 allowlist is discovered against the gate:

1. Flip the fallback to fail-closed with only carve-out (1) admitted.
2. Run the full-workspace gate against a `main` worktree.
3. Triage every newly-red:
   - **legitimate fail-soft** (a host side-effect no-op the acceptance/golden path needs to
     continue past) ⇒ add its surface to allowlist entry (2), with a comment naming the fixture
     that requires it;
   - **silent-0 value-consumer** (the call's `0` was being consumed as data) ⇒ it correctly
     became honest; pin it (silent-0 → E5506) and leave it denied.
4. Repeat until newly-red is fully accounted for (0 unattributed).

**Preserve:** split static-ASCII lane (`string_split_static_ascii.rs`, 10 tests) and the static
`toString`/`valueOf` folds — all upstream of the fallback, untouched.

### 3.2 Stream B — array spread fails closed

- During array-literal classification / the length+index static folds, detect that a child is
  a spread node (`Value("spread")`) and **fail closed `E5506`** instead of counting it as one
  element. Mirror the object-spread `{...o}` E5506 that already exists.
- No static-fold expansion is added (that would be a capability, out of scope). A future
  follow-up may fold statically-foldable spreads; item 4 is fail-closed only.
- **Preserve:** ordinary array literals with no spread child keep folding.

### 3.3 Stream C — `Object.freeze`/`Object.isFrozen` fail closed, narrowly

- Fail closed `E5506` only for `Object.freeze(o)` / `Object.isFrozen(o)` on a **program-bound
  object receiver** — the write-barrier-needed case that is silently miscompiling.
- **Preserve** the identity-passthrough lanes: `Object.freeze(<intrinsic>)` (Math.round etc.),
  frozen-array-from-call, folded-object-literal. The implementer must map which of the ~15
  `is_object_freeze_call` sites are passthrough (keep) vs. the mutation-barrier gap
  (fail closed).
- **Verify with the bind-first probe**, never the folding one.
- **Risk / escape hatch:** Stream C is the most woven-through. If implementation shows it is
  not a contained fix, it splits to its own plan — it is register/ledger **item 8**, distinct
  from item 4, and item 4 does not depend on it.

### 3.4 Fail-closed contract

All three streams emit **`E5506` (FEATURE_UNAVAILABLE)**, house-style message
("`<op>` is unavailable unless … ; use explicit literals or the later compatibility path"),
matching `parseFloat` / substring / first-class-value. **Not** `E3100` — that code is reserved
for genuinely-unrecognized identifiers (`Number`, `Frobnicate`), which already fail closed via
resolve.

---

## 4. Scope

**In:** R-19 (`String`/`toString`), R-20 (`JSON.stringify`), R-15 (runtime-`split` fallback
miss), R-25 (array spread), R-24 (`Object.freeze`/`isFrozen`, narrow).

**Out:**
- Implementing any of these builtins (Group 4 / architectural).
- Static-fold *expansion* for spread or `String(literal)` (capability adds; possible
  follow-ups).
- The split-in-concat number-flow corruption is a *separate* defect (split itself works bare:
  `"a,b,c".split(",").length` → `3`); this spec only fail-closes split's genuinely-unproven
  runtime-receiver miss and does not touch the concat lane.
- R-24's full write-barrier implementation (item 4 fails closed per register item-8 guidance;
  a real barrier is a later capability).

---

## 5. Gate, baseline, and tests

- **Gate:** `cargo test --workspace`, diffed against a **`main` worktree** — never
  `.worktrees/kali-main` (fake-green, 8401/0). See [[ci-gate-vs-poisoned-baseline]]. Honest-red
  base ≈ **712** (P3 re-measure; re-measure at this stage's base before starting).
- **New pins** (a new `soundness_unimplemented_builtins.rs`, plus additions to existing
  `soundness_*.rs`):
  - **flip pins** — each member RED (silent-0) → GREEN (E5506 fail-closed): `String(x)`,
    `(42).toString()`, `JSON.stringify(o)`, runtime-`split` concat miss, `[...a]`,
    bind-first `Object.freeze`+`isFrozen`.
  - **preserve pins** — must stay GREEN: `"abc".split("")[0]` and the 10
    `string_split_static_ascii` tests; `Object.freeze(Math.round)(4.6)` → `5`; object-spread
    `{...o}` still E5506; ordinary array literals still fold; static `toString` folds.
- **Newly-red budget:** every newly-red must be attributable — either a legitimate fail-soft
  admitted to the Stream-A allowlist (with a naming comment) or a silent-0-consumer that
  correctly became honest (pinned). **0 unattributed newly-red.**
- 6/6 CLBG goldens + `acceptance_web_baseline_prefix_matches_node_byte_for_byte` byte-for-byte
  throughout. `cargo fmt` + `clippy` clean.

---

## 6. Sequencing & review

- **B → C → A.** B and C are independent leaf fixes (small, low blast radius). A is the
  substantive one (allowlist-invert + gate-driven convergence) and may warrant its own PR if
  the churn is large; C may split to its own plan if it proves non-contained.
- **Whole-stream adversarial review at the end**, on opus, fresh-probe re-verification of every
  fix claim on a freshly-built binary. The repo's ~8× lesson: per-task review + full gate both
  miss fail-opens in an unexercised class; only a hand-written adversarial sweep finds them
  (e.g. Fix-5's gate passed at 22 newly-red while `arr[0]()`, `{g:f}`, `(c?f:g)()` were still
  silently `0`).

---

## 7. Risks

1. **Stream A blast radius (primary).** Allowlist-invert flips every host free-name call that
   currently warns+0 into E5506. Some acceptance/golden paths may rely on fail-soft
   continuation; the convergence procedure (§3.1) is how they get admitted, but the size of the
   allowlist is unknown until the gate runs. If it balloons past a contained fix, A is really
   Group-3-scale and gets its own budgeted project.
2. **Stream C weave.** `is_object_freeze_call` at ~15 sites; a fail-closed edit that is too
   broad regresses the intrinsic-hardening lane. Narrow to program-bound object receivers;
   split to its own plan if not contained.
3. **Oracle mirroring.** kali_types predicates and codegen oracles are hand-mirrored
   (recurring repo hazard). Any new fail-closed arm needs the matching side or it fails open on
   the other. Prefer structural changes that make the two impossible to desynchronize.
4. **Register unreliability.** Wave 0 found R-08/R-30 materially wrong multiple times and R-34
   missing. Treat every register claim used here as a hypothesis verified on a fresh binary,
   not as fact (this spec already re-verified all five members + the two controls).
