# Blast-radius definition, register re-measurement, and the frontier ranking

## 1) Problem

`docs/superpowers/followups/kali-silent-miscompile-register.md` §0.1, amendment
2026-07-29 (`64438bf0ef`), closed R-35 and then declined to name the next fix
target. Its exact position:

> **the frontier is unranked, and it is somewhere in
> {R-10, R-13, R-14, R-31, and the rest of the pre-existing SILENT set} — not in
> {R-51, R-52, R-53}.**

It gave two structural reasons and a three-step remedy. Both reasons still hold
today:

1. **"Blast radius" has never had an operational definition.** It has been used
   informally to mean *tier × construct frequency*, and no frequency model over
   real JS has ever been built for this project. `sweep-common.md`'s deliverable
   template even has a **"Blast radius guess"** field — per-agent, prose, never
   counted.
2. **The verdict data is stale.** The ~26 `SILENT` verdicts in §0.2 are dated
   **2026-07-24 / `62d786e74`** and have not been re-measured wholesale since.
   At least one has already moved: R-21's absent-field lane now fails closed
   `E5506` at `64438bf0ef`, where §0.2 still records it `SILENT`. Ranking a
   stale table is not a measurement.

The remedy §0.1 named, in order: (i) write down an operational definition of
blast radius; (ii) re-run the four surface sweeps at current HEAD to refresh
every §0.2 verdict; (iii) *then* rank.

This project does all three.

### 1.1 A third problem the remedy does not name

The staleness in (2) is not an accident of scheduling. Every measurement this
register has ever recorded was an **agent transcript pasted into prose**. Prose
does not re-run. The next re-derivation after this one would start from zero
again, and §0.2 would be stale again within weeks.

So (ii) is not "re-run the sweeps". It is "make the verdict table a thing that
regenerates", or the same section of this same document will need writing a
third time.

### 1.2 The oracle itself has drifted

The register measured against `node v26.5.0`. The current environment has
`v26.7.0`. No verdict in §0.2 has been measured under the current oracle. The
size of that difference is unknown and currently unknowable, because nothing
re-runs.

## 2) Goals and non-goals

**Goals**

- An operational, written definition of blast radius that later fix-projects
  cite, and that any reader can recompute.
- A re-measurement of every §2 register entry at current HEAD against the
  current oracle, mechanised so it regenerates rather than rots.
- A banded, cluster-aggregated ranking of the currently-`SILENT` set, with the
  per-entry numbers published underneath it.

**Non-goals**

- Fixing any miscompile. This project ends at a ranking. The fix it points to is
  the next project.
- Changing any kali compile or runtime behaviour. The one behavioural-adjacent
  item is a documentation gap in `specs/15-errors.md` (§7.1), included only
  because the verdict classifier depends on it.
- Ranking the fail-closed residual. §7.11 of the register enumerates what
  `switch` refuses; that is a real limit on what kali compiles, but it is not
  silent damage and it is not what this ranking orders.
- A general-purpose JS frequency analyser. The counter answers exactly the
  predicates in this project's catalogue.

## 3) The definition (normative)

Blast radius is a **pair**, not a scalar:

```
blast_radius(entry) = (tier, reachable_frequency)
```

### 3.1 Tier — already defined, reused unchanged

§2 of the register defines tier by damage kind, in its own section headers:

| tier | meaning |
|---|---|
| 1 | silently drops code or output |
| 2 | silently produces a wrong value |
| 3 | silently wrong control flow (value otherwise intact) |
| 4 | rendering-only (the in-memory value is correct) |

This axis needs no new work. It is an ordinal on *kind of damage*, it is already
assigned per entry, and this project does not redefine it.

### 3.2 Reachable frequency — the axis that was missing

`reachable_frequency(entry)` is the number of occurrences of the entry's
**triggering construct predicate** across corpus programs **that kali accepts**.

Three rules make it honest.

