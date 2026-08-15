# The case runner has no escape for a literal `${` in fixture source

**Filed** 2026-08-15, from the blast-radius ranking's Task 9 (`oracle/tier4.toml`).
**Status:** worked around in one file; the proper fix is not done and is not urgent.

## What collides

`crates/kali_case_runner/src/expand.rs`'s `substitute` scans a `[source]` body
for `${`, takes everything up to the first `}` as a binding name, and hard-errors
on any name it cannot resolve:

```
unresolved placeholder `${1<2}` in "…console.log(`${1<2}`);…"
```

A JavaScript **template literal** uses the same three characters. So any fixture
source containing `` `${expr}` `` fails to expand, and the whole case file fails
with it — not the one case, the file.

The hard error itself is correct and should stay: `substitute` refuses to let an
unresolved `${…}` survive into a comparison, which is what stops a case from
asserting against a needle that can never match. The gap is that there is no way
to say "this `${` is data".

## Where it bit

The register's R-30 entry
(`docs/superpowers/followups/kali-silent-miscompile-register.md` §2, as of
`61c2d48ea9`) states its concat/template boundary with four fragments, one of
which is `` `${1<2}` `` → `true`. An oracle case must carry the register's repro
unreduced, so dropping the fragment was not available.

## The workaround used

`crates/kali_cli/tests/cases/oracle/tier4.toml` declares:

```toml
[constants]
"1<2" = "${1<2}"
```

`substitute` is **single-pass** over the bindings — a documented property, stated
in `model.rs`'s `referenced_placeholders` doc comment ("`substitute` is
single-pass over `file.constants`, so `A = "x"` / `B = "${A}"` leaves a literal
`${A}` in the output"). So `${1<2}` in a source body is replaced by the
four-character text `${1<2}`, the emitted `.js` carries the register's fragment
character for character, and the output is not re-scanned. The constant is
referenced, so `check_bindings_are_referenced` is satisfied.

A sibling idiom already exists in the corpus for the *other* half of this problem
— `dollar = "$"`, written `${dollar}{expr}` to emit a literal `${` — and is
pinned by `model_tests.rs::the_dollar_escape_idiom_counts_as_a_reference`. It
requires editing the fixture text (splitting `${` across a substitution), which a
verbatim-repro case cannot do. The self-referencing constant leaves the text
alone, which is why Task 9 used it.

## How it was verified

A green case would **not** have proved the workaround worked. Had the
substitution collapsed `${1<2}` to the text `1<2`, the emitted line would be
`` console.log(`1<2`) ``, which prints the string `1<2` on *both* engines and
still classifies `fixed` — the workaround failing would have looked exactly like
it working.

So it was checked directly at `61aa9043b3`: the `r30d` module-scope case's
`verdict` was temporarily set to `silent` to force a mismatch report, and node's
captured stdout was read out of it.

```
verdict mismatch for R-30: expected `silent`, measured `fixed`
  kali: exit Some(0) ... stdout "v=true\ntrue\nv=true\nv=true\n" stderr ""
  node: exit Some(0) ... stdout "v=true\ntrue\nv=true\nv=true\n" stderr ""
```

Line 2 is `true`, not `1<2`: the template hole reached node intact. The verdict
was then restored.

## Why a real escape form is the proper fix

The workaround is legible only because `tier4.toml` spends a header block
explaining it. It has three costs a fixture author should not have to carry:

- **One constant per distinct expression.** `` `${a}` `` and `` `${b}` `` need
  two entries; a fixture with five template holes needs five.
- **It reads as a puzzle.** `"1<2" = "${1<2}"` looks like a mistake until the
  single-pass property is known.
- **It depends on that property staying true.** If `substitute` ever became
  fixed-point or multi-pass — a reasonable-looking change — every such constant
  would start recursing or erroring, and the failure would land in a case file
  rather than in the module that changed.

The proper fix is a one-character escape in `substitute`: treat `$${` as a
literal `${` and skip it, exactly as the format's own `deny_unknown_fields`
strictness would suggest. It is a small, testable change (`expand.rs`, plus
`expand_tests.rs` for "an escaped `${` survives verbatim" and "an escaped
sequence is not scanned for a binding name"), and it would let a template-literal
fixture be written the way it reads.

## Why it was not done here

Task 9 is a measurement task. Its scope is authoring oracle cases and recording
verdicts; changing the substitution engine would put a production-code change
inside a measurement whose whole point is that the instrument was fixed while the
cases were written. The one production change that task did make — the
`observe` stream selector — was made because a case was reporting a **false
green** for a live defect, which is a correctness failure of the measurement.
A template literal that needs an awkward constant is an ergonomics problem, and
the fixture it affects is correct.
