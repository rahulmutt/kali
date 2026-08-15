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
(`API_CALL_NOT_PERMITTED`, `_error_codes.rs:82`) is the same kind of denial
for host APIs rather than effects.

So the spec's own two families disagree with the code on which numeric range
owns "an effect the sandbox refuses": the spec says `E54xx`, the code says
`E40xx`.

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
`E4001`/`E4002`: those are policy denials, i.e. honest refusals in the same
sense as `E5506`, and a stricter classifier might want to score them as
`FAIL_CLOSED` rather than `FL_INTERNAL`.

This is left as-is (see the doc comment on `is_documented_code`) because no
register entry measured by this project (`docs/superpowers/followups/kali-silent-miscompile-register.md`)
exercises a sandbox-effect denial — confirmed by:

```
grep -in "capability\|E4001\|E4002\|sandbox polic" docs/superpowers/followups/kali-silent-miscompile-register.md
```

which finds no register row about an effect/capability denial. So no
recorded verdict depends on this today. If a future register entry adds a
case that trips `E4001`/`E4002`, this classifier will misclassify it as
`FL_INTERNAL` and should be revisited alongside resolving the spec
collision above.
