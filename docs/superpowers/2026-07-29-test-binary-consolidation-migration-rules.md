# Test-binary consolidation: migration rules glossary

Case files under `crates/kali_cli/tests/cases/` cite their governing rules by number in
their `#` headers — `rule 6`, `ruling 12`, `U5`. This document is where those numbers are
defined. Measured over the shipped corpus (287 case files):

```bash
$ cd /workspace
$ grep -rlEi '\brule [0-9]+'    --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 221
$ grep -rlEi '\bruling [0-9]+'  --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 167
$ grep -rlE  '\bU[0-9]+\b'      --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 179
```

The numbering is fixed by ruling 1 and must not be re-derived: renumbering here would
break every citation in the tree. Rules 1-13, rulings 1-19 and U1-U16 are all defined
below; the corpus cites rules 1-13, rulings 3-9, 11 and 13-19, and U1-U10, U13, U14 and
U16.

The design spec, `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`,
outranks this document. Where a rule here appears to contradict §5.1-§5.11, the spec wins
and the rule is mis-reconstructed. Two cautions carried from the working document. First,
the numbered rule list is a **reconstruction**: the original list lived only in a dispatch
prompt and was lost, and each rule was recovered from the citations of it across the
project's reports and the shipped headers. Ruling 1 makes the numbering below canonical
regardless. Second, the two preceding tasks were dispatched with their *own* numbered lists
that reuse some numbers for different content — Task 17's rule 7 was `[constants]` hoisting,
Task 16's rule 6 was fixture dedup — so a rule number in a report from those tasks does not
mean what it means here.

## What this document is not

It is the durable half of a working document that lived outside the repository. Dropped in
the move, and why:

- **The `Evidence:` and `Confidence:` blocks under each rule.** They were the reconstruction's
  audit trail, and they cited a project ledger and per-batch reports that do not exist in the
  repository. The in-tree citations worth keeping were folded into the rule text or into
  `In-tree examples:` lines.
- **Line numbers on every carried citation.** Nothing in the shipped tree re-resolves a
  citation in this file, and ruling 11 is explicit that a pointer nothing re-resolves is a
  figure in disguise. Several were already stale when this was extracted — inserting a
  retention header shifts every line beneath it, which is the failure U3 warns about. File
  paths and named constructs survive; they are searchable and do not rot.
- **The "Open questions for the controller" section.** All seven were answered, and the
  answers are the rulings below. Ruling 1 in particular settles the rule 8/9/10 boundary
  that most of them turned on.
- **Remaining-work counts.** U10's "eight `browser_*` files with this shape remain to
  migrate" and its like described a queue that no longer exists. Ruling 16 is the reason:
  a figure whose truth depends on work that has not happened yet cannot be made true by
  measuring it more carefully.
- **Pointers to scratch working files** of any kind.

Figures appearing *inside* rule text — "0 of the 45 case files", "13 of 139", "48/48" — are
the measurements taken when that rule was written, kept because they are part of the ruling's
reasoning. They are historical record, not claims about the tree today, and were not
re-derived for this document.

## Reading a citation

- **`rule N`** — one of the thirteen numbered rules. Canonical from Task 18 batch 3 onward
  (ruling 1) and carried into Task 19.
- **`ruling N`** — one of the nineteen controller rulings. Where a ruling and a rule
  disagree, the ruling wins: the reconstruction is evidence, the rulings are decisions.
- **`U<N>`** — one of the unnumbered governing rules, imposed after the original list as
  reviewer or controller findings. Binding, but deliberately not renumbered into the
  numbered list.
- **`batch N`** — **not a single namespace, and it is the one piece of this vocabulary that
  is not defined here.** In `browser/` headers it counts Task 18's browser batches (a pilot,
  then 2 through 8, with letter sub-batches such as 6A, 6B, 7A, 7B, 8A, 8C). In `misc/`,
  `runtime/` and `nullish/` headers it generally counts Task 19's batches, 2 through 5, which
  are the ones the generators under `tools/migration/` own. Other numberings also appear in
  the corpus and mean something else entirely — "soundness batch 1", "PR #16 rev2, batch 7".
  Read a bare `batch N` against the family the file sits in and whatever task the surrounding
  sentence names; do not assume one sequence.

---

# Controller rulings

Binding decisions. Where a ruling and a reconstructed rule disagree, the ruling wins.

### Ruling 1 — this file's numbering is canonical from batch 3 onward

Rules 1-13 as titled here are what a batch 3-8 header means when it cites "rule N". Do not
re-derive numbering from the pilot or batch 2 reports, and do not relitigate the rule 8/9/10
boundary per file: rule 8 is *never hand-simulate a generated string*, rule 9 is *never
rewrite the program under test*, rule 10 is *genuine `${…}` is escaped through `[constants]`*.
`[source]` key renaming for disambiguation is **U5**, not rule 8 — two batch 2 headers cite it
as rule 8 and are wrong; leave them, do not repeat them.

### Ruling 2 — rules 4, 6 and 12 stand as reconstructed

Despite their `partial` reconstruction confidence. Nothing downstream turns on their original
wording — only on their content, which is corroborated. Cite them.

### Ruling 3 — assertion strengthening: one policy, no per-file judgment. MIRROR THE SOURCE

- exact source assertion (`==`, `.matches(x).count() == 1`) → exact pin
- position-anchored source claim (`starts_with`, `ends_with`) → **never** downgraded to
  `*_contains`; pin it exactly
- plain `.contains(x)` against a field that HAS a substring form → keep `*_contains`.
  Do **not** strengthen to an exact pin because you observed the exact output.
- ~~plain `.contains(x)` against a field with NO substring form (a `json` leaf) → exact pin,
  and only after live-verifying the value against the real binary.~~
  **AMENDED after batch 8C — clause 4's factual premise is now false.** When this ruling was
  written a `json` leaf genuinely had no substring form. `json_count` arrived in the batch-4
  interlude, *after* this ruling, and `JsonCountClaim { path, needle, bound }` with
  `at_least = 1` (`crates/kali_case_runner/src/model.rs`; `assertions.rs`'s `check_json_count`,
  which requires a string leaf and counts non-overlapping substring occurrences) **is** exactly
  that substring form. The rule that binds from now on:

  > **plain `.contains(x)` against a `json` string leaf → `json_count` with `at_least = 1`.
  > An exact `json.…` pin only where the source's own assertion is exact.**

  This restores clause 3's symmetry: mirror the source, and do not strengthen because you
  observed the exact output. It binds Task 19's non-browser families and Task 20.

**Grandfathered — TWO sets, and the second is large.**

