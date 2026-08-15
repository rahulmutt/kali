# The blast-radius corpus

The population for a frequency counter that asks how often each register
defect's triggering construct occurs in programs kali targets.

Two strata, reported separately. Accept rates are **never pooled**; construct
counts are reported per stratum **as well as** pooled. Both rules, and the
different reasons behind them, are in "How the two strata are reported" below.

- `anchor/` — the six CLBG programs and the `imperative_core_runtime.rs`
  programs, extracted verbatim from their vendored fixtures and inline Rust
  string literals. 137 programs. These are programs the project already
  committed to compiling, each with an end-to-end design behind it. Provenance
  for all 137 is at `../anchor-provenance.json`; the extractor and an
  independent rustc-based verifier are at `../extract_anchor_corpus.py` and
  `../verify_anchor_extraction.py`.
- `extension/` — 40 programs curated for this measurement, each a complete
  runnable program doing a plausible job: text transformations, numeric
  simulation steps, data reshaping, encoders and checksums, and argv-driven
  utilities.

## The curation rule

A program earns its place because it is what someone would plausibly write to
do a job kali targets. **Never because kali compiles it.**

This is load-bearing. If curation filtered on acceptance, the corpus would
exclude exactly the constructs the SILENT register entries trigger on, every
reachable frequency would be measured over a population selected for already
working, and the scores would be circular. Curation is independent of
measurement; reachability is applied afterwards and reported separately.

The counterpart rule binds just as hard: **programs are not contrived around
the predicates.** `../predicates.json` was read to check that each countable
predicate has a realistic chance of appearing somewhere, and the programs were
then written to do jobs. Where a predicate would only ever appear in a
contrived program, that is recorded below as a finding about the predicate —
no program was manufactured to feed it.

## How the two strata are reported

**Accept rates: never pooled.** The anchor's accept rate is uninformative: it
is fixed by which tests happen to exist, not by anything the ranking measures.
Pooling it with the extension's would destroy the only informative number.

The anchor is **not** accepted at ~100% "by construction", and an earlier
draft of this file said so wrongly. The measured anchor accept rate is
**124/137 = 90.5%**. 13 of the 131 `imperative_core_runtime.rs` programs come
from `run_js_expect_failure` call sites — the suite commits kali to
*rejecting* them (E3200 / E5506 gates). Dropping them would have been curation
by acceptance, which the curation rule above forbids in load-bearing terms, so
they stay in the corpus and the anchor rate is 90.5%. See the design spec
§4.1, amended 2026-08-15 (Task 11), for the full correction.

**Construct counts: per stratum as well as pooled.** Counts are reported both
ways (coordinator ruling, 2026-08-15, recorded in spec §4.1). The anchor is 131
micro-snippets plus 6 real programs — the 6 are 4.4% of anchor programs but
56.7% of anchor bytes, and the non-CLBG median is 52 bytes. A pooled count
alone would be dominated by the anchor's shape and would hide which stratum a
frequency came from.

## Node executability

All 40 programs in `extension/` run clean under `node <file>` (exit 0, node
v26.7.0), verified program by program. A program node refuses measures
nothing.

**One anchor exemption.** `anchor/clbg_mandelbrot.js` cannot run under node:
it calls the kali host API `Kali.writeStdoutBytes`, which node does not
provide. Its extraction was verified by byte comparison against its vendored
fixture and by `node --check` instead of by execution. "Every program runs
under node" is a claim about `extension/`, not about the anchor.

## The dialect this population is written in

A frequency is only as general as the population behind it, so the population's
shape is stated here rather than left for a reader to discover. The 40
extension programs are written in the project's **imperative-core dialect**:
functions, `const`/`let`, C-style and `for...of` loops, arrays, plain objects,
and string work done with index loops. Verified by search across all 40 files,
these features appear **zero** times:

- regular expressions (no literal and no `RegExp`), and no `.match`/`.replace`
- destructuring, in declarations or parameters
- template literals (the backticks in `markdown_toc.js` are inside string
  literals — they are markdown fence samples, not template syntax)
- shorthand object properties (`{ rows }`); every property is written
  `{ rows: rows }`
- `.includes(...)`, `??`, `Map`, `Set`, `class`, `async`/`await`, `try`/`catch`

**This is not the result of steering by acceptance.** Nothing here was written
or withheld because of what kali compiles; kali was never run during curation.
The counter-evidence is in the corpus itself: it uses arrow functions, spread,
default parameters, `for...of` and optional call (`hooks.onStart?.(...)`) —
exactly the constructs a young compiler is most likely to reject, and the first
things anyone steering by acceptance would have dropped.

**It does bias some counts, and that must be read into the results.** Eight of
the jobs — CSV parsing, tokenising prose, slugifying headings, scanning
`{{...}}` placeholders, identifier casing, query-string handling, semver
parsing and INI parsing — are jobs where a JavaScript author reaches for a
regex first. All eight hand-roll character loops instead. Consequently:

- **R-13** (computed member, non-literal key), **R-14** (member read on a call
  result), **R-16** (string method result in concat position) and **R-19**
  (string conversion) are biased **upward** relative to a population that used
  regexes for the same jobs: index loops spell in `s[i]`, `.charAt`, `.slice`
  and `+` what a regex spells in one call.
