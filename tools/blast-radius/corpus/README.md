# The blast-radius corpus

The population for a frequency counter that asks how often each register
defect's triggering construct occurs in programs kali targets.

Two strata, **never pooled** — neither accept rates nor construct counts.

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

## Why the strata are never pooled

**Accept rates.** The anchor's accept rate is uninformative: it is fixed by
which tests happen to exist, not by anything the ranking measures. Pooling it
with the extension's would destroy the only informative number.

The anchor is **not** accepted at ~100% "by construction", and an earlier
draft of this file said so wrongly. The measured anchor accept rate is
**124/137 = 90.5%**. 13 of the 131 `imperative_core_runtime.rs` programs come
from `run_js_expect_failure` call sites — the suite commits kali to
*rejecting* them (E3200 / E5506 gates). Dropping them would have been curation
by acceptance, which the curation rule above forbids in load-bearing terms, so
they stay in the corpus and the anchor rate is 90.5%. See the design spec
§4.1, amended 2026-08-15 (Task 11), for the full correction.

**Construct counts.** Counts are reported per stratum as well as pooled
(coordinator ruling, 2026-08-15, recorded in spec §4.1). The anchor is 131
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

## Findings about predicates, not about the corpus

Five countable predicates in `../predicates.json` have no plausible occurrence
in the extension. In each case the absence is a fact about the predicate's
triggering construct, not a gap in curation, and **no program was manufactured
to feed them**:

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
  initialized with a numeric literal and later assigned an array. That is a
  representation change mid-life; a program that wants an array field
  initializes it to `[]`, and one that wants a sentinel uses `null`.

These five are reported as zero-count in the extension stratum. A zero here
means "the construct is not one people write", which is itself information the
ranking should carry — it is not an instruction to add programs.

## The freeze

`manifest.json` is committed before the counter runs for record. Neither the
corpus nor `../predicates.json` may be adjusted after scores are visible; any
later change must be a separate, explicitly-justified commit that says why.
The published ranking carries the corpus hash so a reader can tell exactly
what was measured. See the design spec §4.3.
