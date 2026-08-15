# The blast-radius ranking

## 1. What this is, and what it supersedes

This document ranks the kali silent-miscompile register's §2 entries by **blast
radius**, on the operational definition in
`docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §3: the pair
`(tier, reachable_frequency)`, where tier is the register's own damage-kind axis
and frequency is a count of the triggering construct over the corpus programs
**kali accepts**. It is a set of Pareto bands, not a 1-through-N order — §3.3 of
that spec declines to invent the weight a total order would need.

**What it supersedes.** The register's §0.1, in its amendment dated 2026-07-29
(`64438bf0ef`), ended point 2 with: *"the frontier is unranked, and it is
somewhere in {R-10, R-13, R-14, R-31, and the rest of the pre-existing SILENT
set}"*. That amendment named three things that would settle it — an operational
definition, a re-measurement of every §0.2 verdict, and then a ranking. All
three now exist; the third is this file. §0.1 carries a matching amendment
striking the sentence, added by the same commit that adds this document.

### 1.1 Read this before any number below

**One extension program in forty type-checks.** The corpus has two strata: 137
`anchor` programs, extracted from the project's own fixtures and inline test
sources, and 40 `extension` programs, written to do plausible jobs and never
steered by what kali compiles. `kali check` accepts **1 of the 40** — 2.5%, and
the one is `extension/unit_conversions.js`. That rate was independently
reproduced by a reviewer running a raw `kali check` loop over the stratum. It is
a finding about the compiler, not a defect of the corpus: the curation rule in
`tools/blast-radius/corpus/README.md` forbids selecting programs by acceptance,
and had it not, every frequency here would have been measured over a population
chosen for already working.

The consequence runs through everything that follows. **126 of the 127
reachable programs are anchor micro-snippets**, so the reachable axis — the
axis the definition is built on — is in substance a frequency over the
compiler's own test snippets. It measures what kali has been tested on at least
as much as what programs do. The **raw** axis, counted over all 177 programs
accepted or not, is where the extension stratum's evidence lives, and the two
axes are therefore published **side by side** in §2. Raw is never substituted
for reachable, and the corpus is not widened to close the gap: spec §4.3 forbids
adjusting the corpus once scores are visible, and a corpus adjusted to improve a
score it has already produced is not a measurement.

### 1.2 Three zeros, never pooled

`counts.json` classifies every zero, and the classification is carried through
this document rather than re-derived:

- **structurally uncountable** — the construct cannot appear in any conforming
  corpus program. **R-29 alone** (an assignment to a `const` is a run-time
  `TypeError`, so no program that runs clean under node can execute one). Never
  published as a frequency; banded on tier alone with the uncountable set in §4.
- **unsampled** — legal and countable, absent from this corpus. An ordinary zero
  over this population, silent about a larger or differently-shaped one.
- **present but unreachable** — `raw > 0`, `reachable = 0`: the construct *does*
  occur, but every program carrying it is rejected by kali as a whole, usually
  for an unrelated reason elsewhere in the file. **20 entries** are in this class
  at the corpus hash below. It means neither "rare" nor "kali fails closed here",
  and it is the most misreadable number in §3's table.

Separately from those three, four entries (R-17, R-21, R-22, R-54) have **no
predicate at all**, because the condition is a representation or a run-time type
rather than a syntactic shape — or, for R-54, because only invalid JavaScript
triggers it. Those four and R-29 make up §4's uncountable list: banded on tier,
never as a `0`.

### 1.3 What a band is

Band 1 is the Pareto frontier: a cluster is in it when no other cluster is at
least as bad on both axes and strictly worse on at least one. Band 2 is the
frontier of what remains. Tier 1 is the worst tier, so a lower tier dominates.
A cluster with an uncountable member has no frequency at all; `dominates`
therefore neither dominates it nor lets it dominate, and it lands in band 1 by
**non-comparability, not by measurement**. Every such cluster is marked, and a
countable-only frontier is printed beside each axis for readers who want the
measured answer alone.

**Band 1 is not "the worst".** It is "the set nothing beats on both axes at
once". A tier-1 cluster with a frequency of zero is in it because no tier-2
cluster, however frequent, can dominate a tier-1 one. That is the definition
working as designed, and it is the first thing §6 discusses.

### 1.4 Provenance

<!-- GENERATED-PROVENANCE:BEGIN -->
| what | value | where it is recorded |
|---|---|---|
| corpus hash | `ca6f53339feb61b1ad988f5075c2648fd95a96b1796d67bcf2cd3af69090660f` | `tools/blast-radius/corpus/manifest.json`, verified on every run |
| node | `v26.7.0` | `counts.json` |
| acorn | `8.18.0` | `counts.json` |
| kali binary | `kali 0.1.0` (`/workspace/.cache/cargo-target/debug/kali`) | `accepts.json` |
| §0.2's verdicts, measured at | `4cfa218814` | `kali-silent-miscompile-register.md` §0.2's own sentence |
| this document generated at | `0a3c4ec0cb` | `git rev-parse HEAD`, recorded by the generator |
<!-- GENERATED-PROVENANCE:END -->

**Everything from §2 to §5 is generated**, by
`cargo run -p kali_blast_radius --example rank`, from four committed inputs: the
register (tiers via `parse_register`, verdicts via §0.2's own generated table),
`tools/blast-radius/counts.json`, `tools/blast-radius/clusters.json` and
`tools/blast-radius/accepts.json`. The banding itself is `aggregate` then `band`
from `crates/kali_blast_radius`. No figure in those sections was typed by hand,
and the region between the markers below is the generator's stdout verbatim.
**That is a test, not a promise:**
`kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`
re-renders both regions and asserts they equal the committed text, modulo the
one HEAD cell that cannot match. Edit inside the markers and `cargo test` goes
red. The HEAD recorded above is the one generation ran at, which is the
**parent** of the commit that adds this file. **§6 is authored commentary and is
marked as such.**

**Citation convention.** Every reference this document makes to another file —
the register's §0.2 and §3, the design spec's §3.3/§4.3/§8.1/§8.2, the corpus
README's curation rule — is a citation **as of the HEAD in the table above**,
not a claim about what those files say now. Where a cited statement carries its
own baseline, that baseline is named inline instead. This project has had to
correct the same defect six times: a document asserting another document's
present-tense state is a claim that rots the moment either one moves.

<!-- GENERATED:BEGIN — verbatim stdout of `cargo run -p kali_blast_radius --example rank` -->
## 2. The bands

Bands, not a total order. Band 1 is the Pareto frontier over `(tier, frequency)`: a cluster is in it when no other cluster is at least as bad on both axes and strictly worse on one. Band 2 is the frontier of what remains, and so on. No weight relates a tier to a count, so none is invented — design spec §3.3, §8.2.

**A cluster with an uncountable member has no frequency at all**, and `dominates` makes it neither dominate nor be dominated. Such a cluster therefore appears in band 1 *by non-comparability*, not by measurement, and is marked `n/a` and flagged. Do not read it as a measured frontier member. The countable-only frontier, which is the one a reader wanting a measured answer should use, is printed after each axis.

### 2.1 The clusters, and where each assignment came from

A cluster is a **root cause** — the unit a fix ships in — not a topic. Every assignment is the register's own `Root-cause group:` line on that entry in §2, quoted below so it can be checked against the source rather than trusted. Nothing here is a fresh diagnosis: §3 of the register says grouping errors are cheap to make and expensive to act on, and this ranking is not the place to make one.

| cluster | origin | why it is a cluster |
|---|---|---|
| G2 — call lowering: unresolvable callee folds to constant `0` | register §3 | §3's own header. Its other members (R-02, R-05, R-03) all measure FAIL_CLOSED at `4cfa218814` and do not enter the ranking; R-51 is the cluster's whole silent surface. |
| G3 — guards whose own diagnostic text names the unsoundness that leaks past them | register §3 | §3's own header. §3 states this cluster asserts NO SHARED CODE PATH -- it is a shape of mistake, six independent instances. Its sum is therefore a sum over a pattern, not over a fix unit, and must not be read as 'one allowlist closes this many sites'. |
| G4 — there is no value distinct from the scalar `0` | register §3 | §3's own header. |
| G5 — a string handle reaches a consumer that never proved it was a string | register §3 | §3's own header. |
| G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | register §3 | §3's own header. |
| G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | register §3 | §3's own header. §3 calls R-10's placement 'the weakest in this document'; it is kept because the register makes it, not because this measurement confirms it. |
| G8 — per-sink rendering divergence: direct-log and concat are separate formatters | register §3 | §3's own header. |
| N1 — escape/provenance loss | named here from §2 | §2's R-48 line names the family and its co-member: 'unclustered (escape/provenance-loss family, with R-14)'. §3 has no such cluster, so the NAME is new here; the membership claim is the register's. |
| R-09 (unclustered) | singleton | §2: 'unclustered (isolated lowering bug)'. |
| R-22 (unclustered) | singleton | §2: 'unclustered (missing coercion-table rung; *not* G4 -- the special-case table is present, one rung is absent)'. The exclusion from G4 is explicit and is honoured. |
| R-26 (unclustered) | singleton | §2: 'unclustered (missing range guard in one lowering)'. |
| R-27 (unclustered) | singleton | §2: 'unclustered'. |
| R-28 (unclustered) | singleton | §2: 'unclustered'. §3 lists R-28's RENDERING half in G8 and its VALUE half as unclustered; §2's per-entry line is unqualified, and splitting a per-entry count across two clusters is not possible, so the entry stays whole and unclustered. |
| R-34 (unclustered) | singleton | §2: 'not G8 (see below) -- currently unclustered'. §3's G8 member list still names R-34; §2's entry line refuses the membership in its own words and is the later, more specific statement. |
| R-47 (unclustered) | singleton | §2: 'unclustered. It has **G3**'s shape ... with a **G7** flavour ... It is deliberately *not* added to G3's member list'. |
| R-52 (unclustered) | singleton | §2: 'unclustered (an isolated lowering/emit contract mismatch)'. |

`aggregate` sums a cluster over its members, so an entry in two clusters would be counted twice; the assignment below is a partition. Where the register names two groups it names them in order, and the first is taken. Counts are per **entry**, not per lane, so a cluster sum carries an entry's whole frequency even where the register splits that entry across two clusters by lane.

| entry | tier | cluster | the register's own §2 line | the second reading, and why it was not taken |
|---|---|---|---|---|
| R-06 | 2 | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | G7 (binding storage: `const` inlined, non-`const` composite initializers lost). | — |
| R-08 | 2 | G4 — there is no value distinct from the scalar `0` | G4 (no value distinct from scalar `0`). | §3's G3 member list also names R-08's `??` half. G3 asserts no shared code path, so the mechanism cluster is taken. |
| R-09 | 2 | R-09 (unclustered) | unclustered (isolated lowering bug). | — |
| R-10 | 2 | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | G7. | — |
| R-12 | 2 | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | G3. | — |
| R-13 | 2 | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | G3. | — |
| R-14 | 2 | N1 — escape/provenance loss | unclustered (arena/escape suspicion, untraced). | Paired with R-48 by §2's R-48 line, not by R-14's own. |
| R-15 | 2 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G6 (unimplemented builtin folds to a default instead of failing closed). | §3's G5 also claims R-15's element half, and §0.2 records the live lane as the leaked handle -- the G5 shape -- with the G6 runtime lane deny-set-closed. §2's per-entry line says G6 and is followed. |
| R-16 | 2 | G5 — a string handle reaches a consumer that never proved it was a string | G5 (string handle reaches a consumer that never proved it was a string). | — |
| R-17 | 2 | G5 — a string handle reaches a consumer that never proved it was a string | G5. | — |
| R-18 | 2 | G5 — a string handle reaches a consumer that never proved it was a string | G5 + G3 (it is a hole in an existing guard). | G3, named second by the register. |
| R-21 | 2 | G4 — there is no value distinct from the scalar `0` | G4. | §3's G8 also names R-21's rendering divergence (`"v="+undefined` vs `console.log(x)` giving two different wrong answers). |
| R-22 | 2 | R-22 (unclustered) | unclustered (missing coercion-table rung; *not* G4 -- the special-case table is present, one rung is absent). | — |
| R-23 | 2 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G8 (per-sink rendering) / G4. | G4, named second by the register. Contested: §0.2's row stresses that `typeof x` yields a NUMBER, which breaks `typeof x === "string"` dispatch -- a value defect, not only a rendering one. The ranking reports a sensitivity check for moving it. |
| R-24 | 2 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G6. | — |
| R-25 | 2 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G6. | — |
| R-26 | 2 | R-26 (unclustered) | unclustered (missing range guard in one lowering). | — |
| R-27 | 2 | R-27 (unclustered) | unclustered. | — |
| R-28 | 2 | R-28 (unclustered) | unclustered. | §3's G8 member list names R-28's rendering half; §3's own Unclustered line names its value half. Both lanes measure SILENT, and the count is per entry. |
| R-30 | 4 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G8. | — |
| R-31 | 4 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G8. | — |
| R-32 | 4 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G8. | — |
| R-33 | 4 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G8. | — |
| R-34 | 4 | R-34 (unclustered) | not G8 (see below) -- currently unclustered. | §3's G8 member list still names R-34; §2's own entry refuses it. |
| R-47 | 2 | R-47 (unclustered) | unclustered. It has **G3**'s shape ... with a **G7** flavour ... deliberately *not* added to G3's member list. | G3 by shape, G7 by discriminator -- both named and both declined by the register. |
| R-48 | 2 | N1 — escape/provenance loss | unclustered (escape/provenance-loss family, with R-14). | — |
| R-51 | 1 | G2 — call lowering: unresolvable callee folds to constant `0` | G2 (call lowering: unresolvable callee folds to constant `0`) -- ... Recorded as G2 by symptom; the mechanism is named below. | By symptom only: the route is the optional-chain lowering, not an unresolvable callee. |
| R-52 | 1 | R-52 (unclustered) | unclustered (an isolated lowering/emit contract mismatch), but it is a textbook instance of the pattern §3's G-clusters keep circling. | — |
| R-53 | 2 | G4 — there is no value distinct from the scalar `0` | G4 (there is no value distinct from the scalar `0`) by symptom; plausibly G7 (binding storage) by mechanism, which is not traced. Recorded as G4. | G7, named and declined by the register. |

### 2.2 The reachable axis — the ranking's own definition

Frequency is the count over the 127 corpus programs kali accepts, of which 126 are anchor micro-snippets. This is the axis the design spec §3 defines the ranking on, and in substance it is a ranking over test snippets: 1 of the 40 programs written to do a job rather than to probe the compiler is reachable.

**Band 1**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G2 — call lowering: unresolvable callee folds to constant `0` | 1 | 0 | R-51 |
| R-52 (unclustered) | 1 | 0 | R-52 |
| G4 — there is no value distinct from the scalar `0` | 2 | n/a — uncountable member | R-08, R-21, R-53 |
| G5 — a string handle reaches a consumer that never proved it was a string | 2 | n/a — uncountable member | R-16, R-17, R-18 |
| R-22 (unclustered) | 2 | n/a — uncountable member | R-22 |
| G8 — per-sink rendering divergence: direct-log and concat are separate formatters | 2 | 65 | R-23, R-30, R-31, R-32, R-33 |

*Band 1 is contingent on the cluster assignment. §2.4 re-runs every contested assignment and finds two that move a band 1: R-21 (both axes) and R-23 (the reachable axis, by changing G8's worst tier). Quote this table with §2.4, not on its own.*

**Band 2**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G3 — guards whose own diagnostic text names the unsoundness that leaks past them | 2 | 45 | R-12, R-13 |

**Band 3**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| N1 — escape/provenance loss | 2 | 11 | R-14, R-48 |

**Band 4**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | 2 | 2 | R-06, R-10 |

**Band 5**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | 2 | 0 | R-15, R-24, R-25 |
| R-09 (unclustered) | 2 | 0 | R-09 |
| R-26 (unclustered) | 2 | 0 | R-26 |
| R-27 (unclustered) | 2 | 0 | R-27 |
| R-28 (unclustered) | 2 | 0 | R-28 |
| R-47 (unclustered) | 2 | 0 | R-47 |

**Band 6**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-34 (unclustered) | 4 | 0 | R-34 |

**Countable-only band 1** (the same computation with every uncountable cluster dropped rather than carried, so a reader can see the measured frontier on its own): G2 — call lowering: unresolvable callee folds to constant `0` (tier 1, 0); R-52 (unclustered) (tier 1, 0); G8 — per-sink rendering divergence: direct-log and concat are separate formatters (tier 2, 65).

### 2.3 The raw axis — published beside it, never instead of it

The same clusters banded on the count over all 177 corpus programs, accepted or not. This is the axis that carries what the extension stratum says, because 39 of its 40 programs are unreachable. It is published so a reader can see how far the reachability gate moved each cluster; it is NOT a substitute for the reachable axis, and the corpus is not widened to make the two agree (spec §4.3 forbids adjusting the corpus once scores are visible).

**Band 1**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-52 (unclustered) | 1 | 5 | R-52 |
| G4 — there is no value distinct from the scalar `0` | 2 | n/a — uncountable member | R-08, R-21, R-53 |
| G5 — a string handle reaches a consumer that never proved it was a string | 2 | n/a — uncountable member | R-16, R-17, R-18 |
| R-22 (unclustered) | 2 | n/a — uncountable member | R-22 |
| G3 — guards whose own diagnostic text names the unsoundness that leaks past them | 2 | 305 | R-12, R-13 |

*Band 1 is contingent on the cluster assignment. §2.4 re-runs every contested assignment and finds two that move a band 1: R-21 (both axes) and R-23 (the reachable axis, by changing G8's worst tier). Quote this table with §2.4, not on its own.*

**Band 2**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G2 — call lowering: unresolvable callee folds to constant `0` | 1 | 3 | R-51 |
| G8 — per-sink rendering divergence: direct-log and concat are separate formatters | 2 | 102 | R-23, R-30, R-31, R-32, R-33 |

**Band 3**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| N1 — escape/provenance loss | 2 | 99 | R-14, R-48 |

**Band 4**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | 2 | 17 | R-06, R-10 |

**Band 5**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-09 (unclustered) | 2 | 16 | R-09 |

**Band 6**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-26 (unclustered) | 2 | 12 | R-26 |

**Band 7**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | 2 | 3 | R-15, R-24, R-25 |
| R-34 (unclustered) | 4 | 4 | R-34 |

**Band 8**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-47 (unclustered) | 2 | 1 | R-47 |

**Band 9**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-27 (unclustered) | 2 | 0 | R-27 |
| R-28 (unclustered) | 2 | 0 | R-28 |

**Countable-only band 1** (the same computation with every uncountable cluster dropped rather than carried, so a reader can see the measured frontier on its own): R-52 (unclustered) (tier 1, 5); G3 — guards whose own diagnostic text names the unsoundness that leaks past them (tier 2, 305).

### 2.4 How much the contested assignments matter

9 of the 29 ranked entries have a second cluster the register names with a concrete destination. Each is moved to it, alone, and both band 1s are recomputed. A clustering that cannot be argued with is not a measurement, so the argument is priced here rather than asserted away.

| entry | assigned | moved to | reachable band 1 | raw band 1 |
|---|---|---|---|---|
| R-08 | G4 — there is no value distinct from the scalar `0` | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-15 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G5 — a string handle reaches a consumer that never proved it was a string | unchanged | unchanged |
| R-18 | G5 — a string handle reaches a consumer that never proved it was a string | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-21 | G4 — there is no value distinct from the scalar `0` | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | gains **G3 — guards whose own diagnostic text names the unsoundness that leaks past them**; loses **G4 — there is no value distinct from the scalar `0`** | gains **G8 — per-sink rendering divergence: direct-log and concat are separate formatters**; loses **G4 — there is no value distinct from the scalar `0`** |
| R-23 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G4 — there is no value distinct from the scalar `0` | gains **G3 — guards whose own diagnostic text names the unsoundness that leaks past them** | unchanged |
| R-28 | R-28 (unclustered) | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | unchanged | unchanged |
| R-34 | R-34 (unclustered) | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | unchanged | unchanged |
| R-47 | R-47 (unclustered) | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-53 | G4 — there is no value distinct from the scalar `0` | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | unchanged | unchanged |

## 3. The per-entry table

Every input to §2, so a reader who disagrees with the clustering can re-band from here. `raw` counts all 177 programs; `reachable` counts only the 127 kali accepts. `zero` names WHICH KIND of zero a zero is — the three are not the same claim and must never be pooled (`counts.json` `zeroKinds`).

| entry | tier | raw | reachable | anchor raw/reach | extension raw/reach | §0.2 lanes | zero kind | upper bound | cluster |
|---|---|---|---|---|---|---|---|---|---|
| R-06 | 2 | 6 | 1 | 4 / 1 | 2 / 0 | FIXED / SILENT / SILENT | — | — | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost |
| R-08 | 2 | 95 | 15 | 14 / 14 | 81 / 1 | FAIL_CLOSED / SILENT | — | yes (disclosed in record) | G4 — there is no value distinct from the scalar `0` |
| R-09 | 2 | 16 | 0 | 0 / 0 | 16 / 0 | SILENT / FL_INTERNAL | present-but-unreachable | — | R-09 (unclustered) |
| R-10 | 2 | 11 | 1 | 1 / 1 | 10 / 0 | SILENT | — | — | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost |
| R-12 | 2 | 3 | 0 | 0 / 0 | 3 / 0 | SILENT | present-but-unreachable | — | G3 — guards whose own diagnostic text names the unsoundness that leaks past them |
| R-13 | 2 | 302 | 45 | 47 / 43 | 255 / 2 | SILENT / SILENT | — | yes (**not** disclosed in record) | G3 — guards whose own diagnostic text names the unsoundness that leaks past them |
| R-14 | 2 | 99 | 11 | 8 / 7 | 91 / 4 | SILENT | — | yes (**not** disclosed in record) | N1 — escape/provenance loss |
| R-15 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | — | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed |
| R-16 | 2 | 6 | 0 | 0 / 0 | 6 / 0 | SILENT | present-but-unreachable | yes (disclosed in record) | G5 — a string handle reaches a consumer that never proved it was a string |
| R-17 | 2 | uncountable | uncountable | — | — | SILENT | — | — | G5 — a string handle reaches a consumer that never proved it was a string |
| R-18 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | — | G5 — a string handle reaches a consumer that never proved it was a string |
| R-21 | 2 | uncountable | uncountable | — | — | FAIL_CLOSED / SILENT | — | — | G4 — there is no value distinct from the scalar `0` |
| R-22 | 2 | uncountable | uncountable | — | — | SILENT | — | — | R-22 (unclustered) |
| R-23 | 2 | 3 | 0 | 0 / 0 | 3 / 0 | SILENT | present-but-unreachable | — | G8 — per-sink rendering divergence: direct-log and concat are separate formatters |
| R-24 | 2 | 2 | 0 | 0 / 0 | 2 / 0 | SILENT | present-but-unreachable | — | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed |
| R-25 | 2 | 1 | 0 | 0 / 0 | 1 / 0 | FAIL_CLOSED / SILENT | present-but-unreachable | — | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed |
| R-26 | 2 | 12 | 0 | 1 / 0 | 11 / 0 | SILENT | present-but-unreachable | yes (disclosed in record) | R-26 (unclustered) |
| R-27 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | — | R-27 (unclustered) |
| R-28 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT / SILENT | unsampled | — | R-28 (unclustered) |
| R-30 | 4 | 73 | 57 | 61 / 57 | 12 / 0 | SILENT / FIXED | — | yes (disclosed in record) | G8 — per-sink rendering divergence: direct-log and concat are separate formatters |
| R-31 | 4 | 3 | 2 | 2 / 2 | 1 / 0 | SILENT / SILENT | — | — | G8 — per-sink rendering divergence: direct-log and concat are separate formatters |
| R-32 | 4 | 6 | 5 | 0 / 0 | 6 / 5 | SILENT / FIXED | — | — | G8 — per-sink rendering divergence: direct-log and concat are separate formatters |
| R-33 | 4 | 17 | 1 | 0 / 0 | 17 / 1 | SILENT / FIXED | — | — | G8 — per-sink rendering divergence: direct-log and concat are separate formatters |
| R-34 | 4 | 4 | 0 | 0 / 0 | 4 / 0 | SILENT | present-but-unreachable | — | R-34 (unclustered) |
| R-47 | 2 | 1 | 0 | 0 / 0 | 1 / 0 | SILENT / FAIL_CLOSED / FIXED | present-but-unreachable | — | R-47 (unclustered) |
| R-48 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | — | N1 — escape/provenance loss |
| R-51 | 1 | 3 | 0 | 0 / 0 | 3 / 0 | SILENT | present-but-unreachable | — | G2 — call lowering: unresolvable callee folds to constant `0` |
| R-52 | 1 | 5 | 0 | 0 / 0 | 5 / 0 | SILENT / FL_INTERNAL | present-but-unreachable | — | R-52 (unclustered) |
| R-53 | 2 | 1 | 0 | 0 / 0 | 1 / 0 | SILENT / FIXED | present-but-unreachable | — | G4 — there is no value distinct from the scalar `0` |

### 3.1 What the SILENT filter removed, and what it cost the ranking

Spec §8.1 removes these 12 entries for **two different reasons**, and collapsing them would misdescribe 2 of them:

- **Not damage** — `FIXED`, `FAIL_CLOSED`, `BOTH_REJECT`. kali either agrees with node or refuses honestly. 10 entries leave this way: R-01, R-02, R-03, R-04, R-05, R-07, R-11, R-19, R-20, R-49.
- **Outside this ranking's question** — `ACCEPTS_INVALID`, `FL_INTERNAL`, `TIMEOUT`, `NONDETERMINISTIC`. §8.1 *reports* these in the regenerated table and keeps them out of the ranking, whose question is *what silent defect should be fixed next*. 2 entries leave this way: R-29, R-54. The distinction is not pedantic: R-29's §0.2 row records kali printing `r=1` at exit 0 with no diagnostic, which is silent by any plain reading. It is out because accepting a program node rejects is a different defect class from giving a wrong answer to a valid one — not because nothing bad happens.

Their counts are printed because the removal is not cosmetic: it takes the largest reachable count in the whole measurement out of the ranking.

| entry | tier | raw | reachable | §0.2 lanes |
|---|---|---|---|---|
| R-07 | 2 | 449 | 82 | FIXED |
| R-04 | 1 | 112 | 5 | FIXED |
| R-01 | 1 | 18 | 0 | FAIL_CLOSED |
| R-02 | 1 | 2 | 0 | FAIL_CLOSED |
| R-03 | 1 | 15 | 0 | FAIL_CLOSED |
| R-05 | 1 | 6 | 0 | FAIL_CLOSED |
| R-11 | 2 | 10 | 0 | FIXED |
| R-19 | 2 | 27 | 0 | FIXED / FAIL_CLOSED |
| R-20 | 2 | 4 | 0 | FAIL_CLOSED |
| R-29 | 3 | uncountable | uncountable | ACCEPTS_INVALID |
| R-49 | 1 | 2 | 0 | FAIL_CLOSED |
| R-54 | 3 | uncountable | uncountable | ACCEPTS_INVALID |

Only 2 of the 12 removed entries have a nonzero reachable count at all: R-04 (5) and R-07 (82). The largest of them, R-07 at 82, is **the largest reachable count anywhere in `counts.json`** — larger than the largest that survives the filter (R-30 at 57). The ranking's numeric input is much thinner than the raw measurement looks.

And of the 29 entries that do enter, **9 have a reachable count above zero** (R-06 = 1, R-08 = 15, R-10 = 1, R-13 = 45, R-14 = 11, R-30 = 57, R-31 = 2, R-32 = 5, R-33 = 1); 17 measure zero and 3 have no count at all. The bands below separate 16 clusters on the evidence of 9 nonzero entries.

### 3.2 R-13's number is not R-13's shape

The register's R-13 repro is an **object read with a variable key**. `computedMemberNonLiteralKey` counts every computed member access with a non-literal key, which includes ordinary array indexing `a[i]` — and array indexing demonstrably works. The committed breakdown in `counts.json` splits the same sites by receiver and by position:

| axis | total | object-literal receiver | array-like receiver | store target |
|---|---|---|---|---|
| raw (all programs) | 302 | 56 | 45 | 67 |
| reachable (pooled) | 45 | 2 | 26 | 18 |
| reachable — anchor | 43 | 0 | 26 | 18 |
| reachable — extension | 2 | 2 | 0 | 0 |

Read down the reachable rows: of R-13's 45 reachable sites, **2 have the object-literal receiver the register's repro describes**, and the anchor's share of those is **0** — so **all 43 reachable anchor sites have none**. Both register-shaped sites are in the extension stratum, whose reachable population is the 1 program kali accepts. A further **18 of the 45 are store targets**, not reads: the register treats the write half as the worse one, but it is a different site class from the read its repro shows. R-13's 45 is an upper bound on a construct family, not a count of how often R-13's defect is triggered.

### 3.3 Which counts are upper bounds

A count is an upper bound when the predicate admits sites the defect does not reach — because the AST cannot see a runtime type, a representation, or a compiler-internal proof. 4 records disclose their own upper bound: R-08, R-16, R-26, R-30. 3 more are upper bounds their records do **not** disclose, found by this measurement: R-07 (not in the ranking), R-13, R-14. Every note is in `counts.json` under `upperBound`.

### 3.4 A lane result is not an entry result

11 of the 29 ranked entries measure something other than SILENT on at least one lane, and none of them is thereby retired: R-06 (FIXED / SILENT / SILENT); R-08 (FAIL_CLOSED / SILENT); R-09 (SILENT / FL_INTERNAL); R-21 (FAIL_CLOSED / SILENT); R-25 (FAIL_CLOSED / SILENT); R-30 (SILENT / FIXED); R-32 (SILENT / FIXED); R-33 (SILENT / FIXED); R-47 (SILENT / FAIL_CLOSED / FIXED); R-52 (SILENT / FL_INTERNAL); R-53 (SILENT / FIXED). §0.2 records why in each case — R-47's and R-53's FIXED lanes are the `const` controls those entries declare for themselves, and R-30's two FIXED lanes are its `const`-scalar lane *and* its concat/template sinks, so *declared control* is the accurate description and *`const` lane* is not. R-08's `===` half fails closed while its `??` half is **still SILENT**, unchanged by that move. R-49 — not in the ranking at all — fails closed by **R-35's** switch allowlist rather than by its own gate. An entry is retired when every lane moves, which is a claim no single lane can make.

## 4. The uncountable entries

No frequency exists for these, so they are banded on **tier alone** and are never merged into §2's numeric bands. An uncountable entry is not a rare one: it is one the counter cannot see at all, and publishing it as `0` would rank it below every entry the corpus happens to contain.

| entry | tier | in the ranking? | kind | why no count exists |
|---|---|---|---|---|
| R-17 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a representation condition of the same G5 family as R-16 -- a string handle reaching a consumer that never proved it was a string; whether an array element or `Object.keys` result is a string is a repr fact, not a syntactic one |
| R-21 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a representation condition -- there is no `undefined` distinct from scalar `0`, so it fires wherever an expression *evaluates* to absent or void (a missing field, an out-of-range read, a void return), which is a runtime-value fact rather than a construct in the source |
| R-22 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a runtime-type condition -- the missing rung is number/string coercion, so it fires only when the two operands actually hold a number and a string at run time; same-type `==` comparisons are correct, and the operator alone does not identify the case |
| R-29 | 3 | no — removed by the SILENT filter | structurally uncountable | An assignment to a `const` binding is a TypeError at run time, so no program that runs clean under node can execute one; the construct and this corpus's runnability requirement are mutually exclusive (corpus/README.md). This zero is not a frequency and must never be ranked as one. |
| R-54 | 3 | no — removed by the SILENT filter | no syntactic predicate (representation- or runtime-typed) | only invalid JavaScript triggers it -- acorn, like node, rejects a second `default` clause as a SyntaxError, so the shape can never appear in a corpus file that parses |

**Banded on tier alone** (only the entries the SILENT filter admits):

- Tier 2: R-17, R-21, R-22

The clusters carrying them have no frequency either, which is why they sit in §2's band 1 marked `n/a` — there by non-comparability, not by measurement:

- R-17 → **G5 — a string handle reaches a consumer that never proved it was a string**
- R-21 → **G4 — there is no value distinct from the scalar `0`**
- R-22 → **R-22 (unclustered)**

## 5. The accept rates

Per stratum, never pooled: the anchor's rate is fixed by which tests happen to exist and would destroy the only informative number if averaged into it (`corpus/README.md`).

| stratum | accepted | programs | rate |
|---|---|---|---|
| anchor | 126 | 137 | 92.0% |
| extension | 1 | 40 | 2.5% |

**Two anchor rates, both true, different instruments.** The table above is measured by running `kali check` over every anchor program (`accepts.mjs`, recorded in `accepts.json`). `corpus/README.md` states a different one, from the suite's own run expectation:

> The anchor is **not** accepted at ~100% "by construction", and an earlier
> draft of this file said so wrongly. The measured anchor accept rate is
> **124/137 = 90.5%**. 13 of the 131 `imperative_core_runtime.rs` programs come
> from `run_js_expect_failure` call sites — the suite commits kali to
> *rejecting* them (E3200 / E5506 gates). Dropping them would have been curation
> by acceptance, which the curation rule above forbids in load-bearing terms, so
> they stay in the corpus and the anchor rate is 90.5%. See the design spec
> §4.1, amended 2026-08-15 (Task 11), for the full correction.

The 13 `run_js_expect_failure` programs are ones the suite commits kali to *rejecting*, and the README's rate counts all 13 as not-accepted. Reconciled program by program against `accepts.json`, the whole difference is 4 programs in two directions:

- **3 the suite expects to fail but `kali check` accepts** — `console_log_of_object_reference_is_rejected.js`, `impure_module_const_read_from_function_is_rejected.js`, `object_in_arithmetic_is_rejected.js`. A program the suite commits to failing at *run* time can still pass a *check*.
- **1 the suite expects to pass but `kali check` rejects** — `clbg_fasta.js`.

Neither number is wrong and neither supersedes the other — they answer two different questions about two different instruments, and a reader who sees only one will take the other for a typo.

**What the reachable column is a frequency over.**

> 127 of 177 programs are reachable, and 126 of those 127 are ANCHOR programs -- a stratum that is 131 micro-snippets written to probe compiler behaviour plus 6 real CLBG programs. Every reachable ranking is therefore, in substance, a ranking over test snippets. Read the per-entry `strata` split before treating any reachable figure as a frequency in real code.

> 1/40 extension programs are accepted (2.5%). The extension is the stratum written to do jobs rather than to probe the compiler, so almost everything it measures about real programs lands in the RAW column only. Its accept rate is a finding in its own right, not a defect of the corpus: curation was independent of acceptance.

> The extension is written in the project's imperative-core dialect: no regex, no destructuring, no template literals, no `??`, no class/Map/Set/async. See corpus/README.md for which counts that biases and in which direction. A frequency here is a frequency in *programs of that dialect*, not in JavaScript generally.
<!-- GENERATED:END -->

## 6. Commentary — authored, not generated

**This section is written by hand.** Nothing in it is computed, and where it
argues it says so. Sections 2 to 5 are the measurement; this section is what one
reader thinks the measurement means, and a later reader is free to disagree with
it without disturbing a single number above.

### 6.1 The most important thing here is not a rank

It is §5's `1 / 40`. A corpus written to do jobs, never filtered by what kali
compiles, type-checks at 2.5%. Every reachable frequency in §2 is therefore a
frequency over a population that is 126-of-127 compiler test snippets — and a
snippet corpus tells you what the compiler was tested on, which correlates with
what already works, which is close to the opposite of what a blast-radius
ranking wants to measure. The honest summary of §2.2's reachable axis is: *this
is the best frontier the accepted population can support, and the accepted
population is small and unrepresentative.* §2.3's raw axis is the one carrying
the extension stratum's evidence, which is why both are published and why
neither is allowed to stand alone.

### 6.2 Band 1's shape is a property of the definition, and it needs saying plainly

On the reachable axis, band 1 contains two tier-1 clusters whose frequency is
**zero** — G2 (R-51) and R-52 — alongside G8 at 65. That is not a glitch. Tier 1
is the worst damage class, and no tier-2 cluster can dominate a tier-1 one no
matter how frequent, so any tier-1 cluster is in band 1 unconditionally. Both
are `present-but-unreachable`: the construct occurs in the extension stratum and
every carrying program is rejected as a whole.

**All three of the entries §0.1 ruled out are in band-1 clusters, for three
different reasons, and none of the three is a measured frequency.** R-51 (G2) and
R-52 are there on tier, at frequency 0. **R-53** is there too, as a member of G4
— but G4 is in band 1 only because R-21, its co-member, has no predicate at all,
so G4 has no frequency to be dominated on. R-53's own reachable count is 0. The
2026-07-29 amendment's confident *"not in {R-51, R-52, R-53}"* is wrong on all
three names, and it is wrong in a way that should not comfort anyone: they are on
the frontier because the frontier is a partial order over a thin measurement, not
because they turned out to be common.

A reader who wants "what should be fixed first" should read band 1 as *the set
of candidates no other candidate beats outright*, and then use §3's table to
choose among them on grounds the measurement does not supply — cost, confidence
in the cluster, whether the construct is one the project intends to support.
Anyone who wants a single winner is asking for the weight §3.3 of the spec
declined to invent.

### 6.3 The most consequential judgment call in this document is R-23's cluster

§2.4 prices every contested assignment, and one of them moves band 1: **R-23**.
The register's §2 line for it reads `G8 (per-sink rendering) / G4`, and this
ranking takes the first. R-23 is tier 2; every other G8 member is tier 4. A
cluster's tier is its worst member's, so R-23 alone is what makes G8 a tier-2
cluster — and it is G8's tier-2-with-65 combination that dominates G3's
tier-2-with-45. Move R-23 to G4, as the register's own second reading allows,
and G8 becomes tier 4, stops dominating, and **G3 enters band 1**.

So the frontier's shape rests on one entry whose reachable count is zero and
whose cluster the register states two ways. That is worth knowing before anyone
cites band 1 as settled. It is also an argument for tracing G4 and G8 rather
than inferring them: §3 of the register labels both as inference, and this is
what an inferred cluster costs downstream.

The same subsection shows a second, larger swing: moving **R-21** into G8 makes
G8 uncountable and removes it from the numeric frontier entirely. R-21 has no
predicate, and an uncountable member poisons its cluster's sum by design
(`aggregate`'s doc comment says why: a partially-counted sum is smaller than the
truth while looking complete).

### 6.4 R-13 was the nominee, and its number is the one that means least

§0.1 named R-13 among the likely frontier. It has the second-largest reachable
count that survives the SILENT filter, and §3.2 shows what that count is made
of: of 45 reachable sites, **2** have the object-literal receiver the register's
repro describes, both in the single accepted extension program, so **all 43
reachable anchor sites have none**; 18 of the 45 are store targets rather than
reads. The predicate's own record does not disclose that it is an upper bound —
that disclosure is this measurement's, not the register's.

This is the clearest instance of a general hazard: a count is only as sharp as
the predicate, and a broad predicate on a common construct produces a large,
confident, wrong-shaped number. R-13's count is real; what it counts is a
construct family, most of which works.

### 6.5 What this ranking does not license

- **It does not retire anything.** §3.4 lists the eleven ranked entries with a
  non-SILENT lane. A lane is not an entry: R-47's and R-53's FIXED lanes are the
  `const` controls those entries declare for themselves, R-30's two FIXED lanes
  are its `const`-scalar lane *and* its concat/template sinks, R-08's `===` half
  fails closed while its `??` half is **still SILENT**, and R-49 — outside the
  ranking entirely — fails closed by *R-35's* switch allowlist rather than by any
  gate of its own.
- **It does not license reading a cluster sum as a fix estimate.** G3 in
  particular: the register's §3 says in terms that G3's members are *not one code
  path* — it is a shape of mistake with six independent instances. **Both of G3's
  numbers are sums over a pattern**: the 45 that puts it in reachable band 2 and,
  more dangerously, the **305** that leads the raw axis's band 1, of which 302 is
  R-13's construct-family count that §3.2 takes apart. Neither is an estimate of
  what one allowlist would close. G4, G5, G6, G7 and G8 are labelled inference
  too; only G1 (which contributes nothing here, having no SILENT member) and part
  of G7 are traced in source.
- **It does not turn a zero into "rare".** Twenty entries are
  `present-but-unreachable`. Their construct occurs; the carrying program was
  rejected for something else. Fixing an unrelated defect can move several of
  them into the reachable column at once without anything about them changing.

### 6.6 What would make the next version of this better, in order

1. **Raise the extension accept rate by fixing kali** — not by re-curating the
   corpus, which §4.3 forbids and which would make every future score circular.
   At 2.5%, 39 of 40 job-shaped programs contribute to the raw axis only. This
   is the single change that would most improve the measurement, and it is the
   same change the register exists to prompt.
2. **Trace G4, G7 and G8.** Three of the four judgment calls that move a band
   are memberships in inferred clusters. §3 of the register already names the
   experiments that would settle each.
3. **Qualify R-13's and R-14's predicates** so their counts describe the
   register's repro rather than its construct family, and disclose the bound in
   the record where the record is what a reader reaches for first.
4. **Re-run, do not re-read.** Every figure above is regenerable by one command
   against a hashed corpus. The failure this project was built to end was a table
   of numbers nobody could re-derive; the counter-measure is only worth anything
   if the command is actually run again the next time someone cites this file.