(a) The 16 exact `stdout = "1\n2\n"` pins in `cases/browser/array_iteration_spread_runtime.toml`
predate this ruling and were live-captured. They stay. Batch 3 adds a note in that file's header
recording that they predate the policy and would be written as `stdout_contains` today. Do not
churn the pins.

(b) **Every exact `json` string-leaf pin in the `browser/` family** — the whole class, stated
without an integer on purpose (ruling 16: a family-wide population count has no gateable home,
and writing a corrected integer only resets the clock). Each was written under clause 4 as it
then stood, so they are compliant with the ruling that bound their authors, not a deviation
from it. They stay as shipped, **and the reason is clause (a)'s plus one clause (a) did not
have: their sources were deleted by batch 8C, so any rewrite must be re-derived from history,
and a family-wide sweep across shipped artifacts is the precise shape of change rulings 9, 11
and 15 exist to prevent.** Do not churn them. A migration that adds a *new* json-leaf
`.contains` claim from here on uses `json_count`.

*Population, if a later reader needs one, with the command beside it (ruling 13) — and a
disagreement recorded rather than smoothed over.* Batch 8C's review stated "226 pins across
25 files". 8C could not reproduce that figure under any predicate it tried, and reports its
own instead:

```bash
cd crates/kali_cli/tests/cases/browser
# json string leaves that are the `.contains`-migrated class (stdout/stderr leaves)
python3 -c 'import glob,re;leaf=re.compile(r"(\w+)\s*=\s*\"((?:[^\"\\\\]|\\\\.)*)\"");\
print(sum(1 for t in glob.glob("*.toml") for l in open(t) if l.strip().startswith("json = {")\
for k,v in leaf.findall(l) if k in ("stdout","stderr")))'          # -> 420, across 52 files
grep -h "^# EXTRA-OK:" *.toml | grep -ci json                       # -> 129, across 57 files
```

The grandfathering is of the CLASS and does not depend on which integer is right.

### Ruling 4 — the fixture-self-inspection audit blind spot is documented, not tooled around

A helper whose `.contains()` self-checks read the fixture's own source text before any command
is built is invisible to `audit-case-migration.py`. Do **not** extend the script for it
(measured at ~15 of the remaining files). Each hit is escalated per rule 3 and retained
hand-written with a `//!` header per U3, matching `browser_array_from_set_map_bundle.rs`'s
shape. U4's trim-and-keep still applies first: retain whole only when *every* test in the file
reaches the self-inspecting helper, and state the count in the header. The two batch 2
retentions were upheld on review and are **not** reopened.

### Ruling 5 — `comment_coverage.py` gets a floor before it gates a batch loop

It exited 0 on "0 non-divider comment lines checked" — a vacuous green about to run across 133
files. Batch 3 adds the guard (nonzero exit when the checked-line count is 0) as its first
commit, per U11 and U12.

### Ruling 6 — rule 13 does NOT reach cross-crate runner infrastructure docs

*(Added after batch 4; same binding force.)*

`kali_runtime_contract::browser_bundle_harness_script` and `::browser_harness_command_parts_for`
carry `///` docs and sit in every bundle case's *original* call chain, but in the migrated form
**the case file never calls them — the `browser_bundle_harness` step kind means the runner
does**. Their docs describe shared runner infrastructure (spec §5.3), not what the case claims.
0 of the 45 case files shipped before batch 4 carry either string, and batch 4 followed that
precedent. **This exemption is now written into the rule rather than left to implementer
discretion** — which is what batch 2's review asked for and did not get. Rule 13 still binds
fully for every helper whose *output* the case reproduces. The test: if the migrated case still
depends on what that helper computed, carry its docs; if the helper's job is now the runner's
job, do not.

### Ruling 7 — do NOT hoist duplicated `[source]` bodies into `[constants]` in `browser/`

*(Added after batch 4; same binding force.)*

U13's hoist collides with two things here: `check_fixtures.py` searches only `[source]` values
and step `body`, so a hoisted body makes the rule-9 gate go red on a *correct* file; and U13's
own recorded counter-hazard applies, since hoisting moves program text onto the surface
`assertion_strings()` searches. Batch 4 declined the hoist and asserted duplicate identity
mechanically in its generators instead. That is the ruling for batches 5-8: **decline, but the
mechanical identity assertion is mandatory** — duplication without a check is just duplication.
This matches all 45+ shipped `browser/` files and knowingly contradicts `switch/runtime.toml`,
where hoisting was a review fix. The family-vs-family inconsistency is recorded for the final
review to triage once, with full sight of both; it is not for batch 5-8 to relitigate per file.

### Ruling 8 — stale source `#[test]` fn names are carried as flagged notes, not corrected

*(Added after batch 4; same binding force.)*

*(Corrected after batch 4's review: this ruling originally said "three sources." That count came
from a batch-4 report that listed only the files that happened to receive notes — the tree has
**seven**. The policy below is what binds; the count never did. Every source whose fn name
misdescribes its own body needs a note, and the label must be `MIGRATION NOTE` — `FACTUAL NOTE
ON THE LOOPED FN'S NAME` is a second label for the same thing and should not be used again.)*

Sources carry names that lie about their own bodies (`_in_js_and_ts_input` on fns looping all
four extensions; 8 `json_`-prefixed fns in `imul_omitted_operands` issuing non-JSON commands).
U7 governs stale *prose*; a fn name is not a comment, so U7 does not literally apply and the
implementer's reading is **confirmed**. Two further reasons to leave them: the `.rs` files are
deleted wholesale after batch 8, so any edit is churn with a short half-life; and editing a
source invalidates every audit that runs against its pre-trim blob. Carry a `MIGRATION NOTE`
naming the discrepancy so the case file records what the source actually did.

### Ruling 9 — every U4 retention pair red-lists its gates, in one place

