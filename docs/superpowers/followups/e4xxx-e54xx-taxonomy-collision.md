# `E54xx` is documented but does not exist; the code it describes is `E4001`

Found while implementing Task 4 (document the `E4xxx` family) of the
blast-radius ranking plan. Not fixed here — recording it so it isn't lost.

## The collision

`specs/15-errors.md:62` (and the range-clarification bullets around it,
`specs/15-errors.md:83`) document `E54xx` as "the runtime/effect-semantics
side (for example a capability use denied during execution)."

But no `E54xx` constant exists anywhere in `crates/`. Confirmed by:

```
grep -rn "\bE54[0-9][0-9]\b" crates/ --include=*.rs   # zero hits
```

and by reading the full `E5xxx` constant table,
`crates/kali_error/src/_error_codes.rs:96`-`196` (`pub mod e5`): the numbers
present are `5000`-`5003`, `5100`-`5101`, `5200`, `5506`-`5511`, and the
higher `E5xxx` sub-modules for the rest of the checker-facing ranges. There is
no `54xx` block.

The concept `E54xx` describes — "a capability use denied during execution" —
is real, but it is implemented as `E4001` (`EFFECT_NOT_PERMITTED`,
`crates/kali_error/src/_error_codes.rs:81`, value `4001`), emitted from
`crates/kali_sandbox/src/diagnostics.rs:22,34` and referenced in
`crates/kali_runtime/src/host/enforce.rs:12`. `E4002`
(`API_CALL_NOT_PERMITTED`, `_error_codes.rs:82`) would be the same kind of
denial for host APIs rather than effects **if it were reachable** — by name
and numeric band, not by traced behaviour; see the dead-code note below.

So the spec's own two families disagree with the code on which numeric range
owns "an effect the sandbox refuses": the spec says `E54xx`, the code says
`E40xx`.

## Separate finding: `E4002` looks unreachable today

`grep -rn "API_CALL_NOT_PERMITTED" crates/ --include=*.rs` returns exactly
one hit: the constant's own definition at
`crates/kali_error/src/_error_codes.rs:82`. No call site in `crates/`
constructs a diagnostic with it — not even one shared with `E4001`'s
emitters. So as of this task, `E4002` appears to be dead code: a reserved
error code with no path that produces it. A maintainer should either wire up
an emitter for it (if "host API not permitted" is meant to be distinguished
from "effect not permitted" going forward) or remove the constant if the
distinction isn't needed. This task does not resolve that question — it
only records the finding — and the "same kind of denial as `E4001`" reading
used elsewhere in this document and in `verdict.rs`/`specs/15-errors.md` is
inference from the constant's name and band, conditioned on it becoming
reachable, not a claim about current behaviour.

## Why this is out of scope for Task 4

Task 4's brief was to give the real, reachable `E4xxx` codes a row in the
spec (they had none) and make `kali_blast_radius`'s classifier comments
truthful about what the family is. Reconciling the `E54xx`/`E4001`-`E4002`
collision is a different kind of change: it is a decision about which
numeric range an emitted error code should live in (renumber `E4001`/`E4002`
into `E54xx`, or update the spec text to stop claiming `E54xx` and describe
`E4001`/`E4002` instead), which changes emitted diagnostics or is a larger
spec rewrite either way. That is out of scope for a documentation task that
must not touch `is_documented_code`'s behavior or any emitted code.

## Consequence for the blast-radius classifier

`crates/kali_blast_radius/src/verdict.rs`'s `is_documented_code` treats all
of `E4xxx` as undocumented (so `classify` returns `FL_INTERNAL` for it). That
is correct for `E4003`/`E4201` (genuinely internal) but arguably wrong for
`E4001`, a traced policy denial (an honest refusal in the same sense as
`E5506`) that a stricter classifier might want to score as `FAIL_CLOSED`
rather than `FL_INTERNAL`. `E4002` has no emitter today (see above), so this
concern for it is conditional on it becoming reachable, not a present
misclassification.

This is left as-is (see the doc comment on `is_documented_code`) because no
register entry measured by this project (`docs/superpowers/followups/kali-silent-miscompile-register.md`)
exercises a sandbox-effect denial — confirmed by:

```
grep -in "capability\|E4001\|E4002\|sandbox polic" docs/superpowers/followups/kali-silent-miscompile-register.md
```

which finds no register row about an effect/capability denial. So no
recorded verdict depends on this today. If a future register entry adds a
case that trips `E4001` (or `E4002`, should it become reachable), this
classifier will misclassify it as `FL_INTERNAL` and should be revisited
alongside resolving the spec collision above.