**Reachability gates frequency.** An occurrence inside a corpus program that
kali rejects scores zero. A defect kali fails closed on does no damage — the
user gets an honest diagnostic, not a wrong answer. This is the correction to
the formula §0.1 proposed: raw frequency over "real JS" would weight the ranking
by a population kali will never compile.

**Predicates are written down before counting.** Each entry gets a syntactic
predicate in the catalogue, committed before the counter runs. Examples:

- R-13 → computed member access whose key expression is not a literal.
- R-10 → a `let`/`const` declaration in a nested block whose declared name is
  also bound in an enclosing scope.
- R-14 → a member/computed read applied directly to a call expression's result.

**Uncountable entries are flagged, never estimated.** Some entries do not reduce
to a syntactic predicate. R-16 ("leaks the handle in concat position") is a
semantic condition about representation, not a shape acorn can match. Those
entries are marked `UNCOUNTABLE`, carry tier only, band on tier alone, and are
listed under their flag. Inventing a number for them would be the precise
failure §0.1 refused to commit.

### 3.3 What the definition deliberately does not do

It does not produce a scalar score, and it does not produce a strict total
order. Two rejected alternatives, recorded so they are not silently
re-litigated:

- **`tier_weight × frequency`, sorted.** The weights are invented. Choose
  1000/100/10/1 and tier always wins; choose 4/3/2/1 and frequency dominates.
  The constants, not the measurements, would decide the outcome — an argued
  ranking wearing a number's clothes.
- **Lexicographic: tier first, frequency as tiebreak.** No invented constants,
  but it contradicts the analysis that motivated this project. §0.1 argued the
  Tier-2 entries R-13, R-10, R-14 and R-31 outrank the Tier-1 entries R-51 and
  R-52, because a computed property read and a block-scoped `let` are ordinary
  where `s?.(7)` and `for (init; ;)` are exotic. Under lexicographic ordering
  R-51 outranks all four by construction.

Banding is by Pareto dominance instead — see §8.2.

## 4) The corpus

Location: `tools/blast-radius/corpus/`. JS only (`.js`).

TypeScript and JSX are excluded deliberately, and it costs no fidelity: no
predicate in the catalogue is sensitive to type annotations or JSX syntax — a
computed member read is the same construct in all four source classes. Excluding
them keeps the counter free of a TS grammar.

### 4.1 Two strata, reported separately

**Anchor.** The six CLBG programs (`crates/kali_cli/tests/clbg_*_runtime.rs`:
binary-trees, fannkuch, fasta, mandelbrot, nbody, spectral-norm) and the
programs in `crates/kali_cli/tests/imperative_core_runtime.rs`. These live today
as **inline string fixtures inside Rust test files**, so building the corpus
means extracting them into real `.js` files. They are programs the project has
already committed to compiling, each with an end-to-end design behind it.

**Extension.** Programs curated for this project, representing plausible kali
workloads.

The two strata's accept rates are reported **separately, never pooled**. The
anchor is accepted at essentially 100% by construction — it is a set of passing
tests. A pooled rate would inherit that and mean nothing. The extension's accept
rate is the informative number.

### 4.2 Curate by intent, never by acceptance

A program earns its place because it is what someone would plausibly write to do
a job kali targets — **not** because kali compiles it.

This rule is load-bearing. If curation filtered on acceptance, the corpus would
exclude exactly the constructs the `SILENT` entries trigger on, every reachable
frequency would be measured over a population selected for already working, and
the scores would be circular. Curation is independent of measurement;
reachability is applied afterwards, as a separate and separately-reported step.

### 4.3 Freeze before counting

The corpus manifest and the predicate catalogue are committed and hashed
**before** the counter is run for record. Neither may be adjusted after scores
are visible.

Without this rule the ranking is unfalsifiable: any desired answer can be
produced by adding or removing programs, and no reader could detect it from the
result. With it, the corpus hash in the published ranking pins exactly what was
measured.

## 5) The construct counter

Location: `tools/blast-radius/`. Node, with **`acorn`** at a pinned exact
version and a committed lockfile.

### 5.1 Not `kali_parser`