*(Added after batch 4's re-review.)*

A U4 trim-and-keep retention leaves the on-disk `.rs` shorter than the source its case file was
migrated from. **Every literal-comparison gate therefore goes red on that pair when run the
normal way**, against the post-trim file: `audit-case-migration.py`, `comment_coverage.py`, the
U8 check, and now `check_extra_claims.py`. Each is a false failure; all of them pass against the
pre-trim blob recovered from git.

This is systemic to every retention pair, not a property of any one file — it reproduces on
batch 3's `math_atan2_global_this_root` as readily as on batch 4's two. So:

- Every retained `.rs` header carries a **CONSEQUENCE FOR THE GATES** paragraph naming **every**
  gate that is expected-red post-trim, and the pre-trim ref to run them against. When a new gate
  is added, that paragraph is part of the gate's own change — batch 4 added
  `check_extra_claims.py` and edited this exact block in the same commit without adding it.
- **Never write "all N pairs, every gate exits 0" when retention pairs are in the set.** State
  the retention pairs separately, with the ref they were verified against. Batch 4's
  verification table made the blanket claim and it was untrue for 2 of its 22 pairs — the
  numbers were right, the sentence was not.
- The batch-8 family gate needs the same carve-out; this ruling is its per-file counterpart.

See also ruling 12, which corrects the left-hand side, and ruling 19, which makes it per gate.

### Ruling 10 — find the fixture-self-inspection retentions before writing the brief

*(Added after batch 5.)*

***CORRECTED after batch 5's review — the first form of this ruling was wrong and was briefly
binding. Do not use it.*** The original predicate was "which `#[test]` fns never construct a
`Command`." It has a demonstrated false-negative class: when the self-inspecting
`assert!(source.contains(...))` lives **inside the assert helper that also builds the
`Command`**, every test constructs one and the predicate returns nothing. It misses
`browser_promise_any_bundle.rs` and `browser_promise_any_harness.rs` (both unmigrated), and it
also fails to return four **already-adjudicated** retentions —
`browser_array_from_set_map_bundle.rs`, `browser_array_from_set_map_harness.rs`,
`browser_generator_default_export_rejection.rs`, `browser_math_pow_exponent_one.rs`. Batch 5's
three hits happened to put the self-inspection in a standalone Command-free test; that is a
property of those files, not of the blind spot, and the validation that "it returned exactly
three" bounds over-reporting only, never under-reporting.

**Use instead:** a `.contains()` / `assert!` whose receiver is a **fixture-builder return
value**, reachable from any `#[test]`, *regardless of whether a `Command` is constructed*.

**This ruling is now a tool, not a sentence.** Run
`tools/task-18-browser-pilot/find_fixture_self_inspection.py` and require its `--selftest` to
pass; do not re-implement the predicate from this prose, which is what the first version's
reader did. Its `KNOWN` list is load-bearing ground truth: **every newly adjudicated instance
must be added to it**, or the selftest silently weakens as the corpus grows.

The corrected predicate found only **6 of 9** ground-truth instances on its first run — two
further receiver-binding cases (a conditional initializer, and a parameter fed an inline
literal) had to be implemented to reach 9/9. A version validated against its own sample would
have shipped at 6 and looked correct. Family scan: **13 of 139** files carry the shape; 11 are
already adjudicated and **exactly 2** are not — `browser_promise_any_bundle.rs` (8/8 reach) and
`browser_promise_any_harness.rs` (16/16), both unmigrated. No other unmigrated target has it.

### Ruling 11 — no figure that any edit to the header can move

*(Added after batch 5.)*

This generalizes ruling 9's "no count of the header's own length," and batch 5 found why it had
to: `check_extra_claims.py` accepts any claim string occurring anywhere in the `.rs`, **comments
included**, so drafting a red-list paragraph dropped one file's unexplained-extras figure from 38
to 37. The header's own wording is an input to the number the header reports. So retention
headers state the **class** for any such figure and give no integer. Figures counting things
genuinely outside the header — test counts, literal counts, trial counts — stay, and must be
exact.

**`:N` code citations are exempt, and the exemption is conditional.** U3 requires citations into
the retained `.rs`, and every one of them is moved by an edit to the header above it — so U3 and
ruling 11 collide as written. The resolution: a citation is a *pointer*, not a measurement, and
there is no way to point at code without one, so citations stay. But they only stay because they
are **mechanically gated**: `batch5_crosscheck.py` must resolve every citation, including the
`:N` citations inside the retained `.rs` header itself. A pointer nothing re-resolves is a figure
in disguise.

*Granularity, corrected after batch 5's re-review:* this originally said "**exact** line
matching." The implementation resolves to the enclosing **syntactic statement** instead, which is
better — a citation onto the `) {` of a rustfmt-split signature, or into the interior of a
multi-line `assert!`, is not drift. Statement granularity is the rule. But it is only sound while
statement detection is sound: a bracket counter that mis-parses **puts no bound on the window at
all**, which is strictly weaker than the ±3 tolerance it replaced. Statement expansion must
therefore be clamped, and its parser must ignore comment text and raw strings — the retention
header's own prose is an input to it otherwise, which is this same self-referential trap in a
third disguise.

### Ruling 12 — a U4 trim's gate baseline is the pre-trim/retained DIFFERENCE, not the raw pre-trim blob

*(Added after batch 6A; same binding force.)*

Ruling 9 said to run a retention pair's gates against its pre-trim blob. That is the wrong
left-hand side whenever the *retained* half carries literal claims of its own: those literals are
in the pre-trim blob too, so the audit and fixture gates are red against **both** sides and the
red looks permanent. The correct left-hand side is the part that was actually migrated — the
complement of the retained half — built mechanically by
`tools/task-18-browser-pilot/migrated_complement.py`. Retention headers therefore carry a
**three-column** red-list: post-trim / pre-trim / migrated-complement.

*Measured population, and the conditional matters:* ten U4 trims exist (stems carrying a
`PRE-TRIM REF:` **and** a case file; the other six `CONSEQUENCE FOR THE GATES` files are
whole-file retentions with no case file). **Five need the third column and five do not** — the
discriminator is exactly whether the retained tests carry literal claims, and it is clean: zero
literal claims ⇔ green on both older sides. Of the five that need it, only two
(`math_pow_exponent_one`, `math_unsupported_member_calls_harness_jsx_tsx`) are also red on
pre-trim `check_fixtures.py`; the other three are red on the audit alone. **Do not restate this
as "every trim" — that overreach cost a fix round.**

*Retroactive scope is FOUR headers, and it is batch 7's:* `math_max_min_frozen_aliases`,
`math_abs_sign_frozen_aliases`, `math_atan2_global_this_root`, `math_pow_exponent_one` each
declare their audit red *"the escalation itself, not a trim artifact"*, which ruling 12 makes
false — measured against the complement all four go green. **The four retentions themselves
stand:** every one is adjudicated on the fixture-self-inspection ground and sits in the
predicate's `KNOWN` list. The audit red was always a *consequence* of the trim, never the
escalation ground. Correcting the description retires no retention and migrates no test.

### Ruling 13 — a prose quantifier's enumerating command runs BEFORE the sentence is written, and appears beside the claim

*(Added after batch 6A; same binding force.)*

A sentence that quantifies over a set of files must have its enumerating command run first, and
the command must appear beside the claim. Numbers have ruling 11; citations have
`batch5_crosscheck.py`; **a prose quantifier has nothing that reads it.** Batch 6A spent two fix
rounds on one — "the first", corrected to "every", both false, neither a number nor a citation.
`all`, `every`, `none`, `only`, `the first`, `the only` are the trigger words.

**This binds briefs and dispatch prompts, not just headers.** The controller propagated three
underived figures into prompts in a single batch (a wrong pre-trim ref, a wrong gate count, a
stale scan total); all three were caught downstream by implementers and reviewers, none by the
controller. A figure handed over without its derivation is the same defect with the quantifier
left implicit. Hand over the command, or hand over nothing.

### Ruling 14 — `audit-case-migration.py` models an OR as one claim, and the suppression is site-scoped

*(Ratified in batch 6B; backfilled during 7B, having been cited as binding while absent.)*

`audit-case-migration.py` modelled a source's `.contains` claims **conjunctively**. Rule 11 says
an OR-shaped assertion is resolved against the real binary and the observed branch pinned — so
the audit demanded a literal that no correct case file could carry. That is a modelling bug in
the tool, not a policy gap: the source makes **one** claim (a disjunction) and the tool counted
two.

**Ratified:** `disjunctive_contains_groups` teaches the audit that an OR is one claim. Two
conditions were required and met — no already-shipped pair's verdict changes (verified three ways
over all 79 stem-matched pairs), and the pinned branch is what the binary actually emits (48/48).