- `.indexOf(x) >= 0` appears in 10 of the 40 files and `.includes(x)` in none,
  which is the same bias in miniature.
- **R-08**'s `??` lane is unsampled here. R-08 is still reached, through
  `=== null` / `=== undefined` / numeric-literal comparisons, but a count for
  it carries no evidence about nullish coalescing.

Nothing in this section is a reason to change the corpus, which is frozen. It
is a reason to describe the ranking's population precisely: *programs in the
imperative-core dialect kali targets*, not *JavaScript in general*.

### Counts that rest on a single site or a single idiom

Three counts are thin enough that a reader should know it before treating them
as frequencies:

- **R-47** (`for...of` over a `let` array binding) has exactly **one** site in
  the corpus: `extension/diff_lines.js:76-77`, where `let changes = []` is
  overwritten on the next line. The predicate needs `let` *and* an
  array-literal initialiser; the equally natural `const changes = diff(...)`
  would take the count to zero. One site, one idiom.
- **R-30** rests on one site, `semver_compare.js:66`
  (`console.log(satisfies("1.2.10", "^1.2.3"));`), and **R-31** on one,
  `inventory_ledger.js:68` (`console.log(lowStock);`). A third site of the same
  kind, `rle_codec.js:57`, carries R-16 and R-14. All three print something the
  program had already printed in another form. The idiom is natural — programs
  do print a bare answer next to a labelled one — but these are incidental
  sites, not load-bearing ones, and R-30 and R-31 would be zero without them.
- **R-11** (bitwise compound assignment) is **inflated** by six
  declare-then-compound-assign sites that spell one expression as two
  statements: `permission_flags.js:27-29`, `:33-35`, `:39-41` and
  `crc32_checksum.js:28-29`, `:43-45`. Inflated, not created: R-11 is still
  reached by the wholly natural `c >>>= 1` (`crc32_checksum.js:15`) and
  `accumulator <<= 6` (`base64_encode.js:34`), so removing the six would lower
  the count without zeroing it.

## Findings about predicates, not about the corpus

Five countable predicates in `../predicates.json` have no occurrence in the
extension. In each case the absence is a fact about the predicate's triggering
construct, not a gap in curation, and **no program was manufactured to feed
them**.

The five are not alike, and the difference decides how their zeros may be
published. **R-29 is structurally impossible**: its construct and this corpus's
runnability requirement are mutually exclusive, so its zero is published as
*structurally uncountable in this corpus*, never as frequency 0. The other four
are **rare, not impossible** — a program containing them could run — so their
zeros are ordinary unsampled zeros over this population, and must not inherit
R-29's framing.

- **R-15** (`staticSplitElementInConcatPosition`) fires only when the receiver,
  the separator and the index of a `.split(...)` element read are *all*
  literals, so the whole access folds statically — and then it is concatenated.
  A person writing that would write the folded string. The construct survives
  as a compiler residual, not as something anyone types.
- **R-18** (`stringLiteralLogicalOperand`) needs a string *literal* as the left
  operand of `&&` / `||`. A literal left operand makes the operator's outcome
  constant, so the expression is dead by construction. Real code puts the
  literal on the right (`name || "anonymous"`), which the predicate correctly
  does not match.
- **R-28** (`negativeZeroLiteral`) needs a source-level `-0`. Programs that
  mean zero write `0`; `-0` appears in practice only in code written to probe
  negative-zero semantics, which is a test, not a job.
- **R-29** (`assignmentToConstBinding`) is a `TypeError` at run time in
  JavaScript. No program that runs clean can execute one, so it can only be
  reached in a deliberately dead branch — i.e. only in a contrived program.
  This is the sharpest of the five: the predicate's construct and the corpus's
  runnability requirement are mutually exclusive.
- **R-48** (`arrayStoreIntoScalarObjectField`) needs an object field
  initialized with a numeric literal and later assigned an array — a
  representation change mid-life. This one is **uncommon, not impossible**, and
  it is deliberately not grouped with R-29: real code does write it, for
  instance a counter field lazily promoted to the list it was counting. A
  program that knows it wants an array field initializes it to `[]`, and one
  that wants a sentinel uses `null`, so none of these 40 jobs produced the
  shape — but a larger corpus of the same dialect plausibly would.

Four of the five are reported as zero-count in the extension stratum, meaning
"none of these 40 jobs produced the construct". R-29's zero is not a count at
all, for the reason above. Neither is an instruction to add programs: the
corpus is frozen.

## The freeze

`manifest.json` is committed before the counter runs for record. Neither the
corpus nor `../predicates.json` may be adjusted after scores are visible; any
later change must be a separate, explicitly-justified commit that says why.
The published ranking carries the corpus hash so a reader can tell exactly
what was measured. See the design spec §4.3.

The frozen values are **177 files** (137 anchor + 40 extension) and

```
corpus_hash = ca6f53339feb61b1ad988f5075c2648fd95a96b1796d67bcf2cd3af69090660f
```

and they are pinned mechanically, not just written down here:
`crates/kali_blast_radius/src/manifest_tests.rs` asserts both, so quietly
dropping programs and regenerating the manifest fails the suite rather than
staying green. Changing either constant is a deliberate act that must arrive
with its justification.