Counting the constructs kali miscompiles, using kali's own parser, is the
confounded-instrument trap `sweep-common.md` rule 3 exists to prevent:

> **Beware confounded probes.** A probe built out of a broken lane measures
> nothing.

R-49 is the proof this is not hypothetical. `parse_switch_statement` inspected
`RightBrace` without consuming it, so every statement after a `switch` was
reparented to module scope and executed at module load — silently, for weeks,
with the suite green. A parser with a demonstrated history of silent structural
defects cannot be the instrument that measures its own blast radius.

### 5.2 Shape

One matcher function per **countable predicate in the catalogue** over the
ESTree AST — `UNCOUNTABLE` entries have no matcher by construction — each with
positive *and* negative fixtures. Output is a table keyed by entry, sorted, with
no timestamps, so it diffs cleanly in git.

The counter emits **two counts per predicate**: `raw` over the whole corpus, and
`reachable` over accepted programs only. Publishing both lets a reader see how
much the reachability gate moved each entry instead of taking the gated number
on faith. A predicate with a high `raw` and a zero `reachable` is a specific and
useful statement: *kali already refuses this, so it does no damage.*

## 6) Reachability

Per corpus program, binary: does `kali check` exit 0?

Occurrences in rejected programs score zero. Acceptance is about compile-time
rejection only — a program that compiles and then silently miscompiles is
accepted, which is the entire point of the exercise.

The accept rate is published per stratum (§4.1).

## 7) The oracle step kind and verdict classification

A fourth step kind, `oracle`, joins `cli`, `file_json` and
`browser_bundle_harness` in the file-driven case runner. It runs `kali run` and
`node` over one source, captures `(stdout, stderr, exit)` from both, and derives
a verdict class:

| kali | node | stdout | verdict |
|---|---|---|---|
| exit 0 | exit 0 | equal | `FIXED` |
| exit 0 | exit 0 | differs | `SILENT` |
| exit ≠0, documented code | exit 0 | — | `FAIL_CLOSED` |
| exit ≠0, `E0xxx` or undocumented code | exit 0 | — | `FL_INTERNAL` |
| exit 0 | exit ≠0 | — | `ACCEPTS_INVALID` |
| exit ≠0 | exit ≠0 | — | `BOTH_REJECT` |
| either side exceeds the timeout | — | — | `TIMEOUT` |
| two runs of the same side disagree | — | — | `NONDETERMINISTIC` |

An oracle case's source is the register entry's own **minimal repro** from §2 —
not a corpus program. The corpus and the oracle fixtures are disjoint
populations serving different questions: the corpus answers *how often does this
construct occur in programs kali accepts*, the repro answers *does this defect
still reproduce at HEAD*. Nothing is counted over the oracle fixtures, and
nothing is classified over the corpus.

A case asserts the **class**, not literal output:

```toml
verdict = "silent"
```

This is what stops the table rotting. Today a verdict is prose that a human must
re-derive by hand, which is why §0.2 has been stale since 2026-07-24. As a case,
a class change is a red test.

Scope coverage follows `sweep-common.md`'s binding method rule: top-level and
in-function are different programs in kali, so each entry gets a case in both
scopes. Roughly 84 cases (42 entries × 2), less the entries where scope is moot.

The oracle's `node --version` is recorded in every generated table, and one case
pins the expected version so drift is loud rather than silent. The first run
re-derives everything under `v26.7.0`; the register was measured under
`v26.5.0`, and that difference is unmeasured today.

### 7.1 `E4xxx` is undocumented — a dependency of the classifier

The classifier must separate an honest denial from an internal failure.
"Documented code" means present in `specs/15-errors.md`'s public range registry
and not `E0xxx`.

`specs/15-errors.md`'s range table has **no `E4xxx` row**. It lists `E0xxx`
internal, then `E51xx`, `E52xx`, `E53xx`, `E54xx`, `E55xx`, `E6xxx`, `E7xxx`,
`E8xxx`, `E9xxx`, and the `W` families. But `E4003` (fuel trap) and `E4201`
(WebAssembly translation error) are real, reachable, and are what the register
means by `FL-INTERNAL`.