**Suppression is site-scoped, and that is load-bearing.** A literal that is an unpinned disjunct
at one assert site and an **unconditional claim** at another must still be reported. Keying
suppression on the literal globally made the gate go green on a genuinely dropped claim in
`browser_wasm_threads_browser_surface.rs`. Suppress a member only if **every** `.contains` site
of that literal lies inside a satisfied group.

*This ruling was cited as binding by two shipped case files, `scripts/audit-case-migration.py`
and `gen_batch6b.py` for two batches before its text existed. The behaviour was in the tool; the
rule a later batch would read was not. Ledger entries are not rules until they are written here.*

### Ruling 15 — the three legitimate answers to a figure

*(Added after batch 7's instrument work.)*

Ruling 13 says a prose quantifier needs its enumerating command run first. Batch 7 spent **five
fix rounds** discovering that "record the command beside the figure" is not enough, because the
figure still rots. There are exactly three sound answers, and batches 7A/7B/8 must pick one:

1. **Declare it and gate it.** Put the figure in a constant the gate compares against its own
   answer every run — `NO_NEEDLE_DECLARED` and `PINNED_SPLIT_DECLARED` are the worked examples.
   The declaration must be *the gate's own output*, recorded from inside the gate's own loop,
   never a second computation of the same thing. This is the only answer that survives an
   unrelated edit.
2. **Pin it to an immutable ref — both sides.** A command's output is immutable only if
   *everything* it reads is pinned. **This is narrower than it looks and the unnarrowed form is
   false.** Batch 7 claimed "a command pinned to an immutable git ref has immutable output" for a
   command that took its *corpus* from a `git archive` but its *patterns* from the working tree;
   adding one lookbehind alternative to `WRITTEN_CITE`, with ref and corpus and command text
   untouched, moved its output from `57 files / 873` to `59 / 881`. So: pin both sides, or state
   explicitly which side is live and re-run the block whenever that side changes.
3. **Delete it.** Most figures in prose are illustrative and nothing computes from them. Batch 4
   learned this for self-referential counts ("the number should never have existed"); it
   generalises. A figure that is neither gated nor pinned and that no reader acts on is pure
   liability.

**The trap that caught four consecutive rounds:** the sentence *describing* a correct mechanism
is where the defect lives, and each attempt to fix the description introduced the next one —
"the first" → "every" → "none pins one statement" → "there are no figures in it now" → "every
figure is ref-pinned", each false, each in the same paragraph. If you find yourself writing a
universal quantifier about a set of figures, prefer answer 3.

**A green suite is not evidence for a gate change.** A removed check is silent by definition.
Batch 7 deleted a live check while reporting it as removing dead code, and the deletion passed
its own review because the sweep stayed green. Verify a gate change only with an **injection
probe** showing the check still fires on the thing it exists to catch — and check one layer up,
because the probe itself may be ungated.

### Ruling 16 — a family-wide count has no gateable home inside a case file

*(Added during batch 7A.)*

Batch 7A's migration falsified a figure in a **batch 6B** case file: it said `-> 80` and `0 of
the 80 case files`, and 7A's thirteen new files made it 88. A count of *other files*, invalidated
by adding those other files — and it would break again in every remaining batch by construction.

Work ruling 15's three answers and only one survives:

- **Pinning it fails by definition.** The count describes a live corpus; a ref-pinned count
  describes a tree that no longer exists. That is not a stable figure, it is a differently-false
  one.
- **Gating it from inside the artifact is impossible.** Nothing reads a `#` header, so the gate
  must live outside the case file and be re-run by every batch — which is precisely the "record
  the command beside the figure" disposition ruling 15 rejects, because the figure still rots.
- **Deleting it works**, and the claim almost always survives without the integer.

**So: a case file may not state a family-wide population count.** State the class and keep any
enumerating command that supports the surviving quantifier, or move the count into a gate that
derives the population. **Writing a corrected integer only resets the clock** — the next batch
invalidates it again, and nothing fails when it does.

The general form, which applies beyond case files: *a figure whose truth depends on work that has
not happened yet cannot be made true by measuring it more carefully.*

### Ruling 17 — rule 11 when more than one disjunct is true

*(Added during batch 7B.)*

Rule 11 resolves an OR against the real binary. Batch 7B hit the case rule 11 does not cover: the
resolved stream carries **several** disjuncts, all true on every cell. The procedure:

1. **Resolve the stream by observation.** Raise if a cell is ambiguous or if cells disagree.
2. **Among disjuncts universally true on that stream, pin the first in source order.** Raise if
   none is universal, or if the output modes disagree about which others hold.
3. **Disclose the others** in the header.

**Pinning all true disjuncts is a rule-2 invention, not the conservative choice.** Both `A` and
`A ∧ B` are stronger than `A ∨ B`, but they are *ordered* — `A ∧ B ⊢ A` — so pinning one is the
strictly weaker and more faithful strengthening. Rule 2 bars adding an assertion merely because
it is true: the source never asserted `B` unconditionally, and pinning it makes the case fail on
a benign output change the source explicitly tolerated.

Note the decomposition, which is most of the answer: an OR has **two** degrees of freedom, and
only the needle needed a policy — the stream is forced by observation.

Batch 6B's `literal array` is **not** precedent here; only one disjunct was true there, so there
was no tie. Source order's one weakness — a no-op reordering in the `.rs` would move the pin — is
inert once the sources are deleted, and the generator raises rather than drifting.

### Ruling 18 — a marker-string gate is only as good as its normalisation

*(Added during batch 7B.)*

Several gates in this project decide *which* check to run by looking for a marker sentence inside
emitted prose. That coupling is fragile in a specific, repeatable way: **the gate's input is the
prose it is policing, so every edit to the prose is an edit to the gate's input.**

Batch 7B's non-axis gate went silently one-armed **twice in consecutive rounds, for two different
cosmetic reasons** — first because the haystack was joined with `\n` while the marker spans two
hand-wrapped lines, then because a rewording pushed the sentence onto a line break and a
case-sensitive match stopped hitting. Both times the arm still *existed*, still *passed*, and
caught nothing. Both times an injection probe found it and reading the code did not.

**Normalisation alone is not enough, and this was measured rather than assumed.** After
case-folding and whitespace collapse, an adversarial sweep still silenced the gate three more
ways: a curly apostrophe (U+2019) killed two arms, an en-dash killed a third, and a hyphenated
re-wrap inside the marker killed all three. These files already carry six em-dashes as rule-12
verbatim carries, so it is not hypothetical. **You cannot enumerate the ways prose can be
edited** — normalisation is only ever a whitelist of the failures you have already seen.

So, in order of strength:

1. **Derive the property; do not mark it.** This is the only sufficient answer. 7B's best fix
   stopped marking: `fail_closed_non_axes_with_claim(rs_text)` derives what it needs from the
   caller's own source and **refuses to render** when the property does not hold, so an importer
   cannot inherit a false clause. A gate that cannot be desynchronised beats a gate that is
   currently synchronised.
2. **Dispatch on provenance, not prose.** When a block must be selected, have the emitting
   function return a tagged value the caller carries through, so the gate selects its arm from
   *which block was called*. Keep marker matching only as a secondary consistency check that
   raises on disagreement.
3. **Make a non-match an error.** The structural defect is that `if MARKER in text:` makes
   *failure to match* indistinguishable from *nothing to check*. Require that exactly one of the
   mutually exclusive markers matches, and that a match on one implies the match its clause
   depends on. Every silencing mutation then becomes a loud failure instead of a quiet pass.
4. **Then** normalise both sides — case-fold, collapse whitespace, join by space not newline —
   assert the markers are already in normal form, and **re-probe every time the prose moves.** A
   wrap-width change is a gate change; probe at several widths.

**A gated clause must not render unless both sides of every dimension it names are present.** 7B's
arms compare only groups that happen to exist, so a file with one command or one output mode
would ship both clauses unverified *and unverifiable*, with no signal.

The general form: **a green arm is not evidence the arm is wired.** Ruling 15's last paragraph
says a removed check is silent by definition; this is the same fact for a check that is present
but no longer matching. Substring matching also has no sentence boundary — a *negated* mention of
a marker makes the gate fire falsely, so the failure is available in both directions.

### Ruling 19 — a U4 trim's correct left-hand side is per GATE, by DIRECTION OF CHECK

*(Added during Task 19 batch 2.)*

Ruling 12 says a trim's baseline is the migrated complement. That is right for some gates and
wrong for others, and the discriminator is **which way the gate reads**:

- **FORWARD coverage** — "did everything in the source reach the case file?" — wants the
  **migrated complement**. `audit-case-migration.py`'s literal arm, `check_fixtures.py`, and
  `comment_coverage.py`. Given the pre-trim blob these report the RETAINED half's content as
  missing, because the case file legitimately does not carry it.
- **REVERSE existence** — "does everything the case file cites exist in the source?" — wants the
  **pre-trim blob**, which is the only side carrying both halves' names.
  `check_rationale_fn_names.py`, `check_extra_claims.py`, and `audit-case-migration.py`'s
  count-correspondence arm. A rationale legitimately names fns on both sides of a trim: the
  migrated fns it was built from, and, in its trim paragraph, the retained one it explains.

**The first form of this ruling said "prose gates vs claim gates" and was wrong.** It put
`comment_coverage.py` on the pre-trim side because its subject is prose. The reviewer disproved
it by construction: `comment_coverage` is a forward coverage gate and wants the complement. It
happens to be green on both older sides of `object_has_own_frozen_js_input` only because that
trim's retained half carries no comments — build a pre-trim variant with a comment on the
retained test and pre-trim goes red while the complement stays green. Direction of check is the
property; what the gate reads is not.

Measured on the one trim this task has produced (`object_has_own_frozen_js_input`, its own
`PRE-TRIM REF:`), all five gates, three sides:

```
gate                          post-trim  pre-trim  complement   correct side
audit-case-migration.py       RED        RED       GREEN        complement
check_fixtures.py             GREEN      GREEN     GREEN        complement
comment_coverage.py           RED        GREEN     GREEN        complement
check_extra_claims.py         RED        GREEN     GREEN        pre-trim
check_rationale_fn_names.py   RED        GREEN     RED          pre-trim
```

Ruling 9's "every retention pair red-lists **every** gate that is expected-red" is unchanged and
is what the fifth row is: it was omitted from that header's first two drafts, which is the exact
failure ruling 9 exists to catch.

---

# The numbered rules

## Rule 1 — Never weaken a claim; an exact source assertion becomes an exact pin

A migrated case must assert everything its predecessor asserted, at equal or greater strength.
Weakening is never acceptable. Strengthening is acceptable only when the stronger value has been
captured from the real `kali` binary, never guessed. When the source's own assertion is already
exact — `assert_eq!`, `.matches(x).count() == N`, `starts_with` — translating it as an exact pin
is "rule 1's non-negotiable direction, not a choice." If a claim cannot be expressed at any
strength, the target stays hand-written per spec §5.11 and that fact is reported (rules 3/4).
An `#[ignore]`d test is *not* run against the real binary to harvest a value: pin the literal the
source asserts and set `ignore = true`.

A position-anchored claim (`starts_with`) is never downgraded to `contains`: pin the exact
`stdout`, or keep the target hand-written.

Ruling 3 is the settled operational form of this rule; read it alongside.

## Rule 2 — Never invent a claim the source did not make

Do not add an assertion to satisfy the audit, to round out a case, or merely because it is true.
A dead literal — a value computed but never asserted, an unread `_expected_stdout` parameter —
is not a claim and must not be turned into one. A `[matrix]` axis or a fold that produces a
combination the source never exercised *invents* a case and is a rule-2 violation. The hazard is
directional for stream selection: narrowing a **presence** OR to the stream that actually carries
it is a verified strengthening, but narrowing an **absence** claim to one stream weakens it and
is forbidden. The qualifier that matters is *purely to satisfy the gate*: an exact `stdout = ""`
on a taken, verified path is the same exact-stdout discipline used everywhere else, not an
invention.

In-tree example: `crates/kali_cli/tests/cases/browser/for_await_object_string_enumeration_harness.toml`.

## Rule 3 — The audit gate is absolute; a blind spot is escalated, not shipped around

"Never ship a file whose audit exits non-zero. No `.rs` deleted until its audit is clean AND
`scripts/test-gate.sh` is green with both suites compiled. A claim the tool genuinely cannot see
is a tool bug — escalate to me, do not disclose-and-ship." The escalation's *scope* is itself
subject to review: when only some tests in a file reach the un-seeable construct, only those stay
hand-written (see rule 4 and U4's trim-and-keep).

In-tree examples: `crates/kali_cli/tests/browser_math_pow_exponent_one.rs`,
`crates/kali_cli/tests/cases/browser/array_iteration_spread.toml`.

## Rule 4 — When the format or the tool genuinely cannot carry a claim: keep hand-written per §5.11, report it, never force a false green

Only ever cited jointly with rule 3 ("per rule 3/4"), as the disposition half of the escalation:
the correct outcome is a documented §5.11 retention (its `.toml`, or the affected part of it,
deleted) plus a report — never a fabricated assertion, never a green run obtained by dropping or
restating the claim. Two mechanisms are explicitly and permanently ruled out: a per-file
audit-exception mechanism (a bypass in the branch's central gate), and teaching the audit script
Rust reachability analysis. §5.11 is the mechanical default for this situation, not a judgment
call, and every instance is logged.

*Reconstruction note (ruling 2 confirms the content stands):* rule 4 is only ever cited jointly
with rule 3, so the split of this material between the two is a reading of those joint citations
rather than recovered text.

## Rule 5 — Split, don't fold

A source `#[test]` fn that makes N independent assertions over N independent programs becomes N
sibling `[[case]]` entries, each with its own `[source]` fixture, named descriptively (not
numbered). The prohibition runs the other way too: never fold two independent programs into one
case.

In-tree examples: `crates/kali_cli/tests/cases/soundness/reserved_bindings.toml`,
`crates/kali_cli/tests/cases/soundness/console_multiarg.toml`.

## Rule 6 — Preserve the 1:1 mapping from source `#[test]` fn to `[[case]]`

Two distinct source `#[test]` fns are not folded into one `[[case]]`, even when their invocations
are literally identical, because the case is the only remaining trace of the fn. The sanctioned
exception is a `[matrix]` fold under rule 7 (a fanned case may correspond to several fns or loop
iterations, provided the assertion mapping stays 1:1 per trial); that convention must be stated
in the file's `#` header.

*Reconstruction note (ruling 2 confirms the content stands):* Task 16's rule 6 was fixture dedup
into `[constants]`, which is live here as U13. A "rule 6" citation in a Task 16 artifact means
the dedup rule; in this corpus it means the 1:1 mapping.

In-tree example:
`crates/kali_cli/tests/cases/browser/for_await_object_string_enumeration_browser_smoke.toml`.

## Rule 7 — Matrix arithmetic must close exactly

`[matrix]` may only be used for an axis over which *every* case in the file varies uniformly.
Before declaring one, enumerate every real helper invocation in the source (expanding every loop)
and confirm `total invocations == cases × axis product`. An axis that fans a case which does not
vary over it produces duplicate trials and breaks the arithmetic; an axis that would require
excluding a case invents untested combinations (also a rule-2 violation). If the arithmetic does
not close, drop `[matrix]` for the whole file and write named siblings, and record the arithmetic
— including a declined matrix and why — in the file's `#` header.

Cross-reading note: Task 17's rule 7 was `[constants]` hoisting
(`crates/kali_cli/tests/cases/switch/runtime.toml`) — a different rule under the same number.

In-tree examples, all under `crates/kali_cli/tests/cases/browser/`:
`bundle_reserved_export_names.toml`, `array_from_frozen_set_map_constructor_result.toml`,
`array_iteration_spread.toml`, `array_iteration_spread_runtime.toml`,
`for_await_array_iteration_alias_chain.toml`, `for_await_object_string_enumeration_harness.toml`,
`bundle_toplevel_start.toml`.

## Rule 8 — Never hand-simulate a `format!`; execute the real code and capture what it produces

Any fixture body built by `format!` must be obtained by *running* the real `format!` (a temporary
`#[test] fn dump()` run with `--nocapture`, or a standalone `rustc` compile), never by
hand-applying Rust's substitution and `{{`/`}}` brace-collapse rules. Hand-derivation ships a
different program that can still trip the same diagnostic, so the real-binary check will not
catch it — it verifies the corrupted fixture against itself.

*Citation hazard (settled by ruling 1):* two batch 2 headers cite "rule 8" for `[source]`
filename disambiguation, which is U5, and a third file by the same author lists rules 8/9/10's
triggers as `format!` / library-crate import / genuine `${…}`. The rule 8 above is what binds;
the mis-citations were left in place rather than churned, and must not be repeated.

In-tree examples: `crates/kali_cli/tests/cases/browser/math_pow_exponent_one.toml`,
`crates/kali_cli/tests/cases/browser/array_iteration_spread_runtime.toml`.

## Rule 9 — Never rewrite the program under test

The text written into `[source]` must be byte-identical to the program the source test actually
wrote. That covers the *resolved* string after every transformation, not just placeholder
substitution; it covers fixtures built one level removed, inside a library crate
(`kali_common::…`), which must likewise be captured by executing the real code; and it forbids
semantically-inert edits — renaming an `import()`/`require()` specifier baked into the JS is a
rule-9 violation even when the target file is byte-identical.

Rules 8, 9 and 10 form one cluster about fixture-text fidelity; ruling 1 fixes the boundary
between them.

In-tree example: `crates/kali_cli/tests/cases/browser/array_iteration_spread.toml`.

## Rule 10 — Genuine `${…}` in fixture text is escaped through `[constants]`, never deleted or altered

`expand.rs`'s `substitute()` hard-fails on any `${…}` left unresolved, so a fixture containing a
real JS template literal must declare `[constants] dollar = "$"` and spell every genuine `${` as
`${dollar}{`. The resolved program text is unchanged (this is an encoding of rule 9, not an
exception to it). See spec §5.7: substitution is closed at two forms, and an unresolved `${…}` is
an error.

In-tree examples:
`crates/kali_cli/tests/cases/browser/for_await_object_string_enumeration_browser_smoke.toml`,
`crates/kali_cli/tests/cases/browser/array_iteration_spread_runtime.toml`,
`crates/kali_cli/tests/cases/browser/bundle_template_literal_interpolation.toml`.

## Rule 11 — An OR-shaped source assertion is resolved against the real binary, not reproduced

The format has no disjunction. When the source accepts either of two outcomes — two diagnostic
codes, two streams carrying the same code, two tolerated stdouts — run the real binary, determine
which one actually occurs for each case/mode, and pin that one. This is a verified strengthening
(every run satisfying the new assertion satisfies the old). The source's full disjunction sentence
must be carried verbatim into each affected case's `rationale`, so the narrowing is recorded
rather than silent. Presence claims only: an absence OR may not be narrowed (rule 2).

Ruling 17 extends this to the case where several disjuncts are true at once; ruling 14 is the
audit's side of it.

In-tree examples, under `crates/kali_cli/tests/cases/browser/`:
`for_await_object_string_enumeration_harness.toml`, `array_iteration_spread.toml`,
`set_map_iteration_bundle.toml`.

## Rule 12 — Carry every source comment verbatim into the migrated file

No Rust comment is dropped in migration. File-wide prose (structural notes, pinning discipline,
section banners) goes into the case file's `#` header; prose attached to a helper or a section
goes into the `rationale` of every case that helper's call path reaches, in full — a pointer
("see the file header") does not satisfy it, because a reader of one failing trial sees only that
trial's rationale. Text is copied, not retyped: an em-dash retyped as `--` is a violation the
mechanical checker catches.

Prose is extracted BOTTOM-UP — list every comment block with its line number before writing any
TOML, then assert coverage mechanically afterwards; `comment_coverage.py` is mandatory per file.
Where a section exceeds roughly 15 cases, keep the banner as a `#` divider *as well as* in each
rationale. A `#` header comment alone is the one place prose must not live, because it will not
print on failure.

*Reconstruction note (ruling 2 confirms the content stands):* the rule demonstrably bound this
work, but the number 12 for it was attached from the preceding tasks' artifacts rather than from a
Task 18 one.

See spec §5.5: `rationale` is a field precisely so the text prints on failure. U16 records the
gate's blind spot for trailing comments and its closure.

## Rule 13 — Transitive helper doc comments

A case produced through a helper call chain carries **every** `///` doc comment on **every**
helper in that chain, not just the one nearest the case, and not only the ones judged
behaviorally significant. This explicitly includes helpers in other crates (`kali_common`). There
is no "purely descriptive" exemption. Helper `///` docs are claim prose whenever they encode what
"passing" means; they must land in the rationale of every case that helper produced, and a
paraphrase in the file header does not satisfy it.

Ruling 6 carves out cross-crate *runner infrastructure* docs, and is the only exemption.

In-tree example: `crates/kali_cli/tests/cases/browser/array_iteration_spread.toml`.

---

# Unnumbered governing rules

Imposed after the original list, as reviewer or controller findings. Binding, but not part of the
numbered list — do not renumber them into it.

## U1 — `[matrix]` is FILE-WIDE, not per-case

`expand()` fans *every* `[[case]]` in a file by the full cross-product of every axis, whether or
not that case references the axis. There is no per-case opt-out. If even one case in a file does
not vary over the axis, drop `[matrix]` for the whole file and write named siblings.

Mechanism: `crates/kali_case_runner/src/expand.rs`, `expand()`.

## U2 — `[source]` is likewise FILE-WIDE; conditional or presence/absence fixtures need separate case FILES

`expand.rs` clones the whole `[source]` map into every trial. So a fixture the source wrote
**conditionally** (`if inherited { fs::write(&manifest_path, …) }`), or one whose mere **presence
or absence** is a case's entire point (a manifest, config, lockfile, sibling module), silently
becomes unconditionally present in every trial once merged — destroying the case's discriminating
power. A `[[case]]` cannot opt out of a fixture a sibling case needs; only a separate case file
has its own `[source]` table. This failure is invisible to `audit-case-migration.py` (no literal
is dropped) and invisible to `cargo test` (the trial still passes). Check for it explicitly before
merging two source fns' fixtures into one file.

Mechanism: `crates/kali_case_runner/src/expand.rs`, `expand()`.

## U3 — Every retained `.rs` ships an in-tree `//!` header explaining why

A retention whose reasoning lives only in a report is indistinguishable from a skipped file: the
family gate prints `MISSING: …toml` and halts with no signal, and a later batch agent rescanning
`browser_*.rs` cannot tell "adjudicated" from "overlooked". Every retention — whole-file or
trimmed — must carry a `//!` header matching `browser_math_pow_exponent_one.rs`'s shape: the
blocking helper by name and line, the count and exact line range of the blocking construct,
whether all or only some tests reach it, why the audit or the format cannot carry it, and a
pointer to where the decision was recorded. Cited line numbers must be verified by opening the
shipped file at that line (inserting the header shifts every line below it).

**CITATION CONVENTION (settled in batch 6; the second option in that batch's brief was
declined).** "Exact line range" above was never honoured uniformly and could not be, so the
convention is stated rather than left to be inferred per file:

- A range `:A-B` runs from the **first line of the first construct** to the **first line of the
  last construct** — the last construct's *opening* line, not its closing line. A five-`assert!`
  self-check block is cited from the first `assert!(` line to the fifth `assert!(` line, even
  though the fifth closes further down.
- A citation is a **pointer, not a measurement** (ruling 11), and it resolves at **enclosing
  syntactic statement** granularity — which is exactly what `batch5_crosscheck.py`'s expander
  implements. A citation landing on the `) {` of a rustfmt-split signature, or inside a
  multi-line `assert!`, is not drift.
- The convention was chosen over tightening every citation to a closing line for one reason:
  rewriting roughly forty citations across fourteen retention headers would itself shift every
  line below each edit, which is the exact failure mode rulings 9 and 11 exist to stop. A rule
  costs nothing and cannot go stale; a sweep of renumbering is a fresh opportunity for all of them
  to.
- Because the gate resolves against the whole shipped `.rs`, a citation that has drifted *into the
  `//!` header* can spuriously resolve against header prose. That happened: batch 5's retroactive
  red-list insertions left `browser_array_from_set_map_bundle.rs`,
  `browser_array_from_set_map_harness.rs` and `browser_math_atan2_global_this_root.rs` with stale
  citations, one of which the gate could not see for this reason. Re-derive by searching for the
  construct, then re-run the gate to a **fixed point** — the first re-derivation shifts the header
  again.

## U4 — The §5.11 precedent is TRIM-AND-KEEP, not whole-file retention

Retain only the `#[test]`s that genuinely reach the un-expressible construct; migrate the rest of
the file. "Keep this file hand-written" is a starting hypothesis, not the answer — trace the
actual call graph first. Whole-file retention is legitimate only when *every* test in the file
reaches the construct unconditionally, and that must be stated explicitly in the retention header
so a later reader does not assume a partial split was possible and missed.

Rulings 9, 12 and 19 govern which baseline a trim's gates run against.

## U5 — Rename `[source]` keys freely for disambiguation, but never one whose text is read back inside a fixture body

`[source]` is one flat file-wide namespace, so a bare filename reused across variants must be
given a variant-suffixed stem — and because `kali build --bundle` names its output directory
after the input **stem**, the `file_json` `path` and `browser_bundle_harness` `entry` must track
the rename rather than staying hardcoded to `"app"`. An entry filename passed as a CLI argument is
always safe to rename; a filename the program itself references by string (`import()`,
`require()`) is not — renaming it is a rule-9 violation. Check every renamed key against every
fixture body's text, not just against the argv.

Two batch 2 headers cite this as "rule 8"; ruling 1 settles that it is U5.

In-tree examples: `crates/kali_cli/tests/cases/browser/array_iteration_spread.toml`,
`crates/kali_cli/tests/cases/nullish/assignment_wrapped_local_binding.toml`.

## U6 — Prose attribution is bottom-up, per helper — never pooled, never over-attributed

A comment belongs in the rationale of exactly the cases its producing helper reaches. Copying both
of a two-helper file's comment blocks into all its cases to make `comment_coverage.py` report
clean is over-attribution and is forbidden, even though it turns the checker green; on such a file
the checker's false "missing" must be documented in the header instead. A checker that pools
header plus all rationales and tests membership in the union cannot verify per-case coverage at
all.

## U7 — Stale source prose is moved VERBATIM and annotated, never silently corrected

When a source comment is factually wrong about the tree as migrated, carry it verbatim and append
a flagged `MIGRATION NOTE` stating the discrepancy. Never quietly fix the sentence, and never let
prose describe a state the shipped file no longer has (a trim that changes the case count
invalidates every rationale written against the old count).

Ruling 8 settles that a stale `#[test]` fn *name* is not prose and is likewise carried with a note
rather than corrected.

## U8 — Rationale prose is audited by NOTHING; verify its own factual claims by hand

`audit-case-migration.py` deliberately never reads `rationale`, `name`, comments, or `[source]`;
`comment_coverage.py` only checks that source comment text *appears* in a rationale, never that a
rationale's own assertions are true. Batch 2 shipped rationales citing source fn names that do not
exist. Required verification step for every batch: grep every backticked `fn`-shaped identifier in
every `rationale` against the real source file's fn list —

```bash
grep -oP '(?<=`)[a-z_]+(?=`)' | sort -u     # cross-referenced against:
grep -oP '(?<=fn )[a-z_]+'
```

The script's own module docstring states what it excludes and why:
`scripts/audit-case-migration.py`.

## U9 — Live-verify every case against the real binary, not a sample

Expected `stdout`/`json` values are captured by running the real built `kali` (with `node` as the
harness backend), per case, not hand-computed and not spot-checked. This is what caught two real
emitter bugs in the pilot and a dropped `json.stdout` claim in batch 2; `cargo test` passing does
not imply nothing was dropped, and only the audit caught the pilot's missing `passed`/`failed`
fields.

A real-binary run proves REALISM (the case matches what the binary does today); it does **not**
prove FIDELITY (that nothing was dropped or rewritten relative to the source). Only a
source-vs-TOML diff proves the latter — a corrupted fixture run against the real binary reports
"0 discrepancies" *because of* the corruption.

## U10 — Submodule-shaped targets: inventory, audit, and delete the sibling directory too

For a target whose tests live behind `#[path = "…"] mod …;` (or a plain `mod`), `grep -c '#[test]'`
on the top-level file returns 0 and silently drops every test. Inventory the top-level `.rs`
**and** every `.rs` in its sibling directory; migrate all of them into one `.toml`; and delete the
top-level file **and** its sibling directory together. `audit-case-migration.py` resolves these
chains itself and hard-fails on "0 `#[test]` fns found", but a batch loop that restores a deleted
file from git for re-auditing must restore its whole directory or the script exits 2.

## U11 — A subagent's own green checkers are necessary, not sufficient

When a batch is split across parallel implementers, the orchestrator independently re-runs (not
re-reads) every audit, `cargo test`, fidelity and comment-coverage claim before shipping. Batch 2
caught a real defect this way that had already passed all of the subagent's own gates — because
the defect *made* the mechanical checker green. Someone must read the actual diff.

## U12 — Verification tooling is committed, not scratchpad-only

`tools/task-18-browser-pilot/` is the standing practice for the remaining batches (reuse
`lexer.py`, `kali_run.py`, `fidelity.py`, `comment_coverage.py`, `toml_emit.py` rather than
rebuilding per batch), so independence is auditable. Any committed script must actually run from a
clean checkout and must gate (exit non-zero) rather than merely report.

## U13 — Byte-identical shared fixture bodies are hoisted into `[constants]`

Any `[source]` value duplicated across entries, or a long common prefix shared by two or more
entries, is hoisted into `[constants]` and referenced as `${NAME}` — and the identity must be
*asserted mechanically*, not eyeballed. Counter-hazard to record when doing it: hoisting program
text into `[constants]` moves it onto a surface `assertion_strings()` *does* search, so a future
phantom claim could be satisfied by a hoisted fixture body, and the "rule-constant count matches
`const` count" gate weakens correspondingly.

Ruling 7 declines this hoist for the `browser/` family specifically, and makes the mechanical
identity assertion mandatory in its place.

## U14 — The fidelity diff is required by default, and its `extra` side must be printed and justified

A source-vs-TOML fidelity diff runs for every pair, not on suspicion. Both directions are
reported: `missing` catches drops, and `extra` — an assertion present in the case file but absent
from the source — is the checkable invariant behind "never invent an assertion" (rule 2). A
checker that computes `extra` and discards it has disabled the gate that catches inventions. Also
print `claims()`'s per-kind counts beside the case count for every pair; a low ratio means the
audit did not really verify that file.

## U15 — Standing fixture prohibition

Do not introduce `br"…"` / `cr"…"` fixtures in migrated case files.

## U16 — `comment_coverage.py` was blind to TRAILING comments, and that was a false green

*(Closed in Task 19 batch 2.)*

`extract_comment_paragraphs` matched `^\s*//`, so a comment sharing a line with code was not
reported missing — it was not reported at all. A source comment could be dropped in migration and
the rule-12 gate would say nothing. Found by review on a `// 4*19999 + (0+1+2+3)` trailing comment
in the since-deleted `heap_grow_runtime.rs`, which was genuinely uncarried.

`extract_trailing_comments` adds them to the checked population, quote-aware and
raw-string-aware (a `//` inside a fixture body is program text, not source prose, and a
`"http://…"` inside a plain string is neither). Rule 12 does not distinguish a comment that owns
its line from one that shares a line with code, and neither does the gate now.
