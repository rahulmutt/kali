# throw-fallout Stage 1 — Runtime string equality (design)

**Date:** 2026-07-11 · **Branch:** `soundness-batch1-pra` · **Umbrella:**
`2026-07-11-throw-fallout-design.md` (Stage 1 of 7) · **Status:** approved design

## Problem

Runtime `==`/`!=`/`===`/`!==` on strings compiles to a raw `i64.eq` on the two
tagged handles (`crates/kali_codegen/src/emit/operators.rs:1556-1580`). This is
correct only by accident: equal string **literals** intern to the same
`(offset,len)` handle (`crates/kali_codegen/src/ctx.rs:213-225`), so handle
identity coincides with value equality for interned-vs-interned. Everything else
splits into two broken lanes:

1. **Silent miscompile (the 656-test bucket, #2/#3 in the denominator):**
   fresh-buffer strings that are *not* in the concat-taint seed set —
   enumeration keys (`Object.keys(o)[i]`), `Object.entries` keys, for-in keys
   read back through arrays — reach the `i64.eq` lane with a fresh handle and
   compare unequal to an interned literal of the same text.
   `Object.keys(o)[0] !== 'b'` is true even though the key prints `b`, so
   self-check `throw`s fire (honestly, post-Task-1) and 656 tests are red.
2. **Fail-closed reject:** operands tainted as runtime-fresh (concat `+`,
   `substring`, `join`, `process.argv[i]`) are rejected with E3200 at
   `operators.rs:1410` (`is_runtime_concat_string`). Honest but unsupported;
   blocks the deno-env / web-baseline slice of the corpus.

## Decision (maintainer-approved)

- **Lift + re-pin:** Stage 1 replaces *both* lanes with real content
  comparison — preempting the E3200 equality reject for both-string
  operands — and re-pins the
  main-green tests that assert that reject (same honest re-pin move as
  Stage 0's 43).
- **Approach A:** a `__streq` synthetic wasm function; ALL both-string equality
  routes through it. (Rejected: B host-import compare — touches the four
  hand-mirrored browser import lists and pays a host crossing per compare;
  C runtime interning — needs an in-memory intern table, kills zero-copy
  `__substring`, conflicts with arena reclamation.)

## Scope

**In:** `==`, `!=`, `===`, `!==` where **both** operands are provably
string-valued (repr proof on both sides). For two strings `==` and `===` are
the same JS relation; both become content equality. Byte-wise UTF-8 comparison
is exact for equality (two strings are equal iff their UTF-8 encodings are
equal); the UTF-16 divergence hazard applies only to relational *ordering*,
which stays out.

**Out (unchanged):**

- Relational `<  <=  >  >=` on strings: existing static-ASCII fold
  (`intrinsics/string.rs:41`) + reject lane stay as-is.
- String truthiness / logical-operand rejects (E3200 family) stay.
- `Object.is`, `switch` dispatch: not in the bucket; untouched. If stage-start
  triage finds bucket tests using them, those entries are reassigned, not
  absorbed.
- **Mixed-type equality** (`s == 5`): today one-string-one-number equality falls
  through to handle-vs-number `i64.eq` → always false — accidentally
  node-correct for `===`, silently WRONG for `"5" == 5` (node coerces: true).
  Pre-existing fail-open, not in the 656, and touching it risks flipping green
  tests. **Recorded as follow-up F-Stage1-1** (recommend fail-closing
  one-side-provably-string equality in a later stage), not absorbed here.

## Mechanism

### `__streq` synthetic

Follows the `__substring`/`__join` synthetic pattern exactly:

- Name added to `SYNTHETIC_FUNCTIONS` (`crates/kali_codegen/src/lower.rs:37-45`).
- Signature plan `(i64, i64) -> i64` registered alongside the `__substring` plan
  (~`lower.rs:277`); body wired in the dispatch match (`lower.rs:716-736`);
  `streq_fn_index()` accessor added in `crates/kali_codegen/src/emitter.rs`
  beside `substring_fn_index`.
- Body (`emit_streq_body`), in order:
  1. **Handle-identity fast path:** raw handles equal → return 1. Keeps
     interned-vs-interned (and aliased same-handle) compares cheap.
  2. **Length pre-check:** low-32 lengths (`raw & 0xffff_ffff`) differ →
     return 0.
  3. **Byte loop:** decode both offsets exactly as the runtime does
     (`(raw >> 32) & 0x7fff_ffff` — mask, not plain shift; mirror of
     `crates/kali_runtime/src/host/memory.rs:27-56`), then `I64Load8U` compare
     byte-by-byte using the `Loop`/`If`/`Br` idiom from `emit_join_body`
     (`lower.rs:3540-3615`). First mismatch → 0; loop completion → 1.
     Zero-length strings never enter the loop → 1, correct for
     empty-vs-empty regardless of offsets.
- Byte-at-a-time on purpose (simplicity/correctness); word-width compare is a
  later optimization only if a benchmark ever cares.

### Codegen equality lane (`operators.rs`)

- New arm ahead of the numeric emission: `is_equality` **and**
  `is_string_valued(left) && is_string_valued(right)` → emit both operands as
  string handles, `Call(streq_fn_index())`; for `!=`/`!==` append negation
  (`I64Eqz` + extend) matching the existing `!=` idiom.
- The equality branch of the reject lane (`is_runtime_concat_string` check at
  `operators.rs:1408-1427`) is **preempted**: both-string equality returns from
  the new arm and never reaches it. The reject itself is **retained** as the
  fail-closed backstop for the residue — a tainted string against a NON-string
  operand (e.g. `("a"+s) == 5`), which must keep rejecting per Error handling
  below. Handle-identity `i64.eq` on strings ceases to exist as a semantics; it
  survives only as the fast path *inside* `__streq`. Content equality is
  defined at one choke point — no provenance chasing (the Spec-4a structural
  default-deny lesson).
- Order/arith string rejects in the same lane are untouched.

### Types-side mirror (`crates/kali_types/src/resolve/expression.rs`)

Code-reading during planning established that `kali_types` has **no
equality-specific gate**: the E3200 equality reject lives only in codegen
(`operators.rs:1410`); the types-side rejecters cover `+`, logical operands,
truthiness, `.length`, and stores — none fires on an equality operand. The
mirror obligation for this stage is therefore **verification, not code
change**: fixtures per operand form (literal, identifier repr, ternary,
`substring`/`join` call, `process.argv` element, computed string-array element)
prove admitted forms compile-and-run and non-admitted forms still diagnose. The
hand-mirrored oracle pair — codegen `is_string_valued` (`operators.rs:808-887`)
↔ types `operand_repr_is_string` (`expression.rs:889-937`) — is untouched.

## Data flow (end to end)

`Object.keys(o)` materializes fresh key strings → array element read yields a
fresh handle → `keys[0] !== 'b'` classifies both operands string-valued on both
oracle sides → codegen emits `Call __streq` + negation → `__streq` misses the
identity fast path, passes the length check, byte-compares `b` vs `b` → 1 →
negated → `false` → the self-check `throw` does not fire → test green, output
byte-for-byte with node.

## Error handling

- Both-string proven: never rejected, never traps — `__streq` is total on valid
  handles.
- Exactly-one-side string proven, or string-valued but unproven repr: unchanged
  behavior (mixed lane as today; unproven forms keep their existing E3200
  rejects). No new diagnostics introduced; the equality taint-reject no longer
  fires for both-string operands (preempted) but still guards the mixed-tainted
  residue.

## Re-pin policy

Main-green tests pinning the E3200 equality reject (Spec-1 concat-taint corpus
plus any substring/join/argv equality-reject pins; enumerated by grep over the
equality-reject fixtures at stage start) are re-pinned to assert the correct
comparison result. Each re-pin's expectation is derived by running the fixture
under **node**, never from whatever makes the test pass. Re-pins land within
the stage so the checkpoint never shows a main-green test red.

## Testing

New reproducer coverage (integration tests, following existing
`kali_cli/tests` conventions):

- `Object.keys` key vs interned literal — the headline miscompile, now true.
- Concat (`+`), `substring`, `join`, `argv` operands vs literals — previously
  rejected, now compared.
- `!=`/`!==` negation; equal-length-different-bytes; different-length;
  empty-vs-empty; empty-vs-nonempty; interned-vs-interned still true.
- **Re-mask check (Invariant 3):** a fixture whose self-check `throw` fires on
  a deliberately wrong comparison must still fail — the fix must not
  re-silence throws.
- **Oracle-desync reproducers:** per operand form, a fixture proving the types
  gate and codegen agree (admitted forms compile and run; non-admitted forms
  diagnose, not miscompile).
- Node parity byte-for-byte on every touched fixture.

## Gate mechanics

- Verdict: `cargo test --workspace` on the branch; enumeration:
  `cargo test --workspace --no-fail-fast` (plain run fail-fasts at the first
  failing binary).
- Baseline: the `main` worktree at `/workspace/.worktrees/kali-main`
  (machine-local path; 0 failures).
- Pass = the 977 denominator strictly shrank **and** no main-green test is red
  at the checkpoint.
- Expected drain ≈ the 656 bucket minus entries whose stage-start triage
  reassigns them (e.g. `for_await` enumeration fixtures also needing Stage 7
  async machinery); the triage names which entries are expected to remain red
  and why.
- The drain is snapshotted into
  `docs/superpowers/followups/throw-fallout-stage0-denominator.md` (or an
  adjacent progress file) per program convention.

## Risks

1. **Hand-mirror desync** — types gate admits what codegen can't emit (or
   vice-versa) → fail-open. Mitigation: both classification changes in one
   reviewed unit + the per-operand-form desync reproducers.
2. **Bucket overlap** — some of the 656 stay red for non-equality reasons
   (async wrappers, delete-reinsert). Expected; they drain in their own
   stages. The gate demands strict shrinkage, not bucket-complete drain.
3. **Perf** — a call + byte loop per compare. Keys are short; fast paths cover
   the hot interned case. Noted, not gated.
4. **Handle decode divergence** — `__streq` must mask the offset with
   `0x7fff_ffff` exactly as `read_guest_string_handle` does, or high-offset
   strings compare garbage.

## Follow-up inventory (recorded, not absorbed)

- **F-Stage1-1:** mixed-type equality (`"5" == 5` → node-true, kali-false;
  `s === 5` accidentally correct). Recommend fail-closing
  one-side-provably-string equality in a later stage, then deciding whether
  coercion is ever in scope.

## Definition of done

`__streq` shipped and routed as above; E3200 equality reject deleted with
types-side mirror in lockstep; reject-pinning tests re-pinned from node-derived
expectations; reproducers + re-mask + desync tests green; gate passes (strict
denominator shrink, zero main-green regressions); drain snapshotted; follow-up
F-Stage1-1 recorded.