Under the rule above they classify as `FL_INTERNAL` **because they are
undocumented** — the right verdict for the wrong reason. This project therefore
carries a sub-task: either document the `E4xxx` family in `specs/15-errors.md`
or reclassify those codes into an existing range. Encoding a hardcoded exception
list in the runner is rejected; it would hide the taxonomy gap inside a test
tool.

## 8) Pipeline, scoring, and banding

### 8.1 Pipeline

```
frozen corpus ─┬─→ [acorn counter] ──────→ raw counts per predicate
               └─→ [kali check × N] ─────→ accept table
                                             │
                                   reachable_frequency
register §2 ──→ tier + cluster ──────────────┤
oracle cases (cargo test) ──→ verdict class ─┤
                                             ▼
                            scoring: SILENT entries only
                                             ▼
                              cluster aggregation → bands
                                             ▼
                     ranking doc + regenerated register §0.2
```

The two instruments never communicate. They meet only in the scoring step, which
is arithmetic over two tables. Each is independently testable, and either can
land before the other.

**Only entries whose current verdict is `SILENT` enter the ranking.** `FIXED`,
`FAIL_CLOSED`, `BOTH_REJECT` and the closed entries drop out — they are not
damage. `ACCEPTS_INVALID`, `FL_INTERNAL`, `TIMEOUT` and `NONDETERMINISTIC` are
reported in the regenerated table but are outside this ranking's question, which
is *what silent defect should be fixed next*.

### 8.2 Cluster aggregation and Pareto banding

Scores aggregate by **root cause**, not by entry. The register's own history
says fixes land per cluster: R-02, R-03 and R-05 were all closed by a single
allowlist at the call-lowering choke, and R-49 and R-54 both live in
`parse_switch_statement`. The unit a fix ships in is the cluster, so the cluster
is the unit worth ranking.

Bands are the **Pareto frontier, iterated**:

- Cluster A **dominates** B when A is at least as bad on tier and at least as
  high on reachable frequency, and strictly worse on at least one.
- **Band 1** is the non-dominated set. **Band 2** is the non-dominated set of
  what remains. And so on.

No weights, no thresholds, no invented constants — which is the point, since
inventing a threshold here would reintroduce the exact flaw §3.3 rejects.

`UNCOUNTABLE` entries cannot participate on the frequency axis. They band on
tier alone and are listed separately under their flag, never merged into the
numeric bands.

### 8.3 Outputs

| output | path | generated or authored |
|---|---|---|
| corpus + manifest hash + predicate catalogue + counts table | `tools/blast-radius/` | corpus authored, tables generated |
| oracle cases | `crates/kali_cli/tests/cases/oracle/` | authored |
| the ranking | `docs/superpowers/followups/blast-radius-ranking.md` | tables generated, commentary authored and marked as such |
| regenerated register §0.2 | the register | generated from the oracle table |
| the normative definition | this spec | authored |

§0.1's "the frontier is unranked" amendment is superseded by a measured
statement, following the register's own precedence convention: the new section
states what it supersedes, and struck text is retained rather than deleted.

Generated tables and authored commentary are kept visibly distinct so a reader
never has to guess which one they are reading.

## 9) Failure modes

**Timeouts are a verdict, not a hang.** R-09 already produces infinite loops
that terminate only at a fuel trap (`E4003`), and node has no fuel. Both sides
run under a timeout; exceeding it classifies as `TIMEOUT` and is never green.

**Nondeterministic output.** Every oracle case runs both engines twice. A
mismatch between runs classifies as `NONDETERMINISTIC` rather than recording
whichever verdict happened to come out first.

**A near-zero extension accept rate.** If the curated programs mostly fail to
compile, that is the headline finding and is published as one. The response is
explicitly *not* to widen the corpus until the numbers improve — §4.3's freeze
exists to make that impossible.

**Ran nothing and passed.** `scripts/check-determinism.sh` has been green while
executing zero tests since `2448dd8839` (2026-07-23), in CI, on main, for weeks:
its `--exact` filters name unqualified functions that libtest resolves under
`#[path]` submodule prefixes, so every invocation reports `0 passed; 1829
filtered out` and `set -euo pipefail` never sees a failure. Every lane in this
project asserts a nonzero expected count. A missing fixture, a node that fails
to launch, or a filter matching nothing is a failure, never a pass.

## 10) Testing the instruments

The project's own rule — validate the instrument before trusting it — applied to
itself.

**The counter.** Positive and negative fixtures per predicate; known-answer
files whose true counts are fixed by construction; and a completeness test
asserting that every `SILENT` register entry has either a predicate or an
explicit `UNCOUNTABLE` flag, so no entry disappears by omission.

**The classifier.** One fixture per verdict class, built from known ground
truth: a confirmed `SILENT` (R-13's computed key), a `FIXED` (R-07), a
`FAIL_CLOSED` (R-20 → `E5506`), an `ACCEPTS_INVALID` (R-54's second `default`),
plus deliberate `TIMEOUT` and `NONDETERMINISTIC` fixtures. Each asserts the class
the classifier assigns.

**The scoring step.** Unit tests over synthetic tables covering ties, a single
cluster, an all-`UNCOUNTABLE` input, and an empty `SILENT` set.

## 11) Constraint: the do-not-modify files

`scripts/test-gate.sh`, `scripts/check-determinism.sh`, `mise.toml` and
`.github/workflows/ci.yml` are do-not-modify for agent work.

**The oracle cases are unaffected.** They are `.toml` files under the existing
`cases` binary, which the gate already runs, so they are picked up with no gate
edit. This is a direct dividend of the test-binary-consolidation project.

**The counter cannot dodge it.** It is a node tool, and wiring it into CI would
touch a constrained file. So the counter is a documented developer command with
its **output table committed**, plus a cheap Rust test asserting that the corpus
hash matches the manifest, so corpus drift is caught inside the existing gate.

Wiring the counter into CI needs the same human decision the determinism lane is
waiting on (`docs/superpowers/followups/test-binary-consolidation-determinism-lane.md`).
It is recorded, not worked around.

## 12) Risks

**The risk that decides sequencing.** If most entries turn out `UNCOUNTABLE`,
the frequency axis thins and the ranking collapses to tier-only — which is the
lexicographic approach §3.3 rejects for contradicting §0.1. Mitigation: **the
predicate catalogue is written first**, before any corpus or counter work. If
this risk is going to fire, it fires in the first task rather than after the
corpus is built.

**Curation judgment drives the answer.** Mitigated, not eliminated, by
freeze-before-count (§4.3), published curation criteria, separate strata
(§4.1), and publishing raw alongside reachable counts (§5.2). The judgment is
stated openly rather than hidden inside a number.

**Bulk.** Roughly 84 oracle cases need authoring. The case runner was built for
exactly this kind of bulk, but the fixtures are still real work.

**Cluster assignments may not survive re-measurement.** §2's clusters are
hand-made prose. R-21 has already moved once. If re-measurement moves enough
entries, cluster membership needs re-deriving before §8.2's aggregation is
meaningful.

## 13) Sequencing

1. **Predicate catalogue** — every `SILENT` entry gets a predicate or an
   `UNCOUNTABLE` flag. Fires the §12 headline risk immediately if it is going to
   fire.
2. **Oracle step kind + classifier + its ground-truth fixtures** — the
   instrument, validated before use.
3. **Oracle cases** — ~84 fixtures; §0.2 regenerated from the result. The
   `E4xxx` taxonomy sub-task (§7.1) lands here, since the classifier depends on
   it.
4. **Corpus** — extract the anchor from its inline Rust fixtures, curate the
   extension, freeze and hash.
5. **Counter** — acorn matchers, known-answer tests, raw and reachable counts.
6. **Scoring, banding, publication** — cluster aggregation, Pareto bands, the
   ranking document, and the superseding register amendment.

Steps 2–3 and 4–5 are independent after step 1 and may proceed in either order.
